//! Upvote system tests
//!
//! Tests for both the AJAX endpoint (/upvote.php) and form POST fallback.
//! All tests are serialized to prevent interference (vote state is tracked by IP).

mod common;

use common::url;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesRef, Event};
use quick_xml::Reader;
use reqwest::Client;
use scraper::{Html, Selector};
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
            .header("origin", common::origin())
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
            .header("origin", common::origin())
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

/// The child elements of `<response>`, in the order defuse-rust's `xml_response` writes them.
const RESPONSE_FIELDS: [&str; 4] = ["status", "uparrow", "downarrow", "total"];

/// Where `parse_xml_response` is in the document.
enum ParseState {
    BeforeResponse,
    InResponse,
    /// Inside the `RESPONSE_FIELDS[fields.len()]` element, collecting its text.
    InField(Vec<TextPiece>),
    AfterResponse,
}

/// One piece of an element's text. quick-xml reports `&...;` references as separate
/// GeneralRef events, so an element's text arrives as raw text and resolved references.
enum TextPiece {
    Raw(String),
    Reference(String),
}

/// Parse XML response from /upvote.php
///
/// Accepts only the exact shape defuse-rust's `xml_response` produces: an optional XML
/// declaration, then `<response>` containing `<status>`, `<uparrow>`, `<downarrow>` and
/// `<total>` exactly once each, in that order, holding only text, with only whitespace
/// between elements. Anything else panics. For every response it accepts, it gives the same
/// result the quick-xml 0.37 parser it replaced did. That parser silently tolerated
/// comments, duplicate or missing elements and stray text, so this one is never more
/// lenient than it.
fn parse_xml_response(xml: &str) -> VoteState {
    let mut reader = Reader::from_str(xml);
    let mut fields: Vec<String> = Vec::new();
    let mut state = ParseState::BeforeResponse;

    loop {
        let event = reader
            .read_event()
            .unwrap_or_else(|e| panic!("Error parsing XML: {}", e));
        state = match (state, event) {
            (ParseState::BeforeResponse, Event::Decl(_)) => ParseState::BeforeResponse,
            (ParseState::BeforeResponse, Event::Start(e)) if e.name().as_ref() == "response" => {
                ParseState::InResponse
            }
            (ParseState::InResponse, Event::Start(e))
                if fields.len() < RESPONSE_FIELDS.len()
                    && e.name().as_ref() == RESPONSE_FIELDS[fields.len()] =>
            {
                ParseState::InField(Vec::new())
            }
            (ParseState::InField(mut pieces), Event::Text(e)) => {
                pieces.push(TextPiece::Raw(e.to_string()));
                ParseState::InField(pieces)
            }
            (ParseState::InField(mut pieces), Event::GeneralRef(e)) => {
                pieces.push(TextPiece::Reference(resolve_reference(&e)));
                ParseState::InField(pieces)
            }
            (ParseState::InField(pieces), Event::End(e))
                if e.name().as_ref() == RESPONSE_FIELDS[fields.len()] =>
            {
                fields.push(element_text(pieces));
                ParseState::InResponse
            }
            (ParseState::InResponse, Event::End(e))
                if fields.len() == RESPONSE_FIELDS.len() && e.name().as_ref() == "response" =>
            {
                ParseState::AfterResponse
            }
            (ParseState::AfterResponse, Event::Eof) => break,
            (
                state @ (ParseState::BeforeResponse
                | ParseState::InResponse
                | ParseState::AfterResponse),
                Event::Text(e),
            ) if e.chars().all(is_xml_whitespace) => state,
            (_, other) => panic!("Unexpected XML in upvote response: {:?}", other),
        };
    }

    let [status, uparrow, downarrow, total]: [String; 4] = fields
        .try_into()
        .expect("AfterResponse is only reached once all four fields are read");
    assert_eq!(status, "pass", "Expected status=pass, got: {}", status);

    VoteState {
        uparrow: uparrow == "Y",
        downarrow: downarrow == "Y",
        total: total.parse().expect("Failed to parse total as i32"),
    }
}

