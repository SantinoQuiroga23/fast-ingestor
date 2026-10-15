//! Integration tests for the HTTP layer.
//!
//! Uses `tower::ServiceExt::oneshot` to test the router without
//! binding a real TCP port.

use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

use _lib_ingestor::domain::config::DomainConfig;
use _lib_ingestor::infrastructure::http::handlers::AppState;
use _lib_ingestor::infrastructure::http::router::build_router;
use _lib_ingestor::infrastructure::queue::bounded::BoundedQueue;

const TEST_API_KEY: &str = "test-secret-key";

fn test_state(queue_capacity: usize) -> Arc<AppState> {
    Arc::new(AppState {
        queue: Arc::new(BoundedQueue::with_capacity(queue_capacity)),
        domain_config: Arc::new(DomainConfig::default()),
        api_key: TEST_API_KEY.to_string(),
    })
}

fn valid_json() -> String {
    r#"{
        "timestamp": 1700000000000,
        "device_id": "sensor-001",
        "metric_type": "temperature",
        "value": 23.5
    }"#
    .to_string()
}

async fn post_ingest(router: axum::Router, api_key: &str, body: String) -> StatusCode {
    let request = Request::builder()
        .method("POST")
        .uri("/ingest")
        .header("Content-Type", "application/json")
        .header("X-API-Key", api_key)
        .body(Body::from(body))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    response.status()
}

#[tokio::test]
async fn accepts_valid_request() {
    let router = build_router(test_state(10));
    let status = post_ingest(router, TEST_API_KEY, valid_json()).await;
    assert_eq!(status, StatusCode::ACCEPTED);
}

#[tokio::test]
async fn rejects_missing_api_key() {
    let router = build_router(test_state(10));

    let request = Request::builder()
        .method("POST")
        .uri("/ingest")
        .header("Content-Type", "application/json")
        .body(Body::from(valid_json()))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_wrong_api_key() {
    let router = build_router(test_state(10));
    let status = post_ingest(router, "wrong-key", valid_json()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_empty_api_key_config() {
    let state = Arc::new(AppState {
        queue: Arc::new(BoundedQueue::with_capacity(10)),
        domain_config: Arc::new(DomainConfig::default()),
        api_key: String::new(), // No key configured
    });

    let router = build_router(state);
    let status = post_ingest(router, "any-key", valid_json()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_malformed_json() {
    let router = build_router(test_state(10));
    let status = post_ingest(router, TEST_API_KEY, "not json".to_string()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_domain_validation_failure() {
    let router = build_router(test_state(10));

    let invalid = r#"{
        "timestamp": 999,
        "device_id": "sensor-001",
        "metric_type": "temperature",
        "value": 23.5
    }"#
    .to_string();

    let status = post_ingest(router, TEST_API_KEY, invalid).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn rejects_when_queue_full() {
    // Queue with capacity of only 1
    let router: axum::Router = build_router(test_state(1));

    // First request fills the queue
    let status1 = post_ingest(router.clone(), TEST_API_KEY, valid_json()).await;
    assert_eq!(status1, StatusCode::ACCEPTED);

    // Second request should be rejected
    let status2 = post_ingest(router, TEST_API_KEY, valid_json()).await;
    assert_eq!(status2, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn rejects_oversized_body() {
    // Use a small body limit (4KB) for this specific test
    let state = test_state(10);
    let router = build_router(state).layer(DefaultBodyLimit::max(4096));

    // Body exceeds the 4KB limit
    let large_body = "x".repeat(5000);
    
    let request: Request<Body> = Request::builder()
                .method("POST")
                .uri("/ingest")
                .header("Content-Type", "application/json")
                .header("X-API-Key", TEST_API_KEY)
                .body(Body::from(large_body))
                .unwrap();
    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}