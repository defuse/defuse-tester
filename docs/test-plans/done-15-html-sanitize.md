# HTML Sanitize/Escape Tool Tests

**Status:** PLANNED (~120 test cases)

**File:** `tests/pages/html_sanitize.rs`

## Overview

The HTML escape tool converts text to HTML-safe output while preserving visual appearance. Key features:

- Tab-to-spaces conversion (cursor-position-aware)
- HTML entity escaping (`<`, `>`, `&`, `"`, `'`)
- Smart space handling (preserves whitespace appearance using `&nbsp;`)
- Line ending normalization with optional `<br />` tags
- Configurable tab width

The implementation is subtle - it's designed so that the HTML output **looks and behaves exactly** like the original text in a text editor.

## Test Cases

### HTML Entity Escaping

| Test | Input | Expected |
|------|-------|----------|
| `escape_less_than` | `<` | `&lt;` |
| `escape_greater_than` | `>` | `&gt;` |
| `escape_ampersand` | `&` | `&amp;` |
| `escape_double_quote` | `"` | `&quot;` |
| `escape_single_quote` | `'` | `&#039;` or `&apos;` |
| `escape_script_tag` | `<script>alert("xss")</script>` | `&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;` |
| `escape_html_tag` | `<div class="foo">` | `&lt;div class=&quot;foo&quot;&gt;` |
| `escape_existing_entity` | `&amp;` | `&amp;amp;` (double-escaped) |
| `escape_existing_lt` | `&lt;` | `&amp;lt;` |
| `escape_combined` | `<a href="x?a=1&b=2">` | Fully escaped |
| `escape_angle_brackets_text` | `1 < 2 && 2 > 1` | `1 &lt; 2 &amp;&amp; 2 &gt; 1` |

### Tab Conversion (Cursor Position Aware)

The tab key advances to the next tab stop. With tab width 8:
- Position 0 → tab adds 8 spaces (to position 8)
- Position 1 → tab adds 7 spaces (to position 8)
- Position 7 → tab adds 1 space (to position 8)
- Position 8 → tab adds 8 spaces (to position 16)

| Test | Input | Tab Width | Expected Spaces |
|------|-------|-----------|-----------------|
| `tab_at_start` | `\t` | 8 | 8 spaces |
| `tab_at_pos_1` | `a\t` | 8 | `a` + 7 spaces (total 8 chars) |
| `tab_at_pos_7` | `aaaaaaa\t` | 8 | 7 chars + 1 space = 8 |
| `tab_at_pos_8` | `aaaaaaaa\t` | 8 | 8 chars + 8 spaces = 16 |
| `tab_width_4_at_start` | `\t` | 4 | 4 spaces |
| `tab_width_4_at_pos_3` | `aaa\t` | 4 | 3 chars + 1 space = 4 |
| `tab_width_4_at_pos_4` | `aaaa\t` | 4 | 4 chars + 4 spaces = 8 |
| `tab_width_1` | `\t` | 1 | 1 space (minimum) |
| `tab_width_1_multiple` | `\t\t\t` | 1 | 3 spaces |
| `tab_width_2` | `a\t` | 2 | `a` + 1 space = 2 |
| `multiple_tabs` | `\t\t` | 8 | 16 spaces |
| `tabs_with_text` | `a\tb\tc` | 8 | `a` + 7 spaces + `b` + 7 spaces + `c` |
| `tab_after_newline` | `aaaa\n\t` | 8 | Cursor resets, tab = 8 spaces |
| `tab_after_cr` | `aaaa\r\t` | 8 | Cursor resets, tab = 8 spaces |
| `tab_after_crlf` | `aaaa\r\n\t` | 8 | Cursor resets, tab = 8 spaces |
| `tab_before_newline` | `aaa\t\n` | 8 | 3 chars + 5 spaces + newline |
| `tab_at_eol` | `\t\n\t\n` | 8 | Each line: 8 spaces + newline |
| `tab_only` | `\t` | 8 | 8 spaces |
| `tab_space_tab` | `\t \t` | 8 | 8 + 1 + 7 = 16 chars total |
| `space_tab` | ` \t` | 8 | 1 + 7 = 8 chars |
| `two_space_tab` | `  \t` | 8 | 2 + 6 = 8 chars |
| `three_space_tab` | `   \t` | 8 | 3 + 5 = 8 chars |
| `four_space_tab` | `    \t` | 8 | 4 + 4 = 8 chars |
| `five_space_tab` | `     \t` | 8 | 5 + 3 = 8 chars |
| `six_space_tab` | `      \t` | 8 | 6 + 2 = 8 chars |
| `seven_space_tab` | `       \t` | 8 | 7 + 1 = 8 chars |
| `eight_space_tab` | `        \t` | 8 | 8 + 8 = 16 chars |
| `tab_at_eof` | `aaa\t` | 8 | 3 chars + 5 spaces (no newline) |
| `consecutive_tabs_4` | `\t\t\t\t` | 8 | 32 spaces |
| `tab_width_3` | `a\t` | 3 | `a` + 2 spaces = 3 |
| `tab_width_3_at_3` | `aaa\t` | 3 | 3 + 3 = 6 |

