mod common;
// Tests for pastebin functionality
//
// Endpoints tested:
// - GET /pastebin.htm - Main form page
// - POST /bin/add.php - Create paste
// - GET /b/{key} - View paste (HTML)
// - GET /b/{key}?raw=true - Raw paste content
// - GET /bin/ and /b/ - Redirects to main page

use common::{client, url};

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate unique test content to avoid collisions between test runs
fn test_content(base: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{} [test-{}]", base, ts)
}

/// Create a paste with default options, returns the URL key
async fn create_paste(content: &str) -> String {
    create_paste_opts(content, false, None, false)
        .await
        .expect("Failed to create paste")
}

/// Create a paste with full options, returns the URL key or None if failed
async fn create_paste_opts(
    content: &str,
    jscrypt: bool,
    lifetime: Option<u32>,
    shorturl: bool,
) -> Option<String> {
    let resp = create_paste_response(content, jscrypt, lifetime, shorturl).await;
    extract_key_from_response(&resp)
}

/// Create a paste and return the raw response (for checking status/headers)
async fn create_paste_response(
    content: &str,
    jscrypt: bool,
    lifetime: Option<u32>,
    shorturl: bool,
) -> reqwest::Response {
    let mut form: Vec<(&str, String)> = vec![("paste", content.to_string())];
    if jscrypt {
        form.push(("jscrypt", "yes".to_string()));
    }
    if let Some(lt) = lifetime {
        form.push(("lifetime", lt.to_string()));
    }
    if shorturl {
        form.push(("shorturl", "yes".to_string()));
    }

    client()
        .post(url("/bin/add.php"))
        .form(&form)
        .send()
        .await
        .expect("Failed to send create paste request")
}

/// Create a paste with a string lifetime value (for testing invalid inputs)
async fn create_paste_with_str_lifetime(content: &str, lifetime: &str) -> reqwest::Response {
    let form: Vec<(&str, &str)> = vec![("paste", content), ("lifetime", lifetime)];

    client()
        .post(url("/bin/add.php"))
        .form(&form)
        .send()
        .await
        .expect("Failed to send create paste request")
}

/// Extract key from response Location header
fn extract_key_from_response(resp: &reqwest::Response) -> Option<String> {
    resp.headers()
        .get("location")
        .and_then(|loc| loc.to_str().ok())
        .and_then(extract_key)
}

/// Extract the key from a location like "/b/abc123"
fn extract_key(location: &str) -> Option<String> {
    if location.contains("/b/") {
        location
            .rsplit("/b/")
            .next()
            .map(|s| s.trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty())
    } else {
        None
    }
}

/// GET /b/{key} and return response
async fn get_paste(key: &str) -> reqwest::Response {
    client()
        .get(url(&format!("/b/{}", key)))
        .send()
        .await
        .expect("Failed to GET paste")
}

/// GET /b/{key}?raw=true and return response
async fn get_paste_raw(key: &str) -> reqwest::Response {
    client()
        .get(url(&format!("/b/{}?raw=true", key)))
        .send()
        .await
        .expect("Failed to GET raw paste")
}

/// Extract textarea content from paste view HTML
fn extract_textarea_content(html: &str) -> Option<String> {
    // Look for <textarea name="paste"...>content</textarea>
    let start_pattern = "name=\"paste\"";
    let start = html.find(start_pattern)?;
    let after_name = &html[start..];

    // Find the closing > of the textarea tag
    let tag_end = after_name.find('>')?;
    let content_start = tag_end + 1;

    // Find </textarea>
    let content_end = after_name.find("</textarea>")?;

    if content_start >= content_end {
        return Some(String::new());
    }

    let content = &after_name[content_start..content_end];

    // Strip leading newline if present (HTML formatting artifact)
    let content = content.strip_prefix('\n').unwrap_or(content);

    Some(html_unescape(content))
}

/// Unescape HTML entities - order matters!
fn html_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&amp;", "&")
}

