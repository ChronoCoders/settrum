use crate::error::ApiError;
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use chrono::Utc;
use futures::future::{ready, Ready};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Account address (SS58).
    pub sub: String,
    /// Operator numeric ID.
    pub operator_id: u64,
    /// Expiry (UNIX seconds).
    pub exp: i64,
}

pub fn issue_token(
    account: &str,
    operator_id: u64,
    jwt_secret: &str,
    expiration_secs: u64,
) -> Result<String, ApiError> {
    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(expiration_secs as i64))
        .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("timestamp overflow")))?
        .timestamp();

    let claims = Claims {
        sub: account.to_owned(),
        operator_id,
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("JWT encode error: {e}")))
}

pub fn verify_token(token: &str, jwt_secret: &str) -> Result<Claims, ApiError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|td| td.claims)
    .map_err(|_| ApiError::Unauthorized)
}

/// Extractor: pulls JWT claims from `Authorization: Bearer <token>`.
#[derive(Debug, Clone)]
pub struct AuthClaims(pub Claims);

impl FromRequest for AuthClaims {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let result = extract_claims(req);
        ready(result)
    }
}

fn extract_claims(req: &HttpRequest) -> Result<AuthClaims, ApiError> {
    let config = req
        .app_data::<web::Data<crate::AppState>>()
        .ok_or(ApiError::Unauthorized)?;

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(ApiError::Unauthorized)?;

    verify_token(token, &config.config.jwt_secret).map(AuthClaims)
}

/// Extractor: validates `X-Admin-Key` header.
pub struct AdminKey;

impl FromRequest for AdminKey {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let result = (|| {
            let config = req
                .app_data::<web::Data<crate::AppState>>()
                .ok_or(ApiError::Unauthorized)?;

            let key = req
                .headers()
                .get("X-Admin-Key")
                .and_then(|v| v.to_str().ok())
                .ok_or(ApiError::Unauthorized)?;

            if key == config.config.admin_api_key {
                Ok(AdminKey)
            } else {
                Err(ApiError::Forbidden)
            }
        })();
        ready(result)
    }
}