### Space Handling (Critical for Visual Fidelity)

HTML collapses multiple spaces. The tool uses `&nbsp;` strategically to preserve spacing while allowing word wrap.

**Rules:**
1. Double spaces become ` &nbsp;` (space + nbsp) - allows line breaking
2. Leading space at line start becomes `&nbsp;` (HTML ignores leading spaces)
3. Trailing space before CR/LF becomes `&nbsp;` (preserves copy-paste)

| Test | Input | Expected | Notes |
|------|-------|----------|-------|
| `single_space` | `a b` | `a b` | Single space unchanged |
| `double_space` | `a  b` | `a &nbsp;b` | Space + nbsp |
| `triple_space` | `a   b` | `a &nbsp; b` | space+nbsp+space |
| `quad_space` | `a    b` | `a &nbsp; &nbsp;b` | Pattern repeats |
| `five_spaces` | `a     b` | `a &nbsp; &nbsp; b` | Alternating |
| `six_spaces` | `a      b` | `a &nbsp; &nbsp; &nbsp;b` | 6 = 3 pairs |
| `seven_spaces` | `a       b` | `a &nbsp; &nbsp; &nbsp; b` | 7 spaces |
| `eight_spaces` | `a        b` | `a &nbsp; &nbsp; &nbsp; &nbsp;b` | 8 = 4 pairs |
| `leading_space` | ` a` | `&nbsp;a` | Leading space → nbsp |
| `leading_two_spaces` | `  a` | `&nbsp;&nbsp;a` | Both become nbsp |
| `leading_three_spaces` | `   a` | `&nbsp;&nbsp; a` | nbsp+nbsp+space |
| `leading_four_spaces` | `    a` | `&nbsp;&nbsp; &nbsp;a` | 4 leading |
| `trailing_space_lf` | `a \n` | `a&nbsp;\n` | Trailing → nbsp |
| `trailing_space_cr` | `a \r` | `a&nbsp;\r` | Trailing → nbsp |
| `trailing_space_crlf` | `a \r\n` | `a&nbsp;\r\n` | Trailing → nbsp |
| `trailing_two_spaces_lf` | `a  \n` | `a &nbsp;\n` | Double space, then trailing handled |
| `trailing_three_spaces_lf` | `a   \n` | `a &nbsp;&nbsp;\n` | Triple, last becomes nbsp |
| `trailing_space_eof` | `a ` | `a ` | Trailing at EOF - NO nbsp (no \r\n follows) |
| `trailing_two_spaces_eof` | `a  ` | `a &nbsp;` | Double space pattern only |
| `trailing_three_spaces_eof` | `a   ` | `a &nbsp; ` | Pattern + trailing space |
| `space_only_line` | ` \n` | `&nbsp;\n` | Single space line |
| `two_space_only_line` | `  \n` | `&nbsp;&nbsp;\n` | Leading + trailing overlap |
| `multiple_space_line` | `   \n` | `&nbsp;&nbsp; \n` | 3 spaces on empty line |
| `leading_after_newline` | `a\n b` | `a\n&nbsp;b` | Leading on line 2 |
| `leading_after_crlf` | `a\r\n b` | `a\r\n&nbsp;b` | Leading after Windows newline |
| `leading_after_cr` | `a\r b` | `a\r&nbsp;b` | Leading after Mac newline |
| `word_wrap_preserved` | `word word` | `word word` | Single spaces allow wrap |
| `only_spaces` | `     ` | `&nbsp;&nbsp; &nbsp; ` | 5 spaces only |
| `space_between_newlines` | `\n \n` | `\n&nbsp;\n` | Space on otherwise empty line |

### Line Ending Handling (br=yes)

When `br=yes`, line endings are normalized and `<br />` tags are added.