/// Normalize line endings the way the server does
fn normalize_line_endings(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

/// Check if string is alphanumeric only
fn is_alphanumeric(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_alphanumeric())
}

// ============================================================================
// 1. Main Page Tests (/pastebin.htm)
// ============================================================================

#[tokio::test]
async fn main_page_loads() {
    let resp = client().get(url("/pastebin.htm")).send().await.unwrap();
    assert!(
        resp.status().is_success(),
        "/pastebin.htm should return 200, got {}",
        resp.status()
    );
}

#[tokio::test]
async fn main_page_has_form_fields() {
    let resp = client().get(url("/pastebin.htm")).send().await.unwrap();
    let body = resp.text().await.unwrap();

    assert!(
        body.contains("name=\"paste\""),
        "Should have paste textarea"
    );
    assert!(
        body.contains("name=\"lifetime\""),
        "Should have lifetime select"
    );
    assert!(
        body.contains("name=\"shorturl\""),
        "Should have shorturl checkbox"
    );
    assert!(
        body.contains("type=\"submit\"") || body.contains("type='submit'"),
        "Should have submit button"
    );
}

#[tokio::test]
async fn main_page_has_lifetime_options() {
    let resp = client().get(url("/pastebin.htm")).send().await.unwrap();
    let body = resp.text().await.unwrap();

    // Check all lifetime values are present
    let expected_lifetimes = [
        "15552000", // 6 months
        "2592000",  // 30 days
        "864000",   // 10 days
        "86400",    // 1 day
        "3600",     // 60 min
        "600",      // 10 min
    ];

    for lt in expected_lifetimes {
        assert!(
            body.contains(&format!("value=\"{}\"", lt)),
            "Should have lifetime option for {} seconds",
            lt
        );
    }
}

// ============================================================================
// 2. Redirect Tests
// ============================================================================

#[tokio::test]
async fn bin_index_redirects() {
    let resp = client().get(url("/bin/")).send().await.unwrap();
    assert_eq!(
        resp.status().as_u16(),
        301,
        "/bin/ should redirect, got {}",
        resp.status()
    );
    let location = resp
        .headers()
        .get("location")
        .expect("Should have Location header")
        .to_str()
        .unwrap();
    assert!(
        location.contains("pastebin.htm"),
        "Should redirect to pastebin.htm, got {}",
        location
    );
}

#[tokio::test]
async fn b_index_redirects() {
    let resp = client().get(url("/b/")).send().await.unwrap();
    assert_eq!(
        resp.status().as_u16(),
        301,
        "/b/ should redirect, got {}",
        resp.status()
    );
    let location = resp
        .headers()
        .get("location")
        .expect("Should have Location header")
        .to_str()
        .unwrap();
    assert!(
        location.contains("pastebin.htm"),
        "Should redirect to pastebin.htm, got {}",
        location
    );
}

#[tokio::test]
async fn b_no_slash_redirects() {
    let resp = client().get(url("/b")).send().await.unwrap();
    assert_eq!(
        resp.status().as_u16(),
        301,
        "/b (no trailing slash) should redirect, got {}",
        resp.status()
    );
    let location = resp
        .headers()
        .get("location")
        .expect("Should have Location header")
        .to_str()
        .unwrap();
    assert!(
        location.contains("pastebin.htm"),
        "Should redirect to pastebin.htm, got {}",
        location
    );
}

#[tokio::test]
async fn bin_no_slash_redirects() {
    let resp = client().get(url("/bin")).send().await.unwrap();
    assert_eq!(
        resp.status().as_u16(),
        301,
        "/bin (no trailing slash) should redirect, got {}",
        resp.status()
    );
    let location = resp
        .headers()
        .get("location")
        .expect("Should have Location header")
        .to_str()
        .unwrap();
    assert!(
        location.contains("pastebin.htm"),
        "Should redirect to pastebin.htm, got {}",
        location
    );
}

