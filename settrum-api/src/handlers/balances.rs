use crate::{
    db,
    error::{ok, ApiError},
    AppState,
};
use actix_web::{web, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
struct BalanceResponse {
    account: String,
    asset_id: i64,
    balance: String,
}

pub async fn get_balance(
    state: web::Data<AppState>,
    path: web::Path<(i64, String)>,
) -> Result<HttpResponse, ApiError> {
    let (asset_id, account) = path.into_inner();

    let row = db::get_balance(&state.db, &account, asset_id).await?;

    Ok(ok(BalanceResponse {
        account,
        asset_id,
        balance: row.map(|r| r.balance).unwrap_or_else(|| "0".into()),
    }))
}

pub async fn get_locked_balance(
    state: web::Data<AppState>,
    path: web::Path<(i64, String)>,
) -> Result<HttpResponse, ApiError> {
    let (asset_id, account) = path.into_inner();

    let row = db::get_locked_balance(&state.db, &account, asset_id).await?;

    Ok(ok(BalanceResponse {
        account,
        asset_id,
        balance: row.map(|r| r.balance).unwrap_or_else(|| "0".into()),
    }))
}
