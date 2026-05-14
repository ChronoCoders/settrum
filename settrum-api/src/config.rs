use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub chain_rpc_url: String,
    pub jwt_secret: String,
    pub admin_api_key: String,
    pub admin_seed: String,
    pub jwt_expiration_secs: u64,
    pub rate_limit_requests: u32,
    pub rate_limit_window_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .context("SERVER_PORT must be a valid port number")?,
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            chain_rpc_url: std::env::var("CHAIN_RPC_URL")
                .unwrap_or_else(|_| "ws://localhost:9944".into()),
            jwt_secret: std::env::var("JWT_SECRET").context("JWT_SECRET is required")?,
            admin_api_key: std::env::var("ADMIN_API_KEY").context("ADMIN_API_KEY is required")?,
            admin_seed: std::env::var("ADMIN_SEED").unwrap_or_else(|_| "//Alice".into()),
            jwt_expiration_secs: std::env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "86400".into())
                .parse()
                .context("JWT_EXPIRATION must be a number of seconds")?,
            rate_limit_requests: std::env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "100".into())
                .parse()
                .context("RATE_LIMIT_REQUESTS must be a number")?,
            rate_limit_window_secs: std::env::var("RATE_LIMIT_WINDOW_SECONDS")
                .unwrap_or_else(|_| "60".into())
                .parse()
                .context("RATE_LIMIT_WINDOW_SECONDS must be a number")?,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
