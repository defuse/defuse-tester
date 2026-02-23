//! Static page smoke tests
//!
//! These tests verify that static content pages load successfully.
//! They don't check content in detail - just that the page returns 200 OK
//! and contains expected elements like navigation and the page title.

mod common;

use common::{client, url, assert_success};

// =============================================================================
// Research Pages
// =============================================================================

#[tokio::test]
async fn research_flush_reload_side_channel() {
    let resp = client()
        .get(url("/flush-reload-side-channel.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "flush-reload-side-channel");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("FLUSH+RELOAD") || body.contains("Flush+Reload") || body.contains("flush+reload"),
        "Page should contain 'FLUSH+RELOAD'"
    );
}

#[tokio::test]
async fn research_password_hashing_after_phc() {
    let resp = client()
        .get(url("/password-hashing-after-phc.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "password-hashing-after-phc");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("cache_put"),
        "Page should contain 'cache_put'"
    );
}

#[tokio::test]
async fn research_side_channel_attacks() {
    let resp = client()
        .get(url("/side-channel-attacks-on-everyday-applications.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "side-channel-attacks-on-everyday-applications");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Black Hat USA 2016"),
        "Page should contain 'Black Hat USA 2016'"
    );
}

#[tokio::test]
async fn research_breach_mitigation() {
    let resp = client()
        .get(url("/mitigating-breach-tls-attack-in-php.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "mitigating-breach-tls-attack-in-php");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Masking Secrets"),
        "Page should contain 'Masking Secrets'"
    );
}

#[tokio::test]
async fn research_triangle_secure_code() {
    let resp = client()
        .get(url("/triangle-of-secure-code-delivery.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "triangle-of-secure-code-delivery");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Reproducible Builds"),
        "Page should contain 'Reproducible Builds'"
    );
}

// =============================================================================
// Audit Pages
// =============================================================================

#[tokio::test]
async fn audit_encfs() {
    let resp = client()
        .get(url("/audits/encfs.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "audits/encfs");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("EncFS") || body.contains("encfs"),
        "Page should contain 'EncFS'"
    );
}

#[tokio::test]
async fn audit_ecryptfs() {
    let resp = client()
        .get(url("/audits/ecryptfs.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "audits/ecryptfs");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("eCryptfs") || body.contains("ecryptfs"),
        "Page should contain 'eCryptfs'"
    );
}

#[tokio::test]
async fn audit_zerobin() {
    let resp = client()
        .get(url("/audits/zerobin.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "audits/zerobin");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("ZeroBin") || body.contains("zerobin") || body.contains("Zerobin"),
        "Page should contain 'ZeroBin'"
    );
}

#[tokio::test]
async fn audit_gocryptfs() {
    let resp = client()
        .get(url("/audits/gocryptfs.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "audits/gocryptfs");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("gocryptfs") || body.contains("Gocryptfs"),
        "Page should contain 'gocryptfs'"
    );
}

// =============================================================================
// Project Pages
// =============================================================================

#[tokio::test]
async fn project_secure_php_encryption() {
    let resp = client()
        .get(url("/secure-php-encryption.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "secure-php-encryption");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("MCRYPT_RIJNDAEL_256"),
        "Page should contain 'MCRYPT_RIJNDAEL_256'"
    );
}

#[tokio::test]
async fn project_php_pbkdf2() {
    let resp = client()
        .get(url("/php-pbkdf2.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "php-pbkdf2");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("PBKDF2") || body.contains("pbkdf2"),
        "Page should contain 'PBKDF2'"
    );
}

#[tokio::test]
async fn project_generating_random_passwords() {
    let resp = client()
        .get(url("/generating-random-passwords.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "generating-random-passwords");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("PasswordGenerator"),
        "Page should contain 'PasswordGenerator'"
    );
}

// =============================================================================
// Misc Pages
// =============================================================================

#[tokio::test]
async fn misc_fractal_zoom() {
    let resp = client()
        .get(url("/fractal-zoom.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "fractal-zoom");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("fractal zoom draws you in"),
        "Page should contain 'fractal zoom draws you in'"
    );
}

#[tokio::test]
async fn misc_reading_list() {
    let resp = client()
        .get(url("/reading-list.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "reading-list");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("bibliography"),
        "Page should contain 'bibliography'"
    );
}

// =============================================================================
// Core Pages
// =============================================================================

#[tokio::test]
async fn page_about() {
    let resp = client()
        .get(url("/about.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "about");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("About Taylor Hornby"),
        "Page should contain 'About Taylor Hornby'"
    );
}

#[tokio::test]
async fn page_contact() {
    let resp = client()
        .get(url("/contact.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "contact");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("PGP Keys"),
        "Page should contain 'PGP Keys'"
    );
}

#[tokio::test]
async fn page_honestyware() {
    let resp = client()
        .get(url("/honestyware.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "honestyware");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Honestyware is a philosophy"),
        "Page should contain 'Honestyware is a philosophy'"
    );
}
