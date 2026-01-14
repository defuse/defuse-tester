# Missing Tests Checklist

Tests for Apache/PHP behaviors not yet covered by our test suite.

## Special PHP Endpoints - `special_endpoints.rs`

- [x] `/s.php` - "Shout" page
  - [x] `?e=text` redirects to `?s=base64(text)` with 302
  - [x] `?s=base64text` displays decoded text in 300pt font
  - [x] No params shows form with input field

- [x] `/ip.php` - Raw IP address
  - [x] Returns plain text (defuse.ca returns text/html but body is just IP)
  - [x] Returns client IP address with newline

- [x] `/ip-insecure.php` - IP address HTML page
  - [x] Returns HTML page
  - [x] Shows "HTTP IP:" with IP address
  - [x] Uses 30pt font styling

## Source Directory - Download Instead of Execute

All files in `/source/` must be downloaded, not executed:

- [x] `/source/breach.php` - Downloads with `Content-Disposition: attachment` - `static_files.rs`

## Directory Access Restrictions

These directories should return 403 Forbidden:

- [ ] `/pages/` returns 403
- [ ] `/libs/` returns 403
- [ ] `/pages/someFile.php` returns 403
- [ ] `/libs/someFile.php` returns 403

## Redirects

- [ ] Blog slug redirect (301)
  - `/blog/some-post-name` → `/blog/some-post-name.html`

Production-only redirects (can't test locally):
- [x] HTTP to HTTPS redirect (301) - `security_headers.rs`
- [x] www to non-www redirect (301) - `domain_redirects.rs`
- [x] bin.defuse.ca redirect to defuse.ca/b/ - `domain_redirects.rs`

## Content-Disposition Headers

Need to verify `Content-Disposition: attachment` on aliased directories:

- [ ] `/files/*` forces download
- [ ] `/mirrors/*` forces download
- [ ] `/upload/*` forces download

Note: We have existence tests for files in these directories, but need to verify the Content-Disposition header.

## Cache Headers

- [ ] Images (.gif, .jpeg, .png) have `Expires` header set to 7 days
- [ ] HTML pages have `Expires` header set to ~1 second

## Security Headers

- [x] HSTS header present on HTTPS responses - `security_headers.rs`
  - `Strict-Transport-Security: max-age=31536000; includeSubDomains; preload`
  - Note: Test skips on localhost/HTTP

- [x] X-Frame-Options header present - `security_headers.rs`
  - `X-Frame-Options: SAMEORIGIN`

## Verification Files

- [ ] `/googlec56659c80ebb2d30.html` exists (Google site verification)
- [ ] `/have-i-been-pwned-verification.txt` exists (HIBP verification)
- [ ] `/b06c1611fa2dac4cc177a7a63c70902c453cdb92.html` exists (unknown verification)
