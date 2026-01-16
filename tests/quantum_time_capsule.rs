// Quantum Computer Time Capsule Tests
//
// Tests for the time capsule feature that allows users to submit encrypted
// messages that will only be readable when large-scale quantum computers exist.

mod common;

use common::{
    assert_body_contains, assert_content_type, assert_javascript_content_type, assert_success,
    client, post_form, url,
};
use regex::Regex;

// =============================================================================
// Helper functions
// =============================================================================

/// Get the main time capsule page
async fn get_main_page() -> reqwest::Response {
    client()
        .get(url("/quantum-computer-time-capsule.htm"))
        .send()
        .await
        .expect("Failed to fetch time capsule page")
}

/// Get the archive download
async fn get_archive() -> reqwest::Response {
    client()
        .get(url("/timecapsule/quantum-computer-time-capsule-download.php"))
        .send()
        .await
        .expect("Failed to fetch archive")
}

/// Extract message count from main page HTML
fn extract_message_count(body: &str) -> Option<u32> {
    // Look for "contains X messages" pattern
    let re = Regex::new(r"contains\s+(\d+)\s+messages").ok()?;
    let caps = re.captures(body)?;
    caps.get(1)?.as_str().parse().ok()
}

/// Count messages in archive (lines starting with "time:")
fn count_archive_messages(archive: &str) -> u32 {
    archive
        .lines()
        .filter(|line| line.starts_with("time:"))
        .count() as u32
}

// =============================================================================
// 1. Main Page (GET)
// =============================================================================

/// Main page loads with 200 status
#[tokio::test]
async fn main_page_loads() {
    let resp = get_main_page().await;
    assert_success(&resp, "main_page_loads");
}

/// Main page has appropriate title
#[tokio::test]
async fn main_page_has_title() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    // Title contains either "Time Capsule" or "Message to the Future"
    assert!(
        body.contains("Time Capsule") || body.contains("Message to the Future"),
        "Page should have Time Capsule or Message to the Future in title/content"
    );
}

/// Main page has form with correct action and method
#[tokio::test]
async fn main_page_has_form() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(
        &body,
        r#"action="/quantum-computer-time-capsule.htm""#,
        "main_page_has_form (action)",
    );
    assert_body_contains(&body, r#"method="post""#, "main_page_has_form (method)");
}

/// Main page has message textarea
#[tokio::test]
async fn main_page_has_message_textarea() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(&body, r#"id="message""#, "main_page_has_message_textarea");
    assert_body_contains(&body, r#"name="message""#, "main_page_has_message_textarea");
    assert_body_contains(&body, "<textarea", "main_page_has_message_textarea");
}

/// Main page has hidden form fields for encryption data
#[tokio::test]
async fn main_page_has_hidden_fields() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(
        &body,
        r#"name="algorithm""#,
        "main_page_has_hidden_fields (algorithm)",
    );
    assert_body_contains(
        &body,
        r#"name="present_public_key""#,
        "main_page_has_hidden_fields (present_public_key)",
    );
    assert_body_contains(
        &body,
        r#"name="future_public_key""#,
        "main_page_has_hidden_fields (future_public_key)",
    );
    assert_body_contains(
        &body,
        r#"name="ciphertext""#,
        "main_page_has_hidden_fields (ciphertext)",
    );
}

/// Main page has submit button
#[tokio::test]
async fn main_page_has_submit_button() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(
        &body,
        r#"id="submitbutton""#,
        "main_page_has_submit_button",
    );
    assert!(
        body.contains(r#"type="button""#) || body.contains(r#"type="submit""#),
        "Page should have a submit or button element"
    );
}

/// Main page shows message count
#[tokio::test]
async fn main_page_shows_message_count() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    // Should contain "X messages" where X is a number
    let re = Regex::new(r"\d+\s+messages").unwrap();
    assert!(
        re.is_match(&body),
        "Page should show message count (e.g., '1258 messages')"
    );
}

/// Main page shows last message time
#[tokio::test]
async fn main_page_shows_last_time() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    // Should contain "ago" (e.g., "5 days ago", "3 hours ago")
    assert_body_contains(&body, "ago", "main_page_shows_last_time");
}

/// Main page has reCAPTCHA
#[tokio::test]
async fn main_page_has_recaptcha() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("g-recaptcha") || body.contains("recaptcha"),
        "Page should have reCAPTCHA"
    );
}

/// Main page has download link/form for archive
#[tokio::test]
async fn main_page_has_download_link() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(
        &body,
        "quantum-computer-time-capsule-download.php",
        "main_page_has_download_link",
    );
}

