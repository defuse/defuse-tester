//! Tests for special PHP endpoints
//!
//! These are utility endpoints that don't fit the normal page pattern:
//! - /s.php - "Shout" page (display large text)
//! - /ip.php - Raw IP address
//! - /ip-insecure.php - IP address HTML page
//! - /getmyip.php - IP/hostname/user-agent display

mod common;

use common::{client, url};
use reqwest::header::USER_AGENT;

// =============================================================================
// /s.php - "Shout" page
// =============================================================================

/// /s.php with no params shows a form
#[tokio::test]
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

    // "hello" in base64 (no padding) is "aGVsbG8"
    assert!(
        location.ends_with("/s.php?s=aGVsbG8"),
        "/s.php?e=hello should redirect to /s.php?s=aGVsbG8, got: {}",
        location
    );
}

/// /s.php?s=base64text displays decoded text in large font
#[tokio::test]
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
        body.contains("300pt"),
        "/s.php should use 300pt font styling"
    );
}

/// /s.php XSS test: HTML special chars in decoded base64 must be escaped
#[tokio::test]
async fn shout_xss_escaped() {
    // base64("<script>alert(1)</script>") = "PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg"
    let payload_b64 = "PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg";
    let resp = client()
        .get(url(&format!("/s.php?s={}", payload_b64)))
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success(), "/s.php?s=... should return 200");
    let body = resp.text().await.unwrap();

    // Must NOT contain unescaped script tag
    assert!(
        !body.contains("<script>"),
        "/s.php should HTML-escape decoded text, but found raw <script> tag"
    );
    // Should contain the escaped form
    assert!(
        body.contains("&lt;script&gt;"),
        "/s.php should contain escaped &lt;script&gt;, got: {}",
        &body.chars().take(500).collect::<String>()
    );
}

// =============================================================================
// /ip.php - Raw IP address
// =============================================================================

/// /ip.php should return text/plain content-type since the body is just an IP address.
///
/// BUG: PHP returns text/html content-type even though the response contains no HTML,
/// just a plain IP address string.
#[tokio::test]
async fn ip_returns_plain_text() {
    let resp = client().get(url("/ip.php")).send().await.unwrap();
    assert!(resp.status().is_success(), "/ip.php should return 200");

    let content_type = resp
        .headers()
        .get("content-type")
        .expect("/ip.php should have Content-Type header")
        .to_str()
        .unwrap();

    assert!(
        content_type.starts_with("text/plain"),
        "/ip.php should return text/plain content-type for plain IP address, got: {}",
        content_type
    );
}

/// /ip.php returns a valid IP address
#[tokio::test]
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
async fn ip_insecure_uses_large_font() {
    let resp = client().get(url("/ip-insecure.php")).send().await.unwrap();
    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("30pt"),
        "/ip-insecure.php should use 30pt font styling"
    );
}

// =============================================================================
// /getmyip.php - IP/hostname/user-agent display
// =============================================================================

/// /getmyip.php returns 200 OK
#[tokio::test]
async fn getmyip_returns_200() {
    let resp = client().get(url("/getmyip.php")).send().await.unwrap();
    assert!(
        resp.status().is_success(),
        "/getmyip.php should return 200, got {}",
        resp.status()
    );
}

/// /getmyip.php returns HTML content type
#[tokio::test]
async fn getmyip_content_type_html() {
    let resp = client().get(url("/getmyip.php")).send().await.unwrap();
    assert!(resp.status().is_success());

    let content_type = resp
        .headers()
        .get("content-type")
        .expect("/getmyip.php should have Content-Type header")
        .to_str()
        .unwrap();

    assert!(
        content_type.starts_with("text/html"),
        "/getmyip.php should return text/html, got: {}",
        content_type
    );
}

/// /getmyip.php shows an IP address
#[tokio::test]
async fn getmyip_shows_ip() {
    let resp = client().get(url("/getmyip.php")).send().await.unwrap();
    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();

    // Check for IPv4 pattern
    let has_ipv4 = regex::Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}")
        .unwrap()
        .is_match(&body);

    // Check for IPv6 pattern (at least 3 colon-separated hex groups)
    let has_ipv6 = regex::Regex::new(r"[0-9a-fA-F]{1,4}(:[0-9a-fA-F]{1,4}){2,7}")
        .unwrap()
        .is_match(&body);

    assert!(
        has_ipv4 || has_ipv6,
        "/getmyip.php should display an IP address"
    );
}

/// /getmyip.php shows hostname section
#[tokio::test]
async fn getmyip_shows_hostname() {
    let resp = client().get(url("/getmyip.php")).send().await.unwrap();
    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();

    assert!(
        body.contains("Hostname") || body.contains("hostname") || body.contains("HOSTNAME"),
        "/getmyip.php should display hostname section"
    );
}

/// /getmyip.php shows user agent (displayed without label, just the raw UA string)
#[tokio::test]
async fn getmyip_shows_user_agent() {
    // Send with a known user agent
    let test_ua = "TestBrowser/1.0";
    let resp = client()
        .get(url("/getmyip.php"))
        .header(USER_AGENT, test_ua)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();

    // The page displays the user agent (without a label)
    assert!(
        body.contains(test_ua),
        "/getmyip.php should display the user agent string"
    );
}

/// /getmyip.php displays the custom user agent we send
#[tokio::test]
async fn getmyip_custom_user_agent() {
    let custom_ua = "DefuseTestSuite/1.0 (Testing getmyip.php)";

    let resp = client()
        .get(url("/getmyip.php"))
        .header(USER_AGENT, custom_ua)
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();

    assert!(
        body.contains(custom_ua),
        "/getmyip.php should display the custom User-Agent '{}', body: {}",
        custom_ua,
        &body.chars().take(500).collect::<String>()
    );
}

/// /getmyip.php XSS test: HTML special chars in User-Agent must be escaped
#[tokio::test]
async fn getmyip_xss_user_agent_escaped() {
    let malicious_ua = "<script>alert(1)</script>";
    let resp = client()
        .get(url("/getmyip.php"))
        .header(USER_AGENT, malicious_ua)
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();

    // Must NOT contain unescaped script tag
    assert!(
        !body.contains("<script>"),
        "/getmyip.php should HTML-escape the User-Agent, but found raw <script> tag"
    );
    // Should contain the escaped form
    assert!(
        body.contains("&lt;script&gt;"),
        "/getmyip.php should contain escaped &lt;script&gt;"
    );
}
