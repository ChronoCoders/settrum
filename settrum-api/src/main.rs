#![forbid(unsafe_code)]

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware, web, App, HttpServer, ResponseError as _};
use settrum_api::{
    chain::StubChainClient,
    config::Config,
    db,
    handlers::{assets, balances, cross_settlements, health, operators, proofs, settlements},
    rate_limit::PeerIpExtractor,
    AppState,
};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env if present (ignore error in production)
    let _ = dotenvy::dotenv();

    // Tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env().unwrap_or_else(|e| {
        tracing::error!("configuration error: {e}");
        std::process::exit(1);
    });

    let pool = db::connect(&config.database_url).await.unwrap_or_else(|e| {
        tracing::error!("database connection error: {e}");
        std::process::exit(1);
    });

    db::run_migrations(&pool).await.unwrap_or_else(|e| {
        tracing::error!("migration error: {e}");
        std::process::exit(1);
    });

    let chain = Arc::new(StubChainClient::new(config.chain_rpc_url.clone()));
    let bind_addr = config.bind_addr();

    if config.rate_limit_requests == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "RATE_LIMIT_REQUESTS must be greater than 0",
        ));
    }
    let interval_ms =
        (config.rate_limit_window_secs * 1000) / u64::from(config.rate_limit_requests);
    if interval_ms == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "RATE_LIMIT_REQUESTS / RATE_LIMIT_WINDOW_SECONDS resolves to a sub-millisecond interval",
        ));
    }
    let governor_conf = GovernorConfigBuilder::default()
        .milliseconds_per_request(interval_ms)
        .burst_size(config.rate_limit_requests)
        .key_extractor(PeerIpExtractor)
        .finish()
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid rate limit config")
        })?;

    tracing::info!(
        "starting settrum-api on {bind_addr} (rate limit: {} req per {}s, ~{}ms interval)",
        config.rate_limit_requests,
        config.rate_limit_window_secs,
        interval_ms,
    );

    let state = web::Data::new(AppState {
        db: pool,
        chain,
        config,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(web::JsonConfig::default().error_handler(|err, _| {
                actix_web::error::InternalError::from_response(
                    err,
                    settrum_api::error::ApiError::BadRequest("invalid JSON body".into())
                        .error_response(),
                )
                .into()
            }))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api/v1")
                    // Health endpoints — exempt from rate limiting
                    .route("/health", web::get().to(health::health))
                    .route("/status", web::get().to(health::status))
                    .service(
                        web::scope("")
                            .wrap(Governor::new(&governor_conf))
                            // Auth
                            .route("/auth/login", web::post().to(operators::login))
                            // Operators
                            .route("/operators", web::post().to(operators::register))
                            .route("/operators", web::get().to(operators::list))
                            .route("/operators/me", web::get().to(operators::me))
                            .route("/operators/{id}", web::get().to(operators::get))
                            .route(
                                "/operators/{id}/status",
                                web::put().to(operators::update_status),
                            )
                            // Assets
                            .route("/assets", web::post().to(assets::register))
                            .route("/assets", web::get().to(assets::list))
                            .route("/assets/{id}", web::get().to(assets::get))
                            .route("/assets/{id}/supply", web::put().to(assets::update_supply))
                            // Settlements
                            .route("/settlements", web::post().to(settlements::submit))
                            .route("/settlements", web::get().to(settlements::list))
                            .route("/settlements/{id}", web::get().to(settlements::get))
                            .route(
                                "/settlements/{id}/finalize",
                                web::post().to(settlements::finalize),
                            )
                            // Balances
                            .route(
                                "/balances/{asset_id}/{account_id}",
                                web::get().to(balances::get_balance),
                            )
                            .route(
                                "/balances/locked/{asset_id}/{account_id}",
                                web::get().to(balances::get_locked_balance),
                            )
                            // Proofs
                            .route("/proofs", web::post().to(proofs::submit))
                            .route("/proofs/{id}", web::get().to(proofs::get))
                            .route("/proofs/{id}/verify", web::put().to(proofs::verify))
                            // Cross-settlements
                            .route(
                                "/cross-settlements",
                                web::post().to(cross_settlements::propose),
                            )
                            .route(
                                "/cross-settlements/{id}",
                                web::get().to(cross_settlements::get),
                            )
                            .route(
                                "/cross-settlements/{id}/approve",
                                web::post().to(cross_settlements::approve),
                            )
                            .route(
                                "/cross-settlements/{id}/execute",
                                web::post().to(cross_settlements::execute),
                            ),
                    ),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}
