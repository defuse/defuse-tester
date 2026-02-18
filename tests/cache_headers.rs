//! Cache header tests
//!
//! Tests for caching-related HTTP headers.
//! Static files should have ETag and Last-Modified for conditional requests.
//! Dynamic pages (HTML) should not have these headers.

mod common;

use common::{client, url};

// =============================================================================
// Static Files - Should have ETag and Last-Modified
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

/// Images should have ETag header for caching
#[tokio::test]
async fn image_has_etag() {
    let resp = client()
        .get(url("/images/1by1.gif"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    assert!(
        resp.headers().contains_key("etag"),
        "Images should have ETag header"
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

/// CSS files should have ETag header for caching
#[tokio::test]
async fn css_has_etag() {
    let resp = client()
        .get(url("/main.css"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    assert!(
        resp.headers().contains_key("etag"),
        "CSS files should have ETag header"
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

/// JS files should have ETag header for caching
#[tokio::test]
async fn js_has_etag() {
    let resp = client()
        .get(url("/js/upvote.js"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    assert!(
        resp.headers().contains_key("etag"),
        "JS files should have ETag header"
    );
}

/// ETag should be in Apache-compatible format: "<hex>-<hex>"
/// This implicitly tests that the Last-Modified date was parsed correctly,
/// since a parse failure would result in no ETag being generated.
#[tokio::test]
async fn etag_format_is_valid() {
    let resp = client()
        .get(url("/main.css"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let etag = resp.headers().get("etag")
        .expect("Static file should have ETag header")
        .to_str()
        .expect("ETag should be valid ASCII");

    // Format: "<hex>-<hex>" with surrounding quotes
    assert!(etag.starts_with('"') && etag.ends_with('"'), "ETag should be quoted: {}", etag);
    let inner = &etag[1..etag.len()-1];
    let parts: Vec<&str> = inner.split('-').collect();
    assert_eq!(parts.len(), 2, "ETag should have exactly one hyphen: {}", etag);
    assert!(u64::from_str_radix(parts[0], 16).is_ok(), "ETag size should be hex: {}", etag);
    assert!(u64::from_str_radix(parts[1], 16).is_ok(), "ETag mtime should be hex: {}", etag);
    // mtime should be a reasonable Unix timestamp (after 2020, before 2100)
    let mtime = u64::from_str_radix(parts[1], 16).unwrap();
    assert!(mtime > 1262304000, "ETag mtime too small (before 2010): {}", mtime);
    assert!(mtime < 4102444800, "ETag mtime too large (after 2100): {}", mtime);
}

// =============================================================================
// Conditional Requests (If-None-Match / 304 Not Modified)
// =============================================================================

/// Sending If-None-Match with a matching ETag should return 304 Not Modified
#[tokio::test]
async fn if_none_match_returns_304() {
    let c = client();

    // First request: get the ETag
    let resp1 = c.get(url("/main.css")).send().await.unwrap();
    assert!(resp1.status().is_success());
    let etag = resp1
        .headers()
        .get("etag")
        .expect("Static file should have ETag")
        .to_str()
        .unwrap()
        .to_owned();

    // Second request: send If-None-Match with the same ETag
    let resp2 = c
        .get(url("/main.css"))
        .header("if-none-match", &etag)
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp2.status().as_u16(),
        304,
        "Matching If-None-Match should return 304, got {}",
        resp2.status()
    );
}

/// 304 response should have an empty body
#[tokio::test]
async fn not_modified_has_empty_body() {
    let c = client();

    let resp1 = c.get(url("/main.css")).send().await.unwrap();
    let etag = resp1
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let resp2 = c
        .get(url("/main.css"))
        .header("if-none-match", &etag)
        .send()
        .await
        .unwrap();

    assert_eq!(resp2.status().as_u16(), 304);
    let body = resp2.bytes().await.unwrap();
    assert!(
        body.is_empty(),
        "304 response body should be empty, got {} bytes",
        body.len()
    );
}

/// 304 response should still include the ETag header
#[tokio::test]
async fn not_modified_includes_etag() {
    let c = client();

    let resp1 = c.get(url("/main.css")).send().await.unwrap();
    let etag = resp1
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let resp2 = c
        .get(url("/main.css"))
        .header("if-none-match", &etag)
        .send()
        .await
        .unwrap();

    assert_eq!(resp2.status().as_u16(), 304);
    let etag2 = resp2
        .headers()
        .get("etag")
        .expect("304 response should still include ETag header")
        .to_str()
        .unwrap();
    assert_eq!(etag, etag2, "ETag should be the same in 304 response");
}

/// 304 response should not have Content-Type header
#[tokio::test]
async fn not_modified_no_content_type() {
    let c = client();

    let resp1 = c.get(url("/main.css")).send().await.unwrap();
    let etag = resp1
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let resp2 = c
        .get(url("/main.css"))
        .header("if-none-match", &etag)
        .send()
        .await
        .unwrap();

    assert_eq!(resp2.status().as_u16(), 304);
    assert!(
        !resp2.headers().contains_key("content-type"),
        "304 response should not have Content-Type header"
    );
}

/// Non-matching If-None-Match should return 200 with full body
#[tokio::test]
async fn if_none_match_mismatch_returns_200() {
    let c = client();

    let resp = c
        .get(url("/main.css"))
        .header("if-none-match", "\"wrong-etag\"")
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status().as_u16(),
        200,
        "Non-matching If-None-Match should return 200, got {}",
        resp.status()
    );

    let body = resp.bytes().await.unwrap();
    assert!(!body.is_empty(), "200 response should have a body");
}

/// If-None-Match with wildcard * should return 304 for any static file
#[tokio::test]
async fn if_none_match_wildcard_returns_304() {
    let c = client();

    let resp = c
        .get(url("/main.css"))
        .header("if-none-match", "*")
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status().as_u16(),
        304,
        "If-None-Match: * should return 304 for static files, got {}",
        resp.status()
    );
}

/// Dynamic pages should not return 304 (they don't have ETags)
#[tokio::test]
async fn dynamic_page_ignores_if_none_match() {
    let c = client();

    let resp = c
        .get(url("/about.htm"))
        .header("if-none-match", "*")
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status().as_u16(),
        200,
        "Dynamic pages should ignore If-None-Match (no ETag), got {}",
        resp.status()
    );
}

// =============================================================================
// Dynamic Pages - Should NOT have ETag/Last-Modified (generated dynamically)
// =============================================================================

/// Dynamic HTML pages should not have ETag (content is generated)
#[tokio::test]
async fn html_page_no_etag() {
    let resp = client()
        .get(url("/about.htm"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "/about.htm should return 200");

    assert!(
        !resp.headers().contains_key("etag"),
        "Dynamic HTML pages should not have ETag header (content is generated)"
    );
}

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
