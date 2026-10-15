use futures::future::join_all;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

const URL: &str = "http://localhost:8080/ingest";

fn get_api_key() -> String {
    std::env::var("INGESTOR_API_KEY").unwrap_or_else(|_| "my-secret-key".to_string())
}

const METRIC_TYPES: [&str; 3] = ["temperature", "humidity", "pressure"];

/// Generate a single telemetry record.
fn generate_record(seq: u64) -> serde_json::Value {
    let metric_type = METRIC_TYPES[seq as usize % 3];
    
    json!({
        "timestamp": 1700000000000_i64 + (seq as i64 * 1000),
        "device_id": format!("sensor_{:04}", seq % 1000),
        "metric_type": metric_type,
        "value": (seq % 100) as f64 * 1.5,
    })
}

/// Generate a batch of records.
fn generate_batch(batch_size: usize, start_seq: u64) -> serde_json::Value {
    let records: Vec<serde_json::Value> = (0..batch_size)
        .map(|i| generate_record(start_seq + i as u64))
        .collect();
    serde_json::Value::Array(records)
}

/// Worker that sends batches of requests.
async fn worker(
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,
    worker_id: usize,
    num_requests: usize,
    batch_size: usize,
) -> (usize, usize, usize) {
    let mut success = 0;
    let mut backpressure = 0;
    let mut failed = 0;
    
    let start_seq = (worker_id * num_requests * batch_size) as u64;
    
    for i in 0..num_requests {
        let _permit = semaphore.acquire().await.unwrap();
        
        let batch = generate_batch(batch_size, start_seq + (i * batch_size) as u64);
        
        match client
            .post(URL)
            .header("X-API-Key", get_api_key())
            .header("Content-Type", "application/json")
            .json(&batch)
            .send()
            .await
        {
            Ok(response) => {
                match response.status().as_u16() {
                    202 => success += 1,
                    503 => backpressure += 1,
                    _ => failed += 1,
                }
            }
            Err(_) => failed += 1,
        }
    }
    
    (success, backpressure, failed)
}

#[tokio::main]
async fn main() {
    let total_requests = std::env::var("TOTAL_REQUESTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);
    
    let batch_size = std::env::var("BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    
    let num_workers = std::env::var("NUM_WORKERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);
    
    let concurrency = std::env::var("CONCURRENCY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);
    
    println!("{}", "=".repeat(70));
    println!("FastIngestor Rust Benchmark");
    println!("{}", "=".repeat(70));
    println!("  Total Requests:  {}", total_requests);
    println!("  Batch Size:      {} records/request", batch_size);
    println!("  Total Records:   {}", total_requests * batch_size);
    println!("  Workers:         {}", num_workers);
    println!("  Concurrency:     {}", concurrency);
    println!("  Target:          {}", URL);
    println!("{}", "=".repeat(70));
    println!();
    
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(concurrency)
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to build HTTP client");
    
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let requests_per_worker = total_requests / num_workers;
    
    let start = Instant::now();
    
    let handles: Vec<JoinHandle<(usize, usize, usize)>> = (0..num_workers)
        .map(|worker_id| {
            let client = client.clone();
            let sem = semaphore.clone();
            tokio::spawn(async move {
                worker(client, sem, worker_id, requests_per_worker, batch_size).await
            })
        })
        .collect();
    
    let results = join_all(handles).await;
    
    let elapsed = start.elapsed();
    
    let mut total_success = 0;
    let mut total_backpressure = 0;
    let mut total_failed = 0;
    
    for result in results {
        if let Ok((success, backpressure, failed)) = result {
            total_success += success;
            total_backpressure += backpressure;
            total_failed += failed;
        }
    }
    
    let total_records = total_success * batch_size;
    let request_throughput = total_success as f64 / elapsed.as_secs_f64();
    let record_throughput = total_records as f64 / elapsed.as_secs_f64();
    
    println!();
    println!("{}", "=".repeat(70));
    println!("Results");
    println!("{}", "=".repeat(70));
    println!("  ✅ Successful Requests:  {}", total_success);
    println!("  ⚠️  Backpressure (503):   {}", total_backpressure);
    println!("  ❌ Failed Requests:      {}", total_failed);
    println!("  📊 Total Records Sent:   {}", total_records);
    println!("  ⏱️  Duration:            {:.2}s", elapsed.as_secs_f64());
    println!();
    println!("  📈 Request Throughput:   {:.0} req/sec", request_throughput);
    println!("  📈 Record Throughput:    {:.0} records/sec", record_throughput);
    println!("{}", "=".repeat(70));
}