/// XML whitespace, as quick-xml 0.37's `trim_text` defined it (not Unicode whitespace).
fn is_xml_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\r' | '\n')
}

/// Join an element's pieces, trimmed the way quick-xml 0.37's `trim_text(true)` trimmed
/// the raw text before unescaping it: XML whitespace is removed from the very start and
/// end of the raw text only, never from what a reference such as `&#32;` resolves to.
fn element_text(pieces: Vec<TextPiece>) -> String {
    let piece_count = pieces.len();
    pieces
        .into_iter()
        .enumerate()
        .map(|(index, piece)| match piece {
            TextPiece::Reference(resolved) => resolved,
            TextPiece::Raw(raw) => {
                let mut text = raw.as_str();
                if index == 0 {
                    text = text.trim_start_matches(is_xml_whitespace);
                }
                if index + 1 == piece_count {
                    text = text.trim_end_matches(is_xml_whitespace);
                }
                text.to_string()
            }
        })
        .collect()
}

/// Resolve a `&...;` reference: a character reference or one of the five predefined XML
/// entities, which is what quick-xml 0.37's `BytesText::unescape` used to resolve.
fn resolve_reference(reference: &BytesRef) -> String {
    match reference
        .resolve_char_ref()
        .expect("Invalid character reference in XML")
    {
        Some(character) => character.to_string(),
        None => resolve_predefined_entity(reference)
            .unwrap_or_else(|| panic!("Unknown entity reference in XML: &{};", &**reference))
            .to_string(),
    }
}

// Offline tests for the XML parser above (no server needed).

/// Same shape as defuse-rust's `xml_response` in src/upvote.rs.
fn upvote_xml(status: &str, uparrow: &str, downarrow: &str, total: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<response>\n\
         <status>{status}</status>\n<uparrow>{uparrow}</uparrow>\n\
         <downarrow>{downarrow}</downarrow>\n<total>{total}</total>\n</response>\n"
    )
}

#[test]
fn parse_xml_response_reads_plain_values() {
    assert_eq!(
        parse_xml_response(&upvote_xml("pass", "Y", "N", "-3")),
        VoteState { uparrow: true, downarrow: false, total: -3 }
    );
    assert_eq!(
        parse_xml_response(&upvote_xml("pass", "N", "Y", "12")),
        VoteState { uparrow: false, downarrow: true, total: 12 }
    );
}

#[test]
fn parse_xml_response_joins_text_split_by_references() {
    // Character and predefined-entity references arrive as separate events and
    // must be stitched back into the element's text.
    assert_eq!(
        parse_xml_response(&upvote_xml("p&#97;s&#x73;", "&#89;", "N", "&#45;4&#50;")),
        VoteState { uparrow: true, downarrow: false, total: -42 }
    );
}

#[test]
fn parse_xml_response_trims_whole_element_text() {
    assert_eq!(
        parse_xml_response(&upvote_xml(" pass\n", "\n  Y  ", "N", "\t&#45;7 ")),
        VoteState { uparrow: true, downarrow: false, total: -7 }
    );
}

#[test]
#[should_panic(expected = "Expected status=pass, got: it&'s \"bad\" <x>")]
fn parse_xml_response_resolves_predefined_entities_in_status() {
    parse_xml_response(&upvote_xml("it&amp;&apos;s &quot;bad&quot; &lt;x&gt;", "N", "N", "0"));
}

#[test]
#[should_panic(expected = "Unknown entity reference in XML: &nbsp;")]
fn parse_xml_response_rejects_unknown_entities() {
    parse_xml_response(&upvote_xml("pass&nbsp;", "N", "N", "0"));
}

#[test]
#[should_panic(expected = "Invalid character reference in XML")]
fn parse_xml_response_rejects_invalid_character_references() {
    parse_xml_response(&upvote_xml("pass", "&#0;", "N", "0"));
}

#[test]
#[should_panic(expected = "Error parsing XML")]
fn parse_xml_response_rejects_malformed_xml() {
    parse_xml_response("<response><status>pass</uparrow></response>");
}

