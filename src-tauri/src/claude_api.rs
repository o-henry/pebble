use std::time::{Duration, Instant};

use reqwest::{header::HeaderValue, redirect::Policy, StatusCode};
use serde_json::{json, Value};

use crate::claude_credentials::StoredSecret;

const ANTHROPIC_MESSAGES_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_MODELS_URL: &str = "https://api.anthropic.com/v1/models?limit=100";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const MAX_RESPONSE_BYTES: usize = 1_048_576;
const MAX_ANSWER_CHARS: usize = 4_000;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const STATUS_TIMEOUT: Duration = Duration::from_secs(20);
const MESSAGE_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeApiError {
    Authentication,
    InvalidResponse,
    ModelUnavailable,
    RateLimited,
    RequestFailed,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeApiAnswer {
    pub text: String,
    pub model: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeApiModel {
    pub id: String,
    pub label: String,
}

pub struct ClaudeApiRequest<'a> {
    pub model: &'a str,
    pub system: &'a str,
    pub prompt: String,
    pub image_data_urls: Vec<String>,
    pub max_tokens: u32,
}

pub async fn list_models(api_key: &StoredSecret) -> Result<Vec<ClaudeApiModel>, ClaudeApiError> {
    let client = client(STATUS_TIMEOUT)?;
    let mut response = authenticated(client.get(ANTHROPIC_MODELS_URL), api_key)?
        .send()
        .await
        .map_err(map_transport_error)?;
    require_success(response.status(), true)?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(map_transport_error)? {
        if chunk.len() > MAX_RESPONSE_BYTES.saturating_sub(bytes.len()) {
            return Err(ClaudeApiError::InvalidResponse);
        }
        bytes.extend_from_slice(&chunk);
    }
    parse_models(&bytes)
}

pub async fn create_message(
    api_key: &StoredSecret,
    request: ClaudeApiRequest<'_>,
) -> Result<ClaudeApiAnswer, ClaudeApiError> {
    let content = message_content(request.image_data_urls, request.prompt)?;
    if !valid_model_id(request.model) {
        return Err(ClaudeApiError::ModelUnavailable);
    }
    let body = json!({
        "model": request.model,
        "max_tokens": request.max_tokens,
        "system": request.system,
        "messages": [{ "role": "user", "content": content }],
        "output_config": { "effort": "medium" }
    });
    let client = client(MESSAGE_TIMEOUT)?;
    let started = Instant::now();
    let mut response = authenticated(client.post(ANTHROPIC_MESSAGES_URL), api_key)?
        .json(&body)
        .send()
        .await
        .map_err(map_transport_error)?;
    require_success(response.status(), false)?;
    if response
        .content_length()
        .is_some_and(|size| usize::try_from(size).map_or(true, |size| size > MAX_RESPONSE_BYTES))
    {
        return Err(ClaudeApiError::InvalidResponse);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(map_transport_error)? {
        if chunk.len() > MAX_RESPONSE_BYTES.saturating_sub(bytes.len()) {
            return Err(ClaudeApiError::InvalidResponse);
        }
        bytes.extend_from_slice(&chunk);
    }
    let (text, model) = parse_message(&bytes)?;
    Ok(ClaudeApiAnswer {
        text,
        model,
        duration_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
    })
}

fn parse_models(bytes: &[u8]) -> Result<Vec<ClaudeApiModel>, ClaudeApiError> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|_| ClaudeApiError::InvalidResponse)?;
    let data = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or(ClaudeApiError::InvalidResponse)?;
    let mut models = data
        .iter()
        .filter_map(|model| {
            let id = model.get("id")?.as_str()?;
            if !valid_model_id(id) || !(id.contains("sonnet") || id.contains("opus")) {
                return None;
            }
            let label = model
                .get("display_name")
                .and_then(Value::as_str)
                .filter(|label| !label.is_empty() && label.len() <= 100)
                .unwrap_or(id);
            Some(ClaudeApiModel {
                id: id.to_string(),
                label: label.to_ascii_uppercase(),
            })
        })
        .collect::<Vec<_>>();
    models.sort_by_key(|model| if model.id.contains("sonnet") { 0 } else { 1 });
    models.dedup_by(|left, right| left.id == right.id);
    if models.is_empty() {
        Err(ClaudeApiError::ModelUnavailable)
    } else {
        Ok(models)
    }
}

