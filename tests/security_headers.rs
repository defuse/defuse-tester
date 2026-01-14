//! Security header tests
//!
//! Tests for security-related HTTP headers and behaviors.

mod common;

use common::{client, is_production_url, url};

// =============================================================================
// X-Frame-Options
// =============================================================================

/// All pages should have X-Frame-Options: SAMEORIGIN to prevent clickjacking
#[tokio::test]
async fn x_frame_options_present() {
    let resp = client().get(url("/")).send().await.unwrap();

    let header = resp
        .headers()
        .get("x-frame-options")
        .expect("X-Frame-Options header should be present")
        .to_str()
        .unwrap();

    assert_eq!(
        header.to_uppercase(),
        "SAMEORIGIN",
        "X-Frame-Options should be SAMEORIGIN, got: {}",
        header
    );
}

/// X-Frame-Options should be present on subpages too
#[tokio::test]
async fn x_frame_options_on_subpage() {
    let resp = client().get(url("/about.htm")).send().await.unwrap();

    let header = resp
        .headers()
        .get("x-frame-options")
        .expect("X-Frame-Options header should be present on subpages")
        .to_str()
        .unwrap();

    assert_eq!(
        header.to_uppercase(),
        "SAMEORIGIN",
        "X-Frame-Options should be SAMEORIGIN on subpages"
    );
}

// =============================================================================
// HTTPS Redirect (production only)
// =============================================================================

/// HTTP requests should redirect to HTTPS (301)
/// Only runs against production URLs, not localhost
#[tokio::test]
async fn http_redirects_to_https() {
    if !is_production_url() {
        eprintln!("Skipping HTTPS redirect test on local URL");
        return;
    }

    // Build HTTP URL (not HTTPS)
    let http_url = url("/").replace("https://", "http://");

    let resp = client().get(&http_url).send().await.unwrap();

    assert_eq!(
        resp.status().as_u16(),
        301,
        "HTTP should redirect with 301, got: {}",
        resp.status()
    );

    let location = resp
        .headers()
        .get("location")
        .expect("Redirect should have Location header")
        .to_str()
        .unwrap();

    assert!(
        location.starts_with("https://"),
        "Should redirect to HTTPS, got: {}",
        location
    );
}

// =============================================================================
// HSTS (production HTTPS only)
// =============================================================================

/// HSTS header should be present on HTTPS responses
/// Only runs against production URLs, not localhost
#[tokio::test]
async fn hsts_header_present() {
    if !is_production_url() {
        eprintln!("Skipping HSTS test on local URL");
        return;
    }

    // Make sure we're using HTTPS
    let https_url = url("/").replace("http://", "https://");

    let resp = client().get(&https_url).send().await.unwrap();

    let header = resp
        .headers()
        .get("strict-transport-security")
        .expect("HSTS header should be present on HTTPS")
        .to_str()
        .unwrap();

    assert!(
        header.contains("max-age="),
        "HSTS should have max-age directive, got: {}",
        header
    );
}
