use crate::{
    auth::{AdminKey, AuthClaims},
    db,
    error::{created, ok, ApiError},
    AppState,
};
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SubmitProofRequest {
    pub settlement_id: i32,
    pub proof_type: String,
    pub hash: String,
    pub data: Option<String>,
}

pub async fn submit(
    state: web::Data<AppState>,
    body: web::Json<SubmitProofRequest>,
    claims: AuthClaims,
) -> Result<HttpResponse, ApiError> {
    match body.proof_type.as_str() {
        "Signature" | "Oracle" | "Multisig" | "ZeroKnowledge" | "Documentary" => {}
        _ => {
            return Err(ApiError::BadRequest(
                "proof_type must be Signature, Oracle, Multisig, ZeroKnowledge, or Documentary"
                    .into(),
            ));
        }
    }

    if body.hash.is_empty() {
        return Err(ApiError::BadRequest("hash is required".into()));
    }

    // Verify settlement exists
    db::get_settlement_by_id(&state.db, body.settlement_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("settlement {} not found", body.settlement_id))
        })?;

    // Check hash uniqueness
    let existing: Option<i32> = sqlx::query_scalar("SELECT id FROM proofs WHERE hash = $1")
        .bind(&body.hash)
        .fetch_optional(&state.db)
        .await?;
    if existing.is_some() {
        return Err(ApiError::Conflict(
            "a proof with this hash already exists".into(),
        ));
    }

    let next_id: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(id) + 1, 0) FROM proofs")
        .fetch_one(&state.db)
        .await?;

    let operator = db::get_operator_by_id(&state.db, claims.0.operator_id as i32)
        .await?
        .ok_or_else(|| ApiError::NotFound("operator not found".into()))?;

    let data = body.data.as_deref().unwrap_or("");
    if data.len() > 1024 {
        return Err(ApiError::BadRequest(
            "data must be at most 1024 characters".into(),
        ));
    }

    let proof = db::insert_proof(
        &state.db,
        db::NewProof {
            id: next_id,
            settlement_id: body.settlement_id,
            proof_type: &body.proof_type,
            hash: &body.hash,
            submitter: &operator.account,
            data,
            submitted_at: 0,
        },
    )
    .await?;

    tracing::info!(
        proof_id = proof.id,
        settlement_id = body.settlement_id,
        "proof submitted"
    );

    Ok(created(proof))
}

pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let proof = db::get_proof_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("proof {id} not found")))?;
    Ok(ok(proof))
}

pub async fn verify(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    _admin: AdminKey,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();

    let existing = db::get_proof_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("proof {id} not found")))?;

    if existing.status != "Pending" {
        return Err(ApiError::Conflict(format!(
            "proof {id} is not in Pending status"
        )));
    }

    let proof = db::verify_proof(&state.db, id, 0)
        .await?
        .ok_or_else(|| ApiError::Conflict(format!("proof {id} could not be verified")))?;

    tracing::info!(proof_id = id, "proof verified");

    Ok(ok(proof))
}
