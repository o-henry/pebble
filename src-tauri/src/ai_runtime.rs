use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{async_runtime::Receiver, AppHandle, Manager, WebviewWindow};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt,
};
use tokio::{io::AsyncWriteExt, process::Command, time::timeout};
use url::Url;

use crate::{
    capture_backend::{capture_error, CaptureBackend, CaptureErrorCode, CroppedFramePayload},
    pebble_session::{AuthorizedAiCapture, PebbleSessionState, PEBBLE_TILE_LABEL},
    platform_capture::PlatformCaptureBackend,
};

const MAX_QUESTION_CHARS: usize = 1_000;
const MAX_ANSWER_CHARS: usize = 4_000;
const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);
const TURN_TIMEOUT: Duration = Duration::from_secs(120);
const OPENAI_MODEL_PREFERENCES: [&str; 2] = ["gpt-5.6-terra", "gpt-5.6-luna"];
const OPENAI_WATCH_MODEL_PREFERENCES: [&str; 1] = ["gpt-5.6-terra"];
const OPENAI_MODEL_LABEL: &str = "GPT-5.6-TERRA";
const OPENAI_REASONING_EFFORT: &str = "medium";
const CLAUDE_MODEL_ID: &str = "claude-sonnet-5";
const CLAUDE_MODEL_LABEL: &str = "CLAUDE SONNET 5";
const CLAUDE_REASONING_EFFORT: &str = "medium";
const CLAUDE_INSTALL_URL: &str = "https://code.claude.com/docs/en/quickstart";

