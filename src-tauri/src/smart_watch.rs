use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;

use crate::ai_runtime::AiProvider;

pub const SMART_WATCH_CONSENT_VERSION: u16 = 5;
pub const WATCH_CAPTURE_INTERVAL_SECONDS: u64 = 5;
pub const DEFAULT_ANALYSIS_INTERVAL_MINUTES: u16 = 5;
pub const ANALYSIS_INTERVAL_OPTIONS_MINUTES: [u16; 4] = [1, 5, 30, 60];
pub const SMART_WATCH_STATUS_EVENT: &str = "pebble://smart-watch-status";
pub const STARTUP_NOTICE_TITLE: &str = "PEBBLE WATCH";
pub const STARTUP_NOTICE_BODY: &str =
    "WHEN ENABLED, WATCH CHECKS ONLY THE SELECTED REGION EVERY 5S, INCLUDING WHILE THE WINDOW IS HIDDEN. AI RUNS ONLY AFTER A MATERIAL CHANGE AND NO MORE OFTEN THAN YOUR SELECTED INTERVAL.";

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
    pub analyses_completed: u32,
    pub analysis_interval_minutes: u16,
    pub model: String,
}

#[derive(Debug, Clone, Default)]
pub struct SmartWatchState {
    data: Arc<Mutex<SmartWatchData>>,
}

