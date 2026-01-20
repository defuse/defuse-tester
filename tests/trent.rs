// TRENT (Trusted Random Entropy) Tests
//
// Tests for the trusted third party random number generator service.
// TRENT provides verifiable random numbers for drawings, contests, and lotteries.

mod common;

use common::{assert_body_contains, assert_content_type, assert_success, client, post_form, url};
use regex::Regex;
use reqwest::multipart;
use std::time::Duration;

// =============================================================================
// Helper functions
// =============================================================================

/// Get the main TRENT page
async fn get_main_page() -> reqwest::Response {
    client()
        .get(url("/trustedthirdparty.htm"))
        .send()
        .await
        .expect("Failed to fetch TRENT page")
}

/// View a specific drawing by number
async fn view_drawing(drawing_num: u32) -> reqwest::Response {
    client()
        .get(url(&format!(
            "/trustedthirdparty.htm?drawingnum={}",
            drawing_num
        )))
        .send()
        .await
        .expect("Failed to view drawing")
}

/// Extract drawing number from reservation response
/// Looks for pattern: "Your drawing number is: <b>123</b>"
fn extract_drawing_number(body: &str) -> Option<u32> {
    let re = Regex::new(r"Your drawing number is:\s*<b>(\d+)</b>").ok()?;
    let caps = re.captures(body)?;
    caps.get(1)?.as_str().parse().ok()
}

/// Extract passcode from reservation response
/// Looks for pattern: "Your passcode is: <b>abc123...</b>"
fn extract_passcode(body: &str) -> Option<String> {
    let re = Regex::new(r"Your passcode is:\s*<b>([a-f0-9]+)</b>").ok()?;
    let caps = re.captures(body)?;
    Some(caps.get(1)?.as_str().to_string())
}

/// Extract file hash from confirmation page
/// Looks for pattern: name="file1hash" value="abc123..."
fn extract_file_hash(body: &str, file_num: u8) -> Option<String> {
    let pattern = format!(r#"name="file{}hash"\s+value="([a-f0-9]+)""#, file_num);
    let re = Regex::new(&pattern).ok()?;
    let caps = re.captures(body)?;
    Some(caps.get(1)?.as_str().to_string())
}

/// Reserve a drawing with the given review time (in seconds)
/// Returns (drawing_number, passcode)
async fn reserve_drawing(prereview: u32) -> (u32, String) {
    let resp = post_form(
        "/trustedthirdparty.htm",
        &[
            ("makedrawingnumber", "Reserve a Drawing Number"),
            ("prereview", &prereview.to_string()),
        ],
    )
    .await;
    let body = resp.text().await.unwrap();
    let drawing_num = extract_drawing_number(&body).expect("Should get drawing number");
    let passcode = extract_passcode(&body).expect("Should get passcode");
    (drawing_num, passcode)
}

/// Parameters for creating a drawing
#[derive(Default)]
struct CreateParams<'a> {
    drawing_num: u32,
    passcode: &'a str,
    name: &'a str,
    description: &'a str,
    lowval: i32,
    highval: i32,
    numgen: i32,
    randlines1: i32,
    randlines2: i32,
    randlines3: i32,
    chosentwice: bool,
    file1hash: &'a str,
    file2hash: &'a str,
    file3hash: &'a str,
}

/// Create drawing step 1 - gets confirmation page or error (no files)
async fn create_step1(params: &CreateParams<'_>) -> reqwest::Response {
    let chosentwice_val = if params.chosentwice { "true" } else { "" };
    post_form(
        "/trustedthirdparty.htm",
        &[
            ("create", "Pick the Random Numbers!"),
            ("drawingnumber", &params.drawing_num.to_string()),
            ("passcode", params.passcode),
            ("name", params.name),
            ("description", params.description),
            ("lowval", &params.lowval.to_string()),
            ("highval", &params.highval.to_string()),
            ("numgen", &params.numgen.to_string()),
            ("randlines1", &params.randlines1.to_string()),
            ("randlines2", &params.randlines2.to_string()),
            ("randlines3", &params.randlines3.to_string()),
            ("chosentwice", chosentwice_val),
        ],
    )
    .await
}

/// Create drawing step 2 - confirm and complete (no files)
async fn confirm_step2(params: &CreateParams<'_>) -> reqwest::Response {
    let chosentwice_val = if params.chosentwice { "true" } else { "" };
    post_form(
        "/trustedthirdparty.htm",
        &[
            (
                "create",
                "These values are correct, draw my random numbers!",
            ),
            ("confirmed", "true"),
            ("drawingnumber", &params.drawing_num.to_string()),
            ("passcode", params.passcode),
            ("name", params.name),
            ("description", params.description),
            ("lowval", &params.lowval.to_string()),
            ("highval", &params.highval.to_string()),
            ("numgen", &params.numgen.to_string()),
            ("randlines1", &params.randlines1.to_string()),
            ("randlines2", &params.randlines2.to_string()),
            ("randlines3", &params.randlines3.to_string()),
            ("chosentwice", chosentwice_val),
            ("file1hash", params.file1hash),
            ("file2hash", params.file2hash),
            ("file3hash", params.file3hash),
        ],
    )
    .await
}

/// Reserve and complete a drawing in one step (for tests needing completed drawings)
/// Returns the drawing number
async fn complete_drawing(
    name: &str,
    desc: &str,
    numgen: i32,
    low: i32,
    high: i32,
) -> (u32, String) {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let params = CreateParams {
        drawing_num,
        passcode: &passcode,
        name,
        description: desc,
        lowval: low,
        highval: high,
        numgen,
        ..Default::default()
    };
    let resp = confirm_step2(&params).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Drawing Complete!"),
        "Drawing should complete successfully"
    );
    (drawing_num, passcode)
}