const BASE_INSTRUCTIONS: &str = "You answer a user's question about one explicitly supplied cropped screen-region image. Use only visible evidence in that image. Do not use tools, files, shell, web search, plugins, skills, MCP, memory, or outside context. If the image does not contain enough evidence, say so plainly. Reply in the language of the user's question, directly and concisely, in at most five short sentences.";
const DEVELOPER_INSTRUCTIONS: &str = "Pebble sends exactly one user-requested cropped image. Never invoke any tool or request more access.";
const WATCH_INSTRUCTIONS: &str = "Compare exactly two cropped images from one user-selected screen region. The first image is BEFORE and the second is AFTER. Identify concrete visible content and explain what changed, why it may matter, and uncertainty. Do not use tools, files, shell, web search, plugins, skills, MCP, memory, or outside context. Never invent names, numbers, causes, or current events that are not visible. Reply in the requested locale in at most three compact sentences.";
const WATCH_DEVELOPER_INSTRUCTIONS: &str = "This is an automatic, user-enabled Watch analysis. Use only the two supplied images and the local visual signal. Never invoke a tool or request more access.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AiProvider {
    OpenAi,
    Claude,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConnectionStatus {
    pub provider: AiProvider,
    pub available: bool,
    pub connected: bool,
    pub model: String,
    pub install_url: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiAnswer {
    pub answer: String,
    pub provider: AiProvider,
    pub model: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchAnalysis {
    pub summary: String,
    pub model: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct WatchAnalysisRequest {
    pub revision: u64,
    pub provider: AiProvider,
    pub locale: String,
    pub local_signal: &'static str,
    pub previous_frame: CroppedFramePayload,
    pub current_frame: CroppedFramePayload,
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
                    "Finish the current AI action before starting another one.",
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
    provider: AiProvider,
) -> Result<AiConnectionStatus, AiRuntimeError> {
    let _guard = state.begin_request()?;
    match provider {
        AiProvider::OpenAi => {
            let mut server = AppServerProcess::start(app).await?;
            read_openai_connection_status(&mut server).await
        }
        AiProvider::Claude => claude_connection_status(app).await,
    }
}

pub async fn connect_provider(
    app: &AppHandle,
    state: &AiRuntimeState,
    provider: AiProvider,
) -> Result<AiConnectionStatus, AiRuntimeError> {
    let _guard = state.begin_request()?;
    match provider {
        AiProvider::OpenAi => connect_openai(app).await,
        AiProvider::Claude => connect_claude(app).await,
    }
}

async fn connect_openai(app: &AppHandle) -> Result<AiConnectionStatus, AiRuntimeError> {
    let mut server = AppServerProcess::start(app).await?;
    let status = read_openai_connection_status(&mut server).await?;
    if status.connected {
        return Ok(status);
    }

    let login = server
        .request(
            "account/login/start",
            chatgpt_login_params(),
            REQUEST_TIMEOUT,
        )
        .await?;
    let login_id = required_string(&login, &["loginId"], "OpenAI login did not start.")?;
    let auth_url = required_string(&login, &["authUrl"], "OpenAI login URL is unavailable.")?;
    validate_auth_url(auth_url)?;

    app.opener().open_url(auth_url, None::<&str>).map_err(|_| {
        runtime_error(
            AiRuntimeErrorCode::AuthenticationFailed,
            "The OpenAI sign-in page could not be opened.",
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
            let message = login_failure_message(params.get("error").and_then(Value::as_str));
            return Err(runtime_error(
                AiRuntimeErrorCode::AuthenticationFailed,
                message,
                true,
            ));
        }

        return read_openai_connection_status(&mut server).await;
    }
}

pub async fn ask_selected_region(
    app: &AppHandle,
    window: &WebviewWindow,
    runtime: &AiRuntimeState,
    session: &PebbleSessionState,
    provider: AiProvider,
    question: String,
    locale: String,
) -> Result<AiAnswer, AiRuntimeError> {
    let _guard = runtime.begin_request()?;
    let question = normalize_question(&question)?;
    ensure_authorized_window(window)?;

    let monitors = crate::current_monitor_geometries(app).map_err(capture_runtime_error)?;
    let authorized = session
        .authorize_ai_capture(&monitors)
        .map_err(capture_runtime_error)?;
    let capture_region = authorized.region().clone();
    let capture_scale = authorized.scale_factor();
    let frame = tauri::async_runtime::spawn_blocking(move || {
        PlatformCaptureBackend.capture_region_at_scale(&capture_region, capture_scale)
    })
    .await
    .map_err(|_| {
        capture_runtime_error(capture_error(
            CaptureErrorCode::CaptureUnavailable,
            "ai-capture",
            "The native capture worker stopped unexpectedly.",
        ))
    })?
    .map_err(capture_runtime_error)?;
    ensure_capture_is_current(app, window, session, &authorized)?;
    let image_data_url = encode_frame_data_url(&frame)?;

    match provider {
        AiProvider::OpenAi => {
            ask_openai(
                app,
                window,
                session,
                &authorized,
                image_data_url,
                question,
                locale,
            )
            .await
        }
        AiProvider::Claude => {
            ask_claude(
                app,
                window,
                session,
                &authorized,
                image_data_url,
                question,
                locale,
            )
            .await
        }
    }
}

pub async fn analyze_watch_change(
    app: &AppHandle,
    runtime: &AiRuntimeState,
    session: &PebbleSessionState,
    request: WatchAnalysisRequest,
) -> Result<WatchAnalysis, AiRuntimeError> {
    let _guard = runtime.begin_request()?;
    ensure_watch_session_current(app, session, request.revision)?;
    let previous_image = encode_frame_data_url(&request.previous_frame)?;
    let current_image = encode_frame_data_url(&request.current_frame)?;

    match request.provider {
        AiProvider::OpenAi => {
            analyze_watch_openai(
                app,
                session,
                request.revision,
                request.locale,
                request.local_signal,
                previous_image,
                current_image,
            )
            .await
        }
        AiProvider::Claude => {
            analyze_watch_claude(
                app,
                session,
                request.revision,
                request.locale,
                request.local_signal,
                previous_image,
                current_image,
            )
            .await
        }
    }
}

async fn analyze_watch_openai(
    app: &AppHandle,
    session: &PebbleSessionState,
    revision: u64,
    locale: String,
    local_signal: &'static str,
    previous_image: String,
    current_image: String,
) -> Result<WatchAnalysis, AiRuntimeError> {
    let mut server = AppServerProcess::start(app).await?;
    if !read_chatgpt_account(&mut server).await?.connected {
        return Err(runtime_error(
            AiRuntimeErrorCode::NotConnected,
            "Connect OpenAI before enabling semantic Watch analysis.",
            true,
        ));
    }
    let model = watch_image_model(&mut server).await?;
    let thread = server
        .request(
            "thread/start",
            json!({
                "model": model,
                "cwd": server.codex_home,
                "approvalPolicy": "never",
                "sandbox": "read-only",
                "baseInstructions": WATCH_INSTRUCTIONS,
                "developerInstructions": WATCH_DEVELOPER_INSTRUCTIONS,
                "ephemeral": true
            }),
            REQUEST_TIMEOUT,
        )
        .await?;
    let thread_id = required_string(
        &thread,
        &["thread", "id"],
        "The private Watch analysis session did not start.",
    )?
    .to_string();
    ensure_watch_session_current(app, session, revision)?;
    let started = Instant::now();
    server
        .request(
            "turn/start",
            json!({
                "threadId": thread_id,
                "input": [
                    { "type": "text", "text": watch_prompt(&locale, local_signal), "text_elements": [] },
                    { "type": "image", "url": previous_image },
                    { "type": "image", "url": current_image }
                ],
                "approvalPolicy": "never",
                "model": model,
                "effort": OPENAI_REASONING_EFFORT,
                "summary": "none"
            }),
            REQUEST_TIMEOUT,
        )
        .await?;
    let summary = collect_answer(&mut server, &thread_id).await?;
    ensure_watch_session_current(app, session, revision)?;
    Ok(WatchAnalysis {
        summary,
        model,
        duration_ms: duration_millis(started.elapsed()),
    })
}

async fn ask_openai(
    app: &AppHandle,
    window: &WebviewWindow,
    session: &PebbleSessionState,
    authorized: &AuthorizedAiCapture,
    image_data_url: String,
    question: String,
    locale: String,
) -> Result<AiAnswer, AiRuntimeError> {
    let mut server = AppServerProcess::start(app).await?;
    let account = read_chatgpt_account(&mut server).await?;
    if !account.connected {
        return Err(runtime_error(
            AiRuntimeErrorCode::NotConnected,
            "Connect an OpenAI account before asking about the selected region.",
            true,
        ));
    }

    let model = balanced_image_model(&mut server).await?;
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
        "The private AI session did not start.",
    )?
    .to_string();

    ensure_capture_is_current(app, window, session, authorized)?;
    let generation_started = Instant::now();
    server
        .request(
            "turn/start",
            json!({
                "threadId": thread_id,
                "input": [
                    {
                        "type": "text",
                        "text": question_prompt(&question, &locale),
                        "text_elements": []
                    },
                    { "type": "image", "url": image_data_url }
                ],
                "approvalPolicy": "never",
                "model": model,
                "effort": OPENAI_REASONING_EFFORT,
                "summary": "none"
            }),
            REQUEST_TIMEOUT,
        )
        .await?;

    let answer = collect_answer(&mut server, &thread_id).await?;
    Ok(AiAnswer {
        answer,
        provider: AiProvider::OpenAi,
        model,
        duration_ms: duration_millis(generation_started.elapsed()),
    })
}

async fn claude_connection_status(app: &AppHandle) -> Result<AiConnectionStatus, AiRuntimeError> {
    let Some(binary) = claude_binary(app) else {
        return Ok(claude_status(false, false));
    };
    let output = timeout(
        REQUEST_TIMEOUT,
        claude_command(app, &binary)?
            .args(["auth", "status", "--json"])
            .output(),
    )
    .await
    .map_err(|_| timeout_error("CLAUDE AUTH STATUS TOOK TOO LONG."))?
    .map_err(|_| sidecar_error_for(AiProvider::Claude))?;
    Ok(claude_status(true, claude_auth_is_connected(&output)))
}

async fn connect_claude(app: &AppHandle) -> Result<AiConnectionStatus, AiRuntimeError> {
    let Some(binary) = claude_binary(app) else {
        app.opener()
            .open_url(CLAUDE_INSTALL_URL, None::<&str>)
            .map_err(|_| sidecar_error_for(AiProvider::Claude))?;
        return Ok(claude_status(false, false));
    };
    let current = claude_connection_status(app).await?;
    if current.connected {
        return Ok(current);
    }

    let status = timeout(
        LOGIN_TIMEOUT,
        claude_command(app, &binary)?
            .args(["auth", "login"])
            .status(),
    )
    .await
    .map_err(|_| timeout_error("CLAUDE SIGN-IN TOOK TOO LONG."))?
    .map_err(|_| sidecar_error_for(AiProvider::Claude))?;
    if !status.success() {
        return Err(runtime_error(
            AiRuntimeErrorCode::AuthenticationFailed,
            "CLAUDE SIGN-IN WAS NOT COMPLETED.",
            true,
        ));
    }
    claude_connection_status(app).await
}

async fn ask_claude(
    app: &AppHandle,
    window: &WebviewWindow,
    session: &PebbleSessionState,
    authorized: &AuthorizedAiCapture,
    image_data_url: String,
    question: String,
    locale: String,
) -> Result<AiAnswer, AiRuntimeError> {
    let binary = claude_binary(app).ok_or_else(|| sidecar_error_for(AiProvider::Claude))?;
    let status = claude_connection_status(app).await?;
    if !status.connected {
        return Err(runtime_error(
            AiRuntimeErrorCode::NotConnected,
            "CONNECT CLAUDE BEFORE ASKING ABOUT THE SELECTED REGION.",
            true,
        ));
    }
    ensure_capture_is_current(app, window, session, authorized)?;

    let mut command = claude_command(app, &binary)?;
    command.args([
        "-p",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--verbose",
        "--safe-mode",
        "--disable-slash-commands",
        "--strict-mcp-config",
        "--tools",
        "",
        "--disallowedTools",
        "*",
        "--model",
        CLAUDE_MODEL_ID,
        "--effort",
        CLAUDE_REASONING_EFFORT,
        "--max-turns",
        "1",
        "--system-prompt",
        BASE_INSTRUCTIONS,
    ]);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let generation_started = Instant::now();
    let mut child = command
        .spawn()
        .map_err(|_| sidecar_error_for(AiProvider::Claude))?;
    let input = claude_image_input(&question, &locale, &image_data_url)?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| sidecar_error_for(AiProvider::Claude))?;
    stdin
        .write_all(&input)
        .await
        .map_err(|_| sidecar_error_for(AiProvider::Claude))?;
    drop(stdin);
    let output = timeout(TURN_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| timeout_error("CLAUDE TOOK TOO LONG TO RESPOND."))?
        .map_err(|_| sidecar_error_for(AiProvider::Claude))?;
    if !output.status.success() {
        return Err(runtime_error(
            AiRuntimeErrorCode::ResponseFailed,
            "CLAUDE COULD NOT COMPLETE THIS IMAGE QUESTION.",
            true,
        ));
    }
    let (answer, model, reported_duration) = parse_claude_answer(&output.stdout)?;
    Ok(AiAnswer {
        answer,
        provider: AiProvider::Claude,
        model,
        duration_ms: reported_duration
            .unwrap_or_else(|| duration_millis(generation_started.elapsed())),
    })
}

async fn analyze_watch_claude(
    app: &AppHandle,
    session: &PebbleSessionState,
    revision: u64,
    locale: String,
    local_signal: &'static str,
    previous_image: String,
    current_image: String,
) -> Result<WatchAnalysis, AiRuntimeError> {
    let binary = claude_binary(app).ok_or_else(|| sidecar_error_for(AiProvider::Claude))?;
    if !claude_connection_status(app).await?.connected {
        return Err(runtime_error(
            AiRuntimeErrorCode::NotConnected,
            "CONNECT CLAUDE BEFORE ENABLING SEMANTIC WATCH ANALYSIS.",
            true,
        ));
    }
    ensure_watch_session_current(app, session, revision)?;
    let mut command = claude_command(app, &binary)?;
    command.args([
        "-p",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--verbose",
        "--safe-mode",
        "--disable-slash-commands",
        "--strict-mcp-config",
        "--tools",
        "",
        "--disallowedTools",
        "*",
        "--model",
        CLAUDE_MODEL_ID,
        "--effort",
        CLAUDE_REASONING_EFFORT,
        "--max-turns",
        "1",
        "--system-prompt",
        WATCH_INSTRUCTIONS,
    ]);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let started = Instant::now();
    let mut child = command
        .spawn()
        .map_err(|_| sidecar_error_for(AiProvider::Claude))?;
    let input = claude_watch_input(
        &watch_prompt(&locale, local_signal),
        &previous_image,
        &current_image,
    )?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| sidecar_error_for(AiProvider::Claude))?;
    stdin
        .write_all(&input)
        .await
        .map_err(|_| sidecar_error_for(AiProvider::Claude))?;
    drop(stdin);
    let output = timeout(TURN_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| timeout_error("CLAUDE WATCH ANALYSIS TOOK TOO LONG."))?
        .map_err(|_| sidecar_error_for(AiProvider::Claude))?;
    if !output.status.success() {
        return Err(response_error("CLAUDE COULD NOT COMPLETE WATCH ANALYSIS."));
    }
    let (summary, model, reported_duration) = parse_claude_answer(&output.stdout)?;
    ensure_watch_session_current(app, session, revision)?;
    Ok(WatchAnalysis {
        summary,
        model,
        duration_ms: reported_duration.unwrap_or_else(|| duration_millis(started.elapsed())),
    })
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
        return Ok(openai_status(false));
    };

    if account.get("type").and_then(Value::as_str) != Some("chatgpt") {
        return Err(runtime_error(
            AiRuntimeErrorCode::AuthenticationFailed,
            "Pebble accepts OpenAI account sign-in only; API keys are not used.",
            true,
        ));
    }

    Ok(openai_status(true))
}

