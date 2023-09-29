use serde::Deserialize;

use crate::protocol::{Protocol, ValidatorData};

pub struct NearProtocol {
    /// The RPC endpoint to query. See [`Self::new`] for more info.
    rpc_url: String,
    /// The block height for which validator is queried. See [`Self::new`] for more info.
    block: Option<u64>,
}

impl NearProtocol {
    /// Constructs an instance to download validator data from `rpc_url` for `block_height`.
    ///
    /// Near [RPC docs] list several providers.
    ///
    /// If `block` is `None` data for the latest block will be downloaded. In case a `block_height`
    /// is provided, it must refer to the last block of an epoch as described in the [RPC method
    /// docs].
    ///
    /// [RPC docs]: https://docs.near.org/api/rpc/providers
    /// [RPC method docs]: https://docs.near.org/api/rpc/network#validation-status
    pub fn new(rpc_url: String, block: Option<u64>) -> Self {
        Self { rpc_url, block }
    }
}

impl Protocol for NearProtocol {
    /// Downloads Near validator data via the [`validators`] RPC method.
    ///
    /// [`validators`]: https://docs.near.org/api/rpc/network#validation-status
    fn download_validator_data(&self) -> anyhow::Result<Vec<ValidatorData>> {
        // Construct parameters for the HTTP request to RPC.
        let params = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "validators",
            "id": "dontcare",
            // If `self.block.is_none()` this serializes to `[null]` and latest block is queried.
            "params": vec![self.block],
        });

        // Construct and send the HTTP request.
        let client = reqwest::blocking::Client::new();
        let res = client.post(&self.rpc_url).json(&params).send()?;

        // Handle HTTP and RPC errors.
        anyhow::ensure!(
            res.status() == 200,
            "expected HTTP status code 200, got {} with response body\n\t{}",
            res.status(),
            res.text()?,
        );
        let rpc_res: RpcResponse = res.json()?;
        if let Some(err) = rpc_res.error {
            anyhow::bail!("rpc error: {}", err);
        }

        let validators = rpc_res
            .result
            .expect("response without error should have a result")
            .current_validators
            .into_iter()
            .map(|v| v.into())
            .collect();

        Ok(validators)
    }
}

/// The expected response for the `validators` RPC method. This struct contains only fields used in
/// this module. Additional fields are documented in Near RPC [docs].
///
/// [docs]: https://docs.near.org/api/rpc/network#validation-status
#[derive(Deserialize, Debug)]
struct RpcResponse {
    result: Option<RpcResult>,
    /// Near RPC returns some errors in the response body of a 'successful' (status code 200) HTTP
    /// request.
    error: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct RpcResult {
    current_validators: Vec<RpcValidatorData>,
}

/// Data per validator returned by the RPC. This struct contains only fields used in
/// this module. Additional fields are documented in Near RPC [docs].
///
/// [docs]: https://docs.near.org/api/rpc/network#validation-status
#[derive(Deserialize, Debug)]
struct RpcValidatorData {
    account_id: String,
    /// String representation of `u128`.
    stake: String,
}

impl From<RpcValidatorData> for ValidatorData {
    fn from(data: RpcValidatorData) -> Self {
        Self {
            account_id: data.account_id,
            stake: data
                .stake
                .parse::<u128>()
                .expect("should parse stake as u128"),
            is_malicious: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::Protocol;

    use super::NearProtocol;

    const RPC_URL: &str = "https://archival-rpc.testnet.near.org";
    /// Use a constant block in tests to have deterministic results of RPC queries.
    const BLOCK_HEIGHT: u64 = 139491540;

    #[test]
    fn test_download_validator_data() -> anyhow::Result<()> {
        let protocol = NearProtocol::new(RPC_URL.to_owned(), Some(BLOCK_HEIGHT));
        let validators = protocol.download_validator_data()?;

        // The downloaded `valdiators` should be deterministic since we use a fixed block height.
        assert_eq!(validators.len(), 31);
        // Looking at a subset of `validators` suffices to test validator data is handled correclty.
        insta::assert_debug_snapshot!(&validators[..2]);

        Ok(())
    }

    /// Test querying the latest block. Since the set of active validators changes over time, this
    /// test only verifies that validator data was download but does not check the data itself.
    #[test]
    fn test_download_validator_data_latest() -> anyhow::Result<()> {
        let protocol = NearProtocol::new(RPC_URL.to_owned(), None);
        let validators = protocol.download_validator_data()?;
        assert!(validators.len() > 0);
        Ok(())
    }

    /// Test handling of errors which the RPC sends in the body of a successful (status code 200)
    /// response.
    #[test]
    fn test_rpc_body_error() -> anyhow::Result<()> {
        // Querying a block height that is not the last block in an epoch causes an error.
        let protocol = NearProtocol::new(RPC_URL.to_owned(), Some(42));
        let err = protocol
            .download_validator_data()
            .expect_err("querying an invalid block should lead to an error");
        insta::assert_debug_snapshot!(
            // Verify getting the expected error message.
            err
        );
        Ok(())
    }
}
