use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;

use crate::ai_runtime::AiProvider;

pub const SMART_WATCH_CONSENT_VERSION: u16 = 4;
pub const SMART_WATCH_SESSION_LIMIT: u16 = 6;
pub const SMART_WATCH_STATUS_EVENT: &str = "pebble://smart-watch-status";
pub const STARTUP_NOTICE_TITLE: &str = "PEBBLE WATCH";
pub const STARTUP_NOTICE_BODY: &str =
    "WHEN ENABLED, WATCH CHECKS THE SELECTED REGION EVERY 5S, INCLUDING WHILE THE WINDOW IS HIDDEN. ONLY MATERIAL CHANGES ARE SENT TO YOUR CHOSEN AI. LIMITED TO 6 ANALYSES PER APP SESSION.";

pub fn show_startup_notice(app: &tauri::AppHandle) {
    let _ = app
        .notification()
        .builder()
        .title(STARTUP_NOTICE_TITLE)
        .body(STARTUP_NOTICE_BODY)
        .show();
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartWatchStatus {
    pub enabled: bool,
    pub notifications_sent: u16,
    pub session_limit: u16,
    pub remaining: u16,
}

#[derive(Debug, Clone, Default)]
pub struct SmartWatchState {
    data: Arc<Mutex<SmartWatchData>>,
}

#[derive(Debug)]
struct SmartWatchData {
    enabled: bool,
    revision: Option<u64>,
    notifications_sent: u16,
    analysis_in_flight: bool,
    provider: AiProvider,
    locale: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchAnalysisContext {
    pub provider: AiProvider,
    pub locale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSmartWatchRequest {
    pub enabled: bool,
    pub consent_version: u16,
    pub provider: AiProvider,
    pub locale: String,
}

impl SmartWatchState {
    pub fn configure(
        &self,
        enabled: bool,
        revision: u64,
        consent_version: u16,
        provider: AiProvider,
        locale: String,
    ) -> Result<SmartWatchStatus, SmartWatchError> {
        if enabled && consent_version != SMART_WATCH_CONSENT_VERSION {
            return Err(SmartWatchError::consent_required());
        }

        let mut data = self
            .data
            .lock()
            .map_err(|_| SmartWatchError::unavailable())?;
        data.enabled = enabled;
        data.revision = enabled.then_some(revision);
        data.analysis_in_flight = false;
        data.provider = provider;
        data.locale = normalized_locale(locale);
        Ok(data.status())
    }

    pub fn disable(&self) -> SmartWatchStatus {
        let mut data = self.data.lock().expect("smart watch state lock");
        data.enabled = false;
        data.revision = None;
        data.analysis_in_flight = false;
        data.status()
    }

    pub fn status(&self) -> SmartWatchStatus {
        let data = self.data.lock().expect("smart watch state lock");
        data.status()
    }

    pub fn begin_analysis(&self, revision: u64) -> Option<WatchAnalysisContext> {
        let mut data = self.data.lock().ok()?;
        if !data.enabled {
            return None;
        }
        if data.revision != Some(revision) {
            data.enabled = false;
            data.revision = None;
            return None;
        }
        if data.analysis_in_flight || data.notifications_sent >= SMART_WATCH_SESSION_LIMIT {
            return None;
        }

        data.analysis_in_flight = true;
        Some(WatchAnalysisContext {
            provider: data.provider,
            locale: data.locale.clone(),
        })
    }

    pub fn finish_analysis(&self, revision: u64, completed: bool) -> (SmartWatchStatus, bool) {
        let mut data = self.data.lock().expect("smart watch state lock");
        let accepted = data.enabled && data.revision == Some(revision);
        if accepted {
            data.analysis_in_flight = false;
            if completed {
                data.notifications_sent = data.notifications_sent.saturating_add(1);
            }
        }
        (data.status(), accepted)
    }
}

impl Default for SmartWatchData {
    fn default() -> Self {
        Self {
            enabled: false,
            revision: None,
            notifications_sent: 0,
            analysis_in_flight: false,
            provider: AiProvider::OpenAi,
            locale: "und".to_string(),
        }
    }
}

fn normalized_locale(locale: String) -> String {
    let valid = !locale.is_empty()
        && locale.len() <= 35
        && locale
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        locale
    } else {
        "und".to_string()
    }
}

impl SmartWatchData {
    fn status(&self) -> SmartWatchStatus {
        SmartWatchStatus {
            enabled: self.enabled,
            notifications_sent: self.notifications_sent,
            session_limit: SMART_WATCH_SESSION_LIMIT,
            remaining: SMART_WATCH_SESSION_LIMIT.saturating_sub(self.notifications_sent),
        }
    }
}

pub fn emit_status(app: &tauri::AppHandle, status: SmartWatchStatus) {
    let _ = app.emit_to(
        crate::pebble_session::PEBBLE_TILE_LABEL,
        SMART_WATCH_STATUS_EVENT,
        status,
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SmartWatchErrorCode {
    ConsentRequired,
    InvalidSession,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartWatchError {
    pub code: SmartWatchErrorCode,
    pub message: &'static str,
}

impl SmartWatchError {
    fn consent_required() -> Self {
        Self {
            code: SmartWatchErrorCode::ConsentRequired,
            message: "REVIEW AND ACCEPT THE SMART WATCH NOTICE BEFORE ENABLING IT.",
        }
    }

    pub fn invalid_session() -> Self {
        Self {
            code: SmartWatchErrorCode::InvalidSession,
            message: "SMART WATCH NEEDS A VISIBLE, ACTIVE SELECTED REGION.",
        }
    }

    pub fn unavailable() -> Self {
        Self {
            code: SmartWatchErrorCode::Unavailable,
            message: "SMART WATCH STATE IS UNAVAILABLE.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SmartWatchErrorCode, SmartWatchState, SMART_WATCH_CONSENT_VERSION,
        SMART_WATCH_SESSION_LIMIT, STARTUP_NOTICE_BODY,
    };

    #[test]
    fn startup_notice_explains_activation_and_local_privacy() {
        assert!(STARTUP_NOTICE_BODY.contains("EVERY 5S"));
        assert!(STARTUP_NOTICE_BODY.contains("WINDOW IS HIDDEN"));
        assert!(STARTUP_NOTICE_BODY.contains("ONLY MATERIAL CHANGES"));
        assert!(STARTUP_NOTICE_BODY.contains("6 ANALYSES"));
    }

    #[test]
    fn watch_is_off_until_current_consent_is_supplied() {
        let state = SmartWatchState::default();
        assert!(!state.status().enabled);
        assert_eq!(
            state
                .configure(
                    true,
                    7,
                    0,
                    crate::ai_runtime::AiProvider::OpenAi,
                    "ko-KR".into()
                )
                .unwrap_err()
                .code,
            SmartWatchErrorCode::ConsentRequired
        );
        assert!(
            state
                .configure(
                    true,
                    7,
                    SMART_WATCH_CONSENT_VERSION,
                    crate::ai_runtime::AiProvider::OpenAi,
                    "ko-KR".into()
                )
                .unwrap()
                .enabled
        );
    }

    #[test]
    fn region_change_disables_watch() {
        let state = SmartWatchState::default();
        state
            .configure(
                true,
                7,
                SMART_WATCH_CONSENT_VERSION,
                crate::ai_runtime::AiProvider::OpenAi,
                "ko-KR".into(),
            )
            .unwrap();

        assert!(state.begin_analysis(8).is_none());
        assert!(!state.status().enabled);
    }

    #[test]
    fn watch_exposes_a_bounded_session_notification_budget() {
        let state = SmartWatchState::default();
        state
            .configure(
                true,
                2,
                SMART_WATCH_CONSENT_VERSION,
                crate::ai_runtime::AiProvider::OpenAi,
                "ko-KR".into(),
            )
            .unwrap();

        for _ in 0..SMART_WATCH_SESSION_LIMIT {
            assert!(state.begin_analysis(2).is_some());
            state.finish_analysis(2, true);
        }
        assert!(state.begin_analysis(2).is_none());
        assert_eq!(state.status().remaining, 0);
    }

    #[test]
    fn failed_or_cancelled_analysis_does_not_consume_budget_or_emit_late_results() {
        let state = SmartWatchState::default();
        state
            .configure(
                true,
                9,
                SMART_WATCH_CONSENT_VERSION,
                crate::ai_runtime::AiProvider::OpenAi,
                "ko-KR".into(),
            )
            .unwrap();

        assert!(state.begin_analysis(9).is_some());
        assert!(state.begin_analysis(9).is_none());
        let (status, accepted) = state.finish_analysis(9, false);
        assert!(accepted);
        assert_eq!(status.remaining, SMART_WATCH_SESSION_LIMIT);

        assert!(state.begin_analysis(9).is_some());
        state.disable();
        let (_, accepted) = state.finish_analysis(9, true);
        assert!(!accepted);
    }

    #[test]
    fn disable_stops_notifications_immediately() {
        let state = SmartWatchState::default();
        state
            .configure(
                true,
                3,
                SMART_WATCH_CONSENT_VERSION,
                crate::ai_runtime::AiProvider::OpenAi,
                "ko-KR".into(),
            )
            .unwrap();
        state.disable();

        assert!(state.begin_analysis(3).is_none());
    }
}
