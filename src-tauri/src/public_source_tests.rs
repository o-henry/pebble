use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::{
    public_source::PublicSourceErrorCode,
    public_source_fetch::{
        client_url_is_allowed, is_public_ip, parse_document, validate_source_url,
    },
};
use url::Url;

#[test]
fn source_url_requires_public_https_without_credentials_or_fragments() {
    assert!(validate_source_url("https://example.com/feed.xml").is_ok());
    for value in [
        "http://example.com/feed",
        "https://user:pass@example.com/feed",
        "https://localhost/feed",
        "https://service.local/feed",
        "https://example.com/feed#private",
    ] {
        assert_eq!(
            validate_source_url(value).unwrap_err().code,
            PublicSourceErrorCode::InvalidUrl
        );
    }
}

#[test]
fn network_guard_blocks_private_reserved_and_documentation_addresses() {
    for ip in [
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)),
        IpAddr::V6(Ipv6Addr::LOCALHOST),
        "fc00::1".parse().unwrap(),
        "fe80::1".parse().unwrap(),
        "2001:db8::1".parse().unwrap(),
    ] {
        assert!(!is_public_ip(ip), "{ip} should be blocked");
    }
    assert!(is_public_ip(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))));
    assert!(is_public_ip("2606:4700:4700::1111".parse().unwrap()));
}

#[test]
fn pinned_client_rejects_cross_host_and_unsafe_urls() {
    assert!(client_url_is_allowed(
        "example.com",
        &Url::parse("https://example.com/item.json").unwrap()
    ));
    for value in [
        "https://other.example/item.json",
        "http://example.com/item.json",
        "https://user@example.com/item.json",
        "https://example.com:444/item.json",
        "https://example.com/item.json#fragment",
    ] {
        assert!(!client_url_is_allowed(
            "example.com",
            &Url::parse(value).unwrap()
        ));
    }
}

#[test]
fn parser_extracts_the_latest_rss_or_atom_entry_without_article_body() {
    let rss = r#"<rss><channel><title>Channel</title><item><title>Release 2</title><guid>two</guid><description>private body</description></item></channel></rss>"#;
    let atom = r#"<feed><title>Channel</title><entry><title>News update</title><id>three</id><content>private body</content></entry></feed>"#;

    let rss = parse_document(rss).expect("rss");
    assert_eq!(rss.title, "Release 2");
    assert!(!rss.identity.contains("private body"));
    let atom = parse_document(atom).expect("atom");
    assert_eq!(atom.title, "News update");
    assert!(!atom.identity.contains("private body"));
}

#[test]
fn parser_falls_back_to_a_web_page_title() {
    let page =
        parse_document("<html><head><title>Status page</title></head></html>").expect("page title");
    assert_eq!(page.title, "Status page");
}
