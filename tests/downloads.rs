//! Tests for downloadable files
//!
//! Verifies that important downloadable files exist and are served.

mod common;

use common::{assert_success, client, url};

// =============================================================================
// /downloads/ Directory (served from static/downloads/)
// =============================================================================

#[tokio::test]
async fn winpassgen_exe() {
    let resp = client().get(url("/downloads/WinPassGen.exe")).send().await.unwrap();
    assert_success(&resp, "/downloads/WinPassGen.exe");
}

#[tokio::test]
async fn thesis_pdf() {
    let resp = client().get(url("/downloads/thesis.pdf")).send().await.unwrap();
    assert_success(&resp, "/downloads/thesis.pdf");
}

#[tokio::test]
async fn sockstress_txt() {
    let resp = client().get(url("/downloads/sockstress.txt")).send().await.unwrap();
    assert_success(&resp, "/downloads/sockstress.txt");
}

#[tokio::test]
async fn taylor_asc() {
    let resp = client().get(url("/downloads/taylor.asc")).send().await.unwrap();
    assert_success(&resp, "/downloads/taylor.asc");
}

#[tokio::test]
async fn poc_zip() {
    let resp = client().get(url("/downloads/poc.zip")).send().await.unwrap();
    assert_success(&resp, "/downloads/poc.zip");
}

#[tokio::test]
async fn gocryptfs_audit() {
    let resp = client()
        .get(url("/downloads/audits/gocryptfs-cryptography-design-audit.pdf"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "/downloads/audits/gocryptfs-cryptography-design-audit.pdf");
}

#[tokio::test]
async fn bh2016_whitepaper() {
    let resp = client()
        .get(url("/downloads/bh2016-whitepaper.pdf"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "/downloads/bh2016-whitepaper.pdf");
}

// =============================================================================
// /files2/ Directory (served from STORAGE_PATH/extras/files2)
//
// These files are served from the storage directory configured via STORAGE_PATH.
// Requires STORAGE_PATH to be set and the files to exist.
// =============================================================================

/// Helper to assert that /files2/ responses are viewable in browser (no forced download)
fn assert_no_attachment(resp: &reqwest::Response, context: &str) {
    assert!(
        resp.headers().get("content-disposition").is_none(),
        "{}: /files2/ files should be viewable in browser (no Content-Disposition), got: {:?}",
        context,
        resp.headers().get("content-disposition")
    );
}

#[tokio::test]
async fn pocorgtfo_00() {
    let resp = client()
        .get(url("/files2/poc/pocorgtfo00.pdf"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "/files2/poc/pocorgtfo00.pdf");
    assert_no_attachment(&resp, "/files2/poc/pocorgtfo00.pdf");
}

#[tokio::test]
async fn pocorgtfo_10() {
    // Spot check middle of the series
    let resp = client()
        .get(url("/files2/poc/pocorgtfo10.pdf"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "/files2/poc/pocorgtfo10.pdf");
    assert_no_attachment(&resp, "/files2/poc/pocorgtfo10.pdf");
}

#[tokio::test]
async fn pocorgtfo_21() {
    // Spot check last of the series
    let resp = client()
        .get(url("/files2/poc/pocorgtfo21.pdf"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "/files2/poc/pocorgtfo21.pdf");
    assert_no_attachment(&resp, "/files2/poc/pocorgtfo21.pdf");
}

#[tokio::test]
async fn insfilter_pdf() {
    let resp = client()
        .get(url("/files2/insfilter.pdf"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "/files2/insfilter.pdf");
    assert_no_attachment(&resp, "/files2/insfilter.pdf");
}
