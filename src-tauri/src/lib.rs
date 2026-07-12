mod activity_feed;
#[cfg(test)]
mod activity_feed_tests;
pub mod ai_handoff;
#[cfg(test)]
mod ai_handoff_tests;
mod ai_handoff_types;
mod ai_runtime;
mod app_status;
pub mod capture_backend;
#[cfg(test)]
mod capture_backend_tests;
pub mod capture_lifecycle;
#[cfg(test)]
mod capture_lifecycle_tests;
pub mod capture_scheduler;
pub mod diff_engine;
#[cfg(test)]
mod diff_engine_tests;
mod diff_engine_types;
pub mod live_tile;
#[cfg(test)]
mod live_tile_tests;
mod menu_bar;
mod monitoring;
pub mod ocr_engine;
#[cfg(test)]
mod ocr_engine_tests;
mod pebble_session;
#[cfg(test)]
mod pebble_session_tests;
pub mod pebble_store;
#[cfg(test)]
mod pebble_store_tests;
pub mod performance_limits;
#[cfg(test)]
mod performance_limits_tests;
pub mod platform_capture;
#[cfg(test)]
mod platform_capture_tests;
mod region_selection;
#[cfg(test)]
mod region_selection_tests;
pub mod region_selection_types;
mod region_selector_window;
mod smart_watch;
mod window_shell;
#[cfg(test)]
mod window_shell_tests;

use activity_feed::{ActivityFeedState, UpdateFeedSnapshot};
use ai_runtime::{AiAnswer, AiConnectionStatus, AiProvider, AiRuntimeError, AiRuntimeState};
use app_status::AppStatus;
use capture_backend::{capture_error, CaptureError, CaptureErrorCode};
use live_tile::{LiveTileCaptureRequest, LiveTileCaptureResponse, LiveTileState};
use ocr_engine::OcrStatus;
use pebble_session::{PebbleSessionError, PebbleSessionSnapshot, PebbleSessionState};
use pebble_store::{PebbleStore, PebbleStoreDocument, PebbleStoreError};
use performance_limits::{PerformanceLimitRequest, PerformanceLimits, PerformanceValidation};
use platform_capture::BackdropColor;
use region_selection_types::{RegionSelection, RegionSelectionIssue, RegionSelectionRequest};
use region_selector_window::RegionSelectorWindowShell;
use smart_watch::{SmartWatchError, SmartWatchState, SmartWatchStatus};
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use window_shell::WindowShellError;

#[tauri::command]
fn get_app_status() -> AppStatus {
    AppStatus::pre_alpha()
}

#[tauri::command]
fn get_performance_limits() -> PerformanceLimits {
    PerformanceLimits::default()
}

#[tauri::command]
fn validate_performance_request(request: PerformanceLimitRequest) -> PerformanceValidation {
    let limits = PerformanceLimits::default();

    limits.validate(request).into()
}

#[tauri::command]
fn resolve_region_selection(
    request: RegionSelectionRequest,
) -> Result<RegionSelection, RegionSelectionIssue> {
    region_selection::select_region(request)
}

#[tauri::command]
async fn open_region_selector_window(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
) -> Result<RegionSelectorWindowShell, WindowShellError> {
    if !is_pebble_window(window.label()) {
        return Err(WindowShellError::unavailable(
            "Region selection is available only from the Pebble window.",
        ));
    }
    region_selector_window::open_region_selector_window(&app, Some(&window))
}

#[tauri::command]
fn get_region_selector_monitor(
    window: tauri::WebviewWindow,
) -> Result<region_selection_types::MonitorGeometry, WindowShellError> {
    region_selector_window::region_selector_monitor_geometry(&window)
}

#[tauri::command]
fn close_region_selector_window(window: tauri::WebviewWindow) -> Result<(), WindowShellError> {
    region_selector_window::close_region_selector_window(&window)
}

#[tauri::command]
fn get_pebble_session(
    state: tauri::State<'_, PebbleSessionState>,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    state.snapshot()
}