/// Generate a unique test name with timestamp
fn test_name() -> String {
    format!(
        "TRENT_TEST_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    )
}

/// Upload a file and get the confirmation page (step 1 with file)
async fn create_with_file(
    params: &CreateParams<'_>,
    file_content: &[u8],
    filename: &str,
) -> reqwest::Response {
    let part = multipart::Part::bytes(file_content.to_vec())
        .file_name(filename.to_string())
        .mime_str("text/plain")
        .unwrap();

    let chosentwice_val = if params.chosentwice { "true" } else { "" };

    let form = multipart::Form::new()
        .text("create", "Pick the Random Numbers!")
        .text("drawingnumber", params.drawing_num.to_string())
        .text("passcode", params.passcode.to_string())
        .text("name", params.name.to_string())
        .text("description", params.description.to_string())
        .text("lowval", params.lowval.to_string())
        .text("highval", params.highval.to_string())
        .text("numgen", params.numgen.to_string())
        .text("randlines1", params.randlines1.to_string())
        .text("randlines2", params.randlines2.to_string())
        .text("randlines3", params.randlines3.to_string())
        .text("chosentwice", chosentwice_val.to_string())
        .part("file1", part);

    client()
        .post(url("/trustedthirdparty.htm"))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload file")
}

// =============================================================================
// 1. Main Page (GET)
// =============================================================================

#[tokio::test]
async fn main_page_loads() {
    let resp = get_main_page().await;
    assert_success(&resp, "main_page_loads");
}

#[tokio::test]
async fn main_page_has_title() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("TRENT"),
        "Page should contain TRENT in title or content"
    );
}

#[tokio::test]
async fn main_page_shows_utc_time() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(&body, "Current UTC Time:", "main_page_shows_utc_time");
}

#[tokio::test]
async fn main_page_has_validation_form() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(&body, r#"name="drawingnum""#, "validation form input");
    assert_body_contains(&body, r#"value="Check""#, "validation form submit");
}

#[tokio::test]
async fn main_page_has_reservation_form() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(&body, r#"name="prereview""#, "reservation form select");
    assert_body_contains(
        &body,
        r#"name="makedrawingnumber""#,
        "reservation form submit",
    );
}

#[tokio::test]
async fn main_page_has_review_time_options() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    // Check for various review time options
    assert_body_contains(&body, r#"value="0""#, "instant option");
    assert_body_contains(&body, r#"value="21600""#, "6 hour option");
    assert_body_contains(&body, r#"value="86400""#, "24 hour option");
    assert_body_contains(&body, r#"value="345600""#, "4 day option");
    assert_body_contains(&body, r#"value="2592000""#, "30 day option");
}

#[tokio::test]
async fn main_page_has_creation_form() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(&body, r#"name="drawingnumber""#, "drawing number input");
    assert_body_contains(&body, r#"name="passcode""#, "passcode input");
    assert_body_contains(&body, r#"name="name""#, "name input");
    assert_body_contains(&body, r#"name="description""#, "description input");
    assert_body_contains(&body, r#"name="lowval""#, "lowval input");
    assert_body_contains(&body, r#"name="highval""#, "highval input");
    assert_body_contains(&body, r#"name="numgen""#, "numgen input");
}

#[tokio::test]
async fn main_page_has_file_inputs() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(&body, r#"name="file1""#, "file1 input");
    assert_body_contains(&body, r#"name="file2""#, "file2 input");
    assert_body_contains(&body, r#"name="file3""#, "file3 input");
    assert_body_contains(&body, "10MB", "file size limit note");
}

#[tokio::test]
async fn main_page_has_chosentwice_checkbox() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(&body, r#"name="chosentwice""#, "chosentwice checkbox");
}

#[tokio::test]
async fn main_page_has_explanation() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(&body, "What is TRENT?", "explanation section");
}

#[tokio::test]
async fn main_page_has_trent_image() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert_body_contains(&body, "trusted_third_party.png", "TRENT image");
}

// =============================================================================
// 2. View Drawing Status (GET ?drawingnum=X)
// =============================================================================

#[tokio::test]
async fn view_nonexistent_drawing() {
    let resp = view_drawing(999999999).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("does not exist"),
        "Should show 'does not exist' for non-existent drawing"
    );
    // Should have red background error styling
    assert!(
        body.contains("#FF9F9F") || body.contains("ff9f9f"),
        "Should have red error background"
    );
}

#[tokio::test]
async fn view_drawing_number_zero() {
    let resp = view_drawing(0).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("does not exist"),
        "Drawing 0 should not exist"
    );
}

#[tokio::test]
async fn view_drawing_negative_number() {
    // PHP casts to int, so -1 becomes a large number or 0
    let resp = client()
        .get(url("/trustedthirdparty.htm?drawingnum=-1"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "view_drawing_negative_number");
    let _body = resp.text().await.unwrap();
    // Should handle gracefully (either not exist or show something)
}

#[tokio::test]
async fn view_drawing_non_numeric() {
    let resp = client()
        .get(url("/trustedthirdparty.htm?drawingnum=abc"))
        .send()
        .await
        .unwrap();
    let body = resp.text().await.unwrap();
    // PHP (int)"abc" = 0, so should show "does not exist"
    assert!(
        body.contains("does not exist"),
        "Non-numeric should cast to 0 and not exist"
    );
}

#[tokio::test]
async fn view_pending_drawing() {
    // Reserve a drawing with 24-hour review time (won't be completable)
    let (drawing_num, _) = reserve_drawing(86400).await;

    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("not yet been chosen"),
        "Pending drawing should show 'not yet been chosen'"
    );
    // Should show future date when it can be completed
    assert!(
        body.contains("after"),
        "Should show when drawing can be completed"
    );
}

#[tokio::test]
async fn view_completed_drawing() {
    // Create and complete a drawing
    let (drawing_num, _) = complete_drawing(&test_name(), "Test description", 1, 1, 10).await;

    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    // Should show results, not pending message
    assert!(
        !body.contains("not yet been chosen"),
        "Completed drawing should not show pending message"
    );
    assert!(
        body.contains("Results for drawing"),
        "Should show results section"
    );
}

#[tokio::test]
async fn view_completed_has_description() {
    let (drawing_num, _) = complete_drawing(&test_name(), "My test description here", 1, 1, 10).await;

    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Description of drawing"),
        "Should have description section header"
    );
}

#[tokio::test]
async fn view_completed_has_results() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 1, 1, 10).await;

    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains(&format!("Results for drawing #{}", drawing_num)),
        "Should have results section header with drawing number"
    );
}

#[tokio::test]
async fn view_completed_has_verification_info() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 1, 1, 10).await;

    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("How do I tell if the drawing was fair"),
        "Should have verification info section"
    );
}

// =============================================================================
// 3. Reserve Drawing Number (POST makedrawingnumber)
// =============================================================================

#[tokio::test]
async fn reserve_instant() {
    let resp = post_form(
        "/trustedthirdparty.htm",
        &[
            ("makedrawingnumber", "Reserve a Drawing Number"),
            ("prereview", "0"),
        ],
    )
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Your Drawing Number Has Been Reserved"),
        "Should show reservation success"
    );
    // With instant (0), should be able to do drawing immediately
    // The date shown should be very close to now
}

#[tokio::test]
async fn reserve_6_hours() {
    let resp = post_form(
        "/trustedthirdparty.htm",
        &[
            ("makedrawingnumber", "Reserve a Drawing Number"),
            ("prereview", "21600"),
        ],
    )
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Your Drawing Number Has Been Reserved"),
        "Should show reservation success"
    );
    // Should show a future date ~6 hours from now
    assert!(
        body.contains("after"),
        "Should mention when drawing can be done"
    );
}

