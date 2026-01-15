# Upvote Tests

**File:** `tests/upvote.rs`

Test page: `/blind-birthday-attack.htm` (low traffic, unlikely to get real votes during testing)

## Actual Behavior (verified on defuse.ca)

### State Transitions and Deltas

| From State | Action | To State | Delta |
|------------|--------|----------|-------|
| neutral | upvote | upvoted | +1 |
| neutral | downvote | downvoted | -1 |
| upvoted | upvote (same) | neutral | -1 |
| downvoted | downvote (same) | neutral | +1 |
| upvoted | downvote (switch) | downvoted | **-2** |
| downvoted | upvote (switch) | upvoted | **+2** |

**Key insight:** Switching from upvote to downvote (or vice versa) changes the count by **2**, not 1, because it removes the old vote AND adds the opposite vote.

### Two Endpoints

1. **`POST /upvote.php`** (AJAX/JavaScript endpoint)
   - Parameters: `upvotes_id` (page slug), `upvotes_direction` ("up" or "down")
   - Returns XML:
     ```xml
     <response>
       <status>pass</status>
       <uparrow>Y/N</uparrow>
       <downarrow>Y/N</downarrow>
       <total>N</total>
     </response>
     ```

2. **`POST /{page}.htm`** (non-JS form fallback)
   - Same parameters as above
   - Returns 302 redirect to same page
   - Follow redirect to get rendered HTML with updated vote state
   - Vote state visible in HTML:
     - Arrow images: `upvote-selected.gif` / `upvote.gif` and `downvote-selected.gif` / `downvote.gif`
     - Counter class: `upvotecount_upvoted` / `upvotecount_downvoted` / `upvotecount` (neutral)
     - Count value inside the counter div

## Test Cases

Each state transition must verify:
1. **Delta**: The vote count changes by the expected amount
2. **Button states**: `uparrow` and `downarrow` are Y/N as expected

### State Transition Tests

These tests apply to BOTH endpoints. For AJAX, check the XML response directly. For form POST, verify via a subsequent AJAX call.

| Test | Action | Expected Delta | Expected uparrow | Expected downarrow |
|------|--------|----------------|------------------|-------------------|
| `upvote_from_neutral` | neutral → upvote | +1 | Y | N |
| `downvote_from_neutral` | neutral → downvote | -1 | N | Y |
| `clear_upvote` | upvoted → upvote | -1 | N | N |
| `clear_downvote` | downvoted → downvote | +1 | N | N |
| `switch_upvote_to_downvote` | upvoted → downvote | -2 | N | Y |
| `switch_downvote_to_upvote` | downvoted → upvote | +2 | Y | N |

### AJAX Endpoint (`/upvote.php`)

| Test | Description |
|------|-------------|
| `ajax_upvote_from_neutral` | All checks from table above |
| `ajax_downvote_from_neutral` | All checks from table above |
| `ajax_clear_upvote` | All checks from table above |
| `ajax_clear_downvote` | All checks from table above |
| `ajax_switch_upvote_to_downvote` | All checks from table above |
| `ajax_switch_downvote_to_upvote` | All checks from table above |
| `ajax_returns_valid_xml` | Response is valid XML with status=pass, uparrow, downarrow, total |
| `ajax_votes_isolated_by_page` | Voting on page A doesn't affect page B count |

### Form Fallback Endpoint (`POST /{page}.htm`)

| Test | Description |
|------|-------------|
| `form_upvote_from_neutral` | Returns 302, redirects to same page, vote recorded with correct delta and button states |
| `form_downvote_from_neutral` | Returns 302, redirects to same page, vote recorded with correct delta and button states |
| `form_clear_upvote` | Returns 302, redirects to same page, vote cleared with correct delta and button states |
| `form_clear_downvote` | Returns 302, redirects to same page, vote cleared with correct delta and button states |
| `form_switch_upvote_to_downvote` | Returns 302, redirects to same page, vote switched with correct delta (-2) and button states |
| `form_switch_downvote_to_upvote` | Returns 302, redirects to same page, vote switched with correct delta (+2) and button states |
| `form_votes_isolated_by_page` | Voting on page A doesn't affect page B count |

## Implementation Notes

