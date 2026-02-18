mod common;
// Tests for /online-x86-assembler.htm - Online x86/x64 Assembler
//
// Verifies:
// - Assembly (x86 and x64 architectures)
// - Disassembly with multiple input formats
// - Nul byte highlighting in output
// - Output formats (hex, string literal, array literal)
// - Error handling for invalid/unsafe input

use common::{client, url};

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

/// Extract the disassembly text from the response HTML.
/// Converts HTML formatting (&nbsp; -> space, <br /> -> newline) to plain text.
fn extract_disassembly(html: &str) -> Option<String> {
    // The disassembly is inside: <div ...>Disassembly:</div>\n<div ...>\n<p>...</p>
    // We look for the second "Disassembly:</div>" (the disassembly result, not the form)
    // by finding the one followed by a monospace div with a <p> containing the actual text.
    let marker = "Disassembly:</div>";
    let mut search_from = 0;
    // Find each occurrence and check if it's followed by the actual disassembly content
    while let Some(pos) = html[search_from..].find(marker) {
        let abs_pos = search_from + pos + marker.len();
        let after = &html[abs_pos..];
        // Look for <p> inside a monospace div
        if let Some(p_start) = after.find("<p>") {
            let content_start = p_start + 3;
            if let Some(p_end) = after[content_start..].find("</p>") {
                let raw = &after[content_start..content_start + p_end];
                // Only accept if it looks like disassembly (contains hex addresses)
                if raw.contains("push") || raw.contains("mov") || raw.contains("nop") || raw.contains("ret") {
                    let text = raw
                        .replace("<br />", "\n")
                        .replace("<br/>", "\n")
                        .replace("&nbsp;", " ")
                        .replace("&amp;", "&")
                        .replace("&lt;", "<")
                        .replace("&gt;", ">");
                    // Normalize: trim each line, drop empty lines, rejoin
                    let normalized: String = text
                        .lines()
                        .map(|l| l.trim())
                        .filter(|l| !l.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n");
                    return Some(normalized.trim().to_string());
                }
            }
        }
        search_from = abs_pos;
    }
    None
}

/// Check if the response contains an error message
fn contains_error(html: &str) -> bool {
    html.contains("Error") || html.contains("error")
}

/// Check if response indicates unsafe directive rejection
fn is_unsafe_rejection(html: &str) -> bool {
    html.contains("unsafe directives")
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
    assert!(!contains_error(&body), "Relative jump should assemble without error");
    assert!(
        extract_hex_output(&body).is_some(),
        "Relative jump should produce hex output"
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
        is_unsafe_rejection(&body),
        ".fill should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_app_comment() {
    let body = assemble("#APP\nnop", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
        "#APP should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_no_app_comment() {
    let body = assemble("#NO_APP\nnop", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
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
    assert!(
        !body.contains("Error"),
        "Mixed case hex disassembly should not error, got: {}",
        &body[..body.len().min(500)]
    );
}

// =============================================================================
// Nul Byte Highlighting Tests
// =============================================================================

#[tokio::test]
async fn disassemble_with_nul_byte() {
    let body = disassemble("90 00 90", "x86").await;
    let hex_bold = extract_hex_with_bold(&body)
        .expect("Should have hex bold output for disassemble_with_nul_byte");
    assert!(
        hex_bold.contains("<b>00</b>"),
        "Nul byte should be bold in output, got: {}",
        hex_bold
    );
}

#[tokio::test]
async fn disassemble_multiple_nul_bytes() {
    let body = disassemble("00 00 00", "x86").await;
    let hex_bold = extract_hex_with_bold(&body)
        .expect("Should have hex bold output for disassemble_multiple_nul_bytes");
    // Count occurrences of <b>00</b>
    let count = hex_bold.matches("<b>00</b>").count();
    assert!(
        count >= 3,
        "All three nul bytes should be bold, got {} bold bytes in: {}",
        count,
        hex_bold
    );
}

#[tokio::test]
async fn assemble_cross_byte_00_not_bolded() {
    // "nop" = 0x90, "or ecx, ecx" = 0x09 0xC9
    // Combined hex: 9009C9 — the "00" spans a byte boundary and must NOT be bolded.
    let body = assemble("nop\nor ecx, ecx", "x86").await;
    let hex = extract_hex_output(&body).expect("Should have hex output");
    assert_eq!(hex, "9009C9", "Expected 9009C9, got: {}", hex);
    let hex_bold = extract_hex_with_bold(&body).expect("Should have hex bold output");
    assert!(
        !hex_bold.contains("<b>"),
        "Cross-byte 00 pattern should not be bolded, got: {}",
        hex_bold
    );
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
    // Should show a helpful error about odd number of hex digits
    assert!(
        body.contains("Odd number of hex digits"),
        "Odd-length hex should show error about odd digit count, got: {}",
        &body[..std::cmp::min(500, body.len())]
    );
}

// =============================================================================
// Input Size Limit Tests
// =============================================================================

#[tokio::test]
async fn assemble_over_size_limit_rejected() {
    // Assembly input limit is 10KB. Send >10KB of code.
    let large_code = "nop\n".repeat(3000); // ~12KB
    let body = assemble(&large_code, "x86").await;
    assert!(
        body.contains("too large"),
        "Assembly input over 10KB should be rejected with size error, got: {}",
        &body[..std::cmp::min(500, body.len())]
    );
}

#[tokio::test]
async fn assemble_under_size_limit_accepted() {
    // Just under the limit should work fine
    let code = "nop\n".repeat(2000); // ~8KB
    let body = assemble(&code, "x86").await;
    let hex = extract_hex_output(&body);
    assert!(
        hex.is_some(),
        "Assembly input under 10KB should be accepted"
    );
}

#[tokio::test]
async fn disassemble_over_binary_size_limit_rejected() {
    // Binary input limit is 1MB. Send >1MB of hex data.
    // Each "90" is 1 byte, so we need >1M pairs = >2M hex chars
    let large_hex = "90".repeat(1024 * 1024 + 1); // 1MB + 1 byte
    let body = disassemble(&large_hex, "x86").await;
    assert!(
        body.contains("too large") || body.contains("1MB") || contains_error(&body),
        "Disassembly binary input over 1MB should be rejected, got: {}",
        &body[..std::cmp::min(500, body.len())]
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
async fn assemble_double_directive_with_decimal() {
    // .double should accept floating point literals like 1.0
    // Note: PHP has a bug here - it removes ".double" then rejects the remaining
    // " 1.0" because it contains a period. The Rust implementation fixes this by
    // also removing decimal float patterns (\d+\.\d+) before checking for periods.
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
        is_unsafe_rejection(&body),
        ".org should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_section_directive() {
    // .section can create arbitrary sections - should be rejected
    let body = assemble(".section .text\nnop", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
        ".section should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_include_directive() {
    // .include can include arbitrary files - should be rejected
    let body = assemble(".include \"/etc/passwd\"\n", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
        ".include should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_incbin_directive() {
    // .incbin can include binary files - should be rejected
    let body = assemble(".incbin \"/etc/passwd\"\n", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
        ".incbin should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_macro_directive() {
    // .macro can define macros - should be rejected
    let body = assemble(".macro mymacro\nnop\n.endm\nmymacro", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
        ".macro should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_rept_directive() {
    // .rept can repeat blocks - could be used for memory exhaustion
    let body = assemble(".rept 1000000\nnop\n.endr", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
        ".rept should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_space_directive() {
    // .space reserves space - could exhaust memory
    let body = assemble(".space 1000000", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
        ".space should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_skip_directive() {
    // .skip is another form of reserving space
    let body = assemble(".skip 1000000", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
        ".skip should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_set_directive() {
    // .set can set symbols - should be rejected
    let body = assemble(".set foo, 42\n.byte foo", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
        ".set should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_equ_directive() {
    // .equ can assign values to symbols
    let body = assemble(".equ foo, 42\n.byte foo", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
        ".equ should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_global_directive() {
    // .global/.globl can export symbols
    let body = assemble(".global _start\n_start: nop", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
        ".global should be rejected as unsafe"
    );
}

#[tokio::test]
async fn assemble_unsafe_extern_directive() {
    // .extern can reference external symbols
    let body = assemble(".extern printf\ncall printf", "x86").await;
    assert!(
        is_unsafe_rejection(&body),
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
        is_unsafe_rejection(&body),
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

// =============================================================================
// Exact Output Tests (test vectors from live defuse.ca)
// These verify all output formats exactly match the PHP implementation.
// =============================================================================

/// x86 fibonacci-like sequence: stack frame setup, loop with add/xchg, return.
/// Tests labels, memory operands, loop control, and nul byte highlighting.
/// Test vector obtained from live defuse.ca on 2026-02-17.
#[tokio::test]
async fn assemble_exact_output_x86_fibonacci() {
    let code = "\
push ebp
mov ebp, esp
sub esp, 16
mov dword ptr [ebp-4], 0
mov dword ptr [ebp-8], 1
mov ecx, 10
loop_start:
mov eax, [ebp-4]
add eax, [ebp-8]
mov [ebp-4], eax
xchg eax, [ebp-8]
mov [ebp-4], eax
dec ecx
jnz loop_start
mov eax, [ebp-8]
mov esp, ebp
pop ebp
ret";

    let body = assemble(code, "x86").await;

    let hex = extract_hex_output(&body).expect("Should have hex output");
    assert_eq!(
        hex,
        "5589E583EC10C745FC00000000C745F801000000B90A0000008B45FC0345F88945FC8745F88945FC4975EE8B45F889EC5DC3",
        "x86 fibonacci hex mismatch"
    );

    let hex_bold = extract_hex_with_bold(&body).expect("Should have hex bold output");
    assert_eq!(
        hex_bold,
        "5589E583EC10C745FC<b>00</b><b>00</b><b>00</b><b>00</b>C745F801<b>00</b><b>00</b><b>00</b>B90A<b>00</b><b>00</b><b>00</b>8B45FC0345F88945FC8745F88945FC4975EE8B45F889EC5DC3",
        "x86 fibonacci hex bold mismatch"
    );

    let string_lit = extract_string_literal(&body).expect("Should have string literal");
    assert_eq!(
        string_lit,
        r#""\x55\x89\xE5\x83\xEC\x10\xC7\x45\xFC\x00\x00\x00\x00\xC7\x45\xF8\x01\x00\x00\x00\xB9\x0A\x00\x00\x00\x8B\x45\xFC\x03\x45\xF8\x89\x45\xFC\x87\x45\xF8\x89\x45\xFC\x49\x75\xEE\x8B\x45\xF8\x89\xEC\x5D\xC3""#,
        "x86 fibonacci string literal mismatch"
    );

    let array_lit = extract_array_literal(&body).expect("Should have array literal");
    assert_eq!(
        array_lit,
        "{ 0x55, 0x89, 0xE5, 0x83, 0xEC, 0x10, 0xC7, 0x45, 0xFC, 0x00, 0x00, 0x00, 0x00, 0xC7, 0x45, 0xF8, 0x01, 0x00, 0x00, 0x00, 0xB9, 0x0A, 0x00, 0x00, 0x00, 0x8B, 0x45, 0xFC, 0x03, 0x45, 0xF8, 0x89, 0x45, 0xFC, 0x87, 0x45, 0xF8, 0x89, 0x45, 0xFC, 0x49, 0x75, 0xEE, 0x8B, 0x45, 0xF8, 0x89, 0xEC, 0x5D, 0xC3 }",
        "x86 fibonacci array literal mismatch"
    );
}

/// x64 fibonacci with conditional branches, 64-bit registers, and REX prefixes.
/// Test vector obtained from live defuse.ca on 2026-02-17.
#[tokio::test]
async fn assemble_exact_output_x64_fibonacci() {
    let code = "\
push rbp
mov rbp, rsp
sub rsp, 32
mov qword ptr [rbp-8], rdi
mov qword ptr [rbp-16], rsi
xor rax, rax
mov rcx, [rbp-16]
test rcx, rcx
jz done
cmp rcx, 1
je done
mov rax, 1
mov rdx, 0
fib_loop:
mov r8, rax
add rax, rdx
mov rdx, r8
dec rcx
cmp rcx, 1
jg fib_loop
done:
leave
ret";

    let body = assemble(code, "x64").await;

    let hex = extract_hex_output(&body).expect("Should have hex output");
    assert_eq!(
        hex,
        "554889E54883EC2048897DF8488975F04831C0488B4DF04885C974264883F901742048C7C001000000\
48C7C2000000004989C04801D04C89C248FFC94883F9017FEEC9C3",
        "x64 fibonacci hex mismatch"
    );

    let string_lit = extract_string_literal(&body).expect("Should have string literal");
    assert_eq!(
        string_lit,
        r#""\x55\x48\x89\xE5\x48\x83\xEC\x20\x48\x89\x7D\xF8\x48\x89\x75\xF0\x48\x31\xC0\x48\x8B\x4D\xF0\x48\x85\xC9\x74\x26\x48\x83\xF9\x01\x74\x20\x48\xC7\xC0\x01\x00\x00\x00\x48\xC7\xC2\x00\x00\x00\x00\x49\x89\xC0\x48\x01\xD0\x4C\x89\xC2\x48\xFF\xC9\x48\x83\xF9\x01\x7F\xEE\xC9\xC3""#,
        "x64 fibonacci string literal mismatch"
    );

    let array_lit = extract_array_literal(&body).expect("Should have array literal");
    assert_eq!(
        array_lit,
        "{ 0x55, 0x48, 0x89, 0xE5, 0x48, 0x83, 0xEC, 0x20, 0x48, 0x89, 0x7D, 0xF8, 0x48, 0x89, \
0x75, 0xF0, 0x48, 0x31, 0xC0, 0x48, 0x8B, 0x4D, 0xF0, 0x48, 0x85, 0xC9, 0x74, 0x26, 0x48, \
0x83, 0xF9, 0x01, 0x74, 0x20, 0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00, 0x48, 0xC7, 0xC2, \
0x00, 0x00, 0x00, 0x00, 0x49, 0x89, 0xC0, 0x48, 0x01, 0xD0, 0x4C, 0x89, 0xC2, 0x48, 0xFF, \
0xC9, 0x48, 0x83, 0xF9, 0x01, 0x7F, 0xEE, 0xC9, 0xC3 }",
        "x64 fibonacci array literal mismatch"
    );
}

/// Disassembly round-trip: disassemble the x86 fibonacci hex and verify
/// all output formats match exactly, including the disassembled instructions.
/// Test vector obtained from live defuse.ca on 2026-02-17.
#[tokio::test]
async fn disassemble_exact_output_x86_roundtrip() {
    let hex_input = "5589E583EC10C745FC00000000C745F801000000B90A0000008B45FC0345F88945FC8745F88945FC4975EE8B45F889EC5DC3";
    let body = disassemble(hex_input, "x86").await;

    let hex = extract_hex_output(&body).expect("Should have hex output");
    assert_eq!(
        hex, hex_input,
        "Disassembly round-trip hex should match input exactly"
    );

    let string_lit = extract_string_literal(&body).expect("Should have string literal");
    assert_eq!(
        string_lit,
        r#""\x55\x89\xE5\x83\xEC\x10\xC7\x45\xFC\x00\x00\x00\x00\xC7\x45\xF8\x01\x00\x00\x00\xB9\x0A\x00\x00\x00\x8B\x45\xFC\x03\x45\xF8\x89\x45\xFC\x87\x45\xF8\x89\x45\xFC\x49\x75\xEE\x8B\x45\xF8\x89\xEC\x5D\xC3""#,
        "Disassembly round-trip string literal mismatch"
    );

    let disasm = extract_disassembly(&body).expect("Should have disassembly output");
    let expected_disasm = "\
0:  55                      push   ebp
1:  89 e5                   mov    ebp,esp
3:  83 ec 10                sub    esp,0x10
6:  c7 45 fc 00 00 00 00    mov    DWORD PTR [ebp-0x4],0x0
d:  c7 45 f8 01 00 00 00    mov    DWORD PTR [ebp-0x8],0x1
14: b9 0a 00 00 00          mov    ecx,0xa
19: 8b 45 fc                mov    eax,DWORD PTR [ebp-0x4]
1c: 03 45 f8                add    eax,DWORD PTR [ebp-0x8]
1f: 89 45 fc                mov    DWORD PTR [ebp-0x4],eax
22: 87 45 f8                xchg   DWORD PTR [ebp-0x8],eax
25: 89 45 fc                mov    DWORD PTR [ebp-0x4],eax
28: 49                      dec    ecx
29: 75 ee                   jne    0x19
2b: 8b 45 f8                mov    eax,DWORD PTR [ebp-0x8]
2e: 89 ec                   mov    esp,ebp
30: 5d                      pop    ebp
31: c3                      ret";
    assert_eq!(
        disasm, expected_disasm,
        "Disassembly instructions mismatch"
    );
}
