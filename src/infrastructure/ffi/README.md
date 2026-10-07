
# FFI Layer

Foreign Function Interface for Python integration.

## Files

| File         | Purpose                                                                   |
| ------------ | ------------------------------------------------------------------------- |
| `arrow.rs` | Exports`Vec<TelemetryRecord>` as Arrow RecordBatch via C Data Interface |

## Zero-Copy Design

Arrow's C Data Interface allows Python (via PyArrow) to read Rust-owned
memory buffers without copying. The lifecycle is:

1. Rust builds Arrow arrays from `Vec<TelemetryRecord>`
2. Arrays exported via `FFI_ArrowArray` + `FFI_ArrowSchema`
3. PyArrow imports pointers, not data
4. When Python releases the array, Arrow's release callback frees Rust memory

This means `drain_batch()` produces a DataFrame with no heap duplication.