- Create `tests/upvote.rs` with `mod common;` at top
- Parameters: `upvotes_id` (e.g., "blind-birthday-attack"), `upvotes_direction` ("up" or "down")
- Use a low-traffic test page to minimize interference from real users

### Dependencies

Add to `Cargo.toml`:
```toml
[dev-dependencies]
serial_test = "3"
quick-xml = "0.31"
```

### Preventing Parallel Test Interference

Vote state is tracked by IP address, so parallel tests would interfere with each other. Use `serial_test` crate:

```rust
use serial_test::serial;

#[tokio::test]
#[serial]
async fn ajax_upvote_from_neutral() {
    // ...
}
```

All upvote tests must be marked `#[serial]` to run one at a time.

### Architecture

```rust
// === Core Types ===

struct VoteState {
    uparrow: bool,    // true = Y, false = N
    downarrow: bool,
    total: i32,
}

enum Direction { Up, Down }

// === Trait abstracts both endpoint types ===

trait VoteClient {
    async fn vote(&self, page: &str, dir: Direction) -> VoteState;
}

struct AjaxClient(Client);   // POST /upvote.php, parse XML
struct FormClient(Client);   // POST /{page}.htm, follow redirect, parse HTML
```

### Resetting to Neutral State

Tests cannot assume the current vote state. Reset to neutral before each test:

```rust
/// Reset to neutral state, return the baseline total
async fn reset_to_neutral(client: &impl VoteClient, page: &str) -> i32 {
    // Vote up - results in either upvoted (uparrow=Y) or neutral
    let state = client.vote(page, Direction::Up).await;

    if state.uparrow {
        // We're upvoted, vote up again to clear to neutral
        let state = client.vote(page, Direction::Up).await;
        state.total
    } else {
        // Already neutral
        state.total
    }
}
```

This handles any starting state in at most 2 requests.

### Test Flow

```rust
async fn run_vote_test(
    client: &impl VoteClient,
    page: &str,
    starting_state: Option<Direction>,  // None = neutral
    action: Direction,
    expected_delta: i32,
    expected_up: bool,
    expected_down: bool,
) {
    // 1. Reset to neutral, get baseline total
    let baseline = reset_to_neutral(client, page).await;

    // 2. Set up starting state if needed (and track setup delta)
    let setup_delta = if let Some(dir) = starting_state {
        client.vote(page, dir).await;
        if dir == Direction::Up { 1 } else { -1 }
    } else {
        0
    };

    // 3. Perform the action being tested
    let result = client.vote(page, action).await;

    // 4. Verify delta and button states
    assert_eq!(result.total, baseline + setup_delta + expected_delta);
    assert_eq!(result.uparrow, expected_up);
    assert_eq!(result.downarrow, expected_down);

    // 5. Clean up: reset back to neutral for next test
    reset_to_neutral(client, page).await;
}
```

### Individual Tests (One-Liners)

```rust
#[tokio::test]
#[serial]
async fn ajax_upvote_from_neutral() {
    run_vote_test(&AjaxClient::new(), PAGE, None, Up, 1, true, false).await;
}

#[tokio::test]
#[serial]
async fn ajax_switch_upvote_to_downvote() {
    run_vote_test(&AjaxClient::new(), PAGE, Some(Up), Down, -2, false, true).await;
}

#[tokio::test]
#[serial]
async fn form_upvote_from_neutral() {
    run_vote_test(&FormClient::new(), PAGE, None, Up, 1, true, false).await;
}
```

### Verifying AJAX Endpoint Responses

- Parse XML response using `quick-xml` crate
- Check `<status>pass</status>`, `<uparrow>Y/N</uparrow>`, `<downarrow>Y/N</downarrow>`, `<total>N</total>`

### Verifying Form POST Responses

- POST returns 302 redirect
- Follow redirect (or make separate GET) to fetch rendered page
- Check for `upvote-selected.gif` or `upvote.gif` in the HTML to determine up arrow state
- Check for `downvote-selected.gif` or `downvote.gif` to determine down arrow state
- Check counter div class: `upvotecount_upvoted`, `upvotecount_downvoted`, or `upvotecount`
- Extract count value from inside the counter div
