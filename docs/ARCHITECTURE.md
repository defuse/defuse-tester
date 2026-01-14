# Test Architecture

## Directory Structure

```
tests/
  common/
    mod.rs              # Shared test utilities (client, assertions, helpers)
  pages/
    mod.rs              # (doesn't exist yet - pages.rs serves as entry point)
    home.rs             # Home page tests
    checksums.rs        # Checksums page tests (to be created)
    trent.rs            # TRENT service tests (placeholder)
    pastebin.rs         # Pastebin tests (placeholder)
    ...                 # Other service placeholders
  content_snapshot/     # (to be created)
    snapshots/          # HTML snapshot files
  pages.rs              # Entry point for pages module
  static_files.rs       # Static file existence tests
  downloads.rs          # Download file existence tests
  url_edge_cases.rs     # Trailing slash, double slash, URL-encoded char tests
  http_methods.rs       # HEAD requests, POST to GET-only pages
  content_types.rs      # Content-Type and Content-Disposition header tests
  url_canonicalization.rs  # (to be created) Redirect tests
  security_headers.rs   # (to be created) Security header tests
  not_found.rs          # (to be created) 404 page tests
  panic_recovery.rs     # (to be created) Server recovery tests
  upvote.rs             # (to be created) Upvote system tests
  hit_counter.rs        # (to be created) Hit counter tests
  content_snapshot.rs   # (to be created) Content comparison tests
```

## File Conventions

### Standalone Test Files (`tests/*.rs`)

Use standalone files for distinct feature areas:
- URL canonicalization, security headers, downloads, etc.
- Each file starts with `mod common;` to access shared utilities

### Page Module (`tests/pages/*.rs`)

Use the pages module for page-specific functionality tests:
- Service pages (TRENT, pastebin, passgen, checksums, etc.)
- Tests that need HTML parsing of specific page structures
- Entry point is `tests/pages.rs`

### Common Module (`tests/common/mod.rs`)

Shared utilities available to all test files:

```rust
// Client & URLs
common::client()           // HTTP client (no auto-redirect)
common::base_url()         // DEFUSE_URL env var
common::url(path)          // Build full URL from path

// Assertions
common::assert_success(response, context)
common::assert_status(response, code, context)
common::assert_redirect(response, location, context)  // 301 + Location
common::assert_header(response, header, value, context)
common::assert_header_present(response, header, context)
common::assert_content_type(response, prefix, context)
common::assert_javascript_content_type(response, context)
common::assert_body_contains(body, needle, context)
common::assert_body_does_not_contain(body, needle, context)

// Helpers
common::post_form(path, fields).await  // POST form data
```

## Test Patterns

### Basic GET Test

```rust
mod common;
use common::{client, url, assert_success};

#[tokio::test]
async fn my_test() {
    let resp = client().get(url("/path")).send().await.unwrap();
    assert_success(&resp, "/path");
}
```

### Redirect Test

```rust
mod common;
use common::{client, url, assert_redirect};

#[tokio::test]
async fn redirect_test() {
    let resp = client().get(url("/old")).send().await.unwrap();
    assert_redirect(&resp, "/new.htm", "/old -> /new.htm");
}
```

### Form POST Test

```rust
mod common;
use common::{post_form, assert_success};

#[tokio::test]
async fn form_test() {
    let resp = post_form("/endpoint", &[("key", "value")]).await;
    assert_success(&resp, "POST /endpoint");
}
```

### HTML Parsing Test

```rust
mod common;
use common::{client, url};
use scraper::{Html, Selector};

#[tokio::test]
async fn parsing_test() {
    let resp = client().get(url("/page.htm")).send().await.unwrap();
    let body = resp.text().await.unwrap();
    let doc = Html::parse_document(&body);

    let selector = Selector::parse("div.class").unwrap();
    let element = doc.select(&selector).next().expect("element should exist");
    // ... assertions on element
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `reqwest` | HTTP client (with cookies) |
| `tokio` | Async runtime |
| `scraper` | HTML parsing |
| `quick-xml` | XML parsing (for upvote responses) |
| `serde` / `serde_json` | JSON serialization |
| `anyhow` | Error handling |

## Environment Variables

- `DEFUSE_URL` - Required. Base URL to test against (e.g., `http://localhost:8080` or `https://defuse.ca`)

## Running Tests

**Always use `--no-fail-fast`** to ensure all tests run even if some fail. Without this flag, cargo stops at the first failing test binary and you may miss the test you're looking for.

```bash
# Run all tests
DEFUSE_URL=http://localhost:8080 cargo test --no-fail-fast

# Run specific test file
DEFUSE_URL=http://localhost:8080 cargo test --no-fail-fast --test static_files

# Run specific test
DEFUSE_URL=http://localhost:8080 cargo test --no-fail-fast --test static_files css_main

# Run tests matching a pattern
DEFUSE_URL=http://localhost:8080 cargo test --no-fail-fast redirect
```

## Test Status Conventions

- **Implemented tests that pass**: Normal test functions
- **Implemented tests that fail**: Normal test functions (fail loudly - this is intentional)
- **Unimplemented tests**: Use `#[ignore = "reason"]` attribute

The distinction matters: ignored tests show as "ignored" in output, failing tests show as "failed". We want to clearly see which tests are waiting for implementation vs which are testing functionality that doesn't work yet.
