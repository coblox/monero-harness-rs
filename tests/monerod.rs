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

    assert_that!(header.height).is_equal_to(0);
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
    let header = cli
        .monerod
        .get_block_header_by_height(5)
        .await
        .expect("failed to get block");

    assert_that!(header.height).is_equal_to(5);
}
