# Hit Counter Tests

**File:** `tests/hit_counter.rs`

Test page: `/blind-birthday-attack.htm`

## Test Cases

| Test | Description |
|------|-------------|
| `hit_increments_non_unique_only` | Hit the page, extract hit counts from footer, hit again, verify non-unique increased but unique did not |
| `hit_counts_visible_in_footer` | Hit page, verify footer contains hit count display (implied by above but explicit) |

## Implementation Notes

- Create `tests/hit_counter.rs` with `mod common;` at top
- Parse footer HTML to extract hit counts using `scraper` crate (already in Cargo.toml)
- The same IP hitting twice should increment non-unique but not unique (within 30-day window)
- Look for elements containing hit count data in the page footer
