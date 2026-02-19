//! Checksums page tests
//!
//! Tests for the checksums calculator functionality.

mod common;

use common::{post_form, post_multipart_file};
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};

/// Complete test vectors for input "test" (all 57 hash types)
const TEST_VECTORS: &[(&str, &str)] = &[
    ("adler32", "045d01c1"),
    ("crc32", "accf8b33"),
    ("crc32b", "d87f7e0c"),
    ("crc32c", "86a072c0"),
    ("fnv132", "bc2c0be9"),
    ("fnv164", "8c093f7e9fccbf69"),
    ("fnv1a32", "afd071e5"),
    ("fnv1a64", "f9e6e6ef197c2b25"),
    ("gost", "a6e1acdd0cc7e00d02b90bccb2e21892289d1e93f622b8760cb0e076def1f42b"),
    ("gost-crypto", "ee67303696d205ddd2b2363e8e01b4b7199a80957d94d7678eaad3fc834c5a27"),
    ("haval128,3", "a26075021e24a5bda74794d85e9fdb7f"),
    ("haval128,4", "1ba3b2186ad54d024603d61ddb9d2f42"),
    ("haval128,5", "f5b480f6965efd5f5e6232925c5eed14"),
    ("haval160,3", "858c2c8f76afa7dd067d3d94c667c8aec6ac2650"),
    ("haval160,4", "516d3243a12ce3af38a005003c7221bf85299714"),
    ("haval160,5", "f5e3770031ebc6c46fe78d92890e17b1bef93b87"),
    ("haval192,3", "c4b8741917dabc27e2bebf58a6663a05b0d3dc43072a64b4"),
    ("haval192,4", "16ff6de6751cb654c1f788ee2f14ceddb86eec343ef87cd5"),
    ("haval192,5", "527383196142f6f3352f8a152dd06c9c0a50efcb83a646f0"),
    ("haval224,3", "f5b30a47580d8bfa256d6ed7604ffd2bb787abb22b53ad9f693e8d31"),
    ("haval224,4", "deea192a84b5e29ab958202b22a0b604c1df1298ee7d32ee5d7e2954"),
    ("haval224,5", "9666797abc57d096c2a9922e350390437f9c2e378ce2e43e0d816d90"),
    ("haval256,3", "593c9aed973bb51a3c852fb4e051d7c26686b9468b4e405350cb6805dc1b99e6"),
    ("haval256,4", "435ded7266cba07f389d6e74c954b184e1ddacc8a7b8dc022db3ca4450a738cd"),
    ("haval256,5", "a4b59d68e0111000856baca9e6573a2adc2b56b6b4d87f7cf31de24a77b93768"),
    ("joaat", "3f75ccc1"),
    ("LM", "01fc5a6be7bc6929aad3b435b51404ee"),
    ("md2", "dd34716876364a02d0195e2fb9ae2d1b"),
    ("md4", "db346d691d7acc4dc2625db19f9e3f52"),
    ("md5", "098f6bcd4621d373cade4e832627b4f6"),
    ("md5(md5())", "fb469d7ef430b0baf0cab6c436e70375"),
    ("MySQL4.1+", "94bdcebe19083ce2a1f959fd02f964c7af4cfc29"),
    ("NTLM", "0cb6948805f797bf2a82807973b89537"),
    ("ripemd128", "f1abb5083c9ff8a9dbbca9cd2b11fead"),
    ("ripemd160", "5e52fee47e6b070565f74372468cdc699de89107"),
    ("ripemd256", "fe0289110d07daeee9d9500e14c57787d9083f6ba10e6bcb256f86bb4fe7b981"),
    ("ripemd320", "3b0a2e841e589cf583634a5dd265d2b5d497c4cc44b241e34e0f62d03e98c1b9dc72970b9bc20eb5"),
    ("sha1", "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3"),
    ("sha224", "90a3ed9e32b2aaf4c61c410eb925426119e1a9dc53d4286ade99a809"),
    ("sha256", "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"),
    ("sha3-224", "3797bf0afbbfca4a7bbba7602a2b552746876517a7f9b7ce2db0ae7b"),
    ("sha3-256", "36f028580bb02cc8272a9a020f4200e346e276ae664e45ee80745574e2f5ab80"),
    ("sha3-384", "e516dabb23b6e30026863543282780a3ae0dccf05551cf0295178d7ff0f1b41eecb9db3ff219007c4e097260d58621bd"),
    ("sha3-512", "9ece086e9bac491fac5c1d1046ca11d737b92a2b2ebd93f005d7b710110c0a678288166e7fbe796883a4f2e9b3ca9f484f521d0ce464345cc1aec96779149c14"),
    ("sha384", "768412320f7b0aa5812fce428dc4706b3cae50e02a64caa16a782249bfe8efc4b7ef1ccb126255d196047dfedf17a0a9"),
    ("sha512", "ee26b0dd4af7e749aa1a8ee3c10ae9923f618980772e473f8819a5d4940e0db27ac185f8a0e1d5f84f88bc887fd67b143732c304cc5fa9ad8e6f57f50028a8ff"),
    ("sha512/224", "06001bf08dfb17d2b54925116823be230e98b5c6c278303bc4909a8c"),
    ("sha512/256", "3d37fe58435e0d87323dee4a2c1b339ef954de63716ee79f5747f94d974f913f"),
    ("snefru", "8d25dd0b5715f7e4c799ade3a34b5f6148d0ce416992b5c2eaf614d35d5b3d30"),
    ("snefru256", "8d25dd0b5715f7e4c799ade3a34b5f6148d0ce416992b5c2eaf614d35d5b3d30"),
    ("tiger128,3", "7ab383fc29d81f8d0d68e87c69bae5f1"),
    ("tiger128,4", "14b5375c7b29cbf5f9e70a199a40e59d"),
    ("tiger160,3", "7ab383fc29d81f8d0d68e87c69bae5f1f18266d7"),
    ("tiger160,4", "14b5375c7b29cbf5f9e70a199a40e59dd4d5f1df"),
    ("tiger192,3", "7ab383fc29d81f8d0d68e87c69bae5f1f18266d730c48b1d"),
    ("tiger192,4", "14b5375c7b29cbf5f9e70a199a40e59dd4d5f1df218b5249"),
    ("whirlpool", "b913d5bbb8e461c2c5961cbe0edcdadfd29f068225ceb37da6defcf89849368f8c6c2eb6a4c4ac75775d032a0ecfdfe8550573062b653fe92fc7b8fb3b7be8d6"),
];