#[tokio::test]
async fn reserve_24_hours() {
    let resp = post_form(
        "/trustedthirdparty.htm",
        &[
            ("makedrawingnumber", "Reserve a Drawing Number"),
            ("prereview", "86400"),
        ],
    )
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Your Drawing Number Has Been Reserved"),
        "Should show reservation success"
    );
}

#[tokio::test]
async fn reserve_returns_drawing_number() {
    let resp = post_form(
        "/trustedthirdparty.htm",
        &[
            ("makedrawingnumber", "Reserve a Drawing Number"),
            ("prereview", "0"),
        ],
    )
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Your drawing number is:"),
        "Should show drawing number"
    );
    let num = extract_drawing_number(&body);
    assert!(num.is_some(), "Should be able to extract drawing number");
    assert!(num.unwrap() > 0, "Drawing number should be positive");
}

#[tokio::test]
async fn reserve_returns_passcode() {
    let resp = post_form(
        "/trustedthirdparty.htm",
        &[
            ("makedrawingnumber", "Reserve a Drawing Number"),
            ("prereview", "0"),
        ],
    )
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Your passcode is:"),
        "Should show passcode"
    );
    let passcode = extract_passcode(&body);
    assert!(passcode.is_some(), "Should be able to extract passcode");
    assert_eq!(
        passcode.unwrap().len(),
        32,
        "Passcode should be 32 hex chars"
    );
}

#[tokio::test]
async fn reserve_returns_url() {
    let resp = post_form(
        "/trustedthirdparty.htm",
        &[
            ("makedrawingnumber", "Reserve a Drawing Number"),
            ("prereview", "0"),
        ],
    )
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("trustedthirdparty.htm?drawingnum="),
        "Should show URL with drawing number"
    );
}

#[tokio::test]
async fn reserve_shows_availability_date() {
    let resp = post_form(
        "/trustedthirdparty.htm",
        &[
            ("makedrawingnumber", "Reserve a Drawing Number"),
            ("prereview", "86400"),
        ],
    )
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("You will be able to do the drawing after"),
        "Should show availability date"
    );
}

#[tokio::test]
async fn reserve_drawing_viewable() {
    let (drawing_num, _) = reserve_drawing(0).await;

    let resp = view_drawing(drawing_num).await;
    assert_success(&resp, "reserve_drawing_viewable");
    // Drawing should exist (not "does not exist")
    let body = resp.text().await.unwrap();
    assert!(
        !body.contains("does not exist"),
        "Reserved drawing should be viewable"
    );
}

#[tokio::test]
async fn reserve_creates_unique_numbers() {
    let (num1, _) = reserve_drawing(0).await;
    let (num2, _) = reserve_drawing(0).await;
    assert_ne!(num1, num2, "Each reservation should get a unique number");
}

// =============================================================================
// 4. Create Random Numbers - Validation Errors
// =============================================================================

#[tokio::test]
async fn create_wrong_password() {
    let (drawing_num, _) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: "wrongpasswordwrongpasswordwrong1",
        name: "Test",
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Incorrect password"),
        "Should show incorrect password error"
    );
}

#[tokio::test]
async fn create_drawing_not_exists() {
    let resp = create_step1(&CreateParams {
        drawing_num: 999999999,
        passcode: "doesntmatter12345678901234567890",
        name: "Test",
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("does not exist"),
        "Should show 'does not exist' error"
    );
}

#[tokio::test]
async fn create_already_complete() {
    // Create and complete a drawing
    let (drawing_num, passcode) = complete_drawing(&test_name(), "Test", 1, 1, 10).await;

    // Try to complete it again
    let resp = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("already been chosen"),
        "Should show 'already been chosen' error"
    );
}

#[tokio::test]
async fn create_review_not_complete() {
    // Reserve with 24-hour review
    let (drawing_num, passcode) = reserve_drawing(86400).await;

    // Try to complete immediately
    let resp = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("review period") && body.contains("not complete"),
        "Should show review period error"
    );
}

#[tokio::test]
async fn create_invalid_range_low_ge_high() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: 1,
        lowval: 10,  // low >= high
        highval: 5,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("range is invalid"),
        "Should show invalid range error"
    );
}

#[tokio::test]
async fn create_negative_numgen() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: -1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("NEGATIVE"),
        "Should show negative amount error"
    );
}

#[tokio::test]
async fn create_negative_randlines() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: 0,
        randlines1: -1,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("NEGATIVE"),
        "Should show negative amount error"
    );
}

#[tokio::test]
async fn create_too_many_numbers() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: 1001,
        lowval: 1,
        highval: 10000,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("1000"),
        "Should mention 1000 limit"
    );
}

#[tokio::test]
async fn create_too_many_randlines() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: 0,
        randlines1: 1001,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("1000"),
        "Should mention 1000 limit"
    );
}

#[tokio::test]
async fn create_file_too_large() {
    let (drawing_num, passcode) = reserve_drawing(0).await;

    // Create an 11MB file (just over the 10MB limit)
    let file_content = vec![b'x'; 11 * 1024 * 1024];

    let resp = create_with_file(
        &CreateParams {
            drawing_num,
            passcode: &passcode,
            name: "Test",
            numgen: 1,
            lowval: 1,
            highval: 10,
            randlines1: 1,
            ..Default::default()
        },
        &file_content,
        "large.txt",
    )
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("maximum file size") || body.contains("10MB") || body.contains("too large"),
        "Should reject file larger than 10MB"
    );
}

