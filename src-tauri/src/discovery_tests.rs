use crate::{
    discovery::{DiscoveryCategory, DiscoveryItem, DiscoveryState},
    discovery_fetch::parse_rss_items,
};

#[test]
fn rss_parser_keeps_safe_https_news_items() {
    let feed = r#"
        <rss><channel>
          <item><title>First &amp; safe</title><link>https://example.com/first</link></item>
          <item><title>Blocked</title><link>http://example.com/blocked</link></item>
        </channel></rss>
    "#;
    let items = parse_rss_items(feed);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "First & safe");
    assert_eq!(items[0].category, DiscoveryCategory::News);
}

#[test]
fn disabled_discovery_starts_empty_and_session_scoped() {
    let state = DiscoveryState::default();
    let status = state.status();
    assert!(!status.enabled);
    assert_eq!(status.interval_minutes, 30);
    assert!(status.items.is_empty());
}

#[test]
fn discovery_items_expose_only_summary_metadata() {
    let item = DiscoveryItem {
        id: "hn:1".to_string(),
        category: DiscoveryCategory::Community,
        title: "A TITLE".to_string(),
        source: "HACKER NEWS".to_string(),
        url: "https://news.ycombinator.com/item?id=1".to_string(),
        score: Some(10),
        comments: Some(2),
    };
    let value = serde_json::to_value(item).expect("serialize discovery item");
    assert!(value.get("title").is_some());
    assert!(value.get("body").is_none());
    assert!(value.get("content").is_none());
}
