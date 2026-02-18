mod common;
// Tests for /big-number-calculator.htm
//
// Verifies:
// - Basic math operations (+, -, *, /, %, ^)
// - Bitwise operations (OR, AND, XOR, SHL, SHR)
// - Input base prefixes (0x hex, 0 octal)
// - Output base conversion (decimal, hex, octal)
// - Digit grouping with spaces
// - Security: character whitelist, error handling

use common::{client, url};

// =============================================================================
// Helper Functions
// =============================================================================

/// Calculate an expression and return the result string.
/// Returns Ok(result) on success, Err(error_message) on error.
async fn calc(expr: &str) -> Result<String, String> {
    calc_with_options(expr, "dec", false).await
}

/// Calculate with output base option.
async fn calc_base(expr: &str, base: &str) -> Result<String, String> {
    calc_with_options(expr, base, false).await
}

/// Calculate with addspaces option.
async fn calc_spaced(expr: &str) -> Result<String, String> {
    calc_with_options(expr, "dec", true).await
}

/// Calculate with all options.
async fn calc_with_options(expr: &str, base: &str, addspaces: bool) -> Result<String, String> {
    let client = client();

    let mut form = vec![
        ("eqn", expr),
        ("base", base),
        ("submit", "Calculate"),
    ];
    if addspaces {
        form.push(("addspaces", "yes"));
    }

    let resp = client
        .post(url("/big-number-calculator.htm"))
        .form(&form)
        .send()
        .await
        .expect("Failed to send request");

    assert!(resp.status().is_success(), "Request failed: {}", resp.status());

    let body = resp.text().await.expect("Failed to read response");
    extract_result(&body)
}

/// Extract result from the bignumber div.
fn extract_result(html: &str) -> Result<String, String> {
    // Find the result div
    let start_marker = r#"id="bignumber""#;
    let Some(div_start) = html.find(start_marker) else {
        return Err("No result div found".to_string());
    };

    // Find the content after the opening tag
    let after_marker = &html[div_start..];
    let Some(content_start) = after_marker.find('>') else {
        return Err("Malformed result div".to_string());
    };

    let content = &after_marker[content_start + 1..];
    let Some(content_end) = content.find("</div>") else {
        return Err("No closing div".to_string());
    };

    let result = content[..content_end].trim();

    // Check for error messages
    if result.contains("Sorry,") {
        return Err(result.to_string());
    }

    Ok(result.to_string())
}

/// Assert calculation equals expected value.
async fn assert_calc(expr: &str, expected: &str) {
    let result = calc(expr).await.unwrap_or_else(|e| panic!("calc({}) failed: {}", expr, e));
    // Normalize: remove line breaks and &nbsp;
    let normalized = result.replace("<br />", "").replace("&nbsp;", "");
    assert_eq!(normalized, expected, "calc({}) = {} (expected {})", expr, normalized, expected);
}

/// Assert calculation with base equals expected value.
async fn assert_calc_base(expr: &str, base: &str, expected: &str) {
    let result = calc_base(expr, base).await.unwrap_or_else(|e| panic!("calc_base({}, {}) failed: {}", expr, base, e));
    let normalized = result.replace("<br />", "").replace("&nbsp;", "");
    assert_eq!(normalized, expected, "calc_base({}, {}) = {} (expected {})", expr, base, normalized, expected);
}

/// Assert calculation with spaces contains expected pattern.
async fn assert_calc_spaced_contains(expr: &str, expected: &str) {
    let result = calc_spaced(expr).await.unwrap_or_else(|e| panic!("calc_spaced({}) failed: {}", expr, e));
    assert!(result.contains(expected), "calc_spaced({}) = {:?} (expected to contain {:?})", expr, result, expected);
}

/// Assert calculation produces an error.
async fn assert_calc_error(expr: &str) {
    let result = calc(expr).await;
    assert!(result.is_err(), "calc({}) should error but got: {:?}", expr, result);
}

/// Assert calculation result starts with expected prefix.
async fn assert_calc_starts_with(expr: &str, prefix: &str) {
    let result = calc(expr).await.unwrap_or_else(|e| panic!("calc({}) failed: {}", expr, e));
    let normalized = result.replace("<br />", "").replace("&nbsp;", "");
    assert!(normalized.starts_with(prefix), "calc({}) = {} (expected to start with {})", expr, normalized, prefix);
}