#[derive(Debug)]
struct SmartWatchData {
    enabled: bool,
    revision: Option<u64>,
    analyses_completed: u32,
    analysis_in_flight: bool,
    analysis_interval_minutes: u16,
    last_analysis_tick: Option<u64>,
    provider: AiProvider,
    model: String,
    locale: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchAnalysisContext {
    pub provider: AiProvider,
    pub model: String,
    pub locale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSmartWatchRequest {
    pub enabled: bool,
    pub consent_version: u16,
    pub provider: AiProvider,
    pub model: String,
    pub locale: String,
    pub analysis_interval_minutes: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSmartWatchIntervalRequest {
    pub analysis_interval_minutes: u16,
}

impl SmartWatchState {
    pub fn configure(
        &self,
        enabled: bool,
        revision: u64,
        consent_version: u16,
        provider: AiProvider,
        model: String,
        locale: String,
        analysis_interval_minutes: u16,
    ) -> Result<SmartWatchStatus, SmartWatchError> {
        if enabled && consent_version != SMART_WATCH_CONSENT_VERSION {
            return Err(SmartWatchError::consent_required());
        }
        validate_analysis_interval(analysis_interval_minutes)?;
        let model = normalized_model(provider, model)?;

        let mut data = self
            .data
            .lock()
            .map_err(|_| SmartWatchError::unavailable())?;
        data.enabled = enabled;
        data.revision = enabled.then_some(revision);
        data.analysis_in_flight = false;
        data.last_analysis_tick = None;
        data.analysis_interval_minutes = analysis_interval_minutes;
        data.provider = provider;
        data.model = model;
        data.locale = normalized_locale(locale);
        Ok(data.status())
    }

    pub fn disable(&self) -> SmartWatchStatus {
        let mut data = self.data.lock().expect("smart watch state lock");
        data.enabled = false;
        data.revision = None;
        data.analysis_in_flight = false;
        data.last_analysis_tick = None;
        data.status()
    }

    pub fn status(&self) -> SmartWatchStatus {
        let data = self.data.lock().expect("smart watch state lock");
        data.status()
    }

    pub fn set_analysis_interval(
        &self,
        analysis_interval_minutes: u16,
    ) -> Result<SmartWatchStatus, SmartWatchError> {
        validate_analysis_interval(analysis_interval_minutes)?;
        let mut data = self
            .data
            .lock()
            .map_err(|_| SmartWatchError::unavailable())?;
        data.analysis_interval_minutes = analysis_interval_minutes;
        Ok(data.status())
    }

    pub fn begin_analysis(&self, revision: u64, tick: u64) -> Option<WatchAnalysisContext> {
        let mut data = self.data.lock().ok()?;
        if !data.enabled {
            return None;
        }
        if data.revision != Some(revision) {
            data.enabled = false;
            data.revision = None;
            data.last_analysis_tick = None;
            return None;
        }
        if data.analysis_in_flight || !data.analysis_interval_elapsed(tick) {
            return None;
        }

        data.analysis_in_flight = true;
        data.last_analysis_tick = Some(tick);
        Some(WatchAnalysisContext {
            provider: data.provider,
            model: data.model.clone(),
            locale: data.locale.clone(),
        })
    }

    pub fn finish_analysis(&self, revision: u64, completed: bool) -> (SmartWatchStatus, bool) {
        let mut data = self.data.lock().expect("smart watch state lock");
        let accepted = data.enabled && data.revision == Some(revision);
        if accepted {
            data.analysis_in_flight = false;
            if completed {
                data.analyses_completed = data.analyses_completed.saturating_add(1);
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
            analyses_completed: 0,
            analysis_in_flight: false,
            analysis_interval_minutes: DEFAULT_ANALYSIS_INTERVAL_MINUTES,
            last_analysis_tick: None,
            provider: AiProvider::OpenAi,
            model: "gpt-5.6-terra".to_string(),
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
            analyses_completed: self.analyses_completed,
            analysis_interval_minutes: self.analysis_interval_minutes,
            model: self.model.clone(),
        }
    }

    fn analysis_interval_elapsed(&self, tick: u64) -> bool {
        self.last_analysis_tick
            .map(|last_tick| {
                tick.saturating_sub(last_tick)
                    >= analysis_interval_ticks(self.analysis_interval_minutes)
            })
            .unwrap_or(true)
    }
}

fn normalized_model(provider: AiProvider, model: String) -> Result<String, SmartWatchError> {
    let valid = match provider {
        AiProvider::OpenAi => matches!(
            model.as_str(),
            "gpt-5.6-sol" | "gpt-5.6-terra" | "gpt-5.6-luna"
        ),
        AiProvider::Claude => {
            matches!(model.as_str(), "sonnet" | "opus")
                || (model.starts_with("claude-")
                    && model.len() <= 100
                    && model
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-'))
        }
    };
    if valid {
        Ok(model)
    } else {
        Err(SmartWatchError::unavailable())
    }
}

fn validate_analysis_interval(minutes: u16) -> Result<(), SmartWatchError> {
    if ANALYSIS_INTERVAL_OPTIONS_MINUTES.contains(&minutes) {
        Ok(())
    } else {
        Err(SmartWatchError::invalid_interval())
    }
}

fn analysis_interval_ticks(minutes: u16) -> u64 {
    u64::from(minutes) * 60 / WATCH_CAPTURE_INTERVAL_SECONDS
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
    InvalidInterval,
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

    fn invalid_interval() -> Self {
        Self {
            code: SmartWatchErrorCode::InvalidInterval,
            message: "CHOOSE A WATCH INTERVAL OF 1, 5, 30, OR 60 MINUTES.",
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
        SmartWatchErrorCode, SmartWatchState, DEFAULT_ANALYSIS_INTERVAL_MINUTES,
        SMART_WATCH_CONSENT_VERSION, STARTUP_NOTICE_BODY,
    };
    use crate::ai_runtime::AiProvider;

    #[test]
    fn startup_notice_explains_activation_and_local_privacy() {
        assert!(STARTUP_NOTICE_BODY.contains("EVERY 5S"));
        assert!(STARTUP_NOTICE_BODY.contains("ONLY THE SELECTED REGION"));
        assert!(STARTUP_NOTICE_BODY.contains("WINDOW IS HIDDEN"));
        assert!(STARTUP_NOTICE_BODY.contains("SELECTED INTERVAL"));
        assert!(!STARTUP_NOTICE_BODY.contains("6 ANALYSES"));
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
                    AiProvider::OpenAi,
                    "gpt-5.6-terra".into(),
                    "ko-KR".into(),
                    DEFAULT_ANALYSIS_INTERVAL_MINUTES,
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
                    AiProvider::OpenAi,
                    "gpt-5.6-terra".into(),
                    "ko-KR".into(),
                    DEFAULT_ANALYSIS_INTERVAL_MINUTES,
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
                AiProvider::OpenAi,
                "gpt-5.6-terra".into(),
                "ko-KR".into(),
                DEFAULT_ANALYSIS_INTERVAL_MINUTES,
            )
            .unwrap();

        assert!(state.begin_analysis(8, 1).is_none());
        assert!(!state.status().enabled);
    }

    #[test]
    fn watch_interval_accepts_only_supported_values() {
        let state = SmartWatchState::default();
        assert_eq!(state.status().analysis_interval_minutes, 5);
        assert_eq!(
            state
                .set_analysis_interval(1)
                .unwrap()
                .analysis_interval_minutes,
            1
        );
        assert_eq!(
            state
                .set_analysis_interval(30)
                .unwrap()
                .analysis_interval_minutes,
            30
        );
        assert_eq!(
            state.set_analysis_interval(2).unwrap_err().code,
            SmartWatchErrorCode::InvalidInterval
        );
    }

    #[test]
    fn selected_interval_limits_ai_analysis_without_stopping_local_watch() {
        let state = enabled_watch(5);

        assert!(state.begin_analysis(2, 1).is_some());
        state.finish_analysis(2, true);
        assert!(state.begin_analysis(2, 60).is_none());
        assert!(state.status().enabled);
        assert!(state.begin_analysis(2, 61).is_some());
    }

    #[test]
    fn changing_interval_applies_to_the_next_analysis() {
        let state = enabled_watch(30);

        assert!(state.begin_analysis(2, 1).is_some());
        state.finish_analysis(2, true);
        state.set_analysis_interval(1).unwrap();
        assert!(state.begin_analysis(2, 13).is_some());
    }

    #[test]
    fn watch_has_no_fixed_session_analysis_cap() {
        let state = enabled_watch(1);

        for analysis in 0..20_u64 {
            assert!(state.begin_analysis(2, analysis * 12).is_some());
            state.finish_analysis(2, true);
        }
        assert_eq!(state.status().analyses_completed, 20);
        assert!(state.status().enabled);
    }

    #[test]
    fn failed_or_cancelled_analysis_does_not_count_or_emit_late_results() {
        let state = enabled_watch(1);

        assert!(state.begin_analysis(2, 1).is_some());
        assert!(state.begin_analysis(2, 2).is_none());
        let (status, accepted) = state.finish_analysis(2, false);
        assert!(accepted);
        assert_eq!(status.analyses_completed, 0);

        assert!(state.begin_analysis(2, 13).is_some());
        state.disable();
        let (_, accepted) = state.finish_analysis(2, true);
        assert!(!accepted);
    }

    #[test]
    fn disable_stops_notifications_immediately() {
        let state = enabled_watch(5);
        state.disable();

        assert!(state.begin_analysis(2, 1).is_none());
    }

    fn enabled_watch(interval_minutes: u16) -> SmartWatchState {
        let state = SmartWatchState::default();
        state
            .configure(
                true,
                2,
                SMART_WATCH_CONSENT_VERSION,
                AiProvider::OpenAi,
                "gpt-5.6-terra".into(),
                "ko-KR".into(),
                interval_minutes,
            )
            .unwrap();
        state
    }
}
