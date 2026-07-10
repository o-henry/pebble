use std::{
    collections::VecDeque,
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::Serialize;
use serde_json::{json, Value};
use tauri::{async_runtime::Receiver, AppHandle, Manager, WebviewWindow};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt,
};
use tokio::time::timeout;
use url::Url;

use crate::{
    capture_backend::{CaptureBackend, CroppedFramePayload},
    pebble_session::{AuthorizedAiCapture, PebbleSessionState},
    platform_capture::PlatformCaptureBackend,
};

const MAIN_WINDOW_LABEL: &str = "main";
const MAX_QUESTION_CHARS: usize = 1_000;
const MAX_ANSWER_CHARS: usize = 4_000;
const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);
const TURN_TIMEOUT: Duration = Duration::from_secs(120);
const COMPACT_MODEL_PREFERENCES: [&str; 1] = ["gpt-5.4-mini"];

const BASE_INSTRUCTIONS: &str = "You answer a user's question about one explicitly supplied cropped screen-region image. Use only visible evidence in that image. Do not use tools, files, shell, web search, plugins, skills, MCP, memory, or outside context. If the image does not contain enough evidence, say so plainly. Reply in the language of the user's question, directly and concisely, in at most five short sentences.";
const DEVELOPER_INSTRUCTIONS: &str = "ScreenPebble sends exactly one user-requested cropped image. Never invoke any tool or request more access.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConnectionStatus {
    pub connected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiAnswer {
    pub answer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRuntimeError {
    pub code: AiRuntimeErrorCode,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AiRuntimeErrorCode {
    AuthenticationFailed,
    Busy,
    CaptureUnavailable,
    InvalidQuestion,
    ModelUnavailable,
    NotConnected,
    ResponseFailed,
    SessionChanged,
    SidecarUnavailable,
    Timeout,
    UnauthorizedWindow,
}

#[derive(Debug, Clone, Default)]
pub struct AiRuntimeState {
    request_in_flight: Arc<AtomicBool>,
}

impl AiRuntimeState {
    fn begin_request(&self) -> Result<AiRequestGuard, AiRuntimeError> {
        self.request_in_flight
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| {
                runtime_error(
                    AiRuntimeErrorCode::Busy,
                    "Finish the current ChatGPT action before starting another one.",
                    true,
                )
            })?;

        Ok(AiRequestGuard {
            request_in_flight: Arc::clone(&self.request_in_flight),
        })
    }
}

struct AiRequestGuard {
    request_in_flight: Arc<AtomicBool>,
}

impl Drop for AiRequestGuard {
    fn drop(&mut self) {
        self.request_in_flight.store(false, Ordering::Release);
    }
}

pub async fn get_connection_status(
    app: &AppHandle,
    state: &AiRuntimeState,
) -> Result<AiConnectionStatus, AiRuntimeError> {
    let _guard = state.begin_request()?;
    let mut server = AppServerProcess::start(app).await?;
    read_chatgpt_account(&mut server).await
}

pub async fn connect_chatgpt(
    app: &AppHandle,
    state: &AiRuntimeState,
) -> Result<AiConnectionStatus, AiRuntimeError> {
    let _guard = state.begin_request()?;
    let mut server = AppServerProcess::start(app).await?;
    let status = read_chatgpt_account(&mut server).await?;
    if status.connected {
        return Ok(status);
    }

    let login = server
        .request(
            "account/login/start",
            json!({ "type": "chatgpt" }),
            REQUEST_TIMEOUT,
        )
        .await?;
    let login_id = required_string(&login, &["loginId"], "ChatGPT login did not start.")?;
    let auth_url = required_string(&login, &["authUrl"], "ChatGPT login URL is unavailable.")?;
    validate_auth_url(auth_url)?;

    app.opener().open_url(auth_url, None::<&str>).map_err(|_| {
        runtime_error(
            AiRuntimeErrorCode::AuthenticationFailed,
            "The ChatGPT sign-in page could not be opened.",
            true,
        )
    })?;

    let deadline = Instant::now() + LOGIN_TIMEOUT;
    loop {
        let message = server.next_message(deadline).await?;
        if message.get("method").and_then(Value::as_str) != Some("account/login/completed") {
            continue;
        }

        let params = message.get("params").unwrap_or(&Value::Null);
        if params.get("loginId").and_then(Value::as_str) != Some(login_id) {
            continue;
        }

        if params.get("success").and_then(Value::as_bool) != Some(true) {
            return Err(runtime_error(
                AiRuntimeErrorCode::AuthenticationFailed,
                "ChatGPT sign-in was not completed.",
                true,
            ));
        }

        return read_chatgpt_account(&mut server).await;
    }
}

pub async fn ask_selected_region(
    app: &AppHandle,
    window: &WebviewWindow,
    runtime: &AiRuntimeState,
    session: &PebbleSessionState,
    question: String,
) -> Result<AiAnswer, AiRuntimeError> {
    let _guard = runtime.begin_request()?;
    let question = normalize_question(&question)?;
    ensure_authorized_window(window)?;

    let monitors = crate::current_monitor_geometries(app).map_err(capture_runtime_error)?;
    let authorized = session
        .authorize_ai_capture(&monitors)
        .map_err(capture_runtime_error)?;
    let frame = PlatformCaptureBackend
        .capture_region_at_scale(authorized.region(), authorized.scale_factor())
        .map_err(capture_runtime_error)?;
    ensure_capture_is_current(app, window, session, &authorized)?;
    let image_data_url = encode_frame_data_url(&frame)?;

    let mut server = AppServerProcess::start(app).await?;
    let account = read_chatgpt_account(&mut server).await?;
    if !account.connected {
        return Err(runtime_error(
            AiRuntimeErrorCode::NotConnected,
            "Connect a ChatGPT account before asking about the selected region.",
            true,
        ));
    }

    let model = compact_image_model(&mut server).await?;
    let thread = server
        .request(
            "thread/start",
            json!({
                "model": model,
                "cwd": server.codex_home,
                "approvalPolicy": "never",
                "sandbox": "read-only",
                "baseInstructions": BASE_INSTRUCTIONS,
                "developerInstructions": DEVELOPER_INSTRUCTIONS,
                "ephemeral": true
            }),
            REQUEST_TIMEOUT,
        )
        .await?;
    let thread_id = required_string(
        &thread,
        &["thread", "id"],
        "The private ChatGPT session did not start.",
    )?
    .to_string();

    ensure_capture_is_current(app, window, session, &authorized)?;
    server
        .request(
            "turn/start",
            json!({
                "threadId": thread_id,
                "input": [
                    {
                        "type": "text",
                        "text": question_prompt(&question),
                        "text_elements": []
                    },
                    { "type": "image", "url": image_data_url }
                ],
                "approvalPolicy": "never",
                "model": model,
                "effort": "low",
                "summary": "none"
            }),
            REQUEST_TIMEOUT,
        )
        .await?;

    let answer = collect_answer(&mut server, &thread_id).await?;
    Ok(AiAnswer { answer })
}

fn ensure_capture_is_current(
    app: &AppHandle,
    window: &WebviewWindow,
    session: &PebbleSessionState,
    authorized: &AuthorizedAiCapture,
) -> Result<(), AiRuntimeError> {
    ensure_authorized_window(window)?;
    let monitors = crate::current_monitor_geometries(app).map_err(capture_runtime_error)?;
    let current = session
        .ai_capture_is_current(authorized.session_revision(), &monitors)
        .map_err(|_| {
            runtime_error(
                AiRuntimeErrorCode::SessionChanged,
                "The selected region changed before it could be sent.",
                true,
            )
        })?;

    if current {
        Ok(())
    } else {
        Err(runtime_error(
            AiRuntimeErrorCode::SessionChanged,
            "The selected region changed before it could be sent.",
            true,
        ))
    }
}

async fn read_chatgpt_account(
    server: &mut AppServerProcess,
) -> Result<AiConnectionStatus, AiRuntimeError> {
    let response = server
        .request(
            "account/read",
            json!({ "refreshToken": false }),
            REQUEST_TIMEOUT,
        )
        .await?;
    let Some(account) = response.get("account").filter(|account| !account.is_null()) else {
        return Ok(AiConnectionStatus { connected: false });
    };

    if account.get("type").and_then(Value::as_str) != Some("chatgpt") {
        return Err(runtime_error(
            AiRuntimeErrorCode::AuthenticationFailed,
            "ScreenPebble accepts ChatGPT account sign-in only; API keys are not used.",
            true,
        ));
    }

    Ok(AiConnectionStatus { connected: true })
}

async fn compact_image_model(server: &mut AppServerProcess) -> Result<String, AiRuntimeError> {
    let response = server
        .request(
            "model/list",
            json!({ "limit": 100, "includeHidden": false }),
            REQUEST_TIMEOUT,
        )
        .await?;
    select_compact_model(
        response
            .get("data")
            .and_then(Value::as_array)
            .map(Vec::as_slice),
    )
    .ok_or_else(|| {
        runtime_error(
            AiRuntimeErrorCode::ModelUnavailable,
            "This ChatGPT account does not currently offer a compact image model.",
            true,
        )
    })
}

fn select_compact_model(models: Option<&[Value]>) -> Option<String> {
    let candidates = models?.iter().filter(|model| {
        has_string(model, "inputModalities", "image")
            && model
                .get("supportedReasoningEfforts")
                .and_then(Value::as_array)
                .is_some_and(|efforts| {
                    efforts.iter().any(|effort| {
                        effort.get("reasoningEffort").and_then(Value::as_str) == Some("low")
                    })
                })
    });
    let candidates = candidates.collect::<Vec<_>>();

    for preferred in COMPACT_MODEL_PREFERENCES {
        if let Some(model) = candidates
            .iter()
            .find(|model| model.get("model").and_then(Value::as_str) == Some(preferred))
        {
            return model.get("model")?.as_str().map(str::to_string);
        }
    }

    candidates
        .iter()
        .find_map(|model| {
            model
                .get("model")
                .and_then(Value::as_str)
                .filter(|id| id.contains("mini"))
        })
        .map(str::to_string)
}

async fn collect_answer(
    server: &mut AppServerProcess,
    thread_id: &str,
) -> Result<String, AiRuntimeError> {
    let deadline = Instant::now() + TURN_TIMEOUT;
    let mut streamed_answer = String::new();
    let mut completed_answer = None;

    loop {
        let message = server.next_message(deadline).await?;
        if message.get("id").is_some() && message.get("method").is_some() {
            return Err(runtime_error(
                AiRuntimeErrorCode::ResponseFailed,
                "The AI response requested an action outside the selected image, so ScreenPebble stopped it.",
                true,
            ));
        }
        let method = message.get("method").and_then(Value::as_str);
        let params = message.get("params").unwrap_or(&Value::Null);

        if params
            .get("threadId")
            .and_then(Value::as_str)
            .is_some_and(|id| id != thread_id)
        {
            continue;
        }

        match method {
            Some("item/agentMessage/delta") => {
                if let Some(delta) = params.get("delta").and_then(Value::as_str) {
                    append_answer(&mut streamed_answer, delta)?;
                }
            }
            Some("item/started") | Some("item/completed") => {
                let item = params.get("item").unwrap_or(&Value::Null);
                let item_type = item.get("type").and_then(Value::as_str).unwrap_or_default();
                if !matches!(item_type, "userMessage" | "agentMessage" | "reasoning") {
                    return Err(runtime_error(
                        AiRuntimeErrorCode::ResponseFailed,
                        "The AI response attempted an action outside the selected image, so ScreenPebble stopped it.",
                        true,
                    ));
                }
                if method == Some("item/completed") && item_type == "agentMessage" {
                    completed_answer = item.get("text").and_then(Value::as_str).map(str::to_string);
                }
            }
            Some("turn/completed") => {
                if params
                    .get("turn")
                    .and_then(|turn| turn.get("status"))
                    .and_then(Value::as_str)
                    != Some("completed")
                {
                    return Err(runtime_error(
                        AiRuntimeErrorCode::ResponseFailed,
                        "ChatGPT could not complete this image question.",
                        true,
                    ));
                }

                let answer = if streamed_answer.trim().is_empty() {
                    completed_answer.unwrap_or_default()
                } else {
                    streamed_answer
                };
                let answer = answer.trim().to_string();
                if answer.is_empty() {
                    return Err(runtime_error(
                        AiRuntimeErrorCode::ResponseFailed,
                        "ChatGPT returned an empty answer.",
                        true,
                    ));
                }
                return Ok(answer);
            }
            _ => {}
        }
    }
}

fn append_answer(answer: &mut String, delta: &str) -> Result<(), AiRuntimeError> {
    if answer.chars().count().saturating_add(delta.chars().count()) > MAX_ANSWER_CHARS {
        return Err(runtime_error(
            AiRuntimeErrorCode::ResponseFailed,
            "The ChatGPT answer exceeded ScreenPebble's compact response limit.",
            true,
        ));
    }
    answer.push_str(delta);
    Ok(())
}

fn normalize_question(question: &str) -> Result<String, AiRuntimeError> {
    let question = question.trim();
    let valid_controls =
        |character: char| !character.is_control() || matches!(character, '\n' | '\r' | '\t');
    if question.is_empty()
        || question.chars().count() > MAX_QUESTION_CHARS
        || !question.chars().all(valid_controls)
    {
        return Err(runtime_error(
            AiRuntimeErrorCode::InvalidQuestion,
            "Enter a question between 1 and 1,000 characters.",
            true,
        ));
    }
    Ok(question.to_string())
}

fn question_prompt(question: &str) -> String {
    format!("Question about this selected screen region:\n{question}")
}

fn encode_frame_data_url(frame: &CroppedFramePayload) -> Result<String, AiRuntimeError> {
    let width = u32::try_from(frame.width).map_err(|_| invalid_frame_error())?;
    let height = u32::try_from(frame.height).map_err(|_| invalid_frame_error())?;
    let expected_len = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(invalid_frame_error)?;
    if frame.bytes.len() != expected_len {
        return Err(invalid_frame_error());
    }

    let mut png_bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|_| invalid_frame_error())?;
        writer
            .write_image_data(&frame.bytes)
            .map_err(|_| invalid_frame_error())?;
    }

    Ok(format!(
        "data:image/png;base64,{}",
        BASE64.encode(png_bytes)
    ))
}

