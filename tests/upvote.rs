//! Upvote system tests
//!
//! Tests for both the AJAX endpoint (/upvote.php) and form POST fallback.
//! All tests are serialized to prevent interference (vote state is tracked by IP).

mod common;

use common::url;
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::Client;
use serial_test::serial;

/// Test page - low traffic, unlikely to get real votes during testing
/// Note: The upvotes_id has no hyphens even though the URL does
const TEST_PAGE: &str = "blindbirthdayattack";
const TEST_PAGE_URL: &str = "blind-birthday-attack";

// =============================================================================
// Core Types
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
struct VoteState {
    uparrow: bool,
    downarrow: bool,
    total: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
}

impl Direction {
    fn as_str(&self) -> &'static str {
        match self {
            Direction::Up => "up",
            Direction::Down => "down",
        }
    }
}

// =============================================================================
// Vote Client Trait and Implementations
// =============================================================================

trait VoteClient {
    /// Vote on a page. `page_id` is the upvotes_id (no hyphens).
    /// `page_url_slug` is the URL slug (with hyphens) - only used by FormClient.
    async fn vote(&self, page_id: &str, page_url_slug: &str, dir: Direction) -> VoteState;
}

/// AJAX client - POSTs to /upvote.php, parses XML response
struct AjaxClient {
    client: Client,
}

