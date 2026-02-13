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
