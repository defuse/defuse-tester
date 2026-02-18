//! URL canonicalization tests
//!
//! Tests for URL normalization, extension handling, and alias redirects.
//! All canonicalization should result in 301 redirects to the canonical absolute URL.

mod common;

use common::{assert_redirect, assert_success, client, url};

// =============================================================================
// Extension & Case Normalization
// =============================================================================

/// No extension redirects to .htm
#[tokio::test]
async fn no_extension_redirects_to_htm() {
    let resp = client().get(url("/about")).send().await.unwrap();
    assert_redirect(&resp, &url("/about.htm"), "/about → /about.htm");
}

/// .html redirects to .htm
#[tokio::test]
async fn html_redirects_to_htm() {
    let resp = client().get(url("/about.html")).send().await.unwrap();
    assert_redirect(&resp, &url("/about.htm"), "/about.html → /about.htm");
}

/// Uppercase .HTM redirects to lowercase
#[tokio::test]
async fn uppercase_htm_redirects() {
    let resp = client().get(url("/about.HTM")).send().await.unwrap();
    assert_redirect(&resp, &url("/about.htm"), "/about.HTM → /about.htm");
}

/// Mixed case .HtM redirects to lowercase
#[tokio::test]
async fn mixed_case_htm_redirects() {
    let resp = client().get(url("/about.HtM")).send().await.unwrap();
    assert_redirect(&resp, &url("/about.htm"), "/about.HtM → /about.htm");
}

/// Uppercase .HTML redirects to .htm
#[tokio::test]
async fn uppercase_html_redirects() {
    let resp = client().get(url("/about.HTML")).send().await.unwrap();
    assert_redirect(&resp, &url("/about.htm"), "/about.HTML → /about.htm");
}

/// Uppercase page name redirects to lowercase
#[tokio::test]
async fn uppercase_page_redirects() {
    let resp = client().get(url("/About.htm")).send().await.unwrap();
    assert_redirect(&resp, &url("/about.htm"), "/About.htm → /about.htm");
}

/// Canonical .htm URL returns 200 (no redirect)
#[tokio::test]
async fn canonical_htm_no_redirect() {
    let resp = client().get(url("/about.htm")).send().await.unwrap();
    assert_success(&resp, "/about.htm should return 200");
}

/// Root URL returns 200 (no redirect)
#[tokio::test]
async fn root_no_redirect() {
    let resp = client().get(url("/")).send().await.unwrap();
    assert_success(&resp, "/ should return 200");
}

/// Query string is preserved during redirect
#[tokio::test]
async fn query_string_preserved() {
    let resp = client().get(url("/about?foo=bar")).send().await.unwrap();
    assert_redirect(&resp, &url("/about.htm?foo=bar"), "/about?foo=bar → /about.htm?foo=bar");
}

// =============================================================================
// Index/Home Redirects
// =============================================================================

/// /index redirects to /
#[tokio::test]
async fn index_redirects_to_root() {
    let resp = client().get(url("/index")).send().await.unwrap();
    assert_redirect(&resp, &url("/"), "/index → /");
}

/// /index.htm redirects to /
#[tokio::test]
async fn index_htm_redirects_to_root() {
    let resp = client().get(url("/index.htm")).send().await.unwrap();
    assert_redirect(&resp, &url("/"), "/index.htm → /");
}

/// /index.html redirects to /
#[tokio::test]
async fn index_html_redirects_to_root() {
    let resp = client().get(url("/index.html")).send().await.unwrap();
    assert_redirect(&resp, &url("/"), "/index.html → /");
}

/// /index.php redirects to /
#[tokio::test]
async fn index_php_redirects_to_root() {
    let resp = client().get(url("/index.php")).send().await.unwrap();
    assert_redirect(&resp, &url("/"), "/index.php → /");
}

// =============================================================================
// Aliases (from PHP URLParse.php)
// =============================================================================

