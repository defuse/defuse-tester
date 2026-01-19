//! Stress test binary for defuse-rust server.
//!
//! Measures server throughput under load by:
//! (a) Running 100 parallel checksums requests (2MB files) while measuring
//!     throughput of /blind-birthday-attack.htm
//! (b) Measuring baseline throughput of /blind-birthday-attack.htm without load

use clap::Parser;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Stress test for defuse-rust server
#[derive(Parser)]
#[command(name = "stress")]
#[command(about = "Run stress tests against defuse-rust server")]
struct Args {
    /// Base URL of the server (e.g., http://localhost:3000)
    base_url: String,

    /// Duration of each test in seconds
    #[arg(short, long, default_value = "10")]
    duration: u64,

    /// Number of parallel checksums requests for the load test
    #[arg(short, long, default_value = "100")]
    parallel: usize,

    /// Size of files to hash in MB
    #[arg(short, long, default_value = "2")]
    file_size_mb: usize,
}

/// Create an HTTP client with appropriate settings for stress testing
fn create_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(120))
        .pool_max_idle_per_host(200)
        .build()
        .expect("Failed to create HTTP client")
}

/// Continuously POST files to /checksums.htm until stop signal.
/// Each worker keeps sending requests in a loop until stopped.
async fn checksums_worker(
    client: Client,
    base_url: String,
    file_content: Arc<Vec<u8>>,
    stop: Arc<AtomicBool>,
    completed: Arc<AtomicU64>,
) {
    let url = format!("{}/checksums.htm", base_url);

    while !stop.load(Ordering::Relaxed) {
        let part = Part::bytes(file_content.as_ref().clone())
            .file_name("stress_test.bin".to_string())
            .mime_str("application/octet-stream")
            .unwrap();

        let form = Form::new().part("filetohash".to_string(), part);

        match client.post(&url).multipart(form).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    // Consume the body to complete the request
                    let _ = resp.bytes().await;
                    completed.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(e) => {
                if !stop.load(Ordering::Relaxed) {
                    eprintln!("Checksums request failed: {}", e);
                }
            }
        }
    }
}

/// Continuously GET /blind-birthday-attack.htm and count successes
async fn throughput_worker(
    client: Client,
    base_url: String,
    stop: Arc<AtomicBool>,
    completed: Arc<AtomicU64>,
) {
    let url = format!("{}/blind-birthday-attack.htm", base_url);

    while !stop.load(Ordering::Relaxed) {
        match client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    // Consume the body to complete the request
                    let _ = resp.bytes().await;
                    completed.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(e) => {
                if !stop.load(Ordering::Relaxed) {
                    eprintln!("Throughput request failed: {}", e);
                }
            }
        }
    }
}

