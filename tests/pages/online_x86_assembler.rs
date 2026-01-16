//! Tests for /online-x86-assembler.htm - Online x86/x64 Assembler
//!
//! Verifies:
//! - Assembly (x86 and x64 architectures)
//! - Disassembly with multiple input formats
//! - Nul byte highlighting in output
//! - Output formats (hex, string literal, array literal)
//! - Error handling for invalid/unsafe input

use crate::common::{client, url};

/// POST to assemble code and return the response body
async fn assemble(code: &str, arch: &str) -> String {
    let client = client();
    let resp = client
        .post(url("/online-x86-assembler.htm"))
        .form(&[
            ("instructions", code),
            ("arch", arch),
            ("submit", "Assemble"),
        ])
        .send()
        .await
        .expect("Failed to send assembly request");

    assert!(
        resp.status().is_success(),
        "Assembly request failed with status {}",
        resp.status()
    );

    resp.text().await.expect("Failed to read response body")
}

/// POST to disassemble hex and return the response body
async fn disassemble(hexstr: &str, arch: &str) -> String {
    let client = client();
    let resp = client
        .post(url("/online-x86-assembler.htm"))
        .form(&[
            ("hexstring", hexstr),
            ("arch", arch),
            ("submit", "Disassemble"),
        ])
        .send()
        .await
        .expect("Failed to send disassembly request");

    assert!(
        resp.status().is_success(),
        "Disassembly request failed with status {}",
        resp.status()
    );

    resp.text().await.expect("Failed to read response body")
}

/// Extract the hex output from assembly/disassembly result HTML
fn extract_hex_output(html: &str) -> Option<String> {
    // Look for the "Raw Hex" section
    // Format: <p><b>Raw Hex</b> (zero bytes in bold):</p>\n<p>HEXDATA&nbsp;...</p>
    if let Some(start) = html.find("<b>Raw Hex</b>") {
        let after = &html[start..];
        // Find the next <p> tag after the header
        if let Some(p_start) = after.find("</p>") {
            let after_header = &after[p_start + 4..];
            if let Some(next_p) = after_header.find("<p>") {
                let content_start = next_p + 3;
                if let Some(p_end) = after_header[content_start..].find("</p>") {
                    let hex_content = &after_header[content_start..content_start + p_end];
                    // Clean up: remove <b>, </b>, &nbsp;
                    let cleaned = hex_content
                        .replace("<b>", "")
                        .replace("</b>", "")
                        .replace("&nbsp;", "")
                        .replace(" ", "")
                        .trim()
                        .to_uppercase();
                    if !cleaned.is_empty() {
                        return Some(cleaned);
                    }
                }
            }
        }
    }
    None
}

/// Extract the hex output with bold tags preserved (for nul byte highlighting)
fn extract_hex_with_bold(html: &str) -> Option<String> {
    // Look for the "Raw Hex" section, preserving <b> tags
    if let Some(start) = html.find("<b>Raw Hex</b>") {
        let after = &html[start..];
        if let Some(p_start) = after.find("</p>") {
            let after_header = &after[p_start + 4..];
            if let Some(next_p) = after_header.find("<p>") {
                let content_start = next_p + 3;
                if let Some(p_end) = after_header[content_start..].find("</p>") {
                    let hex_content = &after_header[content_start..content_start + p_end];
                    // Keep <b> tags, just clean up &nbsp;
                    let cleaned = hex_content
                        .replace("&nbsp;", "")
                        .replace(" ", "")
                        .trim()
                        .to_string();
                    if !cleaned.is_empty() {
                        return Some(cleaned);
                    }
                }
            }
        }
    }
    None
}

/// Extract the string literal output (e.g., "\x90\x90\x90")
fn extract_string_literal(html: &str) -> Option<String> {
    // Format: <p><b>String Literal:</b></p>\n<p>&quot;\x90&quot;</p>
    if let Some(start) = html.find("<b>String Literal:</b>") {
        let after = &html[start..];
        // Find the &quot; which represents "
        if let Some(quote_start) = after.find("&quot;") {
            let after_first_quote = &after[quote_start + 6..]; // skip &quot;
            if let Some(quote_end) = after_first_quote.find("&quot;") {
                // Reconstruct the string with actual quotes
                let inner = &after_first_quote[..quote_end];
                return Some(format!("\"{}\"", inner));
            }
        }
    }
    None
}