**Order matters:**
1. `\r\n` → `\n` (Windows CRLF first)
2. `\r` → `\n` (Mac CR second - if done first, CRLF becomes `\n\n`)
3. `\n` → `<br />\n`

| Test | Input | br | Expected |
|------|-------|-----|----------|
| `br_unix_lf` | `a\nb` | yes | `a<br />\nb` |
| `br_windows_crlf` | `a\r\nb` | yes | `a<br />\nb` (NOT `<br />\n\nb`) |
| `br_mac_cr` | `a\rb` | yes | `a<br />\nb` |
| `br_multiple_lf` | `a\n\nb` | yes | `a<br />\n<br />\nb` |
| `br_multiple_crlf` | `a\r\n\r\nb` | yes | `a<br />\n<br />\nb` |
| `br_multiple_cr` | `a\r\rb` | yes | `a<br />\n<br />\nb` |
| `br_mixed_endings` | `a\nb\r\nc\rd` | yes | All become `<br />\n` |
| `br_reversed_crlf` | `a\n\rb` | yes | `a<br />\n<br />\nb` (LF then CR = 2 lines) |
| `br_crlf_then_lf` | `a\r\n\nb` | yes | `a<br />\n<br />\nb` |
| `br_lf_then_crlf` | `a\n\r\nb` | yes | `a<br />\n<br />\nb` |
| `br_trailing_newline` | `a\n` | yes | `a<br />\n` |
| `br_trailing_crlf` | `a\r\n` | yes | `a<br />\n` |
| `br_trailing_cr` | `a\r` | yes | `a<br />\n` |
| `br_leading_newline` | `\na` | yes | `<br />\na` |
| `br_leading_crlf` | `\r\na` | yes | `<br />\na` |
| `br_only_newlines` | `\n\n\n` | yes | `<br />\n<br />\n<br />\n` |
| `br_only_crlf` | `\r\n\r\n` | yes | `<br />\n<br />\n` |
| `br_only_cr` | `\r\r\r` | yes | `<br />\n<br />\n<br />\n` |
| `br_empty_lines` | `a\n\n\nb` | yes | Multiple `<br />` tags |
| `br_empty_input` | `` | yes | `` (empty, no br) |
| `br_just_text` | `abc` | yes | `abc` (no line endings, no br) |

### Line Ending Handling (br=no)

When `br=no`, line endings are preserved as-is (no normalization, no `<br />`).

| Test | Input | br | Expected |
|------|-------|-----|----------|
| `no_br_unix_lf` | `a\nb` | no | `a\nb` |
| `no_br_windows_crlf` | `a\r\nb` | no | `a\r\nb` |
| `no_br_mac_cr` | `a\rb` | no | `a\rb` |
| `no_br_mixed` | `a\nb\r\nc\rd` | no | `a\nb\r\nc\rd` (preserved) |
| `no_br_multiple_lf` | `a\n\n\nb` | no | `a\n\n\nb` |
| `no_br_multiple_crlf` | `a\r\n\r\nb` | no | `a\r\n\r\nb` |
| `no_br_trailing` | `a\n` | no | `a\n` |
| `no_br_leading` | `\na` | no | `\na` |
| `no_br_only_newlines` | `\n\n\n` | no | `\n\n\n` |
| `no_br_with_spaces` | `a \nb` | no | `a&nbsp;\nb` (trailing space still converted) |
| `no_br_leading_space` | `\n a` | no | `\n&nbsp;a` (leading space still converted) |

### Tab Width Validation

| Test | Input tw | Input data | Expected |
|------|----------|------------|----------|
| `tw_valid_1` | `1` | `\t` | 1 space |
| `tw_valid_8` | `8` | `\t` | 8 spaces |
| `tw_valid_large` | `100` | `\t` | 100 spaces |
| `tw_invalid_0` | `0` | `a` | Error: "Invalid tab width" |
| `tw_invalid_negative` | `-1` | `a` | Error: "Invalid tab width" |
| `tw_invalid_string` | `abc` | `a` | Error: "Invalid tab width" (parsed as 0) |
| `tw_invalid_empty` | `` | `a` | Error: "Invalid tab width" |
| `tw_invalid_float` | `4.5` | `a\t` | Truncated to 4? Or error? |
| `tw_invalid_space` | ` 8` | `\t` | Works as 8? Or error? |

### Combined/Complex Cases