fn valid_model_id(model: &str) -> bool {
    model.starts_with("claude-")
        && model.len() <= 100
        && model
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn client(timeout: Duration) -> Result<reqwest::Client, ClaudeApiError> {
    reqwest::Client::builder()
        .https_only(true)
        .redirect(Policy::none())
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(timeout)
        .build()
        .map_err(|_| ClaudeApiError::RequestFailed)
}

fn authenticated(
    request: reqwest::RequestBuilder,
    api_key: &StoredSecret,
) -> Result<reqwest::RequestBuilder, ClaudeApiError> {
    let mut key =
        HeaderValue::from_bytes(api_key.as_bytes()).map_err(|_| ClaudeApiError::Authentication)?;
    key.set_sensitive(true);
    Ok(request
        .header("x-api-key", key)
        .header("anthropic-version", ANTHROPIC_VERSION))
}

fn message_content(
    image_data_urls: Vec<String>,
    prompt: String,
) -> Result<Vec<Value>, ClaudeApiError> {
    let mut content = Vec::with_capacity(image_data_urls.len() + 1);
    for data_url in image_data_urls {
        let image = data_url
            .strip_prefix("data:image/png;base64,")
            .ok_or(ClaudeApiError::InvalidResponse)?;
        content.push(json!({
            "type": "image",
            "source": { "type": "base64", "media_type": "image/png", "data": image }
        }));
    }
    content.push(json!({ "type": "text", "text": prompt }));
    Ok(content)
}

fn require_success(status: StatusCode, model_lookup: bool) -> Result<(), ClaudeApiError> {
    match status {
        status if status.is_success() => Ok(()),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(ClaudeApiError::Authentication),
        StatusCode::NOT_FOUND if model_lookup => Err(ClaudeApiError::ModelUnavailable),
        StatusCode::TOO_MANY_REQUESTS => Err(ClaudeApiError::RateLimited),
        _ => Err(ClaudeApiError::RequestFailed),
    }
}

fn map_transport_error(error: reqwest::Error) -> ClaudeApiError {
    if error.is_timeout() {
        ClaudeApiError::Timeout
    } else {
        ClaudeApiError::RequestFailed
    }
}

fn parse_message(bytes: &[u8]) -> Result<(String, String), ClaudeApiError> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|_| ClaudeApiError::InvalidResponse)?;
    if value.get("type").and_then(Value::as_str) != Some("message") {
        return Err(ClaudeApiError::InvalidResponse);
    }
    let model = value
        .get("model")
        .and_then(Value::as_str)
        .filter(|model| model.starts_with("claude-") && model.len() <= 100)
        .ok_or(ClaudeApiError::InvalidResponse)?
        .to_string();
    let mut answer = String::new();
    for block in value
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => append_text(
                &mut answer,
                block
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            )?,
            Some("thinking") | Some("redacted_thinking") => {}
            Some("tool_use") | Some("server_tool_use") => {
                return Err(ClaudeApiError::InvalidResponse);
            }
            _ => {}
        }
    }
    let answer = answer.trim().to_string();
    if answer.is_empty() {
        return Err(ClaudeApiError::InvalidResponse);
    }
    Ok((answer, model))
}

fn append_text(answer: &mut String, text: &str) -> Result<(), ClaudeApiError> {
    if !answer.is_empty() && !text.is_empty() {
        answer.push('\n');
    }
    answer.push_str(text);
    if answer.chars().count() > MAX_ANSWER_CHARS {
        return Err(ClaudeApiError::InvalidResponse);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use super::{
        message_content, parse_message, parse_models, require_success, valid_model_id,
        ClaudeApiError,
    };

    #[test]
    fn api_payload_keeps_images_memory_only_and_places_prompt_last() {
        let content = message_content(
            vec!["data:image/png;base64,AAAA".to_string()],
            "Describe the selected region.".to_string(),
        )
        .expect("content");

        assert_eq!(content.len(), 2);
        assert_eq!(content[0]["source"]["data"], "AAAA");
        assert_eq!(content[1]["text"], "Describe the selected region.");
    }

    #[test]
    fn maps_http_status_without_exposing_response_content() {
        assert_eq!(
            require_success(StatusCode::UNAUTHORIZED, false),
            Err(ClaudeApiError::Authentication)
        );
        assert_eq!(
            require_success(StatusCode::NOT_FOUND, true),
            Err(ClaudeApiError::ModelUnavailable)
        );
        assert_eq!(
            require_success(StatusCode::NOT_FOUND, false),
            Err(ClaudeApiError::RequestFailed)
        );
        assert_eq!(
            require_success(StatusCode::TOO_MANY_REQUESTS, false),
            Err(ClaudeApiError::RateLimited)
        );
    }

    #[test]
    fn parses_text_without_accepting_tool_use() {
        let response = br#"{
            "type":"message",
            "model":"claude-sonnet-5",
            "content":[{"type":"text","text":"Visible change."}]
        }"#;
        assert_eq!(
            parse_message(response).expect("response"),
            ("Visible change.".to_string(), "claude-sonnet-5".to_string())
        );

        let tool = br#"{
            "type":"message",
            "model":"claude-sonnet-5",
            "content":[{"type":"tool_use","name":"computer"}]
        }"#;
        assert_eq!(
            parse_message(tool).unwrap_err(),
            ClaudeApiError::InvalidResponse
        );
    }

    #[test]
    fn lists_only_bounded_sonnet_and_opus_models() {
        let response = br#"{
            "data":[
                {"id":"claude-opus-5","display_name":"Claude Opus 5"},
                {"id":"claude-haiku-5","display_name":"Claude Haiku 5"},
                {"id":"claude-sonnet-5","display_name":"Claude Sonnet 5"}
            ]
        }"#;
        let models = parse_models(response).expect("models");
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "claude-sonnet-5");
        assert_eq!(models[1].id, "claude-opus-5");
        assert!(valid_model_id("claude-sonnet-5"));
        assert!(!valid_model_id("claude-sonnet-5;rm"));
    }
}