async fn read_openai_connection_status(
    server: &mut AppServerProcess,
) -> Result<AiConnectionStatus, AiRuntimeError> {
    let mut status = read_chatgpt_account(server).await?;
    if status.connected {
        status.model = balanced_image_model(server).await?.to_ascii_uppercase();
    }
    Ok(status)
}

fn openai_status(connected: bool) -> AiConnectionStatus {
    AiConnectionStatus {
        provider: AiProvider::OpenAi,
        available: true,
        connected,
        model: OPENAI_MODEL_LABEL.to_string(),
        install_url: None,
    }
}

fn claude_status(available: bool, connected: bool) -> AiConnectionStatus {
    AiConnectionStatus {
        provider: AiProvider::Claude,
        available,
        connected: available && connected,
        model: CLAUDE_MODEL_LABEL.to_string(),
        install_url: (!available).then_some(CLAUDE_INSTALL_URL),
    }
}

async fn balanced_image_model(server: &mut AppServerProcess) -> Result<String, AiRuntimeError> {
    let response = server
        .request(
            "model/list",
            json!({ "limit": 100, "includeHidden": false }),
            REQUEST_TIMEOUT,
        )
        .await?;
    select_balanced_model(
        response
            .get("data")
            .and_then(Value::as_array)
            .map(Vec::as_slice),
    )
    .ok_or_else(|| {
        runtime_error(
            AiRuntimeErrorCode::ModelUnavailable,
            "This OpenAI account does not currently offer a supported balanced image model.",
            true,
        )
    })
}

