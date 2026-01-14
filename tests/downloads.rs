//! Tests for downloadable files
//!
//! Verifies that important downloadable files exist and are served.
//! Currently ignored - local dev server doesn't have these files.

mod common;

use common::{assert_success, client, url};

// =============================================================================
// /downloads/ Directory (in git repo)
// =============================================================================

#[tokio::test]
#[ignore = "downloads/ not served by local dev server"]
async fn winpassgen_exe() {
    let resp = client().get(url("/downloads/WinPassGen.exe")).send().await.unwrap();
    assert_success(&resp, "/downloads/WinPassGen.exe");
}

#[tokio::test]
#[ignore = "downloads/ not served by local dev server"]
async fn thesis_pdf() {
    let resp = client().get(url("/downloads/thesis.pdf")).send().await.unwrap();
    assert_success(&resp, "/downloads/thesis.pdf");
}

#[tokio::test]
#[ignore = "downloads/ not served by local dev server"]
async fn sockstress_txt() {
    let resp = client().get(url("/downloads/sockstress.txt")).send().await.unwrap();
    assert_success(&resp, "/downloads/sockstress.txt");
}

#[tokio::test]
#[ignore = "downloads/ not served by local dev server"]
async fn taylor_asc() {
    let resp = client().get(url("/downloads/taylor.asc")).send().await.unwrap();
    assert_success(&resp, "/downloads/taylor.asc");
}

#[tokio::test]
#[ignore = "downloads/ not served by local dev server"]
async fn poc_zip() {
    let resp = client().get(url("/downloads/poc.zip")).send().await.unwrap();
    assert_success(&resp, "/downloads/poc.zip");
}

#[tokio::test]
#[ignore = "downloads/ not served by local dev server"]
async fn gocryptfs_audit() {
    let resp = client()
        .get(url("/downloads/audits/gocryptfs-cryptography-design-audit.pdf"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "/downloads/audits/gocryptfs-cryptography-design-audit.pdf");
}

#[tokio::test]
#[ignore = "downloads/ not served by local dev server"]
async fn bh2016_whitepaper() {
    let resp = client()
        .get(url("/downloads/bh2016-whitepaper.pdf"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "/downloads/bh2016-whitepaper.pdf");
}

// =============================================================================
// /files2/ Directory (large files, not in git repo - may fail locally)
//
// These files are hosted on production but not stored in the git repository
// due to their size. Tests will fail when running against a local development
// server that doesn't have these files.
// =============================================================================

#[tokio::test]
#[ignore = "files2/ not in git repo"]
async fn pocorgtfo_00() {
    let resp = client()
        .get(url("/files2/poc/pocorgtfo00.pdf"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "/files2/poc/pocorgtfo00.pdf");
}

#[tokio::test]
#[ignore = "files2/ not in git repo"]
async fn pocorgtfo_10() {
    // Spot check middle of the series
    let resp = client()
        .get(url("/files2/poc/pocorgtfo10.pdf"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "/files2/poc/pocorgtfo10.pdf");
}

#[tokio::test]
#[ignore = "files2/ not in git repo"]
async fn pocorgtfo_21() {
    // Spot check last of the series
    let resp = client()
        .get(url("/files2/poc/pocorgtfo21.pdf"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "/files2/poc/pocorgtfo21.pdf");
}

#[tokio::test]
#[ignore = "files2/ not in git repo"]
async fn insfilter_pdf() {
    let resp = client()
        .get(url("/files2/insfilter.pdf"))
        .send()
        .await
        .unwrap();
    assert_success(&resp, "/files2/insfilter.pdf");
}