// ============================================================================
// 3. Creating Pastes (POST /bin/add.php)
// ============================================================================

#[tokio::test]
async fn create_simple_paste() {
    let content = test_content("simple paste");
    let resp = create_paste_response(&content, false, None, false).await;

    assert_eq!(
        resp.status().as_u16(),
        302,
        "Should redirect after creating paste, got {}",
        resp.status()
    );

    let key = extract_key_from_response(&resp).expect("Should have key in Location");
    assert_eq!(key.len(), 22, "Standard URL key should be 22 chars");
    assert!(is_alphanumeric(&key), "Key should be alphanumeric");
}

#[tokio::test]
async fn create_empty_paste_fails() {
    // POST without paste field
    let resp = client()
        .post(url("/bin/add.php"))
        .form(&[("lifetime", "864000")])
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Empty post"),
        "Should show 'Empty post' error, got: {}",
        &body[..body.len().min(200)]
    );
}

#[tokio::test]
async fn create_short_url() {
    let content = test_content("short url paste");
    let key = create_paste_opts(&content, false, None, true)
        .await
        .expect("Should create paste");

    assert_eq!(key.len(), 8, "Short URL key should be 8 chars, got {}", key.len());
    assert!(is_alphanumeric(&key), "Key should be alphanumeric");
}

