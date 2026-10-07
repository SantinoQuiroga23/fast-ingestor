import asyncio
import httpx
import polars as pl
from tqdm.asyncio import tqdm_asyncio

URL = "http://localhost:8080/ingest"
BATCH_SIZE = 10_000   # 10k registros por request
TOTAL_BATCHES = 10    # Total: 100k registros

# 1. Generación ultrarrápida del lote repetitivo en Polars
df_batch = pl.DataFrame({
    "device_id": ["sensor_01"] * BATCH_SIZE,
    "temperature": [23.5] * BATCH_SIZE,
    "humidity": [60.0] * BATCH_SIZE
})

# Convertir el lote a lista de diccionarios (JSON payload)
payload = df_batch.to_dicts()

async def send_batch(client: httpx.AsyncClient, semaphore: asyncio.Semaphore):
    async with semaphore:
        try:
            response = await client.post(URL, json=payload)
            return response.status_code
        except Exception as e:
            return None

async def main():
    semaphore = asyncio.Semaphore(5)  # Máximo 5 lotes concurrentes en vuelo
    
    limits = httpx.Limits(max_keepalive_connections=5, max_connections=5)
    
    async with httpx.AsyncClient(limits=limits, timeout=10.0) as client:
        tasks = [send_batch(client, semaphore) for _ in range(TOTAL_BATCHES)]
        
        await tqdm_asyncio.gather(
            *tasks,
            desc=f"Enviando {TOTAL_BATCHES} lotes de {BATCH_SIZE} registros",
            unit="lote"
        )

if __name__ == "__main__":
    asyncio.run(main())