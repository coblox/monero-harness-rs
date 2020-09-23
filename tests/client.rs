use monero_harness_rs::{Client, Container};
use spectral::prelude::*;
use testcontainers::clients::Cli;

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
        .get_balance_alice()
        .await
        .expect("failed to get alice's balance");

    let got_bob_balance = cli
        .get_balance_bob()
        .await
        .expect("failed to get bob's balance");

    assert_that!(got_alice_balance).is_equal_to(want_alice_balance);
    assert_that!(got_bob_balance).is_equal_to(want_bob_balance);
}
