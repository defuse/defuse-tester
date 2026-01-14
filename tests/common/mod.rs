//! Shared test utilities for defuse-tester
//!
//! Provides HTTP client, base URL configuration, and common assertions.

use reqwest::Client;

/// Create a new HTTP client for testing.
/// - Does NOT follow redirects automatically (so we can test redirect behavior)
/// - Has reasonable timeouts
pub fn client() -> Client {
    Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

/// Get the base URL to test against.
/// DEFUSE_URL environment variable must be set.
pub fn base_url() -> String {
    std::env::var("DEFUSE_URL").expect("DEFUSE_URL environment variable must be set")
}

/// Build a full URL from a path
pub fn url(path: &str) -> String {
    let base = base_url();
    let base = base.trim_end_matches('/');
    if path.starts_with('/') {
        format!("{}{}", base, path)
    } else {
        format!("{}/{}", base, path)
    }
}

/// Assert that a response has a successful status code (2xx)
pub fn assert_success(response: &reqwest::Response, context: &str) {
    assert!(
        response.status().is_success(),
        "{}: expected 2xx status, got {}",
        context,
        response.status()
    );
}

/// Assert that a response has a specific status code
pub fn assert_status(response: &reqwest::Response, expected: u16, context: &str) {
    assert_eq!(
        response.status().as_u16(),
        expected,
        "{}: expected status {}, got {}",
        context,
        expected,
        response.status()
    );
}

/// Assert that a response has the expected content-type (prefix match)
pub fn assert_content_type(response: &reqwest::Response, expected_prefix: &str, context: &str) {
    let content_type = response
        .headers()
        .get("content-type")
        .expect(&format!("{}: missing content-type header", context))
        .to_str()
        .expect(&format!("{}: invalid content-type header", context));

    assert!(
        content_type.starts_with(expected_prefix),
        "{}: expected content-type starting with '{}', got '{}'",
        context,
        expected_prefix,
        content_type
    );
}

/// Assert that a header exists and has the expected value
pub fn assert_header(response: &reqwest::Response, header: &str, expected: &str, context: &str) {
    let value = response
        .headers()
        .get(header)
        .expect(&format!("{}: missing {} header", context, header))
        .to_str()
        .expect(&format!("{}: invalid {} header value", context, header));

    assert_eq!(
        value, expected,
        "{}: expected {} header to be '{}', got '{}'",
        context, header, expected, value
    );
}

/// Assert that HTML body contains expected text
pub fn assert_body_contains(body: &str, needle: &str, context: &str) {
    assert!(
        body.contains(needle),
        "{}: expected body to contain '{}', but it didn't.\nBody preview: {}...",
        context,
        needle,
        &body.chars().take(500).collect::<String>()
    );
}

/// Assert that a response has a JavaScript content-type.
/// Accepts both text/javascript and application/javascript (both are valid).
pub fn assert_javascript_content_type(response: &reqwest::Response, context: &str) {
    let content_type = response
        .headers()
        .get("content-type")
        .expect(&format!("{}: missing content-type header", context))
        .to_str()
        .expect(&format!("{}: invalid content-type header", context));

    assert!(
        content_type.starts_with("text/javascript") || content_type.starts_with("application/javascript"),
        "{}: expected JavaScript content-type, got '{}'",
        context,
        content_type
    );
}
