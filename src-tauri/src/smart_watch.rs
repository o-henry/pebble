use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;

use crate::{
    ai_runtime::AiProvider,
    watch_intent::{CompiledWatchIntent, WatchEvaluationMode},
};

#[path = "watch_target_registry.rs"]
mod registry;
pub(crate) use registry::{WatchAuthorization, WatchCaptureTarget, WatchRegionAuthorization};
use registry::{WatchTargetConfig, WatchTargetRegistry};

pub const SMART_WATCH_CONSENT_VERSION: u16 = 7;
pub const WATCH_CAPTURE_INTERVAL_SECONDS: u64 = 5;
pub const DEFAULT_ANALYSIS_INTERVAL_MINUTES: u16 = 5;
pub const ANALYSIS_INTERVAL_OPTIONS_MINUTES: [u16; 4] = [1, 5, 30, 60];
pub const SMART_WATCH_STATUS_EVENT: &str = "pebble://smart-watch-status";
pub const STARTUP_NOTICE_TITLE: &str = "PEBBLE WATCH";
pub const STARTUP_NOTICE_BODY: &str =
    "WHEN ENABLED, WATCH CHECKS ONLY YOUR EXPLICITLY SELECTED REGIONS (UP TO 3) EVERY 5S, INCLUDING WHILE THE WINDOW IS HIDDEN. APPLE VISION OCR AND AI RUN ONLY AFTER A MATERIAL CHANGE AND NO MORE OFTEN THAN EACH REGION'S SELECTED INTERVAL.";
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
    pub target_count: u8,
    pub targets: Vec<SmartWatchTargetStatus>,
    pub analyses_completed: u32,
    pub local_matches_completed: u32,
    pub suppressed_events: u32,
    pub analysis_interval_minutes: u16,
    pub provider: AiProvider,
    pub model: String,
    pub custom_intent: bool,
    pub watching_for: Option<String>,
    pub evaluation_mode: WatchEvaluationMode,
    pub rule_summary: String,
    pub capture_scope: &'static str,
    pub storage_policy: &'static str,
    pub images_saved: bool,
    pub ocr_saved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartWatchTargetStatus {
    pub id: String,
    pub name: String,
    pub current: bool,
    pub analyses_completed: u32,
    pub local_matches_completed: u32,
    pub suppressed_events: u32,
    pub analysis_interval_minutes: u16,
    pub provider: AiProvider,
    pub model: String,
    pub evaluation_mode: WatchEvaluationMode,
    pub rule_summary: String,
}

#[derive(Debug, Clone, Default)]
pub struct SmartWatchState {
    data: Arc<Mutex<WatchTargetRegistry>>,
}

#[derive(Debug, Clone)]
pub struct WatchAnalysisContext {
    pub target_name: String,
    pub provider: AiProvider,
    pub model: String,
    pub intent: String,
    pub locale: String,
    pub plan: CompiledWatchIntent,
    pub authorization: WatchAuthorization,
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
        authorization: WatchRegionAuthorization,
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