#[tokio::test]
async fn create_not_enough_lines() {
    let (drawing_num, passcode) = reserve_drawing(0).await;

    // Create a file with only 2 lines
    let file_content = b"line1\nline2\n";

    let resp = create_with_file(
        &CreateParams {
            drawing_num,
            passcode: &passcode,
            name: "Test",
            numgen: 0,
            randlines1: 5, // Request 5 lines from 2-line file (no repeat)
            ..Default::default()
        },
        file_content,
        "test.txt",
    )
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("enough lines"),
        "Should show not enough lines error"
    );
}

// =============================================================================
// 5. Create Random Numbers - Confirmation Step
// =============================================================================

#[tokio::test]
async fn create_shows_confirmation() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test Name",
        description: "Test Description",
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();

    // Should show confirmation page
    assert!(
        body.contains("Confirm Drawing"),
        "Should show confirmation page"
    );

    // Should NOT be completed yet
    assert!(
        !body.contains("Drawing Complete!"),
        "Should NOT complete without confirmation"
    );

    // Drawing should still be incomplete when viewed
    let view_resp = view_drawing(drawing_num).await;
    let view_body = view_resp.text().await.unwrap();
    assert!(
        view_body.contains("not yet been chosen"),
        "Drawing should still be pending after step 1"
    );
}

#[tokio::test]
async fn confirm_shows_name() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let test_name = "UniqueTestName12345";
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: test_name,
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains(test_name),
        "Confirmation should show the name"
    );
}

#[tokio::test]
async fn confirm_shows_description() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let test_desc = "UniqueDescription67890";
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        description: test_desc,
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains(test_desc),
        "Confirmation should show the description"
    );
}

#[tokio::test]
async fn confirm_shows_file_info() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let file_content = b"test file content\n";

    let resp = create_with_file(
        &CreateParams {
            drawing_num,
            passcode: &passcode,
            name: "Test",
            numgen: 0,
            randlines1: 1,
            ..Default::default()
        },
        file_content,
        "myfile.txt",
    )
    .await;
    let body = resp.text().await.unwrap();

    // Should show file info
    assert!(
        body.contains("SHA256:"),
        "Should show file SHA256"
    );
}

#[tokio::test]
async fn confirm_shows_randlines() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: 0,
        randlines1: 5,
        randlines2: 3,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    // Check both display and hidden field
    assert!(
        body.contains("Random Lines: 5"),
        "Should show Random Lines: 5 in display"
    );
    assert!(
        body.contains(r#"name="randlines1" value="5""#),
        "Should have hidden randlines1 field with value 5"
    );
}

#[tokio::test]
async fn confirm_shows_range() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: 1,
        lowval: 42,
        highval: 99,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("42") && body.contains("99"),
        "Should show range values"
    );
}

#[tokio::test]
async fn confirm_shows_numgen() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: 7,
        lowval: 1,
        highval: 100,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    // Check hidden field has correct value
    assert!(
        body.contains(r#"name="numgen" value="7""#),
        "Should have hidden numgen field with value 7"
    );
    // Check display shows the amount section
    assert!(
        body.contains("Amount of Numbers"),
        "Should show Amount of Numbers heading"
    );
}

#[tokio::test]
async fn confirm_has_hidden_fields() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains(r#"name="confirmed" value="true""#),
        "Should have hidden confirmed field"
    );
    assert!(
        body.contains(r#"name="drawingnumber""#),
        "Should have hidden drawingnumber field"
    );
    assert!(
        body.contains(r#"name="passcode""#),
        "Should have hidden passcode field"
    );
}

#[tokio::test]
async fn confirm_has_confirm_button() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("These values are correct"),
        "Should have confirm button"
    );
}

#[tokio::test]
async fn confirm_has_cancel_button() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = create_step1(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Test",
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Cancel"),
        "Should have cancel button"
    );
}

// =============================================================================
// 6. Create Random Numbers - Success
// =============================================================================

#[tokio::test]
async fn create_success_message() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Drawing Complete!"),
        "Should show completion message"
    );
}

#[tokio::test]
async fn create_success_has_url() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains(&format!("drawingnum={}", drawing_num)),
        "Should show URL to results"
    );
}

#[tokio::test]
async fn create_results_viewable() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 1, 1, 10).await;
    let resp = view_drawing(drawing_num).await;
    assert_success(&resp, "create_results_viewable");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Results for drawing"),
        "Should be able to view results"
    );
}

#[tokio::test]
async fn create_results_has_drawing_number() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 1, 1, 10).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains(&format!("DRAWING NUMBER: {}", drawing_num)),
        "Results should contain drawing number"
    );
}

#[tokio::test]
async fn create_results_has_date() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 1, 1, 10).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("DRAWING DATE:"),
        "Results should contain drawing date"
    );
}

#[tokio::test]
async fn create_results_has_amount() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 5, 1, 100).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("AMOUNT OF NUMBERS: 5"),
        "Results should contain amount of numbers"
    );
}

#[tokio::test]
async fn create_results_has_range() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 1, 42, 99).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("RANGE: 42 TO 99"),
        "Results should contain range"
    );
}

#[tokio::test]
async fn create_results_has_random_numbers() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 3, 1, 100).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("RANDOM NUMBER NUMBER 1:"),
        "Should have random number 1"
    );
    assert!(
        body.contains("RANDOM NUMBER NUMBER 2:"),
        "Should have random number 2"
    );
    assert!(
        body.contains("RANDOM NUMBER NUMBER 3:"),
        "Should have random number 3"
    );
}

#[tokio::test]
async fn create_user_description_preserved() {
    let unique_desc = format!("UniqueDescription_{}", test_name());
    let (drawing_num, _) = complete_drawing(&test_name(), &unique_desc, 1, 1, 10).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains(&unique_desc),
        "User description should be preserved in results"
    );
}

// =============================================================================
// 7. Random Number Generation
// =============================================================================

