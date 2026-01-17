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

    // Check for the vimhighlight div or pre
    assert!(
        body.contains("vimhighlight") || body.contains("vim-highlight") || body.contains("highlight"),
        "VIM highlighting page should have a highlighted code section"
    );
}

/// VIM highlighting contains PHP function keyword
#[tokio::test]
async fn vim_highlight_has_function_keyword() {
    let resp = client()
        .get(url("/syntax-highlighting-in-php-with-vim.htm"))
        .send()
        .await
        .unwrap();

    assert_success(&resp, "vim_highlight_has_function_keyword");
    let body = resp.text().await.unwrap();

    // The page shows PHP code with functions
    assert!(
        body.contains("function"),
        "VIM highlighting page should contain 'function' keyword in code samples"
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

    // The page demonstrates VimHighlight class with these methods
    let has_method = body.contains("vimhl_convert")
        || body.contains("vimhl_exec")
        || body.contains("VimHighlight");

    assert!(
        has_method,
        "VIM highlighting page should contain method names like 'vimhl_convert', 'vimhl_exec', or 'VimHighlight'"
    );
}