#[test]
#[should_panic(expected = "Expected status=pass, got:  pass")]
fn parse_xml_response_does_not_trim_whitespace_from_references() {
    // `&#32;` is a space produced by a reference, not raw whitespace.
    parse_xml_response(&upvote_xml("&#32;pass", "N", "N", "0"));
}

#[test]
#[should_panic(expected = "Expected status=pass, got: \u{a0}pass")]
fn parse_xml_response_does_not_trim_non_xml_whitespace() {
    // U+00A0 is Unicode whitespace but not XML whitespace.
    parse_xml_response(&upvote_xml("\u{a0}pass", "N", "N", "0"));
}

#[test]
#[should_panic(expected = "Unexpected XML in upvote response: Comment(")]
fn parse_xml_response_rejects_comments() {
    parse_xml_response(&upvote_xml("pa<!--c-->ss", "N", "N", "0"));
}

#[test]
#[should_panic(expected = "Unexpected XML in upvote response: CData(")]
fn parse_xml_response_rejects_cdata() {
    parse_xml_response(&upvote_xml("<![CDATA[pass]]>", "N", "N", "0"));
}

#[test]
#[should_panic(expected = "Unexpected XML in upvote response: Empty(")]
fn parse_xml_response_rejects_nested_elements() {
    parse_xml_response(&upvote_xml("pa<b/>ss", "N", "N", "0"));
}

#[test]
#[should_panic(expected = "Unexpected XML in upvote response: Start(")]
fn parse_xml_response_rejects_duplicate_elements() {
    let xml = upvote_xml("pass", "N", "N", "1").replace("</total>", "</total><total>2</total>");
    parse_xml_response(&xml);
}

#[test]
#[should_panic(expected = "Unexpected XML in upvote response: Start(")]
fn parse_xml_response_rejects_out_of_order_elements() {
    parse_xml_response("<response><uparrow>Y</uparrow><status>pass</status></response>");
}

#[test]
#[should_panic(expected = "Unexpected XML in upvote response: End(")]
fn parse_xml_response_rejects_missing_elements() {
    parse_xml_response(
        "<response><status>pass</status><uparrow>Y</uparrow><downarrow>N</downarrow></response>",
    );
}

#[test]
#[should_panic(expected = "Unexpected XML in upvote response: Text(")]
fn parse_xml_response_rejects_stray_text_between_elements() {
    let xml = upvote_xml("pass", "N", "N", "0").replace("</status>", "</status>junk");
    parse_xml_response(&xml);
}

#[test]
#[should_panic(expected = "Unexpected XML in upvote response: Text(")]
fn parse_xml_response_rejects_text_after_response() {
    parse_xml_response(&(upvote_xml("pass", "N", "N", "0") + "extra"));
}

#[test]
#[should_panic(expected = "Unexpected XML in upvote response: Start(")]
fn parse_xml_response_rejects_wrong_root_element() {
    parse_xml_response("<other><status>pass</status></other>");
}

#[test]
#[should_panic(expected = "Unexpected XML in upvote response: Eof")]
fn parse_xml_response_rejects_empty_input() {
    parse_xml_response("");
}