/// Assert calculation result has expected length.
async fn assert_calc_len_approx(expr: &str, min_len: usize, max_len: usize) {
    let result = calc(expr).await.unwrap_or_else(|e| panic!("calc({}) failed: {}", expr, e));
    let normalized = result.replace("<br />", "").replace("&nbsp;", "").replace(" ", "");
    let len = normalized.len();
    assert!(len >= min_len && len <= max_len,
        "calc({}) has {} digits (expected {}-{})", expr, len, min_len, max_len);
}

/// Get the raw HTML of the result div (including attributes and content).
async fn get_result_div_html(expr: &str, base: &str, addspaces: bool) -> String {
    let client = client();

    let mut form = vec![
        ("eqn", expr),
        ("base", base),
        ("submit", "Calculate"),
    ];
    if addspaces {
        form.push(("addspaces", "yes"));
    }

    let resp = client
        .post(url("/big-number-calculator.htm"))
        .form(&form)
        .send()
        .await
        .expect("Failed to send request");

    assert!(resp.status().is_success(), "Request failed: {}", resp.status());

    let body = resp.text().await.expect("Failed to read response");

    // Find the result div including its opening tag
    let start_marker = r#"id="bignumber""#;
    let Some(div_pos) = body.find(start_marker) else {
        panic!("No result div found");
    };

    // Back up to find the <div
    let before = &body[..div_pos];
    let div_start = before.rfind('<').expect("No opening < for div");

    // Find the closing </div>
    let from_div = &body[div_start..];
    let div_end = from_div.find("</div>").expect("No closing </div>") + 6;

    from_div[..div_end].to_string()
}

// =============================================================================
// Basic Math Operations
// =============================================================================

#[tokio::test]
async fn calc_addition() { assert_calc("2+2", "4").await; }

#[tokio::test]
async fn calc_subtraction() { assert_calc("10-3", "7").await; }

#[tokio::test]
async fn calc_multiplication() { assert_calc("6*7", "42").await; }

#[tokio::test]
async fn calc_division_integer() { assert_calc("10/3", "3").await; }

#[tokio::test]
async fn calc_division_float() {
    let result = calc("10.0/3").await.unwrap();
    assert!(result.starts_with("3.333"), "Expected 3.333..., got {}", result);
}

#[tokio::test]
async fn calc_modulo() { assert_calc("10%3", "1").await; }

#[tokio::test]
async fn calc_exponentiation_caret() { assert_calc("2^10", "1024").await; }

#[tokio::test]
async fn calc_exponentiation_stars() { assert_calc("2**10", "1024").await; }

#[tokio::test]
async fn calc_parentheses() { assert_calc("(2+3)*4", "20").await; }

#[tokio::test]
async fn calc_negative_number() { assert_calc("-5", "-5").await; }

#[tokio::test]
async fn calc_double_negative() { assert_calc("--5", "5").await; }

#[tokio::test]
async fn calc_unary_minus_after_op() { assert_calc("2*-3", "-6").await; }

#[tokio::test]
async fn calc_order_of_operations() { assert_calc("2+3*4", "14").await; }

#[tokio::test]
async fn calc_complex_expression() { assert_calc("(2^8-1)*3+10", "775").await; }

// =============================================================================
// Modulo Behavior (Ruby convention: result has same sign as divisor)
// =============================================================================

#[tokio::test]
async fn mod_positive_positive() { assert_calc("10%3", "1").await; }

#[tokio::test]
async fn mod_negative_dividend() { assert_calc("-7%3", "2").await; } // NOT -1

#[tokio::test]
async fn mod_negative_divisor() { assert_calc("7%-3", "-2").await; } // NOT 1

#[tokio::test]
async fn mod_both_negative() { assert_calc("-7%-3", "-1").await; }

// =============================================================================
// Bitwise Operations
// =============================================================================

#[tokio::test]
async fn bitwise_or_word() { assert_calc("5 OR 4", "5").await; }

#[tokio::test]
async fn bitwise_and_word() { assert_calc("5 AND 4", "4").await; }

#[tokio::test]
async fn bitwise_xor_word() { assert_calc("5 XOR 4", "1").await; }

#[tokio::test]
async fn shift_left_word() { assert_calc("1 SHL 8", "256").await; }

#[tokio::test]
async fn shift_right_word() { assert_calc("256 SHR 4", "16").await; }

#[tokio::test]
async fn bitwise_or_symbol() { assert_calc("5|4", "5").await; }

#[tokio::test]
async fn bitwise_and_symbol() { assert_calc("5&4", "4").await; }

#[tokio::test]
async fn shift_left_symbol() { assert_calc("1<<8", "256").await; }

#[tokio::test]
async fn shift_right_symbol() { assert_calc("256>>4", "16").await; }

