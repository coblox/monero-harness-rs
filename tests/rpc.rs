use monero_harness_rs::{Client, Container};
use spectral::prelude::*;
use std::time::Duration;
use testcontainers::clients::Cli;
use tokio::time;

#[tokio::test]
async fn connect_to_monerod() {
    let docker = Cli::default();
    let container = Container::new(&docker);
    let cli = Client::new(container.monerod_rpc_port, container.wallet_rpc_port);

    let header = cli
        .monerod
        .get_block_header_by_height(0)
        .await
        .expect("failed to get block 0");

    assert_eq!(0, header.height);
}

#[tokio::test]
async fn miner_is_running_and_producing_blocks() {
    let docker = Cli::default();
    let container = Container::new(&docker);
    let cli = Client::new(container.monerod_rpc_port, container.wallet_rpc_port);
    cli.init_just_miner(2).await.expect("Failed to initialize");

    // Only need 3 seconds since we mine a block every second but
    // give it 5 just for good measure.
    time::delay_for(Duration::from_secs(5)).await;

    // We should have at least 5 blocks by now.
    let block_header = cli
        .monerod
        .get_block_header_by_height(5)
        .await
        .expect("failed to get block");

    assert_eq!(5, block_header.height);
}

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
        .get_balance_primary()
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

#[tokio::test]
async fn init_accounts_for_alice_and_bob() {
    let docker = Cli::default();
    let container = Container::new(&docker);
    let cli = Client::new(container.monerod_rpc_port, container.wallet_rpc_port);

    let want_alice_balance = 1000;
    let want_bob_balance = 0;

    cli.init(want_alice_balance, want_bob_balance)
        .await
        .expect("failed to init");

    let got_alice_balance = cli
        .wallet
        .get_balance_alice()
        .await
        .expect("failed to get alice's balance");

    let got_bob_balance = cli
        .wallet
        .get_balance_bob()
        .await
        .expect("failed to get bob's balance");

    assert_that!(got_alice_balance).is_equal_to(want_alice_balance);
    assert_that!(got_bob_balance).is_equal_to(want_bob_balance);
}
