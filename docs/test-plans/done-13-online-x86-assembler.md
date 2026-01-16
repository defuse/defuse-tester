# Online x86/x64 Assembler Tests

**Status:** ✅ IMPLEMENTED (71 tests)

**File:** `tests/pages/online_x86_assembler.rs`

## Overview

The online x86 assembler accepts assembly code and produces machine code output in multiple formats. It also accepts hex bytes and disassembles them back to assembly. Key features to test:

- Assembly (x86 and x64 architectures)
- Disassembly with multiple input formats
- Nul byte highlighting in output
- Output formats (hex, string literal, array literal)
- Safety checks for unsafe directives
- Error handling

## Test Cases

### Assembly Tests (x86)

| Test | Input | Expected |
|------|-------|----------|
| `assemble_single_nop_x86` | `nop` | Hex output contains `90` |
| `assemble_multiple_nops_x86` | `nop\nnop\nnop` | Hex output contains `909090` |
| `assemble_mov_produces_nul_bytes_x86` | `mov eax, 0` | Output contains nul bytes, `00` is bold in hex_zero_bold |
| `assemble_xor_x86` | `xor eax, eax` | Valid hex output (common shellcode pattern) |
| `assemble_push_pop_x86` | `push eax\npop ebx` | Multi-instruction assembly works |
| `assemble_relative_jump_x86` | `jmp . + 5` | Tests `. ` safe directive for relative jumps |

### Assembly Tests (x64)

| Test | Input | Expected |
|------|-------|----------|
| `assemble_single_nop_x64` | `nop` (x64 mode) | Hex output contains `90` |
| `assemble_mov_rax_x64` | `mov rax, 0` | 64-bit register, produces nul bytes |
| `assemble_xor_rax_x64` | `xor rax, rax` | Valid x64 assembly |

### Data Directive Tests

| Test | Input | Expected |
|------|-------|----------|
| `assemble_byte_directive` | `.byte 0x41, 0x42, 0x43` | Hex output is `414243` |
| `assemble_ascii_directive` | `.ascii "HELLO"` | Hex output is `48454C4C4F` |
| `assemble_asciz_directive` | `.asciz "HI"` | Hex output is `4849` + nul terminator `00` |

### Assembly Error Cases

| Test | Input | Expected |
|------|-------|----------|
| `assemble_invalid_instruction` | `invalidinstruction` | Error message returned |
| `assemble_unsafe_fill_directive` | `.fill 1000000, 1, 0x90` | Rejected (unsafe directive) |
| `assemble_unsafe_app_comment` | `#APP\nnop` | Rejected (unsafe directive) |
| `assemble_unsafe_no_app_comment` | `#NO_APP\nnop` | Rejected (unsafe directive) |

### Disassembly Input Format Tests

| Test | Input | Expected |
|------|-------|----------|
| `disassemble_raw_hex_no_spaces` | `909090` | Disassembles to 3 NOP instructions |
| `disassemble_hex_with_spaces` | `90 90 90` | Disassembles to 3 NOP instructions |
| `disassemble_hex_with_0x_prefix` | `0x90 0x90 0x90` | Strips prefix, disassembles correctly |
| `disassemble_mixed_0x_prefix` | `0x90900x90` | Handles mixed format |
| `disassemble_c_string_literal` | `"\x90\x90\x90"` | Parses \x escape sequences |
| `disassemble_c_array_format` | `{ 0x90, 0x90, 0x90 }` | Parses array notation |
| `disassemble_mixed_case_hex` | `9a9A9a` | Handles mixed case input |

### Nul Byte Highlighting Tests

| Test | Input | Expected |
|------|-------|----------|
| `disassemble_with_nul_byte` | `90 00 90` | `00` appears as `<b>00</b>` in highlighted output |
| `disassemble_multiple_nul_bytes` | `00 00 00` | All nul bytes are bold |
| `assemble_verify_nul_highlighting` | `mov eax, 0` | Assembly producing nuls shows bold `00` |

### Disassembly Architecture Tests

| Test | Input | Expected |
|------|-------|----------|
| `disassemble_x64_instruction` | `4831c0` (x64 mode) | Disassembles to `xor rax, rax` |
| `disassemble_x86_vs_x64_difference` | `4831c0` (x86 vs x64) | Different output per architecture |

### Output Format Verification Tests

| Test | Description |
|------|-------------|
| `output_hex_format` | Verify hex output is uppercase, no spaces |
| `output_string_literal_format` | Verify output contains `"\xNN\xNN..."` format |
| `output_array_literal_format` | Verify output contains `{ 0xNN, 0xNN, ... }` format |
| `output_disassembly_text` | Verify disassembly contains instruction mnemonics |

### Edge Cases

| Test | Input | Expected |
|------|-------|----------|
| `assemble_empty_input` | (empty string) | Handles gracefully, no crash |
| `disassemble_empty_input` | (empty string) | Handles gracefully, no crash |
| `disassemble_single_byte` | `90` | Disassembles single NOP |
| `disassemble_odd_length_hex` | `909` | Handles gracefully (error or truncate) |

## Implementation Notes

- Create `tests/pages/online_x86_assembler.rs`
- Add `pub mod online_x86_assembler;` to `tests/pages/mod.rs`
- Use `common::post_form()` for form submissions
- Assembly form fields: `instructions`, `architecture` ("x86" or "x64"), submit button
- Disassembly form fields: `hexstring`, `architecture` ("x86" or "x64"), submit button
- Parse HTML response using `scraper` crate to extract:
  - Hex output (look for specific div/class)
  - String literal output
  - Array literal output
  - Disassembly text
  - Error messages
- For nul byte highlighting: check for `<b>00</b>` in hex output section
- Tests require `gcc` and `objdump` to be installed on test server

## PHP Implementation Reference

- **Assembler class:** `/defuse.ca/src/libs/Assembler.php`
- **Page handler:** `/defuse.ca/src/pages/services/online-x86-assembler.php`
- **Safe directives:** `.ascii`, `.asciz`, `.align`, `.balign`, `.byte`, `.int`, `.double`, `.quad`, `.octa`, `.word`, `. ` (for relative jumps)
- **Max input size:** 10 KB
- **Disassembly input preprocessing:** Strips `0x`, filters non-hex characters
