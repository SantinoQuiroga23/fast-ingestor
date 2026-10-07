import os
import time

from fast_ingestor import FastIngestor


def main() -> None:
    api_key = os.environ.get("INGESTOR_API_KEY", "my-secret-key")

    ingestor = FastIngestor(
        api_key=api_key,
        port=8080,
        capacity=100_000,
        multi_tenant=False,
    )

    print("FastIngestor listening on http://0.0.0.0:8080")
    print("Press Ctrl+C to stop.")

    while True:
        time.sleep(2)
        df = ingestor.drain_batch()
        if not df.is_empty():
            print(f"\nDrained {len(df)} rows:")
            print(df)


if __name__ == "__main__":
    main()