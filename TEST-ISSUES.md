# Test Suite Quality Issues

Full QA review of the defuse-tester integration test suite.

Categories:
- **(a)** Tests expecting actually-buggy behavior
- **(b)** Tests not correctly testing what they intend (too lax, wrong assertions, logic bugs)
- **(c)** Critical missing tests

## DEFERRED

### passgen.rs:89-116 — Password length assertion too lax **(b)**

The server generates passwords of exactly 64 characters, but tests only assert `>= 32`. Combined with issue #1, a password with apostrophes would appear much longer than 64 characters. Even a totally broken implementation returning 32 random chars would pass.

**Fix:** Assert `== 64` for all three password types.

### 3. passgen.rs — No test for full ASCII character range **(c)**

The password generator uses 94 printable ASCII chars (codes 33-126). The test only checks that generated characters are printable ASCII, never that special characters (`!`, `@`, `#`, etc.) actually appear. A bug restricting output to alphanumeric-only would pass the tests, significantly reducing password entropy.

**Fix:** Generate a long password and assert that non-alphanumeric special characters appear.

### 6. quantum_time_capsule.rs:640-663 — submit_too_large_rejected doesn't verify no DB write **(b)**

The test creates a 200,000-char ciphertext and checks for an error message, but doesn't verify the archive message count stayed the same. A bug where the server shows an error but still writes to the DB would pass this test.

**Fix:** Compare archive message count before and after the request.

### 19. breach.rs:30-35 — Fragile section extraction **(b)**

The breach section extraction finds the first `</div>` after `<div style="padding-left: 30px;">`. If the template ever adds nested divs inside this section, extraction breaks silently.

**Fix:** Use a unique id attribute on the div or an HTML parser for section extraction.

### common/mod.rs:152-173 — `assert_redirect` hardcodes 301, `post_form` missing Origin **(b)**

`assert_redirect` always asserts 301, preventing use for 302 redirects. Some tests work around this with `is_redirection()` instead. Also, `post_form` doesn't set an Origin header (the `origin()` function exists but is unused), so POST tests needing CSRF would fail.

**Fix:** Add an `expected_status` parameter to `assert_redirect`. Add Origin header to `post_form`.



---
---
---

## FIXED

### 24. quantum_time_capsule.rs:784-806 — Single-line test is tautological **(b)**

`message_format_single_line` claims to verify messages are single-line, but as the code comment itself admits: "we're iterating by lines so this is implicitly true." The test only checks that `time:` lines contain `algorithm:` and `ciphertext:`, which is already tested elsewhere.

**Fix:** Count `time:` lines and verify it equals the expected message count, or remove the redundant test.

### domain_redirects.rs:342-375 — `bin.defuse.ca` key redirect expects path rewriting that doesn't exist **(a)**

`bin_subdomain_redirects_to_b_path` asserts that `https://bin.defuse.ca/abc123XYZ` redirects to `https://defuse.ca/b/abc123XYZ`. The middleware only does host canonicalization — it would redirect to `https://defuse.ca/abc123XYZ` (no `/b/` prefix). The pastebin_view.rs code explicitly says: "We've removed the bin.defuse.ca/key -> defuse.ca/b/key redirect since these URLs haven't been generated in years."

**Fix:** Expect redirect to `https://defuse.ca/abc123XYZ`, or remove the test.

### domain_redirects.rs:91-122 — Combined redirect chain expects single-hop multi-canonicalization **(a)**

`full_redirect_chain_no_extension` sends `http://www.defuse.ca/about` and expects a single redirect to `https://defuse.ca/about.htm`. But the middleware handles redirects in strict priority: Step 1 (host canonicalization) returns immediately, never reaching Step 3 (URL path canonicalization). The first redirect goes to `https://defuse.ca/about`; the `.htm` addition only happens on a subsequent request. The test uses `Policy::none()` (no redirect following).

**Fix:** Expect `https://defuse.ca/about` as the first redirect, or follow redirects to verify final destination.

### domain_redirects.rs:127-158 — Same single-hop problem for case-insensitive chain **(a)**

`full_redirect_chain_case_insensitive` sends `http://www.defuse.ca/aBouT.htm` and expects redirect to `https://defuse.ca/about.htm`. The middleware redirects to `https://defuse.ca/aBouT.htm` (host canonicalization only, preserving case). Case normalization only runs on a subsequent request.

**Fix:** Expect `https://defuse.ca/aBouT.htm` as the first redirect.