#[tokio::test]
async fn random_numbers_in_range() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 10, 50, 60).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();

    // Extract random numbers and verify they're in range
    let re = Regex::new(r"RANDOM NUMBER NUMBER \d+: (\d+)").unwrap();
    for cap in re.captures_iter(&body) {
        let num: i32 = cap.get(1).unwrap().as_str().parse().unwrap();
        assert!(
            num >= 50 && num <= 60,
            "Random number {} should be between 50 and 60",
            num
        );
    }
}

#[tokio::test]
async fn random_single_number() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 1, 1, 100).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();

    assert!(
        body.contains("RANDOM NUMBER NUMBER 1:"),
        "Should have exactly one random number"
    );
    assert!(
        !body.contains("RANDOM NUMBER NUMBER 2:"),
        "Should not have second random number"
    );
}

#[tokio::test]
async fn random_multiple_numbers() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 5, 1, 100).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();

    for i in 1..=5 {
        assert!(
            body.contains(&format!("RANDOM NUMBER NUMBER {}:", i)),
            "Should have random number {}",
            i
        );
    }
    assert!(
        !body.contains("RANDOM NUMBER NUMBER 6:"),
        "Should not have 6th random number"
    );
}

#[tokio::test]
async fn random_zero_numbers() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 0, 1, 100).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();

    assert!(
        !body.contains("RANDOM NUMBER NUMBER"),
        "Should not have any random numbers when numgen=0"
    );
    // But should still have the drawing metadata
    assert!(
        body.contains("AMOUNT OF NUMBERS: 0"),
        "Should show amount is 0"
    );
}

#[tokio::test]
async fn random_range_single_value() {
    // When lowval == highval and numgen > 0, PHP considers this invalid
    // ("range is invalid" error because low >= high)
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let resp = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        numgen: 1,
        lowval: 5,
        highval: 5, // Same as lowval
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("range is invalid"),
        "Should reject when lowval >= highval with numgen > 0"
    );
}

#[tokio::test]
async fn random_large_range() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 5, -1000000000, 1000000000).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();

    assert!(
        body.contains("RANGE: -1000000000 TO 1000000000"),
        "Should handle large range"
    );
    // Verify numbers were generated
    assert!(
        body.contains("RANDOM NUMBER NUMBER 1:"),
        "Should generate numbers in large range"
    );
}

#[tokio::test]
async fn random_negative_range() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Test", 5, -100, -10).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();

    assert!(
        body.contains("RANGE: -100 TO -10"),
        "Should handle negative range"
    );

    // Extract and verify numbers are negative
    let re = Regex::new(r"RANDOM NUMBER NUMBER \d+: (-?\d+)").unwrap();
    for cap in re.captures_iter(&body) {
        let num: i32 = cap.get(1).unwrap().as_str().parse().unwrap();
        assert!(
            num >= -100 && num <= -10,
            "Random number {} should be between -100 and -10",
            num
        );
    }
}

// =============================================================================
// 8. File Upload and Random Lines
// =============================================================================

#[tokio::test]
async fn file_sha256_recorded() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let file_content = b"line1\nline2\nline3\n";

    // Step 1: Upload file
    let resp = create_with_file(
        &CreateParams {
            drawing_num,
            passcode: &passcode,
            name: &test_name(),
            numgen: 0,
            randlines1: 1,
            ..Default::default()
        },
        file_content,
        "test.txt",
    )
    .await;
    let body = resp.text().await.unwrap();

    // Extract the hash from confirmation
    let file_hash = extract_file_hash(&body, 1).expect("Should get file hash");

    // Step 2: Confirm
    let resp = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        numgen: 0,
        randlines1: 1,
        file1hash: &file_hash,
        ..Default::default()
    })
    .await;

    // Verify completion
    let body = resp.text().await.unwrap();
    assert!(body.contains("Drawing Complete!"), "Should complete");

    // View results and check hash is recorded
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("FILE1 SHA256:"),
        "Results should contain file SHA256"
    );
}

#[tokio::test]
async fn file_random_lines_selected() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let file_content = b"apple\nbanana\ncherry\norange\ngrape\n";

    // Step 1: Upload file
    let resp = create_with_file(
        &CreateParams {
            drawing_num,
            passcode: &passcode,
            name: &test_name(),
            numgen: 0,
            randlines1: 2,
            ..Default::default()
        },
        file_content,
        "fruits.txt",
    )
    .await;
    let body = resp.text().await.unwrap();
    let file_hash = extract_file_hash(&body, 1).expect("Should get file hash");

    // Step 2: Confirm
    let _ = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        numgen: 0,
        randlines1: 2,
        file1hash: &file_hash,
        ..Default::default()
    })
    .await;

    // View results
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("FILE1 RANDOM LINE 1:"),
        "Should have first random line"
    );
    assert!(
        body.contains("FILE1 RANDOM LINE 2:"),
        "Should have second random line"
    );
}

#[tokio::test]
async fn file_line_preview_shown() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let file_content = b"uniquelinecontentxyz123\nanotherline\n";

    // Step 1: Upload file
    let resp = create_with_file(
        &CreateParams {
            drawing_num,
            passcode: &passcode,
            name: &test_name(),
            numgen: 0,
            randlines1: 1,
            ..Default::default()
        },
        file_content,
        "test.txt",
    )
    .await;
    let body = resp.text().await.unwrap();
    let file_hash = extract_file_hash(&body, 1).expect("Should get file hash");

    // Step 2: Confirm
    let _ = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        numgen: 0,
        randlines1: 1,
        file1hash: &file_hash,
        ..Default::default()
    })
    .await;

    // View results
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("LINE PREVIEW:"),
        "Should show line preview"
    );
}

#[tokio::test]
async fn file_line_number_shown() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let file_content = b"line0\nline1\nline2\nline3\nline4\n";

    // Step 1: Upload file
    let resp = create_with_file(
        &CreateParams {
            drawing_num,
            passcode: &passcode,
            name: &test_name(),
            numgen: 0,
            randlines1: 1,
            ..Default::default()
        },
        file_content,
        "test.txt",
    )
    .await;
    let body = resp.text().await.unwrap();
    let file_hash = extract_file_hash(&body, 1).expect("Should get file hash");

    // Step 2: Confirm
    let _ = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        numgen: 0,
        randlines1: 1,
        file1hash: &file_hash,
        ..Default::default()
    })
    .await;

    // View results
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("RANDOM LINE NUMBER (FILE1):"),
        "Should show random line number"
    );
}

