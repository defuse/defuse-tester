# Pastebin Test Plan

**Status:** PLANNED

**File:** `tests/pages/pastebin.rs`

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
| `main_page_has_form` | Check form fields | Has paste textarea, lifetime select, shorturl checkbox, submit button |
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
| `create_custom_lifetime` | POST with lifetime=600 | Paste expires in ~10 minutes |
| `create_default_lifetime` | POST without lifetime | Defaults to 10 days |
| `line_endings_normalized_crlf` | POST paste with \r\n | Retrieved paste has \n only |
| `line_endings_normalized_cr` | POST paste with \r | Retrieved paste has \n only |

### 4. Viewing Pastes (`GET /b/{key}`)

| Test | Description | Expected |
|------|-------------|----------|
| `view_paste_exists` | GET valid paste URL | 200, shows paste content |
| `view_paste_not_found` | GET /b/nonexistent | Shows "paste you were looking for could not be found" |
| `view_paste_shows_timeleft` | View paste | Shows "will be deleted in X days, Y hours, Z minutes" |
| `view_paste_content_escaped` | Paste with `<script>` | Content is HTML-escaped |
| `view_paste_tabs_converted` | Paste with tabs | Tabs shown as 4 nbsp |
| `view_paste_double_spaces` | Paste with "a  b" | Double spaces shown as double nbsp |
| `view_paste_has_form` | View page structure | Has textarea with paste content, lifetime dropdown, submit button |
| `view_cache_disabled` | Check headers | Cache-Control: no-cache, must-revalidate |

### 5. Raw Paste Retrieval (`GET /b/{key}?raw=true`)

| Test | Description | Expected |
|------|-------------|----------|
| `raw_paste_plain_text` | GET ?raw=true on normal paste | Content-Type: text/plain, exact paste content |
| `raw_paste_jscrypt_error` | GET ?raw=true on jscrypt paste | Returns error message about client-side encryption |
| `raw_paste_preserves_content` | Create paste, get raw | Exact content returned (minus line ending normalization) |
| `raw_paste_not_found` | GET /b/nonexistent?raw=true | Empty or error |

### 6. JS-Encrypted Pastes (jscrypt=yes)

| Test | Description | Expected |
|------|-------------|----------|
| `jscrypt_paste_shows_password_prompt` | View jscrypt paste | Shows "Enter Password" field |
| `jscrypt_paste_no_content_visible` | View jscrypt paste | Paste content not visible in HTML (only in JS var) |
| `jscrypt_paste_raw_blocked` | GET ?raw=true | Returns error, not ciphertext |
| `jscrypt_flag_preserved` | Create jscrypt, view | jscrypt flag is set |

### 7. URL Key Format

| Test | Description | Expected |
|------|-------------|----------|
| `url_key_alphanumeric` | Check key format | Only [A-Za-z0-9] |
| `url_key_standard_length` | Without shorturl | 22 characters |
| `url_key_short_length` | With shorturl=yes | 8 characters |
| `url_key_unique` | Create two pastes | Different keys |

### 8. Content Handling

| Test | Description | Expected |
|------|-------------|----------|
| `unicode_preserved` | Paste with unicode | Unicode correctly stored and retrieved |
| `special_chars_preserved` | Paste with &<>"' | Characters preserved (escaped in HTML view, raw in ?raw) |
| `whitespace_preserved` | Paste with leading/trailing whitespace | Whitespace preserved |
| `empty_lines_preserved` | Paste with empty lines | Empty lines preserved |
| `large_paste` | 100KB paste | Successfully stored and retrieved |

### 9. Lifetime/Expiration

| Test | Description | Expected |
|------|-------------|----------|
| `lifetime_600_accepted` | 10 minutes | Accepted, paste created |
| `lifetime_3600_accepted` | 60 minutes | Accepted |
| `lifetime_86400_accepted` | 1 day | Accepted |
| `lifetime_864000_accepted` | 10 days | Accepted |
| `lifetime_2592000_accepted` | 30 days | Accepted |
| `lifetime_15552000_accepted` | 6 months | Accepted |
| `lifetime_invalid_defaults` | lifetime=abc | PHP casts to 0, paste expires immediately |
| `timeleft_display_format` | View paste | Shows "X days, Y hours, Z minutes" format |
| `paste_expires_and_deleted` | Create with lifetime=5, wait 8s, view | Paste not found after expiry |
| `lifetime_zero_expires_immediately` | lifetime=0 | Paste created but expires immediately |

### 9a. Lifetime Validation Bug (IGNORED - documents current behavior)

| Test | Description | Expected | Notes |
|------|-------------|----------|-------|
| `lifetime_exceeds_max_accepted` | lifetime=315360000 (10 years) | **BUG:** Accepted with no validation | See docs/issues/pastebin-no-max-lifetime.md |
| `lifetime_negative_accepted` | lifetime=-1 | Expires immediately (time in past) | Edge case |

### 10. Round-Trip Tests

| Test | Description | Expected |
|------|-------------|----------|
| `roundtrip_simple` | Create paste, view it | Content matches |
| `roundtrip_with_html` | Paste containing HTML | Escaped in view, exact in raw |
| `roundtrip_multiline` | Multi-line paste | Lines preserved |
| `roundtrip_repost` | View paste, repost from form | New paste created with same content |

### 11. Error Handling

| Test | Description | Expected |
|------|-------------|----------|
| `post_without_paste_field` | POST with no paste | "Empty post!" error |
| `invalid_url_key` | GET /b/!@#$%^ | Not found page (or error) |
| `extremely_long_key` | GET /b/aaaa...aaa (1000 chars) | Not found (graceful handling) |

### 12. Form Field Passthrough

When viewing a paste, the form allows reposting. Verify these are preserved:

| Test | Description | Expected |
|------|-------------|----------|
| `view_has_paste_in_textarea` | Non-jscrypt paste | Textarea contains paste text |
| `view_has_lifetime_dropdown` | View page | Lifetime select present with options |
| `view_has_shorturl_checkbox` | View page | Short URL checkbox present |
| `view_can_repost` | Submit form from view page | Creates new paste |

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
5. **Display escaping**: `htmlentities()` then tabs→4 spaces, double-space→double-nbsp
6. **Expired cleanup**: Happens on each add/view request

### What We're NOT Testing

- Client-side JavaScript encryption/decryption (requires browser)
- The specific delete hash (admin functionality)
- Database-level encryption details (implementation detail)
- bin.defuse.ca domain redirect (legacy domain)

### 13. Security Headers

| Test | Description | Expected |
|------|-------------|----------|
| `view_no_cache_headers` | GET /b/{key} headers | Cache-Control: no-cache, must-revalidate |
| `view_expires_header` | GET /b/{key} headers | Expires: Mon, 01 Jan 1990 00:00:00 GMT |
| `https_enforced_on_view` | HTTP request to /b/{key} | 301 redirect to HTTPS |
| `view_content_type_html` | GET /b/{key} | Content-Type: text/html; charset=utf-8 |
| `raw_content_type_plain` | GET /b/{key}?raw=true | Content-Type: text/plain |

## Test Dependencies

- Tests create real pastes on the server - use short lifetimes (600 seconds or less)
- Expiry test (`paste_expires_and_deleted`) requires ~8 second wait
- Tests should be idempotent where possible
- Use unique content to avoid collisions with other test runs
