# Quantum Computer Time Capsule Test Plan

**Status:** PLANNED

**File:** `tests/quantum_time_capsule.rs`

## Overview

The time capsule allows users to submit encrypted messages that will only be readable when large-scale quantum computers are built. The encryption uses X25519 elliptic curve cryptography (via TweetNaCl.js in the browser) which is vulnerable to quantum attacks but infeasible to break with classical computers.

Key features:
- Client-side encryption using TweetNaCl.js (X25519 + XSalsa20-Poly1305)
- reCAPTCHA protection against spam
- Message archive downloadable as text file
- Message count and "last added" timestamp displayed
- Blockchain checkpoints for archive authenticity

## Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/quantum-computer-time-capsule.htm` | GET | Main page with form, message count, last timestamp |
| `/quantum-computer-time-capsule.htm` | POST | Submit encrypted message (requires reCAPTCHA) |
| `/timecapsule/quantum-computer-time-capsule-download.php` | GET | Download full archive as text file |
| `/timecapsule/timecapsule-save.js` | GET | Client-side encryption code |
| `/timecapsule/tweetnacl-time-capsule.js` | GET | TweetNaCl library |
| `/timecapsule/tweetnacl-util-time-capsule.js` | GET | TweetNaCl utilities |

## Test Cases

### 1. Main Page (GET)

| Test | Description | Expected |
|------|-------------|----------|
| `main_page_loads` | GET /quantum-computer-time-capsule.htm | 200 |
| `main_page_has_title` | Check page title | Contains "Time Capsule" or "Message to the Future" |
| `main_page_has_form` | Check form exists | Has form with action to same page, method POST |
| `main_page_has_message_textarea` | Check message input | Has textarea with id/name "message" |
| `main_page_has_hidden_fields` | Check hidden form fields | Has hidden inputs: algorithm, present_public_key, future_public_key, ciphertext |
| `main_page_has_submit_button` | Check submit button | Has submit/button element |
| `main_page_shows_message_count` | Check message count display | Contains "X messages" where X is a number |
| `main_page_shows_last_time` | Check last message time | Contains "ago" (e.g., "5 days ago", "3 hours ago") |
| `main_page_has_recaptcha` | Check for reCAPTCHA | Contains "g-recaptcha" or reCAPTCHA script |
| `main_page_has_download_link` | Check archive download | Has link/form to download endpoint |
| `main_page_loads_encryption_js` | Check JS includes | References timecapsule-save.js and tweetnacl scripts |

### 2. Archive Download

| Test | Description | Expected |
|------|-------------|----------|
| `download_returns_text_plain` | GET download endpoint | Content-Type: text/plain |
| `download_has_attachment_header` | Check Content-Disposition | Contains "attachment" and filename with .txt |
| `download_filename_has_timestamp` | Check filename format | Filename contains "MessagesToTheQuantumComputingFuture" and date |
| `download_has_header_section` | Check archive header | Contains "TIME CAPSULE FOR A QUANTUM-COMPUTING FUTURE" |
| `download_has_source_timecapsule_js` | Check source code section | Contains "SOURCE CODE: timecapsule-save.js" |
| `download_has_source_tweetnacl` | Check TweetNaCl source | Contains "SOURCE CODE: tweetnacl-time-capsule.js" |
| `download_has_messages_section` | Check messages section marker | Contains "MESSAGES FOR THE FUTURE" |
| `download_has_footer_section` | Check footer | Contains "ADDITIONAL INFORMATION" |
| `download_has_actual_messages` | Check for message format | Contains lines starting with "time:" (message format) |
| `download_messages_have_expected_format` | Validate message structure | Messages contain: time, algorithm, presentpublickey, futurepublickey, ciphertext |

### 3. Static JS Files

| Test | Description | Expected |
|------|-------------|----------|
| `timecapsule_js_loads` | GET /timecapsule/timecapsule-save.js | 200 |
| `timecapsule_js_has_send_function` | Check JS content | Contains "function sendMessage" |
| `timecapsule_js_content_type` | Check content type | text/javascript or application/javascript |
| `tweetnacl_js_loads` | GET /timecapsule/tweetnacl-time-capsule.js | 200 |
| `tweetnacl_util_js_loads` | GET /timecapsule/tweetnacl-util-time-capsule.js | 200 |

### 4. POST Without CAPTCHA (verifies validation)

| Test | Description | Expected |
|------|-------------|----------|
| `post_without_captcha_rejected` | POST with valid data but no g-recaptcha-response | Shows "Please solve the CAPTCHA" |
| `post_preserves_message_on_error` | POST with message, CAPTCHA fails | Textarea contains the submitted message |

### 5. POST With CAPTCHA Bypass (IGNORED - requires CAPTCHA bypass)

These tests require either:
1. A test mode that bypasses reCAPTCHA verification
2. A way to programmatically solve CAPTCHAs (not practical)

For the Rust port, consider adding a bypass mechanism (e.g., secret header, env var, or test endpoint).

