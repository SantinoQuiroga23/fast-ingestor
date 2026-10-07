"""
FastIngestor Load Test Benchmark

Measures throughput, latency, memory usage, and backpressure behavior.

Usage:
    python benchmarks/load_test.py --requests 10000 --concurrency 200

Requirements:
    pip install httpx tqdm psutil polars
"""
import argparse
import asyncio
import json
import os
import random
import time
from dataclasses import dataclass, field
from typing import Optional

import httpx
import psutil
from tqdm.asyncio import tqdm_asyncio


@dataclass
class BenchmarkConfig:
    url: str = "http://localhost:8080/ingest"
    api_key: str = "my-secret-key"
    total_requests: int = 10_000
    concurrency: int = 200
    timeout: float = 10.0


@dataclass
class BenchmarkResult:
    total_requests: int = 0
    concurrency: int = 0
    duration_seconds: float = 0.0
    status_codes: dict = field(default_factory=dict)
    latencies: list = field(default_factory=list)
    memory_before_mb: float = 0.0
    memory_after_mb: float = 0.0
    throughput_rps: float = 0.0
    p50_latency_ms: float = 0.0
    p95_latency_ms: float = 0.0
    p99_latency_ms: float = 0.0
    backpressure_count: int = 0
    accepted_count: int = 0
    error_count: int = 0


def make_payload() -> dict:
    """Generate a valid telemetry payload."""
    return {
        "timestamp": int(time.time() * 1000),
        "device_id": f"sensor_{random.randint(1, 1000):04d}",
        "metric_type": random.choice(["temperature", "humidity", "pressure"]),
        "value": round(random.uniform(-50.0, 100.0), 2),
    }


async def send_one(
    client: httpx.AsyncClient,
    semaphore: asyncio.Semaphore,
    config: BenchmarkConfig,
) -> tuple[int, float]:
    """Send one request, return (status_code, latency_ms)."""
    async with semaphore:
        start = time.perf_counter()
        try:
            # Usar streaming para no acumular responses en memoria
            async with client.stream(
                "POST",
                config.url,
                json=make_payload(),
                headers={"X-API-Key": config.api_key},
            ) as response:
                # Consumir el body para liberar la conexión
                await response.aread()
                status_code = response.status_code
            latency_ms = (time.perf_counter() - start) * 1000
            return status_code, latency_ms
        except Exception as e:
            latency_ms = (time.perf_counter() - start) * 1000
            return 0, latency_ms


async def run_benchmark(config: BenchmarkConfig) -> BenchmarkResult:
    """Execute the benchmark and collect results."""
    result = BenchmarkResult(
        total_requests=config.total_requests,
        concurrency=config.concurrency,
    )

    semaphore = asyncio.Semaphore(config.concurrency)
    limits = httpx.Limits(
        max_keepalive_connections=config.concurrency,
        max_connections=config.concurrency,
    )

    # Measure memory before
    process = psutil.Process(os.getpid())
    result.memory_before_mb = process.memory_info().rss / (1024 * 1024)

    print(f"\n{'='*60}")
    print(f"Benchmark Configuration")
    print(f"{'='*60}")
    print(f"  Total Requests:  {config.total_requests:,}")
    print(f"  Concurrency:     {config.concurrency}")
    print(f"  Target URL:      {config.url}")
    print(f"  Timeout:         {config.timeout}s")
    print(f"{'='*60}\n")

    async with httpx.AsyncClient(limits=limits, timeout=config.timeout) as client:
        start_time = time.perf_counter()
        
        tasks = [
            send_one(client, semaphore, config)
            for _ in range(config.total_requests)
        ]

        results = await tqdm_asyncio.gather(
            *tasks,
            desc="Sending requests",
            unit="req",
        )
        
        result.duration_seconds = time.perf_counter() - start_time

    # Process results
    latencies = []
    for status, latency in results:
        latencies.append(latency)
        result.status_codes[status] = result.status_codes.get(status, 0) + 1

    result.latencies = latencies
    result.accepted_count = result.status_codes.get(202, 0)
    result.backpressure_count = result.status_codes.get(503, 0)
    result.error_count = result.status_codes.get(0, 0)

    # Calculate throughput
    result.throughput_rps = result.accepted_count / result.duration_seconds

    # Calculate percentiles
    sorted_latencies = sorted(latencies)
    n = len(sorted_latencies)
    if n > 0:
        result.p50_latency_ms = sorted_latencies[int(n * 0.50)]
        result.p95_latency_ms = sorted_latencies[int(n * 0.95)]
        result.p99_latency_ms = sorted_latencies[int(n * 0.99)]

    # Measure memory after
    result.memory_after_mb = process.memory_info().rss / (1024 * 1024)

    return result