        let config = WatchTargetConfig {
            provider,
            model,
            plan,
            custom_intent,
            locale: normalized_locale(locale),
            analysis_interval_minutes,
        };
        let mut data = self
            .data
            .lock()
            .map_err(|_| SmartWatchError::unavailable())?;
        if enabled {
            data.upsert(authorization, config)
                .map_err(|_| SmartWatchError::target_limit_reached())?;
        } else {
            data.select_current(authorization.revision);
            data.remove_current();
        }
        Ok(data.status())
    }

    pub fn select_current(&self, revision: u64) -> SmartWatchStatus {
        let mut data = self.data.lock().expect("smart watch state lock");
        data.select_current(revision);
        data.status()
    }

    pub fn disable(&self) -> SmartWatchStatus {
        let mut data = self.data.lock().expect("smart watch state lock");
        data.remove_all();
        data.status()
    }

    pub fn remove_target(&self, id: &str) -> Result<SmartWatchStatus, SmartWatchError> {
        if !valid_target_id(id) {
            return Err(SmartWatchError::invalid_target());
        }
        let mut data = self
            .data
            .lock()
            .map_err(|_| SmartWatchError::unavailable())?;
        if !data.remove_target(id) {
            return Err(SmartWatchError::invalid_target());
        }
        Ok(data.status())
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
        data.set_current_interval(analysis_interval_minutes);
        Ok(data.status())
    }

    pub fn capture_targets(&self) -> Vec<WatchCaptureTarget> {
        self.data
            .lock()
            .map(|data| data.capture_targets())
            .unwrap_or_default()
    }

    pub fn contains_target(&self, id: &str) -> bool {
        self.data.lock().is_ok_and(|data| data.contains(id))
    }

    pub fn begin_analysis(&self, id: &str, tick: u64) -> Option<WatchAnalysisContext> {
        let mut data = self.data.lock().ok()?;
        data.begin_analysis(id, tick)
    }

    pub fn current_context(&self, id: &str) -> Option<WatchAnalysisContext> {
        let data = self.data.lock().ok()?;
        data.current_context(id)
    }

    pub fn finish_local_match(
        &self,
        id: &str,
        fingerprint: &str,
        tick: u64,
    ) -> (SmartWatchStatus, bool) {
        let mut data = self.data.lock().expect("smart watch state lock");
        let accepted = data.finish_local_match(id, fingerprint, tick);
        (data.status(), accepted)
    }

    pub fn finish_analysis(&self, id: &str, completed: bool) -> (SmartWatchStatus, bool) {
        let mut data = self.data.lock().expect("smart watch state lock");
        let accepted = data.finish_analysis(id, completed);
        (data.status(), accepted)
    }

    pub fn accept_ai_event(&self, id: &str, summary: &str, tick: u64) -> (SmartWatchStatus, bool) {
        let mut data = self.data.lock().expect("smart watch state lock");
        let accepted = data.accept_ai_event(id, summary, tick);
        (data.status(), accepted)
    }
}

