//! Domain-level redirect tests
//!
//! Tests for redirects that involve different domains/subdomains.
//! These only run against production URLs, not localhost.

mod common;

use common::{client, is_production_url};

// =============================================================================
// www to non-www redirect
// =============================================================================

/// www.defuse.ca should redirect to defuse.ca (301)
#[tokio::test]
async fn www_redirects_to_non_www() {
    if !is_production_url() {
        eprintln!("Skipping www redirect test on local URL");
        return;
    }

    let resp = client()
        .get("https://www.defuse.ca/")
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status().as_u16(),
        301,
        "www.defuse.ca should redirect with 301, got: {}",
        resp.status()
    );

    let location = resp
        .headers()
        .get("location")
        .expect("Redirect should have Location header")
        .to_str()
        .unwrap();

    // Apache rewrite produces double slash, but that's okay - browsers normalize it
    assert!(
        location.starts_with("https://defuse.ca/"),
        "Should redirect to https://defuse.ca/, got: {}",
        location
    );
}

/// www.defuse.ca/path should redirect to defuse.ca/path (preserving path)
#[tokio::test]
async fn www_redirect_preserves_path() {
    if !is_production_url() {
        eprintln!("Skipping www redirect test on local URL");
        return;
    }

    let resp = client()
        .get("https://www.defuse.ca/about.htm")
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status().as_u16(),
        301,
        "www.defuse.ca/about.htm should redirect with 301, got: {}",
        resp.status()
    );

    let location = resp
        .headers()
        .get("location")
        .expect("Redirect should have Location header")
        .to_str()
        .unwrap();

    // Apache rewrite produces double slash, but path is preserved
    assert!(
        location.contains("defuse.ca/") && location.contains("about.htm"),
        "Should redirect to defuse.ca with path preserved, got: {}",
        location
    );
}

// =============================================================================
// bin.defuse.ca redirect
// =============================================================================

/// bin.defuse.ca should redirect to defuse.ca/b/
#[tokio::test]
async fn bin_subdomain_redirects_to_b_path() {
    if !is_production_url() {
        eprintln!("Skipping bin.defuse.ca redirect test on local URL");
        return;
    }

    // Request bin.defuse.ca with a paste key
    let resp = client()
        .get("https://bin.defuse.ca/test123")
        .send()
        .await
        .unwrap();

    // Should redirect (could be 301 or 302 depending on implementation)
    assert!(
        resp.status().is_redirection(),
        "bin.defuse.ca should redirect, got: {}",
        resp.status()
    );

    let location = resp
        .headers()
        .get("location")
        .expect("Redirect should have Location header")
        .to_str()
        .unwrap();

    assert!(
        location.starts_with("https://defuse.ca/b/"),
        "Should redirect to https://defuse.ca/b/*, got: {}",
        location
    );
}
