use anyhow::{Context, Result};
use clap::Parser;
use reqwest::Client;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "snapshot")]
#[command(about = "Capture page snapshots from a server for diff-based comparison")]
struct Args {
    /// Base URL of the server (e.g., https://defuse.ca/ or http://localhost:3000/)
    base_url: String,

    /// Output directory for snapshot files
    output_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct PageInfo {
    file: Option<String>,
    redirect: Option<String>,
    #[allow(dead_code)]
    title: Option<String>,
    #[allow(dead_code)]
    description: Option<String>,
    #[allow(dead_code)]
    keywords: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Ensure base URL ends with /
    let base_url = if args.base_url.ends_with('/') {
        args.base_url
    } else {
        format!("{}/", args.base_url)
    };

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("Failed to create output directory: {:?}", args.output_dir))?;

    // Load pages.json
    let pages_json = std::fs::read_to_string("data/pages.json")
        .context("Failed to read data/pages.json")?;
    let pages: BTreeMap<String, PageInfo> =
        serde_json::from_str(&pages_json).context("Failed to parse pages.json")?;

    // Filter to real pages (has file, no redirect) and sort by slug
    let mut real_pages: Vec<_> = pages
        .into_iter()
        .filter(|(_, info)| info.file.is_some() && info.redirect.is_none())
        .collect();
    real_pages.sort_by(|(a, _), (b, _)| a.cmp(b));

    println!("Found {} pages to snapshot", real_pages.len());

    let client = Client::builder().build()?;

    let mut success_count = 0;
    let mut error_count = 0;

    for (slug, _info) in &real_pages {
        let url = format!("{}{}", base_url, slug);
        let filename = if slug.is_empty() {
            "_index.html".to_string()
        } else {
            format!("{}.html", slug.trim_end_matches('/'))
        };
        let output_path = args.output_dir.join(&filename);

        print!("Fetching {} ... ", slug);

        match client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    match response.text().await {
                        Ok(body) => {
                            // Create parent directories if needed (for slugs like "audits/ecryptfs")
                            if let Some(parent) = output_path.parent() {
                                std::fs::create_dir_all(parent)?;
                            }
                            std::fs::write(&output_path, &body).with_context(|| {
                                format!("Failed to write {:?}", output_path)
                            })?;
                            println!("OK ({} bytes)", body.len());
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("ERROR reading body: {}", e);
                            error_count += 1;
                        }
                    }
                } else {
                    println!("HTTP {}", status);
                    error_count += 1;
                }
            }
            Err(e) => {
                println!("ERROR: {}", e);
                error_count += 1;
            }
        }
    }

    println!("\nComplete: {} succeeded, {} failed", success_count, error_count);

    Ok(())
}