| Test | Input | Expected |
|------|-------|----------|
| `tabs_and_spaces` | `\t  \t` | Tab (8) + double space (` &nbsp;`) + tab from pos 10 |
| `spaces_and_tabs` | `  \t  ` | ` &nbsp;` + tab from pos 2 + ` &nbsp;` |
| `tab_end_of_line` | `aaa\t\n` | 3 chars + 5 spaces + newline, trailing space handling |
| `html_in_code` | `<div>\n  text\n</div>` | Escaped tags, indentation preserved |
| `indented_code` | `if (x) {\n    y();\n}` | Indentation preserved with nbsp |
| `empty_string` | `` | `` (empty output) |
| `whitespace_only_spaces` | `   ` | `&nbsp;&nbsp; ` |
| `whitespace_only_tabs` | `\t\t` | 16 spaces converted to nbsp pattern |
| `whitespace_only_mixed` | ` \t ` | space + 7 spaces + space = nbsp pattern |
| `newline_only_lf` | `\n` | `<br />\n` (br=yes) or `\n` (br=no) |
| `newline_only_crlf` | `\r\n` | `<br />\n` (br=yes) or `\r\n` (br=no) |
| `tab_converted_to_nbsp` | `\t` | 8 spaces → `&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;` |
| `unicode_preserved` | `日本語 テスト` | `日本語 テスト` (unchanged, space between) |
| `unicode_double_space` | `日本語  テスト` | `日本語 &nbsp;テスト` |
| `emoji_preserved` | `Hello 👋 World` | `Hello 👋 World` |
| `emoji_with_spaces` | `👋  👋` | `👋 &nbsp;👋` |
| `very_long_line` | 1000 chars | Handles without truncation |
| `many_lines` | 100 lines | All lines processed |
| `special_and_spaces` | `<  >` | `&lt; &nbsp;&gt;` |
| `amp_and_spaces` | `&  &` | `&amp; &nbsp;&amp;` |
| `quote_and_spaces` | `"  "` | `&quot; &nbsp;&quot;` |
| `tab_then_special` | `\t<` | 8 spaces + `&lt;` |
| `special_then_tab` | `<\t` | `&lt;` + 7 spaces |
| `nbsp_in_input` | `&nbsp;` | `&amp;nbsp;` (& is escaped) |
| `br_in_input` | `<br />` | `&lt;br /&gt;` (escaped) |
| `all_special_chars` | `<>&"'` | `&lt;&gt;&amp;&quot;&#039;` |
| `mixed_everything` | `<a>\t  \n b` | Tabs, spaces, newlines, escaping all combined |

### Edge Cases for Tab Cursor Reset

The cursor position resets to 0 after each CR or LF (not after CRLF as a unit - each char resets).

| Test | Input | Tab Width | Expected | Notes |
|------|-------|-----------|----------|-------|
| `cursor_reset_lf` | `aaa\n\t` | 8 | 3 chars + LF + 8 spaces | After LF, cursor=0 |
| `cursor_reset_cr` | `aaa\r\t` | 8 | 3 chars + CR + 8 spaces | After CR, cursor=0 |
| `cursor_reset_crlf` | `aaa\r\n\t` | 8 | 3 chars + CRLF + 8 spaces | After CRLF, cursor=0 |
| `cursor_mid_line` | `aaa\naaa\t` | 8 | Line 2: `aaa` + 5 spaces | 3 chars + tab to 8 |
| `cursor_multiple_lines` | `a\naa\naaa\t` | 4 | Line 3: `aaa` + 1 space | 3 chars + tab to 4 |
| `cursor_crlf_counts_as_two` | `aaaa\r\n\t` | 4 | 4 + CRLF + 4 spaces | CR resets, \n resets again, tab=4 |
| `cursor_tab_doesnt_reset` | `\t\t` | 4 | 8 spaces | Tab advances cursor, doesn't reset |
| `cursor_after_spaces` | `  \n\t` | 4 | 2 spaces + LF + 4 spaces | Spaces advance cursor, LF resets |
| `cursor_long_line_then_tab` | `12345678\t` | 8 | 8 chars + 8 spaces | At position 8, tab adds 8 |
| `cursor_long_line_lf_tab` | `12345678\n\t` | 8 | 8 chars + LF + 8 spaces | Reset after LF |

### Page Structure and Form Handling

