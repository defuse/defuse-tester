# Test Suite TODO

Issues where tests silently pass without catching broken behavior.

## Critical: Tests that pass without executing any assertions

### 1. COMPLETE — `online_x86_assembler.rs` — `if let Some` skips assertions entirely

Changed `if let Some(hex) = hex_bold { assert!(...) }` to
`let hex_bold = extract_hex_with_bold(&body).expect(...)` in both
`disassemble_with_nul_byte` and `disassemble_multiple_nul_bytes`.

### 2. COMPLETE — `quantum_time_capsule.rs` — `download_no_sniff` security header check

Changed `if let Some(value) = header` to
`.expect("X-Content-Type-Options header should be present")`.

### 3. LEFT AS-IS — `domain_redirects.rs` (11 tests) and `security_headers.rs` (3 tests) — silent skips on local URL

These tests genuinely can't work on localhost (they test www→non-www redirects, HSTS,
HTTP→HTTPS against production domains). Keeping the current `if !is_production_url() { return; }`
pattern.

## High: Overly broad assertions that accept wrong behavior

### 4. COMPLETE — `pastebin.rs` — invalid key tests accept any 2xx or 4xx

Changed to assert exact 200 status and check body contains
"Sorry, the paste you were looking for could not be found." for both
`invalid_url_key` and `extremely_long_key`.

### 5. COMPLETE — `online_x86_assembler.rs` — relative jump test

Split `assert!(!contains_error(&body) || extract_hex_output(&body).is_some())`
into two separate assertions: no error AND has hex output.

### 6. COMPLETE — `online_x86_assembler.rs` — mixed case hex test

Changed three-way `||` assertion to single `assert!(!body.contains("Error"))`.

### 7. COMPLETE — `trent.rs` — file too large error

Changed to check exact message: `"Sorry, maximum file size is 10MB."`.

### 8. COMPLETE — `trent.rs` — empty file rejection

Changed to check exact message substring: `"doesn't have enough lines"`.

## Medium: `.ok()` / `.unwrap_or_default()` hiding parse failures

### 9. COMPLETE — `hit_counter.rs` — `.ok()` swallows parse errors

Changed `.parse().ok()` to `.parse().unwrap_or_else(|e| panic!(...))` with
descriptive error messages showing the actual value that failed to parse.

### 10. COMPLETE — `trent.rs` — `.parse().ok()` inside `filter_map`

Changed `.filter_map(|cap| cap.get(1)?.as_str().parse().ok())` to
`.map()` with `.expect()` and `.unwrap_or_else()` so parse failures panic
with the actual failing value.

### 11. COMPLETE — `trent.rs` — `.unwrap_or_default()` hides extraction failure

Changed `.unwrap_or_default()` to `.expect()` for both `randlines1` and
`chosentwice` hidden inputs, matching the adjacent `.expect()` pattern.

### 12. COMPLETE — `source_security.rs` — `.unwrap_or("")` on headers

Changed both instances to `.expect()` so missing Content-Type and
Content-Disposition headers cause test failures instead of passing vacuously.

## Notable

### 13. COMPLETE — `url_edge_cases.rs` — contradictory test name and comment

Removed the incorrect comment claiming Rust returns 404 for double slashes.
The test correctly expects 200.

### 14. COMPLETE — `trent.rs` — known bug tested as correct behavior

Updated the BUG comment to indicate the bug was fixed. The test correctly
verifies FILE1/FILE2/FILE3 labels work.
