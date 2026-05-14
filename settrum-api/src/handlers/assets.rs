use crate::{
    auth::AuthClaims,
    db,
    error::{created, ok, ApiError},
    AppState,
};
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegisterAssetRequest {
    pub name: String,
    pub symbol: String,
    pub asset_type: String,
    pub decimals: u8,
    pub settlement_rules: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSupplyRequest {
    pub total_supply: String,
}

pub async fn register(
    state: web::Data<AppState>,
    body: web::Json<RegisterAssetRequest>,
    claims: AuthClaims,
) -> Result<HttpResponse, ApiError> {
    match body.asset_type.as_str() {
        "Fiat" | "Commodity" | "Security" | "InternalLedger" => {}
        _ => {
            return Err(ApiError::BadRequest(
                "asset_type must be Fiat, Commodity, Security, or InternalLedger".into(),
            ));
        }
    }

    if body.name.is_empty() || body.name.len() > 64 {
        return Err(ApiError::BadRequest(
            "name must be between 1 and 64 characters".into(),
        ));
    }

    if body.symbol.is_empty() || body.symbol.len() > 12 {
        return Err(ApiError::BadRequest(
            "symbol must be between 1 and 12 characters".into(),
        ));
    }

    if body.decimals > 18 {
        return Err(ApiError::BadRequest(
            "decimals must be between 0 and 18".into(),
        ));
    }

    let settlement_rules = body.settlement_rules.as_deref().unwrap_or("");
    if settlement_rules.len() > 256 {
        return Err(ApiError::BadRequest(
            "settlement_rules must be at most 256 characters".into(),
        ));
    }

    let operator = db::get_operator_by_id(&state.db, claims.0.operator_id as i32)
        .await?
        .ok_or_else(|| ApiError::NotFound("operator not found".into()))?;

    let asset = db::insert_asset(
        &state.db,
        db::NewAsset {
            issuer: &operator.account,
            name: &body.name,
            symbol: &body.symbol,
            asset_type: &body.asset_type,
            decimals: body.decimals as i16,
            total_supply: "0",
            settlement_rules,
            registered_at: 0,
        },
    )
    .await?;

    tracing::info!(asset_id = asset.id, symbol = %body.symbol, "asset registered");

    Ok(created(asset))
}

pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let assets = db::list_assets(&state.db).await?;
    Ok(ok(assets))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let asset = db::get_asset_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("asset {id} not found")))?;
    Ok(ok(asset))
}

pub async fn update_supply(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    body: web::Json<UpdateSupplyRequest>,
    _claims: AuthClaims,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();

    body.total_supply
        .parse::<u128>()
        .map_err(|_| ApiError::BadRequest("total_supply must be a valid u128 integer".into()))?;

    let asset = db::update_asset_supply(&state.db, id, &body.total_supply)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("asset {id} not found")))?;

    Ok(ok(asset))
}