#[tokio::test]
async fn create_jscrypt_paste() {
    let content = test_content("jscrypt paste");
    let key = create_paste_opts(&content, true, None, false)
        .await
        .expect("Should create jscrypt paste");

    // Verify it was created
    let resp = get_paste(&key).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn valid_lifetimes_accepted() {
    // Test all valid lifetime values
    for lifetime in [600, 3600, 86400, 864000, 2592000, 15552000] {
        let content = test_content(&format!("lifetime-{}", lifetime));
        let resp = create_paste_response(&content, false, Some(lifetime), false).await;
        assert_eq!(
            resp.status().as_u16(),
            302,
            "lifetime={} should be accepted, got {}",
            lifetime,
            resp.status()
        );
    }
}

// ============================================================================
// 4. Viewing Pastes (GET /b/{key})
// ============================================================================

#[tokio::test]
async fn view_paste_exists() {
    let content = test_content("view test");
    let key = create_paste(&content).await;

    let resp = get_paste(&key).await;
    assert!(
        resp.status().is_success(),
        "Should return 200 for existing paste"
    );
}

#[tokio::test]
async fn view_paste_not_found() {
    let resp = get_paste("nonexistent123456789012").await;
    assert_eq!(
        resp.status().as_u16(),
        404,
        "Not-found paste should return 404, got: {}",
        resp.status()
    );
    let body = resp.text().await.unwrap();

    assert!(
        body.contains("Sorry, the paste you were looking for could not be found."),
        "Should show not found message, got: {}",
        &body.chars().take(500).collect::<String>()
    );
}

#[tokio::test]
async fn view_paste_shows_timeleft() {
    let content = test_content("timeleft test");
    let key = create_paste(&content).await;

    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();

    // Should show something like "will be deleted in X days"
    assert!(
        body.contains("deleted in") || body.contains("will be deleted"),
        "Should show time remaining"
    );
}

#[tokio::test]
async fn view_paste_content_escaped() {
    let content = test_content("<script>alert('xss')</script>");
    let key = create_paste(&content).await;

    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();

    // The raw <script> should NOT appear in HTML
    assert!(
        !body.contains("<script>alert"),
        "Script tag should be HTML-escaped"
    );
    // But escaped version should
    assert!(
        body.contains("&lt;script&gt;") || body.contains("&lt;script"),
        "Should contain escaped script tag"
    );
}

#[tokio::test]
async fn view_paste_tabs_converted() {
    let content = "X\tY";
    let key = create_paste(content).await;

    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();

    // In the display div (.codebox), tabs are converted to 4 nbsp
    // The textarea preserves the raw tab for re-editing
    assert!(
        body.contains("X&nbsp;&nbsp;&nbsp;&nbsp;Y"),
        "Tab should be converted to 4 nbsp in display area"
    );
}

#[tokio::test]
async fn view_paste_double_spaces() {
    let content = "A  B";
    let key = create_paste(content).await;

    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();

    // Double spaces become double nbsp in the display area
    assert!(
        body.contains("A&nbsp;&nbsp;B"),
        "Double spaces should be converted to double nbsp in display"
    );
}

#[tokio::test]
async fn view_paste_has_form() {
    let content = test_content("form test");
    let key = create_paste(&content).await;

    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();

    assert!(body.contains("name=\"paste\""), "Should have paste textarea");
    assert!(
        body.contains("name=\"lifetime\""),
        "Should have lifetime dropdown"
    );
}

#[tokio::test]
async fn view_cache_disabled() {
    let content = test_content("cache test");
    let key = create_paste(&content).await;

    let resp = get_paste(&key).await;

    let cache_control = resp
        .headers()
        .get("cache-control")
        .expect("Should have Cache-Control header")
        .to_str()
        .unwrap();

    assert!(
        cache_control.contains("no-cache"),
        "Cache-Control should contain no-cache, got: {}",
        cache_control
    );
}

// ============================================================================
// 5. Raw Paste Retrieval (GET /b/{key}?raw=true)
// ============================================================================

#[tokio::test]
async fn raw_paste_plain_text() {
    let content = test_content("raw test");
    let key = create_paste(&content).await;

    let resp = get_paste_raw(&key).await;

    let content_type = resp
        .headers()
        .get("content-type")
        .expect("Should have Content-Type")
        .to_str()
        .unwrap();

    assert!(
        content_type.starts_with("text/plain"),
        "Raw paste should be text/plain, got: {}",
        content_type
    );
}

#[tokio::test]
async fn raw_paste_jscrypt_error() {
    let content = test_content("jscrypt raw test");
    let key = create_paste_opts(&content, true, None, false)
        .await
        .expect("Should create jscrypt paste");

    let resp = get_paste_raw(&key).await;
    let body = resp.text().await.unwrap();

    // Should return an error message, not the ciphertext
    assert!(
        body.contains("client-side") || body.contains("encrypted") || body.contains("password"),
        "Should explain that raw access is not available for jscrypt pastes, got: {}",
        body
    );
}

#[tokio::test]
async fn raw_paste_preserves_content() {
    let content = test_content("preserve test: hello world!");
    let key = create_paste(&content).await;

    let resp = get_paste_raw(&key).await;
    let raw = resp.text().await.unwrap();

    assert_eq!(raw, content, "Raw content should match original");
}

#[tokio::test]
async fn raw_paste_not_found() {
    let resp = get_paste_raw("nonexistent123456789012").await;
    assert_eq!(
        resp.status().as_u16(),
        404,
        "Raw not-found should return 404"
    );
    let body = resp.text().await.unwrap();
    assert_eq!(
        body,
        "Sorry, the paste you were looking for could not be found."
    );
}

// ============================================================================
// 6. JS-Encrypted Pastes (jscrypt=yes)
// ============================================================================

#[tokio::test]
async fn jscrypt_paste_shows_password_prompt() {
    let content = test_content("jscrypt password test");
    let key = create_paste_opts(&content, true, None, false)
        .await
        .expect("Should create jscrypt paste");

    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();

    // Should have password input field
    assert!(
        body.contains("password") || body.contains("Password"),
        "Should show password prompt for jscrypt paste"
    );
}

#[tokio::test]
async fn jscrypt_paste_content_not_visible() {
    let unique_marker = format!("VISIBLE_TEST_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos());
    let content = format!("This should not be visible: {}", unique_marker);

    let key = create_paste_opts(&content, true, None, false)
        .await
        .expect("Should create jscrypt paste");

    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();

    // The unique marker should NOT appear in plaintext in the HTML
    // (it might be in a JS variable as ciphertext, but not readable)
    assert!(
        !body.contains(&unique_marker),
        "Jscrypt paste content should not be visible in HTML"
    );
}

// ============================================================================
// 7. URL Key Format
// ============================================================================

#[tokio::test]
async fn url_key_standard_length() {
    let content = test_content("key length test");
    let key = create_paste(&content).await;

    assert_eq!(key.len(), 22, "Standard key should be 22 chars");
}

#[tokio::test]
async fn url_key_short_length() {
    let content = test_content("short key test");
    let key = create_paste_opts(&content, false, None, true)
        .await
        .expect("Should create short URL paste");

    assert_eq!(key.len(), 8, "Short key should be 8 chars");
}

#[tokio::test]
async fn url_key_alphanumeric() {
    // Create several pastes and verify all keys are alphanumeric
    for i in 0..5 {
        let content = test_content(&format!("alphanum test {}", i));
        let key = create_paste(&content).await;
        assert!(
            is_alphanumeric(&key),
            "Key should be alphanumeric only, got: {}",
            key
        );
    }
}

#[tokio::test]
async fn url_key_unique() {
    let content1 = test_content("unique test 1");
    let content2 = test_content("unique test 2");

    let key1 = create_paste(&content1).await;
    let key2 = create_paste(&content2).await;

    assert_ne!(key1, key2, "Different pastes should have different keys");
}

// ============================================================================
// 8. Content Handling
// ============================================================================

#[tokio::test]
async fn unicode_preserved() {
    let content = test_content("über naïve 日本語 🎉");
    let key = create_paste(&content).await;

    let resp = get_paste_raw(&key).await;
    let raw = resp.text().await.unwrap();

    assert_eq!(raw, content, "Unicode should be preserved");
}

#[tokio::test]
async fn special_chars_preserved() {
    let content = test_content("special: &<>\"' test");
    let key = create_paste(&content).await;

    let resp = get_paste_raw(&key).await;
    let raw = resp.text().await.unwrap();

    assert_eq!(raw, content, "Special chars should be preserved in raw");
}

#[tokio::test]
async fn whitespace_preserved() {
    let content = "  leading and trailing spaces  ";
    let key = create_paste(content).await;

    let resp = get_paste_raw(&key).await;
    let raw = resp.text().await.unwrap();

    assert_eq!(raw, content, "Whitespace should be preserved");
}

#[tokio::test]
async fn empty_lines_preserved() {
    let content = "line1\n\n\nline4";
    let key = create_paste(content).await;

    let resp = get_paste_raw(&key).await;
    let raw = resp.text().await.unwrap();

    assert_eq!(raw, content, "Empty lines should be preserved");
}

#[tokio::test]
async fn large_paste() {
    // Create a 100KB paste
    let content = "x".repeat(100_000);
    let key = create_paste(&content).await;

    let resp = get_paste_raw(&key).await;
    let raw = resp.text().await.unwrap();

    assert_eq!(raw.len(), content.len(), "Large paste should be preserved");
}

#[tokio::test]
async fn line_endings_normalized_crlf() {
    let content = "line1\r\nline2\r\nline3";
    let key = create_paste(content).await;

    let resp = get_paste_raw(&key).await;
    let raw = resp.text().await.unwrap();

    let expected = normalize_line_endings(content);
    assert_eq!(raw, expected, "CRLF should be normalized to LF");
}

#[tokio::test]
async fn line_endings_normalized_cr() {
    let content = "line1\rline2\rline3";
    let key = create_paste(content).await;

    let resp = get_paste_raw(&key).await;
    let raw = resp.text().await.unwrap();

    let expected = normalize_line_endings(content);
    assert_eq!(raw, expected, "CR should be normalized to LF");
}

// ============================================================================
// 9. Lifetime/Expiration
// ============================================================================

#[tokio::test]
async fn timeleft_display_format() {
    let content = test_content("timeleft format test");
    let key = create_paste_opts(&content, false, Some(864000), false)
        .await
        .expect("Should create paste");

    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();

    // Should contain time units
    assert!(
        body.contains("day") || body.contains("hour") || body.contains("minute"),
        "Should show time remaining in human-readable format"
    );
}

#[tokio::test]
async fn paste_expires_and_deleted() {
    let content = test_content("expiry test");
    // Create with 5 second lifetime
    let key = create_paste_opts(&content, false, Some(5), false)
        .await
        .expect("Should create paste with short lifetime");

    // Verify paste exists
    let resp = get_paste(&key).await;
    assert!(
        resp.status().is_success(),
        "Paste should exist immediately after creation"
    );

    // Wait for expiry + buffer
    tokio::time::sleep(std::time::Duration::from_secs(8)).await;

    // Trigger cleanup by creating another paste (cleanup happens on add/view)
    let _ = create_paste(&test_content("trigger cleanup")).await;

    // Verify paste is gone
    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("could not be found"),
        "Paste should be expired and deleted"
    );
}

// ============================================================================
// 9a. Lifetime Validation
// ============================================================================

#[tokio::test]
async fn lifetime_custom_value_accepted() {
    // Any value between 1 and 15552000 should be accepted
    let content = test_content("custom lifetime test");
    let resp = create_paste_response(&content, false, Some(12345), false).await;

    assert_eq!(
        resp.status().as_u16(),
        302,
        "Custom lifetime within range should be accepted"
    );
}

#[tokio::test]
async fn lifetime_at_max_accepted() {
    // Exactly 6 months (15552000 seconds) should be accepted
    let content = test_content("max lifetime test");
    let resp = create_paste_response(&content, false, Some(15552000), false).await;

    assert_eq!(
        resp.status().as_u16(),
        302,
        "Lifetime at max (6 months) should be accepted"
    );
}

#[tokio::test]
async fn lifetime_exceeds_max_rejected() {
    // Max is 15552000 (6 months), so max + 1 should be rejected
    let content = test_content("exceeds max lifetime test");
    let resp = create_paste_response(&content, false, Some(15552001), false).await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "Lifetime exceeding max (15552001) should be rejected with 400"
    );

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Invalid lifetime") || body.contains("exceeds"),
        "Should show error message about invalid lifetime"
    );
}