### pastebin.rs:118 — Not-found pastes return 200 instead of 404 **(a)**

The `invalid_url_key` and `extremely_long_key` tests assert status 200 for paste-not-found in HTML mode. The server at `pastebin_view.rs:118` hardcodes `StatusCode::OK` for both found and not-found pastes. This is a server bug — not-found should return 404. The tests codify this buggy behavior.

**Fix:** Fix the server to return 404 for not-found pastes, then update test assertions.

### trent.rs:361-371 — `view_drawing_negative_number` has no meaningful assertions **(b)**

The test sends `drawingnum=-1` and only asserts the response is 2xx. It reads the body into `_body` (underscore prefix) but never checks it. The server renders a "does not exist" page for negative IDs but this is never verified.

**Fix:** Assert `body.contains("does not exist")` or similar.

### trent.rs:1258-1273 — `random_numbers_in_range` doesn't verify any matches found **(b)**

Uses `Regex::captures_iter` to extract random numbers and verify they're in range, but never asserts that ANY captures were found. If the regex fails to match (e.g., format change), the loop iterates zero times and the test passes vacuously. Should find exactly 10 matches.

**Fix:** Add `assert_eq!(re.captures_iter(&body).count(), 10, "Should have found 10 random numbers")`.

### 23. vim_highlighting.rs:72-74 — Non-existent method names in assertion **(b)**

Two of three alternatives (`vimhl_convert`, `vimhl_exec`) don't exist in the codebase. Only `VimHighlight` is a real identifier.

**Fix:** Remove fabricated method names; only check for `VimHighlight`.

### upvote.rs — No CSRF rejection test **(c)**

The `/upvote.php` endpoint has CSRF protection that rejects requests without a valid Origin/Referer header. No test verifies this. If CSRF protection is accidentally removed, no test catches it.

**Fix:** Add a test sending POST to `/upvote.php` without Origin header (or with `https://evil.com`) and verify the response indicates failure.

### 8. No path traversal tests for storage directories **(c)**

No tests verify that path traversal attacks are blocked on `/files/`, `/upload/`, `/source/`, etc. Requests like `/files/../../.env` or `/upload/../../../etc/passwd` should return 400 or 404. While `tower_http::ServeDir` handles this internally, this is security-critical behavior that deserves explicit test coverage — a `tower_http` version upgrade or storage route refactoring could break it. The server code explicitly warns: "storage itself contains credentials, only ever serve dirs in storage/extras!"

**Fix:** Add tests for path traversal attempts on each storage directory, verifying they return 400 or 404.

### 9. downloads.rs — No Content-Disposition checks for `/files2/` **(c)**

`/files2/` is explicitly designed to be "viewable in browser" (no forced download), unlike `/files/`, `/mirrors/`, and `/upload/`. Tests verify `/files2/` returns 200 but never verify the *absence* of `Content-Disposition: attachment`. A regression accidentally adding `/files2/` to the download list would break UX and only be caught by a single test on a single file in `content_types.rs`.

**Fix:** `/files2/` tests in `downloads.rs` should verify `Content-Disposition` is absent.

### 10. online_x86_assembler.rs:194-196 — `contains_error` is too broad **(b)**

Checks for `"Error"` and `"error"` anywhere in the page HTML. The page HTML may contain these words in contexts other than actual errors (CSS class names, page instructions). Tests using `!contains_error(&body)` could pass even when actual errors occurred.

**Fix:** When asserting not an error, !contains_error() is fine, but tests that expect an error message should check for the actual expected error message.

### 11. online_x86_assembler.rs:625-633 — Empty input test trivially satisfied **(b)**

The test for empty assembly input only asserts `body.len() > 0`. Any HTTP response (even a 500 error page) satisfies this.

**Fix:** Assert no error is displayed and no assembly result section appears.

### 12. online_x86_assembler.rs:1037-1057 — Invalid architecture test doesn't verify behavior **(b)**

Sends invalid architecture `"arm64"` and only asserts `body.len() > 0`. The server defaults to x86 for invalid architectures, but the test doesn't verify this fallback.

**Fix:** Verify the assembly succeeds with x86 semantics (e.g., `nop` assembles to `0x90`).

### 13. online_x86_assembler.rs:461-469 — Mixed case hex test too weak **(b)**

Only checks for capital `"Error"` (not lowercase `"error"`), and doesn't verify any actual disassembly output was produced.

