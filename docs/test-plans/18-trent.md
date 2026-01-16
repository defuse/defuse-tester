# TRENT (Trusted Random Entropy) Test Plan

**Status:** DONE (89 tests, 1 ignored)

**File:** `tests/trent.rs`

## Overview

TRENT is a trusted third party random number generator for drawings, contests, and lotteries. It provides a verifiable way to generate random numbers that neither party can manipulate.

Key features:
- Two-step process: reserve drawing number, then generate random numbers
- Configurable review period (instant to 30 days) prevents repeating until desired result
- Up to 3 file uploads with SHA256 checksums for verifiability
- Random line selection from uploaded files
- Random number generation in a specified range
- Permanent, immutable results once drawing is complete

## Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/trustedthirdparty.htm` | GET | Main page with forms |
| `/trustedthirdparty.htm?drawingnum=X` | GET | View drawing status/results |
| `/trustedthirdparty.htm` | POST | Reserve drawing (makedrawingnumber) |
| `/trustedthirdparty.htm` | POST | Create random numbers (create) |

## Test Cases

### 1. Main Page (GET)

| Test | Description | Expected |
|------|-------------|----------|
| `main_page_loads` | GET /trustedthirdparty.htm | 200 |
| `main_page_has_title` | Check page title | Contains "TRENT" |
| `main_page_shows_utc_time` | Check for clock display | Contains "Current UTC Time:" |
| `main_page_has_validation_form` | Check drawing validation form | Has input name="drawingnum" and submit button "Check" |
| `main_page_has_reservation_form` | Check drawing reservation form | Has select name="prereview" and submit name="makedrawingnumber" |
| `main_page_has_review_time_options` | Check review time dropdown | Has options: 0, 21600, 86400, 345600, 518400, 1123200, 2592000 |
| `main_page_has_creation_form` | Check random number creation form | Has inputs: drawingnumber, passcode, name, description, file1-3, randlines1-3, lowval, highval, numgen |
| `main_page_has_file_inputs` | Check file upload fields | Has 3 file inputs with 10MB max note |
| `main_page_has_chosentwice_checkbox` | Check "allow lines chosen twice" | Has checkbox name="chosentwice" |
| `main_page_has_explanation` | Check explanatory content | Contains "What is TRENT?" |
| `main_page_has_trent_image` | Check TRENT logo | Contains img with "trusted_third_party.png" |

### 2. View Drawing Status (GET ?drawingnum=X)

| Test | Description | Expected |
|------|-------------|----------|
| `view_nonexistent_drawing` | GET ?drawingnum=999999999 | Shows "does not exist" error (red background) |
| `view_drawing_number_zero` | GET ?drawingnum=0 | Shows "does not exist" error |
| `view_drawing_negative_number` | GET ?drawingnum=-1 | Shows "does not exist" error or handles gracefully |
| `view_drawing_non_numeric` | GET ?drawingnum=abc | Shows "does not exist" for drawing 0 (casts to int) |
| `view_pending_drawing` | GET existing pending drawing | Shows "not yet been chosen" message with future date |
| `view_completed_drawing` | GET existing completed drawing | Shows description textarea and results textarea |
| `view_completed_has_description` | Check completed drawing | Has "Description of drawing #X" section |
| `view_completed_has_results` | Check completed drawing | Has "Results for drawing #X" section |
| `view_completed_has_verification_info` | Check completed drawing | Has "How do I tell if the drawing was fair?" section |

### 3. Reserve Drawing Number (POST makedrawingnumber)

| Test | Description | Expected |
|------|-------------|----------|
| `reserve_instant` | POST prereview=0 | Returns drawing number, passcode, immediate availability |
| `reserve_6_hours` | POST prereview=21600 | Returns drawing number with ~6 hour future date |
| `reserve_24_hours` | POST prereview=86400 | Returns drawing number with ~24 hour future date |
| `reserve_returns_drawing_number` | Check response | Contains "Your drawing number is:" followed by number |
| `reserve_returns_passcode` | Check response | Contains "Your passcode is:" followed by 32 hex chars |
| `reserve_returns_url` | Check response | Contains URL to trustedthirdparty.htm?drawingnum=X |
| `reserve_shows_availability_date` | Check response | Contains "You will be able to do the drawing after" |
| `reserve_drawing_viewable` | Reserve then view | Drawing is viewable via ?drawingnum=X |
| `reserve_creates_unique_numbers` | Reserve twice | Two different drawing numbers returned |

### 4. Create Random Numbers - Validation Errors

