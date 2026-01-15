# Static Files Tests (Additions)

**File:** `tests/static_files.rs` (already exists, add these tests)

## Additional Test Cases

| Test | Description |
|------|-------------|
| `google_verification` | `/googlec56659c80ebb2d30.html` exists (200) |
| `hibp_verification` | `/have-i-been-pwned-verification.txt` exists (200) |
| `longcat_easter_egg` | `/longcat.html` exists (200) |
| `unknown_verification_file` | `/b06c1611fa2dac4cc177a7a63c70902c453cdb92.html` exists (200) - unknown purpose, possibly site verification |

## Implementation Notes

- Add these to the existing `tests/static_files.rs` file
- Same pattern as other static file tests: GET, `assert_success`
- The SHA1-named file's purpose is unknown but may be important for some verification service
