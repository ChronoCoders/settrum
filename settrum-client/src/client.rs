use crate::error::ClientError;
use crate::runtime::api;
use std::time::Duration;
use subxt::{config::PolkadotConfig, utils::H256, OnlineClient};
use subxt_signer::sr25519::Keypair;

const RECONNECT_ATTEMPTS: u32 = 5;
const RECONNECT_BACKOFF: Duration = Duration::from_millis(500);

/// Settrum chain client.
///
/// Uses [`PolkadotConfig`] (modern address shape: `MultiAddress<AccountId, ()>`).
/// Settrum's runtime is derived from the standard Substrate node template,
/// which uses the same address shape. If a future runtime change introduces
/// an account index in the address, switch to a custom Config wrapping
/// `SubstrateConfig`.
#[derive(Clone)]
pub struct SettrumClient {
    inner: OnlineClient<PolkadotConfig>,
}

impl SettrumClient {
    /// Connect to the chain at `rpc_url`, retrying with linear backoff
    /// up to `RECONNECT_ATTEMPTS` times.
    pub async fn connect(rpc_url: &str) -> Result<Self, ClientError> {
        let mut last_err: Option<subxt::Error> = None;
        for attempt in 1..=RECONNECT_ATTEMPTS {
            match OnlineClient::<PolkadotConfig>::from_url(rpc_url).await {
                Ok(inner) => {
                    tracing::info!(rpc_url, attempt, "connected to settrum chain");
                    return Ok(Self { inner });
                }
                Err(e) => {
                    tracing::warn!(rpc_url, attempt, error = %e, "connect attempt failed");
                    last_err = Some(e.into());
                    tokio::time::sleep(RECONNECT_BACKOFF * attempt).await;
                }
            }
        }
        Err(ClientError::ConnectFailed {
            attempts: RECONNECT_ATTEMPTS,
            source: last_err.expect("loop ran at least once"),
        })
    }

    /// Cheap health check: fetch the latest block hash via `at_current_block`.
    pub async fn health_check(&self) -> Result<H256, ClientError> {
        let at = self
            .inner
            .at_current_block()
            .await
            .map_err(Into::<subxt::Error>::into)?;
        Ok(at.block_hash())
    }

    /// Submit a `register_operator` extrinsic and return the in-block tx hash.
    /// Does not wait for finality — that lives in the watcher (Phase 4.3).
    pub async fn submit_register_operator(
        &self,
        signer: &Keypair,
        name: Vec<u8>,
        collateral: u128,
    ) -> Result<H256, ClientError> {
        let bounded =
            api::runtime_types::bounded_collections::bounded_vec::BoundedVec(name);
        let tx = api::tx().operators().register_operator(bounded, collateral);

        let mut tx_client = self
            .inner
            .tx()
            .await
            .map_err(Into::<subxt::Error>::into)?;
        let progress = tx_client
            .sign_and_submit_then_watch_default(&tx, signer)
            .await
            .map_err(Into::<subxt::Error>::into)?;

        let hash = progress.extrinsic_hash();
        tracing::info!(?hash, "register_operator submitted");
        Ok(hash)
    }
}
