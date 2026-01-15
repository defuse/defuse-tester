# Content Snapshot Binary

**Binary:** `src/bin/snapshot.rs`

A standalone tool for capturing page snapshots from any server, enabling manual diff-based comparison between implementations.

## Usage

```bash
# Capture snapshots from production
cargo run --bin snapshot -- https://defuse.ca/ ./snapshots-prod

# Capture snapshots from local dev server
cargo run --bin snapshot -- http://localhost:3000/ ./snapshots-local

# Compare the two
diff -r ./snapshots-prod ./snapshots-local
# or use meld, vimdiff, etc.
```

## Behavior

1. Reads `data/pages.json` (exported from PHP's `URLParse::$PAGE_INFO`)
2. Filters to real pages only (has `file` key, no `redirect` key)
3. Fetches each page in deterministic order (sorted by slug)
4. Saves full HTML response to `{output_dir}/{slug}.html`
   - Home page (empty slug) saves as `_index.html`

## Dynamic Content

Some pages have dynamic content that will always differ:
- Hit counters
- Upvote counts
- Timestamps

These differences are expected and can be filtered when reviewing diffs:
```bash
diff -r ./snapshots-prod ./snapshots-local | grep -v 'hit-counter\|upvote'
```

## File Structure

```
snapshots-prod/
  _index.html        # Home page
  about.html
  checksums.html
  contact.html
  ...
```