#[tauri::command]
fn confirm_pebble_region(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    state: tauri::State<'_, PebbleSessionState>,
    smart_watch: tauri::State<'_, SmartWatchState>,
    request: RegionSelectionRequest,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    let snapshot = pebble_session::confirm_region_selection(&app, &window, state.inner(), request)?;
    smart_watch::emit_status(&app, smart_watch.disable());
    Ok(snapshot)
}

#[tauri::command]
fn show_pebble_window(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    state: tauri::State<'_, PebbleSessionState>,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    pebble_session::show_active_pebble_window(&app, &window, state.inner())
}

#[tauri::command]
fn set_pebble_privacy_blank(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    state: tauri::State<'_, PebbleSessionState>,
    smart_watch: tauri::State<'_, SmartWatchState>,
    active: bool,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    let snapshot = pebble_session::set_privacy_blank(&app, &window, state.inner(), active)?;
    if active {
        smart_watch::emit_status(&app, smart_watch.disable());
    }
    Ok(snapshot)
}

#[tauri::command]
fn remove_pebble(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    state: tauri::State<'_, PebbleSessionState>,
    smart_watch: tauri::State<'_, SmartWatchState>,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    let snapshot = pebble_session::remove_active_pebble(&app, &window, state.inner())?;
    smart_watch::emit_status(&app, smart_watch.disable());
    Ok(snapshot)
}

#[tauri::command]
fn close_pebble_window(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    state: tauri::State<'_, PebbleSessionState>,
    smart_watch: tauri::State<'_, SmartWatchState>,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    let snapshot = pebble_session::close_pebble_window(&app, &window, state.inner())?;
    smart_watch::emit_status(&app, smart_watch.disable());
    Ok(snapshot)
}

#[tauri::command]
fn set_pebble_ai_panel_expanded(
    window: tauri::WebviewWindow,
    expanded: bool,
) -> Result<(), PebbleSessionError> {
    pebble_session::set_ai_panel_expanded(&window, expanded)
}

#[tauri::command]
fn start_pebble_window_drag(window: tauri::WebviewWindow) -> Result<(), WindowShellError> {
    if !is_pebble_window(window.label()) {
        return Err(WindowShellError::unavailable(
            "Window dragging is available only from the Pebble window.",
        ));
    }
    window
        .start_dragging()
        .map_err(|error| WindowShellError::unavailable(error.to_string()))
}

#[tauri::command]
fn request_screen_capture_access(window: tauri::WebviewWindow) -> bool {
    is_pebble_window(window.label()) && platform_capture::request_screen_capture_access()
}

#[tauri::command]
fn get_pebble_backdrop_color(window: tauri::WebviewWindow) -> Option<BackdropColor> {
    let visible = window.is_visible().unwrap_or(false);
    let minimized = window.is_minimized().unwrap_or(true);
    backdrop_capture_is_allowed(window.label(), visible, minimized)
        .then(|| platform_capture::capture_window_backdrop_color(&window))
        .flatten()
}

fn backdrop_capture_is_allowed(label: &str, visible: bool, minimized: bool) -> bool {
    is_pebble_window(label) && visible && !minimized
}

#[tauri::command]
async fn get_ai_connection_status(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    state: tauri::State<'_, AiRuntimeState>,
    provider: AiProvider,
) -> Result<AiConnectionStatus, AiRuntimeError> {
    if !pebble_window_allows_ai(&window) {
        return Err(AiRuntimeError {
            code: ai_runtime::AiRuntimeErrorCode::UnauthorizedWindow,
            message: "AI is available only from the visible Pebble window.".to_string(),
            recoverable: true,
        });
    }
    ai_runtime::get_connection_status(&app, state.inner(), provider).await
}

#[tauri::command]
async fn connect_ai_provider(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    state: tauri::State<'_, AiRuntimeState>,
    provider: AiProvider,
) -> Result<AiConnectionStatus, AiRuntimeError> {
    if !pebble_window_allows_ai(&window) {
        return Err(AiRuntimeError {
            code: ai_runtime::AiRuntimeErrorCode::UnauthorizedWindow,
            message: "AI is available only from the visible Pebble window.".to_string(),
            recoverable: true,
        });
    }
    ai_runtime::connect_provider(&app, state.inner(), provider).await
}

