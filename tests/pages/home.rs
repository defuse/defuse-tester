//! Tests for the home page (/)

use crate::common::{client, url};
use scraper::{Html, Selector};

/// Checks that the dynamically generated list of pages on the homepage is present and populated.
#[tokio::test]
async fn top_pages_list_is_present_and_populated() {
    let resp = client().get(url("/")).send().await.unwrap();
    assert!(resp.status().is_success(), "home page should return 200");

    let body = resp.text().await.unwrap();
    let document = Html::parse_document(&body);

    // The top pages are in a table with class "upvote_pagelist"
    let table_selector = Selector::parse("table.upvote_pagelist").unwrap();
    let table = document
        .select(&table_selector)
        .next()
        .expect("home page should have upvote_pagelist table");

    // Each page entry is a table row with a title link
    let title_selector = Selector::parse("a.upvote_list_title").unwrap();
    let titles: Vec<_> = table.select(&title_selector).collect();

    assert!(
        !titles.is_empty(),
        "top pages list should have at least one page entry"
    );

    // Verify each entry has non-empty title text and href
    for (i, title_link) in titles.iter().enumerate() {
        let title_text = title_link.text().collect::<String>();
        assert!(
            !title_text.trim().is_empty(),
            "page {} title should not be empty",
            i
        );
        assert!(
            title_link.value().attr("href").is_some(),
            "page {} should have href",
            i
        );
    }
}
