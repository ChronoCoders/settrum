use crate::{error::ok, AppState};
use actix_web::web;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: &'static str,
    pub best_block: u64,
    pub finalized_block: u64,
    pub peers: usize,
}

pub async fn health() -> actix_web::HttpResponse {
    ok(HealthResponse { status: "ok" })
}

pub async fn status(state: web::Data<AppState>) -> actix_web::HttpResponse {
    match state.chain.status().await {
        Ok(s) => ok(StatusResponse {
            status: "ok",
            best_block: s.best_block,
            finalized_block: s.finalized_block,
            peers: s.peers,
        }),
        Err(e) => {
            tracing::warn!("chain status error: {e}");
            ok(StatusResponse {
                status: "degraded",
                best_block: 0,
                finalized_block: 0,
                peers: 0,
            })
        }
    }
}