impl AjaxClient {
    fn new() -> Self {
        Self {
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

impl VoteClient for AjaxClient {
    async fn vote(&self, page_id: &str, _page_url_slug: &str, dir: Direction) -> VoteState {
        let resp = self
            .client
            .post(url("/upvote.php"))
            .form(&[
                ("upvotes_id", page_id),
                ("upvotes_direction", dir.as_str()),
            ])
            .send()
            .await
            .expect("Failed to send AJAX vote request");

        assert!(
            resp.status().is_success(),
            "AJAX vote request failed with status {}",
            resp.status()
        );

        let body = resp.text().await.expect("Failed to read AJAX response");
        parse_xml_response(&body)
    }
}

/// Form client - POSTs to page URL, follows redirect, parses HTML
struct FormClient {
    client: Client,
}

impl FormClient {
    fn new() -> Self {
        Self {
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::limited(5))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

impl VoteClient for FormClient {
    async fn vote(&self, page_id: &str, page_url_slug: &str, dir: Direction) -> VoteState {
        let page_url = url(&format!("/{}.htm", page_url_slug));

        let resp = self
            .client
            .post(&page_url)
            .form(&[
                ("upvotes_id", page_id),
                ("upvotes_direction", dir.as_str()),
            ])
            .send()
            .await
            .expect("Failed to send form vote request");

        assert!(
            resp.status().is_success(),
            "Form vote request failed with status {} (after redirects)",
            resp.status()
        );

        let body = resp.text().await.expect("Failed to read form response");
        parse_html_vote_state(&body, page_id)
    }
}

// =============================================================================
// Response Parsing
// =============================================================================

/// Parse XML response from /upvote.php
fn parse_xml_response(xml: &str) -> VoteState {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut status = String::new();
    let mut uparrow = String::new();
    let mut downarrow = String::new();
    let mut total = String::new();
    let mut current_tag = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                match current_tag.as_str() {
                    "status" => status = text,
                    "uparrow" => uparrow = text,
                    "downarrow" => downarrow = text,
                    "total" => total = text,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error parsing XML: {}", e),
            _ => {}
        }
    }

    assert_eq!(status, "pass", "Expected status=pass, got: {}", status);

    VoteState {
        uparrow: uparrow == "Y",
        downarrow: downarrow == "Y",
        total: total.parse().expect("Failed to parse total as i32"),
    }
}

/// Parse HTML response from form POST to extract vote state
fn parse_html_vote_state(html: &str, page: &str) -> VoteState {
    // Check for upvote-selected.gif or upvote.gif
    let uparrow = html.contains("upvote-selected.gif");

    // Check for downvote-selected.gif or downvote.gif
    let downarrow = html.contains("downvote-selected.gif");

    // Extract total from the counter div
    // Look for: <div class="upvotecount... upvoteCounter{page}">NUMBER</div>
    let counter_class = format!("upvoteCounter{}", page);
    let total = extract_counter_value(html, &counter_class)
        .expect("Failed to find vote counter in HTML");

    VoteState {
        uparrow,
        downarrow,
        total,
    }
}

/// Extract the numeric value from the upvote counter div
fn extract_counter_value(html: &str, counter_class: &str) -> Option<i32> {
    // Find the counter div by class
    let class_pos = html.find(counter_class)?;

    // Find the closing > of the div tag
    let after_class = &html[class_pos..];
    let tag_end = after_class.find('>')?;

    // Find the content after the >
    let after_tag = &after_class[tag_end + 1..];

    // Find the closing </div>
    let content_end = after_tag.find('<')?;
    let content = after_tag[..content_end].trim();

    content.parse().ok()
}

// =============================================================================
// Test Helpers
// =============================================================================

/// Reset vote state to neutral, return baseline total
async fn reset_to_neutral(client: &impl VoteClient, page_id: &str, page_url_slug: &str) -> i32 {
    // Vote up - results in either upvoted (uparrow=Y) or neutral
    let state = client.vote(page_id, page_url_slug, Direction::Up).await;

    if state.uparrow {
        // We're upvoted, vote up again to clear to neutral
        let state = client.vote(page_id, page_url_slug, Direction::Up).await;
        assert!(
            !state.uparrow && !state.downarrow,
            "Should be neutral after clearing, got uparrow={}, downarrow={}",
            state.uparrow,
            state.downarrow
        );
        state.total
    } else {
        // Already neutral (we cleared from upvoted state)
        assert!(
            !state.downarrow,
            "Expected neutral state, got downarrow={}",
            state.downarrow
        );
        state.total
    }
}

/// Run a vote state transition test
async fn run_vote_test(
    client: &impl VoteClient,
    page_id: &str,
    page_url_slug: &str,
    starting_state: Option<Direction>,
    action: Direction,
    expected_delta: i32,
    expected_up: bool,
    expected_down: bool,
) {
    // 1. Reset to neutral, get baseline total
    let baseline = reset_to_neutral(client, page_id, page_url_slug).await;

    // 2. Set up starting state if needed (and track setup delta)
    let setup_delta = if let Some(dir) = starting_state {
        let _state = client.vote(page_id, page_url_slug, dir).await;
        match dir {
            Direction::Up => 1,
            Direction::Down => -1,
        }
    } else {
        0
    };

    // 3. Perform the action being tested
    let result = client.vote(page_id, page_url_slug, action).await;

    // 4. Verify delta and button states
    let expected_total = baseline + setup_delta + expected_delta;
    assert_eq!(
        result.total, expected_total,
        "Expected total={}, got {} (baseline={}, setup_delta={}, expected_delta={})",
        expected_total, result.total, baseline, setup_delta, expected_delta
    );
    assert_eq!(
        result.uparrow, expected_up,
        "Expected uparrow={}, got {}",
        expected_up, result.uparrow
    );
    assert_eq!(
        result.downarrow, expected_down,
        "Expected downarrow={}, got {}",
        expected_down, result.downarrow
    );

    // 5. Clean up: reset back to neutral for next test
    reset_to_neutral(client, page_id, page_url_slug).await;
}

// =============================================================================
// AJAX Endpoint Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn ajax_upvote_from_neutral() {
    run_vote_test(&AjaxClient::new(), TEST_PAGE, TEST_PAGE_URL, None, Direction::Up, 1, true, false).await;
}

#[tokio::test]
#[serial]
async fn ajax_downvote_from_neutral() {
    run_vote_test(&AjaxClient::new(), TEST_PAGE, TEST_PAGE_URL, None, Direction::Down, -1, false, true).await;
}

#[tokio::test]
#[serial]
async fn ajax_clear_upvote() {
    run_vote_test(
        &AjaxClient::new(),
        TEST_PAGE,
        TEST_PAGE_URL,
        Some(Direction::Up),
        Direction::Up,
        -1,
        false,
        false,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn ajax_clear_downvote() {
    run_vote_test(
        &AjaxClient::new(),
        TEST_PAGE,
        TEST_PAGE_URL,
        Some(Direction::Down),
        Direction::Down,
        1,
        false,
        false,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn ajax_switch_upvote_to_downvote() {
    run_vote_test(
        &AjaxClient::new(),
        TEST_PAGE,
        TEST_PAGE_URL,
        Some(Direction::Up),
        Direction::Down,
        -2,
        false,
        true,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn ajax_switch_downvote_to_upvote() {
    run_vote_test(
        &AjaxClient::new(),
        TEST_PAGE,
        TEST_PAGE_URL,
        Some(Direction::Down),
        Direction::Up,
        2,
        true,
        false,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn ajax_returns_valid_xml() {
    let client = AjaxClient::new();

    // Just verify we can make a vote and get valid XML back
    let _baseline = reset_to_neutral(&client, TEST_PAGE, TEST_PAGE_URL).await;
    let state = client.vote(TEST_PAGE, TEST_PAGE_URL, Direction::Up).await;

    // If we got here without panicking, XML was valid
    // Just verify the state is consistent
    assert!(
        state.uparrow && !state.downarrow,
        "After upvote, expected uparrow=true, downarrow=false"
    );

    // Clean up
    reset_to_neutral(&client, TEST_PAGE, TEST_PAGE_URL).await;
}

#[tokio::test]
#[serial]
async fn ajax_votes_isolated_by_page() {
    let client = AjaxClient::new();
    // Page IDs (no hyphens) and URL slugs (with hyphens)
    let page_a_id = "blindbirthdayattack";
    let page_a_url = "blind-birthday-attack";
    let page_b_id = "cbcmodeiv";
    let page_b_url = "cbcmodeiv";

    // Reset both pages
    let _baseline_a = reset_to_neutral(&client, page_a_id, page_a_url).await;
    let baseline_b = reset_to_neutral(&client, page_b_id, page_b_url).await;

    // Vote on page A
    let _state_a = client.vote(page_a_id, page_a_url, Direction::Up).await;

    // Check page B is unaffected - vote and immediately clear to check total
    let state_b = client.vote(page_b_id, page_b_url, Direction::Up).await;
    // state_b.total should be baseline_b + 1 (our upvote), not affected by page A
    assert_eq!(
        state_b.total,
        baseline_b + 1,
        "Page B total should not be affected by voting on page A"
    );

    // Clean up both pages
    reset_to_neutral(&client, page_a_id, page_a_url).await;
    reset_to_neutral(&client, page_b_id, page_b_url).await;
}

// =============================================================================
// Form POST Endpoint Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn form_upvote_from_neutral() {
    run_vote_test(&FormClient::new(), TEST_PAGE, TEST_PAGE_URL, None, Direction::Up, 1, true, false).await;
}

#[tokio::test]
#[serial]
async fn form_downvote_from_neutral() {
    run_vote_test(&FormClient::new(), TEST_PAGE, TEST_PAGE_URL, None, Direction::Down, -1, false, true).await;
}

#[tokio::test]
#[serial]
async fn form_clear_upvote() {
    run_vote_test(
        &FormClient::new(),
        TEST_PAGE,
        TEST_PAGE_URL,
        Some(Direction::Up),
        Direction::Up,
        -1,
        false,
        false,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn form_clear_downvote() {
    run_vote_test(
        &FormClient::new(),
        TEST_PAGE,
        TEST_PAGE_URL,
        Some(Direction::Down),
        Direction::Down,
        1,
        false,
        false,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn form_switch_upvote_to_downvote() {
    run_vote_test(
        &FormClient::new(),
        TEST_PAGE,
        TEST_PAGE_URL,
        Some(Direction::Up),
        Direction::Down,
        -2,
        false,
        true,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn form_switch_downvote_to_upvote() {
    run_vote_test(
        &FormClient::new(),
        TEST_PAGE,
        TEST_PAGE_URL,
        Some(Direction::Down),
        Direction::Up,
        2,
        true,
        false,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn form_votes_isolated_by_page() {
    let client = FormClient::new();
    // Page IDs (no hyphens) and URL slugs (with hyphens)
    let page_a_id = "blindbirthdayattack";
    let page_a_url = "blind-birthday-attack";
    let page_b_id = "cbcmodeiv";
    let page_b_url = "cbcmodeiv";

    // Reset both pages
    let _baseline_a = reset_to_neutral(&client, page_a_id, page_a_url).await;
    let baseline_b = reset_to_neutral(&client, page_b_id, page_b_url).await;

    // Vote on page A
    let _state_a = client.vote(page_a_id, page_a_url, Direction::Up).await;

    // Check page B is unaffected
    let state_b = client.vote(page_b_id, page_b_url, Direction::Up).await;
    assert_eq!(
        state_b.total,
        baseline_b + 1,
        "Page B total should not be affected by voting on page A"
    );

    // Clean up both pages
    reset_to_neutral(&client, page_a_id, page_a_url).await;
    reset_to_neutral(&client, page_b_id, page_b_url).await;
}