async fn watch_image_model(server: &mut AppServerProcess) -> Result<String, AiRuntimeError> {
    let response = server
        .request(
            "model/list",
            json!({ "limit": 100, "includeHidden": false }),
            REQUEST_TIMEOUT,
        )
        .await?;
    select_image_model(
        response
            .get("data")
            .and_then(Value::as_array)
            .map(Vec::as_slice),
        &OPENAI_WATCH_MODEL_PREFERENCES,
    )
    .ok_or_else(|| {
        runtime_error(
            AiRuntimeErrorCode::ModelUnavailable,
            "This OpenAI account does not offer a supported Watch image model.",
            true,
        )
    })
}

fn select_balanced_model(models: Option<&[Value]>) -> Option<String> {
    select_image_model(models, &OPENAI_MODEL_PREFERENCES)
}

fn select_image_model(models: Option<&[Value]>, preferences: &[&str]) -> Option<String> {
    let candidates = models?.iter().filter(|model| {
        has_string(model, "inputModalities", "image")
            && model
                .get("supportedReasoningEfforts")
                .and_then(Value::as_array)
                .is_some_and(|efforts| {
                    efforts.iter().any(|effort| {
                        effort.get("reasoningEffort").and_then(Value::as_str)
                            == Some(OPENAI_REASONING_EFFORT)
                    })
                })
    });
    let candidates = candidates.collect::<Vec<_>>();

    for preferred in preferences {
        if let Some(model) = candidates
            .iter()
            .find(|model| model.get("model").and_then(Value::as_str) == Some(*preferred))
        {
            return model.get("model")?.as_str().map(str::to_string);
        }
    }

    None
}

