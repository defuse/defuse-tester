//! VIM syntax highlighting tests
//!
//! Tests for the VimHighlight PHP class that provides syntax highlighting.

mod common;

use common::{client, url, assert_success};

/// VIM highlighting page loads successfully
#[tokio::test]
async fn vim_highlight_page_loads() {
    let resp = client()
        .get(url("/syntax-highlighting-in-php-with-vim.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "vim_highlight_page_loads");
}

/// VIM highlighting page has code section with vimhighlight class
#[tokio::test]
async fn vim_highlight_has_code_section() {
    let resp = client()
        .get(url("/syntax-highlighting-in-php-with-vim.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "vim_highlight_has_code_section");
    let body = resp.text().await.unwrap();

    // Check for the vimhighlight CSS class (used by highlight_string output)
    assert!(
        body.contains("vimhighlight"),
        "VIM highlighting page should have a highlighted code section with 'vimhighlight' class"
    );
}

/// VIM highlighting contains syntax-highlighted PHP keywords
#[tokio::test]
async fn vim_highlight_has_highlighted_keywords() {
    let resp = client()
        .get(url("/syntax-highlighting-in-php-with-vim.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "vim_highlight_has_highlighted_keywords");
    let body = resp.text().await.unwrap();

    // PHP keywords like require_once, new, echo should appear inside
    // vim syntax-highlighted <span class="PreProc"> tags
    let re = regex::Regex::new(r#"<span class="PreProc">(require_once|new|echo)</span>"#).unwrap();
    assert!(
        re.is_match(&body),
        "VIM highlighting page should contain PHP keywords inside highlighted <span> tags"
    );
}

/// VIM highlighting contains actual method names from the code
#[tokio::test]
async fn vim_highlight_has_method_name() {
    let resp = client()
        .get(url("/syntax-highlighting-in-php-with-vim.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "vim_highlight_has_method_name");
    let body = resp.text().await.unwrap();

    // The page shows highlighted PHP code using the VimHighlight class
    assert!(
        body.contains("setVimCommand"),
        "VIM highlighting page should contain 'setVimCommand' from the highlighted PHP example"
    );
}
