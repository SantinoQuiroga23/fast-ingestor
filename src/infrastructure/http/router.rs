//! Router assembly with middleware stack.

use axum::{
    middleware,
    routing::post,
    Router,
};
use std::sync::Arc;
use tower_http::limit::RequestBodyLimitLayer;

use crate::infrastructure::http::handlers::{ingest_handler, AppState};
use crate::infrastructure::http::security::auth_middleware;

/// Maximum allowed body size in bytes (1 MB).
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Builds the Axum router with the full middleware stack.
///
/// Middleware order (applied bottom-up in Axum):
/// 1. Body limit
/// 2. Authentication
/// 3. Route handler
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ingest", post(ingest_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .with_state(state)
}