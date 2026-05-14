use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    NotFound(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("rate limit exceeded")]
    TooManyRequests,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
    #[error("chain error: {0}")]
    Chain(String),
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: ErrorBody,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, code) = match self {
            Self::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, "NOT_FOUND"),
            Self::Unauthorized => (actix_web::http::StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            Self::Forbidden => (actix_web::http::StatusCode::FORBIDDEN, "FORBIDDEN"),
            Self::BadRequest(_) => (actix_web::http::StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            Self::Conflict(_) => (actix_web::http::StatusCode::CONFLICT, "CONFLICT"),
            Self::TooManyRequests => (
                actix_web::http::StatusCode::TOO_MANY_REQUESTS,
                "TOO_MANY_REQUESTS",
            ),
            Self::Database(e) => {
                tracing::error!("database error: {e}");
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                )
            }
            Self::Internal(e) => {
                tracing::error!("internal error: {e}");
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                )
            }
            Self::Chain(_) => (actix_web::http::StatusCode::BAD_GATEWAY, "CHAIN_ERROR"),
        };

        let message = match self {
            Self::Database(_) | Self::Internal(_) => "An internal error occurred".to_string(),
            other => other.to_string(),
        };

        HttpResponse::build(status).json(ErrorResponse {
            success: false,
            error: ErrorBody {
                code: code.to_string(),
                message,
            },
        })
    }
}

pub fn ok<T: Serialize>(data: T) -> HttpResponse {
    #[derive(Serialize)]
    struct OkResponse<T: Serialize> {
        success: bool,
        data: T,
    }
    HttpResponse::Ok().json(OkResponse {
        success: true,
        data,
    })
}

pub fn created<T: Serialize>(data: T) -> HttpResponse {
    #[derive(Serialize)]
    struct CreatedResponse<T: Serialize> {
        success: bool,
        data: T,
    }
    HttpResponse::Created().json(CreatedResponse {
        success: true,
        data,
    })
}