#[tauri::command]
async fn ask_selected_region(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    runtime: tauri::State<'_, AiRuntimeState>,
    session: tauri::State<'_, PebbleSessionState>,
    provider: AiProvider,
    question: String,
    locale: String,
) -> Result<AiAnswer, AiRuntimeError> {
    ai_runtime::ask_selected_region(
        &app,
        &window,
        runtime.inner(),
        session.inner(),
        provider,
        question,
        locale,
    )
    .await
}

#[tauri::command]
fn get_update_feed(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, ActivityFeedState>,
) -> Result<UpdateFeedSnapshot, WindowShellError> {
    if !is_pebble_window(window.label()) {
        return Err(WindowShellError::unavailable(
            "Updates are available only from the Pebble window.",
        ));
    }
    Ok(activity_feed::snapshot(state.inner()))
}

#[tauri::command]
fn get_smart_watch_status(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, SmartWatchState>,
) -> Result<SmartWatchStatus, SmartWatchError> {
    if !pebble_window_allows_ai(&window) {
        return Err(SmartWatchError::unavailable());
    }
    Ok(state.status())
}

#[tauri::command]
fn set_smart_watch(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    session: tauri::State<'_, PebbleSessionState>,
    state: tauri::State<'_, SmartWatchState>,
    enabled: bool,
    consent_version: u16,
) -> Result<SmartWatchStatus, SmartWatchError> {
    if !pebble_window_allows_ai(&window) {
        return Err(SmartWatchError::unavailable());
    }
    let snapshot = session
        .snapshot()
        .map_err(|_| SmartWatchError::invalid_session())?;
    if enabled
        && (snapshot.region.is_none() || !snapshot.window_open || snapshot.privacy_blank_active)
    {
        return Err(SmartWatchError::invalid_session());
    }

    let status = state.configure(enabled, snapshot.revision, consent_version)?;
    smart_watch::emit_status(&app, status.clone());
    Ok(status)
}

#[tauri::command]
fn capture_live_tile_once(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    state: tauri::State<'_, LiveTileState>,
    session: tauri::State<'_, PebbleSessionState>,
    monitoring: tauri::State<'_, monitoring::MonitoringState>,
    smart_watch: tauri::State<'_, SmartWatchState>,
    request: LiveTileCaptureRequest,
) -> Result<LiveTileCaptureResponse, CaptureError> {
    if !is_live_tile_window(window.label()) {
        return Err(capture_error(
            CaptureErrorCode::UnauthorizedWindow,
            window.label(),
            "Live capture is available only from the visible Pebble window.",
        ));
    }

    if !capture_window_allows_delivery(
        window.is_visible().unwrap_or(false),
        window.is_minimized().unwrap_or(true),
        true,
    ) {
        return Err(capture_error(
            CaptureErrorCode::UnauthorizedWindow,
            window.label(),
            "Live capture stops while the Pebble window is hidden.",
        ));
    }

    let monitors = current_monitor_geometries(&app)?;
    let authorized = session.authorize_capture(request, &monitors)?;
    let expected_revision = authorized.session_revision();
    let outcome = state.capture_once(authorized)?;

    if let Some(event) = &outcome.frame_event {
        let current = current_monitor_geometries(&app)
            .ok()
            .and_then(|monitors| {
                session
                    .frame_delivery_is_current(expected_revision, &monitors)
                    .ok()
            })
            .unwrap_or(false);
        let visible = window.is_visible().unwrap_or(false);
        let minimized = window.is_minimized().unwrap_or(true);

        if !capture_window_allows_delivery(visible, minimized, current) {
            state.discard_frame(live_tile::MAIN_LIVE_TILE_ID);
            return Err(capture_error(
                CaptureErrorCode::CaptureUnavailable,
                live_tile::MAIN_LIVE_TILE_ID,
                "Captured frame was discarded because the Pebble session changed.",
            ));
        }

        let _ = app.emit_to(
            pebble_session::PEBBLE_TILE_LABEL,
            live_tile::live_tile_frame_event_name(),
            event,
        );

        if let Some(decision) = monitoring.observe(expected_revision, &event.frame, event.sequence)
        {
            match decision {
                monitoring::MonitoringDecision::Baseline => {
                    emit_local_monitoring_insight(&app, decision);
                }
                monitoring::MonitoringDecision::Changed { .. } => {
                    if let Some(status) = smart_watch.claim_notification(expected_revision) {
                        smart_watch::emit_status(&app, status);
                        emit_local_monitoring_insight(&app, decision);
                    } else {
                        smart_watch::emit_status(&app, smart_watch.status());
                    }
                }
            }
        }
    }

    Ok(outcome.response)
}

