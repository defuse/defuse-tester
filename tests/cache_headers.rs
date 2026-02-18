//! Cache header tests
//!
//! Tests for caching-related HTTP headers.
//! Static files should have Last-Modified for conditional requests.
//! Dynamic pages (HTML) should not have these headers.

mod common;

use common::{client, url};

// =============================================================================
// Static Files - Should have Last-Modified
// =============================================================================

/// Images should have Last-Modified header for caching
#[tokio::test]
async fn image_has_last_modified() {
    let resp = client()
        .get(url("/images/1by1.gif"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "/images/1by1.gif should return 200");

    assert!(
        resp.headers().contains_key("last-modified"),
        "Images should have Last-Modified header"
    );
}

/// CSS files should have Last-Modified header for caching
#[tokio::test]
async fn css_has_last_modified() {
    let resp = client()
        .get(url("/main.css"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "/main.css should return 200");

    assert!(
        resp.headers().contains_key("last-modified"),
        "CSS files should have Last-Modified header"
    );
}

/// JS files should have Last-Modified header for caching
#[tokio::test]
async fn js_has_last_modified() {
    let resp = client()
        .get(url("/js/upvote.js"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "/js/upvote.js should return 200");

    assert!(
        resp.headers().contains_key("last-modified"),
        "JS files should have Last-Modified header"
    );
}

// =============================================================================
// Conditional Requests (If-Modified-Since / 304 Not Modified)
// =============================================================================

/// Sending If-Modified-Since with the file's Last-Modified should return 304
#[tokio::test]
async fn if_modified_since_returns_304() {
    let c = client();

    // First request: get Last-Modified
    let resp1 = c.get(url("/main.css")).send().await.unwrap();
    assert!(resp1.status().is_success());
    let last_modified = resp1
        .headers()
        .get("last-modified")
        .expect("Static file should have Last-Modified")
        .to_str()
        .unwrap()
        .to_owned();

    // Second request: send If-Modified-Since with the same timestamp
    let resp2 = c
        .get(url("/main.css"))
        .header("if-modified-since", &last_modified)
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp2.status().as_u16(),
        304,
        "If-Modified-Since matching Last-Modified should return 304, got {}",
        resp2.status()
    );
}

// =============================================================================
// Dynamic Pages - Should NOT have Last-Modified (generated dynamically)
// =============================================================================

/// Dynamic HTML pages should not have Last-Modified (content is generated)
#[tokio::test]
async fn html_page_no_last_modified() {
    let resp = client()
        .get(url("/about.htm"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    assert!(
        !resp.headers().contains_key("last-modified"),
        "Dynamic HTML pages should not have Last-Modified header (content is generated)"
    );
}