/// Run the baseline throughput test (no checksums load)
async fn run_baseline_test(base_url: &str, duration_secs: u64) -> f64 {
    println!("\n=== Test B: Baseline Throughput (no load) ===");
    println!("Duration: {} seconds", duration_secs);
    println!("Measuring throughput of /blind-birthday-attack.htm...\n");

    let client = create_client();
    let stop = Arc::new(AtomicBool::new(false));
    let completed = Arc::new(AtomicU64::new(0));

    // Spawn multiple throughput workers for maximum throughput measurement
    let num_workers = 10;
    let mut handles = Vec::new();

    for _ in 0..num_workers {
        let client = client.clone();
        let base_url = base_url.to_string();
        let stop = stop.clone();
        let completed = completed.clone();

        handles.push(tokio::spawn(async move {
            throughput_worker(client, base_url, stop, completed).await;
        }));
    }

    let start = Instant::now();

    // Wait for test duration
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;

    // Signal stop
    stop.store(true, Ordering::Relaxed);

    // Wait for workers to finish
    for handle in handles {
        let _ = handle.await;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let total_requests = completed.load(Ordering::Relaxed);
    let requests_per_sec = total_requests as f64 / elapsed;

    println!("Completed {} requests in {:.2}s", total_requests, elapsed);
    println!("Throughput: {:.2} requests/second", requests_per_sec);

    requests_per_sec
}

/// Run the loaded throughput test (with parallel checksums requests)
async fn run_loaded_test(
    base_url: &str,
    duration_secs: u64,
    parallel_checksums: usize,
    file_size_mb: usize,
) -> (f64, u64) {
    println!("\n=== Test A: Throughput Under Load ===");
    println!("Duration: {} seconds", duration_secs);
    println!(
        "Load: {} parallel threads continuously hashing {}MB files",
        parallel_checksums, file_size_mb
    );
    println!("Measuring throughput of /blind-birthday-attack.htm...\n");

    let client = create_client();
    let stop = Arc::new(AtomicBool::new(false));
    let throughput_completed = Arc::new(AtomicU64::new(0));
    let checksums_completed = Arc::new(AtomicU64::new(0));

    // Create file content (reuse across workers)
    let file_content = Arc::new(vec![0u8; file_size_mb * 1024 * 1024]);

    let mut handles = Vec::new();

    // Spawn exactly `parallel_checksums` workers, each looping until stopped
    for _ in 0..parallel_checksums {
        let client = client.clone();
        let base_url = base_url.to_string();
        let file_content = file_content.clone();
        let stop = stop.clone();
        let completed = checksums_completed.clone();

        handles.push(tokio::spawn(async move {
            checksums_worker(client, base_url, file_content, stop, completed).await;
        }));
    }

    // Spawn throughput workers
    let num_throughput_workers = 10;
    for _ in 0..num_throughput_workers {
        let client = client.clone();
        let base_url = base_url.to_string();
        let stop = stop.clone();
        let completed = throughput_completed.clone();

        handles.push(tokio::spawn(async move {
            throughput_worker(client, base_url, stop, completed).await;
        }));
    }

    let start = Instant::now();

    // Wait for test duration
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;

    // Signal stop
    stop.store(true, Ordering::Relaxed);

    // Wait for workers to finish
    for handle in handles {
        let _ = handle.await;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let total_throughput_requests = throughput_completed.load(Ordering::Relaxed);
    let total_checksums_requests = checksums_completed.load(Ordering::Relaxed);
    let requests_per_sec = total_throughput_requests as f64 / elapsed;

    println!(
        "Throughput requests: {} in {:.2}s ({:.2} req/s)",
        total_throughput_requests, elapsed, requests_per_sec
    );
    println!(
        "Checksums requests completed: {} ({:.2} req/s)",
        total_checksums_requests,
        total_checksums_requests as f64 / elapsed
    );

    (requests_per_sec, total_checksums_requests)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Normalize base URL (remove trailing slash)
    let base_url = args.base_url.trim_end_matches('/');

    println!("Defuse Stress Test");
    println!("==================");
    println!("Target: {}", base_url);
    println!("Test duration: {} seconds each", args.duration);
    println!(
        "Checksums load: {} parallel threads with {}MB files",
        args.parallel, args.file_size_mb
    );

    // Run baseline test first
    let baseline_throughput = run_baseline_test(base_url, args.duration).await;

    // Run loaded test
    let (loaded_throughput, checksums_completed) =
        run_loaded_test(base_url, args.duration, args.parallel, args.file_size_mb).await;

    // Summary
    println!("\n=== Summary ===");
    println!(
        "Baseline throughput:     {:.2} requests/second",
        baseline_throughput
    );
    println!(
        "Loaded throughput:       {:.2} requests/second",
        loaded_throughput
    );

    if baseline_throughput > 0.0 {
        let ratio = loaded_throughput / baseline_throughput;
        let degradation = (1.0 - ratio) * 100.0;
        println!(
            "Throughput ratio:        {:.2}x ({:.1}% degradation)",
            ratio, degradation
        );
    }

    println!("Checksums completed:     {}", checksums_completed);
}