| Test | Description |
|------|-------------|
| `page_loads` | GET returns 200, contains form |
| `page_has_form_fields` | Has data textarea, tw input, br checkbox, submit |
| `page_default_tw` | Tab width defaults to 8 |
| `page_default_br_checked` | BR checkbox is checked by default |
| `page_shows_source` | Shows HtmlEscape.php source code |
| `form_submission_basic` | POST with data returns escaped output |
| `output_in_textarea` | Escaped text appears in textarea |
| `output_shows_escaped_message` | Shows "Your text has been escaped!" on POST |
| `get_shows_enter_text` | Shows "Enter Your Text:" on GET |
| `empty_data_submission` | POST with empty data field |
| `missing_data_field` | POST without data field at all |
| `br_checkbox_unchecked` | POST without br field (checkbox unchecked) |
| `br_checkbox_checked` | POST with br=yes |
| `preserves_input_in_value` | Original input shown in textarea value attribute |

### Escaping the Escaped Output (Double Escaping for Display)

The page displays the escaped text inside a textarea. The escaped output itself must be HTML-escaped for display in the HTML source. When we read the textarea value, we get the single-escaped version.

| Test | Input | Escaped Output | In HTML Source (double-escaped) |
|------|-------|----------------|--------------------------------|
| `meta_nbsp` | `  ` (2 spaces) | ` &nbsp;` | ` &amp;nbsp;` |
| `meta_br` | `\n` (br=yes) | `<br />\n` | `&lt;br /&gt;\n` |
| `meta_lt` | `<` | `&lt;` | `&amp;lt;` |
| `meta_gt` | `>` | `&gt;` | `&amp;gt;` |
| `meta_amp` | `&` | `&amp;` | `&amp;amp;` |
| `meta_quot` | `"` | `&quot;` | `&amp;quot;` |
| `meta_combined` | `<b>  </b>` | `&lt;b&gt; &nbsp;&lt;/b&gt;` | `&amp;lt;b&amp;gt; &amp;nbsp;&amp;lt;/b&amp;gt;` |
| `meta_script` | `<script>` | `&lt;script&gt;` | `&amp;lt;script&amp;gt;` |
| `meta_already_escaped` | `&lt;` | `&amp;lt;` | `&amp;amp;lt;` |

## Implementation Notes

- **URL:** `/html-sanitize.htm`
- **Method:** POST
- **Form fields:**
  - `data` - the text to escape
  - `tw` - tab width (integer, must be >= 1)
  - `br` - "yes" to add `<br />` tags (checkbox)
  - `sanitize` - submit button
- **Output location:** Inside `<textarea name="data">...</textarea>`
- **The output is double-escaped:** The escaped HTML is then HTML-escaped again for display in the textarea

## Processing Order

1. `tabsToSpaces()` - Convert tabs to spaces (cursor-position aware)
2. `htmlspecialchars()` - Escape HTML entities
3. `str_replace("  ", " &nbsp;")` - Handle double spaces
4. `preg_replace('/^\x20/m', "&nbsp;")` - Handle leading spaces
5. `preg_replace('/\x20(?=\r|\n)/m', "&nbsp;")` - Handle trailing spaces
6. (If br=yes) Normalize line endings and add `<br />`

### Processing Order Verification Tests

These tests verify the operations happen in the correct sequence.

| Test | Input | Verifies |
|------|-------|----------|
| `order_tab_before_escape` | `\t<` | Tab expanded first, then `<` escaped |
| `order_tab_before_nbsp` | `   \t` | **Critical:** Tab at pos 3 adds 5 spaces (total 8), THEN nbsp conversion. If nbsp happened first, `&nbsp;` would be 6 chars and tab calc would be wrong |
| `order_tab_before_nbsp_2` | `  \t` | Tab at pos 2 adds 6 spaces. Result: 8 spaces → `&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;` pattern |
| `order_escape_before_nbsp` | `<  ` | `<` escaped first, then spaces handled (not `&lt&nbsp;`) |
| `order_double_space_before_leading` | `  a` | Double space first → ` &nbsp;a`, then leading → `&nbsp;&nbsp;a` |
| `order_trailing_after_double` | `a  \n` | Double space → `a &nbsp;\n`, trailing space regex won't match |
| `order_br_last` | ` \n` | Leading space → `&nbsp;\n`, then br → `&nbsp;<br />\n` |
| `order_crlf_normalization` | `\r\n` | CRLF becomes single `<br />\n`, not `<br />\n<br />\n` |

## PHP Implementation Reference

- **Page handler:** `/defuse.ca/src/pages/services/html-sanitize.php`
- **Library:** `/defuse.ca/src/libs/HtmlEscape.php`
- **PHP function:** `htmlspecialchars($str, ENT_QUOTES)` for entity escaping
