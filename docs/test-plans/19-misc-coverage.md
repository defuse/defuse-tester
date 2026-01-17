# Miscellaneous Coverage Test Plan

**Status:** DONE (19 tests across 5 files)
**Files:** `tests/breach.rs`, `tests/vim_highlighting.rs`, `tests/bibliography.rs`, `tests/hit_counter.rs`, `tests/special_endpoints.rs`

## Overview

This test plan covers several features that were identified as missing test coverage:
- **CRITICAL: /source/ directory PHP execution vulnerability**
- BREACH attack mitigation HTML output
- VIM syntax highlighting
- Bibliography/citation system
- Bot detection for hit counters
- getmyip.php endpoint

Note: Shout tests already exist in `special_endpoints.rs` (ignored pending Rust implementation).
Vote switching tests already exist in `upvote.rs`.

## 1. Source Directory Security (CRITICAL)

**Directory:** `/source/`

The `/source/` directory is meant to serve PHP source code files for download/viewing, NOT execute them. Currently on defuse.ca, PHP files in this directory are being **executed** instead of served as raw text.

This is a **critical security vulnerability** because `pdfcleaner.php` contains system command execution (`pdf2ps`, `ps2pdf`, `pdftotext`) with user-uploaded files, and the old Debian PDF parsers are likely vulnerable.

| Test | Description | Expected |
|------|-------------|----------|
| `source_php_not_executed` | GET `/source/pdfcleaner.php` | Content-Type should be `text/plain` or `application/octet-stream`, NOT `text/html` |
| `source_php_shows_raw_code` | GET `/source/pdfcleaner.php` | Response body should contain `<?php` (raw PHP source) |
| `source_breach_not_executed` | GET `/source/breach.php` | Should contain `<?php` and `function breach_` |
| `source_files_have_attachment_disposition` | Check Content-Disposition | Should trigger download, not inline display |

### Implementation Notes

On defuse.ca currently:
- `/source/pdfcleaner.php` returns `"Not a PDF file!"` (executed PHP output)
- `/source/breach.php` returns empty (executed, but no output since it only defines functions)
- Both return `Content-Type: text/html` (wrong!)

The Rust implementation MUST serve these as raw source code, not execute them.

**This test is expected to FAIL on defuse.ca** - that's the point, it catches the vulnerability.

## 2. BREACH Attack Mitigation

**Page:** `/mitigating-breach-tls-attack-in-php.htm`

The `breach_visual_html()` function outputs text that appears normal but contains random HTML comments and zero-width spaces to defeat compression-based attacks.

| Test | Description | Expected |
|------|-------------|----------|
| `breach_visual_html_output_is_protected` | GET page, extract BREACH section, verify encoding | See implementation notes |

### Implementation Notes

The BREACH section is inside `<div style="padding-left: 30px;">` containing an `<h2>` and `<p>`. Extract that section and verify ALL of:
1. Contains the visible text characters S, a, m, p, l, e, H, e, a, d, e, r (interspersed)
2. Contains HTML comments with hex content like `<!-- [0-9a-f]+ -->`
3. Contains zero-width space entities (`&#8203;` or `&#x200b;`)
4. The text "Sample Header" is reconstructable from the visible characters

The output is randomized on each page load, so we check structure, not exact content.

## 3. VIM Syntax Highlighting

**Page:** `/syntax-highlighting-in-php-with-vim.htm`

This page demonstrates VIM-based syntax highlighting and contains highlighted code samples.

| Test | Description | Expected |
|------|-------------|----------|
| `vim_highlight_page_loads` | GET the page | 200 OK |
| `vim_highlight_has_code_section` | Check for highlighted code container | Contains `<div class="vimhighlight">` or similar |
| `vim_highlight_has_function_keyword` | Check for PHP function keyword | Contains "function" within highlighted section |
| `vim_highlight_has_method_name` | Check for recognizable method name | Contains "vimhl_convert" or "vimhl_exec" (actual function names in the code) |

### Implementation Notes