fn watch_prompt(locale: &str, local_signal: &str) -> String {
    format!(
        "Reply locale: {}. Local visual signal: {}. Compare BEFORE and AFTER. State the concrete visible change, its likely significance, and uncertainty. Do not claim a cause that is not visible.",
        normalized_locale(locale),
        local_signal
    )
}

fn ensure_watch_session_current(
    app: &AppHandle,
    session: &PebbleSessionState,
    revision: u64,
) -> Result<(), AiRuntimeError> {
    let monitors = crate::current_monitor_geometries(app).map_err(capture_runtime_error)?;
    if session
        .frame_delivery_is_current(revision, &monitors)
        .map_err(|_| session_changed_error())?
    {
        Ok(())
    } else {
        Err(session_changed_error())
    }
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
                "The AI response requested an action outside the selected image, so Pebble stopped it.",
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
                        "The AI response attempted an action outside the selected image, so Pebble stopped it.",
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
                        "AI could not complete this image question.",
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
                        "AI returned an empty answer.",
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
            "The AI answer exceeded Pebble's compact response limit.",
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

fn question_prompt(question: &str, locale: &str) -> String {
    let locale = normalized_locale(locale);
    format!(
        "User locale: {locale}. Reply in the language used by the question; use the locale only as a fallback.\nQuestion about this selected screen region:\n{question}"
    )
}

fn normalized_locale(locale: &str) -> &str {
    let valid = !locale.is_empty()
        && locale.len() <= 35
        && locale
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        locale
    } else {
        "und"
    }
}

