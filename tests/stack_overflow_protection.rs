//! Stack Overflow Protection Tests
//!
//! Tests that deeply-nested expressions in the big number calculator don't crash
//! the server via stack overflow. Stack overflows call abort() directly, bypassing
//! CatchPanicLayer, so the pest parser's call limit must prevent deep recursion.
//!
//! This test sends expressions at many different nesting depths to find any depth
//! that overflows the stack before the call limit kicks in. If such a depth exists,
//! the server will crash and subsequent requests will fail.

mod common;

use common::{client, url};

/// Build a valid deeply-nested expression: (1+(1+(1+...+1)...))
/// Each nesting level adds one open paren, "1+", and one close paren.
fn nested_expression(depth: usize) -> String {
    let mut expr = String::with_capacity(depth * 4 + 1);
    for _ in 0..depth {
        expr.push_str("(1+");
    }
    expr.push('1');
    for _ in 0..depth {
        expr.push(')');
    }
    expr
}

/// Send an expression to the calculator.
/// Returns Ok(Some(result_string)) if the expression computed successfully,
/// Ok(None) if the expression was rejected (got "Sorry" error), or Err if the
/// server didn't respond (crashed or timed out).
async fn try_calc(expr: &str) -> Result<Option<String>, String> {
    let client = client();
    let resp = client
        .post(url("/big-number-calculator.htm"))
        .form(&[
            ("eqn", expr),
            ("base", "dec"),
            ("submit", "Calculate"),
        ])
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let body = resp.text().await.map_err(|e| format!("Body read failed: {}", e))?;

    if body.contains("Sorry,") {
        Ok(None) // Expression rejected by parser/filter (expected for deep nesting)
    } else {
        // Extract the result from the bignumber div
        let marker = r#"id="bignumber""#;
        if let Some(pos) = body.find(marker) {
            let after = &body[pos..];
            if let Some(gt) = after.find('>') {
                let content = &after[gt + 1..];
                if let Some(end) = content.find("</div>") {
                    let result = content[..end].trim();
                    if !result.is_empty() {
                        return Ok(Some(result.to_string()));
                    }
                }
            }
        }
        Ok(None)
    }
}

// The call limit is set to 100,000 and each nesting level uses ~150 pest rule
// calls (the grammar has many precedence levels), so the threshold where
// expressions start being rejected is at depth 667 (depth 666 succeeds, 667 fails).
// Depths below this should compute successfully; depths at or above should
// return a "Sorry" error from the parser call limit.
//
// IMPORTANT: If these thresholds change (e.g. because the call limit or
// grammar changes), update these constants to match.
const MAX_DEPTH_THAT_MUST_SUCCEED: usize = 666;
const MIN_DEPTH_THAT_MUST_ERROR: usize = 667;

// =============================================================================
// Stack Overflow Protection
// =============================================================================

/// Test that the server survives deeply-nested expressions at various depths,
/// and that the call limit correctly rejects deep nesting without crashing.
///
/// Sweeps through nesting depths and verifies:
/// 1. Shallow depths compute correctly (return a numeric result)
/// 2. Deep depths are rejected gracefully (return "Sorry" error)
/// 3. The server stays alive at every depth (no stack overflow crash)
#[tokio::test]
async fn stack_overflow_protection() {
    // Build a range of depths with increasing step sizes.
    // Dense near the boundary where the call limit kicks in, sparser for
    // extreme depths that should be well past the limit.
    let mut depths: Vec<usize> = Vec::new();

    // Coarse sweep up to the boundary
    depths.extend((10..=600).step_by(50));
    // Every depth around the threshold (666 succeeds, 667 fails)
    depths.extend(650..=700);
    // Past the limit
    depths.extend((800..=2000).step_by(200));
    // Extreme depths
    depths.extend([5000, 10000, 50000, 100000]);

    for depth in &depths {
        let expr = nested_expression(*depth);
        let result = try_calc(&expr).await;

        match result {
            Ok(maybe_result) => {
                // Verify correct behavior based on depth
                if *depth <= MAX_DEPTH_THAT_MUST_SUCCEED {
                    let result_str = maybe_result.unwrap_or_else(|| {
                        panic!("depth={} should compute successfully but got error response", depth)
                    });
                    // (1+(1+(1+...+1))) at depth N equals N+1
                    let expected = (depth + 1).to_string();
                    assert_eq!(
                        result_str, expected,
                        "depth={} should compute to {} but got {}",
                        depth, expected, result_str
                    );
                } else if *depth >= MIN_DEPTH_THAT_MUST_ERROR {
                    assert!(
                        maybe_result.is_none(),
                        "depth={} should be rejected by call limit but computed successfully",
                        depth
                    );
                }
                // Depths between the two thresholds are in the transition zone —
                // either result is acceptable.
            }
            Err(e) => {
                // Server didn't respond — likely crashed from stack overflow.
                // Try one more request to confirm the server is truly dead.
                let alive = client()
                    .get(url("/about.htm"))
                    .send()
                    .await;
                panic!(
                    "Server failed at nesting depth {}: {}. Server alive after: {}",
                    depth,
                    e,
                    alive.is_ok()
                );
            }
        }

        // Verify server is still alive after each depth
        let health = client().get(url("/about.htm")).send().await;
        assert!(
            health.is_ok() && health.unwrap().status().is_success(),
            "Server died after processing nesting depth {}",
            depth
        );
    }
}
