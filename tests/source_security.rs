//! Tests for /source/ and /libs/ directory security
//!
//! CRITICAL: The /source/ directory must serve PHP files as raw source code,
//! NOT execute them. This is a security vulnerability on defuse.ca where
//! pdfcleaner.php (which has system command execution) is being executed.
//!
//! The /libs/ directory must return 403 Forbidden for all requests - these
//! are internal PHP includes that should never be web-accessible.
//!
//! Some tests are expected to FAIL on defuse.ca - that's the point.
//! The Rust implementation must handle these correctly.

mod common;

use common::{client, url};

/// /source/pdfcleaner.php should NOT be executed - must return raw PHP source
///
/// SECURITY: pdfcleaner.php contains system commands (pdf2ps, ps2pdf, pdftotext)
/// that process user-uploaded files. If executed, this is a critical vulnerability.
#[tokio::test]
async fn source_php_not_executed() {
    let resp = client()
        .get(url("/source/pdfcleaner.php"))
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_success(),
        "source_php_not_executed: expected 200, got {}",
        resp.status()
    );

    let content_type = resp
        .headers()
        .get("content-type")
        .expect("source_php_not_executed: Content-Type header should be present")
        .to_str()
        .unwrap();

    // Content-Type MUST NOT be text/html (that indicates PHP execution)
    // Should be text/plain, text/x-php, or application/octet-stream
    assert!(
        !content_type.starts_with("text/html"),
        "source_php_not_executed: SECURITY VULNERABILITY! \
         /source/pdfcleaner.php is being EXECUTED (Content-Type: {}). \
         It should be served as raw source code (text/plain or similar), \
         not executed as PHP!",
        content_type
    );
}

/// /source/pdfcleaner.php should contain raw PHP source code (<?php)
#[tokio::test]
async fn source_php_shows_raw_code() {
    let resp = client()
        .get(url("/source/pdfcleaner.php"))
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_success(),
        "source_php_shows_raw_code: expected 200, got {}",
        resp.status()
    );

    let body = resp.text().await.unwrap();

    // If served correctly, body should contain raw PHP source
    assert!(
        body.contains("<?php") || body.contains("<?"),
        "source_php_shows_raw_code: SECURITY VULNERABILITY! \
         /source/pdfcleaner.php does not contain '<?php'. \
         Got: '{}'. \
         This suggests PHP is being executed instead of served as source.",
        if body.len() > 200 { &body[..200] } else { &body }
    );
}

/// /source/breach.php should NOT be executed - must return raw PHP source
#[tokio::test]
async fn source_breach_not_executed() {
    let resp = client()
        .get(url("/source/breach.php"))
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_success(),
        "source_breach_not_executed: expected 200, got {}",
        resp.status()
    );

    let body = resp.text().await.unwrap();

    // breach.php defines the breach_visual_html function - should be visible in source
    assert!(
        body.contains("<?php") && body.contains("function breach"),
        "source_breach_not_executed: /source/breach.php should contain raw PHP source \
         with '<?php' and 'function breach'. Got {} bytes: '{}'",
        body.len(),
        if body.len() > 200 { &body[..200] } else { &body }
    );
}

/// /source/ files should have Content-Disposition header for download
#[tokio::test]
async fn source_files_have_attachment_disposition() {
    let resp = client()
        .get(url("/source/pdfcleaner.php"))
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_success(),
        "source_files_have_attachment_disposition: expected 200, got {}",
        resp.status()
    );

    let disposition = resp
        .headers()
        .get("content-disposition")
        .expect("source_files_have_attachment_disposition: Content-Disposition header should be present")
        .to_str()
        .unwrap();

    // Should have attachment disposition to trigger download
    // Note: "attachment" alone is acceptable, doesn't need filename
    assert!(
        disposition.contains("attachment"),
        "source_files_have_attachment_disposition: expected Content-Disposition \
         to contain 'attachment', got '{}'",
        disposition
    );
}

// =============================================================================
// /libs/ Directory Security Tests
// =============================================================================
// The /libs/ directory contains internal PHP includes that should NEVER be
// web-accessible. All requests must return 403 Forbidden.

