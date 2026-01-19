//! Stress test binary for defuse-rust server.
//!
//! Measures server throughput under load by:
//! (a) Running 100 parallel checksums requests (2MB files) while measuring
//!     throughput of /blind-birthday-attack.htm
//! (b) Measuring baseline throughput of /blind-birthday-attack.htm without load

use clap::Parser;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use std::error::Error;
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

    /// Number of parallel workers for throughput measurement
    #[arg(short, long, default_value = "10")]
    throughput_workers: usize,
}

/// Response time statistics
struct ResponseStats {
    count: u64,
    min_ms: f64,
    max_ms: f64,
    avg_ms: f64,
    median_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
}

impl ResponseStats {
    fn from_times(times: &mut [f64]) -> Option<Self> {
        if times.is_empty() {
            return None;
        }

        times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = times.len() as u64;
        let min_ms = times[0];
        let max_ms = times[times.len() - 1];
        let avg_ms = times.iter().sum::<f64>() / times.len() as f64;

        let median_ms = if times.len() % 2 == 0 {
            (times[times.len() / 2 - 1] + times[times.len() / 2]) / 2.0
        } else {
            times[times.len() / 2]
        };

        let p95_idx = ((times.len() as f64) * 0.95) as usize;
        let p95_ms = times[p95_idx.min(times.len() - 1)];

        let p99_idx = ((times.len() as f64) * 0.99) as usize;
        let p99_ms = times[p99_idx.min(times.len() - 1)];

        Some(Self {
            count,
            min_ms,
            max_ms,
            avg_ms,
            median_ms,
            p95_ms,
            p99_ms,
        })
    }

    fn print(&self, label: &str) {
        println!("{} Response Times:", label);
        println!("  Count:  {}", self.count);
        println!("  Min:    {:.2} ms", self.min_ms);
        println!("  Max:    {:.2} ms", self.max_ms);
        println!("  Avg:    {:.2} ms", self.avg_ms);
        println!("  Median: {:.2} ms", self.median_ms);
        println!("  P95:    {:.2} ms", self.p95_ms);
        println!("  P99:    {:.2} ms", self.p99_ms);
    }
}

/// Print detailed error information for a failed request
fn print_request_error(label: &str, url: &str, e: &reqwest::Error) {
    let mut details = Vec::new();

    if e.is_timeout() {
        details.push("timeout".to_string());
    }
    if e.is_connect() {
        details.push("connection failed".to_string());
    }
    if e.is_request() {
        details.push("request error".to_string());
    }
    if e.is_body() {
        details.push("body error".to_string());
    }
    if e.is_decode() {
        details.push("decode error".to_string());
    }
    if let Some(status) = e.status() {
        details.push(format!("status {}", status));
    }

    // Get the source chain
    let mut source_chain = Vec::new();
    let mut current: Option<&(dyn Error + 'static)> = e.source();
    while let Some(source) = current {
        source_chain.push(format!("{}", source));
        current = source.source();
    }

    let details_str = if details.is_empty() {
        String::new()
    } else {
        format!(" [{}]", details.join(", "))
    };

    let source_str = if source_chain.is_empty() {
        String::new()
    } else {
        format!(" caused by: {}", source_chain.join(" -> "))
    };

    eprintln!("{} request failed{}: {}{}", label, details_str, url, source_str);
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
    failed: Arc<AtomicU64>,
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
                } else {
                    failed.fetch_add(1, Ordering::Relaxed);
                    if !stop.load(Ordering::Relaxed) {
                        eprintln!("Checksums request returned {}: {}", resp.status(), url);
                    }
                }
            }
            Err(e) => {
                failed.fetch_add(1, Ordering::Relaxed);
                if !stop.load(Ordering::Relaxed) {
                    print_request_error("Checksums", &url, &e);
                }
            }
        }
    }
}

/// Continuously GET /blind-birthday-attack.htm, count successes, and collect response times.
/// Returns the collected response times in milliseconds.
async fn throughput_worker(
    client: Client,
    base_url: String,
    stop: Arc<AtomicBool>,
    completed: Arc<AtomicU64>,
    failed: Arc<AtomicU64>,
) -> Vec<f64> {
    let url = format!("{}/blind-birthday-attack.htm", base_url);
    let mut response_times = Vec::new();

    while !stop.load(Ordering::Relaxed) {
        let start = Instant::now();
        match client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    // Consume the body to complete the request
                    let _ = resp.bytes().await;
                    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                    response_times.push(elapsed_ms);
                    completed.fetch_add(1, Ordering::Relaxed);
                } else {
                    failed.fetch_add(1, Ordering::Relaxed);
                    if !stop.load(Ordering::Relaxed) {
                        eprintln!("Throughput request returned {}: {}", resp.status(), url);
                    }
                }
            }
            Err(e) => {
                failed.fetch_add(1, Ordering::Relaxed);
                if !stop.load(Ordering::Relaxed) {
                    print_request_error("Throughput", &url, &e);
                }
            }
        }
    }

    response_times
}