#[tokio::test]
async fn lifetime_zero_rejected() {
    // Zero lifetime should be rejected
    let content = test_content("zero lifetime test");
    let resp = create_paste_response(&content, false, Some(0), false).await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "Zero lifetime should be rejected with 400"
    );

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Invalid lifetime") || body.contains("positive"),
        "Should show error message about invalid lifetime"
    );
}

#[tokio::test]
async fn lifetime_very_large_rejected() {
    // u32::MAX exceeds the 6-month maximum
    let content = test_content("very large lifetime test");
    let resp = create_paste_response(&content, false, Some(u32::MAX), false).await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "Very large lifetime (u32::MAX) should be rejected with 400"
    );

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Invalid lifetime") || body.contains("exceeds"),
        "Should show error message about invalid lifetime"
    );
}

#[tokio::test]
async fn lifetime_minimum_accepted() {
    // 1 second is the minimum valid lifetime
    let content = test_content("minimum lifetime test");
    let resp = create_paste_response(&content, false, Some(1), false).await;

    assert_eq!(
        resp.status().as_u16(),
        302,
        "Minimum lifetime (1 second) should be accepted"
    );
}

#[tokio::test]
async fn lifetime_non_numeric_rejected() {
    // Non-numeric lifetime should be rejected
    let content = test_content("non-numeric lifetime test");
    let resp = create_paste_with_str_lifetime(&content, "abc").await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "Non-numeric lifetime should be rejected with 400"
    );

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Invalid lifetime") || body.contains("not a valid number"),
        "Should show error message about invalid lifetime"
    );
}

