# Big Number Calculator Tests

**Status:** ✅ IMPLEMENTED (155 test cases)

**File:** `tests/pages/big_number_calculator.rs`

## Overview

The big number calculator evaluates mathematical expressions with arbitrarily large integers. It uses Ruby as the backend computation engine with security sandboxing. Key features:

- Basic math operations (+, -, *, /, %, ^)
- Bitwise operations (OR, AND, XOR, SHL, SHR)
- Input base prefixes (0x for hex, 0 for octal)
- Output base conversion (decimal, hex, octal)
- Digit grouping with spaces
- Security: character whitelist, rejects unsafe patterns

## Test Cases

### Basic Math Operations

| Test | Input | Expected |
|------|-------|----------|
| `calc_addition` | `2+2` | `4` |
| `calc_subtraction` | `10-3` | `7` |
| `calc_multiplication` | `6*7` | `42` |
| `calc_division_integer` | `10/3` | `3` (truncated) |
| `calc_division_float` | `10.0/3` | `3.333...` (float result) |
| `calc_modulo` | `10%3` | `1` |
| `calc_exponentiation_caret` | `2^10` | `1024` |
| `calc_exponentiation_stars` | `2**10` | `1024` |
| `calc_parentheses` | `(2+3)*4` | `20` |
| `calc_negative_number` | `-5` | `-5` |
| `calc_double_negative` | `--5` | `5` |
| `calc_unary_minus_after_op` | `2*-3` | `-6` |
| `calc_order_of_operations` | `2+3*4` | `14` (not 20) |
| `calc_complex_expression` | `(2^8-1)*3+10` | `775` |

### Modulo Behavior (Ruby convention: result has same sign as divisor)

| Test | Input | Expected | Notes |
|------|-------|----------|-------|
| `mod_positive_positive` | `10%3` | `1` | Standard case |
| `mod_negative_dividend` | `-7%3` | `2` | NOT -1 (Ruby convention) |
| `mod_negative_divisor` | `7%-3` | `-2` | NOT 1 (Ruby convention) |
| `mod_both_negative` | `-7%-3` | `-1` | Both negative |

### Bitwise Operations

| Test | Input | Expected |
|------|-------|----------|
| `calc_bitwise_or_word` | `5 OR 4` | `5` (101 \| 100 = 101) |
| `calc_bitwise_and_word` | `5 AND 4` | `4` (101 & 100 = 100) |
| `calc_bitwise_xor_word` | `5 XOR 4` | `1` (101 ^ 100 = 001) |
| `calc_shift_left_word` | `1 SHL 8` | `256` |
| `calc_shift_right_word` | `256 SHR 4` | `16` |
| `calc_bitwise_or_symbol` | `5 \| 4` | `5` |
| `calc_bitwise_and_symbol` | `5 & 4` | `4` |
| `calc_shift_left_symbol` | `1 << 8` | `256` |
| `calc_shift_right_symbol` | `256 >> 4` | `16` |
| `calc_bitwise_case_insensitive` | `5 or 4` | `5` (lowercase works) |

### Input Base Prefixes

| Test | Input | Expected |
|------|-------|----------|
| `input_hex_lowercase` | `0xff` | `255` (in decimal output) |
| `input_hex_uppercase` | `0xFF` | `255` |
| `input_hex_mixed_case` | `0xFf` | `255` |
| `input_hex_arithmetic` | `0xff + 1` | `256` |
| `input_octal_basic` | `0777` | `511` |
| `input_octal_arithmetic` | `010 + 1` | `9` (octal 10 = decimal 8) |
| `input_octal_007` | `007` | `7` (valid octal) |
| `input_octal_invalid_09` | `09` | Error (9 not valid in octal) |
| `input_mixed_bases` | `0xff + 0777` | `766` |
| `input_hex_in_expression` | `0x10 * 0x10` | `256` |

### Output Base Conversion

| Test | Input | Base | Expected |
|------|-------|------|----------|
| `output_decimal` | `255` | dec | `255` |
| `output_hexadecimal` | `255` | hex | `ff` |
| `output_octal` | `64` | oct | `100` |
| `output_hex_large` | `65535` | hex | `ffff` |
| `output_octal_large` | `512` | oct | `1000` |

