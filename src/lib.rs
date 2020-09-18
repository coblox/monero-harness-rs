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
