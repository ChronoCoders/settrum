#![forbid(unsafe_code)]

//! Subxt-based client for the Settrum chain.
//!
//! Metadata regeneration (run from repo root with a dev node listening on 9944):
//!
//! ```bash
//! target/release/settrum-node --dev --tmp &
//! subxt metadata --url ws://127.0.0.1:9944 -a -o settrum-client/artifacts/metadata.scale
//! ```

pub mod client;
pub mod error;
pub mod runtime;

pub use client::SettrumClient;
pub use error::ClientError;
