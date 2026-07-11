use std::collections::HashSet;

use quick_xml::{
    escape::resolve_predefined_entity,
    events::{BytesRef, Event},
    Reader,
};
use serde::Deserialize;
use url::Url;

use crate::{
    discovery::{DiscoveryCategory, DiscoveryItem},
    public_source::{PublicSourceError, PublicSourceErrorCode},
    public_source_fetch::{fetch_public_bytes, public_client},
};

const BBC_WORLD_URL: &str = "https://feeds.bbci.co.uk/news/world/rss.xml";
const HN_TOP_URL: &str = "https://hacker-news.firebaseio.com/v0/topstories.json";
const HN_ITEM_ROOT: &str = "https://hacker-news.firebaseio.com/v0/item";
const MAX_CATEGORY_ITEMS: usize = 5;
const MAX_HN_CANDIDATES: usize = 8;
const MAX_TITLE_CHARS: usize = 180;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiscoveryFetch {
    pub(crate) items: Vec<DiscoveryItem>,
    pub(crate) warnings: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
struct HackerNewsItem {
    id: u64,
    title: Option<String>,
    score: Option<u32>,
    descendants: Option<u32>,
    #[serde(rename = "type")]
    item_type: Option<String>,
    dead: Option<bool>,
    deleted: Option<bool>,
}

pub(crate) async fn fetch_discovery() -> Result<DiscoveryFetch, PublicSourceError> {
    let mut items = Vec::new();
    let mut warnings = Vec::new();

    match fetch_news().await {
        Ok(news) => items.extend(news),
        Err(_) => warnings.push("NEWS TEMPORARILY UNAVAILABLE"),
    }
    match fetch_community().await {
        Ok(community) => items.extend(community),
        Err(_) => warnings.push("COMMUNITY TEMPORARILY UNAVAILABLE"),
    }

    deduplicate(&mut items);
    if items.is_empty() {
        return Err(PublicSourceError {
            code: PublicSourceErrorCode::RequestFailed,
            message: "DISCOVERY SOURCES COULD NOT BE CHECKED.",
        });
    }
    Ok(DiscoveryFetch { items, warnings })
}

async fn fetch_news() -> Result<Vec<DiscoveryItem>, PublicSourceError> {
    let url = Url::parse(BBC_WORLD_URL).map_err(|_| PublicSourceError::invalid_url())?;
    let client = public_client(&url).await?;
    let body = fetch_public_bytes(
        &client,
        &url,
        "application/rss+xml, application/xml, text/xml",
    )
    .await?;
    let text = std::str::from_utf8(&body).map_err(|_| PublicSourceError::unsupported_response())?;
    let items = parse_rss_items(text);
    if items.is_empty() {
        Err(PublicSourceError::unsupported_response())
    } else {
        Ok(items)
    }
}

async fn fetch_community() -> Result<Vec<DiscoveryItem>, PublicSourceError> {
    let top_url = Url::parse(HN_TOP_URL).map_err(|_| PublicSourceError::invalid_url())?;
    let client = public_client(&top_url).await?;
    let body = fetch_public_bytes(&client, &top_url, "application/json").await?;
    let ids: Vec<u64> =
        serde_json::from_slice(&body).map_err(|_| PublicSourceError::unsupported_response())?;
    let mut items = Vec::new();

    for id in ids.into_iter().take(MAX_HN_CANDIDATES) {
        let item_url = Url::parse(&format!("{HN_ITEM_ROOT}/{id}.json"))
            .map_err(|_| PublicSourceError::invalid_url())?;
        let Ok(body) = fetch_public_bytes(&client, &item_url, "application/json").await else {
            continue;
        };
        let Ok(item) = serde_json::from_slice::<HackerNewsItem>(&body) else {
            continue;
        };
        if item.item_type.as_deref() != Some("story")
            || item.dead.unwrap_or(false)
            || item.deleted.unwrap_or(false)
        {
            continue;
        }
        let Some(title) = item.title.as_deref().and_then(sanitize_title) else {
            continue;
        };
        items.push(DiscoveryItem {
            id: format!("hn:{}", item.id),
            category: DiscoveryCategory::Community,
            title,
            source: "HACKER NEWS".to_string(),
            url: format!("https://news.ycombinator.com/item?id={}", item.id),
            score: item.score,
            comments: item.descendants,
        });
        if items.len() == MAX_CATEGORY_ITEMS {
            break;
        }
    }
    if items.is_empty() {
        Err(PublicSourceError::unsupported_response())
    } else {
        Ok(items)
    }
}

pub(crate) fn parse_rss_items(value: &str) -> Vec<DiscoveryItem> {
    let mut reader = Reader::from_str(value);
    reader.config_mut().trim_text(false);
    let mut in_item = false;
    let mut field: Option<&str> = None;
    let mut title = String::new();
    let mut link = String::new();
    let mut output = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(start)) => match start.local_name().as_ref() {
                b"item" => {
                    in_item = true;
                    title.clear();
                    link.clear();
                }
                b"title" if in_item => field = Some("title"),
                b"link" if in_item => field = Some("link"),
                _ => {}
            },
            Ok(Event::Text(text)) if in_item => {
                if let Ok(decoded) = text.decode() {
                    match field {
                        Some("title") => title.push_str(&decoded),
                        Some("link") => link.push_str(&decoded),
                        _ => {}
                    }
                }
            }
            Ok(Event::CData(text)) if in_item => {
                if let Ok(decoded) = text.decode() {
                    match field {
                        Some("title") => title.push_str(&decoded),
                        Some("link") => link.push_str(&decoded),
                        _ => {}
                    }
                }
            }
            Ok(Event::GeneralRef(reference)) if in_item => match field {
                Some("title") => append_reference(&mut title, &reference),
                Some("link") => append_reference(&mut link, &reference),
                _ => {}
            },
            Ok(Event::End(end)) => {
                if end.local_name().as_ref() == b"item" {
                    if let (Some(title), Some(url)) =
                        (sanitize_title(&title), sanitize_https_url(&link))
                    {
                        output.push(DiscoveryItem {
                            id: format!("news:{url}"),
                            category: DiscoveryCategory::News,
                            title,
                            source: "BBC WORLD".to_string(),
                            url,
                            score: None,
                            comments: None,
                        });
                    }
                    in_item = false;
                    if output.len() == MAX_CATEGORY_ITEMS {
                        break;
                    }
                }
                field = None;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    output
}

fn append_reference(output: &mut String, reference: &BytesRef<'_>) {
    if let Ok(Some(character)) = reference.resolve_char_ref() {
        output.push(character);
        return;
    }
    if let Ok(name) = reference.decode() {
        if let Some(value) = resolve_predefined_entity(&name) {
            output.push_str(value);
        }
    }
}

fn sanitize_title(value: &str) -> Option<String> {
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!value.is_empty() && !value.chars().any(char::is_control))
        .then(|| value.chars().take(MAX_TITLE_CHARS).collect())
}

fn sanitize_https_url(value: &str) -> Option<String> {
    let url = Url::parse(value.trim()).ok()?;
    (url.scheme() == "https"
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.port_or_known_default() == Some(443))
    .then(|| url.to_string())
}

fn deduplicate(items: &mut Vec<DiscoveryItem>) {
    let mut seen = HashSet::new();
    items.retain(|item| seen.insert(item.title.to_lowercase()));
}
