# URL Canonicalization Tests

**File:** `tests/url_canonicalization.rs`

All tests verify 301 redirect to expected location (or no redirect when already canonical).

## Extension & Case Normalization

| Test | Description |
|------|-------------|
| `no_extension_redirects_to_htm` | `/about` → 301 to `/about.htm` |
| `html_redirects_to_htm` | `/about.html` → 301 to `/about.htm` |
| `uppercase_htm_redirects` | `/about.HTM` → 301 to `/about.htm` |
| `mixed_case_htm_redirects` | `/about.HtM` → 301 to `/about.htm` |
| `uppercase_html_redirects` | `/about.HTML` → 301 to `/about.htm` |
| `uppercase_page_redirects` | `/About.htm` → 301 to `/about.htm` |
| `canonical_htm_no_redirect` | `/about.htm` → 200 (no redirect) |
| `root_no_redirect` | `/` → 200 (no redirect) |
| `query_string_preserved` | `/about?foo=bar` → 301 to `/about.htm?foo=bar` |
| `nonexistent_page_404` | `/thispagedoesnotexist.htm` → 404 (no redirect) |

## Index/Home Redirects

| Test | Description |
|------|-------------|
| `index_redirects_to_root` | `/index` → 301 to `/` |
| `index_htm_redirects_to_root` | `/index.htm` → 301 to `/` |
| `index_html_redirects_to_root` | `/index.html` → 301 to `/` |
| `index_php_redirects_to_root` | `/index.php` → 301 to `/` |

## Aliases (from PHP URLParse.php - source of truth)

| Test | Description |
|------|-------------|
| `alias_key_to_contact` | `/key` → 301 to `/contact.htm` |
| `alias_key_case_insensitive` | `/KEY` → 301 to `/contact.htm` |
| `alias_trent_to_trustedthirdparty` | `/trent` → 301 to `/trustedthirdparty.htm` |
| `alias_trent_case_insensitive` | `/TRENT` → 301 to `/trustedthirdparty.htm` |
| `alias_audits_to_software_security_auditing` | `/audits/` → 301 to `/software-security-auditing.htm` |
| `alias_keyboarddefect` | `/keyboarddefect` → 301 to `/asuskeyboarddefect.htm` |
| `alias_passwords_to_passgen` | `/passwords` → 301 to `/passgen.htm` |
| `alias_password_to_passgen` | `/password` → 301 to `/passgen.htm` |
| `alias_pass_to_passgen` | `/pass` → 301 to `/passgen.htm` |
| `alias_bh2016_to_side_channel` | `/bh2016` → 301 to `/side-channel-attacks-on-everyday-applications.htm` |
| `alias_BH2016_preserved_case` | `/BH2016` → 301 to `/side-channel-attacks-on-everyday-applications.htm` |
| `alias_pphos_to_hall_of_shame` | `/pphos` → 301 to `/password-policy-hall-of-shame.htm` |

## Implementation Notes

- Create `tests/url_canonicalization.rs` with `mod common;` at top
- Use `common::client()` which has `redirect::Policy::none()` to catch redirects
- Use `common::assert_redirect(response, expected_location, context)` for redirect tests
- Use `common::assert_success(response, context)` for no-redirect tests
- Use `common::assert_status(response, 404, context)` for 404 tests
- PHP URLParse.php is the source of truth for alias behavior
