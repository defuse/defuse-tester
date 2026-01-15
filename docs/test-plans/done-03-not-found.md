# 404 Page Tests

**File:** `tests/not_found.rs`

## Test Cases

| Test | Description |
|------|-------------|
| `returns_404_status` | `/nonexistent-page-12345.htm` returns HTTP 404 |
| `has_expected_content` | 404 page contains "not found" or similar text (case-insensitive) |
| `has_navigation` | 404 page still has site navigation (not a bare error) |

## Implementation Notes

- Create `tests/not_found.rs` with `mod common;` at top
- Use a clearly nonexistent path like `/nonexistent-page-12345.htm`
- Use `common::assert_status(response, 404, context)` for status check
- Use `common::assert_body_contains(body, needle, context)` for content checks
- For navigation check, look for the main menu element (e.g., `id="menu"` or similar)
- The 404 page should be a proper HTML page, not just a plain text error