VIM versions may produce different span structures, so we check for content that must be present regardless of highlighting markup. The functions `vimhl_convert` and `vimhl_exec` are defined in the code samples on that page.

## 4. Bibliography/Citation System

**Page:** `/flush-reload-side-channel.htm`

This page uses the Bibliography system for academic citations.

| Test | Description | Expected |
|------|-------------|----------|
| `bibliography_page_loads` | GET the page | 200 OK |
| `bibliography_has_inline_citation` | Check for citation markers | Contains `[1]` as a link or superscript |
| `bibliography_has_references_section` | Check for references at bottom | Contains "References" heading |
| `bibliography_citation_links_to_reference` | Check citation is a link | Contains `href="#cite_1"` or similar anchor link |
| `bibliography_reference_has_author` | Check reference entry format | References section contains author name "Yarom" |

### Implementation Notes

Citations appear as `<sup><a href="#cite_X">[X]</a></sup>` inline.
References appear at the bottom with anchors like `<a name="cite_X">` or `id="cite_X"`.

## 5. Bot Detection (IGNORE_SEARCH_BOTS)

**Mechanism:** The hit counter in `phpcount.php` checks for search bots via `IsSearchBot()` which looks for keywords like "bot", "crawl", "spider" in the User-Agent header.

| Test | Description | Expected |
|------|-------------|----------|
| `hit_counter_normal_user` | Request page with normal User-Agent | Hit counter increments (or at least page works) |
| `hit_counter_googlebot` | Request with User-Agent containing "Googlebot" | Page loads, counter should NOT increment |
| `hit_counter_generic_bot` | Request with User-Agent containing "bot" | Page loads, counter should NOT increment |

### Implementation Notes

Testing whether the counter actually increments is tricky without database access. Alternative approach:
- Check that the page loads successfully with bot User-Agent (doesn't error)
- If there's any visible counter, it shouldn't change for bot requests

For minimal testing, just verify:
1. Pages load with normal User-Agent
2. Pages load with bot User-Agent (no errors from bot detection code path)

## 6. getmyip.php

**Endpoint:** `/getmyip.php`

Returns an HTML page showing the visitor's IP address, hostname, and User-Agent.

| Test | Description | Expected |
|------|-------------|----------|
| `getmyip_returns_200` | GET /getmyip.php | 200 OK |
| `getmyip_content_type_html` | Check Content-Type | text/html |
| `getmyip_shows_ip` | Check for IP display | Contains an IP address pattern (e.g., `\d+\.\d+\.\d+\.\d+` or IPv6) |
| `getmyip_shows_hostname` | Check for hostname section | Contains "Hostname:" or similar label |
| `getmyip_shows_user_agent` | Check for UA display | Contains the User-Agent we sent |
| `getmyip_custom_user_agent` | Send custom UA, verify displayed | Page contains our custom UA string |

### Implementation Notes

Send a distinctive User-Agent like "DefuseTestSuite/1.0" and verify it appears in the response.

## Already Covered (No New Tests Needed)

### Shout (/s.php)

Tests exist in `tests/special_endpoints.rs`:
- `shout_no_params_shows_form`
- `shout_e_param_redirects_to_s_param`
- `shout_s_param_displays_decoded_text`

Currently ignored with `#[ignore = "s.php not yet implemented in Rust"]`.

### Vote Switching

Tests exist in `tests/upvote.rs`:
- `ajax_switch_upvote_to_downvote`
- `ajax_switch_downvote_to_upvote`
- `form_switch_upvote_to_downvote`
- `form_switch_downvote_to_upvote`

## Test Count Summary

| Section | New Tests |
|---------|-----------|
| Source Directory Security (CRITICAL) | 4 (done) |
| BREACH Mitigation | 1 |
| VIM Highlighting | 4 |
| Bibliography | 5 |
| Bot Detection | 3 |
| getmyip.php | 6 |
| **Total** | **23** |

## Dependencies

- reqwest with headers support (already have)
- Regex for IP pattern matching (may need to add)