/// Run the baseline throughput test (no checksums load)
/// Returns (requests_per_sec, stats, failed_count)
async fn run_baseline_test(
    base_url: &str,
    duration_secs: u64,
    throughput_workers: usize,
) -> (f64, Option<ResponseStats>, u64) {
    println!("\n=== Test B: Baseline Throughput (no load) ===");
    println!("Duration: {} seconds", duration_secs);
    println!("Throughput workers: {}", throughput_workers);
    println!("Measuring throughput of /blind-birthday-attack.htm...\n");

    let client = create_client();
    let stop = Arc::new(AtomicBool::new(false));
    let completed = Arc::new(AtomicU64::new(0));
    let failed = Arc::new(AtomicU64::new(0));

    let num_workers = throughput_workers;
    let mut handles = Vec::new();

    for _ in 0..num_workers {
        let client = client.clone();
        let base_url = base_url.to_string();
        let stop = stop.clone();
        let completed = completed.clone();
        let failed = failed.clone();

        handles.push(tokio::spawn(async move {
            throughput_worker(client, base_url, stop, completed, failed).await
        }));
    }

    let start = Instant::now();

    // Wait for test duration
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;

    // Signal stop
    stop.store(true, Ordering::Relaxed);

    // Collect response times from all workers
    let mut all_times = Vec::new();
    for handle in handles {
        if let Ok(times) = handle.await {
            all_times.extend(times);
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    let total_requests = completed.load(Ordering::Relaxed);
    let total_failed = failed.load(Ordering::Relaxed);
    let requests_per_sec = total_requests as f64 / elapsed;

    println!("Completed {} requests in {:.2}s", total_requests, elapsed);
    println!("Throughput: {:.2} requests/second", requests_per_sec);
    if total_failed > 0 {
        println!("Failed: {}", total_failed);
    }

    let stats = ResponseStats::from_times(&mut all_times);
    if let Some(ref s) = stats {
        println!();
        s.print("Baseline");
    }

    (requests_per_sec, stats, total_failed)
}

/// Run the loaded throughput test (with parallel checksums requests)
/// Returns (requests_per_sec, checksums_completed, stats, total_failed)
async fn run_loaded_test(
    base_url: &str,
    duration_secs: u64,
    parallel_checksums: usize,
    file_size_mb: usize,
    throughput_workers: usize,
) -> (f64, u64, Option<ResponseStats>, u64) {
    println!("\n=== Test A: Throughput Under Load ===");
    println!("Duration: {} seconds", duration_secs);
    println!(
        "Load: {} parallel threads continuously hashing {}MB files",
        parallel_checksums, file_size_mb
    );
    println!("Throughput workers: {}", throughput_workers);
    println!("Measuring throughput of /blind-birthday-attack.htm...\n");

    let client = create_client();
    let stop = Arc::new(AtomicBool::new(false));
    let throughput_completed = Arc::new(AtomicU64::new(0));
    let checksums_completed = Arc::new(AtomicU64::new(0));
    let throughput_failed = Arc::new(AtomicU64::new(0));
    let checksums_failed = Arc::new(AtomicU64::new(0));

    // Create file content (reuse across workers)
    let file_content = Arc::new(vec![0u8; file_size_mb * 1024 * 1024]);

    let mut checksums_handles = Vec::new();
    let mut throughput_handles = Vec::new();

    // Spawn exactly `parallel_checksums` workers, each looping until stopped
    for _ in 0..parallel_checksums {
        let client = client.clone();
        let base_url = base_url.to_string();
        let file_content = file_content.clone();
        let stop = stop.clone();
        let completed = checksums_completed.clone();
        let failed = checksums_failed.clone();

        checksums_handles.push(tokio::spawn(async move {
            checksums_worker(client, base_url, file_content, stop, completed, failed).await;
        }));
    }

    // Spawn throughput workers
    for _ in 0..throughput_workers {
        let client = client.clone();
        let base_url = base_url.to_string();
        let stop = stop.clone();
        let completed = throughput_completed.clone();
        let failed = throughput_failed.clone();

        throughput_handles.push(tokio::spawn(async move {
            throughput_worker(client, base_url, stop, completed, failed).await
        }));
    }

    let start = Instant::now();

    // Wait for test duration
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;

    // Signal stop
    stop.store(true, Ordering::Relaxed);

    // Wait for checksums workers to finish
    for handle in checksums_handles {
        let _ = handle.await;
    }

    // Collect response times from throughput workers
    let mut all_times = Vec::new();
    for handle in throughput_handles {
        if let Ok(times) = handle.await {
            all_times.extend(times);
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    let total_throughput_requests = throughput_completed.load(Ordering::Relaxed);
    let total_checksums_requests = checksums_completed.load(Ordering::Relaxed);
    let total_throughput_failed = throughput_failed.load(Ordering::Relaxed);
    let total_checksums_failed = checksums_failed.load(Ordering::Relaxed);
    let total_failed = total_throughput_failed + total_checksums_failed;
    let total_requests = total_throughput_requests + total_checksums_requests;
    let requests_per_sec = total_requests as f64 / elapsed;

    println!(
        "Page requests:      {} ({:.2} req/s)",
        total_throughput_requests,
        total_throughput_requests as f64 / elapsed
    );
    println!(
        "Checksums requests: {} ({:.2} req/s)",
        total_checksums_requests,
        total_checksums_requests as f64 / elapsed
    );
    println!(
        "Total throughput:   {} requests in {:.2}s ({:.2} req/s)",
        total_requests, elapsed, requests_per_sec
    );
    if total_failed > 0 {
        println!(
            "Failed requests:    {} (page: {}, checksums: {})",
            total_failed, total_throughput_failed, total_checksums_failed
        );
    }

    let stats = ResponseStats::from_times(&mut all_times);
    if let Some(ref s) = stats {
        println!();
        s.print("Under Load");
    }

    (requests_per_sec, total_checksums_requests, stats, total_failed)
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
    let (baseline_throughput, baseline_stats, baseline_failed) =
        run_baseline_test(base_url, args.duration, args.throughput_workers).await;

    // Run loaded test
    let (loaded_throughput, checksums_completed, loaded_stats, loaded_failed) = run_loaded_test(
        base_url,
        args.duration,
        args.parallel,
        args.file_size_mb,
        args.throughput_workers,
    )
    .await;

    let total_failed = baseline_failed + loaded_failed;

    // Summary
    println!("\n=== Summary ===");
    println!(
        "Baseline throughput:           {:.2} requests/second",
        baseline_throughput
    );
    println!(
        "Loaded throughput (w/ checksums): {:.2} requests/second",
        loaded_throughput
    );

    if baseline_throughput > 0.0 {
        let ratio = loaded_throughput / baseline_throughput;
        let degradation = (1.0 - ratio) * 100.0;
        println!(
            "Throughput ratio:              {:.2}x ({:.1}% degradation)",
            ratio, degradation
        );
    }

    println!("Checksums completed:           {}", checksums_completed);
    println!("Total failed requests:         {}", total_failed);

    // Response time comparison
    if let (Some(baseline), Some(loaded)) = (baseline_stats, loaded_stats) {
        println!("\n=== Response Time Comparison ===");
        println!(
            "                    Baseline    Under Load    Change"
        );
        println!(
            "  Min:              {:>7.2} ms  {:>7.2} ms    {:>+.1}%",
            baseline.min_ms,
            loaded.min_ms,
            (loaded.min_ms / baseline.min_ms - 1.0) * 100.0
        );
        println!(
            "  Avg:              {:>7.2} ms  {:>7.2} ms    {:>+.1}%",
            baseline.avg_ms,
            loaded.avg_ms,
            (loaded.avg_ms / baseline.avg_ms - 1.0) * 100.0
        );
        println!(
            "  Median:           {:>7.2} ms  {:>7.2} ms    {:>+.1}%",
            baseline.median_ms,
            loaded.median_ms,
            (loaded.median_ms / baseline.median_ms - 1.0) * 100.0
        );
        println!(
            "  P95:              {:>7.2} ms  {:>7.2} ms    {:>+.1}%",
            baseline.p95_ms,
            loaded.p95_ms,
            (loaded.p95_ms / baseline.p95_ms - 1.0) * 100.0
        );
        println!(
            "  P99:              {:>7.2} ms  {:>7.2} ms    {:>+.1}%",
            baseline.p99_ms,
            loaded.p99_ms,
            (loaded.p99_ms / baseline.p99_ms - 1.0) * 100.0
        );
        println!(
            "  Max:              {:>7.2} ms  {:>7.2} ms    {:>+.1}%",
            baseline.max_ms,
            loaded.max_ms,
            (loaded.max_ms / baseline.max_ms - 1.0) * 100.0
        );
    }
}