fn emit_local_monitoring_insight(app: &tauri::AppHandle, decision: monitoring::MonitoringDecision) {
    let summary = match decision {
        monitoring::MonitoringDecision::Baseline => return,
        monitoring::MonitoringDecision::Changed { kind, .. } => {
            let summary = kind.summary();
            menu_bar::set_attention(app, true);
            let _ = app
                .notification()
                .builder()
                .title("PEBBLE WATCH")
                .body(summary)
                .show();
            summary
        }
    };
    activity_feed::record_watch(app, app.state::<ActivityFeedState>().inner(), summary);
}

fn current_monitor_geometries(
    app: &tauri::AppHandle,
) -> Result<Vec<region_selection_types::MonitorGeometry>, CaptureError> {
    region_selector_window::available_monitor_geometries(app).map_err(|error| {
        capture_error(
            CaptureErrorCode::MonitorUnavailable,
            live_tile::MAIN_LIVE_TILE_ID,
            error.message,
        )
    })
}

fn capture_window_allows_delivery(visible: bool, minimized: bool, session_current: bool) -> bool {
    visible && !minimized && session_current
}

fn is_live_tile_window(label: &str) -> bool {
    label == pebble_session::PEBBLE_TILE_LABEL
}

fn is_pebble_window(label: &str) -> bool {
    label == pebble_session::PEBBLE_TILE_LABEL
}

fn pebble_window_allows_ai(window: &tauri::WebviewWindow) -> bool {
    is_pebble_window(window.label())
        && window.is_visible().unwrap_or(false)
        && !window.is_minimized().unwrap_or(true)
}

#[tauri::command]
fn load_pebble_config(app: tauri::AppHandle) -> Result<PebbleStoreDocument, PebbleStoreError> {
    default_pebble_store(&app)?.load_or_default()
}

#[tauri::command]
fn save_pebble_config(
    app: tauri::AppHandle,
    document: PebbleStoreDocument,
) -> Result<PebbleStoreDocument, PebbleStoreError> {
    default_pebble_store(&app)?.save(&document)
}

#[tauri::command]
fn get_ocr_status() -> OcrStatus {
    ocr_engine::local_ocr_status()
}

fn default_pebble_store(app: &tauri::AppHandle) -> Result<PebbleStore, PebbleStoreError> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|_| PebbleStoreError::config_path_unavailable())?;

    Ok(PebbleStore::new(PebbleStore::path_for_config_dir(
        config_dir,
    )))
}

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AiRuntimeState::default())
        .manage(ActivityFeedState::default())
        .manage(LiveTileState::default())
        .manage(monitoring::MonitoringState::default())
        .manage(SmartWatchState::default())
        .manage(PebbleSessionState::default())
        .setup(|app| {
            menu_bar::setup(app)?;
            smart_watch::show_startup_notice(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            get_performance_limits,
            validate_performance_request,
            resolve_region_selection,
            open_region_selector_window,
            get_region_selector_monitor,
            close_region_selector_window,
            get_pebble_session,
            confirm_pebble_region,
            show_pebble_window,
            set_pebble_privacy_blank,
            remove_pebble,
            close_pebble_window,
            set_pebble_ai_panel_expanded,
            start_pebble_window_drag,
            request_screen_capture_access,
            get_pebble_backdrop_color,
            get_ai_connection_status,
            connect_ai_provider,
            ask_selected_region,
            get_update_feed,
            get_smart_watch_status,
            set_smart_watch,
            capture_live_tile_once,
            load_pebble_config,
            save_pebble_config,
            get_ocr_status
        ])
        .run(tauri::generate_context!())
}

