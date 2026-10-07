//! Domain errors.

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum DomainError {
    #[error("timestamp out of valid range: {0}")]
    InvalidTimestamp(i64),

    #[error("device_id is empty or exceeds length limit")]
    InvalidDeviceId,

    #[error("metric_type is empty or exceeds length limit")]
    InvalidMetricType,

    #[error("tenant_id is required but missing or invalid")]
    InvalidTenantId,

    #[error("metric value is non-finite or out of physical bounds: {0}")]
    InvalidMetricValue(f64),

    #[error("internal bounded queue is full")]
    QueueFull,
}