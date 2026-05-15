use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("subxt error: {0}")]
    Subxt(#[from] subxt::Error),
    #[error("connection failed after {attempts} attempts: {source}")]
    ConnectFailed {
        attempts: u32,
        source: subxt::Error,
    },
}