#[tokio::test]
async fn bitwise_case_insensitive() { assert_calc("5 or 4", "5").await; }

// =============================================================================
// Input Base Prefixes
// =============================================================================

#[tokio::test]
async fn input_hex_lowercase() { assert_calc("0xff", "255").await; }

#[tokio::test]
async fn input_hex_uppercase() { assert_calc("0xFF", "255").await; }

#[tokio::test]
async fn input_hex_mixed_case() { assert_calc("0xFf", "255").await; }

#[tokio::test]
async fn input_hex_arithmetic() { assert_calc("0xff+1", "256").await; }

#[tokio::test]
async fn input_octal_basic() { assert_calc("0777", "511").await; }

#[tokio::test]
async fn input_octal_arithmetic() { assert_calc("010+1", "9").await; }

#[tokio::test]
async fn input_octal_007() { assert_calc("007", "7").await; }

#[tokio::test]
async fn input_octal_invalid_09() { assert_calc_error("09").await; }

#[tokio::test]
async fn input_mixed_bases() { assert_calc("0xff+0777", "766").await; }

#[tokio::test]
async fn input_hex_in_expression() { assert_calc("0x10*0x10", "256").await; }

// =============================================================================
// Output Base Conversion
// =============================================================================

#[tokio::test]
async fn output_decimal() { assert_calc_base("255", "dec", "255").await; }

#[tokio::test]
async fn output_hexadecimal() { assert_calc_base("255", "hex", "ff").await; }

#[tokio::test]
async fn output_octal() { assert_calc_base("64", "oct", "100").await; }

#[tokio::test]
async fn output_hex_large() { assert_calc_base("65535", "hex", "ffff").await; }

#[tokio::test]
async fn output_octal_large() { assert_calc_base("512", "oct", "1000").await; }

/// Float results are displayed in decimal with a warning when non-decimal base is selected.
/// This is intentional - converting floats to arbitrary bases is complex and error-prone.
#[tokio::test]
async fn output_float_with_non_decimal_base_shows_warning() {
    // When requesting hex output for a float, should show decimal result with warning
    let result = calc_base("15.5", "hex").await.unwrap();
    // Result should contain the decimal value
    assert!(
        result.contains("15.5"),
        "Float result should show decimal value '15.5'. Got: {}",
        result
    );
    // Result should contain warning about floats being decimal
    assert!(
        result.contains("Floating-point") && result.contains("decimal"),
        "Float result with hex base should include warning about decimal display. Got: {}",
        result
    );
}

/// Float results with decimal base should NOT show any warning.
#[tokio::test]
async fn output_float_with_decimal_base_no_warning() {
    // When requesting decimal output for a float, should show result without warning
    let result = calc_base("15.5", "dec").await.unwrap();
    // Result should contain the decimal value
    assert!(
        result.contains("15.5"),
        "Float result should show decimal value '15.5'. Got: {}",
        result
    );
    // Result should NOT contain any warning
    assert!(
        !result.contains("Floating-point") && !result.contains("Note:"),
        "Float result with decimal base should NOT include warning. Got: {}",
        result
    );
}

// =============================================================================
// Digit Grouping (addspaces option)
// =============================================================================

#[tokio::test]
async fn grouping_dec_exact_multiple() {
    assert_calc_spaced_contains("123456789", "123 456 789").await;
}

#[tokio::test]
async fn grouping_dec_with_padding() {
    let result = calc_spaced("12345678").await.unwrap();
    assert!(result.contains("&nbsp;") && result.contains("12 345 678"),
        "Expected padded grouping, got: {}", result);
}

#[tokio::test]
async fn grouping_dec_small() {
    let result = calc_spaced("42").await.unwrap();
    assert!(result.contains("&nbsp;") && result.contains("42"),
        "Expected padded small number, got: {}", result);
}

#[tokio::test]
async fn grouping_dec_single_digit() {
    let result = calc_spaced("5").await.unwrap();
    assert!(result.contains("&nbsp;") && result.contains("5"),
        "Expected padded single digit, got: {}", result);
}

#[tokio::test]
async fn grouping_hex_8_digits() {
    let result = calc_with_options("0xffffffff", "hex", true).await.unwrap();
    assert!(result.contains("ffff ffff"), "Expected hex grouping by 4, got: {}", result);
}

#[tokio::test]
async fn grouping_oct() {
    let result = calc_with_options("0777777", "oct", true).await.unwrap();
    // Octal groups by 2
    assert!(result.contains(" "), "Expected octal grouping, got: {}", result);
}

