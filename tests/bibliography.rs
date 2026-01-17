//! Bibliography/citation system tests
//!
//! Tests for the Bibliography PHP class that provides academic citations.

mod common;

use common::{client, url, assert_success};

/// Bibliography page loads successfully
#[tokio::test]
async fn bibliography_page_loads() {
    let resp = client()
        .get(url("/flush-reload-side-channel.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "bibliography_page_loads");
}

/// Bibliography page has inline citations like [1]
#[tokio::test]
async fn bibliography_has_inline_citation() {
    let resp = client()
        .get(url("/flush-reload-side-channel.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "bibliography_has_inline_citation");
    let body = resp.text().await.unwrap();

    // Citations appear as [1], [2], etc.
    assert!(
        body.contains("[1]"),
        "Bibliography page should have inline citation [1]"
    );
}

/// Bibliography page has References section
#[tokio::test]
async fn bibliography_has_references_section() {
    let resp = client()
        .get(url("/flush-reload-side-channel.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "bibliography_has_references_section");
    let body = resp.text().await.unwrap();

    // Should have a References heading
    assert!(
        body.contains("References") || body.contains("REFERENCES"),
        "Bibliography page should have a References section"
    );
}

/// Bibliography citations link to references
#[tokio::test]
async fn bibliography_citation_links_to_reference() {
    let resp = client()
        .get(url("/flush-reload-side-channel.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "bibliography_citation_links_to_reference");
    let body = resp.text().await.unwrap();

    // Citations should be links to anchors like #cite_1
    let has_cite_link = body.contains("href=\"#cite_1\"")
        || body.contains("href=\"#cite1\"")
        || body.contains("href='#cite_1'")
        || body.contains("href='#cite1'");

    assert!(
        has_cite_link,
        "Bibliography citations should link to references with href=\"#cite_1\" or similar"
    );
}

/// Bibliography reference entries contain author names
#[tokio::test]
async fn bibliography_reference_has_author() {
    let resp = client()
        .get(url("/flush-reload-side-channel.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "bibliography_reference_has_author");
    let body = resp.text().await.unwrap();

    // The flush-reload paper is by Yarom and Falkner
    assert!(
        body.contains("Yarom"),
        "Bibliography should contain author name 'Yarom' from the FLUSH+RELOAD paper"
    );
}
