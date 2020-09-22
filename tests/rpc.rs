use monero_harness_rs::Client;
use spectral::prelude::*;
use testcontainers::clients;

#[tokio::test]
async fn connect_to_monerod() {
    let docker = clients::Cli::default();
    let cli = Client::new_with_random_local_ports(&docker);

    let header = cli
        .monerod
        .get_block_header_by_height(0)
        .await
        .expect("failed to get block 0");

    assert_eq!(0, header.height);
}

#[tokio::test]
async fn wallet_and_accounts() {
    let docker = clients::Cli::default();
    let cli = Client::new_with_random_local_ports(&docker);

    let _ = cli
        .wallet
        .create_wallet("wallet")
        .await
        .expect("failed to create wallet");

    let got = cli
        .wallet
        .get_balance()
        .await
        .expect("failed to get balance");
    let want = 0;

    assert_that!(got).is_equal_to(want);
}

#[tokio::test]
async fn create_account_and_retrieve_it() {
    let docker = clients::Cli::default();
    let cli = Client::new_with_random_local_ports(&docker);

    let label = "alice";

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
        .get_accounts("")
        .await
        .expect("failed to get accounts");
    for account in accounts.subaddress_accounts {
        if account.label == label {
            found = true;
        }
    }
    assert!(found);
}
