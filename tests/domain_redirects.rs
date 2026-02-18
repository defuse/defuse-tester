//! Domain-level redirect tests
//!
//! Tests for redirects that involve different domains/subdomains.
//! These only run against production URLs, not localhost.

mod common;

use common::{client, follow_redirects, is_production_url};

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

    assert_eq!(
        location, "https://defuse.ca/",
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

    assert_eq!(
        location, "https://defuse.ca/about.htm",
        "Should redirect to https://defuse.ca/about.htm, got: {}",
        location
    );
}

// =============================================================================
// Combined redirects (HTTP + www + canonicalization in single redirect)
// =============================================================================

/// http://www.defuse.ca/about should eventually reach https://defuse.ca/about.htm
/// This tests HTTP→HTTPS (Caddy) + www strip (Rust Step 1) + extension normalization (Rust Step 3)
/// across multiple redirect hops.
#[tokio::test]
async fn full_redirect_chain_no_extension() {
    if !is_production_url() {
        eprintln!("Skipping full redirect chain test on local URL");
        return;
    }

    let final_url = follow_redirects("http://www.defuse.ca/about").await;

    assert_eq!(
        final_url, "https://defuse.ca/about.htm",
        "http://www.defuse.ca/about should eventually reach https://defuse.ca/about.htm, got: {}",
        final_url
    );
}

/// http://www.defuse.ca/aBouT.htm should eventually reach https://defuse.ca/about.htm
/// This tests HTTP→HTTPS (Caddy) + www strip (Rust Step 1) + case normalization (Rust Step 3)
/// across multiple redirect hops.
#[tokio::test]
async fn full_redirect_chain_case_insensitive() {
    if !is_production_url() {
        eprintln!("Skipping full redirect chain test on local URL");
        return;
    }

    let final_url = follow_redirects("http://www.defuse.ca/aBouT.htm").await;

    assert_eq!(
        final_url, "https://defuse.ca/about.htm",
        "http://www.defuse.ca/aBouT.htm should eventually reach https://defuse.ca/about.htm, got: {}",
        final_url
    );
}

// =============================================================================
// HTTP → HTTPS enforcement for sensitive endpoints
// =============================================================================

/// HTTP requests to pastebin view should redirect to HTTPS
/// This is important because pastes may contain sensitive information
#[tokio::test]
async fn pastebin_view_http_redirects_to_https() {
    if !is_production_url() {
        eprintln!("Skipping HTTP→HTTPS redirect test on local URL");
        return;
    }

    let resp = client()
        .get("http://defuse.ca/b/test123")
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_redirection(),
        "HTTP /b/key should redirect, got: {}",
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

/// HTTP requests to view.php directly should redirect to HTTPS
#[tokio::test]
async fn pastebin_viewphp_http_redirects_to_https() {
    if !is_production_url() {
        eprintln!("Skipping HTTP→HTTPS redirect test on local URL");
        return;
    }

    let resp = client()
        .get("http://defuse.ca/bin/view.php")
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_redirection(),
        "HTTP /bin/view.php should redirect, got: {}",
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

/// HTTP requests to add.php should redirect to HTTPS
/// This is critical since paste content would be sent in plaintext otherwise
#[tokio::test]
async fn pastebin_addphp_http_redirects_to_https() {
    if !is_production_url() {
        eprintln!("Skipping HTTP→HTTPS redirect test on local URL");
        return;
    }

    // GET request to add.php (POST would create a paste)
    let resp = client()
        .get("http://defuse.ca/bin/add.php")
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_redirection(),
        "HTTP /bin/add.php should redirect, got: {}",
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

/// HTTP requests to pastebin main page should redirect to HTTPS
#[tokio::test]
async fn pastebin_main_http_redirects_to_https() {
    if !is_production_url() {
        eprintln!("Skipping HTTP→HTTPS redirect test on local URL");
        return;
    }

    let resp = client()
        .get("http://defuse.ca/pastebin.htm")
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_redirection(),
        "HTTP /pastebin.htm should redirect, got: {}",
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
// bin.defuse.ca redirect
// =============================================================================

/// bin.defuse.ca (root) should redirect to defuse.ca/pastebin.htm
#[tokio::test]
async fn bin_subdomain_root_redirects_to_pastebin() {
    if !is_production_url() {
        eprintln!("Skipping bin.defuse.ca redirect test on local URL");
        return;
    }

    let resp = client()
        .get("https://bin.defuse.ca/")
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_redirection(),
        "bin.defuse.ca/ should redirect, got: {}",
        resp.status()
    );

    let location = resp
        .headers()
        .get("location")
        .expect("Redirect should have Location header")
        .to_str()
        .unwrap();

    assert_eq!(
        location, "https://defuse.ca/pastebin.htm",
        "bin.defuse.ca/ should redirect to pastebin.htm, got: {}",
        location
    );
}

/// bin.defuse.ca with a path should still redirect to pastebin.htm
/// (Caddy does a blanket redirect for all bin.defuse.ca paths)
#[tokio::test]
async fn bin_subdomain_with_path_redirects_to_pastebin() {
    if !is_production_url() {
        eprintln!("Skipping bin.defuse.ca redirect test on local URL");
        return;
    }

    let resp = client()
        .get("https://bin.defuse.ca/abc123XYZ")
        .send()
        .await
        .unwrap();

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

    assert_eq!(
        location, "https://defuse.ca/pastebin.htm",
        "bin.defuse.ca/abc123XYZ should redirect to pastebin.htm, got: {}",
        location
    );
}

// =============================================================================
// Time Capsule HTTP → HTTPS enforcement
// =============================================================================

/// HTTP requests to time capsule archive download should redirect to HTTPS
/// This ensures the archive integrity (no MITM tampering) during download
#[tokio::test]
async fn timecapsule_download_http_redirects_to_https() {
    if !is_production_url() {
        eprintln!("Skipping HTTP→HTTPS redirect test on local URL");
        return;
    }

    let resp = client()
        .get("http://defuse.ca/timecapsule/quantum-computer-time-capsule-download.php")
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_redirection(),
        "HTTP timecapsule download should redirect, got: {}",
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
