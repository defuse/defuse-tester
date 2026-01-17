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
        body.contains("Password Hashing") || body.contains("password hashing"),
        "Page should contain 'Password Hashing'"
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
        body.contains("Side") || body.contains("side"),
        "Page should contain 'Side'"
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
        body.contains("BREACH") || body.contains("breach"),
        "Page should contain 'BREACH'"
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
        body.contains("Triangle") || body.contains("triangle") || body.contains("secure code"),
        "Page should contain 'Triangle' or 'secure code'"
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
        body.contains("encryption") || body.contains("Encryption") || body.contains("PHP"),
        "Page should contain 'encryption' or 'PHP'"
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
        body.contains("password") || body.contains("Password") || body.contains("random"),
        "Page should contain 'password' or 'random'"
    );
}

// =============================================================================
// Misc Pages
// =============================================================================

#[tokio::test]
async fn misc_writing_tips() {
    let resp = client()
        .get(url("/writing-tips.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "writing-tips");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Writing") || body.contains("writing") || body.contains("Tips"),
        "Page should contain 'Writing' or 'Tips'"
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
        body.contains("Reading") || body.contains("reading") || body.contains("Books") || body.contains("books"),
        "Page should contain 'Reading' or 'Books'"
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
        body.contains("About") || body.contains("about") || body.contains("Taylor"),
        "Page should contain 'About' or author name"
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
        body.contains("Contact") || body.contains("contact") || body.contains("email"),
        "Page should contain 'Contact' or 'email'"
    );
}

#[tokio::test]
async fn page_transparency() {
    let resp = client()
        .get(url("/transparency.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "transparency");
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Transparency") || body.contains("transparency") || body.contains("warrant"),
        "Page should contain 'Transparency' or 'warrant'"
    );
}
