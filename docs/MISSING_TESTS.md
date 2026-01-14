# Missing Tests Checklist

Tests for Apache/PHP behaviors not yet covered by our test suite.

## Special PHP Endpoints

- [ ] `/s.php` - "Shout" page
  - [ ] `?e=text` redirects to `?s=base64(text)` with 302
  - [ ] `?s=base64text` displays decoded text in 300pt font
  - [ ] No params shows form with input field

- [ ] `/rdrand.php` - Random bytes endpoint
  - [ ] Returns `Content-Type: application/octet-stream`
  - [ ] Returns `Content-Disposition: attachment; filename="rand.bin"`
  - [ ] Returns binary data

- [ ] `/ip.php` - Raw IP address
  - [ ] Returns plain text (not HTML)
  - [ ] Returns client IP address with newline

- [ ] `/ip-insecure.php` - IP address HTML page
  - [ ] Returns HTML page
  - [ ] Shows "HTTP IP:" with IP address
  - [ ] Uses 30pt font styling

## Source Directory - Download Instead of Execute

All files in `/source/` must be downloaded, not executed:

- [ ] `/source/mutex.php` - Downloads with `Content-Disposition: attachment`
- [ ] `/source/breach.php` - Downloads with `Content-Disposition: attachment`
- [ ] `/source/pbkdf2.php` - Downloads with `Content-Disposition: attachment`
- [ ] `/source/pdfcleaner.php` - Downloads with `Content-Disposition: attachment`

## Directory Access Restrictions

These directories should return 403 Forbidden:

- [ ] `/pages/` returns 403
- [ ] `/libs/` returns 403
- [ ] `/pages/someFile.php` returns 403
- [ ] `/libs/someFile.php` returns 403

## Redirects

- [ ] HTTP to HTTPS redirect (301)
  - Note: Only testable against production, not localhost

- [ ] www to non-www redirect (301)
  - `www.defuse.ca/*` → `defuse.ca/*`
  - Note: Only testable against production

- [ ] Blog slug redirect (301)
  - `/blog/some-post-name` → `/blog/some-post-name.html`

- [ ] bin.defuse.ca redirect
  - `bin.defuse.ca/{key}` → `defuse.ca/b/{key}`
  - Note: Only testable against production

## Pastebin Routes

- [ ] `/b/` directory routing works
  - Note: Full pastebin testing is separate; this is just route existence

## Aliased Directories with Content-Disposition

These directories force downloads via `Content-Disposition: attachment`:

- [ ] `/files/*` forces download (Content-Disposition: attachment)
- [ ] `/files2/*` forces download (Content-Disposition: attachment)
- [ ] `/mirrors/*` forces download (Content-Disposition: attachment)
- [ ] `/upload/*` forces download (Content-Disposition: attachment)

Note: We already have tests for files in these directories; need to verify Content-Disposition header.

## Cache Headers

- [ ] Images (.gif, .jpeg, .png) have `Expires` header set to 7 days
- [ ] HTML pages have `Expires` header set to ~1 second
- [ ] Pastebin view has `Cache-Control: no-cache, must-revalidate`

## Security Headers

- [ ] HSTS header present on HTTPS responses
  - `Strict-Transport-Security: max-age=31536000; includeSubDomains; preload`
  - Note: Should NOT be present on localhost/HTTP

- [ ] X-Frame-Options header present
  - `X-Frame-Options: SAMEORIGIN`

## Verification Files

- [ ] `/googlec56659c80ebb2d30.html` exists (Google site verification)
- [ ] `/have-i-been-pwned-verification.txt` exists (HIBP verification)
- [ ] `/b06c1611fa2dac4cc177a7a63c70902c453cdb92.html` exists (unknown verification)

## Blog

- [ ] `/blog/` index page exists and returns 200
- [ ] `/blog/some-post.html` returns 200 (static HTML)
- [ ] `/blog/some-post` (no .html) redirects to `/blog/some-post.html` with 301

Note: Blog index and post tests already exist (ignored). Blog redirect test needed.
