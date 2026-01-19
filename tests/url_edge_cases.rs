//! URL edge case tests
//!
//! Tests for URL patterns that might be handled incorrectly:
//! - Trailing slashes
//! - Double slashes
//! - URL-encoded characters in slugs

mod common;

use common::{assert_status, client, url};

// =============================================================================
// Trailing Slash Tests
// =============================================================================

/// Trailing slash on a page URL should return 404 (not redirect or serve the page)
#[tokio::test]
async fn trailing_slash_on_page_returns_404() {
    let resp = client().get(url("/about.htm/")).send().await.unwrap();
    assert_status(&resp, 404, "/about.htm/ (trailing slash)");
}

/// Trailing slash on root is fine (it's just /)
#[tokio::test]
async fn root_with_trailing_slash_ok() {
    let resp = client().get(url("/")).send().await.unwrap();
    assert_status(&resp, 200, "/ (root)");
}

// =============================================================================
// Double Slash Tests
// =============================================================================

/// Double slash at start of path should return 404
#[tokio::test]
async fn double_slash_at_start_returns_404() {
    let resp = client().get(url("//about.htm")).send().await.unwrap();
    assert_status(&resp, 404, "//about.htm (double slash)");
}

/// Double slash in static file paths gets normalized by Apache (serves the file)
/// Rust implementation intentionally does NOT normalize double slashes - returns 404
#[tokio::test]
#[ignore = "Intentional difference: Rust does not normalize // in paths (returns 404)"]
async fn double_slash_in_static_path_normalized() {
    let resp = client().get(url("/downloads//thesis.pdf")).send().await.unwrap();
    assert_status(&resp, 200, "/downloads//thesis.pdf (double slash normalized)");
}

// =============================================================================
// URL-Encoded Character Tests
// =============================================================================

/// URL-encoded dot in slug should return 404 (not treated as .htm)
#[tokio::test]
async fn url_encoded_dot_in_extension_returns_404() {
    let resp = client().get(url("/about%2ehtm")).send().await.unwrap();
    assert_status(&resp, 404, "/about%2ehtm (encoded dot)");
}

/// URL-encoded slash should return 404
#[tokio::test]
async fn url_encoded_slash_returns_404() {
    let resp = client().get(url("/about%2fhtm")).send().await.unwrap();
    assert_status(&resp, 404, "/about%2fhtm (encoded slash)");
}

/// URL-encoded characters in page slug should return 404
#[tokio::test]
async fn url_encoded_chars_in_slug_returns_404() {
    let resp = client().get(url("/%61bout.htm")).send().await.unwrap(); // %61 = 'a'
    assert_status(&resp, 404, "/%61bout.htm (encoded 'a')");
}
