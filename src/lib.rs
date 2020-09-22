#![warn(
    unused_extern_crates,
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::fallible_impl_from,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::dbg_macro
)]
#![forbid(unsafe_code)]

//! Provides a JSON RPC client for monerod and monero-wallet-rpc

mod monerod;
mod wallet;

use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use testcontainers::{
    clients,
    core::Port,
    images,
    images::generic::{GenericImage, Stream, WaitFor},
    Container, Docker, Image,
};

// #[cfg(not(test))]
// use tracing::debug;

// #[cfg(test)]
use std::println as debug;

/// RPC client for monerod and monero-wallet-rpc.
#[derive(Debug)]
pub struct Client<'c> {
    container: Container<'c, clients::Cli, GenericImage>,
    pub wallet: wallet::Client,
    pub monerod: monerod::Client,
}

impl<'c> Client<'c> {
    pub fn new(
        docker_client: &'c clients::Cli,
        monerod_rpc_port: u16,
        wallet_rpc_port: u16,
    ) -> Self {
        let image = images::generic::GenericImage::new("xmrto/monero")
            .with_mapped_port(Port {
                local: monerod_rpc_port,
                internal: 28081,
            })
            .with_mapped_port(Port {
                local: wallet_rpc_port,
                internal: 28083,
            })
            .with_entrypoint("")
            .with_args(vec![
                "/bin/bash".to_string(),
                "-c".to_string(),
                "monerod --confirm-external-bind --non-interactive --regtest --rpc-bind-ip 0.0.0.0 --rpc-bind-port 28081 --no-igd --hide-my-port --fixed-difficulty 1 --rpc-payment-allow-free-loopback --data-dir /monero & \
                monero-wallet-rpc --log-level 4 --daemon-address localhost:28081 --confirm-external-bind --disable-rpc-login --rpc-bind-ip 0.0.0.0 --rpc-bind-port 28083  --wallet-dir /monero/".to_string(),
            ])
            .with_wait_for(WaitFor::LogMessage { message: "You are now synchronized with the network. You may now start monero-wallet-cli".to_string(), stream: Stream::StdOut });
        let container = docker_client.run(image);
        Self {
            container,
            wallet: wallet::Client::localhost(wallet_rpc_port)
                .expect("failed to create wallet client"),
            monerod: monerod::Client::localhost(monerod_rpc_port)
                .expect("failed to create monerod client"),
        }
    }

    pub fn new_with_random_local_ports(cli: &'c clients::Cli) -> Self {
        let mut rng = rand::thread_rng();
        let monerod_port: u16 = rng.gen_range(1024, u16::MAX);
        let wallet_port: u16 = rng.gen_range(1024, u16::MAX);

        Client::new(cli, monerod_port, wallet_port)
    }

    /// Initialise by creating a wallet, generating some `blocks`, and starting
    /// a miner thread that mines to the primary account.
    pub async fn init(&self, blocks: u32) -> Result<()> {
        self.wallet.create_wallet("miner_wallet").await?;
        let miner_address = self.wallet.get_address_primary().await?.address;

        self.monerod
            .generate_blocks(blocks, miner_address.clone())
            .await?;

        // TODO: disabled for now
        // let _ = tokio::spawn(mine(self.monerod.clone(), miner_address.clone()));

        Ok(())
    }

    // TODO: This is separate from init() in order to debug, consider merging the
    // two methods onec its all functional and tested.
    pub async fn init_with_accounts(&self) -> Result<()> {
        // creating wallets first should speed wallet sync-time up
        self.wallet.create_wallet("miner_wallet").await?;
        let alice = self.wallet.create_account("alice").await?;
        let _bob = self.wallet.create_account("bob").await?;

        let miner_address = self.wallet.get_address_primary().await?.address;
        let generate_blocks_result = self
            .monerod
            .generate_blocks(70, miner_address.clone())
            .await?;

        // wait until wallet has caught up
        tokio::time::delay_for(Duration::from_secs(1)).await;
        while self.wallet.block_height().await?.height < generate_blocks_result.height {
            tokio::time::delay_for(Duration::from_secs(2)).await;
        }

        debug!("Initializing accounts");
        self.wallet
            .transfer_from_primary(1000, alice.address)
            .await?;

        let blocks_result = self
            .monerod
            .generate_blocks(2, miner_address.clone())
            .await?;

        tokio::time::delay_for(Duration::from_secs(1)).await;
        while self.wallet.block_height().await?.height < blocks_result.height {
            tokio::time::delay_for(Duration::from_secs(2)).await;
        }
        let balance_alice = self.wallet.get_balance_alice().await?;
        debug!("Current balance_alice after funding: {}", balance_alice);

        Ok(())
    }
}

async fn mine(monerod: monerod::Client, reward_address: String) -> Result<()> {
    loop {
        tokio::time::delay_for(Duration::from_secs(1)).await;
        monerod.generate_blocks(1, reward_address.clone()).await?;
    }
}

// We should be able to use monero-rs for this but it does not include all
// the fields.
#[derive(Clone, Debug, Deserialize)]
pub struct BlockHeader {
    pub block_size: u32,
    pub depth: u32,
    pub difficulty: u32,
    pub hash: String,
    pub height: u32,
    pub major_version: u32,
    pub minor_version: u32,
    pub nonce: u32,
    pub num_txes: u32,
    pub orphan_status: bool,
    pub prev_hash: String,
    pub reward: u64,
    pub timestamp: u32,
}

#[derive(Serialize, Debug, Clone)]
pub struct Request<T> {
    /// JSON RPC version, we hard cod this to 2.0.
    jsonrpc: String,
    /// Client controlled identifier, we hard code this to 1.
    id: String,
    /// The method to call.
    method: String,
    /// The method parameters.
    params: T,
}

/// JSON RPC request.
impl<T> Request<T> {
    pub fn new(method: &str, params: T) -> Self {
        Self {
            jsonrpc: "2.0".to_owned(),
            id: "1".to_owned(),
            method: method.to_owned(),
            params,
        }
    }
}

/// JSON RPC response.
#[derive(Deserialize, Serialize, Debug, Clone)]
struct Response<T> {
    pub id: String,
    pub jsonrpc: String,
    pub result: T,
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[derive(Serialize, Debug, Clone)]
    struct Params {
        val: u32,
    }

    #[test]
    fn can_serialize_request_with_params() {
        // Dummy method and parameters.
        let params = Params { val: 0 };
        let method = "get_block";

        let r = Request::new(method, &params);
        let got = serde_json::to_string(&r).expect("failed to serialize request");

        let want =
            "{\"jsonrpc\":\"2.0\",\"id\":\"1\",\"method\":\"get_block\",\"params\":{\"val\":0}}"
                .to_string();
        assert_that!(got).is_equal_to(want);
    }
}