### Digit Grouping (addspaces option)

| Test | Input | Base | Spaces | Expected |
|------|-------|------|--------|----------|
| `grouping_dec_exact_multiple` | `123456789` | dec | yes | `123 456 789` (no leading padding) |
| `grouping_dec_with_padding` | `12345678` | dec | yes | `&nbsp;12 345 678` (1 char padding) |
| `grouping_dec_2_char_pad` | `1234567` | dec | yes | `&nbsp;&nbsp;1 234 567` (2 char padding) |
| `grouping_dec_small` | `42` | dec | yes | `&nbsp;42 ` (padded to 3) |
| `grouping_dec_single_digit` | `5` | dec | yes | `&nbsp;&nbsp;5 ` (padded to 3) |
| `grouping_hex_8_digits` | `0xffffffff` | hex | yes | `ffff ffff` (groups of 4) |
| `grouping_hex_with_padding` | `0xfffff` | hex | yes | `&nbsp;&nbsp;&nbsp;f ffff` (3 char padding) |
| `grouping_oct_6_digits` | `0777777` | oct | yes | Groups of 2: `77 77 77` |
| `grouping_oct_with_padding` | `0777` | oct | yes | `&nbsp;1 77` (padded, 777 octal = 511 dec = 777 oct output) |
| `grouping_disabled_dec` | `12345678` | dec | no | `12345678` (no spaces, line breaks at 60) |
| `grouping_disabled_long` | `2^200` | dec | no | Line breaks with `<br />` at 60 chars |
| `grouping_negative` | `-12345678` | dec | yes | Handles negative sign correctly |
| `grouping_big_number` | `2^3000` | dec | yes | ~904 digits grouped by 3s |

### Big Numbers (1000+ digits)

| Test | Input | Expected |
|------|-------|----------|
| `big_number_2_pow_100` | `2^100` | 31-digit number starting with `1267650600...` |
| `big_number_2_pow_64` | `2^64` | `18446744073709551616` |
| `big_number_2_pow_3000` | `2^3000` | ~904 digit number (verify first/last digits) |
| `big_number_2_pow_10000` | `2^10000` | ~3011 digit number (tests really big) |
| `big_number_arithmetic` | `2^1000 + 2^1000` | Equals `2^1001` |
| `big_number_multiplication` | `2^500 * 2^500` | Equals `2^1000` |
| `big_number_hex_output` | `2^64 - 1` (hex) | `ffffffffffffffff` |
| `big_number_with_spaces` | `2^3000` (addspaces) | Grouped by 3s with leading &nbsp; padding |

### Operator Precedence (critical for parser compatibility)

| Test | Input | Expected | Notes |
|------|-------|----------|-------|
| `prec_exp_right_assoc` | `2^3^2` | `512` | Right-to-left: 2^(3^2)=2^9, NOT (2^3)^2=64 |
| `prec_exp_over_mult` | `2*3^2` | `18` | ^ before *: 2*(3^2) |
| `prec_mult_over_add` | `2+3*4` | `14` | * before +: 2+(3*4) |
| `prec_add_over_shift` | `1<<2+3` | `32` | + before <<: 1<<(2+3)=1<<5 |
| `prec_shift_over_bitor` | `1<<4\|2` | `18` | << before \|: (1<<4)\|2=16\|2 |
| `prec_add_over_bitor` | `2+3\|4` | `5` | + before \|: (2+3)\|4=5\|4 |
| `prec_complex` | `2+3*4^2` | `50` | Full chain: 2+(3*(4^2)) |

### Error Cases

