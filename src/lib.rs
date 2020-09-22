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

use serde::{Deserialize, Serialize};
use testcontainers::core::Port;
use testcontainers::images::generic::{GenericImage, Stream, WaitFor};
use testcontainers::{clients, images, Container, Docker, Image};

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
                monero-wallet-rpc --log-level 4 --daemon-address localhost:28081 --confirm-external-bind --rpc-login username:password --rpc-bind-ip 0.0.0.0 --rpc-bind-port 28083 --daemon-login username:password --wallet-dir /monero/".to_string(),
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
}

// We should be able to use monero-rs for this but it does not include all
// the fields.
#[derive(Clone, Debug, Deserialize)]
pub struct BlockHeader {
    block_size: u32,
    depth: u32,
    difficulty: u32,
    hash: String,
    height: u32,
    major_version: u32,
    minor_version: u32,
    nonce: u32,
    num_txes: u32,
    orphan_status: bool,
    prev_hash: String,
    reward: u64,
    timestamp: u32,
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
    use testcontainers::{self, clients};

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

    #[tokio::test]
    async fn can_get_genesis_block_header() {
        let docker_client = clients::Cli::default();
        let client = Client::new(&docker_client, 28081, 28083);

        let block_header = client
            .monerod
            .get_block_header_by_height(0)
            .await
            .expect("failed to get block 0");

        assert_eq!(0, block_header.height);
    }
}
