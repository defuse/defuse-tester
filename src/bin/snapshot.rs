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

    /// Skip elision of dynamic content (hit counts, IPs, etc.)
    #[arg(long)]
    no_elision: bool,

    /// Normalize vim syntax highlighting tags within vimhighlight sections.
    /// Strips whitelisted span tags (Identifier, Constant, etc.) but preserves text content.
    /// Use this to compare source code content across different vim versions.
    #[arg(long)]
    normalize_vim_tags: bool,
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
    let mut failures: Vec<(String, String)> = Vec::new();

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
                            // Normalize to elide meaningless differences (unless --no-elision)
                            let mut normalized = if args.no_elision {
                                body.clone()
                            } else {
                                normalize_html(&body, !args.normalize_vim_tags)
                            };

                            // Optionally normalize vim syntax tags
                            if args.normalize_vim_tags {
                                normalized = normalize_vim_syntax_tags(&normalized);
                            }

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
                            let err_msg = format!("ERROR reading body: {}", e);
                            println!("{}", err_msg);
                            failures.push((slug.clone(), err_msg));
                        }
                    }
                } else {
                    let err_msg = format!("HTTP {}", status);
                    println!("{}", err_msg);
                    failures.push((slug.clone(), err_msg));
                }
            }
            Err(e) => {
                let err_msg = format!("ERROR: {}", e);
                println!("{}", err_msg);
                failures.push((slug.clone(), err_msg));
            }
        }
    }

    if !failures.is_empty() {
        println!();
        for (slug, err) in &failures {
            println!("FAILURE: {} - {}", slug, err);
        }
    }

    println!("\nComplete: {} succeeded, {} failed", success_count, failures.len());

    Ok(())
}