fn invalid_frame_error() -> AiRuntimeError {
    runtime_error(
        AiRuntimeErrorCode::CaptureUnavailable,
        "The selected region could not be encoded safely.",
        true,
    )
}

fn ensure_authorized_window(window: &WebviewWindow) -> Result<(), AiRuntimeError> {
    let authorized = window.label() == MAIN_WINDOW_LABEL
        && window.is_visible().unwrap_or(false)
        && !window.is_minimized().unwrap_or(true);
    if authorized {
        Ok(())
    } else {
        Err(runtime_error(
            AiRuntimeErrorCode::UnauthorizedWindow,
            "Image questions are available only from the visible ScreenPebble window.",
            true,
        ))
    }
}

fn validate_auth_url(value: &str) -> Result<(), AiRuntimeError> {
    let url = Url::parse(value).map_err(|_| invalid_auth_url_error())?;
    if url.scheme() == "https" && url.host_str() == Some("auth.openai.com") {
        Ok(())
    } else {
        Err(invalid_auth_url_error())
    }
}

fn invalid_auth_url_error() -> AiRuntimeError {
    runtime_error(
        AiRuntimeErrorCode::AuthenticationFailed,
        "ScreenPebble rejected an unexpected sign-in URL.",
        true,
    )
}

fn has_string(value: &Value, field: &str, expected: &str) -> bool {
    value
        .get(field)
        .and_then(Value::as_array)
        .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(expected)))
}

