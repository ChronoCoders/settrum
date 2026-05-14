#![forbid(unsafe_code)]

pub mod auth;
pub mod chain;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;

use std::sync::Arc;

use config::Config;
use sqlx::PgPool;

pub struct AppState {
    pub db: PgPool,
    pub chain: Arc<dyn chain::ChainClient>,
    pub config: Config,
}