#[tokio::test]
async fn grouping_disabled() {
    let result = calc_base("12345678", "dec").await.unwrap();
    assert!(!result.contains(" ") || result.contains("<br"),
        "Without addspaces, should have no grouping spaces: {}", result);
}

#[tokio::test]
async fn grouping_negative() {
    let result = calc_spaced("-12345678").await.unwrap();
    assert!(result.contains("-") && result.contains("12 345 678"),
        "Negative with grouping: {}", result);
}

// =============================================================================
// Big Numbers (1000+ digits)
// =============================================================================

#[tokio::test]
async fn big_number_2_pow_100() {
    assert_calc_starts_with("2^100", "1267650600228229401496703205376").await;
}

#[tokio::test]
async fn big_number_2_pow_64() {
    assert_calc("2^64", "18446744073709551616").await;
}

#[tokio::test]
async fn big_number_2_pow_3000() {
    // 2^3000 has ~904 digits
    assert_calc_len_approx("2^3000", 900, 910).await;
}

#[tokio::test]
async fn big_number_2_pow_10000() {
    // 2^10000 has ~3011 digits
    assert_calc_len_approx("2^10000", 3000, 3020).await;
}

#[tokio::test]
async fn big_number_arithmetic() {
    // 2^1000 + 2^1000 = 2^1001
    let sum = calc("2^1000+2^1000").await.unwrap().replace("<br />", "").replace("&nbsp;", "");
    let doubled = calc("2^1001").await.unwrap().replace("<br />", "").replace("&nbsp;", "");
    assert_eq!(sum, doubled, "2^1000 + 2^1000 should equal 2^1001");
}

#[tokio::test]
async fn big_number_multiplication() {
    // 2^500 * 2^500 = 2^1000
    let product = calc("2^500*2^500").await.unwrap().replace("<br />", "").replace("&nbsp;", "");
    let power = calc("2^1000").await.unwrap().replace("<br />", "").replace("&nbsp;", "");
    assert_eq!(product, power, "2^500 * 2^500 should equal 2^1000");
}

#[tokio::test]
async fn big_number_hex_output() {
    assert_calc_base("2^64-1", "hex", "ffffffffffffffff").await;
}

#[tokio::test]
async fn big_number_with_spaces() {
    let result = calc_spaced("2^3000").await.unwrap();
    // Should have many space-separated groups
    let space_count = result.matches(' ').count();
    assert!(space_count > 200, "Big number should have many digit groups, got {} spaces", space_count);
}

// =============================================================================
// Operator Precedence (critical for parser compatibility)
// =============================================================================

#[tokio::test]
async fn prec_exp_right_assoc() {
    // 2^3^2 = 2^(3^2) = 2^9 = 512 (NOT (2^3)^2 = 64)
    assert_calc("2^3^2", "512").await;
}

#[tokio::test]
async fn prec_exp_over_mult() { assert_calc("2*3^2", "18").await; }

#[tokio::test]
async fn prec_mult_over_add() { assert_calc("2+3*4", "14").await; }

#[tokio::test]
async fn prec_add_over_shift() { assert_calc("1<<2+3", "32").await; }

#[tokio::test]
async fn prec_shift_over_bitor() { assert_calc("1<<4|2", "18").await; }

#[tokio::test]
async fn prec_add_over_bitor() { assert_calc("2+3|4", "5").await; }

#[tokio::test]
async fn prec_complex() { assert_calc("2+3*4^2", "50").await; }

// =============================================================================
// Error Cases
// =============================================================================

#[tokio::test]
async fn error_invalid_expression() { assert_calc_error("abc").await; }

#[tokio::test]
async fn error_division_by_zero() { assert_calc_error("1/0").await; }

#[tokio::test]
async fn error_range_operator() { assert_calc_error("2..5").await; }

#[tokio::test]
async fn error_triple_dot() { assert_calc_error("2...5").await; }

#[tokio::test]
async fn error_command_injection() { assert_calc_error("system(\"ls\")").await; }

#[tokio::test]
async fn error_backticks() { assert_calc_error("`ls`").await; }

#[tokio::test]
async fn error_eval() { assert_calc_error("eval(\"1\")").await; }

#[tokio::test]
async fn error_dollar() { assert_calc_error("$foo").await; }

#[tokio::test]
async fn error_at() { assert_calc_error("@foo").await; }

#[tokio::test]
async fn error_semicolon() { assert_calc_error("1;2").await; }

#[tokio::test]
async fn error_newline() { assert_calc_error("1\n2").await; }

#[tokio::test]
async fn error_double_quotes() { assert_calc_error("\"test\"").await; }

