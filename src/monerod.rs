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
    pub block_header: BlockHeader,
    pub status: String,
    pub untrusted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn can_get_genesis_block_header() {
        // TODO: Make this test executable on CI.
        let cli = Client::localhost(38081).unwrap();
        let _ = cli
            .get_block_header_by_height(0)
            .await
            .expect("failed to get block 0");
    }
}
