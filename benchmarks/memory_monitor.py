"""
Monitor server memory usage during load test.
"""
import argparse
import time
import psutil
import sys


def find_python_server() -> psutil.Process:
    """Find the Python process running examples/demo.py."""
    for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
        try:
            if proc.info['name'] and 'python' in proc.info['name'].lower():
                cmdline = ' '.join(proc.info['cmdline'] or [])
                if 'demo.py' in cmdline or 'fast_ingestor' in cmdline:
                    return proc
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
    return None


def monitor_server(duration: int, interval: float):
    """Monitor memory of the FastIngestor server."""
    print("Looking for FastIngestor server...")
    proc = find_python_server()
    
    if proc is None:
        print("Server not found. Make sure examples/demo.py is running.")
        sys.exit(1)

    print(f"Found: PID {proc.pid}")
    print(f"Command: {' '.join(proc.cmdline())}")
    print(f"Monitoring for {duration} seconds...\n")
    
    samples = []
    start_time = time.time()
    
    try:
        while time.time() - start_time < duration:
            try:
                mem = proc.memory_info()
                mem_mb = mem.rss / (1024 * 1024)
                samples.append(mem_mb)
                elapsed = time.time() - start_time
                print(f"[{elapsed:6.1f}s] Memory: {mem_mb:8.1f} MB")
                time.sleep(interval)
            except psutil.NoSuchProcess:
                print("Server terminated")
                break
    except KeyboardInterrupt:
        pass

    if samples:
        print(f"\n{'='*60}")
        print(f"Memory Summary (FastIngestor Server)")
        print(f"{'='*60}")
        print(f"  Min:     {min(samples):8.1f} MB")
        print(f"  Max:     {max(samples):8.1f} MB")
        print(f"  Avg:     {sum(samples)/len(samples):8.1f} MB")
        print(f"  Samples: {len(samples)}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--duration", type=int, default=60)
    parser.add_argument("--interval", type=float, default=1.0)
    args = parser.parse_args()
    
    monitor_server(args.duration, args.interval)


if __name__ == "__main__":
    main()