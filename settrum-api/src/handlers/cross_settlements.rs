use crate::{
    auth::AuthClaims,
    db,
    error::{created, ok, ApiError},
    AppState,
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Leg {
    pub asset_id: i32,
    pub from_account: String,
    pub to_account: String,
    pub amount: String,
}

#[derive(Debug, Deserialize)]
pub struct ProposeCrossSettlementRequest {
    pub participants: Vec<i32>,
    pub legs: Vec<Leg>,
    pub reference: Option<String>,
    pub expires_in_blocks: Option<i32>,
}

pub async fn propose(
    state: web::Data<AppState>,
    body: web::Json<ProposeCrossSettlementRequest>,
    claims: AuthClaims,
) -> Result<HttpResponse, ApiError> {
    if body.participants.is_empty() || body.participants.len() > 10 {
        return Err(ApiError::BadRequest(
            "participants must contain 1 to 10 operator IDs".into(),
        ));
    }

    if body.legs.is_empty() || body.legs.len() > 20 {
        return Err(ApiError::BadRequest(
            "legs must contain 1 to 20 entries".into(),
        ));
    }

    // Validate all amounts
    for leg in &body.legs {
        leg.amount
            .parse::<u128>()
            .map_err(|_| ApiError::BadRequest("leg amount must be a valid u128 integer".into()))?;
    }

    let reference = body.reference.as_deref().unwrap_or("");
    if reference.len() > 256 {
        return Err(ApiError::BadRequest(
            "reference must be at most 256 characters".into(),
        ));
    }

    let expires_in = body.expires_in_blocks.unwrap_or(100);

    let next_id: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(id) + 1, 0) FROM cross_settlements")
        .fetch_one(&state.db)
        .await?;

    let legs_json = serde_json::to_value(&body.legs)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("leg serialization error: {e}")))?;

    let cross = db::insert_cross_settlement(
        &state.db,
        next_id,
        claims.0.operator_id as i32,
        &body.participants,
        &legs_json,
        reference,
        0,
        expires_in,
    )
    .await?;

    tracing::info!(
        cross_settlement_id = cross.id,
        initiator_id = claims.0.operator_id,
        "cross-settlement proposed"
    );

    Ok(created(cross))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let cross = db::get_cross_settlement_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("cross-settlement {id} not found")))?;
    Ok(ok(cross))
}

pub async fn approve(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    claims: AuthClaims,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let operator_id = claims.0.operator_id as i32;

    // Check cross-settlement exists
    let cross = db::get_cross_settlement_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("cross-settlement {id} not found")))?;

    if cross.status != "Pending" {
        return Err(ApiError::Conflict(format!(
            "cross-settlement {id} is not in Pending status"
        )));
    }

    if !cross.participants.contains(&operator_id) {
        return Err(ApiError::Forbidden);
    }

    let updated = db::approve_cross_settlement(&state.db, id, operator_id)
        .await?
        .ok_or_else(|| {
            ApiError::Conflict(format!(
                "cross-settlement {id} approval failed (already approved or status changed)"
            ))
        })?;

    // Check if all participants have approved
    let all_approved = cross
        .participants
        .iter()
        .all(|p| updated.approvals.contains(p));

    let final_cross = if all_approved {
        sqlx::query_as::<_, db::CrossSettlementRow>(
            "UPDATE cross_settlements SET status = 'Approved', updated_at = NOW()
             WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_one(&state.db)
        .await?
    } else {
        updated
    };

    tracing::info!(
        cross_settlement_id = id,
        operator_id,
        "cross-settlement approved"
    );

    Ok(ok(final_cross))
}

pub async fn execute(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    claims: AuthClaims,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();

    let cross = db::get_cross_settlement_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("cross-settlement {id} not found")))?;

    if cross.status != "Approved" {
        return Err(ApiError::Conflict(format!(
            "cross-settlement {id} must be in Approved status to execute"
        )));
    }

    if cross.initiator_id != claims.0.operator_id as i32 {
        return Err(ApiError::Forbidden);
    }

    let executed = db::execute_cross_settlement(&state.db, id, 0)
        .await?
        .ok_or_else(|| {
            ApiError::Conflict(format!("cross-settlement {id} could not be executed"))
        })?;

    tracing::info!(cross_settlement_id = id, "cross-settlement executed");

    Ok(ok(executed))
}
