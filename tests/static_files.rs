//! Tests for static file serving (CSS, JS, favicon, robots.txt)

mod common;

use common::{assert_content_type, assert_status, client, url};

// =============================================================================
// CSS Files
// =============================================================================

#[tokio::test]
async fn css_main() {
    let resp = client().get(url("/main.css")).send().await.unwrap();
    assert_status(&resp, 200, "/main.css");
    assert_content_type(&resp, "text/css", "/main.css");

    let body = resp.text().await.unwrap();
    assert!(!body.is_empty(), "/main.css should not be empty");
    // Sanity check - CSS should contain some CSS-like content
    assert!(
        body.contains("{") && body.contains("}"),
        "/main.css should contain CSS rules"
    );
}

#[tokio::test]
async fn css_mainmenu() {
    let resp = client().get(url("/mainmenu.css")).send().await.unwrap();
    assert_status(&resp, 200, "/mainmenu.css");
    assert_content_type(&resp, "text/css", "/mainmenu.css");
}

#[tokio::test]
async fn css_vimhl() {
    let resp = client().get(url("/vimhl.css")).send().await.unwrap();
    assert_status(&resp, 200, "/vimhl.css");
    assert_content_type(&resp, "text/css", "/vimhl.css");
}

#[tokio::test]
async fn css_print() {
    let resp = client().get(url("/print.css")).send().await.unwrap();
    assert_status(&resp, 200, "/print.css");
    assert_content_type(&resp, "text/css", "/print.css");
}

// =============================================================================
// JavaScript Files
// =============================================================================

#[tokio::test]
async fn js_defuse() {
    let resp = client().get(url("/js/defuse.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/defuse.js");
    assert_content_type(&resp, "application/javascript", "/js/defuse.js");

    let body = resp.text().await.unwrap();
    assert!(!body.is_empty(), "/js/defuse.js should not be empty");
}

#[tokio::test]
async fn js_upvote() {
    let resp = client().get(url("/js/upvote.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/upvote.js");
    assert_content_type(&resp, "application/javascript", "/js/upvote.js");
}

#[tokio::test]
async fn js_aes() {
    let resp = client().get(url("/js/aes.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/aes.js");
    assert_content_type(&resp, "application/javascript", "/js/aes.js");
}

#[tokio::test]
async fn js_crypto_helpers() {
    let resp = client().get(url("/js/cryptoHelpers.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/cryptoHelpers.js");
    assert_content_type(&resp, "application/javascript", "/js/cryptoHelpers.js");
}

#[tokio::test]
async fn js_csprng() {
    let resp = client().get(url("/js/csprng.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/csprng.js");
    assert_content_type(&resp, "application/javascript", "/js/csprng.js");
}

#[tokio::test]
async fn js_csprng_extensions() {
    let resp = client().get(url("/js/csprng-extensions.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/csprng-extensions.js");
    assert_content_type(&resp, "application/javascript", "/js/csprng-extensions.js");
}

#[tokio::test]
async fn js_encrypt() {
    let resp = client().get(url("/js/encrypt.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/encrypt.js");
    assert_content_type(&resp, "application/javascript", "/js/encrypt.js");
}

#[tokio::test]
async fn js_hash() {
    let resp = client().get(url("/js/jsHash.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/jsHash.js");
    assert_content_type(&resp, "application/javascript", "/js/jsHash.js");
}

#[tokio::test]
async fn js_leet() {
    let resp = client().get(url("/js/leet.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/leet.js");
    assert_content_type(&resp, "application/javascript", "/js/leet.js");
}

#[tokio::test]
async fn js_modernizr() {
    let resp = client().get(url("/js/modernizr.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/modernizr.js");
    assert_content_type(&resp, "application/javascript", "/js/modernizr.js");
}

#[tokio::test]
async fn js_passwordblocks() {
    let resp = client().get(url("/js/passwordblocks.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/passwordblocks.js");
    assert_content_type(&resp, "application/javascript", "/js/passwordblocks.js");
}

#[tokio::test]
async fn js_sjcl() {
    let resp = client().get(url("/js/sjcl.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/sjcl.js");
    assert_content_type(&resp, "application/javascript", "/js/sjcl.js");
}

#[tokio::test]
async fn js_wordlist() {
    let resp = client().get(url("/js/wordlist.js")).send().await.unwrap();
    assert_status(&resp, 200, "/js/wordlist.js");
    assert_content_type(&resp, "application/javascript", "/js/wordlist.js");
}

// =============================================================================
// Other Static Files
// =============================================================================

#[tokio::test]
async fn favicon() {
    let resp = client().get(url("/favicon.ico")).send().await.unwrap();
    assert_status(&resp, 200, "/favicon.ico");
    // favicon can be image/x-icon, image/vnd.microsoft.icon, or image/ico
    let content_type = resp
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    assert!(
        content_type.contains("icon") || content_type.contains("octet-stream"),
        "/favicon.ico: unexpected content-type: {}",
        content_type
    );
}

#[tokio::test]
async fn robots_txt() {
    let resp = client().get(url("/robots.txt")).send().await.unwrap();
    assert_status(&resp, 200, "/robots.txt");
    assert_content_type(&resp, "text/plain", "/robots.txt");

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("User-agent"),
        "/robots.txt should contain User-agent directive"
    );
}

// =============================================================================
// Sample Images (spot check a few)
// =============================================================================

#[tokio::test]
async fn image_bg() {
    let resp = client().get(url("/images/bg.png")).send().await.unwrap();
    assert_status(&resp, 200, "/images/bg.png");
    assert_content_type(&resp, "image/png", "/images/bg.png");
}

#[tokio::test]
async fn image_download() {
    let resp = client().get(url("/images/download.png")).send().await.unwrap();
    assert_status(&resp, 200, "/images/download.png");
    assert_content_type(&resp, "image/png", "/images/download.png");
}