| Test | Description | Expected |
|------|-------------|----------|
| `create_wrong_password` | POST with incorrect passcode | Shows "Incorrect password" error (red) |
| `create_drawing_not_exists` | POST with non-existent drawing number | Shows "does not exist" error |
| `create_already_complete` | POST on already-completed drawing | Shows "already been chosen" error |
| `create_review_not_complete` | POST before review period ends | Shows "review period not complete" error with date |
| `create_invalid_range_low_ge_high` | POST lowval >= highval with numgen > 0 | Shows "range is invalid" error |
| `create_negative_numgen` | POST numgen=-1 | Shows "NEGATIVE amount" error |
| `create_negative_randlines` | POST randlines1=-1 | Shows "NEGATIVE amount" error |
| `create_too_many_numbers` | POST numgen=1001 | Shows "only generate 1000" error |
| `create_too_many_randlines` | POST randlines1=1001 | Shows "only generate 1000" error |
| `create_file_too_large` | POST with >10MB file | Shows "maximum file size is 10MB" error |
| `create_not_enough_lines` | POST randlines > file line count (no repeat) | Shows "doesn't have enough lines" error |

### 5. Create Random Numbers - Confirmation Step

| Test | Description | Expected |
|------|-------------|----------|
| `create_shows_confirmation` | POST valid params (no confirmed=true) | Shows confirmation page with parameters |
| `confirm_shows_name` | Check confirmation | Displays submitted name |
| `confirm_shows_description` | Check confirmation | Displays submitted description |
| `confirm_shows_file_info` | Check confirmation with file | Shows file name, size, SHA256 |
| `confirm_shows_randlines` | Check confirmation | Shows random lines count for each file |
| `confirm_shows_range` | Check confirmation | Shows "X to Y" range |
| `confirm_shows_numgen` | Check confirmation | Shows amount of numbers |
| `confirm_has_hidden_fields` | Check confirmation form | Has hidden fields for all parameters |
| `confirm_has_confirm_button` | Check confirmation | Has submit button to confirm |
| `confirm_has_cancel_button` | Check confirmation | Has cancel button |

### 6. Create Random Numbers - Success

| Test | Description | Expected |
|------|-------------|----------|
| `create_success_message` | POST with confirmed=true | Shows "Drawing Complete!" message |
| `create_success_has_url` | Check success | Shows URL to results page |
| `create_results_viewable` | Create then view | Results visible via ?drawingnum=X |
| `create_results_has_drawing_number` | Check results | Contains "DRAWING NUMBER: X" |
| `create_results_has_date` | Check results | Contains "DRAWING DATE:" with timestamp |
| `create_results_has_amount` | Check results | Contains "AMOUNT OF NUMBERS: X" |
| `create_results_has_range` | Check results | Contains "RANGE: X TO Y" |
| `create_results_has_random_numbers` | Check results with numgen > 0 | Contains "RANDOM NUMBER NUMBER X:" lines |
| `create_user_description_preserved` | Check results | User's name and description in first textarea |

### 7. Random Number Generation

| Test | Description | Expected |
|------|-------------|----------|
| `random_numbers_in_range` | Generate numbers, check range | All numbers between lowval and highval inclusive |
| `random_single_number` | numgen=1 | Exactly one "RANDOM NUMBER NUMBER 1:" line |
| `random_multiple_numbers` | numgen=5 | Five "RANDOM NUMBER NUMBER X:" lines (1-5) |
| `random_zero_numbers` | numgen=0 | No "RANDOM NUMBER NUMBER" lines (valid) |
| `random_range_single_value` | lowval=highval=5, numgen=1 | Should work (range is valid when equal if numgen=0... wait, code says low >= high is invalid if numgen != 0) |
| `random_large_range` | lowval=-1000000000, highval=1000000000 | Numbers in valid range |
| `random_negative_range` | lowval=-100, highval=-1 | Negative numbers in range |

### 8. File Upload and Random Lines

| Test | Description | Expected |
|------|-------------|----------|
| `file_sha256_recorded` | Upload file, complete drawing | Results contain "FILE1 SHA256:" with correct hash |
| `file_random_lines_selected` | Upload file, randlines1=2 | Results contain "FILE1 RANDOM LINE 1:" and "FILE1 RANDOM LINE 2:" |
| `file_line_preview_shown` | Upload file with known content | Results contain "LINE PREVIEW:" with actual line content |
| `file_line_number_shown` | Upload file, request random line | Results contain "RANDOM LINE NUMBER (FILE1):" |
| `file_multiple_files` | Upload 3 files | All 3 file hashes in results |
| `file_no_repeat_lines` | Upload file, request multiple lines, no chosentwice | All line numbers are different |
| `file_allow_repeat_lines` | Upload file, chosentwice=true | Same line can be chosen multiple times |
| `file_zero_randlines` | Upload file but randlines1=0 | File hash recorded but no random lines |

### 9. Edge Cases