#[tokio::test]
async fn error_single_quotes() { assert_calc_error("'test'").await; }

#[tokio::test]
async fn error_square_brackets() { assert_calc_error("[1,2]").await; }

#[tokio::test]
async fn error_curly_braces() { assert_calc_error("{1}").await; }

#[tokio::test]
async fn error_underscore() { assert_calc_error("1_000").await; }

#[tokio::test]
async fn error_hash() { assert_calc_error("#comment").await; }

#[tokio::test]
async fn error_colon() { assert_calc_error("1:2").await; }

#[tokio::test]
async fn error_question() { assert_calc_error("1?2:3").await; }

#[tokio::test]
async fn error_tilde() { assert_calc_error("~5").await; }

#[tokio::test]
async fn error_exclamation() { assert_calc_error("!true").await; }

#[tokio::test]
async fn error_unmatched_paren_open() { assert_calc_error("(2+3").await; }

#[tokio::test]
async fn error_unmatched_paren_close() { assert_calc_error("2+3)").await; }

#[tokio::test]
async fn error_empty_parens() { assert_calc_error("()").await; }

#[tokio::test]
async fn error_trailing_operator() { assert_calc_error("2+").await; }

#[tokio::test]
async fn error_leading_operator_mult() { assert_calc_error("*2").await; }

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn edge_just_number() { assert_calc("42", "42").await; }

#[tokio::test]
async fn edge_zero() { assert_calc("0", "0").await; }

#[tokio::test]
async fn edge_negative_result() { assert_calc("5-10", "-5").await; }

#[tokio::test]
async fn edge_large_negative() { assert_calc("-2^64", "-18446744073709551616").await; }

#[tokio::test]
async fn edge_whitespace_around() { assert_calc(" 2 + 2 ", "4").await; }

#[tokio::test]
async fn edge_whitespace_internal() { assert_calc("2  +  2", "4").await; }

#[tokio::test]
async fn edge_float_input() {
    let result = calc("3.14159").await.unwrap();
    assert!(result.starts_with("3.14159"), "Expected 3.14159, got {}", result);
}

#[tokio::test]
async fn edge_float_arithmetic() {
    let result = calc("3.14*2").await.unwrap();
    assert!(result.starts_with("6.28"), "Expected 6.28, got {}", result);
}

#[tokio::test]
async fn edge_nested_parens() { assert_calc("((2+3)*4)", "20").await; }

#[tokio::test]
async fn edge_deeply_nested() { assert_calc("(((1+2)))", "3").await; }

#[tokio::test]
async fn edge_adjacent_operators() { assert_calc("2+-3", "-1").await; }

#[tokio::test]
async fn edge_zero_exponent() { assert_calc("5^0", "1").await; }

#[tokio::test]
async fn edge_one_exponent() { assert_calc("5^1", "5").await; }

#[tokio::test]
async fn edge_zero_base() { assert_calc("0^5", "0").await; }

#[tokio::test]
async fn edge_zero_zero() { assert_calc("0^0", "1").await; }

// =============================================================================
// Word Operator Case Sensitivity
// =============================================================================

#[tokio::test]
async fn word_or_lowercase() { assert_calc("5 or 4", "5").await; }

#[tokio::test]
async fn word_or_uppercase() { assert_calc("5 OR 4", "5").await; }

#[tokio::test]
async fn word_or_mixedcase() { assert_calc("5 Or 4", "5").await; }

#[tokio::test]
async fn word_and_lowercase() { assert_calc("5 and 4", "4").await; }

#[tokio::test]
async fn word_xor_lowercase() { assert_calc("5 xor 4", "1").await; }

#[tokio::test]
async fn word_shl_lowercase() { assert_calc("1 shl 8", "256").await; }

#[tokio::test]
async fn word_shr_lowercase() { assert_calc("256 shr 4", "16").await; }

// =============================================================================
// Page Structure
// =============================================================================

#[tokio::test]
async fn page_loads() {
    let client = client();
    let resp = client.get(url("/big-number-calculator.htm")).send().await.unwrap();
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    assert!(body.contains("<form") && body.contains("eqn"));
}

#[tokio::test]
async fn page_has_base_options() {
    let client = client();
    let resp = client.get(url("/big-number-calculator.htm")).send().await.unwrap();
    let body = resp.text().await.unwrap();
    assert!(body.contains("value=\"dec\"") && body.contains("value=\"hex\"") && body.contains("value=\"oct\""));
}

