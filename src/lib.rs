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

//! # monero-harness
//! A simple lib to start a monero container. Does the following:
//!
//! - Run docker container with monerod and monero-wallet-cli
//! - Create a wallet.
//! - Start monerod, mine to the wallet primary address.
//! - Create two wallet sub accounts labelled Alice, and Bob.
//! - Send initial amount of moneroj to Alice's address from the primary
//!   account.

mod monerod;
mod wallet;

use serde::{Deserialize, Serialize};

const MONERO_WALLET_RPC_PORT: u16 = 2021; // Arbitrarily chosen.
const MONEROD_RPC_PORT: u16 = 38081; // Default stagenet port.

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
}
