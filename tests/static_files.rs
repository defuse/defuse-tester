//! Tests for static file serving (CSS, JS, favicon, robots.txt)
//!
//! These tests verify files exist and are served. Content-type checks are separate.

mod common;

use common::{client, url};

// =============================================================================
// CSS Files
// =============================================================================

#[tokio::test]
async fn css_main() {
    let resp = client().get(url("/main.css")).send().await.unwrap();
    assert!(resp.status().is_success(), "/main.css should exist");
}

#[tokio::test]
async fn css_mainmenu() {
    let resp = client().get(url("/mainmenu.css")).send().await.unwrap();
    assert!(resp.status().is_success(), "/mainmenu.css should exist");
}

#[tokio::test]
async fn css_vimhl() {
    let resp = client().get(url("/vimhl.css")).send().await.unwrap();
    assert!(resp.status().is_success(), "/vimhl.css should exist");
}

#[tokio::test]
async fn css_print() {
    let resp = client().get(url("/print.css")).send().await.unwrap();
    assert!(resp.status().is_success(), "/print.css should exist");
}

// =============================================================================
// JavaScript Files
// =============================================================================

#[tokio::test]
async fn js_defuse() {
    let resp = client().get(url("/js/defuse.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/defuse.js should exist");
}

#[tokio::test]
async fn js_upvote() {
    let resp = client().get(url("/js/upvote.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/upvote.js should exist");
}

#[tokio::test]
async fn js_aes() {
    let resp = client().get(url("/js/aes.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/aes.js should exist");
}

#[tokio::test]
async fn js_crypto_helpers() {
    let resp = client().get(url("/js/cryptoHelpers.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/cryptoHelpers.js should exist");
}

#[tokio::test]
async fn js_csprng() {
    let resp = client().get(url("/js/csprng.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/csprng.js should exist");
}

#[tokio::test]
async fn js_csprng_extensions() {
    let resp = client().get(url("/js/csprng-extensions.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/csprng-extensions.js should exist");
}

#[tokio::test]
async fn js_encrypt() {
    let resp = client().get(url("/js/encrypt.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/encrypt.js should exist");
}

#[tokio::test]
async fn js_hash() {
    let resp = client().get(url("/js/jsHash.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/jsHash.js should exist");
}

#[tokio::test]
async fn js_leet() {
    let resp = client().get(url("/js/leet.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/leet.js should exist");
}

#[tokio::test]
async fn js_modernizr() {
    let resp = client().get(url("/js/modernizr.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/modernizr.js should exist");
}

#[tokio::test]
async fn js_passwordblocks() {
    let resp = client().get(url("/js/passwordblocks.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/passwordblocks.js should exist");
}

#[tokio::test]
async fn js_sjcl() {
    let resp = client().get(url("/js/sjcl.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/sjcl.js should exist");
}

#[tokio::test]
async fn js_wordlist() {
    let resp = client().get(url("/js/wordlist.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/wordlist.js should exist");
}

// =============================================================================
// Other Static Files
// =============================================================================

#[tokio::test]
async fn favicon() {
    let resp = client().get(url("/favicon.ico")).send().await.unwrap();
    assert!(resp.status().is_success(), "/favicon.ico should exist");
}

#[tokio::test]
async fn robots_txt() {
    let resp = client().get(url("/robots.txt")).send().await.unwrap();
    assert!(resp.status().is_success(), "/robots.txt should exist");

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("User-agent"),
        "/robots.txt should contain User-agent directive"
    );
}

// =============================================================================
// Sample Images (spot check)
// =============================================================================

#[tokio::test]
async fn image_bg() {
    let resp = client().get(url("/images/bg.png")).send().await.unwrap();
    assert!(resp.status().is_success(), "/images/bg.png should exist");
}

#[tokio::test]
async fn image_download() {
    let resp = client().get(url("/images/download.png")).send().await.unwrap();
    assert!(resp.status().is_success(), "/images/download.png should exist");
}

// =============================================================================
// Content-Type Tests (may fail on legacy PHP which uses application/javascript)
// =============================================================================

#[tokio::test]
async fn content_type_css() {
    let resp = client().get(url("/main.css")).send().await.unwrap();
    let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(
        content_type.starts_with("text/css"),
        "CSS should have text/css content-type, got: {}",
        content_type
    );
}

#[tokio::test]
async fn content_type_js() {
    let resp = client().get(url("/js/defuse.js")).send().await.unwrap();
    let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
    // text/javascript is the current standard (RFC 9239)
    assert!(
        content_type.starts_with("text/javascript"),
        "JS should have text/javascript content-type, got: {}",
        content_type
    );
}