#[tokio::test]
async fn page_has_operation_help() {
    let client = client();
    let resp = client.get(url("/big-number-calculator.htm")).send().await.unwrap();
    let body = resp.text().await.unwrap();
    assert!(body.contains("OR") && body.contains("AND") && body.contains("XOR"));
}

#[tokio::test]
async fn page_has_form_fields() {
    let client = client();
    let resp = client.get(url("/big-number-calculator.htm")).send().await.unwrap();
    let body = resp.text().await.unwrap();
    assert!(body.contains("name=\"eqn\""), "Missing eqn field");
    assert!(body.contains("name=\"base\""), "Missing base field");
    assert!(body.contains("name=\"addspaces\""), "Missing addspaces field");
    assert!(body.contains("name=\"submit\"") || body.contains("type=\"submit\""), "Missing submit");
}

// =============================================================================
// Negative Numbers with Base Conversion
// =============================================================================

#[tokio::test]
async fn neg_hex_output() { assert_calc_base("-255", "hex", "-ff").await; }

#[tokio::test]
async fn neg_oct_output() { assert_calc_base("-64", "oct", "-100").await; }

#[tokio::test]
async fn hex_math_hex_output() { assert_calc_base("0xff+0xff", "hex", "1fe").await; }

// =============================================================================
// Shift Operations Edge Cases
// =============================================================================

#[tokio::test]
async fn shift_big_number() {
    // 1<<10000 = 2^10000, should have ~3011 digits
    assert_calc_len_approx("1<<10000", 3000, 3020).await;
}

#[tokio::test]
async fn shift_right_negative() {
    // Arithmetic shift: -256 >> 4 = -16
    assert_calc("-256>>4", "-16").await;
}

#[tokio::test]
async fn shift_negative_amount() {
    // Shifting by negative amount - may error or have specific behavior
    let result = calc("1<<-1").await;
    // Document actual behavior (likely error or 0)
    assert!(result.is_err() || result.as_ref().map(|s| s == "0").unwrap_or(false),
        "1<<-1 should error or return 0, got: {:?}", result);
}

// =============================================================================
// Exponent Edge Cases
// =============================================================================

#[tokio::test]
async fn negative_exponent_direct() {
    // 2^-3 = 1/8 as a rational
    let result = calc("2^-3").await.unwrap();
    assert_eq!(result, "1 / 8", "Expected '1 / 8', got {}", result);
}

#[tokio::test]
async fn fractional_exponent() {
    let result = calc("8^0.5").await.unwrap();
    // sqrt(8) ≈ 2.828...
    assert!(result.starts_with("2.828"), "Expected ~2.828, got {}", result);
}

#[tokio::test]
async fn negative_base_even_exp() {
    // (-5)^2 = 25
    assert_calc("(-5)^2", "25").await;
}

#[tokio::test]
async fn negative_base_odd_exp() {
    // (-5)^3 = -125
    assert_calc("(-5)^3", "-125").await;
}

#[tokio::test]
async fn unary_minus_exp() {
    // -5^2 = -(5^2) = -25 (unary minus binds looser than exponentiation)
    assert_calc("-5^2", "-25").await;
}

#[tokio::test]
async fn triple_negative() {
    assert_calc("---5", "-5").await;
}

// =============================================================================
// Rational Numbers (using r suffix)
// =============================================================================

/// Basic rational division: 1r/2 = 1/2
#[tokio::test]
async fn rational_basic_division() {
    let result = calc("1r/2").await.unwrap();
    assert_eq!(result, "1 / 2", "Expected '1 / 2', got '{}'", result);
}

/// Rational with negative exponent: 2r^-3 = 1/8
#[tokio::test]
async fn rational_negative_exponent() {
    let result = calc("2r^-3").await.unwrap();
    assert_eq!(result, "1 / 8", "Expected '1 / 8', got '{}'", result);
}

/// Rational multiplication: (1r/2)*3 = 3/2
#[tokio::test]
async fn rational_multiplication() {
    let result = calc("(1r/2)*3").await.unwrap();
    assert_eq!(result, "3 / 2", "Expected '3 / 2', got '{}'", result);
}

/// Rational addition: 1r/2 + 1r/4 = 3/4
#[tokio::test]
async fn rational_addition() {
    let result = calc("1r/2 + 1r/4").await.unwrap();
    assert_eq!(result, "3 / 4", "Expected '3 / 4', got '{}'", result);
}

/// Rational division stays as rational even when it could simplify: 4r/2 = 2/1
#[tokio::test]
async fn rational_stays_rational() {
    let result = calc("4r/2").await.unwrap();
    assert_eq!(result, "2 / 1", "Expected '2 / 1', got '{}'", result);
}

