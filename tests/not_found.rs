//! 404 Page Tests
//!
//! Tests for the 404 "not found" page behavior and content.

mod common;

use common::{assert_body_contains, assert_status, client, url};

// =============================================================================
// 404 Status
// =============================================================================

/// Nonexistent page returns 404 status
#[tokio::test]
async fn returns_404_status() {
    let resp = client().get(url("/nonexistent-page-12345.htm")).send().await.unwrap();
    assert_status(&resp, 404, "/nonexistent-page-12345.htm should return 404");
}

// =============================================================================
// 404 Page Content
// =============================================================================

/// 404 page contains error message text
#[tokio::test]
async fn has_expected_content() {
    let resp = client().get(url("/nonexistent-page-12345.htm")).send().await.unwrap();
    let body = resp.text().await.unwrap();

    // defuse.ca shows "This page does not exist." and "(ERROR 404)"
    // Check for "404" which should be present in any reasonable 404 page
    assert_body_contains(&body, "404", "404 page should mention '404'");
}

/// 404 page has site navigation (not a bare error page)
#[tokio::test]
async fn has_navigation() {
    let resp = client().get(url("/nonexistent-page-12345.htm")).send().await.unwrap();
    let body = resp.text().await.unwrap();

    // The site uses <div class="menu"> for navigation
    assert_body_contains(&body, "class=\"menu\"", "404 page should have site navigation menu");
}
