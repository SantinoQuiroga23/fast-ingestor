//! PyO3 entrypoint.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::domain::config::DomainConfig;
use crate::engine::IngestorEngine;
use crate::infrastructure::ffi::arrow::export_to_pyarrow;

pub mod domain;
mod engine;
pub mod infrastructure;

/// High-throughput ingestion server.
#[pyclass]
pub struct PyIngestor {
    engine: IngestorEngine,
}

#[pymethods]
impl PyIngestor {
    #[new]
    #[pyo3(signature = (api_key, port=8080, capacity=100_000, multi_tenant=false))]
    fn new(
        api_key: String,
        port: u16,
        capacity: usize,
        multi_tenant: bool,
    ) -> Self {
        let domain_config = if multi_tenant {
            DomainConfig::default().with_multi_tenant()
        } else {
            DomainConfig::default()
        };

        Self {
            engine: IngestorEngine::new(api_key, port, capacity, domain_config),
        }
    }

    /// Drains buffered records and returns a PyArrow RecordBatch.
    ///
    /// Raises ValueError when there is no data to drain.
    fn drain_batch(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        // Clone the Arc<BoundedQueue> so the closure is Send
        let queue = self.engine.queue_handle();

        // Drain without GIL (pure Rust operation)
        // NOTE: pyo3::allow_threads is the correct API for PyO3 0.21+
        let records = py.detach(move || queue.drain());

        if records.is_empty() {
            return Err(PyValueError::new_err("buffer is empty"));
        }

        // Export to Arrow (requires GIL for PyArrow interaction)
        export_to_pyarrow(py, records)
    }
}

#[pymodule]
fn _lib_ingestor(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyIngestor>()?;
    Ok(())
}