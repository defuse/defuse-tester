mod common;
// Tests for /html-sanitize.htm
//
// Verifies:
// - Tab-to-spaces conversion (cursor-position-aware)
// - HTML entity escaping (<, >, &, ", ')
// - Smart space handling (preserves whitespace using &nbsp;)
// - Line ending normalization with optional <br /> tags
// - Configurable tab width

use common::{client, url};

// =============================================================================
// Helper Functions
// =============================================================================

/// Escape text with default settings (tw=8, br=yes)
async fn escape(input: &str) -> String {
    escape_with_options(input, 8, true).await
}

/// Escape text with custom tab width, br=yes
async fn escape_tw(input: &str, tw: u32) -> String {
    escape_with_options(input, tw, true).await
}

/// Escape text with br=no
async fn escape_no_br(input: &str) -> String {
    escape_with_options(input, 8, false).await
}

/// Escape text with all options
async fn escape_with_options(input: &str, tw: u32, br: bool) -> String {
    let client = client();
    let tw_string = tw.to_string();

    let mut form = vec![
        ("data", input),
        ("tw", tw_string.as_str()),
        ("sanitize", "HTML-Escape Text"),
    ];

    if br {
        form.push(("br", "yes"));
    }

    let resp = client
        .post(url("/html-sanitize.htm"))
        .form(&form)
        .send()
        .await
        .expect("Failed to send request");

    assert!(resp.status().is_success(), "Request failed: {}", resp.status());

    let body = resp.text().await.expect("Failed to read response");
    extract_textarea_value(&body)
}

/// Escape with string tab width (for testing invalid values)
async fn escape_with_tw_string(input: &str, tw: &str, br: bool) -> Result<String, String> {
    let client = client();

    let mut form = vec![
        ("data", input),
        ("tw", tw),
        ("sanitize", "HTML-Escape Text"),
    ];

    if br {
        form.push(("br", "yes"));
    }

    let resp = client
        .post(url("/html-sanitize.htm"))
        .form(&form)
        .send()
        .await
        .expect("Failed to send request");

    assert!(resp.status().is_success(), "Request failed: {}", resp.status());

    let body = resp.text().await.expect("Failed to read response");
    let value = extract_textarea_value(&body);

    if value.contains("Invalid tab width") {
        Err("Invalid tab width".to_string())
    } else {
        Ok(value)
    }
}

/// Extract the value from the textarea.
/// The textarea contains the escaped output, which is itself HTML-escaped for display.
/// We need to unescape the HTML entities to get the actual escaped output.
fn extract_textarea_value(html: &str) -> String {
    // Find <textarea name="data" ...>...</textarea>
    let start_marker = r#"<textarea name="data""#;
    let Some(textarea_start) = html.find(start_marker) else {
        panic!("No textarea found in response");
    };

    // Find the end of the opening tag
    let after_start = &html[textarea_start..];
    let Some(content_start) = after_start.find('>') else {
        panic!("Malformed textarea tag");
    };

    let content = &after_start[content_start + 1..];
    let Some(content_end) = content.find("</textarea>") else {
        panic!("No closing textarea tag");
    };

    let raw_value = &content[..content_end];

    // Strip leading newline that's part of HTML formatting (not content)
    let raw_value = raw_value.strip_prefix('\n').unwrap_or(raw_value);

    // The value is HTML-escaped for display in the textarea, so unescape it
    html_unescape(raw_value)
}

/// Unescape HTML entities from textarea display.
///
/// Order matters! We must unescape &lt;, &gt;, etc. BEFORE &amp;.
/// Otherwise double-escaped entities like &amp;lt; get incorrectly converted:
/// - Wrong order: &amp;lt; → &lt; → < (double conversion!)
/// - Correct order: &amp;lt; stays (no &lt; match), then &amp; → & gives &lt; (correct!)
///
/// Note: We can't use scraper/HTML parser here because it normalizes line endings.
fn html_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
     .replace("&gt;", ">")
     .replace("&quot;", "\"")
     .replace("&#039;", "'")
     .replace("&amp;", "&")
}

/// Assert that escaping produces exact expected output (tw=8, br=yes)
async fn assert_escape(input: &str, expected: &str) {
    let result = escape(input).await;
    assert_eq!(result, expected, "\n  input: {:?}\n  expected: {:?}\n  got: {:?}", input, expected, result);
}

/// Assert escape with custom tab width (br=yes)
async fn assert_escape_tw(input: &str, tw: u32, expected: &str) {
    let result = escape_tw(input, tw).await;
    assert_eq!(result, expected, "\n  input: {:?}, tw={}\n  expected: {:?}\n  got: {:?}", input, tw, expected, result);
}