#[tokio::test]
async fn lifetime_negative_rejected() {
    // Negative lifetime should be rejected
    let content = test_content("negative lifetime test");
    let resp = create_paste_with_str_lifetime(&content, "-100").await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "Negative lifetime should be rejected with 400"
    );

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Invalid lifetime") || body.contains("positive"),
        "Should show error message about invalid lifetime"
    );
}

// ============================================================================
// 10. Round-Trip Tests
// ============================================================================

#[tokio::test]
async fn roundtrip_simple() {
    let content = test_content("roundtrip simple");
    let key = create_paste(&content).await;

    let resp = get_paste_raw(&key).await;
    let raw = resp.text().await.unwrap();

    assert_eq!(raw, content, "Simple roundtrip should preserve content");
}

#[tokio::test]
async fn roundtrip_with_html() {
    let content = test_content("<div class=\"test\">Hello & goodbye</div>");
    let key = create_paste(&content).await;

    // Check raw preserves exactly
    let resp = get_paste_raw(&key).await;
    let raw = resp.text().await.unwrap();
    assert_eq!(raw, content, "Raw should preserve HTML exactly");

    // Check HTML view has it escaped
    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();
    assert!(
        !body.contains("<div class=\"test\">Hello"),
        "HTML view should escape the content"
    );
}

