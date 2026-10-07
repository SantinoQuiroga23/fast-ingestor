class PyIngestor:
    def __init__(
        self,
        api_key: str,
        port: int = 8080,
        capacity: int = 100_000,
        multi_tenant: bool = False,
    ) -> None: ...

    def drain_batch(self) -> object: ...