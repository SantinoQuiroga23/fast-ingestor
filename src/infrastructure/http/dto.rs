//! Data Transfer Objects for HTTP input.
//!
//! DTOs represent untrusted external data. They are validated and
//! converted into domain records before entering the queue.

use serde::Deserialize;

/// JSON payload expected by `POST /ingest`.
///
/// This is intentionally separate from `TelemetryRecord` so that
/// the wire format can evolve independently of the domain model.
#[derive(Debug, Deserialize)]
pub struct IngestionRequest {
    pub timestamp: i64,
    pub device_id: String,
    pub metric_type: String,
    pub value: f64,

    /// Optional tenant identifier.
    /// Required when `DomainConfig::multi_tenant` is true.
    #[serde(default)]
    pub tenant_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_minimal_payload() {
        let json = r#"{
            "timestamp": 1700000000000,
            "device_id": "sensor-1",
            "metric_type": "temperature",
            "value": 23.5
        }"#;

        let request: IngestionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.device_id, "sensor-1");
        assert!(request.tenant_id.is_none());
    }

    #[test]
    fn deserializes_payload_with_tenant() {
        let json = r#"{
            "timestamp": 1700000000000,
            "device_id": "sensor-1",
            "metric_type": "temperature",
            "value": 23.5,
            "tenant_id": "acme"
        }"#;

        let request: IngestionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.tenant_id, Some("acme".to_string()));
    }

    #[test]
    fn rejects_missing_required_field() {
        let json = r#"{
            "timestamp": 1700000000000,
            "device_id": "sensor-1",
            "value": 23.5
        }"#;

        let result = serde_json::from_str::<IngestionRequest>(json);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_wrong_type() {
        let json = r#"{
            "timestamp": "not-a-number",
            "device_id": "sensor-1",
            "metric_type": "temperature",
            "value": 23.5
        }"#;

        let result = serde_json::from_str::<IngestionRequest>(json);
        assert!(result.is_err());
    }
}