| Test | Description | Expected |
|------|-------------|----------|
| `name_with_unicode` | Name contains "Über Tëst 日本語" | Unicode preserved in results |
| `description_with_html` | Description contains `<script>alert(1)</script>` | HTML escaped in display |
| `description_multiline` | Description with multiple lines | Newlines preserved |
| `empty_name` | Name is empty string | Accepted (name is optional) |
| `empty_description` | Description is empty | Accepted |
| `no_files_no_numbers` | No files, numgen=0 | Valid drawing with just description |
| `whitespace_passcode` | Passcode with leading/trailing spaces | Trimmed and verified |
| `max_numbers_1000` | numgen=1000 | All 1000 numbers generated |
| `drawing_number_large` | Very large drawing number lookup | Handles gracefully |

### 10. Security

| Test | Description | Expected |
|------|-------------|----------|
| `password_case_sensitive` | Try password with wrong case | Incorrect password error |
| `drawing_immutable_after_complete` | Try to recreate completed drawing | "already been chosen" error |
| `review_period_enforced` | Try to complete before review period | "review period not complete" error |
| `xss_prevention_name` | Check HTML escaping in name | Special chars escaped with htmlentities |
| `xss_prevention_description` | Check HTML escaping in description | Special chars escaped |

### 11. Review Period Timing

| Test | Description | Expected |
|------|-------------|----------|
| `review_period_blocks_then_allows` | Reserve with prereview=5, try immediately (fail), wait 8s, try again (success) | First attempt blocked, second succeeds after wait |
| `review_period_custom_value` | Reserve with non-standard prereview (e.g., 3) | Server accepts custom value (no dropdown validation) |

Note: The PHP code does `(int)$_POST['prereview']` without validating against dropdown options, so we can use small values like 5 seconds for testing.

### 12. Content and Headers

| Test | Description | Expected |
|------|-------------|----------|
| `main_page_content_type` | Check Content-Type | text/html |
| `main_page_has_source_link` | Check for source code link | Contains link to GitHub source |

## Implementation Notes

### Form Fields for Reserve (POST makedrawingnumber)

```
makedrawingnumber: submit button name (presence triggers reservation)
prereview: review time in seconds (0, 21600, 86400, 345600, 518400, 1123200, 2592000)
```

### Form Fields for Create (POST create)

```
create: submit button name (presence triggers creation)
drawingnumber: integer
passcode: string (32 hex chars)
name: string (optional)
description: string (optional, multiline)
file1, file2, file3: file uploads (10MB max each)
randlines1, randlines2, randlines3: integer (lines to select from each file)
chosentwice: "true" or absent (allow same line multiple times)
lowval: integer (range low, -1000000000 to +1000000000)
highval: integer (range high, -1000000000 to +1000000000)
numgen: integer (amount of random numbers, 0-1000)
```

### Confirmation Step Fields (POST create with confirmed=true)

```
confirmed: "true"
drawingnumber, passcode, name, description: same as above
file1hash, file2hash, file3hash: SHA256 of uploaded files (hex)
randlines1, randlines2, randlines3: same as above
chosentwice: "true" or absent
lowval, highval, numgen: same as above
```

### Server Behavior

1. **Reserve**: Creates database row with complete=0, stores password hash, start time, review time
2. **Review Period**: Enforced via starttime + reviewtime >= current time
3. **Password**: SHA256 hash compared (not stored in plaintext)
4. **Files**: Moved to /tmp/{sha256} for confirmation step, read from there for final creation
5. **Random Generation**: Uses mcrypt_create_iv(32, MCRYPT_DEV_URANDOM)
6. **Immutability**: Once complete=1, no modifications allowed

### What We're NOT Testing

- Actual randomness quality (would need statistical tests)
- File upload size limits at server level (PHP/Apache config)
- Database persistence across server restarts
- Concurrent access / race conditions
- File cleanup in /tmp (server maintenance)

## Test Dependencies

- Tests that create drawings will persist in the database
- Use "instant" review time (prereview=0) for immediate completion tests
- Consider using unique identifiers in name/description to identify test drawings
- Some tests require creating a drawing first, then completing it

## Test Strategy

Since TRENT creates permanent database entries, tests should:
1. Use distinctive test names (e.g., "TRENT_TEST_[timestamp]")
2. Not rely on specific drawing numbers (they auto-increment)
3. Create fresh drawings for each test that needs one
4. Use instant review time (0) for tests that need to complete drawings

## Existing Drawings for View Tests

To test viewing completed drawings without creating new ones, we could:
1. Look up an existing completed drawing number on production
2. Create test drawings during test setup
3. Mark view tests as requiring setup

For the Rust port, consider:
- Test mode that auto-cleans test drawings
- Separate test database
- API to delete test drawings