/// Negative rational: -1r/2 = -1/2
#[tokio::test]
async fn rational_negative() {
    let result = calc("-1r/2").await.unwrap();
    assert_eq!(result, "-1 / 2", "Expected '-1 / 2', got '{}'", result);
}

/// Rational with hex output: 15r/2 in hex = f / 2
#[tokio::test]
async fn rational_hex_output() {
    let result = calc_base("15r/2", "hex").await.unwrap();
    assert_eq!(result, "f / 2", "Expected 'f / 2', got '{}'", result);
}

/// Rational with large numbers
#[tokio::test]
async fn rational_large_numbers() {
    let result = calc("1r/1000000").await.unwrap();
    assert_eq!(result, "1 / 1000000", "Expected '1 / 1000000', got '{}'", result);
}

/// Rational with digit grouping shows multiline format
#[tokio::test]
async fn rational_with_grouping() {
    let result = calc_spaced("1r/1000000").await.unwrap();
    // With grouping, rational is displayed as:
    // numerator
    //  /
    // denominator (with digit grouping)
    assert!(result.contains("/"), "Expected '/' separator, got '{}'", result);
    assert!(result.contains("1 000 000"), "Expected grouped denominator '1 000 000', got '{}'", result);
}

/// Float still works: 0.5*3 = 1.5
#[tokio::test]
async fn float_still_works() {
    let result = calc("0.5*3").await.unwrap();
    assert!(result.contains("1.5"), "Expected '1.5', got '{}'", result);
}

/// User can force float by multiplying by 1.0: 1r/2 * 1.0 = 0.5
#[tokio::test]
async fn rational_to_float_with_multiply() {
    let result = calc("(1r/2)*1.0").await.unwrap();
    assert!(result.contains("0.5"), "Expected '0.5', got '{}'", result);
}

// =============================================================================
// Division Edge Cases (Ruby: rounds toward negative infinity)
// =============================================================================

#[tokio::test]
async fn div_neg_dividend() {
    // Ruby: -10/3 = -4 (floor division)
    assert_calc("-10/3", "-4").await;
}

#[tokio::test]
async fn div_neg_divisor() {
    // Ruby: 10/-3 = -4 (floor division)
    assert_calc("10/-3", "-4").await;
}

#[tokio::test]
async fn div_both_negative() {
    // -10/-3 = 3 (positive result)
    assert_calc("-10/-3", "3").await;
}

// =============================================================================
// Float Operations
// =============================================================================

#[tokio::test]
async fn float_modulo() {
    let result = calc("10.5%3").await.unwrap();
    assert!(result.starts_with("1.5"), "Expected 1.5, got {}", result);
}

#[tokio::test]
async fn float_division() {
    assert_calc("7.0/2", "3.5").await;
}

#[tokio::test]
async fn float_negative() {
    let result = calc("-3.14").await.unwrap();
    assert!(result.starts_with("-3.14"), "Expected -3.14, got {}", result);
}

// =============================================================================
// Chained and Mixed Operations
// =============================================================================

#[tokio::test]
async fn chained_addition() {
    assert_calc("1+2+3+4+5", "15").await;
}

#[tokio::test]
async fn chained_multiplication() {
    assert_calc("2*3*4", "24").await;
}

#[tokio::test]
async fn chained_exponentiation() {
    // Right-to-left: 2^(2^(2^2)) = 2^(2^4) = 2^16 = 65536
    assert_calc("2^2^2^2", "65536").await;
}

#[tokio::test]
async fn mixed_bitwise_and_or() {
    // & has higher precedence than |
    // 5|4&3 = 5|(4&3) = 5|0 = 5
    assert_calc("5|4&3", "5").await;
}

#[tokio::test]
async fn mixed_word_bitwise() {
    // AND before OR: 7 AND 3 OR 4 = (7&3)|4 = 3|4 = 7
    assert_calc("7 AND 3 OR 4", "7").await;
}

// =============================================================================
// Additional Error Cases
// =============================================================================

#[tokio::test]
async fn scientific_notation() {
    assert_calc("1e5", "100000.0").await;
}

#[tokio::test]
async fn scientific_notation_negative_exponent() {
    assert_calc("1e-5", "1.0e-05").await;
}

#[tokio::test]
async fn scientific_notation_float_mantissa() {
    assert_calc("2.5e3", "2500.0").await;
}

#[tokio::test]
async fn error_whitespace_only() {
    let result = calc("   ").await;
    assert!(result.is_err(), "Whitespace-only input should produce an error, got: {:?}", result);
}