#[tokio::test]
async fn file_no_repeat_lines() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    // File with 5 lines, request all 5 (no repeat should give unique lines)
    let file_content = b"line0\nline1\nline2\nline3\nline4\n";

    // Step 1: Upload file
    let resp = create_with_file(
        &CreateParams {
            drawing_num,
            passcode: &passcode,
            name: &test_name(),
            numgen: 0,
            randlines1: 5,
            chosentwice: false,
            ..Default::default()
        },
        file_content,
        "test.txt",
    )
    .await;
    let body = resp.text().await.unwrap();
    let file_hash = extract_file_hash(&body, 1).expect("Should get file hash");

    // Step 2: Confirm
    let _ = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        numgen: 0,
        randlines1: 5,
        chosentwice: false,
        file1hash: &file_hash,
        ..Default::default()
    })
    .await;

    // View results and extract line numbers
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();

    let re = Regex::new(r"RANDOM LINE NUMBER \(FILE1\): (\d+)").unwrap();
    let line_nums: Vec<u32> = re
        .captures_iter(&body)
        .filter_map(|cap| cap.get(1)?.as_str().parse().ok())
        .collect();

    // Should have 5 unique line numbers
    assert_eq!(line_nums.len(), 5, "Should have 5 line numbers");
    let unique: std::collections::HashSet<_> = line_nums.iter().collect();
    assert_eq!(unique.len(), 5, "All line numbers should be unique (no repeat)");
}

#[tokio::test]
async fn file_allow_repeat_lines() {
    // This test verifies that with chosentwice=true, lines CAN repeat
    // We can't guarantee they WILL repeat, but we verify the setting is accepted
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let file_content = b"onlyline\n"; // Single line file

    // Step 1: Upload file - request 3 lines from 1-line file with repeat allowed
    let resp = create_with_file(
        &CreateParams {
            drawing_num,
            passcode: &passcode,
            name: &test_name(),
            numgen: 0,
            randlines1: 3,
            chosentwice: true, // Allow repeat
            ..Default::default()
        },
        file_content,
        "test.txt",
    )
    .await;
    let body = resp.text().await.unwrap();

    // Should show confirmation page (not error about not enough lines)
    assert!(
        body.contains("Confirm Drawing"),
        "Should show confirmation page with chosentwice=true"
    );
}

#[tokio::test]
async fn file_zero_randlines() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    let file_content = b"line1\nline2\n";

    // Step 1: Upload file but request 0 random lines
    let resp = create_with_file(
        &CreateParams {
            drawing_num,
            passcode: &passcode,
            name: &test_name(),
            numgen: 1,
            lowval: 1,
            highval: 10,
            randlines1: 0, // Zero lines requested
            ..Default::default()
        },
        file_content,
        "test.txt",
    )
    .await;
    let body = resp.text().await.unwrap();
    let _file_hash = extract_file_hash(&body, 1);

    // File hash might not be recorded if randlines is 0 and file isn't used
    // Just verify we get confirmation page
    assert!(
        body.contains("Confirm Drawing"),
        "Should show confirmation"
    );
}

#[tokio::test]
async fn file_multiple_files() {
    let (drawing_num, passcode) = reserve_drawing(0).await;

    // Three different files with distinct content
    let file1_content = b"file1 line1\nfile1 line2\n";
    let file2_content = b"file2 content here\n";
    let file3_content = b"third file data\nanother line\n";

    // Build multipart form with all 3 files
    let part1 = multipart::Part::bytes(file1_content.to_vec())
        .file_name("file1.txt")
        .mime_str("text/plain")
        .unwrap();
    let part2 = multipart::Part::bytes(file2_content.to_vec())
        .file_name("file2.txt")
        .mime_str("text/plain")
        .unwrap();
    let part3 = multipart::Part::bytes(file3_content.to_vec())
        .file_name("file3.txt")
        .mime_str("text/plain")
        .unwrap();

    let form = multipart::Form::new()
        .text("create", "Pick the Random Numbers!")
        .text("drawingnumber", drawing_num.to_string())
        .text("passcode", passcode.clone())
        .text("name", test_name())
        .text("description", "Testing 3 files")
        .text("lowval", "1")
        .text("highval", "10")
        .text("numgen", "1")
        .text("randlines1", "1")
        .text("randlines2", "1")
        .text("randlines3", "1")
        .text("chosentwice", "")
        .part("file1", part1)
        .part("file2", part2)
        .part("file3", part3);

    let resp = client()
        .post(url("/trustedthirdparty.htm"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    let body = resp.text().await.unwrap();

    // Extract all 3 file hashes from confirmation page
    let file1_hash = extract_file_hash(&body, 1).expect("Should get file1 hash");
    let file2_hash = extract_file_hash(&body, 2).expect("Should get file2 hash");
    let file3_hash = extract_file_hash(&body, 3).expect("Should get file3 hash");

    // Confirm the drawing
    let _ = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        description: "Testing 3 files",
        numgen: 1,
        lowval: 1,
        highval: 10,
        randlines1: 1,
        randlines2: 1,
        randlines3: 1,
        file1hash: &file1_hash,
        file2hash: &file2_hash,
        file3hash: &file3_hash,
        ..Default::default()
    })
    .await;

    // View results and verify all 3 file hashes are present
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();

    assert!(
        body.contains("FILE1 SHA256:"),
        "Should have FILE1 SHA256 in results"
    );
    assert!(
        body.contains("FILE2 SHA256:"),
        "Should have FILE2 SHA256 in results"
    );
    assert!(
        body.contains("FILE3 SHA256:"),
        "Should have FILE3 SHA256 in results"
    );
    // Verify content from all 3 files was used (checking LINE PREVIEW content)
    assert!(
        body.contains("file1 line"),
        "Should have content from file 1"
    );
    assert!(
        body.contains("file2 content"),
        "Should have content from file 2"
    );
    assert!(
        body.contains("third file") || body.contains("another line"),
        "Should have content from file 3"
    );
}

