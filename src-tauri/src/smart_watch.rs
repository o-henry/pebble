use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;

use crate::{
    ai_runtime::AiProvider,
    watch_intent::{CompiledWatchIntent, WatchEvaluationMode},
};

pub const SMART_WATCH_CONSENT_VERSION: u16 = 6;
pub const WATCH_CAPTURE_INTERVAL_SECONDS: u64 = 5;
pub const DEFAULT_ANALYSIS_INTERVAL_MINUTES: u16 = 5;
pub const ANALYSIS_INTERVAL_OPTIONS_MINUTES: [u16; 4] = [1, 5, 30, 60];
pub const SMART_WATCH_STATUS_EVENT: &str = "pebble://smart-watch-status";
pub const STARTUP_NOTICE_TITLE: &str = "PEBBLE WATCH";
pub const STARTUP_NOTICE_BODY: &str =
    "WHEN ENABLED, WATCH CHECKS ONLY THE SELECTED REGION EVERY 5S, INCLUDING WHILE THE WINDOW IS HIDDEN. APPLE VISION OCR AND AI RUN ONLY AFTER A MATERIAL CHANGE AND NO MORE OFTEN THAN YOUR SELECTED INTERVAL.";
pub const DEFAULT_WATCH_INTENT: &str = "Notify me about any meaningful content change.";
const MAX_WATCH_INTENT_CHARS: usize = 500;

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
    pub local_matches_completed: u32,
    pub suppressed_events: u32,
    pub analysis_interval_minutes: u16,
    pub model: String,
    pub custom_intent: bool,
    pub evaluation_mode: WatchEvaluationMode,
    pub rule_summary: String,
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
    local_matches_completed: u32,
    suppressed_events: u32,
    analysis_in_flight: bool,
    analysis_interval_minutes: u16,
    last_analysis_tick: Option<u64>,
    last_event: Option<WatchEventFingerprint>,
    provider: AiProvider,
    model: String,
    plan: CompiledWatchIntent,
    custom_intent: bool,
    locale: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WatchAnalysisContext {
    pub provider: AiProvider,
    pub model: String,
    pub intent: String,
    pub locale: String,
    pub plan: CompiledWatchIntent,
}

#[derive(Debug, Clone)]
struct WatchEventFingerprint {
    value: String,
    tick: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSmartWatchRequest {
    pub enabled: bool,
    pub consent_version: u16,
    pub provider: AiProvider,
    pub model: String,
    pub intent: String,
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
        revision: u64,
        request: SetSmartWatchRequest,
    ) -> Result<SmartWatchStatus, SmartWatchError> {
        let SetSmartWatchRequest {
            enabled,
            consent_version,
            provider,
            model,
            intent,
            locale,
            analysis_interval_minutes,
        } = request;
        if enabled && consent_version != SMART_WATCH_CONSENT_VERSION {
            return Err(SmartWatchError::consent_required());
        }
        validate_analysis_interval(analysis_interval_minutes)?;
        let model = normalized_model(provider, model)?;
        let (intent, custom_intent) = normalized_intent(intent)?;
        let plan = CompiledWatchIntent::compile(intent);

        let mut data = self
            .data
            .lock()
            .map_err(|_| SmartWatchError::unavailable())?;
        data.enabled = enabled;
        data.revision = enabled.then_some(revision);
        data.analysis_in_flight = false;
        data.last_analysis_tick = None;
        data.last_event = None;
        data.analysis_interval_minutes = analysis_interval_minutes;
        data.provider = provider;
        data.model = model;
        data.plan = plan;
        data.custom_intent = custom_intent;
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
            intent: data.plan.intent().to_string(),
            locale: data.locale.clone(),
            plan: data.plan.clone(),
        })
    }

    pub fn current_context(&self, revision: u64) -> Option<WatchAnalysisContext> {
        let data = self.data.lock().ok()?;
        if !data.enabled || data.revision != Some(revision) || data.analysis_in_flight {
            return None;
        }
        Some(WatchAnalysisContext {
            provider: data.provider,
            model: data.model.clone(),
            intent: data.plan.intent().to_string(),
            locale: data.locale.clone(),
            plan: data.plan.clone(),
        })
    }

    pub fn finish_local_match(
        &self,
        revision: u64,
        fingerprint: &str,
        tick: u64,
    ) -> (SmartWatchStatus, bool) {
        let mut data = self.data.lock().expect("smart watch state lock");
        let accepted =
            data.enabled && data.revision == Some(revision) && data.accept_event(fingerprint, tick);
        if accepted {
            data.local_matches_completed = data.local_matches_completed.saturating_add(1);
        }
        (data.status(), accepted)
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

    pub fn accept_ai_event(
        &self,
        revision: u64,
        summary: &str,
        tick: u64,
    ) -> (SmartWatchStatus, bool) {
        let mut data = self.data.lock().expect("smart watch state lock");
        let fingerprint = semantic_fingerprint(summary);
        let accepted = data.enabled
            && data.revision == Some(revision)
            && data.accept_event(&fingerprint, tick);
        (data.status(), accepted)
    }
}