/// Normalize HTML to elide meaningless differences between prod and local snapshots.
/// When `elide_vim_tags` is true, replace entire vim syntax lines with ELIDED_VIM_SYNTAX_LINE.
/// Pass false when --normalize-vim-tags is used, since that does finer-grained normalization.
fn normalize_html(html: &str, elide_vim_tags: bool) -> String {
    // Match lines with IPv4 address followed by </td> (IP address display)
    let ip_re = Regex::new(r"(?m)^\s*\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\s*</td>$").unwrap();
    let result = ip_re.replace_all(html, "                ELIDED_IP_ADDRESS                </td>");

    // Match lines with <td>NUMBER</td> (hit counts)
    let count_re = Regex::new(r"(?m)^\s*<td>\d+</td>$").unwrap();
    let result = count_re.replace_all(&result, "                <td>ELIDED_HIT_COUNT</td>");

    // Match lines with just whitespace and a number (possibly negative) (upvote counts)
    let upvote_re = Regex::new(r"(?m)^\s+-?\d+\s*$").unwrap();
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
    // Skip this when --normalize-vim-tags is used, since that does finer-grained normalization
    let result = if elide_vim_tags {
        let vim_syntax_re = Regex::new(r#"(?m)^.*<span class="(Identifier|Constant|Special|Statement|LineNr|Comment).*$"#).unwrap();
        vim_syntax_re.replace_all(&result, "ELIDED_VIM_SYNTAX_LINE")
    } else {
        result.into_owned().into()
    };

    // Elide "information" and "informaion" typo so contact page matches
    let result = result.replace("informaion", "ELIDED_INFO_WORD");
    let result = result.replace("information", "ELIDED_INFO_WORD");

    // Normalize JS hex escapes to literal chars
    // PHP's jse() escapes all non-alphanumeric chars, Rust doesn't need to
    let result = result.replace("\\x2D", "-");
    let result = result.replace("\\x5F", "_");

    // Normalize encfs audit URLs (old vs new naming)
    // Replace longer strings first to avoid partial match
    let result = result.replace("auditencfsOLD", "ELIDED_ENCFS_AUDIT");
    let result = result.replace("auditencfsold", "ELIDED_ENCFS_AUDIT");
    let result = result.replace("auditencfs", "ELIDED_ENCFS_AUDIT");

    // Normalize absolute production URLs to relative URLs
    // Production PHP uses full URLs like "https://defuse.ca:443/page.htm"
    // Rust uses relative URLs like "/page.htm"
    result.replace("https://defuse.ca:443", "")
}

/// Normalize vim syntax highlighting tags within vimhighlight sections.
///
/// This strips span tags with whitelisted vim syntax classes while preserving
/// the text content. This allows comparing source code content across different
/// vim versions which may highlight functions/keywords differently.
///
/// Whitelisted classes: Identifier, Constant, Special, Statement, LineNr, Comment, PreProc
fn normalize_vim_syntax_tags(html: &str) -> String {
    // Whitelisted vim syntax highlighting classes
    const VIM_CLASSES: &[&str] = &[
        "Identifier",
        "Constant",
        "Special",
        "Statement",
        "LineNr",
        "Comment",
        "PreProc",
        "Type",
        "String",
        "Number",
        "Function",
        "Keyword",
        "Operator",
        "Error",
        "Todo",
        "Underlined",
    ];

    // Find and process each vimhighlight section
    let vimhl_start = "<div class=\"vimhighlight\">";
    let vimhl_end = "</div>";

    let mut result = String::with_capacity(html.len());
    let mut remaining = html;

    while let Some(start_idx) = remaining.find(vimhl_start) {
        // Copy everything before this vimhighlight section
        result.push_str(&remaining[..start_idx]);

        // Find the end of this section
        let after_start = &remaining[start_idx + vimhl_start.len()..];
        if let Some(end_idx) = after_start.find(vimhl_end) {
            // Copy the opening tag
            result.push_str(vimhl_start);

            // Get the content and normalize it
            let content = &after_start[..end_idx];
            let normalized_content = strip_vim_spans(content, VIM_CLASSES);
            result.push_str(&normalized_content);

            // Copy the closing tag
            result.push_str(vimhl_end);

            // Move past this section
            remaining = &after_start[end_idx + vimhl_end.len()..];
        } else {
            // No closing tag found, copy the rest as-is
            result.push_str(&remaining[start_idx..]);
            remaining = "";
        }
    }

    // Copy any remaining content
    result.push_str(remaining);

    result
}

/// Strip span tags with whitelisted classes from content, preserving text.
fn strip_vim_spans(content: &str, classes: &[&str]) -> String {
    let mut result = String::with_capacity(content.len());
    let mut remaining = content;

    while !remaining.is_empty() {
        // Look for opening span tag
        if let Some(span_start) = remaining.find("<span") {
            // Copy everything before the span
            result.push_str(&remaining[..span_start]);

            // Find the end of the opening tag
            let after_span = &remaining[span_start..];
            if let Some(tag_end) = after_span.find('>') {
                let tag = &after_span[..=tag_end];

                // Check if this span has a whitelisted class
                let is_vim_span = classes.iter().any(|class| {
                    tag.contains(&format!("class=\"{}", class))
                        || tag.contains(&format!("class='{}", class))
                });

                if is_vim_span {
                    // Skip the opening tag (don't add it to result)
                    remaining = &after_span[tag_end + 1..];

                    // Find and skip the matching closing </span>
                    // Note: we need to handle nested spans correctly
                    if let Some((inner_content, after_close)) = find_span_content(remaining) {
                        // Recursively process the inner content (may have nested spans)
                        let processed_inner = strip_vim_spans(inner_content, classes);
                        result.push_str(&processed_inner);
                        remaining = after_close;
                    } else {
                        // No closing tag, just continue
                    }
                } else {
                    // Not a vim span, keep it as-is
                    result.push_str(tag);
                    remaining = &after_span[tag_end + 1..];
                }
            } else {
                // Malformed tag, copy the < and continue
                result.push('<');
                remaining = &remaining[span_start + 1..];
            }
        } else {
            // No more spans, copy the rest
            result.push_str(remaining);
            break;
        }
    }

    result
}

/// Find the content inside a span and return (inner_content, remaining_after_close_tag).
fn find_span_content(s: &str) -> Option<(&str, &str)> {
    let mut depth = 1;
    let mut idx = 0;

    while idx < s.len() && depth > 0 {
        if idx + 6 <= s.len() && &s[idx..idx + 6] == "<span " {
            depth += 1;
            idx += 6;
        } else if idx + 6 <= s.len() && &s[idx..idx + 6] == "<span>" {
            depth += 1;
            idx += 6;
        } else if idx + 7 <= s.len() && &s[idx..idx + 7] == "</span>" {
            depth -= 1;
            if depth == 0 {
                return Some((&s[..idx], &s[idx + 7..]));
            }
            idx += 7;
        } else {
            idx += 1;
        }
    }

    None
}