// BUG: Random lines from FILE2 and FILE3 are incorrectly labeled as FILE1 in results.
// All lines show "FILE1 RANDOM LINE" and "RANDOM LINE NUMBER (FILE1)" regardless of
// which file they actually came from. See docs/issues/trent-multifile-label-bug.md
#[tokio::test]
#[ignore = "BUG: FILE2/FILE3 random lines mislabeled as FILE1 - see docs/issues/trent-multifile-label-bug.md"]
async fn file_multiple_files_correct_labels() {
    let (drawing_num, passcode) = reserve_drawing(0).await;

    let file1_content = b"file1 line1\nfile1 line2\n";
    let file2_content = b"file2 content here\n";
    let file3_content = b"third file data\n";

    let part1 = multipart::Part::bytes(file1_content.to_vec())
        .file_name("file1.txt")
        .mime_str("text/plain")
        .unwrap();
    let part2 = multipart::Part::bytes(file2_content.to_vec())
        .file_name("file2.txt")
        .mime_str("text/plain")
        .unwrap();
    let part3 = multipart::Part::bytes(file3_content.to_vec())
        .file_name("file3.txt")
        .mime_str("text/plain")
        .unwrap();

    let form = multipart::Form::new()
        .text("create", "Pick the Random Numbers!")
        .text("drawingnumber", drawing_num.to_string())
        .text("passcode", passcode.clone())
        .text("name", test_name())
        .text("description", "Test labels")
        .text("lowval", "1")
        .text("highval", "10")
        .text("numgen", "0")
        .text("randlines1", "1")
        .text("randlines2", "1")
        .text("randlines3", "1")
        .text("chosentwice", "")
        .part("file1", part1)
        .part("file2", part2)
        .part("file3", part3);

    let resp = client()
        .post(url("/trustedthirdparty.htm"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    let body = resp.text().await.unwrap();

    let file1_hash = extract_file_hash(&body, 1).expect("Should get file1 hash");
    let file2_hash = extract_file_hash(&body, 2).expect("Should get file2 hash");
    let file3_hash = extract_file_hash(&body, 3).expect("Should get file3 hash");

    let _ = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        description: "Test labels",
        numgen: 0,
        randlines1: 1,
        randlines2: 1,
        randlines3: 1,
        file1hash: &file1_hash,
        file2hash: &file2_hash,
        file3hash: &file3_hash,
        ..Default::default()
    })
    .await;

    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();

    // These assertions verify correct labeling (currently broken)
    assert!(
        body.contains("RANDOM LINE NUMBER (FILE1):"),
        "Should have FILE1 random line label"
    );
    assert!(
        body.contains("RANDOM LINE NUMBER (FILE2):"),
        "Should have FILE2 random line label"
    );
    assert!(
        body.contains("RANDOM LINE NUMBER (FILE3):"),
        "Should have FILE3 random line label"
    );
}

// =============================================================================
// 9. Edge Cases
// =============================================================================

/// Unicode characters in name should show validation error (database is latin1)
/// The form values should be preserved so user can fix the input
#[tokio::test]
async fn name_with_unicode_shows_error() {
    let unicode_name = "Test 日本語 🎉"; // CJK and emoji are NOT latin1
    let (drawing_num, passcode) = reserve_drawing(0).await;

    let params = CreateParams {
        drawing_num,
        passcode: &passcode,
        name: unicode_name,
        description: "Normal description",
        lowval: 1,
        highval: 10,
        numgen: 1,
        ..Default::default()
    };

    // Step 1 should show error, not confirmation
    let resp = create_step1(&params).await;
    let body = resp.text().await.unwrap();

    // Should NOT show confirmation page - should show error instead
    assert!(
        !body.contains("Please verify that the information below is correct"),
        "Should not show confirmation page when name has unsupported unicode. Got confirmation page instead of error."
    );

    // Should show the main form (create random numbers section)
    assert!(
        body.contains("Create Random Numbers"),
        "Should show the create random numbers form so user can fix the input"
    );

    // Should show the Latin-1 error message
    assert!(
        body.contains("can only contain Latin-1 characters"),
        "Should show Latin-1 validation error message"
    );

    // Form values should be preserved for re-editing
    assert!(
        body.contains("Normal description"),
        "Description should be preserved in form"
    );

    // The unicode name should be preserved in the form (HTML-escaped)
    assert!(
        body.contains("Test 日本語") || body.contains("Test "),
        "Name should be preserved in form so user can edit it"
    );
}

/// Unicode characters in description should show validation error
#[tokio::test]
async fn description_with_unicode_shows_error() {
    let unicode_desc = "Description with emoji 🎉 and CJK 日本語";
    let (drawing_num, passcode) = reserve_drawing(0).await;

    let params = CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Normal Name",
        description: unicode_desc,
        lowval: 1,
        highval: 10,
        numgen: 1,
        ..Default::default()
    };

    let resp = create_step1(&params).await;
    let body = resp.text().await.unwrap();

    // Should NOT show confirmation page
    assert!(
        !body.contains("Please verify that the information below is correct"),
        "Should not show confirmation page when description has unsupported unicode"
    );

    // Should show the Latin-1 error message
    assert!(
        body.contains("can only contain Latin-1 characters"),
        "Should show Latin-1 validation error message"
    );

    // Name should be preserved
    assert!(
        body.contains("Normal Name"),
        "Name should be preserved in form"
    );

    // Description should be preserved (so user can fix it)
    assert!(
        body.contains("Description with emoji"),
        "Description should be preserved in form so user can edit it"
    );
}

/// Latin-1 extended characters (like ü, é, ñ) SHOULD work since they're in the 0-255 range
#[tokio::test]
async fn name_with_latin1_extended_works() {
    // These are all valid Latin-1 characters (code points <= 255)
    let latin1_name = "Über café señor";
    let (drawing_num, _) = complete_drawing(latin1_name, "Test", 1, 1, 10).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();

    // Drawing should complete successfully
    assert!(
        !body.contains("not yet been chosen"),
        "Drawing with Latin-1 extended chars should complete"
    );
}

#[tokio::test]
async fn description_with_html() {
    let html_desc = "<script>alert('xss')</script><b>bold</b>";
    let (drawing_num, _) = complete_drawing(&test_name(), html_desc, 1, 1, 10).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    // HTML should be escaped
    assert!(
        !body.contains("<script>alert"),
        "Script tags should be escaped"
    );
    assert!(
        body.contains("&lt;script&gt;"),
        "Script tags should be HTML escaped to &lt;script&gt;"
    );
}

