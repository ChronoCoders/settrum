#![forbid(unsafe_code)]

use actix_web::{middleware, web, App, HttpServer, ResponseError as _};
use settrum_api::{
    chain::StubChainClient,
    config::Config,
    db,
    handlers::{assets, balances, cross_settlements, health, operators, proofs, settlements},
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

    tracing::info!("starting settrum-api on {bind_addr}");

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
                    // Health
                    .route("/health", web::get().to(health::health))
                    .route("/status", web::get().to(health::status))
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
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}