impl Default for SmartWatchData {
    fn default() -> Self {
        Self {
            enabled: false,
            revision: None,
            analyses_completed: 0,
            local_matches_completed: 0,
            suppressed_events: 0,
            analysis_in_flight: false,
            analysis_interval_minutes: DEFAULT_ANALYSIS_INTERVAL_MINUTES,
            last_analysis_tick: None,
            last_event: None,
            provider: AiProvider::OpenAi,
            model: "gpt-5.6-terra".to_string(),
            plan: CompiledWatchIntent::compile(DEFAULT_WATCH_INTENT.to_string()),
            custom_intent: false,
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
            local_matches_completed: self.local_matches_completed,
            suppressed_events: self.suppressed_events,
            analysis_interval_minutes: self.analysis_interval_minutes,
            model: self.model.clone(),
            custom_intent: self.custom_intent,
            evaluation_mode: self.plan.mode(),
            rule_summary: self.plan.rule_summary(),
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

    fn accept_event(&mut self, fingerprint: &str, tick: u64) -> bool {
        let duplicate = self.last_event.as_ref().is_some_and(|last| {
            last.value == fingerprint
                && tick.saturating_sub(last.tick)
                    < analysis_interval_ticks(self.analysis_interval_minutes)
        });
        if duplicate {
            self.suppressed_events = self.suppressed_events.saturating_add(1);
            return false;
        }
        self.last_event = Some(WatchEventFingerprint {
            value: fingerprint.to_string(),
            tick,
        });
        true
    }
}

fn semantic_fingerprint(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .take(80)
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalized_intent(intent: String) -> Result<(String, bool), SmartWatchError> {
    let intent = intent.trim();
    if intent.is_empty() {
        return Ok((DEFAULT_WATCH_INTENT.to_string(), false));
    }
    if intent.chars().count() > MAX_WATCH_INTENT_CHARS
        || intent.chars().any(|character| {
            let code = character as u32;
            (code < 32 && !matches!(code, 9 | 10 | 13)) || code == 127
        })
    {
        return Err(SmartWatchError::invalid_intent());
    }
    Ok((intent.to_string(), true))
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
    InvalidIntent,
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

    fn invalid_intent() -> Self {
        Self {
            code: SmartWatchErrorCode::InvalidIntent,
            message: "WATCH INTENT MUST BE BETWEEN 1 AND 500 SAFE CHARACTERS.",
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
        SetSmartWatchRequest, SmartWatchErrorCode, SmartWatchState,
        DEFAULT_ANALYSIS_INTERVAL_MINUTES, DEFAULT_WATCH_INTENT, SMART_WATCH_CONSENT_VERSION,
        STARTUP_NOTICE_BODY,
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
                    7,
                    watch_request(
                        0,
                        "Alert me when the build fails",
                        DEFAULT_ANALYSIS_INTERVAL_MINUTES,
                    ),
                )
                .unwrap_err()
                .code,
            SmartWatchErrorCode::ConsentRequired
        );
        assert!(
            state
                .configure(
                    7,
                    watch_request(
                        SMART_WATCH_CONSENT_VERSION,
                        "Alert me when the build fails",
                        DEFAULT_ANALYSIS_INTERVAL_MINUTES,
                    ),
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
                7,
                watch_request(
                    SMART_WATCH_CONSENT_VERSION,
                    "Alert me when the build fails",
                    DEFAULT_ANALYSIS_INTERVAL_MINUTES,
                ),
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
    fn watch_freezes_a_safe_intent_without_persisting_it_in_status() {
        let state = SmartWatchState::default();
        state
            .configure(
                4,
                watch_request(
                    SMART_WATCH_CONSENT_VERSION,
                    "  Alert me when the build fails  ",
                    5,
                ),
            )
            .unwrap();
        let context = state.begin_analysis(4, 1).expect("analysis context");
        assert_eq!(context.intent, "Alert me when the build fails");
        assert!(state.status().custom_intent);

        let default_state = SmartWatchState::default();
        default_state
            .configure(5, watch_request(SMART_WATCH_CONSENT_VERSION, "", 5))
            .unwrap();
        assert_eq!(
            default_state.begin_analysis(5, 1).unwrap().intent,
            DEFAULT_WATCH_INTENT
        );
        assert!(!default_state.status().custom_intent);
    }

    #[test]
    fn watch_rejects_unsafe_or_oversized_intents() {
        let state = SmartWatchState::default();
        for intent in ["unsafe\0intent".to_string(), "x".repeat(501)] {
            assert_eq!(
                state
                    .configure(4, watch_request(SMART_WATCH_CONSENT_VERSION, intent, 5),)
                    .unwrap_err()
                    .code,
                SmartWatchErrorCode::InvalidIntent
            );
        }
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

    #[test]
    fn duplicate_events_are_suppressed_within_the_selected_interval() {
        let state = enabled_watch(1);
        let (_, accepted) = state.finish_local_match(2, "local:error", 1);
        assert!(accepted);
        let (status, accepted) = state.finish_local_match(2, "local:error", 2);
        assert!(!accepted);
        assert_eq!(status.suppressed_events, 1);
        assert_eq!(status.local_matches_completed, 1);

        let (_, accepted) = state.finish_local_match(2, "local:error", 13);
        assert!(accepted);
    }

    #[test]
    fn semantic_dedupe_ignores_case_and_punctuation() {
        let state = enabled_watch(5);
        let (_, accepted) = state.accept_ai_event(2, "Build failed: lint.", 1);
        assert!(accepted);
        let (status, accepted) = state.accept_ai_event(2, "BUILD FAILED - LINT", 2);
        assert!(!accepted);
        assert_eq!(status.suppressed_events, 1);
    }

    fn enabled_watch(interval_minutes: u16) -> SmartWatchState {
        let state = SmartWatchState::default();
        state
            .configure(
                2,
                watch_request(
                    SMART_WATCH_CONSENT_VERSION,
                    "Alert me when the build fails",
                    interval_minutes,
                ),
            )
            .unwrap();
        state
    }

    fn watch_request(
        consent_version: u16,
        intent: impl Into<String>,
        analysis_interval_minutes: u16,
    ) -> SetSmartWatchRequest {
        SetSmartWatchRequest {
            enabled: true,
            consent_version,
            provider: AiProvider::OpenAi,
            model: "gpt-5.6-terra".into(),
            intent: intent.into(),
            locale: "ko-KR".into(),
            analysis_interval_minutes,
        }
    }
}