/// /libs/breach.php should return 403 Forbidden or 404 Not Found (access denied)
#[tokio::test]
async fn libs_breach_php_returns_403_or_404() {
    let resp = client()
        .get(url("/libs/breach.php"))
        .send()
        .await
        .unwrap();

    let status = resp.status().as_u16();
    assert!(
        status == 403 || status == 404,
        "libs_breach_php_returns_403_or_404: /libs/breach.php should return 403 or 404, got {}",
        resp.status()
    );
}

/// /libs/phpcount.php should return 403 Forbidden or 404 Not Found (access denied)
#[tokio::test]
async fn libs_phpcount_php_returns_403_or_404() {
    let resp = client()
        .get(url("/libs/phpcount.php"))
        .send()
        .await
        .unwrap();

    let status = resp.status().as_u16();
    assert!(
        status == 403 || status == 404,
        "libs_phpcount_php_returns_403_or_404: /libs/phpcount.php should return 403 or 404, got {}",
        resp.status()
    );
}

/// /libs/Bibliography.php should return 403 Forbidden or 404 Not Found (access denied)
#[tokio::test]
async fn libs_bibliography_php_returns_403_or_404() {
    let resp = client()
        .get(url("/libs/Bibliography.php"))
        .send()
        .await
        .unwrap();

    let status = resp.status().as_u16();
    assert!(
        status == 403 || status == 404,
        "libs_bibliography_php_returns_403_or_404: /libs/Bibliography.php should return 403 or 404, got {}",
        resp.status()
    );
}

// =============================================================================
// Directory Listing Tests
// =============================================================================

/// /source/ directory listing must be denied
#[tokio::test]
async fn source_directory_listing_denied() {
    let resp = client().get(url("/source/")).send().await.unwrap();
    let status = resp.status().as_u16();
    assert!(
        status == 403 || status == 404,
        "/source/ should return 403 or 404 (no directory listing), got {}",
        status
    );
}

// =============================================================================
// Path Traversal Tests
// =============================================================================
// Storage directories serve files from STORAGE_PATH/extras/*. Path traversal
// attacks must not escape the served directory. tower_http::ServeDir handles
// this, but a version upgrade or route refactoring could break it.
// The storage root contains credentials — escaping extras/ is critical.

/// Helper to assert a path traversal attempt returns 400 or 404
fn assert_traversal_blocked(status: u16, path: &str) {
    assert!(
        status == 400 || status == 404,
        "{}: path traversal should return 400 or 404, got {}",
        path,
        status
    );
}

#[tokio::test]
async fn path_traversal_files_dotdot() {
    let resp = client().get(url("/files/../../.env")).send().await.unwrap();
    assert_traversal_blocked(resp.status().as_u16(), "/files/../../.env");
}

#[tokio::test]
async fn path_traversal_files_encoded() {
    let resp = client().get(url("/files/..%2F..%2F.env")).send().await.unwrap();
    assert_traversal_blocked(resp.status().as_u16(), "/files/..%2F..%2F.env");
}

#[tokio::test]
async fn path_traversal_files2_dotdot() {
    let resp = client().get(url("/files2/../../.env")).send().await.unwrap();
    assert_traversal_blocked(resp.status().as_u16(), "/files2/../../.env");
}

#[tokio::test]
async fn path_traversal_mirrors_dotdot() {
    let resp = client().get(url("/mirrors/../../../etc/passwd")).send().await.unwrap();
    assert_traversal_blocked(resp.status().as_u16(), "/mirrors/../../../etc/passwd");
}

#[tokio::test]
async fn path_traversal_upload_dotdot() {
    let resp = client().get(url("/upload/../../../etc/passwd")).send().await.unwrap();
    assert_traversal_blocked(resp.status().as_u16(), "/upload/../../../etc/passwd");
}

#[tokio::test]
async fn path_traversal_source_dotdot() {
    let resp = client().get(url("/source/../../.env")).send().await.unwrap();
    assert_traversal_blocked(resp.status().as_u16(), "/source/../../.env");
}