fn claude_binary(app: &AppHandle) -> Option<PathBuf> {
    let home = app.path().home_dir().ok()?;
    [
        home.join(".local/bin/claude"),
        PathBuf::from("/opt/homebrew/bin/claude"),
        PathBuf::from("/usr/local/bin/claude"),
    ]
    .into_iter()
    .find(|path| trusted_executable(path))
}

fn trusted_executable(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        mode & 0o111 != 0 && mode & 0o022 == 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn claude_command(app: &AppHandle, binary: &Path) -> Result<Command, AiRuntimeError> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|_| sidecar_error_for(AiProvider::Claude))?;
    let runtime_dir = app_data.join("claude-runtime");
    secure_directory(&runtime_dir)?;
    let home = app
        .path()
        .home_dir()
        .map_err(|_| sidecar_error_for(AiProvider::Claude))?;
    let mut command = Command::new(binary);
    command
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin")
        .env("LANG", "en_US.UTF-8")
        .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1")
        .current_dir(runtime_dir);
    Ok(command)
}

fn claude_auth_is_connected(output: &std::process::Output) -> bool {
    if !output.status.success() {
        return false;
    }
    serde_json::from_slice::<Value>(&output.stdout)
        .ok()
        .is_some_and(|value| {
            value.get("loggedIn").and_then(Value::as_bool) == Some(true)
                || value.get("authenticated").and_then(Value::as_bool) == Some(true)
        })
}

fn claude_image_input(
    question: &str,
    locale: &str,
    image_data_url: &str,
) -> Result<Vec<u8>, AiRuntimeError> {
    let image = image_data_url
        .strip_prefix("data:image/png;base64,")
        .ok_or_else(invalid_frame_error)?;
    let mut input = serde_json::to_vec(&json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": [
                { "type": "text", "text": question_prompt(question, locale) },
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": "image/png",
                        "data": image
                    }
                }
            ]
        },
        "parent_tool_use_id": Value::Null
    }))
    .map_err(|_| invalid_frame_error())?;
    input.push(b'\n');
    Ok(input)
}

fn claude_watch_input(
    prompt: &str,
    previous_image_data_url: &str,
    current_image_data_url: &str,
) -> Result<Vec<u8>, AiRuntimeError> {
    let previous = previous_image_data_url
        .strip_prefix("data:image/png;base64,")
        .ok_or_else(invalid_frame_error)?;
    let current = current_image_data_url
        .strip_prefix("data:image/png;base64,")
        .ok_or_else(invalid_frame_error)?;
    let mut input = serde_json::to_vec(&json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": [
                { "type": "text", "text": prompt },
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": "image/png",
                        "data": previous
                    }
                },
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": "image/png",
                        "data": current
                    }
                }
            ]
        },
        "parent_tool_use_id": Value::Null
    }))
    .map_err(|_| invalid_frame_error())?;
    input.push(b'\n');
    Ok(input)
}

fn parse_claude_answer(bytes: &[u8]) -> Result<(String, String, Option<u64>), AiRuntimeError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| response_error("CLAUDE RETURNED AN INVALID RESPONSE."))?;
    let mut answer = String::new();
    let mut model = CLAUDE_MODEL_ID.to_string();
    let mut duration_ms = None;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let value: Value = serde_json::from_str(line)
            .map_err(|_| response_error("CLAUDE RETURNED AN INVALID RESPONSE."))?;
        match value.get("type").and_then(Value::as_str) {
            Some("assistant") => {
                let message = value.get("message").unwrap_or(&Value::Null);
                if let Some(next_model) = message.get("model").and_then(Value::as_str) {
                    model = next_model.to_string();
                }
                for content in message
                    .get("content")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    match content.get("type").and_then(Value::as_str) {
                        Some("text") => append_answer(
                            &mut answer,
                            content
                                .get("text")
                                .and_then(Value::as_str)
                                .unwrap_or_default(),
                        )?,
                        Some("tool_use") => {
                            return Err(response_error("CLAUDE ATTEMPTED A DISALLOWED ACTION."));
                        }
                        _ => {}
                    }
                }
            }
            Some("result") => {
                duration_ms = value.get("duration_ms").and_then(Value::as_u64);
            }
            _ => {}
        }
    }
    let answer = answer.trim().to_string();
    if answer.is_empty() {
        return Err(response_error("CLAUDE RETURNED AN EMPTY ANSWER."));
    }
    Ok((answer, model, duration_ms))
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn timeout_error(message: &'static str) -> AiRuntimeError {
    runtime_error(AiRuntimeErrorCode::Timeout, message, true)
}

