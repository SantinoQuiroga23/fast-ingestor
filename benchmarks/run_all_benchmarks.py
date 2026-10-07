"""
Run multiple benchmark scenarios and generate a comparison report.

Usage:
    python benchmarks/run_all_benchmarks.py
"""
import asyncio
import json
import os
import time
from datetime import datetime

from load_test import BenchmarkConfig, run_benchmark, print_results


SCENARIOS = [
    {"name": "Low Load", "requests": 1_000, "concurrency": 50},
    {"name": "Medium Load", "requests": 5_000, "concurrency": 200},
    {"name": "High Load", "requests": 10_000, "concurrency": 500},
    {"name": "Stress Test", "requests": 25_000, "concurrency": 1000},
]


async def run_all():
    results = []
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_dir = f"benchmarks/results/{timestamp}"
    os.makedirs(output_dir, exist_ok=True)

    print(f"\n{'#'*60}")
    print(f"# FastIngestor Benchmark Suite")
    print(f"# Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"{'#'*60}\n")

    for i, scenario in enumerate(SCENARIOS, 1):
        print(f"\n[{i}/{len(SCENARIOS)}] Running: {scenario['name']}")
        print(f"  Requests: {scenario['requests']:,}")
        print(f"  Concurrency: {scenario['concurrency']}")

        config = BenchmarkConfig(
            total_requests=scenario["requests"],
            concurrency=scenario["concurrency"],
        )

        result = await run_benchmark(config)
        result.scenario_name = scenario["name"]
        results.append(result)

        # Save individual result
        output_path = os.path.join(output_dir, f"scenario_{i}_{scenario['name'].lower().replace(' ', '_')}.json")
        with open(output_path, "w") as f:
            json.dump({
                "scenario": scenario,
                "throughput_rps": result.throughput_rps,
                "p50_ms": result.p50_latency_ms,
                "p95_ms": result.p95_latency_ms,
                "p99_ms": result.p99_latency_ms,
                "accepted": result.accepted_count,
                "backpressure": result.backpressure_count,
            }, f, indent=2)

        # Pause between scenarios
        if i < len(SCENARIOS):
            print("\n  Waiting 5 seconds before next scenario...")
            await asyncio.sleep(5)

    # Generate summary report
    generate_report(results, output_dir)


def generate_report(results, output_dir):
    """Generate a markdown report from benchmark results."""
    report_path = os.path.join(output_dir, "REPORT.md")
    
    with open(report_path, "w") as f:
        f.write("# FastIngestor Benchmark Report\n\n")
        f.write(f"**Date:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
        f.write("## Test Environment\n\n")
        f.write("- **CPU:** [Fill in your CPU model]\n")
        f.write("- **RAM:** [Fill in your RAM]\n")
        f.write("- **OS:** [Fill in your OS]\n")
        f.write("- **Python:** [Fill in Python version]\n")
        f.write("- **Rust:** [Fill in Rust version]\n\n")
        
        f.write("## Results Summary\n\n")
        f.write("| Scenario | Requests | Concurrency | Throughput | P50 | P95 | P99 | Backpressure |\n")
        f.write("|----------|----------|-------------|------------|-----|-----|-----|--------------|\n")
        
        for r in results:
            f.write(f"| {r.scenario_name} | {r.total_requests:,} | {r.concurrency} | "
                    f"{r.throughput_rps:,.0f} req/s | {r.p50_latency_ms:.1f}ms | "
                    f"{r.p95_latency_ms:.1f}ms | {r.p99_latency_ms:.1f}ms | "
                    f"{r.backpressure_count:,} |\n")
        
        f.write("\n## Analysis\n\n")
        f.write("[Add your analysis here]\n")
    
    print(f"\nReport saved to: {report_path}")


if __name__ == "__main__":
    asyncio.run(run_all())