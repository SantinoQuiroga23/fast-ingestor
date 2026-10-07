//! Domain configuration.
//!
//! Controls which validations are active and their thresholds.
//! Created once at startup and passed to every `try_new()` call.

/// Configuration for domain validation rules.
///
/// # Example
///
/// ```
/// use _lib_ingestor::domain::config::DomainConfig;
///
/// let config = DomainConfig::default();
/// assert!(!config.multi_tenant);
/// assert!(config.max_abs_value == 1000.0);
/// ```
///
#[derive(Debug, Clone)]
pub struct DomainConfig {
    /// When true, `tenant_id` is required and validated.
    /// When false, `tenant_id` is ignored.
    pub multi_tenant: bool,

    /// Maximum absolute value allowed for `value`.
    /// Values outside [-max_abs_value, +max_abs_value] are rejected.
    pub max_abs_value: f64,

    /// Minimum allowed timestamp (epoch milliseconds).
    pub min_timestamp_ms: i64,

    /// Maximum allowed timestamp (epoch milliseconds).
    pub max_timestamp_ms: i64,

    /// Maximum byte length for `device_id`.
    pub max_device_id_len: usize,

    /// Maximum byte length for `metric_type`.
    pub max_metric_type_len: usize,

    /// Maximum byte length for `tenant_id` (only when multi_tenant=true).
    pub max_tenant_id_len: usize,
}

impl Default for DomainConfig {
    fn default() -> Self {
        Self {
            multi_tenant: false,
            max_abs_value: 1000.0,
            min_timestamp_ms: 1_577_836_800_000,  // 2020-01-01
            max_timestamp_ms: 2_524_608_000_000,  // 2050-01-01
            max_device_id_len: 64,
            max_metric_type_len: 32,
            max_tenant_id_len: 64,
        }
    }
}

impl DomainConfig {
    /// Creates a config with multi-tenant mode enabled.
    pub fn with_multi_tenant(mut self) -> Self {
        self.multi_tenant = true;
        self
    }

    /// Overrides the maximum absolute value.
    pub fn with_max_value(mut self, max: f64) -> Self {
        self.max_abs_value = max;
        self
    }

    /// Overrides the timestamp range.
    pub fn with_timestamp_range(mut self, min: i64, max: i64) -> Self {
        self.min_timestamp_ms = min;
        self.max_timestamp_ms = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_sane() {
        let config = DomainConfig::default();
        assert!(!config.multi_tenant);
        assert!(config.max_abs_value > 0.0);
        assert!(config.min_timestamp_ms < config.max_timestamp_ms);
    }

    #[test]
    fn builder_methods_work() {
        let config = DomainConfig::default()
            .with_multi_tenant()
            .with_max_value(500.0);

        assert!(config.multi_tenant);
        assert_eq!(config.max_abs_value, 500.0);
    }
}