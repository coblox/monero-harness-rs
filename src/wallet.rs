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
    // curl http://localhost:18082/json_rpc -d '{"jsonrpc":"2.0","id":"0","method":"create_account","params":{"label":"Secondary account"}}' -H 'Content-Type: application/json'
    pub async fn create_account(&self, label: &str) -> Result<CreateAccount> {
        let params = LabelParams {
            label: label.to_owned(),
        };
        let request = Request::new("create_account", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        let r: Response<CreateAccount> = serde_json::from_str(&response)?;
        Ok(r.result)
    }

    // $ curl http://localhost:18082/json_rpc -d '{"jsonrpc":"2.0","id":"0","method":"get_accounts","params":{"tag":"myTag"}}' -H 'Content-Type: application/json'

    // TODO: Make tag optional.
    /// Get accounts, filtered by tag.
    pub async fn get_accounts(&self, tag: &str) -> Result<GetAccounts> {
        let params = TagParams {
            tag: tag.to_owned(),
        };
        let request = Request::new("get_accounts", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        let r: Response<GetAccounts> = serde_json::from_str(&response)?;
        Ok(r.result)
    }

    // $ curl http://localhost:18082/json_rpc -d '{"jsonrpc":"2.0","id":"0","method":"create_wallet","params":{"filename":"mytestwallet","password":"mytestpassword","language":"English"}}' -H 'Content-Type: application/json'
    // {
    // "id": "0",
    // "jsonrpc": "2.0",
    // "result": {
    // }
    // }
    // You need to have set the argument "–wallet-dir" when launching
    // monero-wallet-rpc to make this work.
    pub async fn create_wallet(&self, filename: &str) -> Result<()> {
        let params = CreateWalletParams {
            filename: filename.to_owned(),
            language: "English".to_owned(),
        };
        let request = Request::new("create_wallet", params);

        let _ = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        Ok(())
    }
}

#[derive(Serialize, Debug, Clone)]
struct GetBalanceParams {
    account_index: u32,
}

#[derive(Deserialize, Debug, Clone)]
struct GetBalance {
    balance: u64,
    blocks_to_unlock: u32,
    multisig_import_needed: bool,
    time_to_unlock: u32,
    unlocked_balance: u64,
}

#[derive(Serialize, Debug, Clone)]
struct LabelParams {
    label: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateAccount {
    account_index: u32,
    address: String,
}

#[derive(Serialize, Debug, Clone)]
struct TagParams {
    tag: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetAccounts {
    subaddress_accounts: Vec<SubAddressAccount>,
    total_balance: u64,
    total_unlocked_balance: u64,
}

#[derive(Deserialize, Debug, Clone)]
struct SubAddressAccount {
    account_index: u32,
    balance: u32,
    base_address: String,
    label: String,
    tag: String,
    unlocked_balance: u64,
}

#[derive(Serialize, Debug, Clone)]
struct CreateWalletParams {
    filename: String,
    language: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;
    use std::fs;

    // These tests make state changes (to the filesystem and to the wallet), it is
    // not suggested to just run them all but rather run them individually while
    // watching your monero node setup.

    fn cli() -> Client {
        // TODO: Make this test executable on CI.
        Client::localhost(2021).unwrap()
    }

    #[tokio::test]
    async fn get_balance() {
        let cli = cli();
        let got = cli.get_balance().await.expect("failed to get balance");
        let want = 0;

        assert_that!(got).is_equal_to(want);
    }

    #[tokio::test]
    async fn create_account() {
        let cli = cli();
        let label = "alice";

        let accounts = cli.get_accounts("").await.expect("failed to get accounts");

        let mut found: bool = false;
        for account in accounts.subaddress_accounts {
            if account.label == label {
                found = true;
            }
        }

        if !found {
            let _ = cli
                .create_account(label)
                .await
                .expect("failed to create account");
        }

        assert!(found);
    }

    #[tokio::test]
    async fn create_wallet() {
        let cli = cli();
        let filename = "twallet";

        let _ = cli
            .create_wallet(filename)
            .await
            .expect("failed to create balance");
    }
}