/// /key redirects to /contact.htm
#[tokio::test]
async fn alias_key_to_contact() {
    let resp = client().get(url("/key")).send().await.unwrap();
    assert_redirect(&resp, &url("/contact.htm"), "/key → /contact.htm");
}

/// /KEY redirects to /contact.htm (case insensitive)
#[tokio::test]
async fn alias_key_case_insensitive() {
    let resp = client().get(url("/KEY")).send().await.unwrap();
    assert_redirect(&resp, &url("/contact.htm"), "/KEY → /contact.htm");
}

/// /trent redirects to /trustedthirdparty.htm
#[tokio::test]
async fn alias_trent_to_trustedthirdparty() {
    let resp = client().get(url("/trent")).send().await.unwrap();
    assert_redirect(&resp, &url("/trustedthirdparty.htm"), "/trent → /trustedthirdparty.htm");
}

/// /TRENT redirects to /trustedthirdparty.htm (case insensitive)
#[tokio::test]
async fn alias_trent_case_insensitive() {
    let resp = client().get(url("/TRENT")).send().await.unwrap();
    assert_redirect(&resp, &url("/trustedthirdparty.htm"), "/TRENT → /trustedthirdparty.htm");
}

/// /audits/ redirects to /software-security-auditing.htm
#[tokio::test]
async fn alias_audits_to_software_security_auditing() {
    let resp = client().get(url("/audits/")).send().await.unwrap();
    assert_redirect(&resp, &url("/software-security-auditing.htm"), "/audits/ → /software-security-auditing.htm");
}

/// /keyboarddefect redirects to /asuskeyboarddefect.htm
#[tokio::test]
async fn alias_keyboarddefect() {
    let resp = client().get(url("/keyboarddefect")).send().await.unwrap();
    assert_redirect(&resp, &url("/asuskeyboarddefect.htm"), "/keyboarddefect → /asuskeyboarddefect.htm");
}

/// /passwords redirects to /passgen.htm
#[tokio::test]
async fn alias_passwords_to_passgen() {
    let resp = client().get(url("/passwords")).send().await.unwrap();
    assert_redirect(&resp, &url("/passgen.htm"), "/passwords → /passgen.htm");
}

/// /password redirects to /passgen.htm
#[tokio::test]
async fn alias_password_to_passgen() {
    let resp = client().get(url("/password")).send().await.unwrap();
    assert_redirect(&resp, &url("/passgen.htm"), "/password → /passgen.htm");
}

/// /pass redirects to /passgen.htm
#[tokio::test]
async fn alias_pass_to_passgen() {
    let resp = client().get(url("/pass")).send().await.unwrap();
    assert_redirect(&resp, &url("/passgen.htm"), "/pass → /passgen.htm");
}

/// /bh2016 redirects to /side-channel-attacks-on-everyday-applications.htm
#[tokio::test]
async fn alias_bh2016_to_side_channel() {
    let resp = client().get(url("/bh2016")).send().await.unwrap();
    assert_redirect(&resp, &url("/side-channel-attacks-on-everyday-applications.htm"), "/bh2016 → /side-channel-attacks-on-everyday-applications.htm");
}

/// /BH2016 redirects to /side-channel-attacks-on-everyday-applications.htm
#[tokio::test]
async fn alias_bh2016_preserved_case() {
    let resp = client().get(url("/BH2016")).send().await.unwrap();
    assert_redirect(&resp, &url("/side-channel-attacks-on-everyday-applications.htm"), "/BH2016 → /side-channel-attacks-on-everyday-applications.htm");
}

/// /pphos redirects to /password-policy-hall-of-shame.htm
#[tokio::test]
async fn alias_pphos_to_hall_of_shame() {
    let resp = client().get(url("/pphos")).send().await.unwrap();
    assert_redirect(&resp, &url("/password-policy-hall-of-shame.htm"), "/pphos → /password-policy-hall-of-shame.htm");
}