fn response_error(message: &'static str) -> AiRuntimeError {
    runtime_error(AiRuntimeErrorCode::ResponseFailed, message, true)
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

fn session_changed_error() -> AiRuntimeError {
    runtime_error(
        AiRuntimeErrorCode::SessionChanged,
        "Watch analysis stopped because the selected region changed.",
        true,
    )
}

fn ensure_authorized_window(window: &WebviewWindow) -> Result<(), AiRuntimeError> {
    let authorized = window.label() == PEBBLE_TILE_LABEL
        && window.is_visible().unwrap_or(false)
        && !window.is_minimized().unwrap_or(true);
    if authorized {
        Ok(())
    } else {
        Err(runtime_error(
            AiRuntimeErrorCode::UnauthorizedWindow,
            "Image questions are available only from the visible Pebble window.",
            true,
        ))
    }
}

fn validate_auth_url(value: &str) -> Result<(), AiRuntimeError> {
    let url = Url::parse(value).map_err(|_| invalid_auth_url_error())?;
    let host_allowed = matches!(
        url.host_str(),
        Some("chatgpt.com") | Some("auth.openai.com")
    );
    let origin_allowed = url.scheme() == "https"
        && host_allowed
        && url.username().is_empty()
        && url.password().is_none()
        && url.port_or_known_default() == Some(443);
    if origin_allowed {
        Ok(())
    } else {
        Err(invalid_auth_url_error())
    }
}

fn chatgpt_login_params() -> Value {
    json!({
        "type": "chatgpt",
        "useHostedLoginSuccessPage": true,
        "appBrand": "chatgpt"
    })
}

fn login_failure_message(error: Option<&str>) -> &'static str {
    let error = error.unwrap_or_default().to_ascii_lowercase();
    if error.contains("persist_failed")
        || error.contains("keychain")
        || error.contains("keyring")
        || error.contains("secure storage")
    {
        "OpenAI signed in, but Pebble could not access the system credential store. Make sure your login keychain is available, then try again."
    } else if error.contains("organization") || error.contains("workspace") {
        "This OpenAI workspace could not complete sign-in. Try another workspace or account."
    } else if error.contains("cancel") || error.contains("denied") {
        "OpenAI sign-in was cancelled. Try Connect OpenAI again."
    } else if error.contains("port") || error.contains("callback") || error.contains("localhost") {
        "OpenAI could not return to Pebble. Close other Codex login windows and try again."
    } else {
        "OpenAI sign-in failed. Try Connect OpenAI again."
    }
}

