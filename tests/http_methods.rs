//! HTTP method handling tests
//!
//! Tests for correct handling of different HTTP methods:
//! - HEAD requests on static files
//! - POST to pages that only need GET (should still return 200, same as GET)

mod common;

use common::{client, url};

// =============================================================================
// HEAD Request Tests
// =============================================================================

/// HEAD request on CSS file should return 200 with headers but no body
#[tokio::test]
async fn head_request_on_css_returns_200() {
    let resp = client().head(url("/main.css")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "HEAD /main.css should return 200");

    // Should have Content-Type header
    assert!(
        resp.headers().contains_key("content-type"),
        "HEAD response should include Content-Type header"
    );

    // Should have Content-Length header
    assert!(
        resp.headers().contains_key("content-length"),
        "HEAD response should include Content-Length header"
    );

    // HEAD response must have empty body
    assert_eq!(resp.text().await.unwrap().len(), 0, "HEAD response should have empty body");
}

/// HEAD request on JS file should return 200
#[tokio::test]
async fn head_request_on_js_returns_200() {
    let resp = client().head(url("/js/defuse.js")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "HEAD /js/defuse.js should return 200");
}

/// HEAD request on image should return 200
#[tokio::test]
async fn head_request_on_image_returns_200() {
    let resp = client().head(url("/images/bg.png")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "HEAD /images/bg.png should return 200");
}

/// HEAD request on HTML page should return 200
#[tokio::test]
async fn head_request_on_page_returns_200() {
    let resp = client().head(url("/about.htm")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "HEAD /about.htm should return 200");
}

/// HEAD request on downloadable file should return 200
#[tokio::test]
async fn head_request_on_download_returns_200() {
    let resp = client().head(url("/downloads/thesis.pdf")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "HEAD /downloads/thesis.pdf should return 200");
}

/// HEAD request on nonexistent page should return 404
#[tokio::test]
async fn head_request_on_nonexistent_returns_404() {
    let resp = client().head(url("/nonexistent-page-12345.htm")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 404, "HEAD on nonexistent should return 404");
}

// =============================================================================
// POST to GET-only Pages
// =============================================================================

/// POST to a GET-only page should return 405 Method Not Allowed.
/// This is stricter than PHP (which returns 200) but more correct HTTP behavior.
#[tokio::test]
async fn post_to_get_only_page_returns_405() {
    let resp = client().post(url("/about.htm")).send().await.unwrap();
    assert_eq!(
        resp.status().as_u16(),
        405,
        "POST to /about.htm should return 405 Method Not Allowed"
    );
}

/// POST to home page should return 405 Method Not Allowed.
/// This is stricter than PHP (which returns 200) but more correct HTTP behavior.
#[tokio::test]
async fn post_to_home_returns_405() {
    let resp = client().post(url("/")).send().await.unwrap();
    assert_eq!(
        resp.status().as_u16(),
        405,
        "POST to / should return 405 Method Not Allowed"
    );
}
