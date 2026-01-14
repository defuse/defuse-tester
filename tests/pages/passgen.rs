//! Tests for /passgen.htm - Password Generator
//!
//! Verifies:
//! - Cache-control headers disable caching
//! - Passwords are well-formed (correct character sets)
//! - Passwords change on each request

use crate::common::{client, url};

/// Extract password from HTML table cell following a label
fn extract_password(html: &str, label: &str) -> Option<String> {
    // Look for pattern: <strong>LABEL:</strong></td>\n            <td>PASSWORD</td>
    let pattern = format!("<strong>{}:</strong></td>", label);
    let start = html.find(&pattern)?;
    let after_label = &html[start + pattern.len()..];

    // Find the next <td> and extract its content
    let td_start = after_label.find("<td>")?;
    let content_start = td_start + 4;
    let td_end = after_label[content_start..].find("</td>")?;

    let password = &after_label[content_start..content_start + td_end];
    // Decode HTML entities
    let password = password
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">");

    Some(password)
}

/// Check if string contains only alphanumeric characters
fn is_alphanumeric(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Check if string contains only hex characters (uppercase)
fn is_hex(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_hexdigit() && (c.is_ascii_digit() || c.is_ascii_uppercase()))
}

/// Check if string contains only printable ASCII characters
fn is_printable_ascii(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii() && !c.is_ascii_control())
}

#[tokio::test]
#[ignore = "passgen not yet implemented in Rust"]
async fn cache_control_disables_caching() {
    let resp = client().get(url("/passgen.htm")).send().await.unwrap();
    assert!(resp.status().is_success(), "/passgen.htm should return 200");

    let cache_control = resp
        .headers()
        .get("cache-control")
        .expect("Cache-Control header should be present")
        .to_str()
        .unwrap();

    assert!(
        cache_control.contains("no-cache") || cache_control.contains("no-store"),
        "Cache-Control should disable caching, got: {}",
        cache_control
    );
}

#[tokio::test]
#[ignore = "passgen not yet implemented in Rust"]
async fn passwords_are_well_formed() {
    let resp = client().get(url("/passgen.htm")).send().await.unwrap();
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();

    // Extract passwords
    let ascii = extract_password(&body, "ASCII")
        .expect("Should find ASCII password");
    let alphanum = extract_password(&body, "Alpha-numeric")
        .expect("Should find Alpha-numeric password");
    let hex = extract_password(&body, "Hex")
        .expect("Should find Hex password");

    // Check ASCII password
    assert!(
        is_printable_ascii(&ascii),
        "ASCII password should contain only printable ASCII: {:?}",
        ascii
    );
    assert!(
        ascii.len() >= 32,
        "ASCII password should be at least 32 chars, got {}",
        ascii.len()
    );

    // Check Alpha-numeric password
    assert!(
        is_alphanumeric(&alphanum),
        "Alpha-numeric password should contain only [A-Za-z0-9]: {:?}",
        alphanum
    );
    assert!(
        alphanum.len() >= 32,
        "Alpha-numeric password should be at least 32 chars, got {}",
        alphanum.len()
    );

    // Check Hex password
    assert!(
        is_hex(&hex),
        "Hex password should contain only [0-9A-F]: {:?}",
        hex
    );
    assert!(
        hex.len() >= 32,
        "Hex password should be at least 32 chars, got {}",
        hex.len()
    );
}

#[tokio::test]
#[ignore = "passgen not yet implemented in Rust"]
async fn passwords_change_on_refresh() {
    // First request
    let resp1 = client().get(url("/passgen.htm")).send().await.unwrap();
    assert!(resp1.status().is_success());
    let body1 = resp1.text().await.unwrap();

    let ascii1 = extract_password(&body1, "ASCII").expect("Should find ASCII password");
    let alphanum1 = extract_password(&body1, "Alpha-numeric").expect("Should find Alpha-numeric password");
    let hex1 = extract_password(&body1, "Hex").expect("Should find Hex password");

    // Second request
    let resp2 = client().get(url("/passgen.htm")).send().await.unwrap();
    assert!(resp2.status().is_success());
    let body2 = resp2.text().await.unwrap();

    let ascii2 = extract_password(&body2, "ASCII").expect("Should find ASCII password");
    let alphanum2 = extract_password(&body2, "Alpha-numeric").expect("Should find Alpha-numeric password");
    let hex2 = extract_password(&body2, "Hex").expect("Should find Hex password");

    // All passwords should be different
    assert_ne!(
        ascii1, ascii2,
        "ASCII password should change on refresh"
    );
    assert_ne!(
        alphanum1, alphanum2,
        "Alpha-numeric password should change on refresh"
    );
    assert_ne!(
        hex1, hex2,
        "Hex password should change on refresh"
    );
}
