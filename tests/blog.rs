//! Blog tests
//!
//! Tests for the /blog/ section:
//! - Blog index page
//! - Individual blog posts
//! - Slug redirects (post-name → post-name.html)

mod common;

use common::{client, url};

// =============================================================================
// Blog Index
// =============================================================================

#[tokio::test]
#[ignore = "blog not yet implemented in Rust"]
async fn blog_index_exists() {
    let resp = client().get(url("/blog/")).send().await.unwrap();
    assert!(resp.status().is_success(), "/blog/ should return 200");
}

// =============================================================================
// Blog Posts
// =============================================================================

#[tokio::test]
#[ignore = "blog not yet implemented in Rust"]
async fn blog_post_password_hashing() {
    let resp = client()
        .get(url("/blog/the-economics-of-password-cracking.html"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "blog post should return 200");

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("password"),
        "blog post should contain 'password'"
    );
}

#[tokio::test]
#[ignore = "blog not yet implemented in Rust"]
async fn blog_post_crackstation_linkedin() {
    let resp = client()
        .get(url("/blog/cracking-linkedin-hashes-with-crackstation.html"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "blog post should return 200");

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("LinkedIn"),
        "blog post should contain 'LinkedIn'"
    );
}

#[tokio::test]
#[ignore = "blog not yet implemented in Rust"]
async fn blog_post_full_disclosure() {
    let resp = client()
        .get(url("/blog/full-disclosure-vs-coordinated-disclosure.html"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "blog post should return 200");

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("disclosure"),
        "blog post should contain 'disclosure'"
    );
}

// =============================================================================
// Blog Slug Redirects
// =============================================================================

/// Blog post without .html extension should redirect to .html version
#[tokio::test]
#[ignore = "blog not yet implemented in Rust"]
async fn blog_slug_redirects_to_html() {
    let resp = client()
        .get(url("/blog/the-economics-of-password-cracking"))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status().as_u16(),
        301,
        "/blog/slug should redirect with 301, got: {}",
        resp.status()
    );

    let location = resp
        .headers()
        .get("location")
        .expect("Redirect should have Location header")
        .to_str()
        .unwrap();

    assert!(
        location.ends_with("/blog/the-economics-of-password-cracking.html"),
        "/blog/slug should redirect to /blog/slug.html, got: {}",
        location
    );
}
