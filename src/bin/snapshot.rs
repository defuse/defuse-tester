use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
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
                            // Normalize to elide meaningless differences
                            let normalized = normalize_html(&body);

                            // Create parent directories if needed (for slugs like "audits/ecryptfs")
                            if let Some(parent) = output_path.parent() {
                                std::fs::create_dir_all(parent)?;
                            }
                            std::fs::write(&output_path, &normalized).with_context(|| {
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

/// Normalize HTML to elide meaningless differences between prod and local snapshots.
fn normalize_html(html: &str) -> String {
    // Match lines with IPv4 address followed by </td> (IP address display)
    let ip_re = Regex::new(r"(?m)^\s*\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\s*</td>$").unwrap();
    let result = ip_re.replace_all(html, "                ELIDED_IP_ADDRESS                </td>");

    // Match lines with <td>NUMBER</td> (hit counts)
    let count_re = Regex::new(r"(?m)^\s*<td>\d+</td>$").unwrap();
    let result = count_re.replace_all(&result, "                <td>ELIDED_HIT_COUNT</td>");

    // Match lines with just whitespace and a number (upvote counts)
    let upvote_re = Regex::new(r"(?m)^\s+\d+\s*$").unwrap();
    let result = upvote_re.replace_all(&result, "            ELIDED_UPVOTE_COUNT ");

    // Normalize apostrophe encoding (&#039; and &#x27; are equivalent)
    let result = result.replace("&#039;", "&#x27;");

    // Normalize content div + upvotearrows (prod snapshots inconsistent)
    let result = result.replace("<div id=\"content\" >\n            <div class=\"upvotearrows\">",
                                "<div id=\"content\" >            <div class=\"upvotearrows\">");

    // Remove total site hits comment line
    let hits_re = Regex::new(r"(?m)^<!-- Total site hits: .* -->\n").unwrap();
    let result = hits_re.replace_all(&result, "");

    // Elide vim syntax highlighting lines that differ between vim versions
    let vim_syntax_re = Regex::new(r#"(?m)^.*<span class="(Identifier|Constant|Special|Statement|LineNr|Comment).*$"#).unwrap();
    let result = vim_syntax_re.replace_all(&result, "ELIDED_VIM_SYNTAX_LINE");

    // Elide "information" and "informaion" typo so contact page matches
    let result = result.replace("informaion", "ELIDED_INFO_WORD");
    let result = result.replace("information", "ELIDED_INFO_WORD");

    // Normalize JS hex escapes to literal chars
    // PHP's jse() escapes all non-alphanumeric chars, Rust doesn't need to
    let result = result.replace("\\x2D", "-");
    let result = result.replace("\\x5F", "_");

    // Normalize encfs audit URLs (old vs new naming)
    // Replace longer string first to avoid partial match
    let result = result.replace("auditencfsOLD", "ELIDED_ENCFS_AUDIT");
    let result = result.replace("auditencfs", "ELIDED_ENCFS_AUDIT");

    // Normalize absolute production URLs to relative URLs
    // Production PHP uses full URLs like "https://defuse.ca:443/page.htm"
    // Rust uses relative URLs like "/page.htm"
    result.replace("https://defuse.ca:443", "")
}
