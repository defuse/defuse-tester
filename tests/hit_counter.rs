//! Hit counter tests
//!
//! Tests for page hit counting functionality including bot detection.

mod common;

use common::url;
use reqwest::Client;
use reqwest::header::USER_AGENT;
use scraper::{Html, Selector};

/// Extract hit counts from page HTML
/// Returns (page_hits, unique_hits)
fn extract_hit_counts(html: &str) -> (i64, i64) {
    let document = Html::parse_document(html);

    // Find the table cells after "Page Hits:" and "Unique Hits:" headers
    let th_selector = Selector::parse("th").unwrap();
    let td_selector = Selector::parse("td").unwrap();

    let mut page_hits: Option<i64> = None;
    let mut unique_hits: Option<i64> = None;

    // Find all table rows and look for our headers
    let tr_selector = Selector::parse("tr").unwrap();
    for tr in document.select(&tr_selector) {
        let ths: Vec<_> = tr.select(&th_selector).collect();
        let tds: Vec<_> = tr.select(&td_selector).collect();

        if let (Some(th), Some(td)) = (ths.first(), tds.first()) {
            let header_text = th.text().collect::<String>();
            let value_text = td.text().collect::<String>();

            if header_text.contains("Page Hits") {
                page_hits = Some(value_text.trim().parse()
                    .unwrap_or_else(|e| panic!("Page Hits value '{}' is not a number: {}", value_text.trim(), e)));
            } else if header_text.contains("Unique Hits") {
                unique_hits = Some(value_text.trim().parse()
                    .unwrap_or_else(|e| panic!("Unique Hits value '{}' is not a number: {}", value_text.trim(), e)));
            }
        }
    }

    (
        page_hits.expect("Failed to find Page Hits in footer"),
        unique_hits.expect("Failed to find Unique Hits in footer"),
    )
}

#[tokio::test]
async fn hit_increments_non_unique_only() {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    // Use a specific page to avoid race conditions with other hit counter tests
    let page_url = url("/bochs-images.htm");

    // First hit
    let resp1 = client.get(&page_url).send().await.expect("First request failed");
    assert!(resp1.status().is_success(), "First request returned {}", resp1.status());
    let body1 = resp1.text().await.expect("Failed to read first response");
    let (page_hits_1, unique_hits_1) = extract_hit_counts(&body1);

    // Second hit (same IP, should only increment non-unique)
    let resp2 = client.get(&page_url).send().await.expect("Second request failed");
    assert!(resp2.status().is_success(), "Second request returned {}", resp2.status());
    let body2 = resp2.text().await.expect("Failed to read second response");
    let (page_hits_2, unique_hits_2) = extract_hit_counts(&body2);

    // Non-unique (Page Hits) should have increased by 1
    assert_eq!(
        page_hits_2,
        page_hits_1 + 1,
        "Page Hits should increase by 1 on second hit (was {}, now {})",
        page_hits_1,
        page_hits_2
    );

    // Unique hits should stay the same (same IP within 30-day window)
    assert_eq!(
        unique_hits_2,
        unique_hits_1,
        "Unique Hits should not change for same IP (was {}, now {})",
        unique_hits_1,
        unique_hits_2
    );
}

// =============================================================================
// Bot Detection (IGNORE_SEARCH_BOTS)
// =============================================================================

/// Bot requests should NOT increment the hit counter
///
/// phpcount.php has IGNORE_SEARCH_BOTS = true and IsSearchBot() checks for
/// keywords like "bot" (case-insensitive). This test verifies that requests
/// with bot user agents do not increment the page hit counter.
#[tokio::test]
async fn hit_counter_bot_not_counted() {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    // Use a specific page to avoid race conditions with other hit counter tests
    let page_url = url("/gnutls-psk-client-server-example.htm");

    // Get current count with normal UA
    let resp1 = client
        .get(&page_url)
        .header(USER_AGENT, "Mozilla/5.0 TestBrowser/1.0")
        .send()
        .await
        .expect("First request failed");
    assert!(resp1.status().is_success());
    let body1 = resp1.text().await.expect("Failed to read response");
    let (page_hits_before, _) = extract_hit_counts(&body1);

    // Request with "bot" in UA - should NOT be counted per IGNORE_SEARCH_BOTS
    let resp_bot = client
        .get(&page_url)
        .header(USER_AGENT, "TestBot/1.0")
        .send()
        .await
        .expect("Bot request failed");
    assert!(resp_bot.status().is_success());

    // Check count again with normal UA
    let resp2 = client
        .get(&page_url)
        .header(USER_AGENT, "Mozilla/5.0 TestBrowser/1.0")
        .send()
        .await
        .expect("Second request failed");
    assert!(resp2.status().is_success());
    let body2 = resp2.text().await.expect("Failed to read response");
    let (page_hits_after, _) = extract_hit_counts(&body2);

    // Should increment by exactly 1 (only the second normal request counted, bot was ignored)
    // If this fails with +2, bot detection is not working
    assert_eq!(
        page_hits_after,
        page_hits_before + 1,
        "Bot UA should be ignored (IGNORE_SEARCH_BOTS=true). Before: {}, After: {} (expected +1, got +{})",
        page_hits_before,
        page_hits_after,
        page_hits_after - page_hits_before
    );
}