/// Assert escape with br=no (tw=8)
async fn assert_escape_no_br(input: &str, expected: &str) {
    let result = escape_no_br(input).await;
    assert_eq!(result, expected, "\n  input: {:?} (br=no)\n  expected: {:?}\n  got: {:?}", input, expected, result);
}

// =============================================================================
// Section 1: HTML Entity Escaping
// =============================================================================

#[tokio::test]
async fn escape_less_than() {
    assert_escape("<", "&lt;").await;
}

#[tokio::test]
async fn escape_greater_than() {
    assert_escape(">", "&gt;").await;
}

#[tokio::test]
async fn escape_ampersand() {
    assert_escape("&", "&amp;").await;
}

#[tokio::test]
async fn escape_double_quote() {
    assert_escape("\"", "&quot;").await;
}

#[tokio::test]
async fn escape_single_quote() {
    // PHP htmlspecialchars with ENT_QUOTES uses &#039;
    assert_escape("'", "&#039;").await;
}

#[tokio::test]
async fn escape_script_tag() {
    assert_escape(
        "<script>alert(\"xss\")</script>",
        "&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;"
    ).await;
}

#[tokio::test]
async fn escape_html_tag() {
    assert_escape(
        "<div class=\"foo\">",
        "&lt;div class=&quot;foo&quot;&gt;"
    ).await;
}

#[tokio::test]
async fn escape_existing_entity() {
    // &amp; in input becomes &amp;amp; (the & is escaped)
    assert_escape("&amp;", "&amp;amp;").await;
}

#[tokio::test]
async fn escape_existing_lt() {
    assert_escape("&lt;", "&amp;lt;").await;
}

#[tokio::test]
async fn escape_combined() {
    assert_escape(
        "<a href=\"x?a=1&b=2\">",
        "&lt;a href=&quot;x?a=1&amp;b=2&quot;&gt;"
    ).await;
}

#[tokio::test]
async fn escape_angle_brackets_text() {
    assert_escape("1 < 2 && 2 > 1", "1 &lt; 2 &amp;&amp; 2 &gt; 1").await;
}

#[tokio::test]
async fn escape_all_special_chars() {
    assert_escape("<>&\"'", "&lt;&gt;&amp;&quot;&#039;").await;
}

// =============================================================================
// Section 2: Tab Conversion (Cursor Position Aware)
// =============================================================================

