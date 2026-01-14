# Panic Recovery Tests

**File:** `tests/panic_recovery.rs`

## Test Cases

| Test | Description |
|------|-------------|
| `server_survives_panic` | Hit `/panic-test.htm`, then hit `/about.htm`, verify about returns 200 |

## Implementation Notes

- Create `tests/panic_recovery.rs` with `mod common;` at top
- The `/panic-test.htm` page intentionally panics/crashes the handler
- Server should return 500 for the panic page but keep running
- The subsequent request to `/about.htm` must succeed with 200
- Use `common::assert_success(response, context)` for the recovery check
- This tests that the server doesn't crash entirely when a single request fails
- May need to handle connection errors gracefully on the first request