**Fix:** Use `!contains_error(&body)` for consistency and verify hex output is present.

### 14. big_number_calculator.rs:964-978 — Whitespace-only test allows two behaviors **(b)**

Allows both `Err` and empty `Ok`, masking which behavior actually occurs. The server consistently returns an error for whitespace-only input.

**Fix:** Assert `result.is_err()` specifically.

### 13. big_number_calculator.rs:952-961 — Scientific notation test non-deterministic **(b)**

`1e5` test allows both error and success. The parser consistently rejects `1e5` (not valid hex since it lacks `0x`, not valid decimal with `e`).

**Fix:** Change the parser to allow 1e5 without introducing any security issues (i.e. decimal digits + e + decimal digits) and change the test to expect it to produce the correct float output, e.g. 1e5  100000.0 (without add spaces setting)

### 14. vim_highlighting.rs:34-37 — "highlight" fallback matches page prose **(b)**

The assertion `body.contains("vimhighlight") || body.contains("vim-highlight") || body.contains("highlight")` is trivially true because the word "highlight" appears in the page's regular text content, even without vim output.

**Fix:** Only check for `"vimhighlight"` (the actual CSS class from `highlight_string`).

### 15. vim_highlighting.rs:53-56 — "function" check doesn't verify highlighting **(b)**

