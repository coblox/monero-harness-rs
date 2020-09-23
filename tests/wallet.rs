use monero_harness_rs::{Client, Container};
use spectral::prelude::*;
use testcontainers::clients::Cli;

#[tokio::test]
async fn wallet_and_accounts() {
    let docker = Cli::default();
    let container = Container::new(&docker);
    let cli = Client::new(container.monerod_rpc_port, container.wallet_rpc_port);

    let _ = cli
        .wallet
        .create_wallet("wallet")
        .await
        .expect("failed to create wallet");

    let got = cli
        .wallet
        .get_balance(0)
        .await
        .expect("failed to get balance");
    let want = 0;

    assert_that!(got).is_equal_to(want);
}

#[tokio::test]
async fn create_account_and_retrieve_it() {
    let docker = Cli::default();
    let container = Container::new(&docker);
    let cli = Client::new(container.monerod_rpc_port, container.wallet_rpc_port);

    let label = "Arbitrary Label"; // This is intentionally _not_ Alice or Bob.

    let _ = cli
        .wallet
        .create_wallet("wallet")
        .await
        .expect("failed to create wallet");

    let _ = cli
        .wallet
        .create_account(label)
        .await
        .expect("failed to create account");

    let mut found: bool = false;
    let accounts = cli
        .wallet
        .get_accounts("") // Empty filter.
        .await
        .expect("failed to get accounts");
    for account in accounts.subaddress_accounts {
        if account.label == label {
            found = true;
        }
    }
    assert!(found);
}
