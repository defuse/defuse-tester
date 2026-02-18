//! Security header tests
//!
//! Tests for security-related HTTP headers and behaviors.

mod common;

use common::{client, is_production_url, url};

// =============================================================================
// Test Path Definitions
// =============================================================================

/// All paths to test - covers different routing logic paths
const TEST_PATHS: &[&str] = &[
    "/",                                      // Home page (dynamic)
    "/about.htm",                             // Regular page (dynamic)
    "/main.css",                              // Static CSS
    "/images/1by1.gif",                       // Static image
    "/js/csprng.js",                          // Static JS
    "/files/eharmony-hashes.txt",             // files/ alias (force download)
    "/files2/poc/pocorgtfo00.pdf",            // files2/ alias (force download, large files)
    "/mirrors/zcash-mainnet-2019-10.tar.bz2", // mirrors/ alias (force download)
    "/upload/vim/vimrc",                      // upload/ alias (force download)
    "/source/breach.php",                     // source/ alias (force download)
];

// =============================================================================
// X-Frame-Options
// =============================================================================

/// X-Frame-Options should be SAMEORIGIN on all paths
/// Note: defuse.ca only sets this on dynamic pages, not static files
#[tokio::test]
async fn x_frame_options_on_all_paths() {
    for path in TEST_PATHS {
        let resp = client().get(url(path)).send().await.unwrap();
        assert!(resp.status().is_success(), "{} should return 200, got {}", path, resp.status());

        let header = resp
            .headers()
            .get("x-frame-options")
            .unwrap_or_else(|| panic!("X-Frame-Options missing on {}", path))
            .to_str()
            .unwrap();

        assert_eq!(
            header.to_uppercase(),
            "SAMEORIGIN",
            "X-Frame-Options should be SAMEORIGIN on {}, got: {}",
            path,
            header
        );
    }
}

/// X-Frame-Options on dynamic pages only (what defuse.ca actually does)
#[tokio::test]
async fn x_frame_options_on_dynamic_pages() {
    for path in &["/", "/about.htm"] {
        let resp = client().get(url(path)).send().await.unwrap();

        let header = resp
            .headers()
            .get("x-frame-options")
            .unwrap_or_else(|| panic!("X-Frame-Options missing on {}", path))
            .to_str()
            .unwrap();

        assert_eq!(
            header.to_uppercase(),
            "SAMEORIGIN",
            "X-Frame-Options should be SAMEORIGIN on {}, got: {}",
            path,
            header
        );
    }
}

// =============================================================================
// HSTS (production HTTPS only)
// =============================================================================

/// HSTS should be present on all paths over HTTPS
/// Note: defuse.ca only sets this on dynamic pages, not static files
#[tokio::test]
async fn hsts_on_all_paths() {
    if !is_production_url() {
        eprintln!("Skipping HSTS test on local URL");
        return;
    }

    for path in TEST_PATHS {
        let https_url = url(path).replace("http://", "https://");
        let resp = client().get(&https_url).send().await.unwrap();

        let header = resp
            .headers()
            .get("strict-transport-security")
            .unwrap_or_else(|| panic!("HSTS missing on {}", path))
            .to_str()
            .unwrap();

        assert!(
            header.contains("max-age="),
            "HSTS should have max-age on {}, got: {}",
            path,
            header
        );
    }
}

/// HSTS on dynamic pages only (what defuse.ca actually does)
#[tokio::test]
async fn hsts_on_dynamic_pages() {
    if !is_production_url() {
        eprintln!("Skipping HSTS test on local URL");
        return;
    }

    for path in &["/", "/about.htm"] {
        let https_url = url(path).replace("http://", "https://");
        let resp = client().get(&https_url).send().await.unwrap();

        let header = resp
            .headers()
            .get("strict-transport-security")
            .unwrap_or_else(|| panic!("HSTS missing on {}", path))
            .to_str()
            .unwrap();

        assert!(
            header.contains("max-age="),
            "HSTS should have max-age on {}, got: {}",
            path,
            header
        );
    }
}

