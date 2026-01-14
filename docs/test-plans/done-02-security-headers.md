# Security Headers Tests

**File:** `tests/security_headers.rs`

These tests verify security headers that SHOULD be present. Expected to fail initially on some targets.

## Test Cases

| Test | Description |
|------|-------------|
| `has_x_frame_options` | Response has `X-Frame-Options: SAMEORIGIN` or `DENY` |
| `has_x_content_type_options` | Response has `X-Content-Type-Options: nosniff` |
| `has_strict_transport_security` | Response has `Strict-Transport-Security` with reasonable max-age |
| `has_x_xss_protection` | Response has `X-XSS-Protection: 1; mode=block` (legacy but good to have) |
| `has_referrer_policy` | Response has `Referrer-Policy` header |

Note: `Content-Security-Policy` is tracked separately in defuse-rust issue (requires careful implementation to avoid breaking inline scripts/styles).

## Routing Paths to Test

Core security headers should be tested on each distinct routing path, not just `/`:

| Path | Routing Logic |
|------|---------------|
| `/` | Home page (dynamic) |
| `/about.htm` | Regular page (dynamic) |
| `/main.css` | Static CSS file |
| `/images/1by1.gif` | Static image |
| `/js/checksums.js` | Static JS file |
| `/files/eharmony-hashes.txt` | files/ alias (force download) |
| `/source/breach.php` | source/ alias (force download) |

This ensures security headers are applied consistently regardless of which handler serves the response.

## Implementation Notes

- Create `tests/security_headers.rs` with `mod common;` at top
- Use `common::assert_header_present(response, header, context)` for presence checks
- Use `common::assert_header(response, header, expected, context)` for exact value checks
- For X-Frame-Options, accept either `SAMEORIGIN` or `DENY`
- For HSTS, verify max-age is present and reasonable (e.g., >= 31536000)
- These tests document what headers we want, even if current implementation doesn't have them
