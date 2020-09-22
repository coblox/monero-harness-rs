use crate::{Request, Response};

use anyhow::{anyhow, Result};
use reqwest::Url;
use serde::{Deserialize, Serialize};

// #[cfg(not(test))]
// use tracing::debug;

// #[cfg(test)]
use std::println as debug;

// TODO: Consider using bignum for moneroj instead of u64?

const INITIAL_FUNDS_ALICE: u64 = 1000;

const ACCOUNT_INDEX_PRIMARY: u32 = 0;
const ACCOUNT_INDEX_ALICE: u32 = 1;
const ACCOUNT_INDEX_BOB: u32 = 2;

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

    /// Create accounts Alice and Bob, do initial funding from the primary
    /// account.
    pub async fn init_accounts(&self) -> Result<()> {
        let balance = self.get_balance_primary().await?;
        if balance < INITIAL_FUNDS_ALICE {
            return Err(anyhow!(
                "insufficient funds {}, consider initialising with more blocks",
                balance,
            ));
        }

        let alice = self.create_account("alice").await?;
        let bob = self.create_account("bob").await?;

        debug_assert!(alice.account_index == ACCOUNT_INDEX_ALICE);
        debug_assert!(bob.account_index == ACCOUNT_INDEX_BOB);

        self.transfer_from_primary(INITIAL_FUNDS_ALICE, alice.address)
            .await?;

        let balance = self.get_balance_alice().await?;
        debug_assert!(balance == INITIAL_FUNDS_ALICE);

        Ok(())
    }

    /// Get addresses for the primary account.
    pub async fn get_address_primary(&self) -> Result<GetAddressResponse> {
        self.get_address(ACCOUNT_INDEX_PRIMARY).await
    }

    /// Get addresses for the Alice's account.
    pub async fn get_address_alice(&self) -> Result<GetAddressResponse> {
        self.get_address(ACCOUNT_INDEX_ALICE).await
    }

    /// Get addresses for the Bob's account.
    pub async fn get_address_bob(&self) -> Result<GetAddressResponse> {
        self.get_address(ACCOUNT_INDEX_BOB).await
    }

    /// Get addresses for account by index.
    async fn get_address(&self, account_index: u32) -> Result<GetAddressResponse> {
        let params = GetAddressParams { account_index };
        let request = Request::new("get_address", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("get address RPC response: {}", response);

        let r: Response<GetAddressResponse> = serde_json::from_str(&response)?;
        Ok(r.result)
    }

    /// Gets the balance of the wallet primary account.
    pub async fn get_balance_primary(&self) -> Result<u64> {
        self.get_balance(ACCOUNT_INDEX_PRIMARY).await
    }

    /// Gets the balance of Alice's account.
    pub async fn get_balance_alice(&self) -> Result<u64> {
        self.get_balance(ACCOUNT_INDEX_ALICE).await
    }

    /// Gets the balance of Bob's account.
    pub async fn get_balance_bob(&self) -> Result<u64> {
        self.get_balance(ACCOUNT_INDEX_BOB).await
    }

    /// Gets the balance of account by index.
    async fn get_balance(&self, index: u32) -> Result<u64> {
        let params = GetBalanceParams {
            account_index: index,
        };
        let request = Request::new("get_balance", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!(
            "get balance of account index {} RPC response: {}",
            index, response
        );

        let res: Response<GetBalance> = serde_json::from_str(&response)?;

        let balance = res.result.balance;

        Ok(balance)
    }

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

        debug!("create account RPC response: {}", response);

        let r: Response<CreateAccount> = serde_json::from_str(&response)?;
        Ok(r.result)
    }

    /// Get accounts, filtered by tag ("" for no filtering).
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

        debug!("get accounts RPC response: {}", response);

        let r: Response<GetAccounts> = serde_json::from_str(&response)?;

        Ok(r.result)
    }

    /// Creates a wallet using `filename`.
    pub async fn create_wallet(&self, filename: &str) -> Result<()> {
        let params = CreateWalletParams {
            filename: filename.to_owned(),
            language: "English".to_owned(),
        };
        let request = Request::new("create_wallet", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("create wallet RPC response: {}", response);

        Ok(())
    }

    /// Transfers moneroj from the primary account.
    pub async fn transfer_from_primary(&self, amount: u64, address: String) -> Result<Transfer> {
        let dest = vec![Destination {
            account_index: ACCOUNT_INDEX_PRIMARY,
            amount,
            address,
        }];
        self.multi_transfer(dest).await
    }

    /// Transfers moneroj from Alice's account.
    pub async fn transfer_from_alice(&self, amount: u64, address: String) -> Result<Transfer> {
        let dest = vec![Destination {
            account_index: ACCOUNT_INDEX_ALICE,
            amount,
            address,
        }];
        self.multi_transfer(dest).await
    }

    /// Transfers moneroj from Bob's account.
    pub async fn transfer_from_bob(&self, amount: u64, address: String) -> Result<Transfer> {
        let dest = vec![Destination {
            account_index: ACCOUNT_INDEX_BOB,
            amount,
            address,
        }];
        self.multi_transfer(dest).await
    }

    /// Transfers moneroj to multiple destinations.
    async fn multi_transfer(&self, destinations: Vec<Destination>) -> Result<Transfer> {
        let params = TransferParams { destinations };
        let request = Request::new("transfer", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        debug!("transfer RPC response: {}", response);

        let r: Response<Transfer> = serde_json::from_str(&response)?;
        Ok(r.result)
    }
}

#[derive(Serialize, Debug, Clone)]
struct GetAddressParams {
    account_index: u32,
}
#[derive(Deserialize, Debug, Clone)]
pub struct GetAddressResponse {
    pub address: String,
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
    pub account_index: u32,
    pub address: String,
}

#[derive(Serialize, Debug, Clone)]
struct TagParams {
    tag: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetAccounts {
    pub subaddress_accounts: Vec<SubAddressAccount>,
    pub total_balance: u64,
    pub total_unlocked_balance: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SubAddressAccount {
    pub account_index: u32,
    pub balance: u32,
    pub base_address: String,
    pub label: String,
    pub tag: String,
    pub unlocked_balance: u64,
}

#[derive(Serialize, Debug, Clone)]
struct CreateWalletParams {
    filename: String,
    language: String,
}

#[derive(Serialize, Debug, Clone)]
struct TransferParams {
    destinations: Vec<Destination>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Destination {
    account_index: u32,
    amount: u64,
    address: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Transfer {
    amount: u64,
    fee: u64,
    multisig_txset: String,
    tx_blob: String,
    tx_hash: String,
    tx_key: String,
    tx_metadata: String,
    unsigned_txset: String,
}
