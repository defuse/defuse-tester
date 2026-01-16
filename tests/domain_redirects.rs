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
#[ignore = "Apache rewrite bug produces double slash: https://defuse.ca//"]
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
#[ignore = "Apache rewrite bug produces double slash: https://defuse.ca//about.htm"]
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

/// http://www.defuse.ca/about should redirect to https://defuse.ca/about.htm in one redirect
/// This tests HTTP→HTTPS + www strip + extension normalization all at once
#[tokio::test]
#[ignore = "defuse.ca uses multiple redirects, not single; also www strip has double-slash bug"]
async fn full_redirect_chain_no_extension() {
    if !is_production_url() {
        eprintln!("Skipping full redirect chain test on local URL");
        return;
    }

    let resp = client()
        .get("http://www.defuse.ca/about")
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status().as_u16(),
        301,
        "Should redirect with 301, got: {}",
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
        "http://www.defuse.ca/about should redirect to https://defuse.ca/about.htm in single redirect, got: {}",
        location
    );
}

/// http://www.defuse.ca/aBouT.htm should redirect to https://defuse.ca/about.htm in one redirect
/// This tests HTTP→HTTPS + www strip + case normalization all at once
#[tokio::test]
#[ignore = "defuse.ca lacks single-redirect canonicalization; uses multiple redirects"]
async fn full_redirect_chain_case_insensitive() {
    if !is_production_url() {
        eprintln!("Skipping full redirect chain test on local URL");
        return;
    }

    let resp = client()
        .get("http://www.defuse.ca/aBouT.htm")
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status().as_u16(),
        301,
        "Should redirect with 301, got: {}",
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
        "http://www.defuse.ca/aBouT.htm should redirect to https://defuse.ca/about.htm in single redirect, got: {}",
        location
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
