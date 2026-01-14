# Upvote Tests

**File:** `tests/upvote.rs`

Test page: `/blind-birthday-attack.htm` (low traffic, unlikely to get real votes during testing)

## Test Cases

| Test | Description |
|------|-------------|
| `upvote_increases_count` | Clear vote state, get count, upvote, verify count increased by 1 |
| `downvote_decreases_count` | Clear vote state, upvote first (so we have room to go down), downvote, verify count decreased by 1 |
| `clear_vote_restores_count` | Clear vote, upvote, verify +1, upvote again (clears), verify back to original |
| `votes_only_affect_target_page` | Get count for page A and B, upvote A, verify A changed and B unchanged |

## Implementation Notes

- Create `tests/upvote.rs` with `mod common;` at top
- Always clear vote state at start of each test (vote same direction twice to clear)
- Use `common::post_form("/upvote", &[("upvotes_id", "..."), ("upvotes_direction", "...")])` for votes
- Parse XML response using `quick-xml` crate (already in Cargo.toml)
- The upvote endpoint returns XML with the current vote count and user's vote state
- Tests are stateful but self-contained (each test manages its own state)
