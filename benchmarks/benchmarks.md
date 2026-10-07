
# Benchmarks

Performance benchmarks for FastIngestor.

## Test Environment

| Component | Specification                              |
| --------- | ------------------------------------------ |
| CPU       | [Your CPU, e.g., AMD Ryzen 5 5600X]        |
| RAM       | [Your RAM, e.g., 16GB DDR4]                |
| Storage   | [Your storage, e.g., NVMe SSD]             |
| OS        | [Your OS, e.g., Windows 11 / Ubuntu 22.04] |
| Python    | [Version, e.g., 3.11.5]                    |
| Rust      | [Version, e.g., 1.75.0]                    |

## Methodology

- **Tool**: Custom Python benchmark using `httpx` async client
- **Payload**: Valid telemetry JSON (~120 bytes per request)
- **Authentication**: API key header included
- **Server**: FastIngestor running on same machine
- **Drain**: Python consumer draining queue every 100ms

## Results

### Throughput vs Load

| Scenario    | Requests | Concurrency | Throughput | P50   | P95   | P99   | Backpressure |
| ----------- | -------- | ----------- | ---------- | ----- | ----- | ----- | ------------ |
| Low Load    | 1,000    | 50          | [X] req/s  | [X]ms | [X]ms | [X]ms | [X]          |
| Medium Load | 10,000   | 200         | [X] req/s  | [X]ms | [X]ms | [X]ms | [X]          |
| High Load   | 50,000   | 500         | [X] req/s  | [X]ms | [X]ms | [X]ms | [X]          |
| Stress Test | 100,000  | 1000        | [X] req/s  | [X]ms | [X]ms | [X]ms | [X]          |

### Memory Usage

| Scenario    | Server Memory | Client Memory |
| ----------- | ------------- | ------------- |
| Idle        | [X] MB        | [X] MB        |
| Medium Load | [X] MB        | [X] MB        |
| High Load   | [X] MB        | [X] MB        |
| Stress Test | [X] MB        | [X] MB        |

### Backpressure Behavior

When the bounded queue reaches capacity, FastIngestor returns HTTP 503.
This prevents OOM and protects the system under extreme load.

| Queue Capacity | Requests Sent | Accepted | Rejected (503) |
| -------------- | ------------- | -------- | -------------- |
| 100,000        | 100,000       | [X]      | [X]            |
| 50,000         | 100,000       | [X]      | [X]            |
| 10,000         | 100,000       | [X]      | [X]            |

## Comparison with Alternatives

| Metric                | FastIngestor | FastAPI + Redis | Node.js Express |
| --------------------- | ------------ | --------------- | --------------- |
| Throughput (10k req)  | [X] req/s    | [X] req/s       | [X] req/s       |
| Memory (idle)         | [X] MB       | [X] MB          | [X] MB          |
| Memory (10k buffered) | [X] MB       | [X] MB          | [X] MB          |
| Setup Complexity      | 5 min        | 30 min          | 20 min          |
| External Dependencies | None         | Redis           | Redis           |

## How to Run Benchmarks

```bash
# Install benchmark dependencies
pip install httpx tqdm psutil polars

# Start the server (in one terminal)
python examples/demo.py

# Run single benchmark (in another terminal)
python benchmarks/load_test.py --requests 10000 --concurrency 200

# Run full benchmark suite
python benchmarks/run_all_benchmarks.py
```
