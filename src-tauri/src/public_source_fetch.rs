use std::{
    hash::{DefaultHasher, Hash, Hasher},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};

use quick_xml::{escape::unescape, events::Event, Reader};
use reqwest::{header, redirect, Client};
use tokio::net::lookup_host;
use url::Url;

use crate::public_source::PublicSourceError;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(12);
const MAX_RESPONSE_BYTES: usize = 512 * 1024;
const MAX_TITLE_CHARS: usize = 240;

#[derive(Debug, Clone)]
pub(crate) struct FetchedSource {
    pub(crate) fingerprint: u64,
    pub(crate) title: String,
}

pub(crate) async fn fetch_source(url: &Url) -> Result<FetchedSource, PublicSourceError> {
    let host = url.host_str().ok_or_else(PublicSourceError::invalid_url)?;
    let addresses = resolve_public_addresses(host).await?;
    let client = source_client(host, &addresses)?;
    let mut response = client
        .get(url.clone())
        .header(
            header::ACCEPT,
            "application/atom+xml, application/rss+xml, application/xml, text/xml, text/html, application/json",
        )
        .header(header::USER_AGENT, "Pebble/0.1 public-source-watch")
        .send()
        .await
        .map_err(|_| PublicSourceError::request_failed())?;
    if !response.status().is_success() {
        return Err(PublicSourceError::request_failed());
    }
    validate_content_type(response.headers())?;
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(PublicSourceError::response_too_large());
    }

    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| PublicSourceError::request_failed())?
    {
        if body.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
            return Err(PublicSourceError::response_too_large());
        }
        body.extend_from_slice(&chunk);
    }
    let text = std::str::from_utf8(&body).map_err(|_| PublicSourceError::unsupported_response())?;
    let parsed = parse_document(text);
    let title = parsed
        .as_ref()
        .map(|document| document.title.clone())
        .unwrap_or_else(|| "PUBLIC SOURCE CHANGED".to_string());
    let identity = parsed
        .map(|document| document.identity)
        .unwrap_or_else(|| text.to_string());
    let mut hasher = DefaultHasher::new();
    identity.hash(&mut hasher);
    Ok(FetchedSource {
        fingerprint: hasher.finish(),
        title,
    })
}

fn source_client(host: &str, addresses: &[SocketAddr]) -> Result<Client, PublicSourceError> {
    Client::builder()
        .redirect(redirect::Policy::none())
        .no_proxy()
        .resolve_to_addrs(host, addresses)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|_| PublicSourceError::request_failed())
}

async fn resolve_public_addresses(host: &str) -> Result<Vec<SocketAddr>, PublicSourceError> {
    let addresses = lookup_host((host, 443))
        .await
        .map_err(|_| PublicSourceError::request_failed())?
        .filter(|address| is_public_ip(address.ip()))
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        Err(PublicSourceError::private_destination())
    } else {
        Ok(addresses)
    }
}

pub(crate) fn validate_source_url(value: &str) -> Result<Url, PublicSourceError> {
    if value.len() > 2_048 {
        return Err(PublicSourceError::invalid_url());
    }
    let url = Url::parse(value).map_err(|_| PublicSourceError::invalid_url())?;
    let host = url.host_str().ok_or_else(PublicSourceError::invalid_url)?;
    let host_allowed = !host.eq_ignore_ascii_case("localhost")
        && !host.ends_with(".local")
        && !host.ends_with(".internal");
    let allowed = url.scheme() == "https"
        && host_allowed
        && url.username().is_empty()
        && url.password().is_none()
        && url.port_or_known_default() == Some(443)
        && url.fragment().is_none();
    allowed
        .then_some(url)
        .ok_or_else(PublicSourceError::invalid_url)
}

fn validate_content_type(headers: &header::HeaderMap) -> Result<(), PublicSourceError> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let allowed = [
        "text/",
        "application/xml",
        "application/rss+xml",
        "application/atom+xml",
        "application/json",
    ]
    .iter()
    .any(|allowed| content_type.starts_with(allowed));
    allowed
        .then_some(())
        .ok_or_else(PublicSourceError::unsupported_response)
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedDocument {
    pub(crate) title: String,
    pub(crate) identity: String,
}

pub(crate) fn parse_document(value: &str) -> Option<ParsedDocument> {
    let mut reader = Reader::from_str(value);
    reader.config_mut().trim_text(true);
    let mut in_entry = false;
    let mut current_field = None;
    let mut title = None;
    let mut identity = String::new();
    let mut page_title = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(start)) => {
                let name = start.local_name();
                match name.as_ref() {
                    b"item" | b"entry" => in_entry = true,
                    b"title" if in_entry => current_field = Some("title"),
                    b"guid" | b"id" | b"updated" | b"pubDate" if in_entry => {
                        current_field = Some("identity")
                    }
                    b"title" if page_title.is_none() => current_field = Some("page-title"),
                    _ => {}
                }
            }
            Ok(Event::Text(text)) => {
                let decoded = text.decode().ok()?;
                let decoded = unescape(&decoded).ok()?.trim().to_string();
                match current_field {
                    Some("title") if title.is_none() => {
                        identity.push_str(&decoded);
                        title = sanitize_title(&decoded);
                    }
                    Some("identity") => identity.push_str(&decoded),
                    Some("page-title") if page_title.is_none() => {
                        page_title = sanitize_title(&decoded)
                    }
                    _ => {}
                }
            }
            Ok(Event::End(end)) => {
                let name = end.local_name();
                if name.as_ref() == b"item" || name.as_ref() == b"entry" {
                    if let Some(title) = title {
                        return Some(ParsedDocument {
                            identity: if identity.is_empty() {
                                title.clone()
                            } else {
                                identity
                            },
                            title,
                        });
                    }
                    in_entry = false;
                }
                current_field = None;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    page_title.map(|title| ParsedDocument {
        identity: title.clone(),
        title,
    })
}

fn sanitize_title(value: &str) -> Option<String> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return None;
    }
    Some(value.chars().take(MAX_TITLE_CHARS).collect())
}

pub(crate) fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_ipv4(ip),
        IpAddr::V6(ip) => is_public_ipv6(ip),
    }
}

fn is_public_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(a == 0
        || a == 10
        || a == 127
        || a >= 224
        || (a == 100 && (64..=127).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..=31).contains(&b))
        || (a == 192 && ((b == 0 && (c == 0 || c == 2)) || b == 168))
        || (a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
        || (a == 203 && b == 0 && c == 113))
}

fn is_public_ipv6(ip: Ipv6Addr) -> bool {
    if let Some(ipv4) = ip.to_ipv4_mapped() {
        return is_public_ipv4(ipv4);
    }
    let segments = ip.segments();
    !(ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_multicast()
        || (segments[0] & 0xfe00) == 0xfc00
        || (segments[0] & 0xffc0) == 0xfe80
        || (segments[0] == 0x2001 && segments[1] == 0x0db8))
}
