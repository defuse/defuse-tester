//! BREACH attack mitigation tests
//!
//! Tests that breach_visual_html() properly protects output with
//! random HTML comments and zero-width spaces.

mod common;

use common::{client, url, assert_success};
use regex::Regex;

/// Test that breach_visual_html() output is properly protected
///
/// The BREACH section should contain:
/// 1. HTML comments with hex content like <!-- [0-9a-f]+ -->
/// 2. Zero-width space entities (&#8203; or &#x200b;)
/// 3. The actual visible letters S, a, m, p, l, e (interspersed with padding)
#[tokio::test]
async fn breach_visual_html_output_is_protected() {
    let resp = client()
        .get(url("/mitigating-breach-tls-attack-in-php.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "breach_visual_html_output_is_protected");
    let body = resp.text().await.unwrap();

    // Find the BREACH demonstration section
    // It's in a <div style="padding-left: 30px;"> containing <h2> with "Sample Header"
    let breach_section_start = body.find("<div style=\"padding-left: 30px;\">")
        .expect("Should find BREACH demo section");
    let breach_section_end = body[breach_section_start..].find("</div>")
        .map(|i| breach_section_start + i + 6)
        .expect("Should find end of BREACH section");
    let breach_section = &body[breach_section_start..breach_section_end];

    // 1. Check for HTML comments with hex content
    let comment_regex = Regex::new(r"<!--\s*[0-9a-f]+\s*-->").unwrap();
    assert!(
        comment_regex.is_match(breach_section),
        "BREACH section should contain HTML comments with hex content like <!-- abc123 -->"
    );

    // 2. Check for zero-width space entities
    let has_zwsp = breach_section.contains("&#8203;") || breach_section.contains("&#x200b;");
    assert!(
        has_zwsp,
        "BREACH section should contain zero-width space entities (&#8203; or &#x200b;)"
    );

    // 3. Check that the visible letters appear (they're interspersed with padding)
    // The h2 should contain S, a, m, p, l, e as individual visible characters
    assert!(
        breach_section.contains(">S<") || breach_section.contains(">S&#")
            || breach_section.contains("S&#") || breach_section.contains("-->S"),
        "BREACH section should contain visible letter 'S' from 'Sample'"
    );

    // 4. Count HTML comments to ensure there's substantial padding
    let comment_count = comment_regex.find_iter(breach_section).count();
    assert!(
        comment_count >= 10,
        "BREACH section should have many HTML comments for padding, found {}",
        comment_count
    );
}
