use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::Serialize;
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::time::sleep;
use url::Url;

use crate::{
    activity_feed::ActivityFeedState,
    public_source_fetch::{fetch_source, validate_source_url, FetchedSource},
};

const CHECK_INTERVAL: Duration = Duration::from_secs(15 * 60);
const CHECK_INTERVAL_MINUTES: u16 = 15;
pub const PUBLIC_SOURCE_STATUS_EVENT: &str = "pebble://public-source-status";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicSourceStatus {
    pub enabled: bool,
    pub url: Option<String>,
    pub interval_minutes: u16,
    pub last_checked_at: Option<String>,
    pub title: Option<String>,
    pub error: Option<&'static str>,
}

#[derive(Debug, Clone, Default)]
pub struct PublicSourceState {
    data: Arc<Mutex<PublicSourceData>>,
}

#[derive(Debug, Clone)]
struct PublicSourceData {
    generation: u64,
    status: PublicSourceStatus,
    fingerprint: Option<u64>,
}

impl Default for PublicSourceData {
    fn default() -> Self {
        Self {
            generation: 0,
            status: PublicSourceStatus {
                enabled: false,
                url: None,
                interval_minutes: CHECK_INTERVAL_MINUTES,
                last_checked_at: None,
                title: None,
                error: None,
            },
            fingerprint: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PublicSourceErrorCode {
    InvalidUrl,
    PrivateDestination,
    RequestFailed,
    ResponseTooLarge,
    UnsupportedResponse,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicSourceError {
    pub code: PublicSourceErrorCode,
    pub message: &'static str,
}

impl PublicSourceState {
    pub fn status(&self) -> PublicSourceStatus {
        self.data
            .lock()
            .map(|data| data.status.clone())
            .unwrap_or_default()
    }

    fn begin(&self, url: String) -> Result<(u64, PublicSourceStatus), PublicSourceError> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| PublicSourceError::unavailable())?;
        data.generation = data.generation.saturating_add(1);
        data.fingerprint = None;
        data.status = PublicSourceStatus {
            enabled: true,
            url: Some(url),
            interval_minutes: CHECK_INTERVAL_MINUTES,
            last_checked_at: None,
            title: None,
            error: None,
        };
        Ok((data.generation, data.status.clone()))
    }

    pub fn disable(&self) -> PublicSourceStatus {
        let mut data = self.data.lock().expect("public source state lock");
        data.generation = data.generation.saturating_add(1);
        data.status.enabled = false;
        data.status.url = None;
        data.status.error = None;
        data.fingerprint = None;
        data.status.clone()
    }

    fn is_current(&self, generation: u64) -> bool {
        self.data
            .lock()
            .is_ok_and(|data| data.generation == generation && data.status.enabled)
    }

    fn apply(
        &self,
        generation: u64,
        fetched: FetchedSource,
    ) -> Option<(PublicSourceStatus, bool, bool)> {
        let mut data = self.data.lock().ok()?;
        if data.generation != generation || !data.status.enabled {
            return None;
        }
        let initial = data.fingerprint.is_none();
        let changed = data
            .fingerprint
            .is_some_and(|previous| previous != fetched.fingerprint);
        data.fingerprint = Some(fetched.fingerprint);
        data.status.last_checked_at = Some(now());
        data.status.title = Some(fetched.title);
        data.status.error = None;
        Some((data.status.clone(), initial, changed))
    }

    fn mark_error(&self, generation: u64) -> Option<PublicSourceStatus> {
        let mut data = self.data.lock().ok()?;
        if data.generation != generation || !data.status.enabled {
            return None;
        }
        data.status.last_checked_at = Some(now());
        data.status.error = Some("SOURCE CHECK FAILED");
        Some(data.status.clone())
    }
}

impl Default for PublicSourceStatus {
    fn default() -> Self {
        PublicSourceData::default().status
    }
}

impl PublicSourceError {
    pub(crate) fn invalid_url() -> Self {
        Self {
            code: PublicSourceErrorCode::InvalidUrl,
            message: "ENTER A PUBLIC HTTPS RSS, ATOM, OR WEB URL.",
        }
    }

    pub(crate) fn private_destination() -> Self {
        Self {
            code: PublicSourceErrorCode::PrivateDestination,
            message: "LOCAL, PRIVATE, AND RESERVED NETWORK DESTINATIONS ARE BLOCKED.",
        }
    }

    pub(crate) fn request_failed() -> Self {
        Self {
            code: PublicSourceErrorCode::RequestFailed,
            message: "THE PUBLIC SOURCE COULD NOT BE CHECKED.",
        }
    }

    pub(crate) fn response_too_large() -> Self {
        Self {
            code: PublicSourceErrorCode::ResponseTooLarge,
            message: "THE PUBLIC SOURCE RESPONSE EXCEEDED 512 KB.",
        }
    }

    pub(crate) fn unsupported_response() -> Self {
        Self {
            code: PublicSourceErrorCode::UnsupportedResponse,
            message: "THE PUBLIC SOURCE DID NOT RETURN SUPPORTED TEXT DATA.",
        }
    }

    pub(crate) fn unavailable() -> Self {
        Self {
            code: PublicSourceErrorCode::Unavailable,
            message: "PUBLIC SOURCE WATCH IS UNAVAILABLE.",
        }
    }
}

pub async fn follow(
    app: tauri::AppHandle,
    state: PublicSourceState,
    url: String,
) -> Result<PublicSourceStatus, PublicSourceError> {
    let url = validate_source_url(&url)?;
    let (generation, status) = state.begin(url.to_string())?;
    emit_status(&app, status);
    let fetched = match fetch_source(&url).await {
        Ok(fetched) => fetched,
        Err(error) => {
            emit_status(&app, state.disable());
            return Err(error);
        }
    };
    apply_fetch(&app, &state, generation, &url, fetched);
    if !state.is_current(generation) {
        return Ok(state.status());
    }

    let loop_app = app.clone();
    let loop_state = state.clone();
    let loop_url = url.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            sleep(CHECK_INTERVAL).await;
            if !loop_state.is_current(generation) {
                break;
            }
            match fetch_source(&loop_url).await {
                Ok(fetched) => apply_fetch(&loop_app, &loop_state, generation, &loop_url, fetched),
                Err(_) => {
                    if let Some(status) = loop_state.mark_error(generation) {
                        emit_status(&loop_app, status);
                    }
                }
            }
        }
    });
    Ok(state.status())
}

fn apply_fetch(
    app: &tauri::AppHandle,
    state: &PublicSourceState,
    generation: u64,
    url: &Url,
    fetched: FetchedSource,
) {
    let Some((status, initial, changed)) = state.apply(generation, fetched) else {
        return;
    };
    emit_status(app, status.clone());
    if !(initial || changed) {
        return;
    }
    let Some(title) = status.title.as_deref() else {
        return;
    };
    crate::activity_feed::record_source(
        app,
        app.state::<ActivityFeedState>().inner(),
        title,
        url.as_str(),
    );
    if changed {
        crate::menu_bar::set_attention(app, true);
        let _ = app
            .notification()
            .builder()
            .title("PUBLIC SOURCE UPDATED")
            .body(title)
            .show();
    }
}

pub fn emit_status(app: &tauri::AppHandle, status: PublicSourceStatus) {
    let _ = app.emit_to(
        crate::pebble_session::PEBBLE_TILE_LABEL,
        PUBLIC_SOURCE_STATUS_EVENT,
        status,
    );
}

fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "UNKNOWN".to_string())
}
