
# Queue Layer

Bounded in-memory queue with backpressure.

## Design

The queue uses a fixed-capacity channel. When full, producers receive
an immediate error instead of blocking, allowing the HTTP layer to
respond with `503 Service Unavailable`.

## Memory Budget

Capacity is derived from a memory budget rather than hardcoded:

    capacity = max_buffer_bytes / estimated_record_bytes

This guarantees RAM usage stays within predictable bounds regardless
of traffic spikes.

## Thread Safety

The queue is `Send + Sync`. Producers (HTTP handlers) and the consumer
(Python drain) can operate from different threads concurrently.

## Backpressure Contract

| Condition       | Behavior                           |
| --------------- | ---------------------------------- |
| Queue has space | `try_push` succeeds              |
| Queue is full   | `try_push` returns `QueueFull` |
| Consumer drains | Space freed for new pushes         |
