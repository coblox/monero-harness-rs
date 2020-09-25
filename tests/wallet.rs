use monero_harness::{rpc::wallet::Client, Monero};
use spectral::prelude::*;
use testcontainers::clients::Cli;

#[tokio::test]
async fn wallet_and_accounts() {
    let tc = Cli::default();
    let monero = Monero::new(&tc);
    let cli = Client::localhost(monero.wallet_rpc_port);

    println!("creating wallet ...");

    let _ = cli
        .create_wallet("wallet")
        .await
        .expect("failed to create wallet");

    let got = cli.get_balance(0).await.expect("failed to get balance");
    let want = 0;

    assert_that!(got).is_equal_to(want);
}

// #[tokio::test]
// async fn create_account_and_retrieve_it() {
//     let docker = Cli::default();
//     let container = Container::new(&docker);
//     let cli = Client::new(container.monerod_rpc_port,
// container.wallet_rpc_port);

//     let label = "Arbitrary Label"; // This is intentionally _not_ Alice or
// Bob.

//     let _ = cli
//         .wallet
//         .create_wallet("wallet")
//         .await
//         .expect("failed to create wallet");

//     let _ = cli
//         .wallet
//         .create_account(label)
//         .await
//         .expect("failed to create account");

//     let mut found: bool = false;
//     let accounts = cli
//         .wallet
//         .get_accounts("") // Empty filter.
//         .await
//         .expect("failed to get accounts");
//     for account in accounts.subaddress_accounts {
//         if account.label == label {
//             found = true;
//         }
//     }
//     assert!(found);
// }

// #[tokio::test]
// async fn transfer_and_check_tx_key() {
//     let docker = Cli::default();
//     let container = Container::new(&docker);
//     let cli = Client::new(container.monerod_rpc_port,
// container.wallet_rpc_port);

//     let balance_alice = 1_000_000_000_000;
//     let balance_bob = 0;

//     cli.init(balance_alice, balance_bob)
//         .await
//         .expect("failed to init");

//     let address_bob = cli
//         .get_address_bob()
//         .await
//         .expect("failed to get Bob's address")
//         .address;

//     let transfer_amount = 100;
//     let transfer = cli
//         .transfer_from_alice(transfer_amount, &address_bob)
//         .await
//         .expect("transfer failed");

//     let tx_id = transfer.tx_hash;
//     let tx_key = transfer.tx_key;

//     let res = cli
//         .wallet
//         .check_tx_key(&tx_id, &tx_key, &address_bob)
//         .await
//         .expect("failed to check tx by key");

//     assert_that!(res.received).is_equal_to(transfer_amount);
// }
