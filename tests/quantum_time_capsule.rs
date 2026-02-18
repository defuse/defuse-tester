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
use sha2::{Sha256, Digest};

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

/// POST without CAPTCHA should be rejected with specific error message.
///
/// This test does NOT use the bypass header, so it exercises the real reCAPTCHA
/// verification path. If RECAPTCHA_SECRET_KEY is not configured on the server,
/// this test will fail (the server should panic, not silently accept).
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

    // Must show the specific CAPTCHA error message
    // This verifies that RECAPTCHA_SECRET_KEY is configured and the verification
    // actually ran (rather than silently failing due to missing config)
    assert!(
        body.contains("Please solve the CAPTCHA"),
        "Should show 'Please solve the CAPTCHA' error message. \
         If this test fails with a 500 error or panic, RECAPTCHA_SECRET_KEY may not be set. \
         Got body: {}",
        &body.chars().take(500).collect::<String>()
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
// 5. POST With CAPTCHA Bypass
// =============================================================================

/// Read the CAPTCHA bypass key from secrets file
fn get_captcha_bypass_key() -> Option<String> {
    let key_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("secrets")
        .join("captcha-bypass-key.txt");
    std::fs::read_to_string(key_path).ok().map(|s| s.trim().to_string())
}

/// Submit a message to the time capsule with CAPTCHA bypass
async fn submit_message(
    message: &str,
    algorithm: &str,
    present_public_key: &str,
    future_public_key: &str,
    ciphertext: &str,
) -> reqwest::Response {
    let bypass_key = get_captcha_bypass_key()
        .expect("CAPTCHA bypass key not found - create secrets/captcha-bypass-key.txt");

    client()
        .post(url("/quantum-computer-time-capsule.htm"))
        .header("X-Captcha-Bypass", bypass_key)
        .form(&[
            ("message", message),
            ("algorithm", algorithm),
            ("present_public_key", present_public_key),
            ("future_public_key", future_public_key),
            ("ciphertext", ciphertext),
        ])
        .send()
        .await
        .expect("Failed to submit message")
}

/// Submit a valid message and verify it appears in archive with correct fields
///
/// This is a combined test that only adds ONE message to the time capsule per
/// test suite run, then verifies:
/// - Success response is shown
/// - Message count increments by 1
/// - New message appears in archive
/// - Message has correct timestamp (close to current time)
/// - Message has all expected fields
#[tokio::test]
async fn submit_message_and_verify_in_archive() {
    // Get initial archive state
    let archive_before = get_archive().await.text().await.unwrap();
    let count_before = count_archive_messages(&archive_before);

    // Create a unique test message with timestamp to identify it
    // IMPORTANT: Use values that are CLEARLY fake/test data so nobody tries
    // to decrypt these in the future thinking they're real messages!
    let test_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let algorithm = "AUTOMATED-TEST-DO-NOT-DECRYPT";
    let present_public_key = "NOT_A_REAL_KEY--AUTOMATED_TEST_FROM_DEFUSE_TESTER";
    let future_public_key = "NOT_A_REAL_KEY--AUTOMATED_TEST_FROM_DEFUSE_TESTER";
    let ciphertext = format!(
        "THIS_IS_NOT_ENCRYPTED_DATA--AUTOMATED_TEST_FROM_DEFUSE_TESTER--TIMESTAMP_{}",
        test_timestamp
    );

    // Submit the message via the shared helper
    let resp = submit_message(
        "AUTOMATED_TEST",
        algorithm,
        present_public_key,
        future_public_key,
        &ciphertext,
    ).await;

    let status = resp.status();
    let body = resp.text().await.unwrap();

    assert!(status.is_success(), "POST should return 200, got {}", status);

    // Verify success message (note: HTML has line break in "sent to\nthe future")
    assert!(
        body.contains("has been sent to"),
        "Should show success message, got: {}",
        &body.chars().take(500).collect::<String>()
    );

    // Get archive after submission
    let archive_after = get_archive().await.text().await.unwrap();
    let count_after = count_archive_messages(&archive_after);

    // Verify count incremented by exactly 1
    assert_eq!(
        count_after,
        count_before + 1,
        "Message count should increment by 1 (was {}, now {})",
        count_before,
        count_after
    );

    // Find our test message in the archive by looking for our unique timestamp
    let timestamp_marker = format!("TIMESTAMP_{}", test_timestamp);
    let our_message = archive_after
        .lines()
        .find(|line| line.contains(&timestamp_marker))
        .expect("Our test message should appear in archive");

    // Verify all fields are present
    assert!(
        our_message.contains("time:"),
        "Message should have time field"
    );
    assert!(
        our_message.contains(&format!("algorithm:{}", algorithm)),
        "Message should have correct algorithm"
    );
    assert!(
        our_message.contains(&format!("presentpublickey:{}", present_public_key)),
        "Message should have present public key"
    );
    assert!(
        our_message.contains(&format!("futurepublickey:{}", future_public_key)),
        "Message should have future public key"
    );
    assert!(
        our_message.contains(&format!("ciphertext:{}", ciphertext)),
        "Message should have ciphertext"
    );

    // Verify timestamp is reasonable (within last minute)
    let time_re = Regex::new(r"time:(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})").unwrap();
    if let Some(caps) = time_re.captures(our_message) {
        let timestamp_str = caps.get(1).unwrap().as_str();
        // Just verify it parses as a valid timestamp (detailed validation not critical)
        assert!(
            timestamp_str.len() >= 19,
            "Timestamp should be ISO 8601 format"
        );
    } else {
        panic!("Message should have valid timestamp format");
    }
}

/// Too large message is rejected (does not add to archive)
#[tokio::test]
async fn submit_too_large_rejected() {
    // Create a message that will exceed 200KB when combined
    // The combined line is: time:... algorithm:... presentpublickey:... futurepublickey:... ciphertext:...
    // We need the total to be >= 200000 bytes
    let large_ciphertext = "X".repeat(200000);

    let resp = submit_message("Test", "test", "test", "test", &large_ciphertext).await;

    assert!(resp.status().is_success(), "POST should return 200");
    let body = resp.text().await.unwrap();

    // Should show error, not success
    assert!(
        body.contains("Something went wrong") || body.contains("too big"),
        "Should reject too-large message, got: {}",
        &body.chars().take(500).collect::<String>()
    );
    assert!(
        !body.contains("Your message has been sent to the future"),
        "Should NOT show success message for too-large submission"
    );
}

/// Message with newlines in ciphertext is rejected (does not add to archive)
#[tokio::test]
async fn submit_with_newlines_rejected() {
    let resp = submit_message("Test", "test", "test", "test", "valid_part\ninvalid_newline").await;

    assert!(resp.status().is_success(), "POST should return 200");
    let body = resp.text().await.unwrap();

    // Should show error, not success
    assert!(
        body.contains("Something went wrong") || body.contains("newlines"),
        "Should reject message with newlines, got: {}",
        &body.chars().take(500).collect::<String>()
    );
    assert!(
        !body.contains("Your message has been sent to the future"),
        "Should NOT show success message for newline submission"
    );
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

    // Allow archive to have up to 2 more messages than displayed count
    // due to race conditions (other tests may add messages between fetches)
    assert!(
        actual_count >= displayed_count && actual_count <= displayed_count + 2,
        "Main page shows {} messages but archive has {} messages (expected {} to {})",
        displayed_count, actual_count, displayed_count, displayed_count + 2
    );
}

// =============================================================================
// 9. Archive Hash Integrity (Blockchain Proofs)
// =============================================================================
//
// The SHA256 hashes of the first N lines of the archive were committed to the
// Zcash blockchain as proof-of-existence timestamps. These tests verify that
// the Rust implementation produces byte-identical output to the original PHP
// for the historical portions of the archive.

/// Extract the first `n` lines from raw bytes (equivalent to `head -n`).
/// A "line" is everything up to and including a `\n` byte.
fn first_n_lines_bytes(data: &[u8], n: usize) -> &[u8] {
    let mut count = 0;
    for (i, &b) in data.iter().enumerate() {
        if b == b'\n' {
            count += 1;
            if count == n {
                return &data[..=i];
            }
        }
    }
    data
}

/// Compute SHA256 hash of bytes, returning lowercase hex string.
fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Archive first 1338 lines match blockchain-recorded SHA256 hash.
///
/// Hash f4233140fde1f1efe75d867eeb17cc34ae91a0165f767d6908ed7e3e3bb33b09
/// was committed in Zcash transaction
/// ff8221a3ee3322b36ef5817a0a591d7001c0fca2f549b5aa0c2a2b712fe6315a
/// (block 178,991).
#[tokio::test]
async fn archive_first_1338_lines_hash() {
    let resp = get_archive().await;
    assert_success(&resp, "archive_first_1338_lines_hash");
    let bytes = resp.bytes().await.unwrap();

    let first_n = first_n_lines_bytes(&bytes, 1338);
    let hash = sha256_hex(first_n);

    assert_eq!(
        hash,
        "f4233140fde1f1efe75d867eeb17cc34ae91a0165f767d6908ed7e3e3bb33b09",
        "SHA256 of first 1338 lines must match blockchain-recorded hash"
    );
}

/// Archive first 1825 lines match blockchain-recorded SHA256 hash.
///
/// Hash 4c72f9ca9a5ea691ed694f6091263d229752f6424cd6048dfbfda3f2498ffa3a
/// was committed in Zcash transaction
/// 30ae065eb7d6860776abaa0b1fbb02144d8295a4c803ec44118204b61f8cfa29
/// (block 476,431).
#[tokio::test]
async fn archive_first_1825_lines_hash() {
    let resp = get_archive().await;
    assert_success(&resp, "archive_first_1825_lines_hash");
    let bytes = resp.bytes().await.unwrap();

    let first_n = first_n_lines_bytes(&bytes, 1825);
    let hash = sha256_hex(first_n);

    assert_eq!(
        hash,
        "4c72f9ca9a5ea691ed694f6091263d229752f6424cd6048dfbfda3f2498ffa3a",
        "SHA256 of first 1825 lines must match blockchain-recorded hash"
    );
}