/// Parse HTML response from form POST to extract vote state
fn parse_html_vote_state(html: &str, page: &str) -> VoteState {
    // Look for the specific page's upvote image element and check if it's selected.
    // HTML structure: <input type="image" src="/images/upvote-selected.gif" ... name="upvoteUpImage{page}" />
    // We need to find the image with the page-specific name and check its src.
    let uparrow = is_vote_arrow_selected(html, page, "upvoteUpImage");
    let downarrow = is_vote_arrow_selected(html, page, "upvoteDownImage");

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

/// Check if a specific page's vote arrow is in selected state.
/// Looks for: name="upvoteUpImage{page}" or name="upvoteDownImage{page}"
/// and checks if the nearby src contains "-selected.gif"
fn is_vote_arrow_selected(html: &str, page: &str, arrow_prefix: &str) -> bool {
    // Look for name="upvoteUpImage{page}" or name="upvoteDownImage{page}"
    let name_attr = format!("name=\"{}{}\"", arrow_prefix, page);

    if let Some(name_pos) = html.find(&name_attr) {
        // Find the start of this <input tag by searching backwards
        let before_name = &html[..name_pos];
        if let Some(tag_start) = before_name.rfind("<input") {
            // Extract the full input tag (from <input to the next >)
            let tag_content = &html[tag_start..];
            if let Some(tag_end) = tag_content.find('>') {
                let input_tag = &tag_content[..tag_end + 1];
                // Check if this input tag's src contains "-selected.gif"
                return input_tag.contains("-selected.gif");
            }
        }
    }
    false
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
// Form Extraction (for homepage/all-pages list testing)
// =============================================================================

/// Extracted vote form data from a page listing
#[derive(Debug, Clone)]
struct ExtractedVoteForm {
    /// The form action URL (e.g., "/" or "/all-pages.htm")
    action_url: String,
    /// The upvotes_id value from the hidden input
    upvotes_id: String,
}

/// Extract all upvote forms from a page's HTML.
/// Returns forms in the order they appear on the page.
fn extract_vote_forms(html: &str) -> Vec<ExtractedVoteForm> {
    let document = Html::parse_document(html);
    let form_selector = Selector::parse("form.upvoteform").unwrap();
    let input_selector = Selector::parse("input[name='upvotes_id']").unwrap();

    let mut forms = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for form in document.select(&form_selector) {
        let action = form.value().attr("action").unwrap_or("");

        // Find the upvotes_id input within this form
        if let Some(input) = form.select(&input_selector).next() {
            if let Some(id) = input.value().attr("value") {
                // Only add each page_id once (there are up and down forms for each)
                if seen_ids.insert(id.to_string()) {
                    forms.push(ExtractedVoteForm {
                        action_url: action.to_string(),
                        upvotes_id: id.to_string(),
                    });
                }
            }
        }
    }

    forms
}

/// Vote via form POST to a specific URL.
/// This is the core voting function used for testing list pages.
async fn vote_via_form_post(
    client: &Client,
    form_action_url: &str,
    page_id: &str,
    direction: Direction,
) -> VoteState {
    let resp = client
        .post(url(form_action_url))
        .header("origin", common::origin())
        .form(&[
            ("upvotes_id", page_id),
            ("upvotes_direction", direction.as_str()),
        ])
        .send()
        .await
        .expect("Failed to send form vote request");

    assert!(
        resp.status().is_success(),
        "Form vote request to {} failed with status {} (after redirects)",
        form_action_url,
        resp.status()
    );

    let body = resp.text().await.expect("Failed to read form response");
    parse_html_vote_state(&body, page_id)
}

/// Reset a page to neutral vote state via form POST, return baseline total.
async fn reset_via_form(client: &Client, form_action_url: &str, page_id: &str) -> i32 {
    // Vote up - results in either upvoted (uparrow=Y) or neutral
    let state = vote_via_form_post(client, form_action_url, page_id, Direction::Up).await;

    if state.uparrow {
        // We're upvoted, vote up again to clear to neutral
        let state = vote_via_form_post(client, form_action_url, page_id, Direction::Up).await;
        assert!(
            !state.uparrow && !state.downarrow,
            "Should be neutral after clearing, got uparrow={}, downarrow={}",
            state.uparrow,
            state.downarrow
        );
        state.total
    } else if state.downarrow {
        // We were downvoted before, now neutral after the up vote toggled us
        // Actually no - if we were downvoted and voted up, we're now upvoted
        // Let me re-check the logic...
        // If downvoted and click up: switches to upvote (uparrow=true)
        // So if uparrow is false and downarrow is true, that shouldn't happen after voting up
        panic!("Unexpected state after voting up: uparrow=false, downarrow=true");
    } else {
        // Already neutral
        state.total
    }
}

/// Run a vote test via form POST to a list page.
async fn run_list_vote_test(
    client: &Client,
    form_action_url: &str,
    page_id: &str,
    starting_state: Option<Direction>,
    action: Direction,
    expected_delta: i32,
    expected_up: bool,
    expected_down: bool,
) {
    // 1. Reset to neutral, get baseline total
    let baseline = reset_via_form(client, form_action_url, page_id).await;

    // 2. Set up starting state if needed
    let setup_delta = if let Some(dir) = starting_state {
        let _state = vote_via_form_post(client, form_action_url, page_id, dir).await;
        match dir {
            Direction::Up => 1,
            Direction::Down => -1,
        }
    } else {
        0
    };

    // 3. Perform the action being tested
    let result = vote_via_form_post(client, form_action_url, page_id, action).await;

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

    // 5. Clean up: reset back to neutral
    reset_via_form(client, form_action_url, page_id).await;
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

// =============================================================================
// Homepage Upvote Form Tests
// =============================================================================

/// Helper to create a client with redirect following for form tests
fn form_test_client() -> Client {
    Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

/// Fetch a page and extract its vote forms
async fn fetch_and_extract_forms(client: &Client, page_path: &str) -> Vec<ExtractedVoteForm> {
    let resp = client
        .get(url(page_path))
        .send()
        .await
        .expect("Failed to fetch page");

    assert!(
        resp.status().is_success(),
        "Failed to fetch {}: status {}",
        page_path,
        resp.status()
    );

    let html = resp.text().await.expect("Failed to read page body");
    extract_vote_forms(&html)
}

#[tokio::test]
#[serial]
async fn homepage_has_vote_forms() {
    let client = form_test_client();
    let forms = fetch_and_extract_forms(&client, "/").await;

    assert!(
        forms.len() >= 2,
        "Homepage should have at least 2 vote forms, found {}",
        forms.len()
    );

    // Verify the forms have the expected structure
    for form in &forms {
        assert!(
            !form.action_url.is_empty(),
            "Form action URL should not be empty"
        );
        assert!(
            !form.upvotes_id.is_empty(),
            "Form upvotes_id should not be empty"
        );
    }
}

#[tokio::test]
#[serial]
async fn homepage_first_item_upvote() {
    let client = form_test_client();
    let forms = fetch_and_extract_forms(&client, "/").await;

    assert!(!forms.is_empty(), "Homepage should have vote forms");
    let form = &forms[0];

    run_list_vote_test(
        &client,
        &form.action_url,
        &form.upvotes_id,
        None,
        Direction::Up,
        1,
        true,
        false,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn homepage_first_item_downvote() {
    let client = form_test_client();
    let forms = fetch_and_extract_forms(&client, "/").await;

    assert!(!forms.is_empty(), "Homepage should have vote forms");
    let form = &forms[0];

    run_list_vote_test(
        &client,
        &form.action_url,
        &form.upvotes_id,
        None,
        Direction::Down,
        -1,
        false,
        true,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn homepage_second_item_upvote() {
    let client = form_test_client();
    let forms = fetch_and_extract_forms(&client, "/").await;

    assert!(
        forms.len() >= 2,
        "Homepage should have at least 2 vote forms"
    );
    let form = &forms[1];

    run_list_vote_test(
        &client,
        &form.action_url,
        &form.upvotes_id,
        None,
        Direction::Up,
        1,
        true,
        false,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn homepage_second_item_downvote() {
    let client = form_test_client();
    let forms = fetch_and_extract_forms(&client, "/").await;

    assert!(
        forms.len() >= 2,
        "Homepage should have at least 2 vote forms"
    );
    let form = &forms[1];

    run_list_vote_test(
        &client,
        &form.action_url,
        &form.upvotes_id,
        None,
        Direction::Down,
        -1,
        false,
        true,
    )
    .await;
}

// =============================================================================
// CSRF Protection Tests
// =============================================================================

/// POST to /upvote.php without Origin/Referer header should be rejected
#[tokio::test]
#[serial]
async fn ajax_csrf_rejected_no_origin() {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();

    let resp = client
        .post(url("/upvote.php"))
        // No Origin or Referer header
        .form(&[
            ("upvotes_id", TEST_PAGE),
            ("upvotes_direction", "up"),
        ])
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("<status>fail</status>"),
        "POST without Origin should return status=fail, got: {}",
        body
    );
}

/// POST to /upvote.php with evil Origin header should be rejected
#[tokio::test]
#[serial]
async fn ajax_csrf_rejected_evil_origin() {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();

    let resp = client
        .post(url("/upvote.php"))
        .header("origin", "https://evil.com")
        .form(&[
            ("upvotes_id", TEST_PAGE),
            ("upvotes_direction", "up"),
        ])
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("<status>fail</status>"),
        "POST with evil Origin should return status=fail, got: {}",
        body
    );
}

/// Form POST to a page without Origin/Referer should return 403
#[tokio::test]
#[serial]
async fn form_csrf_rejected_no_origin() {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();

    let resp = client
        .post(url(&format!("/{}.htm", TEST_PAGE_URL)))
        // No Origin or Referer header
        .form(&[
            ("upvotes_id", TEST_PAGE),
            ("upvotes_direction", "up"),
        ])
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status().as_u16(),
        403,
        "Form POST without Origin should return 403, got: {}",
        resp.status()
    );
}

/// Form POST to a page with evil Origin should return 403
#[tokio::test]
#[serial]
async fn form_csrf_rejected_evil_origin() {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();

    let resp = client
        .post(url(&format!("/{}.htm", TEST_PAGE_URL)))
        .header("origin", "https://evil.com")
        .form(&[
            ("upvotes_id", TEST_PAGE),
            ("upvotes_direction", "up"),
        ])
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status().as_u16(),
        403,
        "Form POST with evil Origin should return 403, got: {}",
        resp.status()
    );
}

// =============================================================================
// All-Pages Upvote Form Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn all_pages_has_vote_forms() {
    let client = form_test_client();
    let forms = fetch_and_extract_forms(&client, "/all-pages.htm").await;

    assert!(
        forms.len() >= 2,
        "All-pages should have at least 2 vote forms, found {}",
        forms.len()
    );

    // Verify the forms have the expected structure
    for form in &forms {
        assert!(
            !form.action_url.is_empty(),
            "Form action URL should not be empty"
        );
        assert!(
            !form.upvotes_id.is_empty(),
            "Form upvotes_id should not be empty"
        );
    }
}

#[tokio::test]
#[serial]
async fn all_pages_first_item_upvote() {
    let client = form_test_client();
    let forms = fetch_and_extract_forms(&client, "/all-pages.htm").await;

    assert!(!forms.is_empty(), "All-pages should have vote forms");
    let form = &forms[0];

    run_list_vote_test(
        &client,
        &form.action_url,
        &form.upvotes_id,
        None,
        Direction::Up,
        1,
        true,
        false,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn all_pages_first_item_downvote() {
    let client = form_test_client();
    let forms = fetch_and_extract_forms(&client, "/all-pages.htm").await;

    assert!(!forms.is_empty(), "All-pages should have vote forms");
    let form = &forms[0];

    run_list_vote_test(
        &client,
        &form.action_url,
        &form.upvotes_id,
        None,
        Direction::Down,
        -1,
        false,
        true,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn all_pages_second_item_upvote() {
    let client = form_test_client();
    let forms = fetch_and_extract_forms(&client, "/all-pages.htm").await;

    assert!(
        forms.len() >= 2,
        "All-pages should have at least 2 vote forms"
    );
    let form = &forms[1];

    run_list_vote_test(
        &client,
        &form.action_url,
        &form.upvotes_id,
        None,
        Direction::Up,
        1,
        true,
        false,
    )
    .await;
}

#[tokio::test]
#[serial]
async fn all_pages_second_item_downvote() {
    let client = form_test_client();
    let forms = fetch_and_extract_forms(&client, "/all-pages.htm").await;

    assert!(
        forms.len() >= 2,
        "All-pages should have at least 2 vote forms"
    );
    let form = &forms[1];

    run_list_vote_test(
        &client,
        &form.action_url,
        &form.upvotes_id,
        None,
        Direction::Down,
        -1,
        false,
        true,
    )
    .await;
}
