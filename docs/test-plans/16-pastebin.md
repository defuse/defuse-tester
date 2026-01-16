# Pastebin Test Plan

**Status:** DONE (52 tests, 2 ignored)

**File:** `tests/pastebin.rs`

## Overview

The pastebin allows users to create encrypted text pastes with configurable expiration. Key features:

- Server-side AES-128-CBC encryption (key derived from URL)
- Optional client-side JS encryption (password-protected)
- Configurable expiration (10 min to 6 months)
- Short or long URL keys (8 vs 22 alphanumeric chars)
- Raw text retrieval endpoint
- HTTPS enforced

## Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/pastebin.htm` | GET | Main pastebin form page |
| `/bin/add.php` | POST | Create new paste |
| `/b/{key}` | GET | View paste (HTML) |
| `/b/{key}?raw=true` | GET | Raw paste content (text/plain) |
| `/bin/` | GET | Redirect to /pastebin.htm |
| `/b/` | GET | Redirect to /pastebin.htm |

## Test Cases

### 1. Main Page (`/pastebin.htm`)

| Test | Description | Expected |
|------|-------------|----------|
| `main_page_loads` | GET /pastebin.htm | 200, contains form |
| `main_page_has_form_fields` | Check form fields | Has paste textarea, lifetime select, shorturl checkbox, submit button |
| `main_page_has_lifetime_options` | Check lifetime dropdown | 6 months, 30 days, 10 days (default), 1 day, 60 min, 10 min |

### 2. Redirects

| Test | Description | Expected |
|------|-------------|----------|
| `bin_index_redirects` | GET /bin/ | 301 redirect to /pastebin.htm |
| `b_index_redirects` | GET /b/ | 301 redirect to /pastebin.htm |

### 3. Creating Pastes (`POST /bin/add.php`)

| Test | Description | Expected |
|------|-------------|----------|
| `create_simple_paste` | POST with paste=hello | 302 redirect to /b/{key}, key is 22 chars alphanumeric |
| `create_empty_paste_fails` | POST without paste field | Error message "Empty post!" |
| `create_short_url` | POST with shorturl=yes | Redirect URL key is 8 chars |
| `create_jscrypt_paste` | POST with jscrypt=yes | Paste stored with jscrypt flag |
| `valid_lifetimes_accepted` | POST with each valid lifetime (600, 3600, 86400, 864000, 2592000, 15552000) | All 6 values accepted |
| `line_endings_normalized_crlf` | POST paste with \r\n | Retrieved paste has \n only |
| `line_endings_normalized_cr` | POST paste with \r | Retrieved paste has \n only |

### 4. Viewing Pastes (`GET /b/{key}`)

| Test | Description | Expected |
|------|-------------|----------|
| `view_paste_exists` | GET valid paste URL | 200, shows paste content |
| `view_paste_not_found` | GET /b/nonexistent | Shows "paste you were looking for could not be found" |
| `view_paste_shows_timeleft` | View paste | Shows "will be deleted in X days, Y hours, Z minutes" |
| `view_paste_content_escaped` | Paste with `<script>` | Content is HTML-escaped |
| `view_paste_tabs_converted` | Paste with tabs | Tabs shown as 4 nbsp in display area |
| `view_paste_double_spaces` | Paste with "A  B" | Double spaces shown as double nbsp |
| `view_paste_has_form` | View page structure | Has textarea with paste content, lifetime dropdown |
| `view_cache_disabled` | Check headers | Cache-Control: no-cache |

### 5. Raw Paste Retrieval (`GET /b/{key}?raw=true`)

| Test | Description | Expected |
|------|-------------|----------|
| `raw_paste_plain_text` | GET ?raw=true on normal paste | Content-Type: text/plain |
| `raw_paste_jscrypt_error` | GET ?raw=true on jscrypt paste | Returns error message about client-side encryption |
| `raw_paste_preserves_content` | Create paste, get raw | Exact content returned (minus line ending normalization) |
| `raw_paste_not_found` | GET /b/nonexistent?raw=true | Empty or error |

### 6. JS-Encrypted Pastes (jscrypt=yes)

| Test | Description | Expected |
|------|-------------|----------|
| `jscrypt_paste_shows_password_prompt` | View jscrypt paste | Shows password input field |
| `jscrypt_paste_content_not_visible` | View jscrypt paste | Paste content not visible in HTML plaintext |

### 7. URL Key Format