/// Expected hash names (all 57)
const EXPECTED_HASH_NAMES: &[&str] = &[
    "adler32", "crc32", "crc32b", "crc32c", "fnv132", "fnv164", "fnv1a32", "fnv1a64",
    "gost", "gost-crypto",
    "haval128,3", "haval128,4", "haval128,5",
    "haval160,3", "haval160,4", "haval160,5",
    "haval192,3", "haval192,4", "haval192,5",
    "haval224,3", "haval224,4", "haval224,5",
    "haval256,3", "haval256,4", "haval256,5",
    "joaat", "LM", "md2", "md4", "md5", "md5(md5())", "MySQL4.1+", "NTLM",
    "ripemd128", "ripemd160", "ripemd256", "ripemd320",
    "sha1", "sha224", "sha256",
    "sha3-224", "sha3-256", "sha3-384", "sha3-512",
    "sha384", "sha512", "sha512/224", "sha512/256",
    "snefru", "snefru256",
    "tiger128,3", "tiger128,4", "tiger160,3", "tiger160,4", "tiger192,3", "tiger192,4",
    "whirlpool",
];

/// Extract hash name/value pairs from checksums page HTML response.
/// Returns a map from hash name to hash value.
fn extract_hashes(html: &str) -> HashMap<String, String> {
    let document = Html::parse_document(html);

    // Target the specific checksums table: the table following the checksums anchor
    let table_selector = Selector::parse("a[name=\"checksums\"] ~ table").unwrap();
    let tr_selector = Selector::parse("tr").unwrap();
    let th_selector = Selector::parse("th").unwrap();
    let td_selector = Selector::parse("td").unwrap();

    let mut hashes = HashMap::new();

    // Get the checksums table specifically
    let table = document.select(&table_selector).next()
        .expect("Could not find checksums table (a[name=\"checksums\"] ~ table)");

    for tr in table.select(&tr_selector) {
        let ths: Vec<_> = tr.select(&th_selector).collect();
        let tds: Vec<_> = tr.select(&td_selector).collect();

        if let (Some(th), Some(td)) = (ths.first(), tds.first()) {
            let hash_name = th.text().collect::<String>();
            let hash_value = td.text().collect::<String>();

            if !hash_name.is_empty() && !hash_value.is_empty() {
                hashes.insert(hash_name, hash_value);
            }
        }
    }

    hashes
}

