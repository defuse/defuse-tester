//! Tests for static file serving (CSS, JS, favicon, robots.txt)
//!
//! These tests verify files exist and are served. Content-type checks are separate.

mod common;

use common::{client, url};

// =============================================================================
// CSS Files
// =============================================================================

#[tokio::test]
async fn css_main() {
    let resp = client().get(url("/main.css")).send().await.unwrap();
    assert!(resp.status().is_success(), "/main.css should exist");
}

#[tokio::test]
async fn css_mainmenu() {
    let resp = client().get(url("/mainmenu.css")).send().await.unwrap();
    assert!(resp.status().is_success(), "/mainmenu.css should exist");
}

#[tokio::test]
async fn css_vimhl() {
    let resp = client().get(url("/vimhl.css")).send().await.unwrap();
    assert!(resp.status().is_success(), "/vimhl.css should exist");
}

#[tokio::test]
async fn css_print() {
    let resp = client().get(url("/print.css")).send().await.unwrap();
    assert!(resp.status().is_success(), "/print.css should exist");
}

// =============================================================================
// JavaScript Files
// =============================================================================

#[tokio::test]
async fn js_defuse() {
    let resp = client().get(url("/js/defuse.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/defuse.js should exist");
}

#[tokio::test]
async fn js_upvote() {
    let resp = client().get(url("/js/upvote.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/upvote.js should exist");
}

#[tokio::test]
async fn js_aes() {
    let resp = client().get(url("/js/aes.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/aes.js should exist");
}

#[tokio::test]
async fn js_crypto_helpers() {
    let resp = client().get(url("/js/cryptoHelpers.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/cryptoHelpers.js should exist");
}

#[tokio::test]
async fn js_csprng() {
    let resp = client().get(url("/js/csprng.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/csprng.js should exist");
}

#[tokio::test]
async fn js_csprng_extensions() {
    let resp = client().get(url("/js/csprng-extensions.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/csprng-extensions.js should exist");
}

#[tokio::test]
async fn js_encrypt() {
    let resp = client().get(url("/js/encrypt.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/encrypt.js should exist");
}

#[tokio::test]
async fn js_hash() {
    let resp = client().get(url("/js/jsHash.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/jsHash.js should exist");
}

#[tokio::test]
async fn js_leet() {
    let resp = client().get(url("/js/leet.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/leet.js should exist");
}

#[tokio::test]
async fn js_modernizr() {
    let resp = client().get(url("/js/modernizr.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/modernizr.js should exist");
}

#[tokio::test]
async fn js_passwordblocks() {
    let resp = client().get(url("/js/passwordblocks.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/passwordblocks.js should exist");
}

#[tokio::test]
async fn js_sjcl() {
    let resp = client().get(url("/js/sjcl.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/sjcl.js should exist");
}

#[tokio::test]
async fn js_wordlist() {
    let resp = client().get(url("/js/wordlist.js")).send().await.unwrap();
    assert!(resp.status().is_success(), "/js/wordlist.js should exist");
}

// =============================================================================
// Other Static Files
// =============================================================================

#[tokio::test]
async fn favicon() {
    let resp = client().get(url("/favicon.ico")).send().await.unwrap();
    assert!(resp.status().is_success(), "/favicon.ico should exist");
}

#[tokio::test]
async fn robots_txt() {
    let resp = client().get(url("/robots.txt")).send().await.unwrap();
    assert!(resp.status().is_success(), "/robots.txt should exist");

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("User-agent"),
        "/robots.txt should contain User-agent directive"
    );
}

// =============================================================================
// Easter Eggs
// =============================================================================

#[tokio::test]
async fn longcat_easter_egg() {
    let resp = client().get(url("/longcat.html")).send().await.unwrap();
    assert!(resp.status().is_success(), "/longcat.html should exist");
}

// =============================================================================
// Sample Images (spot check)
// =============================================================================

#[tokio::test]
async fn image_bg() {
    let resp = client().get(url("/images/bg.png")).send().await.unwrap();
    assert!(resp.status().is_success(), "/images/bg.png should exist");
}

#[tokio::test]
async fn image_download() {
    let resp = client().get(url("/images/download.png")).send().await.unwrap();
    assert!(resp.status().is_success(), "/images/download.png should exist");
}

// =============================================================================
// Content-Type Tests (may fail on legacy PHP which uses application/javascript)
// =============================================================================

#[tokio::test]
async fn content_type_css() {
    let resp = client().get(url("/main.css")).send().await.unwrap();
    let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(
        content_type.starts_with("text/css"),
        "CSS should have text/css content-type, got: {}",
        content_type
    );
}

// =============================================================================
// Pages not yet implemented in Rust
// =============================================================================

#[tokio::test]
async fn hypothetico_page_exists() {
    let resp = client().get(url("/hypothetico/")).send().await.unwrap();
    assert!(resp.status().is_success(), "/hypothetico/ should return 200");

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("Hypothetico"),
        "/hypothetico/ should contain 'Hypothetico'"
    );
}

#[tokio::test]
async fn content_type_js() {
    let resp = client().get(url("/js/defuse.js")).send().await.unwrap();
    let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
    // Both text/javascript (RFC 9239) and application/javascript are valid
    assert!(
        content_type.starts_with("text/javascript") || content_type.starts_with("application/javascript"),
        "JS should have javascript content-type, got: {}",
        content_type
    );
}

// =============================================================================
// Source Files (not yet implemented in Rust)
// =============================================================================

#[tokio::test]
async fn source_blind_birthday_rb() {
    let resp = client()
        .get(url("/source/blind-birthday.rb"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "/source/blind-birthday.rb should return 200");

    let content_disp = resp
        .headers()
        .get("content-disposition")
        .expect("/source/blind-birthday.rb should have Content-Disposition header")
        .to_str()
        .unwrap();
    assert!(
        content_disp.contains("attachment"),
        "/source/ non-PHP files should also force download, got: {}",
        content_disp
    );

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("def") || body.contains("HMAC"),
        "/source/blind-birthday.rb should contain Ruby code"
    );
}

/// PHP files in /source/ must be downloaded (Content-Disposition: attachment), not executed
#[tokio::test]
async fn source_php_downloads_not_executes() {
    let resp = client()
        .get(url("/source/breach.php"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "/source/breach.php should return 200");

    // Must have Content-Disposition: attachment to force download
    let content_disp = resp
        .headers()
        .get("content-disposition")
        .expect("/source/breach.php should have Content-Disposition header")
        .to_str()
        .unwrap();

    assert!(
        content_disp.contains("attachment"),
        "/source/breach.php should force download with Content-Disposition: attachment, got: {}",
        content_disp
    );
}

// =============================================================================
// Upload Files (served from STORAGE_PATH/extras/upload, force download)
// =============================================================================

#[tokio::test]
async fn upload_vim_vimrc() {
    let resp = client()
        .get(url("/upload/vim/vimrc"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "/upload/vim/vimrc should return 200");

    // Must force download
    let content_disp = resp
        .headers()
        .get("content-disposition")
        .expect("/upload/ files should have Content-Disposition header")
        .to_str()
        .unwrap();
    assert!(
        content_disp.contains("attachment"),
        "/upload/ files should force download, got: {}",
        content_disp
    );

    // vimrc has no extension — Content-Type should be application/octet-stream or text/plain
    let content_type = resp.headers().get("content-type")
        .expect("/upload/vim/vimrc should have Content-Type header")
        .to_str().unwrap();
    assert!(
        content_type.starts_with("application/octet-stream") || content_type.starts_with("text/plain"),
        "/upload/vim/vimrc Content-Type should be octet-stream or text/plain, got: {}",
        content_type
    );
}

#[tokio::test]
async fn upload_vim_zip() {
    let resp = client()
        .get(url("/upload/vim/vim.zip"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "/upload/vim/vim.zip should return 200");

    // Must force download
    let content_disp = resp
        .headers()
        .get("content-disposition")
        .expect("/upload/ files should have Content-Disposition header")
        .to_str()
        .unwrap();
    assert!(
        content_disp.contains("attachment"),
        "/upload/ files should force download, got: {}",
        content_disp
    );

    // .zip should have application/zip Content-Type
    let content_type = resp.headers().get("content-type")
        .expect("/upload/vim/vim.zip should have Content-Type header")
        .to_str().unwrap();
    assert!(
        content_type.contains("zip"),
        "/upload/vim/vim.zip Content-Type should contain 'zip', got: {}",
        content_type
    );
}

// =============================================================================
// Files Directory (served from STORAGE_PATH/extras/files, force download)
// =============================================================================

#[tokio::test]
async fn files_eharmony_hashes() {
    // Use HEAD request since this is a large file (~50MB)
    let resp = client()
        .head(url("/files/eharmony-hashes.txt"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "/files/eharmony-hashes.txt should return 200");

    // Must force download
    let content_disp = resp
        .headers()
        .get("content-disposition")
        .expect("/files/ should have Content-Disposition header")
        .to_str()
        .unwrap();
    assert!(
        content_disp.contains("attachment"),
        "/files/ should force download, got: {}",
        content_disp
    );

    // .txt should have text/plain Content-Type
    let content_type = resp.headers().get("content-type")
        .expect("/files/eharmony-hashes.txt should have Content-Type header")
        .to_str().unwrap();
    assert!(
        content_type.starts_with("text/plain"),
        "/files/eharmony-hashes.txt Content-Type should be text/plain, got: {}",
        content_type
    );
}

// =============================================================================
// Mirror Files (served from STORAGE_PATH/extras/mirrors, force download)
// =============================================================================

#[tokio::test]
async fn mirrors_zcash_blockchain() {
    // Use HEAD request since this is a large file
    let resp = client()
        .head(url("/mirrors/zcash-mainnet-2019-10.tar.bz2"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "/mirrors/zcash-mainnet-2019-10.tar.bz2 should return 200");

    // Must force download
    let content_disp = resp
        .headers()
        .get("content-disposition")
        .expect("/mirrors/ files should have Content-Disposition header")
        .to_str()
        .unwrap();
    assert!(
        content_disp.contains("attachment"),
        "/mirrors/ files should force download, got: {}",
        content_disp
    );

    // .tar.bz2 should have a compressed archive Content-Type
    let content_type = resp.headers().get("content-type")
        .expect("/mirrors/ file should have Content-Type header")
        .to_str().unwrap();
    assert!(
        content_type.contains("bzip2") || content_type.contains("octet-stream") || content_type.contains("x-tar"),
        "/mirrors/ .tar.bz2 Content-Type should indicate archive, got: {}",
        content_type
    );
}

// =============================================================================
// Storage 404 with page slug collision (BUG-04-07)
// =============================================================================
// nest_service strips the prefix before passing to ServeDir's not_found_service.
// A request to /files/about arrives at the handler as /about, which collides
// with the registered "about" page slug. This must return 404, not serve the
// about page or panic.

#[tokio::test]
async fn storage_404_does_not_collide_with_page_slug() {
    for path in ["/files/about", "/files/about.htm", "/files2/about", "/mirrors/about", "/upload/about"] {
        let resp = client().get(url(path)).send().await.unwrap();
        assert_eq!(
            resp.status().as_u16(),
            404,
            "{} should return 404, not serve the registered 'about' page",
            path
        );
    }
}

// The static file fallback (root-level ServeDir) does NOT strip prefixes, so
// a nonexistent static file like /nonexistent-file-xyz correctly passes the
// original path to the registered page handler, which 404s normally.
#[tokio::test]
async fn static_fallback_preserves_path() {
    let resp = client().get(url("/nonexistent-file-xyz")).send().await.unwrap();
    assert_eq!(
        resp.status().as_u16(),
        404,
        "/nonexistent-file-xyz should 404 cleanly through static fallback"
    );
}
