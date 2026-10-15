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

/// Default maximum body size in bytes (2 MB).
pub const DEFAULT_MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

/// Builds the Axum router with the default middleware stack.
pub fn build_router(state: Arc<AppState>) -> Router {
    build_router_with_limit(state, DEFAULT_MAX_BODY_BYTES)
}

/// Builds the Axum router with a custom body size limit.
/// Useful for tests that need to verify backpressure behavior.
pub fn build_router_with_limit(state: Arc<AppState>, max_body_bytes: usize) -> Router {
    Router::new()
        .route("/ingest", post(ingest_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(RequestBodyLimitLayer::new(max_body_bytes))
        .with_state(state)
}