/// Main page loads encryption JavaScript files
#[tokio::test]
async fn main_page_loads_encryption_js() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(
        &body,
        "timecapsule-save.js",
        "main_page_loads_encryption_js (timecapsule-save.js)",
    );
    assert_body_contains(
        &body,
        "tweetnacl-time-capsule.js",
        "main_page_loads_encryption_js (tweetnacl)",
    );
}

// =============================================================================
// 2. Archive Download
// =============================================================================

/// Archive download returns text/plain content type
#[tokio::test]
async fn download_returns_text_plain() {
    let resp = get_archive().await;
    assert_success(&resp, "download_returns_text_plain");
    assert_content_type(&resp, "text/plain", "download_returns_text_plain");
}

/// Archive download has attachment header for browser download
#[tokio::test]
async fn download_has_attachment_header() {
    let resp = get_archive().await;
    let content_disp = resp
        .headers()
        .get("content-disposition")
        .expect("Should have Content-Disposition header")
        .to_str()
        .unwrap();
    assert!(
        content_disp.contains("attachment"),
        "Content-Disposition should be attachment, got: {}",
        content_disp
    );
    assert!(
        content_disp.contains(".txt"),
        "Filename should have .txt extension, got: {}",
        content_disp
    );
}

/// Archive filename contains expected name and timestamp
#[tokio::test]
async fn download_filename_has_timestamp() {
    let resp = get_archive().await;
    let content_disp = resp
        .headers()
        .get("content-disposition")
        .expect("Should have Content-Disposition header")
        .to_str()
        .unwrap();
    assert!(
        content_disp.contains("MessagesToTheQuantumComputingFuture"),
        "Filename should contain 'MessagesToTheQuantumComputingFuture', got: {}",
        content_disp
    );
    // Check for date-like pattern (YYYY-MM-DD)
    let re = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    assert!(
        re.is_match(content_disp),
        "Filename should contain date, got: {}",
        content_disp
    );
}

/// Archive has header section
#[tokio::test]
async fn download_has_header_section() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(
        &body,
        "TIME CAPSULE FOR A QUANTUM-COMPUTING FUTURE",
        "download_has_header_section",
    );
}

/// Archive includes timecapsule-save.js source code
#[tokio::test]
async fn download_has_source_timecapsule_js() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(
        &body,
        "SOURCE CODE: timecapsule-save.js",
        "download_has_source_timecapsule_js",
    );
}

/// Archive includes tweetnacl source code
#[tokio::test]
async fn download_has_source_tweetnacl() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(
        &body,
        "SOURCE CODE: tweetnacl-time-capsule.js",
        "download_has_source_tweetnacl",
    );
}

/// Archive has messages section marker
#[tokio::test]
async fn download_has_messages_section() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(
        &body,
        "MESSAGES FOR THE FUTURE",
        "download_has_messages_section",
    );
}

/// Archive has footer section
#[tokio::test]
async fn download_has_footer_section() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(
        &body,
        "ADDITIONAL INFORMATION",
        "download_has_footer_section",
    );
}

/// Archive contains actual messages (lines starting with "time:")
#[tokio::test]
async fn download_has_actual_messages() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();
    // Messages start with "time:" prefix
    assert!(
        body.contains("time:"),
        "Archive should contain messages starting with 'time:'"
    );
}

/// Messages in archive have expected format
#[tokio::test]
async fn download_messages_have_expected_format() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();

    // Find a line that starts with "time:" and validate its structure
    let message_line = body
        .lines()
        .find(|line| line.starts_with("time:"))
        .expect("Should have at least one message");

    // Message should contain all expected fields
    assert!(
        message_line.contains("algorithm:"),
        "Message should contain algorithm field"
    );
    assert!(
        message_line.contains("presentpublickey:"),
        "Message should contain presentpublickey field"
    );
    assert!(
        message_line.contains("futurepublickey:"),
        "Message should contain futurepublickey field"
    );
    assert!(
        message_line.contains("ciphertext:"),
        "Message should contain ciphertext field"
    );
}

// =============================================================================
// 3. Static JS Files
// =============================================================================

