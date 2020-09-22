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

const MONERO_WALLET_RPC_PORT: u16 = 28083; // Next available port after monerod's ports.
const MONEROD_RPC_PORT: u16 = 28081; // Default testnet port.

/// RPC client for monerod and monero-wallet-rpc.
#[derive(Debug)]
pub struct Client {
    pub wallet: wallet::Client,
    pub monerod: monerod::Client,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            wallet: wallet::Client::localhost(MONERO_WALLET_RPC_PORT)
                .expect("failed to create wallet client"),
            monerod: monerod::Client::localhost(MONEROD_RPC_PORT)
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
    use testcontainers::{self, clients, images, Docker, Image};

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

    #[test]
    fn can_create_a_client() {
        let _ = Client::default();
    }

    #[test]
    fn start_monerod_container() {
        let docker = clients::Cli::default();

        let image = images::generic::GenericImage::new("xmrto/monero")
            .with_entrypoint("")
            .with_args(vec![
                "/bin/bash  -c \" ".to_string(),
                "monerod".to_string(),
                "--confirm-external-bind".to_string(),
                "--non-interactive".to_string(),
                "--regtest".to_string(),
                "--rpc-bind-ip 0.0.0.0".to_string(),
                "--rpc-bind-port 28081".to_string(),
                "--no-igd".to_string(),
                "--hide-my-port".to_string(),
                "--fixed-difficulty 1".to_string(),
                "--rpc-login username:password".to_string(),
                "--data-dir /monero --detach".to_string(),
                "\" ".to_string(),
            ]);
        docker.run(image);
        // "&&",
        // "monero-wallet-rpc" ,
        // "--log-level 4" \
        // --daemon-address localhost:28081 \
        // --confirm-external-bind \
        // --rpc-login username:password \
        // --rpc-bind-ip 0.0.0.0 \
        // --rpc-bind-port 28083 \
        // --daemon-login username:password \
        // --wallet-dir /monero/""
    }
}