#[tokio::test]
async fn roundtrip_multiline() {
    let content = "line 1\nline 2\nline 3\n\nline 5";
    let key = create_paste(content).await;

    let resp = get_paste_raw(&key).await;
    let raw = resp.text().await.unwrap();

    assert_eq!(raw, content, "Multiline content should be preserved");
}

#[tokio::test]
async fn roundtrip_repost() {
    // Create initial paste
    let content = test_content("repost test");
    let key1 = create_paste(&content).await;

    // View it and extract content from textarea
    let resp = get_paste(&key1).await;
    let body = resp.text().await.unwrap();
    let extracted = extract_textarea_content(&body).expect("Should extract textarea content");

    // Create new paste with extracted content
    let key2 = create_paste(&extracted).await;

    // Verify new paste has same content
    let resp = get_paste_raw(&key2).await;
    let raw = resp.text().await.unwrap();

    assert_eq!(raw, content, "Reposted content should match original");
    assert_ne!(key1, key2, "New paste should have different key");
}

// ============================================================================
// 11. Error Handling
// ============================================================================

#[tokio::test]
async fn post_without_paste_field() {
    let resp = client()
        .post(url("/bin/add.php"))
        .form(&[("jscrypt", "no")])
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Empty post"),
        "Should show empty post error"
    );
}

#[tokio::test]
async fn invalid_url_key() {
    let resp = get_paste("!@#$%^").await;

    assert_eq!(
        resp.status().as_u16(),
        404,
        "Invalid key should return 404 with not-found page, got: {}",
        resp.status()
    );
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Sorry, the paste you were looking for could not be found."),
        "Invalid key should show not-found message"
    );
}

#[tokio::test]
async fn extremely_long_key() {
    let long_key = "a".repeat(1000);
    let resp = get_paste(&long_key).await;

    assert_eq!(
        resp.status().as_u16(),
        404,
        "Extremely long key should return 404 with not-found page, got: {}",
        resp.status()
    );
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Sorry, the paste you were looking for could not be found."),
        "Extremely long key should show not-found message"
    );
}

// ============================================================================
// 12. Form Field Passthrough
// ============================================================================

#[tokio::test]
async fn view_has_paste_in_textarea() {
    let content = test_content("textarea test");
    let key = create_paste(&content).await;

    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();

    let extracted = extract_textarea_content(&body).expect("Should have textarea");
    assert_eq!(extracted, content, "Textarea should contain paste content");
}

