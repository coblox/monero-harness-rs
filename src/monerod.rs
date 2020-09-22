use crate::{BlockHeader, Request, Response};

use anyhow::Result;
use reqwest::Url;
use serde::{Deserialize, Serialize};

/// RPC client for monerod and monero-wallet-rpc.
#[derive(Debug)]
pub struct Client {
    pub inner: reqwest::Client,
    pub url: Url,
}

impl Client {
    /// New local host monerod RPC client.
    pub fn localhost(port: u16) -> Result<Self> {
        let url = format!("http://127.0.0.1:{}/json_rpc", port);
        let url = Url::parse(&url)?;

        Ok(Self {
            inner: reqwest::Client::new(),
            url,
        })
    }

    // $ curl http://127.0.0.1:18081/json_rpc -d '{"jsonrpc":"2.0","id":"0","method":"get_block_header_by_height","params":{"height":1}}' -H 'Content-Type: application/json'
    pub async fn get_block_header_by_height(&self, height: u32) -> Result<BlockHeader> {
        let params = GetBlockHeaderByHeightParams { height };
        let request = Request::new("get_block_header_by_height", params);

        let response = self
            .inner
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        let res: Response<GetBlockHeaderByHeight> = serde_json::from_str(&response)?;

        Ok(res.result.block_header)
    }
}

#[derive(Clone, Debug, Serialize)]
struct GetBlockHeaderByHeightParams {
    height: u32,
}

#[derive(Clone, Debug, Deserialize)]
struct GetBlockHeaderByHeight {
    block_header: BlockHeader,
    status: String,
    untrusted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use testcontainers::{clients, core::Port, images, Docker, Image};

    #[tokio::test]
    async fn can_get_genesis_block_header() {
        let docker = clients::Cli::default();

        // let monerod = "monerod --confirm-external-bind --non-interactive --regtest
        // --rpc-bind-ip 0.0.0.0 --rpc-bind-port 28081 --no-igd --hide-my-port
        // --fixed-difficulty 1 --rpc-payment-allow-free-loopback --data-dir /monero
        // --detach";

        // let wallet = r#"monero-wallet-rpc \
        // 	--log-level 4 \
        // 	--daemon-address localhost:28081 \
        // 	--confirm-external-bind \
        // 	--disable-rpc-login \
        // 	--rpc-bind-ip 0.0.0.0 \
        // 	--rpc-bind-port 28083 \
        // 	--daemon-login username:password \
        // 	--wallet-dir /monero/"#;

        //        let shell = format!("{} && {}", monerod, wallet);

        let image = images::generic::GenericImage::new("xmrto/monero")
            .with_mapped_port(Port {
                local: 28081,
                internal: 28081,
            })
            .with_entrypoint("")
            .with_args(vec![
                // "/bin/bash".to_string(),
                // "-c".to_string(),
                "monerod".to_string(),
                /*                "monerod --confirm-external-bind --non-interactive --regtest
                 * --rpc-bind-ip 0.0.0.0 --rpc-bind-port 28081 --no-igd --hide-my-port
                 * --fixed-difficulty 1 --rpc-payment-allow-free-loopback --data-dir
                 * /monero".to_string(), */
            ]);
        docker.run(image);

        tokio::time::delay_for(Duration::from_secs(30)).await;

        let cli = Client::localhost(28081).unwrap();
        let _ = cli
            .get_block_header_by_height(0)
            .await
            .expect("failed to get block 0");
    }
}
