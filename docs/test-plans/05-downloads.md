# Downloads/Files Tests

**File:** `tests/downloads.rs`

Tests that downloadable files exist and are served.

## Test Cases

### /downloads/ (in git repo)

| Test | Description |
|------|-------------|
| `winpassgen_exe` | `/downloads/WinPassGen.exe` returns 200 |
| `thesis_pdf` | `/downloads/thesis.pdf` returns 200 |
| `sockstress_txt` | `/downloads/sockstress.txt` returns 200 |
| `taylor_asc` | `/downloads/taylor.asc` returns 200 |
| `poc_zip` | `/downloads/poc.zip` returns 200 |
| `gocryptfs_audit` | `/downloads/audits/gocryptfs-cryptography-design-audit.pdf` returns 200 |
| `bh2016_whitepaper` | `/downloads/bh2016-whitepaper.pdf` returns 200 |

### /files2/ (large files, not in git repo - may fail locally)

| Test | Description |
|------|-------------|
| `pocorgtfo_00` | `/files2/poc/pocorgtfo00.pdf` returns 200 |
| `pocorgtfo_10` | `/files2/poc/pocorgtfo10.pdf` returns 200 (spot check middle) |
| `pocorgtfo_21` | `/files2/poc/pocorgtfo21.pdf` returns 200 (spot check last) |
| `insfilter_pdf` | `/files2/insfilter.pdf` returns 200 |

## Implementation Notes

- Create `tests/downloads.rs` with `mod common;` at top
- These are simple existence checks (HEAD or GET, check for 200)
- Use `common::assert_success(response, context)` for each file
- Files were found by grepping the PHP source code
- No need to verify file contents, just that they're served
- The `/files2/` directory contains large files not stored in git; these tests will fail locally but pass against production

## Content-Disposition Behavior

The two directories have different Content-Disposition behavior:

- `/downloads/` - NO Content-Disposition header (PDFs render inline in browser)
- `/files2/` - Has `Content-Disposition: attachment` (forces browser to download)

This distinction is tested in `tests/content_types.rs`.