| Test | Input | Expected |
|------|-------|----------|
| `error_invalid_expression` | `abc` | Error: "wasn't recognized as a valid mathematical expression" |
| `error_division_by_zero` | `1/0` | Error message (Ruby ZeroDivisionError) |
| `error_range_operator` | `2..5` | Error (.. is explicitly rejected for security) |
| `error_triple_dot` | `2...5` | Error (contains ..) |
| `error_command_injection` | `system("ls")` | Error (rejected by whitelist) |
| `error_backticks` | `` `ls` `` | Error (rejected by whitelist) |
| `error_eval` | `eval("1")` | Error (rejected by whitelist) |
| `error_special_chars_dollar` | `$foo` | Error ($ not in whitelist) |
| `error_special_chars_at` | `@foo` | Error (@ not in whitelist) |
| `error_semicolon` | `1;2` | Error (; not in whitelist) |
| `error_newline` | `1\n2` | Error (newline not in whitelist) |
| `error_double_quotes` | `"test"` | Error (quotes not in whitelist) |
| `error_single_quotes` | `'test'` | Error (quotes not in whitelist) |
| `error_square_brackets` | `[1,2]` | Error (brackets not in whitelist) |
| `error_curly_braces` | `{1}` | Error (braces not in whitelist) |
| `error_underscore` | `1_000` | Error (_ not in whitelist) |
| `error_hash` | `#comment` | Error (# not in whitelist) |
| `error_colon` | `1:2` | Error (: not in whitelist) |
| `error_question` | `1?2:3` | Error (? not in whitelist) |
| `error_tilde` | `~5` | Error (~ not in whitelist, bitwise NOT) |
| `error_exclamation` | `!true` | Error (! not in whitelist) |
| `error_invalid_octal` | `09` | Error (9 not valid in octal literal) |
| `error_unmatched_paren_open` | `(2+3` | Error (syntax error) |
| `error_unmatched_paren_close` | `2+3)` | Error (syntax error) |
| `error_empty_parens` | `()` | Error (syntax error) |
| `consecutive_operators_work` | `2**+3` | `8` - works as `2**(+3)` (not an error) |
| `error_trailing_operator` | `2+` | Error (incomplete expression) |
| `error_leading_operator_mult` | `*2` | Error (syntax error) |

### Edge Cases

| Test | Input | Expected |
|------|-------|----------|
| `edge_empty_input` | (empty) | No result div shown (form only) |
| `edge_just_number` | `42` | `42` |
| `edge_zero` | `0` | `0` |
| `edge_negative_result` | `5-10` | `-5` |
| `edge_large_negative` | `-2^64` | `-18446744073709551616` |
| `edge_whitespace_around` | ` 2 + 2 ` | `4` (whitespace handled) |
| `edge_whitespace_internal` | `2  +  2` | `4` (multiple spaces ok) |
| `edge_float_input` | `3.14159` | `3.14159` |
| `edge_float_arithmetic` | `3.14 * 2` | `6.28` |
| `edge_nested_parens` | `((2+3)*4)` | `20` |
| `edge_deeply_nested` | `(((1+2)))` | `3` |
| `edge_adjacent_operators` | `2+-3` | `-1` (2 + (-3)) |
| `edge_zero_exponent` | `5^0` | `1` |
| `edge_one_exponent` | `5^1` | `5` |
| `edge_zero_base` | `0^5` | `0` |
| `edge_zero_zero` | `0^0` | `1` (Ruby convention) |

### Word Operator Case Sensitivity

| Test | Input | Expected |
|------|-------|----------|
| `word_or_lowercase` | `5 or 4` | `5` |
| `word_or_uppercase` | `5 OR 4` | `5` |
| `word_or_mixedcase` | `5 Or 4` | `5` |
| `word_and_lowercase` | `5 and 4` | `4` |
| `word_xor_lowercase` | `5 xor 4` | `1` |
| `word_shl_lowercase` | `1 shl 8` | `256` |
| `word_shr_lowercase` | `256 shr 4` | `16` |

### Page Structure

| Test | Description |
|------|-------------|
| `page_loads` | GET request returns 200, contains form |
| `page_has_form_fields` | Form has eqn, base, addspaces, submit |
| `page_has_base_options` | Select has dec, hex, oct options |
| `page_has_operation_help` | Shows supported operations table |

### Negative Numbers with Base Conversion

| Test | Input | Base | Expected |
|------|-------|------|----------|
| `neg_hex_output` | `-255` | hex | `-ff` |
| `neg_oct_output` | `-64` | oct | `-100` |
| `hex_math_hex_output` | `0xff+0xff` | hex | `1fe` |

### Shift Operations Edge Cases

| Test | Input | Expected |
|------|-------|----------|
| `shift_big_number` | `1<<10000` | ~3011 digit number |
| `shift_negative_amount` | `1<<-1` | Error or specific behavior |
| `shift_right_negative` | `-256>>4` | `-16` (arithmetic shift) |

### Exponent Edge Cases

| Test | Input | Expected |
|------|-------|----------|
| `negative_exponent_direct_error` | `2^-3` | Error (not supported) |
| `negative_exponent_workaround` | `(1.0/2)^3` | `0.125` (use reciprocal) |
| `fractional_exponent` | `8^0.5` | `2.828...` (square root) |
| `negative_base_even_exp` | `(-5)^2` | `25` |
| `negative_base_odd_exp` | `(-5)^3` | `-125` |
| `unary_minus_exp` | `-5^2` | `-25` (unary binds looser) |
| `triple_negative` | `---5` | `-5` |

### Division Edge Cases (Ruby convention: result rounds toward negative infinity)

| Test | Input | Expected | Notes |
|------|-------|----------|-------|
| `div_neg_dividend` | `-10/3` | `-4` | NOT -3 (Ruby convention) |
| `div_neg_divisor` | `10/-3` | `-4` | NOT -3 (Ruby convention) |
| `div_both_negative` | `-10/-3` | `3` | Standard positive result |

### Float Operations

| Test | Input | Expected |
|------|-------|----------|
| `float_modulo` | `10.5%3` | `1.5` |
| `float_division` | `7.0/2` | `3.5` |
| `float_negative` | `-3.14` | `-3.14` |

### Chained and Mixed Operations

| Test | Input | Expected | Notes |
|------|-------|----------|-------|
| `chained_addition` | `1+2+3+4+5` | `15` | Left-to-right |
| `chained_multiplication` | `2*3*4` | `24` | Left-to-right |
| `chained_exponentiation` | `2^2^2^2` | `65536` | Right-to-left: 2^(2^(2^2)) |
| `mixed_bitwise_and_or` | `5\|4&3` | `5` | & before \|: 5\|(4&3)=5\|0 |
| `mixed_word_bitwise` | `7 AND 3 OR 4` | `7` | AND before OR |

### Additional Error Cases

| Test | Input | Expected |
|------|-------|----------|
| `error_newline` | `1\n2` | Error (newline not in whitelist) |
| `error_scientific_notation` | `1e5` | Error (e not valid here) |
| `error_whitespace_only` | `   ` | Error or no result |
| `error_equals` | `x=5` | Error (= not in whitelist) |
| `error_backslash` | `1\2` | Error (\ not in whitelist) |

### Additional Edge Cases

| Test | Input | Expected |
|------|-------|----------|
| `edge_empty_input` | (empty) | No result div shown |
| `edge_leading_plus` | `+5` | `5` |
| `edge_plus_minus` | `5+-+3` | `2` (5+(-1*(+3))) |
| `edge_hex_lowercase_x` | `0Xff` | `255` (uppercase X works) |

## Implementation Notes

- **URL:** `/big-number-calculator.htm`
- **Method:** POST
- **Form fields:**
  - `eqn` - the expression to evaluate
  - `base` - output base: "dec", "hex", or "oct"
  - `addspaces` - "yes" to enable digit grouping (checkbox)
  - `submit` - submit button value "Calculate"
- **Result location:** `<div id="bignumber">...</div>`
- **Character whitelist:** `1234567890abcdefABCDEF()*^|&%/+-<>. x`
- **Rejected patterns:** `..` (Ruby range operator)
- **Transformations before eval:**
  - `^` → `**` (exponentiation)
  - `xor` → `^` (bitwise XOR)
  - `or` → `|` (bitwise OR)
  - `and` → `&` (bitwise AND)
  - `shl` → `<<` (shift left)
  - `shr` → `>>` (shift right)
- **Digit grouping intervals:**
  - Decimal: 3 digits
  - Hex: 4 digits
  - Octal: 2 digits

## PHP Implementation Reference

- **Page handler:** `/defuse.ca/src/pages/services/big-number-calculator.php`
- **Backend:** Ruby interpreter with `$SAFE = 1`
- **Timeout:** 8-10 seconds
- **Result types:** Integer (Fixnum/Bignum), Float
