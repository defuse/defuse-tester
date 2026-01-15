# Automated Content Comparison Tests

**File:** `tests/content_snapshot.rs` and `tests/content_snapshot/`

This is a separate test module that compares page content against snapshots from defuse.ca.

## Prerequisites

**Add `--export-pages` flag to defuse-rust** that outputs JSON of all registry pages:
```json
[
  {"slug": "", "title": "..."},
  {"slug": "about", "title": "About - Defuse Security"},
  {"slug": "contact", "title": "..."},
  ...
]
```

## Setup Script (one-time, or refresh)

1. Run `defuse-rust --export-pages > pages.json`
2. For each page, fetch from defuse.ca, extract `<div id="content">...</div>`
3. Save to `tests/content_snapshot/snapshots/{slug}.html` (use `_index.html` for home page)

## Test Cases

| Test | Description |
|------|-------------|
| `content_matches_snapshot` | For each snapshot file, fetch page from target, extract content div, compare exactly |

## Expected Failures

Pages with dynamic content:
- Home page (vote counts)
- Pages with hit counters in content

## Directory Structure

```
tests/
  content_snapshot.rs        # mod declarations
  content_snapshot/
    mod.rs                   # Snapshot test logic
    snapshots/               # HTML snapshot files (generated)
      _index.html            # Home page (slug "")
      about.html
      contact.html
      ...
```

## Implementation Notes

- Create `tests/content_snapshot.rs` with `mod common;` at top
- Create `tests/content_snapshot/` directory for snapshot files
- This is separate from `tests/pages/` which is for service-specific tests
- Snapshots are generated from production defuse.ca
- Tests compare extracted content div using `scraper` crate, not full page HTML
- Useful for catching unintended content changes during Rust port