/// Verify all test vectors match.
/// Returns a list of mismatches for reporting.
fn verify_test_vectors(hashes: &HashMap<String, String>) -> Vec<String> {
    let mut errors = Vec::new();

    for (name, expected) in TEST_VECTORS {
        match hashes.get(*name) {
            Some(actual) if actual == *expected => {
                // Match - good
            }
            Some(actual) => {
                errors.push(format!(
                    "{}: expected '{}', got '{}'",
                    name, expected, actual
                ));
            }
            None => {
                errors.push(format!("{}: not found in response", name));
            }
        }
    }

    errors
}

#[tokio::test]
async fn hash_test_vectors() {
    let resp = post_form("/checksums.htm", &[("data", "test"), ("normalize", "no")]).await;
    assert!(
        resp.status().is_success(),
        "POST to checksums failed: {}",
        resp.status()
    );

    let body = resp.text().await.expect("Failed to read response body");
    let hashes = extract_hashes(&body);

    let errors = verify_test_vectors(&hashes);

    if !errors.is_empty() {
        panic!(
            "Hash test vector mismatches ({} errors):\n  {}",
            errors.len(),
            errors.join("\n  ")
        );
    }
}

#[tokio::test]
async fn normalize_surrounding_newlines() {
    // "\ntest\n" with normalize=yes should produce same hashes as "test"
    let resp = post_form("/checksums.htm", &[("data", "\ntest\n"), ("normalize", "yes")]).await;
    assert!(
        resp.status().is_success(),
        "POST to checksums failed: {}",
        resp.status()
    );

    let body = resp.text().await.expect("Failed to read response body");
    let hashes = extract_hashes(&body);

    let errors = verify_test_vectors(&hashes);

    if !errors.is_empty() {
        panic!(
            "normalize_surrounding_newlines: hashes don't match 'test' vectors ({} errors):\n  {}",
            errors.len(),
            errors.join("\n  ")
        );
    }
}

/// Test vectors for "testtest" (used to verify normalize removes internal line endings)
const TESTTEST_MD5: &str = "05a671c66aefea124cc08b76ea6d30bb";

#[tokio::test]
async fn normalize_removes_internal_line_endings() {
    // "test\ntest" with normalize=yes should produce same hashes as "testtest"
    // PHP removes ALL \r and \n characters, not just leading/trailing whitespace
    let resp = post_form("/checksums.htm", &[("data", "test\ntest"), ("normalize", "yes")]).await;
    assert!(
        resp.status().is_success(),
        "POST to checksums failed: {}",
        resp.status()
    );

    let body = resp.text().await.expect("Failed to read response body");
    let hashes = extract_hashes(&body);

    let md5_hash = hashes.get("md5").expect("md5 hash not found");
    assert_eq!(
        md5_hash, TESTTEST_MD5,
        "normalize should remove internal line endings: 'test\\ntest' with normalize=yes should hash as 'testtest'. Got md5='{}', expected '{}'",
        md5_hash, TESTTEST_MD5
    );
}

#[tokio::test]
async fn normalize_removes_carriage_returns() {
    // "test\r\ntest" with normalize=yes should produce same hashes as "testtest"
    let resp = post_form("/checksums.htm", &[("data", "test\r\ntest"), ("normalize", "yes")]).await;
    assert!(
        resp.status().is_success(),
        "POST to checksums failed: {}",
        resp.status()
    );

    let body = resp.text().await.expect("Failed to read response body");
    let hashes = extract_hashes(&body);

    let md5_hash = hashes.get("md5").expect("md5 hash not found");
    assert_eq!(
        md5_hash, TESTTEST_MD5,
        "normalize should remove \\r\\n: 'test\\r\\ntest' with normalize=yes should hash as 'testtest'. Got md5='{}', expected '{}'",
        md5_hash, TESTTEST_MD5
    );
}

#[tokio::test]
async fn no_extra_hash_types() {
    let resp = post_form("/checksums.htm", &[("data", "test"), ("normalize", "no")]).await;
    assert!(
        resp.status().is_success(),
        "POST to checksums failed: {}",
        resp.status()
    );

    let body = resp.text().await.expect("Failed to read response body");
    let hashes = extract_hashes(&body);

    let expected_set: HashSet<&str> = EXPECTED_HASH_NAMES.iter().copied().collect();
    let actual_set: HashSet<&str> = hashes.keys().map(|s| s.as_str()).collect();

    // Check for extra hash types
    let extra: Vec<_> = actual_set.difference(&expected_set).collect();
    if !extra.is_empty() {
        panic!(
            "Found {} unexpected hash types: {:?}",
            extra.len(),
            extra
        );
    }

    // Check for missing hash types
    let missing: Vec<_> = expected_set.difference(&actual_set).collect();
    if !missing.is_empty() {
        panic!(
            "Missing {} expected hash types: {:?}",
            missing.len(),
            missing
        );
    }
}

