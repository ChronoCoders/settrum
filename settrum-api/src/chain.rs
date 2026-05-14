/// Minimal chain state snapshot returned by the client.
#[derive(Debug, Clone)]
pub struct ChainStatus {
    pub best_block: u64,
    pub finalized_block: u64,
    pub peers: usize,
}

/// Trait abstracting over the blockchain client.
/// All write operations return the block number at which they were included.
#[async_trait::async_trait]
pub trait ChainClient: Send + Sync + 'static {
    async fn status(&self) -> Result<ChainStatus, String>;
    async fn best_block(&self) -> Result<u64, String>;
}

/// Stub implementation used when no live node is available.
pub struct StubChainClient {
    pub rpc_url: String,
}

#[async_trait::async_trait]
impl ChainClient for StubChainClient {
    async fn status(&self) -> Result<ChainStatus, String> {
        Ok(ChainStatus {
            best_block: 0,
            finalized_block: 0,
            peers: 0,
        })
    }

    async fn best_block(&self) -> Result<u64, String> {
        Ok(0)
    }
}

impl StubChainClient {
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }
}
