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
| `has_content_security_policy` | Response has `Content-Security-Policy` header |
| `has_referrer_policy` | Response has `Referrer-Policy` header |

## Implementation Notes

- Create `tests/security_headers.rs` with `mod common;` at top
- Test against root `/` page
- Use `common::assert_header_present(response, header, context)` for presence checks
- Use `common::assert_header(response, header, expected, context)` for exact value checks
- For X-Frame-Options, accept either `SAMEORIGIN` or `DENY`
- For HSTS, verify max-age is present and reasonable (e.g., >= 31536000)
- These tests document what headers we want, even if current implementation doesn't have them