// =============================================================================
// File upload tests
// =============================================================================

/// Maximum file size allowed by PHP: 5 * 1024 * 1024 = 5,242,880 bytes
const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;

/// MD5 hash of 5MB of null bytes (for verifying max size file upload works)
const NULL_5MB_MD5: &str = "5f363e0e58a95f06cbe9bbc662c5dfb6";

#[tokio::test]
async fn file_upload_max_size() {
    // Create a file exactly at the max size limit (5MB of null bytes)
    let file_content = vec![0u8; MAX_FILE_SIZE];

    let resp = post_multipart_file(
        "/checksums.htm",
        "filetohash",
        "test_5mb.bin",
        file_content,
    )
    .await;

    assert!(
        resp.status().is_success(),
        "File upload at max size should succeed: {}",
        resp.status()
    );

    let body = resp.text().await.expect("Failed to read response body");
    let hashes = extract_hashes(&body);

    let md5_hash = hashes.get("md5").expect("md5 hash not found in response");
    assert_eq!(
        md5_hash, NULL_5MB_MD5,
        "MD5 of 5MB null file should be '{}', got '{}'",
        NULL_5MB_MD5, md5_hash
    );
}

#[tokio::test]
async fn file_upload_over_limit() {
    // Create a file 1 byte over the limit
    let file_content = vec![0u8; MAX_FILE_SIZE + 1];

    let resp = post_multipart_file(
        "/checksums.htm",
        "filetohash",
        "test_over_5mb.bin",
        file_content,
    )
    .await;

    // The response should still be 200 OK but contain an error message
    let body = resp.text().await.expect("Failed to read response body");

    assert!(
        body.contains("File is too big"),
        "Expected error message 'File is too big'. Response body: {}...",
        &body.chars().take(500).collect::<String>()
    );
}

#[tokio::test]
async fn file_upload_small_file() {
    // Test with a small known file content
    let file_content = b"test".to_vec();

    let resp = post_multipart_file(
        "/checksums.htm",
        "filetohash",
        "test.txt",
        file_content,
    )
    .await;

    assert!(
        resp.status().is_success(),
        "Small file upload should succeed: {}",
        resp.status()
    );

    let body = resp.text().await.expect("Failed to read response body");
    let hashes = extract_hashes(&body);

    // Should produce the same hashes as text input "test"
    let errors = verify_test_vectors(&hashes);

    if !errors.is_empty() {
        panic!(
            "File upload hashes don't match 'test' vectors ({} errors):\n  {}",
            errors.len(),
            errors.join("\n  ")
        );
    }
}

/// Test LM hash with a 14-byte binary input (fills both 7-byte DES key halves).
/// Uses bytes with high bits set to exercise all bit positions in the 7-to-8 byte
/// key expansion, where parity bit calculation differences would surface.
/// Expected value from production PHP site.
#[tokio::test]
async fn lm_hash_full_14_byte_binary_input() {
    let file_content: Vec<u8> = vec![
        0xFF, 0x80, 0xC3, 0xA5, 0x96, 0x7E, 0xDB,
        0xE1, 0xF0, 0x0F, 0xAA, 0x55, 0xCC, 0x33,
    ];
    let resp = post_multipart_file(
        "/checksums.htm",
        "filetohash",
        "test.bin",
        file_content,
    )
    .await;
    assert!(resp.status().is_success());
    let body = resp.text().await.unwrap();
    let hashes = extract_hashes(&body);
    let lm = hashes.get("LM").expect("LM hash not found");
    assert_eq!(
        lm, "edc42736d551fef23aba406453d0e148",
        "LM hash of 14-byte binary input should match PHP output"
    );
}