#[tokio::test]
async fn description_multiline() {
    let multiline_desc = "Line 1\nLine 2\nLine 3";
    let (drawing_num, _) = complete_drawing(&test_name(), multiline_desc, 1, 1, 10).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    // Newlines should be preserved in textarea
    assert!(
        body.contains("Line 1") && body.contains("Line 2") && body.contains("Line 3"),
        "Multiline description should be preserved"
    );
}

#[tokio::test]
async fn empty_name() {
    let (drawing_num, _) = complete_drawing("", "Test description", 1, 1, 10).await;
    let resp = view_drawing(drawing_num).await;
    assert_success(&resp, "empty_name");
    // Should still work with empty name
}

#[tokio::test]
async fn empty_description() {
    let (drawing_num, _) = complete_drawing(&test_name(), "", 1, 1, 10).await;
    let resp = view_drawing(drawing_num).await;
    assert_success(&resp, "empty_description");
}

#[tokio::test]
async fn no_files_no_numbers() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Just a description, no random stuff", 0, 0, 0).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Results for drawing"),
        "Should create valid drawing with no random content"
    );
}

#[tokio::test]
async fn whitespace_passcode() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    // Try with whitespace around passcode
    let padded_passcode = format!("  {}  ", passcode);
    let resp = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &padded_passcode,
        name: &test_name(),
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    // PHP trims the passcode, so this should work
    assert!(
        body.contains("Drawing Complete!"),
        "Whitespace-padded passcode should work (trimmed)"
    );
}

#[tokio::test]
async fn max_numbers_1000() {
    let (drawing_num, _) = complete_drawing(&test_name(), "Max numbers test", 1000, 1, 10000).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("AMOUNT OF NUMBERS: 1000"),
        "Should allow 1000 numbers"
    );
    assert!(
        body.contains("RANDOM NUMBER NUMBER 1000:"),
        "Should have 1000th number"
    );
}

#[tokio::test]
async fn drawing_number_large() {
    // Test with a very large drawing number
    let resp = view_drawing(2147483647).await; // Max i32
    let body = resp.text().await.unwrap();
    // Should show "does not exist" error
    assert!(
        body.contains("does not exist"),
        "Should show 'does not exist' for large drawing number"
    );
}

// =============================================================================
// 10. Security
// =============================================================================

#[tokio::test]
async fn password_case_sensitive() {
    let (drawing_num, passcode) = reserve_drawing(0).await;
    // Try with uppercase version
    let wrong_passcode = passcode.to_uppercase();
    let resp = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &wrong_passcode,
        name: &test_name(),
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Incorrect password"),
        "Password should be case sensitive"
    );
}

#[tokio::test]
async fn drawing_immutable_after_complete() {
    let (drawing_num, passcode) = complete_drawing(&test_name(), "Original", 1, 1, 10).await;

    // Try to complete again with different parameters
    let resp = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: "Modified",
        description: "Trying to change",
        numgen: 5,
        lowval: 100,
        highval: 200,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("already been chosen"),
        "Should not allow modification after completion"
    );

    // Verify original values preserved (numgen was 1, not 5)
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    // Check original numgen=1 is preserved (not modified to 5)
    assert!(
        body.contains("AMOUNT OF NUMBERS: 1"),
        "Original numgen=1 should be preserved"
    );
    // The modified numgen=5 should NOT be there
    assert!(
        !body.contains("AMOUNT OF NUMBERS: 5"),
        "Modified numgen=5 should not appear"
    );
}

#[tokio::test]
async fn review_period_enforced() {
    // Reserve with 24-hour review
    let (drawing_num, passcode) = reserve_drawing(86400).await;

    // Try to complete immediately
    let resp = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("review period"),
        "Review period should be enforced"
    );
}

#[tokio::test]
async fn xss_prevention_name() {
    let xss_name = "<img src=x onerror=alert(1)>";
    let (drawing_num, _) = complete_drawing(xss_name, "Test", 1, 1, 10).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        !body.contains("<img src=x onerror"),
        "XSS in name should be escaped"
    );
}

#[tokio::test]
async fn xss_prevention_description() {
    let xss_desc = "Test <script>document.cookie</script> end";
    let (drawing_num, _) = complete_drawing(&test_name(), xss_desc, 1, 1, 10).await;
    let resp = view_drawing(drawing_num).await;
    let body = resp.text().await.unwrap();
    assert!(
        !body.contains("<script>document.cookie"),
        "XSS in description should be escaped"
    );
}

// =============================================================================
// 11. Review Period Timing
// =============================================================================

#[tokio::test]
async fn review_period_blocks_then_allows() {
    // Reserve with 5-second review period
    let (drawing_num, passcode) = reserve_drawing(5).await;

    // Should fail immediately
    let resp = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("review period"),
        "Should be blocked before review period ends"
    );

    // Wait for review period to elapse (with buffer)
    tokio::time::sleep(Duration::from_secs(8)).await;

    // Should succeed now
    let resp = confirm_step2(&CreateParams {
        drawing_num,
        passcode: &passcode,
        name: &test_name(),
        numgen: 1,
        lowval: 1,
        highval: 10,
        ..Default::default()
    })
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Drawing Complete!"),
        "Should succeed after review period"
    );
}

#[tokio::test]
async fn review_period_custom_value() {
    // PHP accepts any integer, not just dropdown values
    let resp = post_form(
        "/trustedthirdparty.htm",
        &[
            ("makedrawingnumber", "Reserve a Drawing Number"),
            ("prereview", "3"), // Custom 3-second value
        ],
    )
    .await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Your Drawing Number Has Been Reserved"),
        "Should accept custom prereview value"
    );
}

// =============================================================================
// 12. Content and Headers
// =============================================================================

#[tokio::test]
async fn main_page_content_type() {
    let resp = get_main_page().await;
    assert_content_type(&resp, "text/html", "main_page_content_type");
}

#[tokio::test]
async fn main_page_has_source_link() {
    let resp = get_main_page().await;
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("github.com") && body.contains("trustedthirdparty"),
        "Should have link to source code on GitHub"
    );
}
