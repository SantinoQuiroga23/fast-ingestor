//! Pure domain model.

use crate::domain::config::DomainConfig;
use crate::domain::error::DomainError;

/// Validated telemetry record.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryRecord {
    pub timestamp_ms: i64,
    pub tenant_id: Option<String>,
    pub device_id: String,
    pub metric_type: String,
    pub value: f64,
}

impl TelemetryRecord {
    /// Creates a record after enforcing domain rules defined in `config`.
    pub fn try_new(
        config: &DomainConfig,
        timestamp_ms: i64,
        tenant_id: Option<String>,
        device_id: String,
        metric_type: String,
        value: f64,
    ) -> Result<Self, DomainError> {
        // Timestamp range
        if timestamp_ms < config.min_timestamp_ms
            || timestamp_ms > config.max_timestamp_ms
        {
            return Err(DomainError::InvalidTimestamp(timestamp_ms));
        }

        // Device ID
        if device_id.is_empty() || device_id.len() > config.max_device_id_len {
            return Err(DomainError::InvalidDeviceId);
        }

        // Metric type
        if metric_type.is_empty()
            || metric_type.len() > config.max_metric_type_len
        {
            return Err(DomainError::InvalidMetricType);
        }

        // Tenant (conditional)
        let validated_tenant = if config.multi_tenant {
            match &tenant_id {
                Some(t) if !t.is_empty() && t.len() <= config.max_tenant_id_len => {
                    Some(t.clone())
                }
                _ => return Err(DomainError::InvalidTenantId),
            }
        } else {
            None
        };

        // Value: finite and within configured bounds
        if !value.is_finite() || value.abs() > config.max_abs_value {
            return Err(DomainError::InvalidMetricValue(value));
        }

        Ok(Self {
            timestamp_ms,
            tenant_id: validated_tenant,
            device_id,
            metric_type,
            value,
        })
    }

    /// Estimates the in-memory size of this record in bytes.
    /// Used by the backpressure system to calculate channel capacity.
    pub fn estimated_size_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.device_id.len()
            + self.metric_type.len()
            + self.tenant_id.as_ref().map_or(0, |t| t.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> DomainConfig {
        DomainConfig::default()
    }

    fn tenant_config() -> DomainConfig {
        DomainConfig::default().with_multi_tenant()
    }

    #[test]
    fn accepts_valid_record() {
        let record = TelemetryRecord::try_new(
            &default_config(),
            1_700_000_000_000,
            None,
            "sensor-001".to_string(),
            "temperature".to_string(),
            23.7,
        );
        assert!(record.is_ok());
        assert!(record.unwrap().tenant_id.is_none());
    }

    #[test]
    fn accepts_valid_record_with_tenant() {
        let record = TelemetryRecord::try_new(
            &tenant_config(),
            1_700_000_000_000,
            Some("acme-corp".to_string()),
            "sensor-001".to_string(),
            "temperature".to_string(),
            23.7,
        );
        assert!(record.is_ok());
        assert_eq!(
            record.unwrap().tenant_id,
            Some("acme-corp".to_string())
        );
    }

    #[test]
    fn rejects_missing_tenant_when_multi_tenant() {
        let record = TelemetryRecord::try_new(
            &tenant_config(),
            1_700_000_000_000,
            None,
            "sensor-001".to_string(),
            "temperature".to_string(),
            23.7,
        );
        assert_eq!(record.unwrap_err(), DomainError::InvalidTenantId);
    }

    #[test]
    fn ignores_tenant_when_not_multi_tenant() {
        let record = TelemetryRecord::try_new(
            &default_config(),
            1_700_000_000_000,
            Some("should-be-ignored".to_string()),
            "sensor-001".to_string(),
            "temperature".to_string(),
            23.7,
        );
        assert!(record.is_ok());
        assert!(record.unwrap().tenant_id.is_none());
    }

    #[test]
    fn rejects_timestamp_out_of_range() {
        let record = TelemetryRecord::try_new(
            &default_config(),
            999,
            None,
            "sensor-001".to_string(),
            "temperature".to_string(),
            23.7,
        );
        assert_eq!(record.unwrap_err(), DomainError::InvalidTimestamp(999));
    }

    #[test]
    fn rejects_empty_device_id() {
        let record = TelemetryRecord::try_new(
            &default_config(),
            1_700_000_000_000,
            None,
            String::new(),
            "temperature".to_string(),
            23.7,
        );
        assert_eq!(record.unwrap_err(), DomainError::InvalidDeviceId);
    }

    #[test]
    fn rejects_nan_value() {
        let record = TelemetryRecord::try_new(
            &default_config(),
            1_700_000_000_000,
            None,
            "sensor-001".to_string(),
            "temperature".to_string(),
            f64::NAN,
        );
        assert!(matches!(
            record.unwrap_err(),
            DomainError::InvalidMetricValue(_)
        ));
    }

    #[test]
    fn respects_custom_max_value() {
        let config = DomainConfig::default().with_max_value(50.0);

        let ok = TelemetryRecord::try_new(
            &config,
            1_700_000_000_000,
            None,
            "sensor-001".to_string(),
            "temperature".to_string(),
            49.9,
        );
        assert!(ok.is_ok());

        let err = TelemetryRecord::try_new(
            &config,
            1_700_000_000_000,
            None,
            "sensor-001".to_string(),
            "temperature".to_string(),
            50.1,
        );
        assert!(err.is_err());
    }

    #[test]
    fn estimated_size_is_reasonable() {
        let record = TelemetryRecord::try_new(
            &default_config(),
            1_700_000_000_000,
            None,
            "sensor-001".to_string(),
            "temperature".to_string(),
            23.7,
        )
        .unwrap();

        let size = record.estimated_size_bytes();
        // Struct ~100 bytes + strings ~25 bytes = ~125 bytes minimum
        assert!(size > 100);
        assert!(size < 1000);
    }
}