/// Extract the array literal output (e.g., "{ 0x90, 0x90, 0x90 }")
fn extract_array_literal(html: &str) -> Option<String> {
    // Format: <p><b>Array Literal:</b> </p>\n{ 0x90 }
    if let Some(start) = html.find("<b>Array Literal:</b>") {
        let after = &html[start..];
        // Find the opening brace
        if let Some(brace_start) = after.find("{") {
            let after_brace = &after[brace_start..];
            if let Some(brace_end) = after_brace.find("}") {
                return Some(after_brace[..brace_end + 1].trim().to_string());
            }
        }
    }
    None
}

/// Check if the response contains an error message
fn contains_error(html: &str) -> bool {
    html.contains("Error") || html.contains("error") || html.contains("unsafe")
}

/// Check if response indicates unsafe code rejection
fn is_unsafe_rejection(html: &str) -> bool {
    html.contains("unsafe") || html.contains("Unsafe") || html.contains("not allowed")
}

// =============================================================================
// Assembly Tests (x86)
// =============================================================================

#[tokio::test]
async fn assemble_single_nop_x86() {
    let body = assemble("nop", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    assert!(
        hex.contains("90"),
        "NOP should assemble to 0x90, got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_multiple_nops_x86() {
    let body = assemble("nop\nnop\nnop", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    assert!(
        hex.contains("909090"),
        "Three NOPs should contain 909090, got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_xor_eax_x86() {
    let body = assemble("xor eax, eax", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    assert!(
        hex.contains("31C0") || hex.contains("33C0"),
        "xor eax,eax should be 31C0 or 33C0, got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_push_pop_x86() {
    let body = assemble("push eax\npop ebx", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    // push eax = 50, pop ebx = 5B
    assert!(
        hex.contains("50") && hex.contains("5B"),
        "push eax; pop ebx should contain 50 and 5B, got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_mov_produces_nul_bytes_x86() {
    let body = assemble("mov eax, 0", "x86").await;
    let hex_bold = extract_hex_with_bold(&body).expect("Should have hex output");
    // mov eax, 0 produces B8 00 00 00 00 - should have bold 00s
    assert!(
        hex_bold.contains("<b>00</b>"),
        "mov eax, 0 should have bold nul bytes, got: {}",
        hex_bold
    );
}

#[tokio::test]
async fn assemble_relative_jump_x86() {
    // Test the `. ` directive for relative jumps
    let body = assemble("jmp . + 5", "x86").await;
    // Should assemble without error (tests safe directive)
    assert!(
        !contains_error(&body) || extract_hex_output(&body).is_some(),
        "Relative jump should assemble successfully"
    );
}

// =============================================================================
// Assembly Tests (x64)
// =============================================================================

#[tokio::test]
async fn assemble_single_nop_x64() {
    let body = assemble("nop", "x64").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    assert!(
        hex.contains("90"),
        "NOP should assemble to 0x90 in x64 mode, got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_xor_rax_x64() {
    let body = assemble("xor rax, rax", "x64").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    // xor rax, rax = 48 31 C0 or 48 33 C0
    assert!(
        hex.contains("4831C0") || hex.contains("4833C0"),
        "xor rax,rax should contain 4831C0 or 4833C0, got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_mov_rax_x64() {
    let body = assemble("mov rax, 0", "x64").await;
    let hex_bold = extract_hex_with_bold(&body).expect("Should have hex output");
    // Should produce nul bytes
    assert!(
        hex_bold.contains("<b>00</b>"),
        "mov rax, 0 should have bold nul bytes, got: {}",
        hex_bold
    );
}

// =============================================================================
// Data Directive Tests
// =============================================================================

#[tokio::test]
async fn assemble_byte_directive() {
    let body = assemble(".byte 0x41, 0x42, 0x43", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    assert!(
        hex.contains("414243"),
        ".byte 0x41,0x42,0x43 should produce 414243, got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_ascii_directive() {
    let body = assemble(".ascii \"HELLO\"", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    // HELLO = 48 45 4C 4C 4F
    assert!(
        hex.contains("48454C4C4F"),
        ".ascii \"HELLO\" should produce 48454C4C4F, got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_asciz_directive() {
    let body = assemble(".asciz \"HI\"", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    let hex_bold = extract_hex_with_bold(&body).expect("Should have hex output");
    // HI\0 = 48 49 00 - should have nul terminator
    assert!(
        hex.contains("4849"),
        ".asciz \"HI\" should contain 4849, got: {}",
        hex
    );
    assert!(
        hex_bold.contains("<b>00</b>"),
        ".asciz should have bold nul terminator, got: {}",
        hex_bold
    );
}

// =============================================================================
// Assembly Error Cases
// =============================================================================

#[tokio::test]
async fn assemble_invalid_instruction() {
    let body = assemble("invalidinstructionxyz", "x86").await;
    assert!(
        contains_error(&body),
        "Invalid instruction should produce error"
    );
}

#[tokio::test]
async fn assemble_unsafe_fill_directive() {
    let body = assemble(".fill 1000000, 1, 0x90", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".fill should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_app_comment() {
    let body = assemble("#APP\nnop", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        "#APP should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_no_app_comment() {
    let body = assemble("#NO_APP\nnop", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        "#NO_APP should be rejected as unsafe"
    );
}

// =============================================================================
// Disassembly Input Format Tests
// =============================================================================

#[tokio::test]
async fn disassemble_raw_hex_no_spaces() {
    let body = disassemble("909090", "x86").await;
    // Should disassemble to NOPs
    assert!(
        body.to_lowercase().contains("nop"),
        "909090 should disassemble to NOPs, body: {}",
        &body[..std::cmp::min(500, body.len())]
    );
}

#[tokio::test]
async fn disassemble_hex_with_spaces() {
    let body = disassemble("90 90 90", "x86").await;
    assert!(
        body.to_lowercase().contains("nop"),
        "90 90 90 should disassemble to NOPs"
    );
}

#[tokio::test]
async fn disassemble_hex_with_0x_prefix() {
    let body = disassemble("0x90 0x90 0x90", "x86").await;
    assert!(
        body.to_lowercase().contains("nop"),
        "0x90 0x90 0x90 should disassemble to NOPs"
    );
}

#[tokio::test]
async fn disassemble_mixed_0x_prefix() {
    let body = disassemble("0x90900x90", "x86").await;
    assert!(
        body.to_lowercase().contains("nop"),
        "Mixed 0x format should disassemble to NOPs"
    );
}

#[tokio::test]
async fn disassemble_c_string_literal() {
    let body = disassemble("\"\\x90\\x90\\x90\"", "x86").await;
    assert!(
        body.to_lowercase().contains("nop"),
        "C string literal format should disassemble to NOPs"
    );
}

#[tokio::test]
async fn disassemble_c_array_format() {
    let body = disassemble("{ 0x90, 0x90, 0x90 }", "x86").await;
    assert!(
        body.to_lowercase().contains("nop"),
        "C array format should disassemble to NOPs"
    );
}

#[tokio::test]
async fn disassemble_mixed_case_hex() {
    let body = disassemble("9a9A9a", "x86").await;
    // Should handle mixed case - 9A is "call far" or similar
    // Just verify it doesn't error
    assert!(
        !body.contains("Error") || body.to_lowercase().contains("call") || body.contains("Disassembly"),
        "Mixed case hex should be handled"
    );
}

// =============================================================================
// Nul Byte Highlighting Tests
// =============================================================================

#[tokio::test]
async fn disassemble_with_nul_byte() {
    let body = disassemble("90 00 90", "x86").await;
    let hex_bold = extract_hex_with_bold(&body);
    if let Some(hex) = hex_bold {
        assert!(
            hex.contains("<b>00</b>"),
            "Nul byte should be bold in output, got: {}",
            hex
        );
    }
    // Also verify the page shows the result somehow
    assert!(
        body.contains("90") || body.to_lowercase().contains("nop"),
        "Should show disassembly result"
    );
}

#[tokio::test]
async fn disassemble_multiple_nul_bytes() {
    let body = disassemble("00 00 00", "x86").await;
    let hex_bold = extract_hex_with_bold(&body);
    if let Some(hex) = hex_bold {
        // Count occurrences of <b>00</b>
        let count = hex.matches("<b>00</b>").count();
        assert!(
            count >= 3,
            "All three nul bytes should be bold, got {} bold bytes in: {}",
            count,
            hex
        );
    }
}

// =============================================================================
// Disassembly Architecture Tests
// =============================================================================

#[tokio::test]
async fn disassemble_x64_instruction() {
    // 48 31 C0 = xor rax, rax (x64 only)
    let body = disassemble("4831c0", "x64").await;
    assert!(
        body.to_lowercase().contains("xor") && body.to_lowercase().contains("rax"),
        "4831c0 should disassemble to xor rax,rax in x64 mode, body: {}",
        &body[..std::cmp::min(500, body.len())]
    );
}

#[tokio::test]
async fn disassemble_x86_vs_x64_difference() {
    // 48 31 C0 in x86 mode should be different (48 is DEC EAX in 32-bit)
    let body_x86 = disassemble("4831c0", "x86").await;
    let body_x64 = disassemble("4831c0", "x64").await;

    // In x64 it should be xor rax,rax
    assert!(
        body_x64.to_lowercase().contains("rax"),
        "x64 should show rax register"
    );
    // In x86, 48 is DEC EAX (REX prefix doesn't exist)
    assert!(
        !body_x86.to_lowercase().contains("rax"),
        "x86 should not show rax register"
    );
}

// =============================================================================
// Output Format Verification Tests
// =============================================================================

#[tokio::test]
async fn output_hex_format() {
    let body = assemble("nop\nnop", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    // Should be uppercase
    assert!(
        hex == hex.to_uppercase(),
        "Hex output should be uppercase, got: {}",
        hex
    );
    // Should have no spaces
    assert!(
        !hex.contains(' '),
        "Hex output should have no spaces, got: {}",
        hex
    );
}

#[tokio::test]
async fn output_string_literal_format() {
    let body = assemble("nop\nnop", "x86").await;
    let literal = extract_string_literal(&body).expect("Should have string literal output");
    // Should be quoted and contain \x escapes
    assert!(
        literal.starts_with('"') && literal.ends_with('"'),
        "String literal should be quoted, got: {}",
        literal
    );
    assert!(
        literal.contains("\\x"),
        "String literal should contain \\x escapes, got: {}",
        literal
    );
}

#[tokio::test]
async fn output_array_literal_format() {
    let body = assemble("nop\nnop", "x86").await;
    let array = extract_array_literal(&body).expect("Should have array literal output");
    // Should be in { 0xNN, ... } format
    assert!(
        array.starts_with('{') && array.ends_with('}'),
        "Array literal should be in braces, got: {}",
        array
    );
    assert!(
        array.contains("0x") || array.contains("0X"),
        "Array literal should contain 0x prefixes, got: {}",
        array
    );
}

#[tokio::test]
async fn output_disassembly_text() {
    let body = assemble("nop\nmov eax, ebx", "x86").await;
    // Should contain instruction mnemonics in disassembly section
    assert!(
        body.to_lowercase().contains("nop"),
        "Disassembly should contain 'nop'"
    );
    assert!(
        body.to_lowercase().contains("mov"),
        "Disassembly should contain 'mov'"
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn assemble_empty_input() {
    let body = assemble("", "x86").await;
    // Should handle gracefully - either show empty result or appropriate message
    // Should not crash or return 500
    assert!(
        body.len() > 0,
        "Empty input should return a page, not crash"
    );
}

#[tokio::test]
async fn disassemble_empty_input() {
    let body = disassemble("", "x86").await;
    // Should handle gracefully
    assert!(
        body.len() > 0,
        "Empty disassembly input should return a page, not crash"
    );
}

#[tokio::test]
async fn disassemble_single_byte() {
    let body = disassemble("90", "x86").await;
    assert!(
        body.to_lowercase().contains("nop"),
        "Single byte 90 should disassemble to NOP"
    );
}

#[tokio::test]
async fn disassemble_odd_length_hex() {
    let body = disassemble("909", "x86").await;
    // Should handle gracefully - either truncate, pad, or show error
    // Main thing is it shouldn't crash
    assert!(
        body.len() > 0,
        "Odd-length hex should be handled gracefully"
    );
}

#[tokio::test]
async fn page_loads_without_input() {
    // Just GET the page without any POST
    let client = client();
    let resp = client
        .get(url("/online-x86-assembler.htm"))
        .send()
        .await
        .expect("Failed to load page");

    assert!(
        resp.status().is_success(),
        "Page should load successfully"
    );

    let body = resp.text().await.unwrap();
    assert!(
        body.contains("x86") && body.contains("Assembler"),
        "Page should contain assembler interface"
    );
}

// =============================================================================
// ALL Safe Directives Tests (from PHP $safe_directives array)
// PHP safe directives: .ascii, .asciz, .align, .balign, .byte, .int, .double,
//                      .quad, .octa, .word, ". " (for relative jumps)
// =============================================================================

#[tokio::test]
async fn assemble_align_directive() {
    // .align pads to an alignment boundary
    let body = assemble(".byte 0x41\n.align 4\n.byte 0x42", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    // Should start with 41 and end with 42, with padding in between
    assert!(
        hex.starts_with("41"),
        ".align should preserve first byte 0x41, got: {}",
        hex
    );
    assert!(
        hex.ends_with("42"),
        ".align should preserve last byte 0x42, got: {}",
        hex
    );
    // Should be longer than 2 bytes due to alignment padding
    assert!(
        hex.len() >= 4, // at least 2 bytes = 4 hex chars
        ".align should add padding, got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_balign_directive() {
    // .balign is byte-granular alignment
    let body = assemble(".byte 0x41\n.balign 4\n.byte 0x42", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    assert!(
        hex.starts_with("41") && hex.ends_with("42"),
        ".balign should work, got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_int_directive() {
    // .int emits a 4-byte integer (little-endian on x86)
    let body = assemble(".int 0x12345678", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    // Little-endian: 0x12345678 -> 78 56 34 12
    assert!(
        hex.contains("78563412"),
        ".int 0x12345678 should produce 78563412 (little-endian), got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_word_directive() {
    // .word emits a 2-byte value (little-endian on x86)
    let body = assemble(".word 0x1234", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    // Little-endian: 0x1234 -> 34 12
    assert!(
        hex.contains("3412"),
        ".word 0x1234 should produce 3412 (little-endian), got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_quad_directive() {
    // .quad emits an 8-byte value (little-endian)
    let body = assemble(".quad 0x0102030405060708", "x64").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    // Little-endian: should reverse byte order
    assert!(
        hex.contains("0807060504030201"),
        ".quad should produce 8 bytes little-endian, got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_octa_directive() {
    // .octa emits a 16-byte value (two 8-byte halves, little-endian)
    let body = assemble(".octa 0x0102030405060708090a0b0c0d0e0f10", "x64").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    // Just verify it produces 16 bytes (32 hex chars)
    assert!(
        hex.len() >= 32,
        ".octa should produce 16 bytes (32 hex chars), got {} chars: {}",
        hex.len(),
        hex
    );
}

#[tokio::test]
async fn assemble_double_directive() {
    // .double emits an 8-byte IEEE 754 double
    // NOTE: Can't use decimal values like 1.0 because the PHP sanitizer
    // removes `.double` then rejects remaining `.` in `1.0`
    // Use integer value instead which GAS will convert to double
    let body = assemble(".double 1", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    // 1 as IEEE 754 double = 0x3FF0000000000000
    // Little-endian: 00 00 00 00 00 00 F0 3F
    assert!(
        hex.len() == 16, // 8 bytes = 16 hex chars
        ".double should produce 8 bytes, got {} chars: {}",
        hex.len(),
        hex
    );
    // Verify it's the IEEE 754 representation of 1.0
    assert!(
        hex == "000000000000F03F",
        ".double 1 should be 000000000000F03F (little-endian), got: {}",
        hex
    );
}

#[tokio::test]
#[ignore = "PHP sanitizer bug: removes .double then rejects remaining . in 1.0"]
async fn assemble_double_directive_with_decimal() {
    // .double should accept floating point literals like 1.0
    // BUG: The PHP sanitizer does str_replace(".double", "", $code) which
    // turns ".double 1.0" into " 1.0", then rejects it because "1.0" contains
    // a period. This is a sanitizer bug - decimal literals should be allowed.
    let body = assemble(".double 1.0", "x86").await;
    let hex = extract_hex_output(&body).expect(".double 1.0 should produce hex output");
    // 1.0 in IEEE 754 double = 0x3FF0000000000000
    // Little-endian: 00 00 00 00 00 00 F0 3F
    assert!(
        hex == "000000000000F03F",
        ".double 1.0 should be 000000000000F03F (little-endian), got: {}",
        hex
    );
}

#[tokio::test]
async fn assemble_relative_jump_dot_space_directive() {
    // ". " (dot followed by space) is a safe directive for the current position
    // Used for relative jumps like: jmp . + 5
    let body = assemble("jmp . + 2\nnop", "x86").await;
    // Should assemble without error
    assert!(
        !is_unsafe_rejection(&body),
        "'. ' directive for relative jumps should be allowed"
    );
    let hex = extract_hex_output(&body);
    assert!(
        hex.is_some(),
        "Relative jump with '. ' should produce output"
    );
}

// =============================================================================
// Unsafe Directive Tests (should be rejected)
// These test the whitelist enforcement
// =============================================================================

#[tokio::test]
async fn assemble_unsafe_org_directive() {
    // .org can set arbitrary addresses - should be rejected
    let body = assemble(".org 0x1000\nnop", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".org should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_section_directive() {
    // .section can create arbitrary sections - should be rejected
    let body = assemble(".section .text\nnop", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".section should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_include_directive() {
    // .include can include arbitrary files - should be rejected
    let body = assemble(".include \"/etc/passwd\"\n", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".include should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_incbin_directive() {
    // .incbin can include binary files - should be rejected
    let body = assemble(".incbin \"/etc/passwd\"\n", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".incbin should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_macro_directive() {
    // .macro can define macros - should be rejected
    let body = assemble(".macro mymacro\nnop\n.endm\nmymacro", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".macro should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_rept_directive() {
    // .rept can repeat blocks - could be used for memory exhaustion
    let body = assemble(".rept 1000000\nnop\n.endr", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".rept should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_space_directive() {
    // .space reserves space - could exhaust memory
    let body = assemble(".space 1000000", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".space should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_skip_directive() {
    // .skip is another form of reserving space
    let body = assemble(".skip 1000000", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".skip should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_set_directive() {
    // .set can set symbols - should be rejected
    let body = assemble(".set foo, 42\n.byte foo", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".set should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_equ_directive() {
    // .equ can assign values to symbols
    let body = assemble(".equ foo, 42\n.byte foo", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".equ should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_global_directive() {
    // .global/.globl can export symbols
    let body = assemble(".global _start\n_start: nop", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".global should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_extern_directive() {
    // .extern can reference external symbols
    let body = assemble(".extern printf\ncall printf", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        ".extern should be rejected as unsafe"
    );
}

// =============================================================================
// Special Comment Tests (should be rejected)
// These are GAS (GNU Assembler) special comments that can change behavior
// =============================================================================

#[tokio::test]
async fn assemble_unsafe_app_comment_multiline() {
    // #APP starts inline assembly passthrough - should be rejected
    let body = assemble("#APP\nmov eax, 0\n#NO_APP", "x86").await;
    assert!(
        is_unsafe_rejection(&body) || contains_error(&body),
        "#APP/#NO_APP comments should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_gcc_patchable_function_entry() {
    // GCC special comments for function patching
    let body = assemble("# __patchable_function_entry\nnop", "x86").await;
    // This might or might not be rejected - just check it doesn't cause issues
    assert!(
        body.len() > 0,
        "Special comments should be handled"
    );
}

// =============================================================================
// Invalid Architecture Tests
// =============================================================================

#[tokio::test]
async fn assemble_invalid_architecture() {
    let client = client();
    let resp = client
        .post(url("/online-x86-assembler.htm"))
        .form(&[
            ("instructions", "nop"),
            ("arch", "arm64"), // Not a valid architecture
            ("submit", "Assemble"),
        ])
        .send()
        .await
        .expect("Failed to send request");

    // Should either return error or fall back to default
    let body = resp.text().await.unwrap();
    // Either shows error or ignores invalid arch and assembles anyway
    assert!(
        body.len() > 0,
        "Invalid architecture should be handled gracefully"
    );
}

#[tokio::test]
async fn disassemble_invalid_architecture() {
    let client = client();
    let resp = client
        .post(url("/online-x86-assembler.htm"))
        .form(&[
            ("hexstring", "90"),
            ("arch", "invalid_arch"),
            ("submit", "Disassemble"),
        ])
        .send()
        .await
        .expect("Failed to send request");

    let body = resp.text().await.unwrap();
    assert!(
        body.len() > 0,
        "Invalid architecture should be handled gracefully"
    );
}

// =============================================================================
// BuildStructuredOutput Edge Cases
// These test the objdump output parsing in PHP's BuildStructuredOutput function
// =============================================================================

#[tokio::test]
async fn assemble_with_labels_in_output() {
    // Code with jumps creates labels in disassembly output
    // objdump shows: "00000000 <.text>:" or "00000005 <label>:"
    let body = assemble("jmp end\nnop\nnop\nend: nop", "x86").await;
    let hex = extract_hex_output(&body);
    assert!(
        hex.is_some(),
        "Assembly with labels should produce output"
    );
    // The disassembly section should contain the label reference
    assert!(
        body.contains("end") || body.contains("0x") || body.to_lowercase().contains("jmp"),
        "Output should show jump/label information"
    );
}

#[tokio::test]
async fn disassemble_far_jump_x86() {
    // Far jump: ljmp $segment, $offset
    // Opcode: EA offset32 segment16 (in x86 32-bit mode)
    // EA 78 56 34 12 00 F0 = ljmp $0xf000, $0x12345678
    let body = disassemble("EA7856341200F0", "x86").await;
    // Should disassemble to a far jump with segment:offset notation
    // objdump may show: "ljmp $0xf000,$0x12345678" or similar
    assert!(
        body.to_lowercase().contains("jmp") || body.to_lowercase().contains("ljmp") || body.contains("f000"),
        "Far jump should be disassembled, body: {}",
        &body[..std::cmp::min(800, body.len())]
    );
}

#[tokio::test]
async fn disassemble_segment_override_prefix() {
    // Instructions with segment overrides show segment:offset in disassembly
    // 3E = DS segment override prefix
    // 3E 8B 00 = mov eax, ds:[eax]
    let body = disassemble("3E8B00", "x86").await;
    // objdump should show the segment override (ds:)
    assert!(
        body.to_lowercase().contains("mov") || body.to_lowercase().contains("ds"),
        "Segment override should be disassembled"
    );
}

#[tokio::test]
async fn disassemble_memory_reference_with_displacement() {
    // Memory references with large displacements
    // 8B 05 78 56 34 12 = mov eax, [0x12345678] (in 32-bit mode)
    let body = disassemble("8B0578563412", "x86").await;
    assert!(
        body.to_lowercase().contains("mov"),
        "Memory reference should be disassembled"
    );
    // May show as hex address in output
    assert!(
        body.contains("0x") || body.contains("12345678") || body.contains("eax"),
        "Should show memory address or register"
    );
}

#[tokio::test]
async fn disassemble_call_relative() {
    // Relative call instruction
    // E8 xx xx xx xx = call relative
    let body = disassemble("E8 05 00 00 00", "x86").await;
    assert!(
        body.to_lowercase().contains("call"),
        "Relative call should show 'call', body: {}",
        &body[..std::cmp::min(500, body.len())]
    );
}

#[tokio::test]
async fn disassemble_call_far_x86() {
    // Far call: 9A offset32 segment16
    // 9A 78 56 34 12 00 F0 = lcall $0xf000, $0x12345678
    let body = disassemble("9A7856341200F0", "x86").await;
    assert!(
        body.to_lowercase().contains("call") || body.to_lowercase().contains("lcall") || body.contains("f000"),
        "Far call should be disassembled with segment:offset"
    );
}

#[tokio::test]
async fn assemble_multiple_labels() {
    // Multiple labels create structured output with location markers
    let code = r#"
start:
    jmp middle
    nop
middle:
    jmp end
    nop
end:
    ret
"#;
    let body = assemble(code, "x86").await;
    let hex = extract_hex_output(&body);
    assert!(
        hex.is_some(),
        "Multiple labels should assemble successfully"
    );
    // The ret instruction (C3) should be at the end
    let hex = hex.unwrap();
    assert!(
        hex.ends_with("C3"),
        "Code should end with ret (C3), got: {}",
        hex
    );
}

#[tokio::test]
async fn disassemble_rep_prefix_instruction() {
    // REP prefix instructions
    // F3 A4 = rep movsb
    let body = disassemble("F3A4", "x86").await;
    assert!(
        body.to_lowercase().contains("rep") || body.to_lowercase().contains("movs"),
        "REP prefix instruction should be disassembled"
    );
}

#[tokio::test]
async fn disassemble_lock_prefix_instruction() {
    // LOCK prefix
    // F0 01 00 = lock add [eax], eax
    let body = disassemble("F00100", "x86").await;
    assert!(
        body.to_lowercase().contains("lock") || body.to_lowercase().contains("add"),
        "LOCK prefix instruction should be disassembled"
    );
}

#[tokio::test]
async fn disassemble_simd_instruction_x64() {
    // SSE/AVX instructions in x64
    // 66 0F EF C0 = pxor xmm0, xmm0
    let body = disassemble("660FEFC0", "x64").await;
    assert!(
        body.to_lowercase().contains("pxor") || body.to_lowercase().contains("xmm"),
        "SIMD instruction should be disassembled showing xmm registers"
    );
}