#[tokio::test]
async fn view_has_lifetime_dropdown() {
    let content = test_content("lifetime dropdown test");
    let key = create_paste(&content).await;

    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();

    assert!(
        body.contains("name=\"lifetime\""),
        "View page should have lifetime dropdown"
    );
}

// ============================================================================
// 13. Size Limits
// ============================================================================

/// Pastes exceeding 50 MB should be rejected with 400
#[tokio::test]
async fn paste_too_large_rejected() {
    // Server limit is 50 MB (50 * 1024 * 1024 = 52428800 bytes)
    // Send 1 byte over the limit
    let content = "x".repeat(50 * 1024 * 1024 + 1);
    let resp = create_paste_response(&content, false, None, false).await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "Paste exceeding 50 MB should be rejected with 400, got: {}",
        resp.status()
    );
}

/// 30 MB paste should be accepted
#[tokio::test]
async fn paste_30mb_accepted() {
    let content = "x".repeat(30 * 1024 * 1024);
    let resp = create_paste_response(&content, false, None, false).await;

    assert_eq!(
        resp.status().as_u16(),
        302,
        "30 MB paste should be accepted (302 redirect), got: {}",
        resp.status()
    );

    let key = extract_key_from_response(&resp)
        .expect("30 MB paste should redirect to /b/{key}");
    assert!(!key.is_empty(), "Key should not be empty");
}

#[tokio::test]
async fn view_has_shorturl_checkbox() {
    let content = test_content("shorturl checkbox test");
    let key = create_paste(&content).await;

    let resp = get_paste(&key).await;
    let body = resp.text().await.unwrap();

    assert!(
        body.contains("name=\"shorturl\""),
        "View page should have shorturl checkbox"
    );
}

#[tokio::test]
async fn view_can_repost() {
    let content = test_content("can repost test");
    let key1 = create_paste(&content).await;

    // View the paste
    let resp = get_paste(&key1).await;
    let body = resp.text().await.unwrap();

    // Verify form action points to add.php
    assert!(
        body.contains("/bin/add.php") || body.contains("action=\"add.php\""),
        "View page form should post to add.php"
    );
}

// ============================================================================
// 13. Security Headers
// ============================================================================

#[tokio::test]
async fn view_no_cache_headers() {
    let content = test_content("no cache headers test");
    let key = create_paste(&content).await;

    let resp = get_paste(&key).await;

    let cache_control = resp
        .headers()
        .get("cache-control")
        .expect("Should have Cache-Control")
        .to_str()
        .unwrap();

    assert!(
        cache_control.contains("no-cache") && cache_control.contains("must-revalidate"),
        "Should have no-cache, must-revalidate, got: {}",
        cache_control
    );
}

#[tokio::test]
async fn view_expires_header() {
    let content = test_content("expires header test");
    let key = create_paste(&content).await;

    let resp = get_paste(&key).await;

    let expires = resp
        .headers()
        .get("expires")
        .expect("Should have Expires header")
        .to_str()
        .unwrap();

    // Should be a date in the past
    assert!(
        expires.contains("1990") || expires.contains("1970"),
        "Expires should be in the past, got: {}",
        expires
    );
}

#[tokio::test]
async fn view_content_type_html() {
    let content = test_content("content type html test");
    let key = create_paste(&content).await;

    let resp = get_paste(&key).await;

    let content_type = resp
        .headers()
        .get("content-type")
        .expect("Should have Content-Type")
        .to_str()
        .unwrap();

    assert!(
        content_type.starts_with("text/html"),
        "View should be text/html, got: {}",
        content_type
    );
}

#[tokio::test]
async fn raw_content_type_plain() {
    let content = test_content("content type plain test");
    let key = create_paste(&content).await;

    let resp = get_paste_raw(&key).await;

    let content_type = resp
        .headers()
        .get("content-type")
        .expect("Should have Content-Type")
        .to_str()
        .unwrap();

    assert!(
        content_type.starts_with("text/plain"),
        "Raw should be text/plain, got: {}",
        content_type
    );
}
