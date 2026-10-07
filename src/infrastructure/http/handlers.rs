//! HTTP request handlers.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::domain::config::DomainConfig;
use crate::domain::model::TelemetryRecord;
use crate::infrastructure::http::dto::IngestionRequest;
use crate::infrastructure::queue::bounded::{BoundedQueue, BoundedQueueError};

/// Shared application state accessible from all handlers.
pub struct AppState {
    pub queue: Arc<BoundedQueue<TelemetryRecord>>,
    pub domain_config: Arc<DomainConfig>,
    pub api_key: String,
}

/// Handles `POST /ingest`.
///
/// Pipeline: DTO → domain validation → queue push → response.
pub async fn ingest_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<IngestionRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Convert DTO to domain record with validation
    let record = TelemetryRecord::try_new(
        &state.domain_config,
        request.timestamp,
        request.tenant_id,
        request.device_id,
        request.metric_type,
        request.value,
    )
    .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()))?;

    // Attempt to enqueue
    match state.queue.try_push(record) {
        Ok(()) => Ok(StatusCode::ACCEPTED),
        Err(BoundedQueueError::Full) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "internal buffer full".to_string(),
        )),
        Err(BoundedQueueError::Disconnected) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "queue unavailable".to_string(),
        )),
    }
}