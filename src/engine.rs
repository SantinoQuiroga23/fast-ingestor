//! Application engine.

use std::env;
use std::sync::Arc;
use std::thread;

use tokio::runtime::Builder;

use crate::domain::config::DomainConfig;
use crate::domain::model::TelemetryRecord;
use crate::infrastructure::http::handlers::AppState;
use crate::infrastructure::http::router::build_router;
use crate::infrastructure::queue::bounded::BoundedQueue;

/// The core ingestion engine.
///
/// Owns the bounded queue and spawns the HTTP server on a dedicated thread.
pub struct IngestorEngine {
    queue: Arc<BoundedQueue<TelemetryRecord>>,
}

impl IngestorEngine {
    pub fn new(
        api_key: String,
        port: u16,
        capacity: usize,
        domain_config: DomainConfig,
    ) -> Self {
        let queue = Arc::new(BoundedQueue::with_capacity(capacity));

        let state = Arc::new(AppState {
            queue: queue.clone(),
            domain_config: Arc::new(domain_config),
            api_key,
        });

        thread::Builder::new()
            .name("fast-ingestor-server".to_string())
            .spawn(move || {
                let worker_threads = env::var("TOKIO_WORKER_THREADS")
                    .ok()
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(2)
                    .max(1);

                let runtime = Builder::new_multi_thread()
                    .worker_threads(worker_threads)
                    .enable_all()
                    .build()
                    .expect("failed to build Tokio runtime");

                runtime.block_on(async move {
                    let app = build_router(state);

                    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
                        .await
                        .unwrap_or_else(|error| {
                            panic!("failed to bind port {port}: {error}")
                        });

                    axum::serve(listener, app).await.unwrap_or_else(|error| {
                        panic!("HTTP server error: {error}")
                    });
                });
            })
            .expect("failed to spawn HTTP server thread");

        Self { queue }
    }

    /// Returns a clone of the Arc to the queue, allowing drain operations
    /// from any thread without borrowing self.
    pub fn queue_handle(&self) -> Arc<BoundedQueue<TelemetryRecord>> {
        self.queue.clone()
    }
}