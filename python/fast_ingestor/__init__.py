from typing import TYPE_CHECKING

import polars as pl

if TYPE_CHECKING:
    from ._lib_ingestor import PyIngestor as _PyIngestor
else:
    from ._lib_ingestor import PyIngestor as _PyIngestor


class FastIngestor:
    """High-throughput ingestion server backed by Rust.

    Args:
        api_key: Required API key for authentication
        port: TCP port to listen on (default: 8080)
        capacity: Maximum buffered records before 503 (default: 100_000)
        multi_tenant: Require tenant_id in payloads (default: False)

    Example:
        >>> ingestor = FastIngestor(api_key="secret", port=8080)
        >>> # Send HTTP POST to http://localhost:8080/ingest
        >>> df = ingestor.drain_batch()
        >>> print(df)
    """

    def __init__(
        self,
        api_key: str,
        port: int = 8080,
        capacity: int = 100_000,
        multi_tenant: bool = False,
    ) -> None:
        self._rust_ingestor = _PyIngestor(
            api_key=api_key,
            port=port,
            capacity=capacity,
            multi_tenant=multi_tenant,
        )

    def drain_batch(self) -> pl.DataFrame:
        """Drain buffered rows into a Polars DataFrame.

        Returns an empty DataFrame when no data is available.
        The conversion is zero-copy via Arrow C Data Interface.
        """
        try:
            record_batch = self._rust_ingestor.drain_batch()
            return pl.from_arrow(record_batch)
        except ValueError:
            return pl.DataFrame()