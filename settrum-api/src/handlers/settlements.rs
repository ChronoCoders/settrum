use crate::{
    auth::{AdminKey, AuthClaims},
    db,
    error::{created, ok, ApiError},
    AppState,
};
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SubmitSettlementRequest {
    pub asset_id: i32,
    pub operation: String,
    pub amount: String,
    pub from_account: String,
    pub to_account: String,
    pub reference: Option<String>,
}

pub async fn submit(
    state: web::Data<AppState>,
    body: web::Json<SubmitSettlementRequest>,
    claims: AuthClaims,
) -> Result<HttpResponse, ApiError> {
    match body.operation.as_str() {
        "Issue" | "Redeem" | "Transfer" | "Lock" | "Unlock" => {}
        _ => {
            return Err(ApiError::BadRequest(
                "operation must be Issue, Redeem, Transfer, Lock, or Unlock".into(),
            ));
        }
    }

    body.amount
        .parse::<u128>()
        .map_err(|_| ApiError::BadRequest("amount must be a valid u128 integer".into()))?;

    let reference = body.reference.as_deref().unwrap_or("");
    if reference.len() > 256 {
        return Err(ApiError::BadRequest(
            "reference must be at most 256 characters".into(),
        ));
    }

    // Verify operator exists and is active
    let operator = db::get_operator_by_id(&state.db, claims.0.operator_id as i32)
        .await?
        .ok_or_else(|| ApiError::NotFound("operator not found".into()))?;

    if operator.status != "Active" {
        return Err(ApiError::Forbidden);
    }

    // Verify asset exists
    db::get_asset_by_id(&state.db, body.asset_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("asset {} not found", body.asset_id)))?;

    let next_id: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(id) + 1, 0) FROM settlements")
        .fetch_one(&state.db)
        .await?;

    let settlement = db::insert_settlement(
        &state.db,
        db::NewSettlement {
            id: next_id,
            operator_id: operator.id,
            asset_id: body.asset_id,
            operation: &body.operation,
            amount: &body.amount,
            from_account: &body.from_account,
            to_account: &body.to_account,
            reference,
            submitted_at: 0,
        },
    )
    .await?;

    tracing::info!(
        settlement_id = settlement.id,
        operator_id = operator.id,
        operation = %body.operation,
        "settlement submitted"
    );

    Ok(created(settlement))
}

pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let settlements = db::list_settlements(&state.db).await?;
    Ok(ok(settlements))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let settlement = db::get_settlement_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("settlement {id} not found")))?;
    Ok(ok(settlement))
}

pub async fn finalize(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    _admin: AdminKey,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();

    // Check it exists first
    let existing = db::get_settlement_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("settlement {id} not found")))?;

    if existing.status != "Pending" {
        return Err(ApiError::Conflict(format!(
            "settlement {id} is not in Pending status"
        )));
    }

    let settlement = db::finalize_settlement(&state.db, id, 0)
        .await?
        .ok_or_else(|| ApiError::Conflict(format!("settlement {id} could not be finalized")))?;

    tracing::info!(settlement_id = id, "settlement finalized");

    Ok(ok(settlement))
}