fn invalid_auth_url_error() -> AiRuntimeError {
    runtime_error(
        AiRuntimeErrorCode::AuthenticationFailed,
        "Pebble rejected an unexpected sign-in URL.",
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

        #[cfg(target_os = "macos")]
        let environment_home = app.path().home_dir().map_err(|_| sidecar_error())?;
        #[cfg(not(target_os = "macos"))]
        let environment_home = app_data_dir.clone();
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
            .env("HOME", environment_home.as_os_str())
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
                        "name": "pebble",
                        "title": "Pebble",
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
                        "The local AI service rejected this request.",
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
                        "The local AI service took too long to respond.",
                        true,
                    )
                })?;
            let event = timeout(remaining, self.receiver.recv())
                .await
                .map_err(|_| {
                    runtime_error(
                        AiRuntimeErrorCode::Timeout,
                        "The local AI service took too long to respond.",
                        true,
                    )
                })?
                .ok_or_else(sidecar_error)?;

            match event {
                CommandEvent::Stdout(line) => {
                    return serde_json::from_slice(&line).map_err(|_| {
                        runtime_error(
                            AiRuntimeErrorCode::ResponseFailed,
                            "The local AI service returned an invalid response.",
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
        "The bundled local AI service is unavailable.",
        true,
    )
}

fn sidecar_error_for(provider: AiProvider) -> AiRuntimeError {
    match provider {
        AiProvider::OpenAi => sidecar_error(),
        AiProvider::Claude => runtime_error(
            AiRuntimeErrorCode::SidecarUnavailable,
            "THE OFFICIAL CLAUDE CLI IS NOT AVAILABLE. INSTALL IT TO USE CLAUDE.",
            true,
        ),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        chatgpt_login_params, claude_image_input, encode_frame_data_url, login_failure_message,
        normalize_question, normalized_locale, parse_claude_answer, select_balanced_model,
        select_image_model, validate_auth_url, AiRuntimeErrorCode,
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
    fn accepts_only_bounded_language_tags() {
        assert_eq!(normalized_locale("ko-KR"), "ko-KR");
        assert_eq!(normalized_locale("../../secret"), "und");
        assert_eq!(normalized_locale(&"x".repeat(36)), "und");
    }

    #[test]
    fn claude_input_contains_one_text_and_one_memory_only_image() {
        let input = claude_image_input("무엇이 바뀌었어?", "ko-KR", "data:image/png;base64,AAAA")
            .expect("Claude input");
        let value: serde_json::Value = serde_json::from_slice(&input).expect("JSON line");
        let content = value["message"]["content"].as_array().expect("content");
        assert_eq!(content.len(), 2);
        assert_eq!(content[1]["source"]["data"], "AAAA");
        assert!(content[0]["text"].as_str().unwrap().contains("ko-KR"));
    }

    #[test]
    fn parses_claude_stream_metadata_without_accepting_tools() {
        let output = br#"{"type":"assistant","message":{"model":"claude-sonnet-5","content":[{"type":"text","text":"Visible change."}]}}
{"type":"result","duration_ms":1234}
"#;
        assert_eq!(
            parse_claude_answer(output).expect("answer"),
            (
                "Visible change.".to_string(),
                "claude-sonnet-5".to_string(),
                Some(1234)
            )
        );

        let tool =
            br#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read"}]}}"#;
        assert_eq!(
            parse_claude_answer(tool).unwrap_err().code,
            AiRuntimeErrorCode::ResponseFailed
        );
    }

    #[test]
    fn accepts_only_the_official_https_login_host() {
        assert!(validate_auth_url("https://chatgpt.com/auth/login?x=1").is_ok());
        assert!(validate_auth_url("https://auth.openai.com/oauth/authorize?x=1").is_ok());
        assert!(validate_auth_url("http://auth.openai.com/oauth/authorize").is_err());
        assert!(validate_auth_url("https://auth.openai.com.evil.test/oauth").is_err());
        assert!(validate_auth_url("https://evil.test@chatgpt.com/oauth").is_err());
    }

    #[test]
    fn maps_login_failures_without_exposing_raw_server_errors() {
        assert_eq!(
            login_failure_message(Some(
                "persist_failed: failed to write OAuth tokens to keyring: token=secret"
            )),
            "OpenAI signed in, but Pebble could not access the system credential store. Make sure your login keychain is available, then try again."
        );
        assert_eq!(
            login_failure_message(Some("callback port already in use: token=secret")),
            "OpenAI could not return to Pebble. Close other Codex login windows and try again."
        );
        assert_eq!(
            login_failure_message(Some("organization_not_supported")),
            "This OpenAI workspace could not complete sign-in. Try another workspace or account."
        );
    }

    #[test]
    fn requests_the_hosted_chatgpt_login_flow() {
        assert_eq!(
            chatgpt_login_params(),
            json!({
                "type": "chatgpt",
                "useHostedLoginSuccessPage": true,
                "appBrand": "chatgpt"
            })
        );
    }

    #[test]
    fn prefers_terra_and_allows_luna_without_falling_back_to_mini() {
        let models = vec![
            json!({
                "model": "gpt-5.6-luna",
                "inputModalities": ["text", "image"],
                "supportedReasoningEfforts": [{ "reasoningEffort": "medium" }]
            }),
            json!({
                "model": "gpt-5.6-terra",
                "inputModalities": ["text", "image"],
                "supportedReasoningEfforts": [{ "reasoningEffort": "medium" }]
            }),
        ];
        assert_eq!(
            select_balanced_model(Some(&models)).as_deref(),
            Some("gpt-5.6-terra")
        );

        assert_eq!(
            select_balanced_model(Some(&models[..1])).as_deref(),
            Some("gpt-5.6-luna")
        );

        let mini_only = vec![json!({
            "model": "gpt-5.4-mini",
            "inputModalities": ["text", "image"],
            "supportedReasoningEfforts": [{ "reasoningEffort": "medium" }]
        })];
        assert_eq!(select_balanced_model(Some(&mini_only)), None);
        assert_eq!(
            select_image_model(Some(&models), &super::OPENAI_WATCH_MODEL_PREFERENCES).as_deref(),
            Some("gpt-5.6-terra")
        );
        assert_eq!(
            select_image_model(Some(&models[..1]), &super::OPENAI_WATCH_MODEL_PREFERENCES),
            None
        );
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
                source_window: None,
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