| Test | Description | Expected |
|------|-------------|----------|
| `url_key_alphanumeric` | Check key format (5 samples) | Only [A-Za-z0-9] |
| `url_key_standard_length` | Without shorturl | 22 characters |
| `url_key_short_length` | With shorturl=yes | 8 characters |
| `url_key_unique` | Create two pastes | Different keys |

### 8. Content Handling

| Test | Description | Expected |
|------|-------------|----------|
| `unicode_preserved` | Paste with unicode (über 日本語 🎉) | Unicode correctly stored and retrieved |
| `special_chars_preserved` | Paste with &<>"' | Characters preserved in raw |
| `whitespace_preserved` | Paste with leading/trailing whitespace | Whitespace preserved |
| `empty_lines_preserved` | Paste with empty lines | Empty lines preserved |
| `large_paste` | 100KB paste | Successfully stored and retrieved |

### 9. Lifetime/Expiration

| Test | Description | Expected |
|------|-------------|----------|
| `timeleft_display_format` | View paste | Shows "X days, Y hours, Z minutes" format |
| `paste_expires_and_deleted` | Create with lifetime=5, wait 8s, view | Paste not found after expiry |

### 9a. Lifetime Validation Bug (IGNORED - documents current behavior)

| Test | Description | Expected | Notes |
|------|-------------|----------|-------|
| `lifetime_exceeds_max_accepted` | lifetime=315360000 (10 years) | **BUG:** Accepted with no validation | See docs/issues/pastebin-no-max-lifetime.md |
| `lifetime_negative_expires_immediately` | lifetime=u32::MAX | Edge case behavior | Overflow to negative may expire immediately |

### 10. Round-Trip Tests

| Test | Description | Expected |
|------|-------------|----------|
| `roundtrip_simple` | Create paste, view it | Content matches |
| `roundtrip_with_html` | Paste containing HTML | Escaped in view, exact in raw |
| `roundtrip_multiline` | Multi-line paste | Lines preserved |
| `roundtrip_repost` | View paste, extract from textarea, repost | New paste created with same content |

### 11. Error Handling

| Test | Description | Expected |
|------|-------------|----------|
| `post_without_paste_field` | POST with no paste | "Empty post!" error |
| `invalid_url_key` | GET /b/!@#$%^ | Not found page (graceful handling) |
| `extremely_long_key` | GET /b/aaaa...aaa (1000 chars) | Not found (graceful handling) |

### 12. Form Field Passthrough

When viewing a paste, the form allows reposting. Verify these are preserved:

| Test | Description | Expected |
|------|-------------|----------|
| `view_has_paste_in_textarea` | Non-jscrypt paste | Textarea contains paste text |
| `view_has_lifetime_dropdown` | View page | Lifetime select present with options |
| `view_has_shorturl_checkbox` | View page | Short URL checkbox present |
| `view_can_repost` | Check form action | Form posts to /bin/add.php |

### 13. Security Headers

| Test | Description | Expected |
|------|-------------|----------|
| `view_no_cache_headers` | GET /b/{key} headers | Cache-Control: no-cache, must-revalidate |
| `view_expires_header` | GET /b/{key} headers | Expires: date in the past (1990 or 1970) |
| `view_content_type_html` | GET /b/{key} | Content-Type: text/html |
| `raw_content_type_plain` | GET /b/{key}?raw=true | Content-Type: text/plain |

## Implementation Notes

### Form Fields for `/bin/add.php`

```
paste: required, the text content
jscrypt: "yes" or omit/other (default: no)
lifetime: seconds as string (default: 864000 = 10 days)
shorturl: "yes" or omit (default: no/long)
```

### Server Behavior Details

1. **Line ending normalization**: `\r\n` → `\n`, then `\r` → `\n`
2. **Encryption**: AES-128-CBC with random IV, key derived from URL key
3. **Database ID**: `HMAC-SHA256("database_identity", urlKey)`
4. **Encryption key**: `HMAC-SHA256("encryption_key", urlKey)` (raw bytes)
5. **Display escaping**: `htmlentities()` then tabs→4 nbsp, double-space→double-nbsp
6. **Expired cleanup**: Happens on each add/view request

### What We're NOT Testing

- Client-side JavaScript encryption/decryption (requires browser)
- The specific delete hash (admin functionality)
- Database-level encryption details (implementation detail)
- bin.defuse.ca domain redirect (legacy domain)
- HTTP→HTTPS redirect (tested elsewhere in domain_redirects)

## Test Dependencies

- Tests create real pastes on the server - use short lifetimes (600 seconds or less)
- Expiry test (`paste_expires_and_deleted`) requires ~8 second wait
- Tests should be idempotent where possible
- Use unique content to avoid collisions with other test runs