def print_results(result: BenchmarkResult) -> None:
    """Print benchmark results in a formatted table."""
    print(f"\n{'='*60}")
    print(f"Benchmark Results")
    print(f"{'='*60}")
    
    print(f"\n📊 Status Codes:")
    for code, count in sorted(result.status_codes.items()):
        if code == 202:
            label = "✅ 202 Accepted"
        elif code == 503:
            label = "⚠️  503 Backpressure"
        elif code == 401:
            label = "🔒 401 Unauthorized"
        elif code == 422:
            label = "❌ 422 Validation Failed"
        elif code == 0:
            label = "💥 Network Error"
        else:
            label = f"❓ {code}"
        print(f"  {label:30s} {count:8,}")

    print(f"\n📈 Throughput:")
    print(f"  Accepted:        {result.accepted_count:,} requests")
    print(f"  Duration:        {result.duration_seconds:.2f} seconds")
    print(f"  Throughput:      {result.throughput_rps:,.0f} req/sec")

    print(f"\n⏱️  Latency:")
    print(f"  P50:             {result.p50_latency_ms:.1f} ms")
    print(f"  P95:             {result.p95_latency_ms:.1f} ms")
    print(f"  P99:             {result.p99_latency_ms:.1f} ms")

    print(f"\n💾 Memory (client process):")
    print(f"  Before:          {result.memory_before_mb:.1f} MB")
    print(f"  After:           {result.memory_after_mb:.1f} MB")
    print(f"  Delta:           {result.memory_after_mb - result.memory_before_mb:+.1f} MB")

    print(f"\n🛡️  Backpressure:")
    print(f"  Queue Full (503): {result.backpressure_count:,}")
    if result.total_requests > 0:
        pct = (result.backpressure_count / result.total_requests) * 100
        print(f"  Backpressure Rate: {pct:.1f}%")

    print(f"\n{'='*60}\n")


def save_results(result: BenchmarkResult, output_path: str) -> None:
    """Save results to JSON file."""
    data = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "config": {
            "total_requests": result.total_requests,
            "concurrency": result.concurrency,
        },
        "results": {
            "throughput_rps": round(result.throughput_rps, 2),
            "duration_seconds": round(result.duration_seconds, 2),
            "accepted_count": result.accepted_count,
            "backpressure_count": result.backpressure_count,
            "error_count": result.error_count,
            "p50_latency_ms": round(result.p50_latency_ms, 2),
            "p95_latency_ms": round(result.p95_latency_ms, 2),
            "p99_latency_ms": round(result.p99_latency_ms, 2),
            "memory_before_mb": round(result.memory_before_mb, 1),
            "memory_after_mb": round(result.memory_after_mb, 1),
            "status_codes": result.status_codes,
        },
    }
    
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(data, f, indent=2)
    
    print(f"Results saved to: {output_path}")


async def main():
    parser = argparse.ArgumentParser(description="FastIngestor Load Test")
    parser.add_argument("--requests", type=int, default=10_000,
                        help="Total number of requests to send")
    parser.add_argument("--concurrency", type=int, default=200,
                        help="Number of concurrent requests")
    parser.add_argument("--url", type=str, default="http://localhost:8080/ingest",
                        help="Target URL")
    parser.add_argument("--api-key", type=str, default="benchmark-key",
                        help="API key for authentication")
    parser.add_argument("--output", type=str, default="benchmarks/results/latest.json",
                        help="Output file for results")
    args = parser.parse_args()

    config = BenchmarkConfig(
        url=args.url,
        api_key=args.api_key,
        total_requests=args.requests,
        concurrency=args.concurrency,
    )

    result = await run_benchmark(config)
    print_results(result)
    save_results(result, args.output)


if __name__ == "__main__":
    asyncio.run(main())