#[cfg(test)]
mod tests {
    use super::{
        get_app_status, get_performance_limits,
        performance_limits::{PerformanceLimitErrorCode, PerformanceLimitRequest, RegionSize},
        region_selection_types::{
            LogicalPoint, LogicalSize, MonitorGeometry, PhysicalPoint, RegionSelectionRequest,
        },
        resolve_region_selection, validate_performance_request,
    };

    #[test]
    fn app_status_reports_platform_capture_and_explicit_ai_availability() {
        let status = get_app_status();

        assert_eq!(status.phase, "pre-alpha");
        assert!(status.scaffold_ready);
        assert_eq!(status.capture_enabled, cfg!(target_os = "macos"));
        assert_eq!(status.ai_enabled, cfg!(target_os = "macos"));
    }

    #[test]
    fn live_capture_is_restricted_to_the_visible_pebble_window() {
        assert!(super::is_live_tile_window(
            super::pebble_session::PEBBLE_TILE_LABEL
        ));
        assert!(!super::is_live_tile_window("main"));
        assert!(!super::is_live_tile_window(
            super::region_selector_window::REGION_SELECTOR_LABEL
        ));
    }

    #[test]
    fn hidden_minimized_or_stale_windows_cannot_deliver_frames() {
        assert!(super::capture_window_allows_delivery(true, false, true));
        assert!(!super::capture_window_allows_delivery(false, false, true));
        assert!(!super::capture_window_allows_delivery(true, true, true));
        assert!(!super::capture_window_allows_delivery(true, false, false));
    }

    #[test]
    fn backdrop_color_is_available_only_to_the_visible_pebble_window() {
        assert!(super::backdrop_capture_is_allowed(
            super::pebble_session::PEBBLE_TILE_LABEL,
            true,
            false
        ));
        assert!(!super::backdrop_capture_is_allowed("main", true, false));
        assert!(!super::backdrop_capture_is_allowed(
            super::pebble_session::PEBBLE_TILE_LABEL,
            false,
            false
        ));
        assert!(!super::backdrop_capture_is_allowed(
            super::pebble_session::PEBBLE_TILE_LABEL,
            true,
            true
        ));
    }

    #[test]
    fn webviews_cannot_emit_authoritative_backend_events() {
        let capability = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/capabilities/default.json"
        ));

        assert!(capability.contains("core:event:allow-listen"));
        assert!(capability.contains("core:event:allow-unlisten"));
        assert!(!capability.contains("core:event:allow-emit"));
        assert!(!capability.contains("core:event:allow-emit-to"));
        assert!(!capability.contains("core:default"));
    }

    #[test]
    fn performance_limits_command_returns_hard_limits() {
        let limits = get_performance_limits();

        assert_eq!(limits.default_fps, 1);
        assert_eq!(limits.max_fps, 5);
        assert_eq!(limits.max_active_tiles, 3);
        assert_eq!(limits.max_region.width, i32::MAX);
        assert_eq!(limits.max_region.height, i32::MAX);
    }

    #[test]
    fn validate_performance_request_returns_typed_errors() {
        let result = validate_performance_request(PerformanceLimitRequest {
            fps: 6,
            active_tile_count: 1,
            region: RegionSize {
                width: 600,
                height: 300,
            },
        });

        assert!(!result.valid);
        assert_eq!(
            result.error.expect("validation error").code,
            PerformanceLimitErrorCode::FpsTooHigh
        );
    }

    #[test]
    fn resolve_region_selection_returns_physical_region() {
        let selection = resolve_region_selection(RegionSelectionRequest {
            monitor: MonitorGeometry {
                id: "main".to_string(),
                logical_origin: LogicalPoint { x: 0.0, y: 0.0 },
                logical_size: LogicalSize {
                    width: 1_920.0,
                    height: 1_080.0,
                },
                physical_origin: PhysicalPoint { x: 0, y: 0 },
                scale_factor: 1.0,
            },
            start: LogicalPoint { x: 10.0, y: 20.0 },
            end: LogicalPoint { x: 210.0, y: 170.0 },
        })
        .expect("region selection");

        assert_eq!(selection.region.width, 200);
        assert_eq!(selection.region.height, 150);
        assert!(selection.warnings.is_empty());
    }
}
