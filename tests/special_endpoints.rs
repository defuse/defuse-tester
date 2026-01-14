//! Tests for special PHP endpoints
//!
//! These are utility endpoints that don't fit the normal page pattern:
//! - /s.php - "Shout" page (display large text)
//! - /rdrand.php - Random bytes download
//! - /ip.php - Raw IP address
//! - /ip-insecure.php - IP address HTML page

mod common;

use common::{client, url};

// =============================================================================
// /s.php - "Shout" page
// =============================================================================

/// /s.php with no params shows a form
#[tokio::test]
#[ignore = "s.php not yet implemented in Rust"]
async fn shout_no_params_shows_form() {
    let resp = client().get(url("/s.php")).send().await.unwrap();
    assert!(resp.status().is_success(), "/s.php should return 200");

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("<form") && body.contains("<input"),
        "/s.php should show a form with input field"
    );
}

/// /s.php?e=text redirects to ?s=base64(text)
#[tokio::test]
#[ignore = "s.php not yet implemented in Rust"]
async fn shout_e_param_redirects_to_s_param() {
    let resp = client()
        .get(url("/s.php?e=hello"))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status().as_u16(),
        302,
        "/s.php?e=text should redirect with 302, got: {}",
        resp.status()
    );

    let location = resp
        .headers()
        .get("location")
        .expect("Redirect should have Location header")
        .to_str()
        .unwrap();

    // "hello" in base64 is "aGVsbG8="
    assert!(
        location.contains("?s=") || location.contains("&s="),
        "/s.php?e=text should redirect to ?s=base64, got: {}",
        location
    );
    assert!(
        location.contains("aGVsbG8"),
        "/s.php?e=hello should redirect to URL containing base64 of 'hello' (aGVsbG8), got: {}",
        location
    );
}

/// /s.php?s=base64text displays decoded text in large font
#[tokio::test]
#[ignore = "s.php not yet implemented in Rust"]
async fn shout_s_param_displays_decoded_text() {
    // "hello" in base64 is "aGVsbG8="
    let resp = client()
        .get(url("/s.php?s=aGVsbG8="))
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success(), "/s.php?s=... should return 200");

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("hello"),
        "/s.php?s=aGVsbG8= should display 'hello', got body: {}",
        &body.chars().take(500).collect::<String>()
    );
    assert!(
        body.contains("300pt") || body.contains("300px"),
        "/s.php should use 300pt font styling"
    );
}

// =============================================================================
// /rdrand.php - Random bytes endpoint
// Note: defuse.ca returns 500 (RDRAND instruction may not be available on server)
// =============================================================================

/// /rdrand.php returns binary data with correct headers
#[tokio::test]
#[ignore = "rdrand.php not yet implemented in Rust; also returns 500 on defuse.ca"]
async fn rdrand_returns_binary_with_correct_headers() {
    let resp = client().get(url("/rdrand.php")).send().await.unwrap();
    assert!(resp.status().is_success(), "/rdrand.php should return 200");

    let content_type = resp
        .headers()
        .get("content-type")
        .expect("/rdrand.php should have Content-Type header")
        .to_str()
        .unwrap();

    assert_eq!(
        content_type, "application/octet-stream",
        "/rdrand.php should return application/octet-stream, got: {}",
        content_type
    );

    let content_disp = resp
        .headers()
        .get("content-disposition")
        .expect("/rdrand.php should have Content-Disposition header")
        .to_str()
        .unwrap();

    assert!(
        content_disp.contains("attachment") && content_disp.contains("rand.bin"),
        "/rdrand.php should have Content-Disposition: attachment; filename=\"rand.bin\", got: {}",
        content_disp
    );
}

/// /rdrand.php returns non-empty binary data
#[tokio::test]
#[ignore = "rdrand.php not yet implemented in Rust; also returns 500 on defuse.ca"]
async fn rdrand_returns_binary_data() {
    let resp = client().get(url("/rdrand.php")).send().await.unwrap();
    assert!(resp.status().is_success());

    let bytes = resp.bytes().await.unwrap();
    assert!(
        !bytes.is_empty(),
        "/rdrand.php should return non-empty binary data"
    );
}

// =============================================================================
// /ip.php - Raw IP address
// Note: defuse.ca returns text/html content-type but body is just the IP
// =============================================================================

/// /ip.php returns plain text IP address
/// Note: We check for text/plain but defuse.ca returns text/html (legacy behavior)
#[tokio::test]
#[ignore = "ip.php not yet implemented in Rust"]
async fn ip_returns_plain_text() {
    let resp = client().get(url("/ip.php")).send().await.unwrap();
    assert!(resp.status().is_success(), "/ip.php should return 200");

    let content_type = resp
        .headers()
        .get("content-type")
        .expect("/ip.php should have Content-Type header")
        .to_str()
        .unwrap();

    // Our Rust impl should return text/plain, but defuse.ca returns text/html
    assert!(
        content_type.starts_with("text/plain") || content_type.starts_with("text/html"),
        "/ip.php should return text/plain (or text/html for legacy), got: {}",
        content_type
    );
}

/// /ip.php returns a valid IP address
#[tokio::test]
#[ignore = "ip.php not yet implemented in Rust"]
async fn ip_returns_valid_ip_address() {
    let resp = client().get(url("/ip.php")).send().await.unwrap();
    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();
    let trimmed = body.trim();

    // Should be a valid IPv4 or IPv6 address
    let is_valid_ip = trimmed.parse::<std::net::IpAddr>().is_ok();
    assert!(
        is_valid_ip,
        "/ip.php should return a valid IP address, got: {:?}",
        trimmed
    );
}

// =============================================================================
// /ip-insecure.php - IP address HTML page
// =============================================================================

/// /ip-insecure.php returns HTML with IP address
#[tokio::test]
#[ignore = "ip-insecure.php not yet implemented in Rust"]
async fn ip_insecure_returns_html() {
    let resp = client().get(url("/ip-insecure.php")).send().await.unwrap();
    assert!(
        resp.status().is_success(),
        "/ip-insecure.php should return 200"
    );

    let content_type = resp
        .headers()
        .get("content-type")
        .expect("/ip-insecure.php should have Content-Type header")
        .to_str()
        .unwrap();

    assert!(
        content_type.starts_with("text/html"),
        "/ip-insecure.php should return text/html, got: {}",
        content_type
    );
}

/// /ip-insecure.php shows "HTTP IP:" label
#[tokio::test]
#[ignore = "ip-insecure.php not yet implemented in Rust"]
async fn ip_insecure_shows_http_ip_label() {
    let resp = client().get(url("/ip-insecure.php")).send().await.unwrap();
    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("HTTP IP:"),
        "/ip-insecure.php should contain 'HTTP IP:', got: {}",
        &body.chars().take(500).collect::<String>()
    );
}

/// /ip-insecure.php uses large font styling
#[tokio::test]
#[ignore = "ip-insecure.php not yet implemented in Rust"]
async fn ip_insecure_uses_large_font() {
    let resp = client().get(url("/ip-insecure.php")).send().await.unwrap();
    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("30pt") || body.contains("30px"),
        "/ip-insecure.php should use 30pt font styling"
    );
}
