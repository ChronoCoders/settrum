use crate::{
    auth::{issue_token, AdminKey, AuthClaims},
    db,
    error::{created, ok, ApiError},
    AppState,
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub account: String,
    pub name: String,
    pub collateral: String,
}

#[derive(Debug, Deserialize)]
pub struct StatusUpdateRequest {
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct OperatorWithToken {
    #[serde(flatten)]
    pub operator: db::OperatorRow,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

pub async fn register(
    state: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, ApiError> {
    let existing = db::get_operator_by_account(&state.db, &body.account).await?;
    if existing.is_some() {
        return Err(ApiError::Conflict(format!(
            "account {} is already registered",
            body.account
        )));
    }

    // Validate collateral is a valid u128
    body.collateral
        .parse::<u128>()
        .map_err(|_| ApiError::BadRequest("collateral must be a valid u128 integer".into()))?;

    // Validate name length
    if body.name.is_empty() || body.name.len() > 64 {
        return Err(ApiError::BadRequest(
            "name must be between 1 and 64 characters".into(),
        ));
    }

    // Get next operator ID from the DB (max existing + 1)
    let next_id: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(id) + 1, 0) FROM operators")
        .fetch_one(&state.db)
        .await?;

    let operator = db::insert_operator(
        &state.db,
        next_id,
        &body.account,
        &body.name,
        &body.collateral,
        0, // block 0 for API-submitted registrations
    )
    .await?;

    let token = issue_token(
        &operator.account,
        operator.id as u32,
        &state.config.jwt_secret,
        state.config.jwt_expiration_secs,
    )?;

    tracing::info!(operator_id = operator.id, "operator registered");

    Ok(created(OperatorWithToken {
        operator,
        token: Some(token),
    }))
}

pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let operators = db::list_operators(&state.db).await?;
    Ok(ok(operators))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let operator = db::get_operator_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("operator {id} not found")))?;
    Ok(ok(operator))
}

pub async fn update_status(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    body: web::Json<StatusUpdateRequest>,
    _admin: AdminKey,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();

    match body.status.as_str() {
        "Active" | "Suspended" | "Terminated" => {}
        _ => {
            return Err(ApiError::BadRequest(
                "status must be Active, Suspended, or Terminated".into(),
            ));
        }
    }

    let operator = db::update_operator_status(&state.db, id, &body.status)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("operator {id} not found")))?;

    tracing::info!(operator_id = id, status = %body.status, "operator status updated");

    Ok(ok(operator))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub account: String,
}

pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let operator = db::get_operator_by_account(&state.db, &body.account)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("operator {} not found", body.account)))?;

    if operator.status != "Active" {
        return Err(ApiError::Forbidden);
    }

    let token = issue_token(
        &operator.account,
        operator.id as u32,
        &state.config.jwt_secret,
        state.config.jwt_expiration_secs,
    )?;

    Ok(ok(serde_json::json!({ "token": token })))
}

pub async fn me(state: web::Data<AppState>, claims: AuthClaims) -> Result<HttpResponse, ApiError> {
    let operator = db::get_operator_by_id(&state.db, claims.0.operator_id as i32)
        .await?
        .ok_or_else(|| ApiError::NotFound("operator not found".into()))?;
    Ok(ok(operator))
}