// =============================================================================
// X-Content-Type-Options
// =============================================================================

/// X-Content-Type-Options: nosniff prevents MIME-sniffing attacks
#[tokio::test]
async fn x_content_type_options_on_all_paths() {
    for path in TEST_PATHS {
        let resp = client().get(url(path)).send().await.unwrap();
        assert!(resp.status().is_success(), "{} should return 200, got {}", path, resp.status());

        let header = resp
            .headers()
            .get("x-content-type-options")
            .unwrap_or_else(|| panic!("X-Content-Type-Options missing on {}", path))
            .to_str()
            .unwrap();

        assert_eq!(
            header.to_lowercase(),
            "nosniff",
            "X-Content-Type-Options should be nosniff on {}, got: {}",
            path,
            header
        );
    }
}

// Note: X-XSS-Protection is intentionally NOT set - it's deprecated and can
// cause security issues. Modern browsers have removed support for it.

// =============================================================================
// Referrer-Policy
// =============================================================================

/// Referrer-Policy controls how much referrer info is sent.
/// We use strict-origin-when-cross-origin to protect sensitive URLs (like paste keys).
#[tokio::test]
async fn referrer_policy_on_all_paths() {
    for path in TEST_PATHS {
        let resp = client().get(url(path)).send().await.unwrap();
        assert!(resp.status().is_success(), "{} should return 200, got {}", path, resp.status());

        let header = resp
            .headers()
            .get("referrer-policy")
            .unwrap_or_else(|| panic!("Referrer-Policy missing on {}", path))
            .to_str()
            .unwrap();

        assert_eq!(
            header, "strict-origin-when-cross-origin",
            "Referrer-Policy should be strict-origin-when-cross-origin on {}, got: {}",
            path,
            header
        );
    }
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
// Security Headers on Error Responses
// =============================================================================

/// 404 error pages should still carry security headers
#[tokio::test]
async fn security_headers_on_404() {
    let resp = client().get(url("/nonexistent-page-12345.htm")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 404, "Should return 404");

    let headers = resp.headers();
    assert_eq!(
        headers.get("x-frame-options").expect("404 should have X-Frame-Options").to_str().unwrap().to_uppercase(),
        "SAMEORIGIN",
        "404 should have X-Frame-Options: SAMEORIGIN"
    );
    assert_eq!(
        headers.get("x-content-type-options").expect("404 should have X-Content-Type-Options").to_str().unwrap().to_lowercase(),
        "nosniff",
        "404 should have X-Content-Type-Options: nosniff"
    );
    assert_eq!(
        headers.get("referrer-policy").expect("404 should have Referrer-Policy").to_str().unwrap(),
        "strict-origin-when-cross-origin",
        "404 should have Referrer-Policy: strict-origin-when-cross-origin"
    );
}

// =============================================================================
// Directory Access Restrictions
// =============================================================================

/// /pages/ directory should return 403 Forbidden or 404 Not Found
/// PHP returns 403, Rust returns 404 - both are acceptable security behaviors
#[tokio::test]
async fn pages_directory_forbidden() {
    let resp = client().get(url("/pages/")).send().await.unwrap();
    let status = resp.status().as_u16();

    assert!(
        status == 403 || status == 404,
        "/pages/ should return 403 or 404, got: {}",
        status
    );
}

/// /libs/ directory should return 403 Forbidden or 404 Not Found
/// PHP returns 403, Rust returns 404 - both are acceptable security behaviors
#[tokio::test]
async fn libs_directory_forbidden() {
    let resp = client().get(url("/libs/")).send().await.unwrap();
    let status = resp.status().as_u16();

    assert!(
        status == 403 || status == 404,
        "/libs/ should return 403 or 404, got: {}",
        status
    );
}