#[tokio::test]
async fn error_equals() { assert_calc_error("x=5").await; }

#[tokio::test]
async fn error_backslash() { assert_calc_error("1\\2").await; }

// =============================================================================
// Additional Edge Cases
// =============================================================================

#[tokio::test]
async fn edge_empty_input() {
    let client = client();
    let resp = client
        .post(url("/big-number-calculator.htm"))
        .form(&[("eqn", ""), ("base", "dec"), ("submit", "Calculate")])
        .send()
        .await
        .unwrap();
    let body = resp.text().await.unwrap();
    // Empty input should not show a result or show error
    let result = extract_result(&body);
    assert!(result.is_err() || result.as_ref().map(|s| s.trim().is_empty()).unwrap_or(false),
        "Empty input should have no result, got: {:?}", result);
}

#[tokio::test]
async fn edge_leading_plus() {
    assert_calc("+5", "5").await;
}

#[tokio::test]
async fn consecutive_operators_work() {
    // 2**+3 works as 2**(+3) = 8 (not an error)
    assert_calc("2**+3", "8").await;
}

#[tokio::test]
async fn edge_plus_minus() {
    // 5+-+3 = 5 + (-(+3)) = 5 + (-3) = 2
    assert_calc("5+-+3", "2").await;
}

#[tokio::test]
async fn edge_hex_uppercase_x() {
    // 0Xff should work (uppercase X)
    assert_calc("0Xff", "255").await;
}

// =============================================================================
// Additional Digit Grouping Tests
// =============================================================================

#[tokio::test]
async fn grouping_dec_2_char_pad() {
    let result = calc_spaced("1234567").await.unwrap();
    // Should have 2 nbsp padding + "1 234 567"
    assert!(result.contains("1 234 567"), "Expected grouped with 2-char pad, got: {}", result);
}

#[tokio::test]
async fn grouping_hex_with_padding() {
    let result = calc_with_options("0xfffff", "hex", true).await.unwrap();
    // 0xfffff = 1048575 = fffff (5 hex digits)
    // Groups of 4: should have padding + "f ffff"
    assert!(result.contains("ffff"), "Expected hex grouping, got: {}", result);
}

#[tokio::test]
async fn grouping_oct_with_padding() {
    let result = calc_with_options("0777", "oct", true).await.unwrap();
    // 0777 octal = 511 decimal = 777 octal output
    // Groups of 2 with padding
    assert!(result.contains("77"), "Expected octal grouping, got: {}", result);
}

#[tokio::test]
async fn grouping_disabled_no_breaks() {
    let result = calc_base("2^200", "dec").await.unwrap();
    // Without addspaces, long numbers do NOT get <br /> line breaks
    // CSS word-break: break-all handles wrapping instead
    assert!(!result.contains("<br />"),
        "Long number without spaces should NOT have <br /> breaks, got: {}", result);
}

// =============================================================================
// CSS Formatting Tests
// =============================================================================

/// Without add_spaces, the result div should have word-break: break-all for long number wrapping.
#[tokio::test]
async fn css_word_break_without_spaces() {
    let div_html = get_result_div_html("12345", "dec", false).await;
    assert!(div_html.contains("word-break: break-all"),
        "Result div without add_spaces should have word-break: break-all. Got: {}", div_html);
}

/// With add_spaces, the result div should NOT have word-break: break-all.
#[tokio::test]
async fn css_no_word_break_with_spaces() {
    let div_html = get_result_div_html("12345", "dec", true).await;
    assert!(!div_html.contains("word-break"),
        "Result div with add_spaces should NOT have word-break. Got: {}", div_html);
}

/// Multiline rational output (with grouping) should be right-aligned.
#[tokio::test]
async fn css_rational_right_aligned() {
    let div_html = get_result_div_html("1r/1000000", "dec", true).await;
    assert!(div_html.contains("text-align: right"),
        "Multiline rational should be right-aligned. Got: {}", div_html);
}

/// Multiline rational output should use <br /> for line breaks.
#[tokio::test]
async fn rational_multiline_uses_br() {
    let result = calc_spaced("1r/1000000").await.unwrap();
    // Multiline rational format: numerator<br /> / <br />denominator
    assert!(result.contains("<br />"),
        "Multiline rational should contain <br /> tags. Got: {}", result);
}

/// Non-rational results should NOT be right-aligned.
#[tokio::test]
async fn css_integer_not_right_aligned() {
    let div_html = get_result_div_html("12345678", "dec", true).await;
    assert!(!div_html.contains("text-align: right"),
        "Integer result should NOT be right-aligned. Got: {}", div_html);
}
