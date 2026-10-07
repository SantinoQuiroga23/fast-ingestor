
# Domain Layer

Pure business logic. Zero I/O dependencies.

## Files

| File          | Purpose                                                   |
| ------------- | --------------------------------------------------------- |
| `config.rs` | `DomainConfig`: parametrizes validation rules           |
| `model.rs`  | `TelemetryRecord`: canonical data model with validation |
| `error.rs`  | `DomainError`: business-rule violations                 |

## Configuration

`DomainConfig` controls validation behavior:

| Parameter               | Default           | Description                         |
| ----------------------- | ----------------- | ----------------------------------- |
| `multi_tenant`        | `false`         | When true,`tenant_id` is required |
| `max_abs_value`       | `1000.0`        | Max absolute value for metrics      |
| `min_timestamp_ms`    | `1577836800000` | Min epoch ms (2020-01-01)           |
| `max_timestamp_ms`    | `2524608000000` | Max epoch ms (2050-01-01)           |
| `max_device_id_len`   | `64`            | Max bytes for device_id             |
| `max_metric_type_len` | `32`            | Max bytes for metric_type           |
| `max_tenant_id_len`   | `64`            | Max bytes for tenant_id             |

### Usage

```rust
let config = DomainConfig::default()
    .with_multi_tenant()
    .with_max_value(500.0);

let record = TelemetryRecord::try_new(
    &config,
    timestamp,
    Some("acme".to_string()),
    device_id,
    metric_type,
    value,
)?;
```
