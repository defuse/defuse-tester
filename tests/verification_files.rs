//! Verification file tests
//!
//! Tests for site verification files used by external services.

mod common;

use common::{client, url};

/// Google site verification file exists
#[tokio::test]
async fn google_verification_exists() {
    let resp = client()
        .get(url("/googlec56659c80ebb2d30.html"))
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_success(),
        "/googlec56659c80ebb2d30.html should return 200, got: {}",
        resp.status()
    );

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("google-site-verification"),
        "Google verification file should contain verification string"
    );
}

/// Have I Been Pwned verification file exists
#[tokio::test]
async fn hibp_verification_exists() {
    let resp = client()
        .get(url("/have-i-been-pwned-verification.txt"))
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_success(),
        "/have-i-been-pwned-verification.txt should return 200, got: {}",
        resp.status()
    );

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("186adba4bd8057689a8d03939479804e"),
        "HIBP verification file should contain verification token"
    );
}

/// Unknown verification file (possibly Keybase or similar)
#[tokio::test]
async fn unknown_verification_exists() {
    let resp = client()
        .get(url("/b06c1611fa2dac4cc177a7a63c70902c453cdb92.html"))
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_success(),
        "/b06c1611fa2dac4cc177a7a63c70902c453cdb92.html should return 200, got: {}",
        resp.status()
    );

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("8487a4236cd8d920fb78072e2cd94d5a2819d18d"),
        "Verification file should contain verification token"
    );
}