impl Default for WatchTargetRegistry {
    fn default() -> Self {
        Self::new(WatchTargetConfig {
            analysis_interval_minutes: DEFAULT_ANALYSIS_INTERVAL_MINUTES,
            provider: AiProvider::OpenAi,
            model: "gpt-5.6-terra".to_string(),
            plan: CompiledWatchIntent::compile(DEFAULT_WATCH_INTENT.to_string()),
            custom_intent: false,
            locale: "und".to_string(),
        })
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

fn valid_target_id(value: &str) -> bool {
    value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
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
    InvalidTarget,
    TargetLimitReached,
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

    fn invalid_target() -> Self {
        Self {
            code: SmartWatchErrorCode::InvalidTarget,
            message: "THE WATCH REGION IS NO LONGER ACTIVE.",
        }
    }

    fn target_limit_reached() -> Self {
        Self {
            code: SmartWatchErrorCode::TargetLimitReached,
            message: "WATCH SUPPORTS UP TO 3 ACTIVE REGIONS. STOP ONE BEFORE ADDING ANOTHER.",
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
        SetSmartWatchRequest, SmartWatchErrorCode, SmartWatchState, WatchRegionAuthorization,
        DEFAULT_ANALYSIS_INTERVAL_MINUTES, DEFAULT_WATCH_INTENT, SMART_WATCH_CONSENT_VERSION,
        STARTUP_NOTICE_BODY,
    };
    use crate::{
        ai_runtime::AiProvider,
        region_selection_types::{PhysicalRegion, WindowCaptureTarget},
    };

    #[test]
    fn startup_notice_explains_activation_and_local_privacy() {
        assert!(STARTUP_NOTICE_BODY.contains("EVERY 5S"));
        assert!(STARTUP_NOTICE_BODY.contains("EXPLICITLY SELECTED REGIONS"));
        assert!(STARTUP_NOTICE_BODY.contains("UP TO 3"));
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
                    authorization(7, 0),
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
                    authorization(7, 0),
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
    fn selecting_another_region_keeps_the_existing_target_running() {
        let state = SmartWatchState::default();
        state
            .configure(
                authorization(7, 0),
                watch_request(
                    SMART_WATCH_CONSENT_VERSION,
                    "Alert me when the build fails",
                    DEFAULT_ANALYSIS_INTERVAL_MINUTES,
                ),
            )
            .unwrap();
        let id = current_target_id(&state);

        let status = state.select_current(8);
        assert!(!status.enabled);
        assert_eq!(status.target_count, 1);
        assert!(state.begin_analysis(&id, 1).is_some());
    }

    #[test]
    fn watch_accepts_three_independent_regions_and_rejects_a_fourth() {
        let state = SmartWatchState::default();
        for revision in 1..=3 {
            state
                .configure(
                    authorization(revision, revision as i32 * 10),
                    watch_request(SMART_WATCH_CONSENT_VERSION, "ERROR appears", 5),
                )
                .unwrap();
        }
        assert_eq!(state.status().target_count, 3);
        assert_eq!(state.capture_targets().len(), 3);
        assert_eq!(
            state
                .configure(
                    authorization(4, 40),
                    watch_request(SMART_WATCH_CONSENT_VERSION, "READY appears", 5),
                )
                .unwrap_err()
                .code,
            SmartWatchErrorCode::TargetLimitReached
        );
    }

    #[test]
    fn selecting_the_same_bound_region_updates_instead_of_duplicating() {
        let state = SmartWatchState::default();
        state
            .configure(
                authorization(1, 10),
                watch_request(SMART_WATCH_CONSENT_VERSION, "ERROR appears", 5),
            )
            .unwrap();
        state
            .configure(
                authorization(2, 10),
                watch_request(SMART_WATCH_CONSENT_VERSION, "READY appears", 30),
            )
            .unwrap();
        let status = state.status();
        assert_eq!(status.target_count, 1);
        assert_eq!(status.rule_summary, "TEXT APPEARS: ready");
    }

    #[test]
    fn watch_interval_accepts_only_supported_values() {
        let state = SmartWatchState::default();
        assert_eq!(state.status().analysis_interval_minutes, 5);
        state
            .configure(
                authorization(1, 0),
                watch_request(SMART_WATCH_CONSENT_VERSION, "ERROR appears", 5),
            )
            .unwrap();
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
    fn watch_exposes_the_active_plan_without_persisting_frames_or_ocr() {
        let state = SmartWatchState::default();
        state
            .configure(
                authorization(4, 0),
                watch_request(
                    SMART_WATCH_CONSENT_VERSION,
                    "  Alert me when the build fails  ",
                    5,
                ),
            )
            .unwrap();
        let id = current_target_id(&state);
        let context = state.begin_analysis(&id, 1).expect("analysis context");
        assert_eq!(context.intent, "Alert me when the build fails");
        let status = state.status();
        assert!(status.custom_intent);
        assert_eq!(
            status.watching_for.as_deref(),
            Some("Alert me when the build fails")
        );
        assert_eq!(status.capture_scope, "selectedRegionOnly");
        assert_eq!(status.storage_policy, "memoryOnly");
        assert!(!status.images_saved);
        assert!(!status.ocr_saved);

        let default_state = SmartWatchState::default();
        default_state
            .configure(
                authorization(5, 0),
                watch_request(SMART_WATCH_CONSENT_VERSION, "", 5),
            )
            .unwrap();
        let default_id = current_target_id(&default_state);
        assert_eq!(
            default_state.begin_analysis(&default_id, 1).unwrap().intent,
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
                    .configure(
                        authorization(4, 0),
                        watch_request(SMART_WATCH_CONSENT_VERSION, intent, 5),
                    )
                    .unwrap_err()
                    .code,
                SmartWatchErrorCode::InvalidIntent
            );
        }
    }

    #[test]
    fn selected_interval_limits_ai_analysis_without_stopping_local_watch() {
        let state = enabled_watch(5);
        let id = current_target_id(&state);

        assert!(state.begin_analysis(&id, 1).is_some());
        state.finish_analysis(&id, true);
        assert!(state.begin_analysis(&id, 60).is_none());
        assert!(state.status().enabled);
        assert!(state.begin_analysis(&id, 61).is_some());
    }

    #[test]
    fn changing_interval_applies_to_the_next_analysis() {
        let state = enabled_watch(30);
        let id = current_target_id(&state);

        assert!(state.begin_analysis(&id, 1).is_some());
        state.finish_analysis(&id, true);
        state.set_analysis_interval(1).unwrap();
        assert!(state.begin_analysis(&id, 13).is_some());
    }

    #[test]
    fn watch_has_no_fixed_session_analysis_cap() {
        let state = enabled_watch(1);
        let id = current_target_id(&state);

        for analysis in 0..20_u64 {
            assert!(state.begin_analysis(&id, analysis * 12).is_some());
            state.finish_analysis(&id, true);
        }
        assert_eq!(state.status().analyses_completed, 20);
        assert!(state.status().enabled);
    }

    #[test]
    fn failed_or_cancelled_analysis_does_not_count_or_emit_late_results() {
        let state = enabled_watch(1);
        let id = current_target_id(&state);

        assert!(state.begin_analysis(&id, 1).is_some());
        assert!(state.begin_analysis(&id, 2).is_none());
        let (status, accepted) = state.finish_analysis(&id, false);
        assert!(accepted);
        assert_eq!(status.analyses_completed, 0);

        let context = state.begin_analysis(&id, 13).expect("analysis context");
        assert!(context.authorization.is_active());
        state.disable();
        assert!(!context.authorization.is_active());
        let (_, accepted) = state.finish_analysis(&id, true);
        assert!(!accepted);
    }

    #[test]
    fn disable_stops_notifications_immediately() {
        let state = enabled_watch(5);
        let id = current_target_id(&state);
        let status = state.disable();

        assert!(state.begin_analysis(&id, 1).is_none());
        assert_eq!(status.watching_for, None);
        assert!(!status.custom_intent);
    }

    #[test]
    fn duplicate_events_are_suppressed_within_the_selected_interval() {
        let state = enabled_watch(1);
        let id = current_target_id(&state);
        let (_, accepted) = state.finish_local_match(&id, "local:error", 1);
        assert!(accepted);
        let (status, accepted) = state.finish_local_match(&id, "local:error", 2);
        assert!(!accepted);
        assert_eq!(status.suppressed_events, 1);
        assert_eq!(status.local_matches_completed, 1);

        let (_, accepted) = state.finish_local_match(&id, "local:error", 13);
        assert!(accepted);
    }

    #[test]
    fn semantic_dedupe_ignores_case_and_punctuation() {
        let state = enabled_watch(5);
        let id = current_target_id(&state);
        let (_, accepted) = state.accept_ai_event(&id, "Build failed: lint.", 1);
        assert!(accepted);
        let (status, accepted) = state.accept_ai_event(&id, "BUILD FAILED - LINT", 2);
        assert!(!accepted);
        assert_eq!(status.suppressed_events, 1);
    }

    #[test]
    fn individual_target_removal_preserves_other_regions() {
        let state = enabled_watch(5);
        let first = current_target_id(&state);
        let first_context = state.begin_analysis(&first, 1).expect("first context");
        state
            .configure(
                authorization(3, 30),
                watch_request(SMART_WATCH_CONSENT_VERSION, "READY appears", 5),
            )
            .unwrap();
        let status = state.remove_target(&first).unwrap();
        assert_eq!(status.target_count, 1);
        assert_eq!(status.targets[0].name, "REGION 2");
        assert!(!state.contains_target(&first));
        assert!(!first_context.authorization.is_active());
    }

    #[test]
    fn serialized_status_never_exposes_capture_coordinates_or_window_ids() {
        let state = enabled_watch(5);
        let raw = serde_json::to_string(&state.status()).expect("status json");
        for forbidden in [
            "sourceWindow",
            "windowId",
            "scaleFactor",
            "monitorId",
            "relativeX",
        ] {
            assert!(!raw.contains(forbidden));
        }
    }

    fn enabled_watch(interval_minutes: u16) -> SmartWatchState {
        let state = SmartWatchState::default();
        state
            .configure(
                authorization(2, 0),
                watch_request(
                    SMART_WATCH_CONSENT_VERSION,
                    "Alert me when the build fails",
                    interval_minutes,
                ),
            )
            .unwrap();
        state
    }

    fn current_target_id(state: &SmartWatchState) -> String {
        state
            .status()
            .targets
            .into_iter()
            .find(|target| target.current)
            .expect("current target")
            .id
    }

    fn authorization(revision: u64, x: i32) -> WatchRegionAuthorization {
        WatchRegionAuthorization {
            revision,
            region: PhysicalRegion {
                monitor_id: "main".into(),
                x,
                y: 0,
                width: 320,
                height: 200,
                source_window: Some(WindowCaptureTarget {
                    window_id: 1,
                    relative_x_millipoints: i64::from(x) * 1_000,
                    relative_y_millipoints: 0,
                    width_millipoints: 320_000,
                    height_millipoints: 200_000,
                }),
            },
            scale_factor: 2.0,
        }
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
