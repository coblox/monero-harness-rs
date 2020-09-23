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

pub mod monerod;
pub mod wallet;

use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use testcontainers::{
    clients::Cli,
    core::Port,
    images::generic::{GenericImage, Stream, WaitFor},
    Docker, Image,
};

/// How often we mine a block.
const BLOCK_TIME_SECS: u64 = 1;

/// Poll interval when checking if the wallet has synced with monerod.
const WAIT_WALLET_SYNC_MILLIS: u64 = 1000;

/// A docker container with a `monerod` instance and a `monero-wallet-rpc`
/// instance.
#[derive(Debug)]
pub struct Container<'c> {
    inner: testcontainers::Container<'c, Cli, GenericImage>,
    pub monerod_rpc_port: u16,
    pub wallet_rpc_port: u16,
}

impl<'c> Container<'c> {
    /// Creates a new monero container mapping random local ports for the two
    /// RPC endpoints.
    pub fn new(cli: &'c Cli) -> Self {
        let mut rng = rand::thread_rng();
        let monerod_rpc_port: u16 = rng.gen_range(1024, u16::MAX);
        let wallet_rpc_port: u16 = rng.gen_range(1024, u16::MAX);

        let image = GenericImage::new("xmrto/monero")
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

        let container = cli.run(image);

        Self {
            inner: container,
            monerod_rpc_port,
            wallet_rpc_port,
        }
    }
}

/// RPC client for `monerod` and `monero-wallet-rpc`. This struct implements
/// methods mostly useful for testing, non-test users probably want to manage
/// the wallet and monerod instance separately.
#[derive(Debug)]
pub struct Client {
    pub wallet: wallet::Client,
    pub monerod: monerod::Client,
}

impl Client {
    /// Create a new Client instance that connects to respective ports.
    pub fn new(monerod_rpc_port: u16, wallet_rpc_port: u16) -> Self {
        Self {
            wallet: wallet::Client::localhost(wallet_rpc_port),
            monerod: monerod::Client::localhost(monerod_rpc_port),
        }
    }

    /// Initialise by creating a wallet, generating some `blocks`, and starting
    /// a miner thread that mines to the primary account. Also create two
    /// sub-accounts, one for Alice and one for Bob. If alice/bob_funding is
    /// some, the value needs to be > 0.
    pub async fn init(&self, alice_funding: u64, bob_funding: u64) -> Result<()> {
        self.wallet.create_wallet("miner_wallet").await?;

        let alice = self.wallet.create_account("alice").await?;
        let bob = self.wallet.create_account("bob").await?;

        let miner = self.wallet.get_address_primary().await?.address;

        let res = self.monerod.generate_blocks(70, &miner).await?;
        self.wait_for_wallet_block_height(res.height).await?;

        if alice_funding > 0 {
            self.fund_account(&alice.address, &miner, alice_funding)
                .await?;
            let balance = self.wallet.get_balance_alice().await?;
            debug_assert!(balance == alice_funding);
        }

        if bob_funding > 0 {
            self.fund_account(&bob.address, &miner, bob_funding).await?;
            let balance = self.wallet.get_balance_bob().await?;
            debug_assert!(balance == bob_funding);
        }

        let _ = tokio::spawn(mine(self.monerod.clone(), miner));

        Ok(())
    }

    /// Just create a wallet and start mining (you probably want `init()`).
    pub async fn init_just_miner(&self, blocks: u32) -> Result<()> {
        self.wallet.create_wallet("miner_wallet").await?;
        let miner = self.wallet.get_address_primary().await?.address;

        let _ = self.monerod.generate_blocks(blocks, &miner).await?;

        let _ = tokio::spawn(mine(self.monerod.clone(), miner));

        Ok(())
    }

    async fn fund_account(&self, address: &str, miner: &str, funding: u64) -> Result<()> {
        self.wallet.transfer_from_primary(funding, address).await?;
        let res = self.monerod.generate_blocks(10, miner).await?;
        self.wait_for_wallet_block_height(res.height).await?;
        Ok(())
    }

    // It takes a little while for the wallet to sync with monerod.
    async fn wait_for_wallet_block_height(&self, height: u32) -> Result<()> {
        while self.wallet.block_height().await?.height < height {
            tokio::time::delay_for(Duration::from_millis(WAIT_WALLET_SYNC_MILLIS)).await;
        }
        Ok(())
    }
}

/// Mine a block ever BLOCK_TIME_SECS seconds.
async fn mine(monerod: monerod::Client, reward_address: String) -> Result<()> {
    loop {
        tokio::time::delay_for(Duration::from_secs(BLOCK_TIME_SECS)).await;
        monerod.generate_blocks(1, &reward_address).await?;
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