| Test | Description | Expected | Status |
|------|-------------|----------|--------|
| `submit_valid_message` | POST with valid encrypted message + CAPTCHA | Success message, encrypted text echoed | IGNORED |
| `submit_message_appears_in_archive` | Submit then download archive | New message at end of archive | IGNORED |
| `submit_message_increments_count` | Check count before/after submit | Count increases by 1 | IGNORED |
| `submit_updates_last_timestamp` | Check timestamp after submit | Shows "seconds ago" or "minutes ago" | IGNORED |
| `submit_too_large_rejected` | POST with >200KB encrypted message | Error about message too big | IGNORED |
| `submit_with_newlines_rejected` | POST with newlines in ciphertext field | Error about newlines | IGNORED |
| `submit_records_timestamp` | Submit and check archive | Message line contains timestamp close to now | IGNORED |
| `submit_stores_all_fields` | Submit and check archive | Message contains algorithm, both keys, ciphertext | IGNORED |

### 6. Message Format Validation

| Test | Description | Expected |
|------|-------------|----------|
| `message_format_has_time` | Check archived message format | Contains "time:" followed by ISO 8601 timestamp |
| `message_format_has_algorithm` | Check algorithm field | Contains "algorithm:tweetnacl-js-x25519-xsalsa20-poly1305-zerononce" |
| `message_format_has_present_key` | Check present public key | Contains "presentpublickey:" followed by base64 |
| `message_format_has_future_key` | Check future public key | Contains "futurepublickey:" followed by base64 |
| `message_format_has_ciphertext` | Check ciphertext | Contains "ciphertext:" followed by base64 |
| `message_format_single_line` | Check message is one line | No newlines within message |

### 7. Security Headers

| Test | Description | Expected |
|------|-------------|----------|
| `main_page_content_type` | Check Content-Type | text/html |
| `download_no_sniff` | Check X-Content-Type-Options | nosniff (prevents MIME confusion) |

### 8. HTTP→HTTPS Redirects (in `domain_redirects.rs`)

| Test | Description | Expected |
|------|-------------|----------|
| `timecapsule_download_http_redirects_to_https` | HTTP GET download endpoint | Redirects to HTTPS |

### 9. Archive Consistency

| Test | Description | Expected |
|------|-------------|----------|
| `message_count_matches_archive` | Compare count on main page to lines in archive | Count matches number of message lines in downloaded archive |

## Implementation Notes

### Form Fields for POST

```
message: plaintext message (echoed back on error, not stored)
algorithm: "tweetnacl-js-x25519-xsalsa20-poly1305-zerononce"
present_public_key: base64-encoded X25519 public key
future_public_key: base64-encoded X25519 public key
ciphertext: base64-encoded encrypted message
g-recaptcha-response: reCAPTCHA token from Google
```

### Server Behavior

1. **CAPTCHA Check**: Verifies g-recaptcha-response with Google's API
2. **Size Validation**: Rejects if formatted message >= 200,000 bytes
3. **Newline Validation**: Rejects if ciphertext contains \n or \r
4. **Storage Format**: `time:{ISO8601} algorithm:{alg} presentpublickey:{key} futurepublickey:{key} ciphertext:{ct}`
5. **Timestamp**: Server records UTC time when message is received
6. **Message Echo**: On success, displays the encrypted message to user

### Archive Structure

```
==== TIME CAPSULE FOR A QUANTUM-COMPUTING FUTURE ====

{archive-header.txt contents}

==== SOURCE CODE: timecapsule-save.js ====
{timecapsule-save.js contents}

==== SOURCE CODE: tweetnacl-time-capsule.js ====
{tweetnacl-time-capsule.js contents}

==== SOURCE CODE: tweetnacl-util-time-capsule.js ====
{tweetnacl-util-time-capsule.js contents}

==== MESSAGES FOR THE FUTURE ====

{one message per line, ordered by database ID}

==== ADDITIONAL INFORMATION ====

{archive-footer.txt contents - blockchain hash locations}
```

### What We're NOT Testing

- Client-side JavaScript encryption (requires browser)
- Actual reCAPTCHA solving
- Blockchain verification of archive hashes
- Database ordering/integrity beyond format checks
- Clock accuracy of timestamps

## Test Dependencies

- Main page tests are read-only and safe
- Archive download is read-only
- POST tests without CAPTCHA will fail validation (expected)
- POST tests with CAPTCHA bypass require implementation support

## CAPTCHA Bypass for Rust Port

Consider implementing one of these approaches for testing:

1. **Environment Variable**: Skip CAPTCHA check if `SKIP_RECAPTCHA=1`
2. **Test Header**: Skip if request has `X-Test-Bypass-Captcha: {secret}`
3. **Test Endpoint**: Separate `/test/timecapsule-submit` that skips CAPTCHA
4. **Mock Mode**: Accept a special `g-recaptcha-response` value in test mode

The bypass should only be available in non-production builds or behind authentication.