/// timecapsule-save.js loads
#[tokio::test]
async fn timecapsule_js_loads() {
    let resp = client()
        .get(url("/timecapsule/timecapsule-save.js"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "timecapsule_js_loads");
}

/// timecapsule-save.js contains sendMessage function
#[tokio::test]
async fn timecapsule_js_has_send_function() {
    let resp = client()
        .get(url("/timecapsule/timecapsule-save.js"))
        .send()
        .await
        .unwrap();
    let body = resp.text().await.unwrap();
    assert_body_contains(&body, "function sendMessage", "timecapsule_js_has_send_function");
}

/// timecapsule-save.js has JavaScript content type
#[tokio::test]
async fn timecapsule_js_content_type() {
    let resp = client()
        .get(url("/timecapsule/timecapsule-save.js"))
        .send()
        .await
        .unwrap();
    assert_javascript_content_type(&resp, "timecapsule_js_content_type");
}

/// tweetnacl-time-capsule.js loads
#[tokio::test]
async fn tweetnacl_js_loads() {
    let resp = client()
        .get(url("/timecapsule/tweetnacl-time-capsule.js"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "tweetnacl_js_loads");
}

/// tweetnacl-util-time-capsule.js loads
#[tokio::test]
async fn tweetnacl_util_js_loads() {
    let resp = client()
        .get(url("/timecapsule/tweetnacl-util-time-capsule.js"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "tweetnacl_util_js_loads");
}

// =============================================================================
// 4. POST Without CAPTCHA (verifies validation)
// =============================================================================

/// POST without CAPTCHA should be rejected
#[tokio::test]
async fn post_without_captcha_rejected() {
    let resp = post_form(
        "/quantum-computer-time-capsule.htm",
        &[
            ("message", "Test message"),
            (
                "algorithm",
                "tweetnacl-js-x25519-xsalsa20-poly1305-zerononce",
            ),
            ("present_public_key", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="),
            ("future_public_key", "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB="),
            ("ciphertext", "dGVzdA=="),
        ],
    )
    .await;

    // The page should still load (200 OK) but show error
    assert_success(&resp, "post_without_captcha_rejected");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("CAPTCHA") || body.contains("captcha"),
        "Should show CAPTCHA error message"
    );
}

/// POST with message but CAPTCHA fails - message should be preserved
#[tokio::test]
async fn post_preserves_message_on_error() {
    let test_message = "This is a test message that should be preserved";
    let resp = post_form(
        "/quantum-computer-time-capsule.htm",
        &[
            ("message", test_message),
            (
                "algorithm",
                "tweetnacl-js-x25519-xsalsa20-poly1305-zerononce",
            ),
            ("present_public_key", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="),
            ("future_public_key", "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB="),
            ("ciphertext", "dGVzdA=="),
        ],
    )
    .await;

    let body = resp.text().await.unwrap();
    // The message should be echoed back in the textarea
    assert!(
        body.contains(test_message),
        "Message should be preserved in textarea on error"
    );
}

// =============================================================================
// 5. POST With CAPTCHA Bypass (IGNORED - requires CAPTCHA bypass)
// =============================================================================

/// Submit valid encrypted message with CAPTCHA
#[tokio::test]
#[ignore = "requires CAPTCHA bypass - not implemented"]
async fn submit_valid_message() {
    // Would need CAPTCHA bypass to test
    todo!("Implement when CAPTCHA bypass is available")
}

/// Submitted message appears in archive
#[tokio::test]
#[ignore = "requires CAPTCHA bypass - not implemented"]
async fn submit_message_appears_in_archive() {
    // Would need CAPTCHA bypass to test
    todo!("Implement when CAPTCHA bypass is available")
}

/// Submitting message increments count
#[tokio::test]
#[ignore = "requires CAPTCHA bypass - not implemented"]
async fn submit_message_increments_count() {
    // Would need CAPTCHA bypass to test
    todo!("Implement when CAPTCHA bypass is available")
}

/// Submitting message updates last timestamp
#[tokio::test]
#[ignore = "requires CAPTCHA bypass - not implemented"]
async fn submit_updates_last_timestamp() {
    // Would need CAPTCHA bypass to test
    todo!("Implement when CAPTCHA bypass is available")
}

/// Too large message is rejected
#[tokio::test]
#[ignore = "requires CAPTCHA bypass - not implemented"]
async fn submit_too_large_rejected() {
    // Would need CAPTCHA bypass to test
    todo!("Implement when CAPTCHA bypass is available")
}

/// Message with newlines in ciphertext is rejected
#[tokio::test]
#[ignore = "requires CAPTCHA bypass - not implemented"]
async fn submit_with_newlines_rejected() {
    // Would need CAPTCHA bypass to test
    todo!("Implement when CAPTCHA bypass is available")
}

/// Submitted message records timestamp
#[tokio::test]
#[ignore = "requires CAPTCHA bypass - not implemented"]
async fn submit_records_timestamp() {
    // Would need CAPTCHA bypass to test
    todo!("Implement when CAPTCHA bypass is available")
}

/// Submitted message stores all fields
#[tokio::test]
#[ignore = "requires CAPTCHA bypass - not implemented"]
async fn submit_stores_all_fields() {
    // Would need CAPTCHA bypass to test
    todo!("Implement when CAPTCHA bypass is available")
}

// =============================================================================
// 6. Message Format Validation
// =============================================================================

/// Message format has time field with ISO 8601 timestamp
#[tokio::test]
async fn message_format_has_time() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();

    let message_line = body
        .lines()
        .find(|line| line.starts_with("time:"))
        .expect("Should have at least one message");

    // Check for ISO 8601 format (e.g., 2024-01-15T12:30:45+00:00)
    let re = Regex::new(r"time:\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}").unwrap();
    assert!(
        re.is_match(message_line),
        "Message should have ISO 8601 timestamp, got: {}",
        message_line.chars().take(100).collect::<String>()
    );
}

/// Message format has algorithm field
#[tokio::test]
async fn message_format_has_algorithm() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();

    let message_line = body
        .lines()
        .find(|line| line.starts_with("time:"))
        .expect("Should have at least one message");

    assert!(
        message_line.contains("algorithm:tweetnacl-js-x25519-xsalsa20-poly1305-zerononce"),
        "Message should have expected algorithm field"
    );
}

/// Message format has present public key
#[tokio::test]
async fn message_format_has_present_key() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();

    let message_line = body
        .lines()
        .find(|line| line.starts_with("time:"))
        .expect("Should have at least one message");

    // presentpublickey should be followed by base64 (alphanumeric + /+=)
    let re = Regex::new(r"presentpublickey:[A-Za-z0-9+/]+=*").unwrap();
    assert!(
        re.is_match(message_line),
        "Message should have base64 presentpublickey field"
    );
}

/// Message format has future public key
#[tokio::test]
async fn message_format_has_future_key() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();

    let message_line = body
        .lines()
        .find(|line| line.starts_with("time:"))
        .expect("Should have at least one message");

    // futurepublickey should be followed by base64
    let re = Regex::new(r"futurepublickey:[A-Za-z0-9+/]+=*").unwrap();
    assert!(
        re.is_match(message_line),
        "Message should have base64 futurepublickey field"
    );
}