fn required_string<'a>(
    value: &'a Value,
    path: &[&str],
    message: &str,
) -> Result<&'a str, AiRuntimeError> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment).unwrap_or(&Value::Null);
    }
    current
        .as_str()
        .ok_or_else(|| runtime_error(AiRuntimeErrorCode::ResponseFailed, message, true))
}

fn capture_runtime_error(_error: impl std::fmt::Debug) -> AiRuntimeError {
    runtime_error(
        AiRuntimeErrorCode::CaptureUnavailable,
        "The selected region is not available for an image question.",
        true,
    )
}

fn runtime_error(
    code: AiRuntimeErrorCode,
    message: impl Into<String>,
    recoverable: bool,
) -> AiRuntimeError {
    AiRuntimeError {
        code,
        message: message.into(),
        recoverable,
    }
}

struct AppServerProcess {
    receiver: Receiver<CommandEvent>,
    child: Option<CommandChild>,
    pending_messages: VecDeque<Value>,
    next_request_id: u64,
    codex_home: PathBuf,
}

impl AppServerProcess {
    async fn start(app: &AppHandle) -> Result<Self, AiRuntimeError> {
        let app_data_dir = app.path().app_data_dir().map_err(|_| sidecar_error())?;
        let codex_home = app_data_dir.join("codex-home");
        secure_directory(&app_data_dir)?;
        secure_directory(&codex_home)?;

        let environment_home = app_data_dir.as_os_str();
        let command = app
            .shell()
            .sidecar("codex")
            .map_err(|_| sidecar_error())?
            .args([
                "app-server",
                "--strict-config",
                "-c",
                "web_search=\"disabled\"",
                "-c",
                "cli_auth_credentials_store=\"keyring\"",
                "-c",
                "analytics.enabled=false",
                "-c",
                "mcp_servers={}",
                "--listen",
                "stdio://",
            ])
            .env_clear()
            .env("HOME", environment_home)
            .env("CODEX_HOME", codex_home.as_os_str())
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("LANG", "en_US.UTF-8")
            .current_dir(&codex_home);
        let (receiver, child) = command.spawn().map_err(|_| sidecar_error())?;
        let mut process = Self {
            receiver,
            child: Some(child),
            pending_messages: VecDeque::new(),
            next_request_id: 1,
            codex_home,
        };

        process
            .request(
                "initialize",
                json!({
                    "clientInfo": {
                        "name": "screenpebble",
                        "title": "ScreenPebble",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "capabilities": {
                        "experimentalApi": false,
                        "requestAttestation": false
                    }
                }),
                STARTUP_TIMEOUT,
            )
            .await?;
        process.notify("initialized")?;
        Ok(process)
    }

    async fn request(
        &mut self,
        method: &str,
        params: Value,
        request_timeout: Duration,
    ) -> Result<Value, AiRuntimeError> {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.write_json(&json!({
            "method": method,
            "id": request_id,
            "params": params
        }))?;

        let deadline = Instant::now() + request_timeout;
        loop {
            let message = self.receive_message(deadline).await?;
            if message.get("id").and_then(Value::as_u64) == Some(request_id) {
                if message.get("error").is_some_and(|error| !error.is_null()) {
                    return Err(runtime_error(
                        AiRuntimeErrorCode::ResponseFailed,
                        "The local ChatGPT service rejected this request.",
                        true,
                    ));
                }
                return Ok(message.get("result").cloned().unwrap_or(Value::Null));
            }
            self.pending_messages.push_back(message);
        }
    }

    fn notify(&mut self, method: &str) -> Result<(), AiRuntimeError> {
        self.write_json(&json!({ "method": method }))
    }

    async fn next_message(&mut self, deadline: Instant) -> Result<Value, AiRuntimeError> {
        if let Some(message) = self.pending_messages.pop_front() {
            return Ok(message);
        }
        self.receive_message(deadline).await
    }

    async fn receive_message(&mut self, deadline: Instant) -> Result<Value, AiRuntimeError> {
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or_else(|| {
                    runtime_error(
                        AiRuntimeErrorCode::Timeout,
                        "The local ChatGPT service took too long to respond.",
                        true,
                    )
                })?;
            let event = timeout(remaining, self.receiver.recv())
                .await
                .map_err(|_| {
                    runtime_error(
                        AiRuntimeErrorCode::Timeout,
                        "The local ChatGPT service took too long to respond.",
                        true,
                    )
                })?
                .ok_or_else(sidecar_error)?;

            match event {
                CommandEvent::Stdout(line) => {
                    return serde_json::from_slice(&line).map_err(|_| {
                        runtime_error(
                            AiRuntimeErrorCode::ResponseFailed,
                            "The local ChatGPT service returned an invalid response.",
                            true,
                        )
                    });
                }
                CommandEvent::Stderr(_) => {}
                CommandEvent::Error(_) | CommandEvent::Terminated(_) => {
                    return Err(sidecar_error());
                }
                _ => {}
            }
        }
    }

