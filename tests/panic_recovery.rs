//! Panic Recovery Tests
//!
//! Tests that the server survives handler panics without crashing entirely.
//! This is Rust-specific - the PHP implementation doesn't have a panic-test endpoint.

mod common;

use common::{assert_success, client, url};

// =============================================================================
// Panic Recovery
// =============================================================================

/// Server should survive a panic and continue serving requests
/// The /panic-test.htm endpoint intentionally panics the handler
#[tokio::test]
async fn server_survives_panic() {
    // First request: hit the panic endpoint
    // This should return 500 (or possibly connection error if panic is severe)
    let panic_result = client().get(url("/panic-test.htm")).send().await;

    // We don't care too much about the exact response - the panic might cause
    // a 500, or might cause a connection reset. What matters is the next request.
    match panic_result {
        Ok(resp) => {
            // If we got a response, it should be 500 (Internal Server Error)
            assert_eq!(
                resp.status().as_u16(),
                500,
                "/panic-test.htm should return 500, got {}",
                resp.status()
            );
        }
        Err(e) => {
            // Connection error is also acceptable - the handler crashed
            eprintln!("Panic endpoint caused connection error (expected): {}", e);
        }
    }

    // Second request: verify the server is still alive
    let resp = client().get(url("/about.htm")).send().await.unwrap();
    assert_success(&resp, "Server should still respond after panic - /about.htm");
}