// Tab at start with tw=8 → 8 spaces → "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;"
#[tokio::test]
async fn tab_at_start() {
    assert_escape_tw("\t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// 'a' + tab: 1 char + 7 spaces = 8 visual
// "a       " (7 spaces) → 3 pairs + 1 trailing = "a &nbsp; &nbsp; &nbsp; "
#[tokio::test]
async fn tab_at_pos_1() {
    assert_escape_tw("a\t", 8, "a &nbsp; &nbsp; &nbsp; ").await;
}

// 7 chars + 1 space = 8 visual
// "aaaaaaa " → "aaaaaaa "
#[tokio::test]
async fn tab_at_pos_7() {
    assert_escape_tw("aaaaaaa\t", 8, "aaaaaaa ").await;
}

// 8 chars + 8 spaces = 16 visual
// "aaaaaaaa        " → "aaaaaaaa &nbsp; &nbsp; &nbsp; &nbsp;"
#[tokio::test]
async fn tab_at_pos_8() {
    assert_escape_tw("aaaaaaaa\t", 8, "aaaaaaaa &nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// Tab at start with tw=4 → 4 spaces → "&nbsp;&nbsp; &nbsp;"
#[tokio::test]
async fn tab_width_4_at_start() {
    assert_escape_tw("\t", 4, "&nbsp;&nbsp; &nbsp;").await;
}

// 3 chars + 1 space = 4 visual
// "aaa " → "aaa "
#[tokio::test]
async fn tab_width_4_at_pos_3() {
    assert_escape_tw("aaa\t", 4, "aaa ").await;
}

// 4 chars + 4 spaces = 8 visual
// "aaaa    " → "aaaa &nbsp; &nbsp;"
#[tokio::test]
async fn tab_width_4_at_pos_4() {
    assert_escape_tw("aaaa\t", 4, "aaaa &nbsp; &nbsp;").await;
}

// Tab with tw=1 → 1 space (no double space pattern needed)
#[tokio::test]
async fn tab_width_1() {
    assert_escape_tw("\t", 1, "&nbsp;").await;
}

// 3 tabs with tw=1 → 3 spaces → "&nbsp;&nbsp; "
#[tokio::test]
async fn tab_width_1_multiple() {
    assert_escape_tw("\t\t\t", 1, "&nbsp;&nbsp; ").await;
}

// 'a' + tab with tw=2 → 'a' + 1 space = 2 visual
// "a " → "a "
#[tokio::test]
async fn tab_width_2() {
    assert_escape_tw("a\t", 2, "a ").await;
}

// Two tabs with tw=8 → 16 spaces
// "                " → "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;"
#[tokio::test]
async fn multiple_tabs() {
    assert_escape_tw("\t\t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// a + 7 spaces + b + 7 spaces + c
// Each set of 7 spaces = 3 pairs + 1 trailing space
// "a       b       c" → "a &nbsp; &nbsp; &nbsp; b &nbsp; &nbsp; &nbsp; c"
#[tokio::test]
async fn tabs_with_text() {
    assert_escape_tw("a\tb\tc", 8, "a &nbsp; &nbsp; &nbsp; b &nbsp; &nbsp; &nbsp; c").await;
}

// After newline, cursor resets. Tab at start of new line = 8 spaces.
// With br=yes: "aaaa\n" becomes "aaaa<br />\n" then tab on new line
#[tokio::test]
async fn tab_after_newline() {
    assert_escape_tw("aaaa\n\t", 8, "aaaa<br />\n&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// After CR (with br=yes, \r → <br />\n), cursor resets to 0, tab = 8 spaces
// NOTE: Leading space regex /^\x20/m only matches after \n, not after \r
// So the 8 spaces after \r don't get leading space conversion
#[tokio::test]
async fn tab_after_cr() {
    assert_escape_tw("aaaa\r\t", 8, "aaaa<br />\n &nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// After CRLF, cursor resets
#[tokio::test]
async fn tab_after_crlf() {
    assert_escape_tw("aaaa\r\n\t", 8, "aaaa<br />\n&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// "aaa" + tab (5 spaces to reach 8) + newline
// 5 spaces = 2 pairs + 1 trailing, trailing space before \n becomes &nbsp;
// "aaa     \n" → "aaa &nbsp; &nbsp;&nbsp;<br />\n"
#[tokio::test]
async fn tab_before_newline() {
    assert_escape_tw("aaa\t\n", 8, "aaa &nbsp; &nbsp;&nbsp;<br />\n").await;
}

// Two lines, each with tab only
#[tokio::test]
async fn tab_at_eol() {
    assert_escape_tw("\t\n\t\n", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;<br />\n&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;<br />\n").await;
}

// Single tab
#[tokio::test]
async fn tab_only() {
    assert_escape_tw("\t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// tab (8) + space (1) + tab from pos 9 → 7 more spaces to reach 16
// Total 16 spaces: "         " (9 spaces) becomes... wait let me recalculate
// tab=8 spaces, then 1 space (pos 8 to 9), then tab from pos 9 goes to 16 (7 more)
// 8 + 1 + 7 = 16 spaces total
#[tokio::test]
async fn tab_space_tab() {
    assert_escape_tw("\t \t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// 1 space + tab from pos 1 → 7 more spaces = 8 total
// "        " → "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;"
#[tokio::test]
async fn space_tab() {
    assert_escape_tw(" \t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// 2 spaces + tab from pos 2 → 6 more = 8 total
#[tokio::test]
async fn two_space_tab() {
    assert_escape_tw("  \t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// 3 spaces + tab from pos 3 → 5 more = 8 total
#[tokio::test]
async fn three_space_tab() {
    assert_escape_tw("   \t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// 4 spaces + tab from pos 4 → 4 more = 8 total
#[tokio::test]
async fn four_space_tab() {
    assert_escape_tw("    \t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// 5 spaces + tab from pos 5 → 3 more = 8 total
#[tokio::test]
async fn five_space_tab() {
    assert_escape_tw("     \t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// 6 spaces + tab from pos 6 → 2 more = 8 total
#[tokio::test]
async fn six_space_tab() {
    assert_escape_tw("      \t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// 7 spaces + tab from pos 7 → 1 more = 8 total
#[tokio::test]
async fn seven_space_tab() {
    assert_escape_tw("       \t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// 8 spaces + tab from pos 8 → 8 more = 16 total
#[tokio::test]
async fn eight_space_tab() {
    assert_escape_tw("        \t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// "aaa" + tab (5 spaces) = 8 chars
// "aaa     " (5 spaces) → 2 pairs + 1 trailing = "aaa &nbsp; &nbsp; "
#[tokio::test]
async fn tab_at_eof() {
    assert_escape_tw("aaa\t", 8, "aaa &nbsp; &nbsp; ").await;
}

// 4 consecutive tabs with tw=8 → 32 spaces
#[tokio::test]
async fn consecutive_tabs_4() {
    // 32 spaces = 16 pairs = 16 " &nbsp;" = " &nbsp;" * 16, with leading converted
    assert_escape_tw("\t\t\t\t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// 'a' + tab with tw=3 → 'a' + 2 spaces = 3 chars
// "a  " → "a &nbsp;"
#[tokio::test]
async fn tab_width_3() {
    assert_escape_tw("a\t", 3, "a &nbsp;").await;
}

// "aaa" + tab with tw=3 → "aaa" + 3 spaces = 6 chars
// "aaa   " → "aaa &nbsp; "
#[tokio::test]
async fn tab_width_3_at_3() {
    assert_escape_tw("aaa\t", 3, "aaa &nbsp; ").await;
}

// =============================================================================
// Section 3: Space Handling (Critical for Visual Fidelity)
// =============================================================================

#[tokio::test]
async fn single_space() {
    assert_escape("a b", "a b").await;
}

#[tokio::test]
async fn double_space() {
    assert_escape("a  b", "a &nbsp;b").await;
}

#[tokio::test]
async fn triple_space() {
    assert_escape("a   b", "a &nbsp; b").await;
}

#[tokio::test]
async fn quad_space() {
    assert_escape("a    b", "a &nbsp; &nbsp;b").await;
}

#[tokio::test]
async fn five_spaces() {
    assert_escape("a     b", "a &nbsp; &nbsp; b").await;
}

#[tokio::test]
async fn six_spaces() {
    assert_escape("a      b", "a &nbsp; &nbsp; &nbsp;b").await;
}

#[tokio::test]
async fn seven_spaces() {
    assert_escape("a       b", "a &nbsp; &nbsp; &nbsp; b").await;
}

#[tokio::test]
async fn eight_spaces() {
    assert_escape("a        b", "a &nbsp; &nbsp; &nbsp; &nbsp;b").await;
}

#[tokio::test]
async fn leading_space() {
    assert_escape(" a", "&nbsp;a").await;
}

#[tokio::test]
async fn leading_two_spaces() {
    // "  a" → double space: " &nbsp;a" → leading: "&nbsp;&nbsp;a"
    assert_escape("  a", "&nbsp;&nbsp;a").await;
}

#[tokio::test]
async fn leading_three_spaces() {
    // "   a" → double space: " &nbsp; a" → leading: "&nbsp;&nbsp; a"
    assert_escape("   a", "&nbsp;&nbsp; a").await;
}

#[tokio::test]
async fn leading_four_spaces() {
    // "    a" → double space: " &nbsp; &nbsp;a" → leading: "&nbsp;&nbsp; &nbsp;a"
    assert_escape("    a", "&nbsp;&nbsp; &nbsp;a").await;
}

#[tokio::test]
async fn trailing_space_lf() {
    // "a \n" → trailing space before \n becomes &nbsp;
    assert_escape("a \n", "a&nbsp;<br />\n").await;
}

#[tokio::test]
async fn trailing_space_cr() {
    // With br=yes, \r becomes <br />\n
    assert_escape("a \r", "a&nbsp;<br />\n").await;
}

#[tokio::test]
async fn trailing_space_crlf() {
    assert_escape("a \r\n", "a&nbsp;<br />\n").await;
}

#[tokio::test]
async fn trailing_two_spaces_lf() {
    // "a  \n" → double space: "a &nbsp;\n" → trailing doesn't match (ends with ;)
    assert_escape("a  \n", "a &nbsp;<br />\n").await;
}

#[tokio::test]
async fn trailing_three_spaces_lf() {
    // "a   \n" → double space: "a &nbsp; \n" → trailing: "a &nbsp;&nbsp;\n"
    assert_escape("a   \n", "a &nbsp;&nbsp;<br />\n").await;
}

#[tokio::test]
async fn trailing_space_eof() {
    // "a " at EOF with br=no - no \r\n follows, so trailing space stays
    assert_escape_no_br("a ", "a ").await;
}

#[tokio::test]
async fn trailing_two_spaces_eof() {
    // "a  " → double space only: "a &nbsp;"
    assert_escape_no_br("a  ", "a &nbsp;").await;
}

#[tokio::test]
async fn trailing_three_spaces_eof() {
    // "a   " → double space: "a &nbsp; " - trailing space at EOF stays
    assert_escape_no_br("a   ", "a &nbsp; ").await;
}

#[tokio::test]
async fn space_only_line() {
    // " \n" → leading space: "&nbsp;\n"
    assert_escape(" \n", "&nbsp;<br />\n").await;
}

#[tokio::test]
async fn two_space_only_line() {
    // "  \n" → double space: " &nbsp;\n" → leading: "&nbsp;&nbsp;\n"
    assert_escape("  \n", "&nbsp;&nbsp;<br />\n").await;
}

#[tokio::test]
async fn multiple_space_line() {
    // "   \n" (3 spaces + LF)
    // double space: " &nbsp; \n"
    // leading: "&nbsp;&nbsp; \n"
    // trailing (space before \n): "&nbsp;&nbsp;&nbsp;\n"
    assert_escape("   \n", "&nbsp;&nbsp;&nbsp;<br />\n").await;
}

#[tokio::test]
async fn leading_after_newline() {
    // "a\n b" - leading space on line 2
    assert_escape("a\n b", "a<br />\n&nbsp;b").await;
}

#[tokio::test]
async fn leading_after_crlf() {
    assert_escape("a\r\n b", "a<br />\n&nbsp;b").await;
}

#[tokio::test]
async fn leading_after_cr() {
    // NOTE: Leading space regex /^\x20/m only matches after \n, not after standalone \r
    // So leading space after \r stays as regular space
    assert_escape("a\r b", "a<br />\n b").await;
}

#[tokio::test]
async fn word_wrap_preserved() {
    // Single spaces stay as spaces, allowing word wrap
    assert_escape("word word", "word word").await;
}

#[tokio::test]
async fn only_spaces() {
    // "     " (5 spaces) → double: " &nbsp; &nbsp; " → leading: "&nbsp;&nbsp; &nbsp; "
    assert_escape_no_br("     ", "&nbsp;&nbsp; &nbsp; ").await;
}

#[tokio::test]
async fn space_between_newlines() {
    // "\n \n" - space is leading on its line
    assert_escape("\n \n", "<br />\n&nbsp;<br />\n").await;
}

// =============================================================================
// Section 4: Line Ending Handling (br=yes)
// =============================================================================

#[tokio::test]
async fn br_unix_lf() {
    assert_escape("a\nb", "a<br />\nb").await;
}

#[tokio::test]
async fn br_windows_crlf() {
    // CRLF should become single <br />\n, NOT two line breaks
    assert_escape("a\r\nb", "a<br />\nb").await;
}

#[tokio::test]
async fn br_mac_cr() {
    assert_escape("a\rb", "a<br />\nb").await;
}

#[tokio::test]
async fn br_multiple_lf() {
    assert_escape("a\n\nb", "a<br />\n<br />\nb").await;
}

#[tokio::test]
async fn br_multiple_crlf() {
    assert_escape("a\r\n\r\nb", "a<br />\n<br />\nb").await;
}

#[tokio::test]
async fn br_multiple_cr() {
    assert_escape("a\r\rb", "a<br />\n<br />\nb").await;
}

#[tokio::test]
async fn br_mixed_endings() {
    // a\nb\r\nc\rd → all become <br />\n
    assert_escape("a\nb\r\nc\rd", "a<br />\nb<br />\nc<br />\nd").await;
}

#[tokio::test]
async fn br_reversed_crlf() {
    // \n\r is LF then CR = two line breaks
    assert_escape("a\n\rb", "a<br />\n<br />\nb").await;
}

#[tokio::test]
async fn br_crlf_then_lf() {
    assert_escape("a\r\n\nb", "a<br />\n<br />\nb").await;
}

#[tokio::test]
async fn br_lf_then_crlf() {
    assert_escape("a\n\r\nb", "a<br />\n<br />\nb").await;
}

#[tokio::test]
async fn br_trailing_newline() {
    assert_escape("a\n", "a<br />\n").await;
}

#[tokio::test]
async fn br_trailing_crlf() {
    assert_escape("a\r\n", "a<br />\n").await;
}

#[tokio::test]
async fn br_trailing_cr() {
    assert_escape("a\r", "a<br />\n").await;
}

#[tokio::test]
async fn br_leading_newline() {
    assert_escape("\na", "<br />\na").await;
}

#[tokio::test]
async fn br_leading_crlf() {
    assert_escape("\r\na", "<br />\na").await;
}

#[tokio::test]
async fn br_only_newlines() {
    assert_escape("\n\n\n", "<br />\n<br />\n<br />\n").await;
}

#[tokio::test]
async fn br_only_crlf() {
    assert_escape("\r\n\r\n", "<br />\n<br />\n").await;
}

#[tokio::test]
async fn br_only_cr() {
    assert_escape("\r\r\r", "<br />\n<br />\n<br />\n").await;
}

#[tokio::test]
async fn br_empty_lines() {
    assert_escape("a\n\n\nb", "a<br />\n<br />\n<br />\nb").await;
}

#[tokio::test]
async fn br_empty_input() {
    assert_escape("", "").await;
}

#[tokio::test]
async fn br_just_text() {
    assert_escape("abc", "abc").await;
}

// =============================================================================
// Section 5: Line Ending Handling (br=no)
// =============================================================================

#[tokio::test]
async fn no_br_unix_lf() {
    assert_escape_no_br("a\nb", "a\nb").await;
}

#[tokio::test]
async fn no_br_windows_crlf() {
    assert_escape_no_br("a\r\nb", "a\r\nb").await;
}

#[tokio::test]
async fn no_br_mac_cr() {
    assert_escape_no_br("a\rb", "a\rb").await;
}

#[tokio::test]
async fn no_br_mixed() {
    assert_escape_no_br("a\nb\r\nc\rd", "a\nb\r\nc\rd").await;
}

#[tokio::test]
async fn no_br_multiple_lf() {
    assert_escape_no_br("a\n\n\nb", "a\n\n\nb").await;
}

#[tokio::test]
async fn no_br_multiple_crlf() {
    assert_escape_no_br("a\r\n\r\nb", "a\r\n\r\nb").await;
}

#[tokio::test]
async fn no_br_trailing() {
    assert_escape_no_br("a\n", "a\n").await;
}

#[tokio::test]
async fn no_br_leading() {
    assert_escape_no_br("\na", "\na").await;
}

#[tokio::test]
async fn no_br_only_newlines() {
    assert_escape_no_br("\n\n\n", "\n\n\n").await;
}

#[tokio::test]
async fn no_br_with_spaces() {
    // Trailing space before \n still gets converted to &nbsp;
    assert_escape_no_br("a \nb", "a&nbsp;\nb").await;
}

#[tokio::test]
async fn no_br_leading_space() {
    // Leading space after \n still gets converted to &nbsp;
    assert_escape_no_br("\n a", "\n&nbsp;a").await;
}

// =============================================================================
// Section 6: Tab Width Validation
// =============================================================================

#[tokio::test]
async fn tw_valid_1() {
    // Just verify it doesn't error - we already tested exact output above
    let result = escape_with_tw_string("\t", "1", true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn tw_valid_8() {
    let result = escape_with_tw_string("\t", "8", true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn tw_valid_large() {
    let result = escape_with_tw_string("\t", "100", true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn tw_invalid_0() {
    let result = escape_with_tw_string("a", "0", true).await;
    assert!(result.is_err(), "Tab width 0 should error");
}

#[tokio::test]
async fn tw_invalid_negative() {
    let result = escape_with_tw_string("a", "-1", true).await;
    assert!(result.is_err(), "Tab width -1 should error");
}

#[tokio::test]
async fn tw_invalid_string() {
    let result = escape_with_tw_string("a", "abc", true).await;
    assert!(result.is_err(), "Tab width 'abc' should error");
}

#[tokio::test]
async fn tw_invalid_empty() {
    let result = escape_with_tw_string("a", "", true).await;
    assert!(result.is_err(), "Empty tab width should error");
}

// =============================================================================
// Section 7: Combined/Complex Cases
// =============================================================================

// Tab + double space + tab
// \t (8 spaces) + "  " (2 spaces, pos 8-9) + \t from pos 10 → 6 more to reach 16
// Total: 8 + 2 + 6 = 16 spaces
#[tokio::test]
async fn tabs_and_spaces() {
    assert_escape_tw("\t  \t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// Double space + tab + double space
// "  " (2) + tab from pos 2 to 8 (6) + "  " (2) = 10 spaces
#[tokio::test]
async fn spaces_and_tabs() {
    assert_escape_tw("  \t  ", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp;").await;
}

#[tokio::test]
async fn html_in_code() {
    // "<div>\n  text\n</div>"
    // Escape: &lt;div&gt;\n  text\n&lt;/div&gt;
    // Double space on "  text" → " &nbsp;text"
    // Leading space: "&nbsp;&nbsp;text"
    // Line endings: <br />
    assert_escape("<div>\n  text\n</div>", "&lt;div&gt;<br />\n&nbsp;&nbsp;text<br />\n&lt;/div&gt;").await;
}

#[tokio::test]
async fn indented_code() {
    // "if (x) {\n    y();\n}"
    // Line 1: "if (x) {" → "if (x) {"
    // Line 2: "    y();" → 4 leading spaces → "&nbsp;&nbsp; &nbsp;y();"
    // Line 3: "}" → "}"
    assert_escape("if (x) {\n    y();\n}", "if (x) {<br />\n&nbsp;&nbsp; &nbsp;y();<br />\n}").await;
}

#[tokio::test]
async fn empty_string() {
    assert_escape("", "").await;
}

#[tokio::test]
async fn whitespace_only_spaces() {
    // "   " (3 spaces) with br=no
    // Double space: " &nbsp; " → leading: "&nbsp;&nbsp; "
    assert_escape_no_br("   ", "&nbsp;&nbsp; ").await;
}

#[tokio::test]
async fn whitespace_only_tabs() {
    // \t\t with tw=8 = 16 spaces (tested above)
    assert_escape_tw("\t\t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp;").await;
}

#[tokio::test]
async fn whitespace_only_mixed() {
    // " \t " = 1 space + 7 spaces (tab from pos 1 to 8) + 1 space = 9 spaces
    // "         " (9 spaces) → double space: " &nbsp; &nbsp; &nbsp; &nbsp; " → leading: "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp; "
    assert_escape_tw(" \t ", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp; ").await;
}

#[tokio::test]
async fn newline_only_lf() {
    assert_escape("\n", "<br />\n").await;
}

#[tokio::test]
async fn newline_only_crlf() {
    assert_escape("\r\n", "<br />\n").await;
}

#[tokio::test]
async fn tab_converted_to_nbsp() {
    // Single tab = 8 spaces → exact pattern (tested above but repeated for doc)
    assert_escape_tw("\t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

#[tokio::test]
async fn unicode_preserved() {
    assert_escape("日本語 テスト", "日本語 テスト").await;
}

#[tokio::test]
async fn unicode_double_space() {
    assert_escape("日本語  テスト", "日本語 &nbsp;テスト").await;
}

#[tokio::test]
async fn emoji_preserved() {
    assert_escape("Hello 👋 World", "Hello 👋 World").await;
}

#[tokio::test]
async fn emoji_with_spaces() {
    assert_escape("👋  👋", "👋 &nbsp;👋").await;
}

#[tokio::test]
async fn special_and_spaces() {
    // "<  >" → escape: "&lt;  &gt;" → double space: "&lt; &nbsp;&gt;"
    assert_escape("<  >", "&lt; &nbsp;&gt;").await;
}

#[tokio::test]
async fn amp_and_spaces() {
    assert_escape("&  &", "&amp; &nbsp;&amp;").await;
}

#[tokio::test]
async fn quote_and_spaces() {
    assert_escape("\"  \"", "&quot; &nbsp;&quot;").await;
}

#[tokio::test]
async fn tab_then_special() {
    // Tab (8 spaces) + '<' → 8 spaces (with nbsp pattern) + "&lt;"
    assert_escape_tw("\t<", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;&lt;").await;
}

#[tokio::test]
async fn special_then_tab() {
    // '<' + tab from pos 1 to 8 (7 spaces)
    // 7 spaces = 3 pairs + 1 trailing space
    assert_escape_tw("<\t", 8, "&lt; &nbsp; &nbsp; &nbsp; ").await;
}

#[tokio::test]
async fn nbsp_in_input() {
    // "&nbsp;" in input - the & gets escaped to &amp;
    assert_escape("&nbsp;", "&amp;nbsp;").await;
}

#[tokio::test]
async fn br_in_input() {
    assert_escape("<br />", "&lt;br /&gt;").await;
}

#[tokio::test]
async fn mixed_everything() {
    // "<a>\t  \n b"
    // Tab expansion: "<a>" (3 chars) + tab to pos 8 (5 spaces) + "  " (2 spaces) = 7 spaces total
    // After double space: "&lt;a&gt; &nbsp; &nbsp; &nbsp; \n b"
    // Trailing space before \n becomes &nbsp;: "&lt;a&gt; &nbsp; &nbsp; &nbsp;&nbsp;\n b"
    // Leading after \n: "&lt;a&gt; &nbsp; &nbsp; &nbsp;&nbsp;\n&nbsp;b"
    // BR conversion: adds <br />
    assert_escape("<a>\t  \n b", "&lt;a&gt; &nbsp; &nbsp; &nbsp;&nbsp;<br />\n&nbsp;b").await;
}

// =============================================================================
// Section 8: Cursor Reset Tests
// =============================================================================

#[tokio::test]
async fn cursor_reset_lf() {
    // "aaa\n\t" with tw=8 - after LF, cursor=0, tab=8 spaces
    assert_escape_tw("aaa\n\t", 8, "aaa<br />\n&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

#[tokio::test]
async fn cursor_reset_cr() {
    // "aaa\r\t" with tw=8 - after CR, cursor=0, tab = 8 spaces
    // NOTE: Leading space regex only matches after \n, not \r
    assert_escape_tw("aaa\r\t", 8, "aaa<br />\n &nbsp; &nbsp; &nbsp; &nbsp;").await;
}

#[tokio::test]
async fn cursor_reset_crlf() {
    assert_escape_tw("aaa\r\n\t", 8, "aaa<br />\n&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

#[tokio::test]
async fn cursor_mid_line() {
    // Line 2: "aaa" + tab = 3 + 5 = 8
    // 5 spaces = 2 pairs + 1 trailing
    assert_escape_tw("aaa\naaa\t", 8, "aaa<br />\naaa &nbsp; &nbsp; ").await;
}

#[tokio::test]
async fn cursor_multiple_lines() {
    // With tw=4:
    // Line 1: "a"
    // Line 2: "aa"
    // Line 3: "aaa\t" = 3 + 1 = 4 → "aaa "
    assert_escape_tw("a\naa\naaa\t", 4, "a<br />\naa<br />\naaa ").await;
}

#[tokio::test]
async fn cursor_tab_doesnt_reset() {
    // "\t\t" with tw=4 = 4 + 4 = 8 spaces (tab advances, doesn't reset)
    assert_escape_tw("\t\t", 4, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

#[tokio::test]
async fn cursor_after_spaces() {
    // "  \n\t" with tw=4 - 2 spaces, newline resets, tab=4
    assert_escape_tw("  \n\t", 4, "&nbsp;&nbsp;<br />\n&nbsp;&nbsp; &nbsp;").await;
}

#[tokio::test]
async fn cursor_long_line_then_tab() {
    // "12345678\t" with tw=8 - at position 8, tab adds 8 more
    // "12345678        " (8 + 8 = 16)
    assert_escape_tw("12345678\t", 8, "12345678 &nbsp; &nbsp; &nbsp; &nbsp;").await;
}

#[tokio::test]
async fn cursor_long_line_lf_tab() {
    // "12345678\n\t" - reset after LF, tab=8
    assert_escape_tw("12345678\n\t", 8, "12345678<br />\n&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// =============================================================================
// Section 9: Processing Order Tests
// =============================================================================

#[tokio::test]
async fn order_tab_before_escape() {
    // Tab at pos 0 = 8 spaces, then '<' escaped
    assert_escape_tw("\t<", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;&lt;").await;
}

// CRITICAL: Tab conversion happens BEFORE nbsp conversion
// "   \t" = 3 spaces + tab from pos 3 → 5 more = 8 total spaces
// If nbsp happened first, "&nbsp;" would be 6 chars and tab would calculate wrong
#[tokio::test]
async fn order_tab_before_nbsp() {
    assert_escape_tw("   \t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

// "  \t" = 2 spaces + tab from pos 2 → 6 more = 8 total spaces
#[tokio::test]
async fn order_tab_before_nbsp_2() {
    assert_escape_tw("  \t", 8, "&nbsp;&nbsp; &nbsp; &nbsp; &nbsp;").await;
}

#[tokio::test]
async fn order_escape_before_nbsp() {
    // "<  " → escape: "&lt;  " → double space: "&lt; &nbsp;"
    assert_escape("<  ", "&lt; &nbsp;").await;
}

#[tokio::test]
async fn order_double_space_before_leading() {
    // "  a" → double space: " &nbsp;a" → leading: "&nbsp;&nbsp;a"
    assert_escape("  a", "&nbsp;&nbsp;a").await;
}

#[tokio::test]
async fn order_trailing_after_double() {
    // "a  \n" → double space: "a &nbsp;\n" → trailing doesn't match (no space before \n)
    assert_escape("a  \n", "a &nbsp;<br />\n").await;
}

#[tokio::test]
async fn order_br_last() {
    // " \n" → leading: "&nbsp;\n" → br: "&nbsp;<br />\n"
    assert_escape(" \n", "&nbsp;<br />\n").await;
}

#[tokio::test]
async fn order_crlf_normalization() {
    // \r\n becomes single <br />\n (CRLF replaced before CR)
    assert_escape("\r\n", "<br />\n").await;
}

// =============================================================================
// Section 10: Page Structure Tests
// =============================================================================

#[tokio::test]
async fn page_loads() {
    let client = client();
    let resp = client
        .get(url("/html-sanitize.htm"))
        .send()
        .await
        .expect("Failed to send request");
    assert!(resp.status().is_success());
    let body = resp.text().await.expect("Failed to read response");
    assert!(body.contains("<form"), "Page should contain form");
    assert!(body.contains("name=\"data\""), "Page should have data field");
}

#[tokio::test]
async fn page_has_form_fields() {
    let client = client();
    let resp = client
        .get(url("/html-sanitize.htm"))
        .send()
        .await
        .expect("Failed to send request");
    let body = resp.text().await.expect("Failed to read response");
    assert!(body.contains("name=\"data\""), "Should have data textarea");
    assert!(body.contains("name=\"tw\""), "Should have tw input");
    assert!(body.contains("name=\"br\""), "Should have br checkbox");
    assert!(body.contains("name=\"sanitize\""), "Should have submit button");
}

#[tokio::test]
async fn output_shows_escaped_message() {
    let client = client();
    let resp = client
        .post(url("/html-sanitize.htm"))
        .form(&[
            ("data", "test"),
            ("tw", "8"),
            ("sanitize", "HTML-Escape Text"),
            ("br", "yes"),
        ])
        .send()
        .await
        .expect("Failed to send request");
    let body = resp.text().await.expect("Failed to read response");
    assert!(body.contains("Your text has been escaped") || body.contains("escaped"),
        "Should show escaped message on POST");
}

#[tokio::test]
async fn get_shows_enter_text() {
    let client = client();
    let resp = client
        .get(url("/html-sanitize.htm"))
        .send()
        .await
        .expect("Failed to send request");
    let body = resp.text().await.expect("Failed to read response");
    assert!(body.contains("Enter Your Text") || body.contains("Enter your text"),
        "Should show enter text prompt on GET");
}