    fn write_json(&mut self, value: &Value) -> Result<(), AiRuntimeError> {
        let mut message = serde_json::to_vec(value).map_err(|_| sidecar_error())?;
        message.push(b'\n');
        self.child
            .as_mut()
            .ok_or_else(sidecar_error)?
            .write(&message)
            .map_err(|_| sidecar_error())
    }
}

impl Drop for AppServerProcess {
    fn drop(&mut self) {
        if let Some(child) = self.child.take() {
            let _ = child.kill();
        }
    }
}

fn secure_directory(path: &PathBuf) -> Result<(), AiRuntimeError> {
    fs::create_dir_all(path).map_err(|_| sidecar_error())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|_| sidecar_error())?;
    }
    Ok(())
}

fn sidecar_error() -> AiRuntimeError {
    runtime_error(
        AiRuntimeErrorCode::SidecarUnavailable,
        "The bundled local ChatGPT service is unavailable.",
        true,
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        encode_frame_data_url, normalize_question, select_compact_model, validate_auth_url,
        AiRuntimeErrorCode,
    };
    use crate::{
        capture_backend::{cropped_frame, FrameStoragePolicy},
        region_selection_types::PhysicalRegion,
    };

    #[test]
    fn accepts_only_non_empty_bounded_questions() {
        assert_eq!(
            normalize_question("  What changed?  ").unwrap(),
            "What changed?"
        );
        assert_eq!(
            normalize_question("").unwrap_err().code,
            AiRuntimeErrorCode::InvalidQuestion
        );
        assert_eq!(
            normalize_question(&"x".repeat(1_001)).unwrap_err().code,
            AiRuntimeErrorCode::InvalidQuestion
        );
        assert_eq!(
            normalize_question("unsafe\0question").unwrap_err().code,
            AiRuntimeErrorCode::InvalidQuestion
        );
    }

    #[test]
    fn accepts_only_the_official_https_login_host() {
        assert!(validate_auth_url("https://auth.openai.com/oauth/authorize?x=1").is_ok());
        assert!(validate_auth_url("http://auth.openai.com/oauth/authorize").is_err());
        assert!(validate_auth_url("https://auth.openai.com.evil.test/oauth").is_err());
    }

    #[test]
    fn selects_only_a_compact_low_effort_image_model() {
        let models = vec![
            json!({
                "model": "gpt-5.5",
                "inputModalities": ["text", "image"],
                "supportedReasoningEfforts": [{ "reasoningEffort": "low" }]
            }),
            json!({
                "model": "gpt-5.4-mini",
                "inputModalities": ["text", "image"],
                "supportedReasoningEfforts": [{ "reasoningEffort": "low" }]
            }),
        ];
        assert_eq!(
            select_compact_model(Some(&models)).as_deref(),
            Some("gpt-5.4-mini")
        );

        let expensive_only = vec![models[0].clone()];
        assert_eq!(select_compact_model(Some(&expensive_only)), None);
    }

    #[test]
    fn encodes_a_memory_only_rgba_crop_as_a_png_data_url() {
        let frame = cropped_frame(
            &PhysicalRegion {
                monitor_id: "main".to_string(),
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            vec![12, 34, 56, 255],
        );
        assert_eq!(frame.storage_policy, FrameStoragePolicy::MemoryOnly);
        assert!(encode_frame_data_url(&frame)
            .unwrap()
            .starts_with("data:image/png;base64,iVBOR"));
    }

    #[test]
    fn webviews_have_no_shell_opener_network_or_ai_plugin_permission() {
        let capability: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/default.json"))
                .expect("valid capability JSON");
        let permissions = capability["permissions"]
            .as_array()
            .expect("permissions array")
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();

        assert_eq!(
            permissions,
            vec!["core:event:allow-listen", "core:event:allow-unlisten"]
        );
        assert!(!permissions.iter().any(|permission| {
            permission.starts_with("shell:") || permission.starts_with("opener:")
        }));
    }
}
