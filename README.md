Monero Harness
==============

Provides a JSON RPC client for monerod and monero-wallet-rpc

Example Usage
-------------
```rust
let cli = Client::default(); <!-- local host RPC client -->

let header = cli.monerod.get_block_header_by_height(HEIGHT).await?;
let balance = cli.wallet.get_balance().await?;

```