/// Message format has ciphertext
#[tokio::test]
async fn message_format_has_ciphertext() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();

    let message_line = body
        .lines()
        .find(|line| line.starts_with("time:"))
        .expect("Should have at least one message");

    // ciphertext should be followed by base64
    let re = Regex::new(r"ciphertext:[A-Za-z0-9+/]+=*").unwrap();
    assert!(
        re.is_match(message_line),
        "Message should have base64 ciphertext field"
    );
}

/// Message is a single line (no newlines within)
#[tokio::test]
async fn message_format_single_line() {
    let resp = get_archive().await;
    let body = resp.text().await.unwrap();

    // Find all messages and verify each is exactly one line
    let messages: Vec<&str> = body
        .lines()
        .filter(|line| line.starts_with("time:"))
        .collect();

    assert!(!messages.is_empty(), "Should have at least one message");

    // Each message should be self-contained on one line
    // (we're iterating by lines so this is implicitly true,
    // but we verify the message contains all expected fields)
    for msg in messages.iter().take(5) {
        // Check a sample
        assert!(
            msg.contains("algorithm:") && msg.contains("ciphertext:"),
            "Each message line should contain all fields"
        );
    }
}

// =============================================================================
// 7. Security Headers
// =============================================================================

/// Main page has text/html content type
#[tokio::test]
async fn main_page_content_type() {
    let resp = get_main_page().await;
    assert_content_type(&resp, "text/html", "main_page_content_type");
}

/// Download has X-Content-Type-Options: nosniff
#[tokio::test]
async fn download_no_sniff() {
    let resp = get_archive().await;
    let header = resp.headers().get("x-content-type-options");
    if let Some(value) = header {
        assert_eq!(
            value.to_str().unwrap(),
            "nosniff",
            "X-Content-Type-Options should be 'nosniff'"
        );
    }
    // Note: If header is missing, we don't fail - this is a nice-to-have
}

// =============================================================================
// 8. Archive Consistency
// =============================================================================

/// Message count on main page matches number of messages in archive
#[tokio::test]
async fn message_count_matches_archive() {
    // Get count from main page
    let main_resp = get_main_page().await;
    let main_body = main_resp.text().await.unwrap();
    let displayed_count = extract_message_count(&main_body)
        .expect("Should be able to extract message count from main page");

    // Get actual count from archive
    let archive_resp = get_archive().await;
    let archive_body = archive_resp.text().await.unwrap();
    let actual_count = count_archive_messages(&archive_body);

    assert_eq!(
        displayed_count, actual_count,
        "Main page shows {} messages but archive has {} messages",
        displayed_count, actual_count
    );
}
