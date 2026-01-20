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
    "/js/checksums.js",                       // Static JS
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
#[ignore = "defuse.ca doesn't set X-Frame-Options on static files"]
async fn x_frame_options_on_all_paths() {
    for path in TEST_PATHS {
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
#[ignore = "defuse.ca doesn't set HSTS on static files"]
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
// X-Content-Type-Options (defuse.ca does NOT have this)
// =============================================================================

/// X-Content-Type-Options: nosniff prevents MIME-sniffing attacks
#[tokio::test]
#[ignore = "defuse.ca doesn't have X-Content-Type-Options"]
async fn x_content_type_options_on_all_paths() {
    for path in TEST_PATHS {
        let resp = client().get(url(path)).send().await.unwrap();

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

// =============================================================================
// X-XSS-Protection (defuse.ca does NOT have this)
// =============================================================================

/// X-XSS-Protection is a legacy header but still good to have
#[tokio::test]
#[ignore = "defuse.ca doesn't have X-XSS-Protection"]
async fn x_xss_protection_on_all_paths() {
    for path in TEST_PATHS {
        let resp = client().get(url(path)).send().await.unwrap();

        let header = resp
            .headers()
            .get("x-xss-protection")
            .unwrap_or_else(|| panic!("X-XSS-Protection missing on {}", path))
            .to_str()
            .unwrap();

        assert!(
            header.contains("1") && header.to_lowercase().contains("mode=block"),
            "X-XSS-Protection should be '1; mode=block' on {}, got: {}",
            path,
            header
        );
    }
}

// =============================================================================
// Referrer-Policy (defuse.ca does NOT have this)
// =============================================================================

/// Referrer-Policy controls how much referrer info is sent
#[tokio::test]
#[ignore = "defuse.ca doesn't have Referrer-Policy"]
async fn referrer_policy_on_all_paths() {
    for path in TEST_PATHS {
        let resp = client().get(url(path)).send().await.unwrap();

        let header = resp
            .headers()
            .get("referrer-policy")
            .unwrap_or_else(|| panic!("Referrer-Policy missing on {}", path));

        // Just check it exists - various values are acceptable
        assert!(
            !header.is_empty(),
            "Referrer-Policy should not be empty on {}",
            path
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