/// Test that binary files with invalid UTF-8 produce correct LM/NTLM hashes.
/// PHP treats strings as raw bytes, so bytes 0x80-0xFF are processed directly.
/// Rust must NOT use String::from_utf8_lossy which replaces invalid UTF-8 with U+FFFD.
#[tokio::test]
async fn file_upload_binary_lm_hash() {
    // Binary content with bytes that are invalid UTF-8 (0x80-0xFF are invalid lead bytes)
    // These bytes are valid Latin-1 characters that PHP handles directly.
    let file_content: Vec<u8> = vec![0x80, 0x81, 0x82, 0x83];

    // For LM: bytes are uppercased (no effect on 0x80-0x83), padded to 14, split, DES encrypted
    const EXPECTED_LM: &str = "e5bb7d8e7d081c54aad3b435b51404ee";

    let resp = post_multipart_file(
        "/checksums.htm",
        "filetohash",
        "binary.bin",
        file_content,
    )
    .await;

    assert!(
        resp.status().is_success(),
        "Binary file upload should succeed: {}",
        resp.status()
    );

    let body = resp.text().await.expect("Failed to read response body");
    let hashes = extract_hashes(&body);

    let lm_hash = hashes.get("LM").expect("LM hash not found");
    assert_eq!(
        lm_hash, EXPECTED_LM,
        "LM hash for binary data [0x80, 0x81, 0x82, 0x83] should be '{}', got '{}'",
        EXPECTED_LM, lm_hash
    );
}

/// Test that NTLM correctly handles binary data with invalid UTF-8 bytes.
///
/// BUG: PHP's NTLMHash function uses iconv('UTF-8', 'UTF-16LE', $input) which
/// silently fails on invalid UTF-8 (due to @ error suppression), returning false.
/// hash('md4', false) coerces false to empty string, so ANY invalid UTF-8 input
/// returns the MD4 hash of empty string: 31d6cfe0d16ae931b73c59d7e0c089c0
///
/// CORRECT behavior: Treat each byte as a Latin-1 code point and convert to UTF-16LE.
/// For input [0x80, 0x81, 0x82, 0x83], UTF-16LE should be [0x80,0x00, 0x81,0x00, 0x82,0x00, 0x83,0x00]
/// then MD4 hash that result.
///
/// See: defuse-rust/docs/issues/2026-01-18-ntlm-hash-invalid-utf8.md
#[tokio::test]
async fn file_upload_binary_ntlm_hash() {
    let file_content: Vec<u8> = vec![0x80, 0x81, 0x82, 0x83];

    let resp = post_multipart_file(
        "/checksums.htm",
        "filetohash",
        "binary.bin",
        file_content,
    )
    .await;

    assert!(
        resp.status().is_success(),
        "Binary file upload should succeed: {}",
        resp.status()
    );

    let body = resp.text().await.expect("Failed to read response body");
    let hashes = extract_hashes(&body);

    let ntlm_hash = hashes.get("NTLM").expect("NTLM hash not found");

    // NTLM requires valid text input (UTF-8 -> UTF-16LE). Binary data should show error.
    // This matches Windows behavior where NTLM is only for text passwords.
    assert!(
        ntlm_hash.contains("error"),
        "NTLM should return an error for binary (non-UTF-8) input. Got: '{}'",
        ntlm_hash
    );
}

/// normalize=no should preserve line endings in the hash input.
const TEST_NEWLINE_TEST_MD5: &str = "be778b473235e210cc577056226536a4";

#[tokio::test]
async fn normalize_no_preserves_line_endings() {
    let resp = post_form("/checksums.htm", &[("data", "test\ntest"), ("normalize", "no")]).await;
    assert!(resp.status().is_success(), "POST to checksums failed: {}", resp.status());

    let body = resp.text().await.expect("Failed to read response body");
    let hashes = extract_hashes(&body);

    let md5_hash = hashes.get("md5").expect("md5 hash not found");
    assert_eq!(
        md5_hash, TEST_NEWLINE_TEST_MD5,
        "normalize=no should preserve line endings: md5('test\\ntest') should be '{}', got '{}'",
        TEST_NEWLINE_TEST_MD5, md5_hash
    );
}

/// Test that empty file uploads produce hashes of empty string (not "no results").
#[tokio::test]
async fn file_upload_empty_file() {
    let file_content: Vec<u8> = vec![];

    // MD5 of empty string
    const EXPECTED_MD5: &str = "d41d8cd98f00b204e9800998ecf8427e";

    let resp = post_multipart_file(
        "/checksums.htm",
        "filetohash",
        "empty.bin",
        file_content,
    )
    .await;

    assert!(
        resp.status().is_success(),
        "Empty file upload should succeed: {}",
        resp.status()
    );

    let body = resp.text().await.expect("Failed to read response body");
    let hashes = extract_hashes(&body);

    let md5_hash = hashes.get("md5");
    assert_eq!(
        md5_hash,
        Some(&EXPECTED_MD5.to_string()),
        "Empty file should produce hash of empty string, got {:?}",
        md5_hash
    );
}
