//! Content-Type header tests
//!
//! Verifies that responses have correct Content-Type headers.

mod common;

use common::{assert_content_type, client, url};

// =============================================================================
// HTML Pages
// =============================================================================

/// HTML pages should have text/html content type with UTF-8 charset
#[tokio::test]
async fn html_page_has_correct_content_type() {
    let resp = client().get(url("/about.htm")).send().await.unwrap();
    assert_content_type(&resp, "text/html", "/about.htm");

    // Also verify charset is UTF-8
    let content_type = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        content_type.contains("charset=utf-8") || content_type.contains("charset=UTF-8"),
        "HTML should specify UTF-8 charset, got: {}",
        content_type
    );
}

/// Home page should have text/html content type
#[tokio::test]
async fn home_page_has_correct_content_type() {
    let resp = client().get(url("/")).send().await.unwrap();
    assert_content_type(&resp, "text/html", "/");
}

/// 404 page should have text/html content type
#[tokio::test]
async fn not_found_page_has_html_content_type() {
    let resp = client().get(url("/nonexistent-page-12345.htm")).send().await.unwrap();
    assert_content_type(&resp, "text/html", "404 page");
}

// =============================================================================
// PDF Files
// =============================================================================

/// PDF downloads should have application/pdf content type
#[tokio::test]
#[ignore = "downloads/ not served by local dev server"]
async fn pdf_download_has_correct_content_type() {
    let resp = client().get(url("/downloads/thesis.pdf")).send().await.unwrap();
    assert_content_type(&resp, "application/pdf", "/downloads/thesis.pdf");
}

// =============================================================================
// Content-Disposition Tests (downloads/ vs files2/)
// =============================================================================

/// Files in /downloads/ should NOT have Content-Disposition: attachment
/// (allows browser to render PDFs inline)
#[tokio::test]
async fn downloads_dir_allows_inline_rendering() {
    let resp = client().get(url("/downloads/thesis.pdf")).send().await.unwrap();

    // Should NOT have Content-Disposition header (or if present, not "attachment")
    if let Some(disposition) = resp.headers().get("content-disposition") {
        let value = disposition.to_str().unwrap();
        assert!(
            !value.to_lowercase().contains("attachment"),
            "/downloads/ files should render inline, but got Content-Disposition: {}",
            value
        );
    }
    // No Content-Disposition header is fine (allows inline rendering)
}

/// Files in /files2/ should have Content-Disposition: attachment
/// (forces browser to download instead of render)
#[tokio::test]
#[ignore = "files2/ not in git repo"]
async fn files2_dir_forces_download() {
    let resp = client().get(url("/files2/poc/pocorgtfo00.pdf")).send().await.unwrap();

    let disposition = resp
        .headers()
        .get("content-disposition")
        .expect("/files2/ should have Content-Disposition header")
        .to_str()
        .unwrap();

    assert!(
        disposition.to_lowercase().contains("attachment"),
        "/files2/ files should force download, got Content-Disposition: {}",
        disposition
    );
}

// =============================================================================
// Other Static Files
// =============================================================================

/// Favicon should have image content type
#[tokio::test]
async fn favicon_has_image_content_type() {
    let resp = client().get(url("/favicon.ico")).send().await.unwrap();
    let content_type = resp
        .headers()
        .get("content-type")
        .expect("favicon should have content-type")
        .to_str()
        .unwrap();

    // Could be image/x-icon, image/vnd.microsoft.icon, or image/ico
    assert!(
        content_type.starts_with("image/"),
        "favicon should have image/* content type, got: {}",
        content_type
    );
}

/// robots.txt should have text/plain content type
#[tokio::test]
async fn robots_txt_has_text_content_type() {
    let resp = client().get(url("/robots.txt")).send().await.unwrap();
    assert_content_type(&resp, "text/plain", "/robots.txt");
}

/// PNG images should have image/png content type
#[tokio::test]
async fn png_image_has_correct_content_type() {
    let resp = client().get(url("/images/bg.png")).send().await.unwrap();
    assert_content_type(&resp, "image/png", "/images/bg.png");
}
