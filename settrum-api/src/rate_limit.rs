use crate::error::ApiError;
use actix_governor::{KeyExtractor, SimpleKeyExtractionError};
use actix_web::{dev::ServiceRequest, HttpResponse, HttpResponseBuilder, ResponseError};
use governor::{clock::QuantaInstant, NotUntil};
use std::net::IpAddr;

/// Per-peer-IP key extractor. actix-governor cannot natively switch keys
/// based on whether a request is authenticated, so we rate-limit every
/// request by IP. Authenticated callers behind a NAT will share a budget;
/// per-operator quotas would require pre-extracting JWT claims in middleware,
/// which is out of scope for this wiring.
#[derive(Clone)]
pub struct PeerIpExtractor;

impl KeyExtractor for PeerIpExtractor {
    type Key = IpAddr;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        req.peer_addr().map(|s| s.ip()).ok_or_else(|| {
            SimpleKeyExtractionError::new("could not determine peer IP for rate limiting")
        })
    }

    fn exceed_rate_limit_response(
        &self,
        _negative: &NotUntil<QuantaInstant>,
        _response: HttpResponseBuilder,
    ) -> HttpResponse {
        ApiError::TooManyRequests.error_response()
    }
}