Checks that the word `"function"` appears in the body but doesn't verify it's inside a syntax-highlighted span. The word would be present even if vim highlighting completely failed (since it's in the raw PHP source).

**Fix:** Verify `function` appears inside a highlighted span (e.g., inside `<span class="`).

### 16. special_endpoints.rs — getmyip IPv6 regex is dead code **(b)**

The IPv6 detection `body.contains(":") && Regex::new(r"[0-9a-fA-F:]+").is_match(&body)` is always true for any HTML page (colons and hex chars appear everywhere in HTML). The `has_ipv4 || has_ipv6` assertion can never fail because `has_ipv6` is always true.

**Fix:** Use a proper IPv6 regex like `[0-9a-fA-F]{1,4}(:[0-9a-fA-F]{1,4}){2,7}` and extract it from a specific element, not the whole page.

### 18. quantum_time_capsule.rs:627-637 — Timestamp check never implemented **(b)**

The comment says "verify timestamp is reasonable (within last minute)" but the code only checks `timestamp_str.len() >= 19`, which is redundant since the regex already ensures this length. Any valid-format date from any year passes.

**Fix:** Parse the timestamp and verify it's within a reasonable time window.

### 20. blog.rs — No test for nonexistent blog slug **(c)**

Only tests that a valid blog slug redirects. No test verifies that `/blog/nonexistent-slug-xyz` returns 404. A bug where all slugs redirect regardless of file existence wouldn't be caught.

**Fix:** Add test for nonexistent blog slug returning 404.

### 22. checksums.rs — No test that normalize=no preserves line endings **(c)**

Tests verify normalize=yes removes line endings and normalize=no works for simple strings. But no test verifies that normalize=no actually *preserves* line endings in the hash input.

**Fix:** Hash `"test\ntest"` with normalize=no and verify the hash differs from hashing `"testtest"`.

### content_types.rs:42-44 — 404 content-type test doesn't assert 404 status **(b)**

`not_found_page_has_html_content_type` verifies Content-Type is `text/html` for a nonexistent page but never asserts the response status is 404. If a bug caused the server to return 200 for nonexistent pages, this test would still pass.

**Fix:** Add `assert_status(&resp, 404, ...)` before checking content type.

### content_types.rs:62-93 — Inconsistent inline rendering assertions **(b)**

`files2_dir_viewable_in_browser` requires NO Content-Disposition header (`is_none()`), while `downloads_dir_allows_inline_rendering` only checks the header doesn't contain "attachment" if present. Both test the same semantic behavior (inline/viewable) but with different strictness. Neither path should have any Content-Disposition header.

**Fix:** Both tests should assert `disposition.is_none()`.

### static_pages.rs — Many content assertions are too weak **(b)**

Several static page tests use generic content assertions that would match almost any page: `research_side_channel_attacks` checks for `"Side"/"side"`, `page_about` checks for `"About"/"about"`, `page_contact` checks for `"Contact"/"contact"`. If the server returned the wrong page for all URLs, many tests would still pass because these terms appear in common site elements (navigation, footers).

**Fix:** Assert on content unique to each specific page (e.g., specific page titles, unique terms from that page's content).

### http_methods.rs:16-32 — HEAD test claims to check "no body" but doesn't **(b)**

The test comment says "HEAD request on CSS file should return 200 with headers but no body" but it only checks status and two headers. It never verifies the response body is actually empty.

**Fix:** Add `assert_eq!(resp.text().await.unwrap().len(), 0, "HEAD response should have empty body")`.

### security_headers.rs — Tests don't verify response status before checking headers **(b)**

The security header tests iterate over `TEST_PATHS` including storage paths that may not exist in all environments. Since the middleware adds headers to ALL responses (including 404s), tests pass even if files are missing. The tests don't verify the routes actually work — only that the middleware applies globally.

**Fix:** Add `assert!(resp.status().is_success(), ...)` before header assertions.

### pastebin.rs:415-424 — `view_paste_not_found` doesn't verify status code **(b)**

Checks body text for "not found" message but doesn't assert any HTTP status code. Should at minimum document what status is expected.

**Fix:** Assert the status code (404)

### trent.rs:1350-1370 — `random_negative_range` doesn't verify match count **(b)**

Same vacuous-pass issue as `random_numbers_in_range`. Iterates regex captures but never verifies the expected 5 matches were found.

**Fix:** Add `assert_eq!(re.captures_iter(&body).count(), 5, ...)`.


---
---
---
---
---

## MEDIUM Severity

### static_files.rs:259-303 — Storage download tests don't verify Content-Type **(b)**

`upload_vim_vimrc`, `upload_vim_zip`, `files_eharmony_hashes`, and `mirrors_zcash_blockchain` verify 200 status and `Content-Disposition: attachment` but don't check Content-Type. A `.zip` served as `application/octet-stream` or wrong MIME type wouldn't be caught.

**Fix:** Spot-check that well-known file types get appropriate Content-Type headers.

### security_headers.rs — No test for security headers on error responses **(c)**

All security header tests only check successful (200) responses. No test verifies that 404/500 error pages also carry security headers. The middleware adds headers to all responses, but a regression (e.g., error handler bypassing middleware) would go undetected.

**Fix:** Request a nonexistent path and verify the 404 response has X-Frame-Options, X-Content-Type-Options, and Referrer-Policy.

### special_endpoints.rs — Missing XSS test for getmyip.php user-agent display **(c)**

`/getmyip.php` displays the user-agent string in HTML. The server properly uses `html_escape(user_agent)`, but no test verifies this. A test should send a UA like `<script>alert(1)</script>` and verify the response contains `&lt;script&gt;`.

**Fix:** Add XSS test with malicious user-agent string.

### special_endpoints.rs — Missing XSS test for /s.php decoded text **(c)**

The shout page decodes base64 and displays in HTML. The server uses `html_escape(&decoded)`, but no test verifies HTML special chars are escaped in decoded text. This is a reflected XSS vector.

**Fix:** Base64-encode `<script>alert(1)</script>`, send via `?s=`, verify response contains escaped form.

### source_security.rs — Missing test for /source/ directory listing **(c)**

No test verifies that `/source/` (with trailing slash) returns 404/403 rather than a directory listing. `ServeDir` doesn't serve listings by default, but this security-critical path should have explicit coverage.

**Fix:** Request `/source/` and assert 404 or 403.

### pastebin.rs — Missing test for maximum paste size rejection **(c)**

The server enforces a 50 MB paste size limit at `pastebin_add.rs:43-44`. No integration test verifies that oversized pastes are rejected.

**Fix:** Add a test sending a paste larger than 50 MB and verify 400 status with appropriate error.

### trent.rs — Missing XSS test with apostrophes **(c)**

The `description_with_html` and `xss_prevention_*` tests verify angle brackets are escaped, but none test apostrophe escaping (used in JS event handlers like `onload='...'`). The server escapes `'` as `&#039;` but this isn't explicitly tested.

**Fix:** Create a drawing with a name containing apostrophes and verify `&#039;` appears in the output.

### source_security.rs — No Content-Disposition check for non-PHP source files **(c)**

`source_blind_birthday_rb` in `static_files.rs` verifies `/source/blind-birthday.rb` returns 200 with Ruby code but doesn't check for `Content-Disposition: attachment`. The middleware adds this for ALL `/source/` files, but only PHP files are tested for it. A regression removing `/source/` from the download path check for non-PHP files wouldn't be caught.

**Fix:** Verify `Content-Disposition: attachment` for non-PHP `/source/` files too.

---