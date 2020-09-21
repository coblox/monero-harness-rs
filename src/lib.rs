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

mod wallet;

use serde::Serialize;

const MONERO_WALLET_RPC_PORT: u16 = 2021; // Arbitrarily chosen.

/// RPC client for monerod and monero-wallet-rpc.
#[derive(Debug)]
pub struct Client {
    pub wallet: wallet::Client,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            wallet: wallet::Client::localhost(MONERO_WALLET_RPC_PORT)
                .expect("failed to create wallet client"),
        }
    }
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
