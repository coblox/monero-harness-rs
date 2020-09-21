use crate::{Request, Response};

use anyhow::Result;
use reqwest::Url;
use serde::{Deserialize, Serialize};

/// JSON RPC client for monero-wallet-rpc.
#[derive(Debug)]
pub struct Client {
    pub inner: reqwest::Client,
    pub url: Url,
}

impl Client {
    /// New local host monero-wallet-rpc client.
    pub fn localhost(port: u16) -> Result<Self> {
        let url = format!("http://127.0.0.1:{}/json_rpc", port);
        let url = Url::parse(&url)?;

        Ok(Self {
            inner: reqwest::Client::new(),
            url,
        })
    }

    // curl http://127.0.0.1:2021/json_rpc -d '{"jsonrpc":"2.0","id":"0","method":"get_balance","params":{"account_index":0}}' -H 'Content-Type: application/json'
    pub async fn get_balance(&self) -> Result<u64> {
        let params = GetBalanceParams { account_index: 0 };
        let request = Request::new("get_balance", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        let res: Response<GetBalance> = serde_json::from_str(&response)?;
        let balance = res.result.balance;

        Ok(balance)
    }
}

#[derive(Serialize, Debug, Clone)]
struct GetBalanceParams {
    account_index: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct GetBalance {
    balance: u64,
    blocks_to_unlock: u32,
    multisig_import_needed: bool,
    time_to_unlock: u32,
    unlocked_balance: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[tokio::test]
    async fn can_get_balance() {
        // TODO: Make this test executable on CI.
        let cli = Client::localhost(2021).unwrap();
        let got = cli.get_balance().await.expect("failed to get balance");
        let want = 0;

        assert_that!(got).is_equal_to(want);
    }
}
