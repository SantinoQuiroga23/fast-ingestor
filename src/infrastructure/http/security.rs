//! Authentication middleware.
//!
//! Uses constant-time comparison to prevent timing-based key enumeration.

use axum::{
    extract::Request,
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use subtle::ConstantTimeEq;

use crate::infrastructure::http::handlers::AppState;

/// Middleware that validates the `X-API-Key` header.
///
/// Fails closed: if no API key is configured, all requests are rejected.
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if state.api_key.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let provided = request
        .headers()
        .get("X-API-Key")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    let is_valid: bool = provided
        .as_bytes()
        .ct_eq(state.api_key.as_bytes())
        .into();

    if is_valid {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use subtle::ConstantTimeEq;

    #[test]
    fn constant_time_eq_works() {
        let a = b"secret-key";
        let b = b"secret-key";
        let c = b"wrong-key!";

        let equal: bool = a.ct_eq(b).into();
        let different: bool = a.ct_eq(c).into();

        assert!(equal);
        assert!(!different);
    }
}