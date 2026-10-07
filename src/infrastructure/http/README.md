# HTTP Layer

Axum-based HTTP server for ingestion.

## Files

| File            | Purpose                                         |
| --------------- | ----------------------------------------------- |
| `dto.rs`      | Data Transfer Objects for JSON input            |
| `security.rs` | Authentication middleware (constant-time)       |
| `handlers.rs` | Request handlers mapping DTO → domain → queue |
| `router.rs`   | Router assembly with middleware stack           |

## Security Stack

Applied in order to every request:

1. **Body limit** (4 KB) — rejects oversized payloads early
2. **API key auth** — constant-time comparison to prevent timing attacks
3. **Schema validation** — rejects malformed JSON with 400
4. **Domain validation** — rejects invalid values with 422
5. **Backpressure** — rejects with 503 when queue is full

## Response Codes

| Code | Meaning                    |
| ---- | -------------------------- |
| 202  | Accepted and queued        |
| 400  | Malformed JSON or schema   |
| 401  | Missing or invalid API key |
| 422  | Domain validation failed   |
| 503  | Queue full (backpressure)  |

## Testing

HTTP tests use `tower::ServiceExt::oneshot` to exercise the router
without binding a real TCP port.
