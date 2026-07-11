use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    WindowEvent,
};

use crate::{
    capture_backend::{capture_error, CaptureError, CaptureErrorCode},
    capture_lifecycle::CaptureTileMode,
    live_tile::{
        AuthorizedLiveTileCapture, LiveTileCaptureRequest, LiveTileState, MAIN_LIVE_TILE_ID,
    },
    region_selection,
    region_selection_types::{MonitorGeometry, PhysicalRegion, RegionSelectionRequest},
    region_selector_window::{
        monitor_identifier, region_selector_monitor_geometry, REGION_SELECTOR_LABEL,
    },
    window_shell::show_existing_window,
};

pub const PEBBLE_TILE_LABEL: &str = "pebble-tile";
pub const PEBBLE_SESSION_UPDATED_EVENT: &str = "pebble://session-updated";

const PEBBLE_WINDOW_WIDTH: f64 = 440.0;
const PEBBLE_WINDOW_HEIGHT: f64 = 340.0;
const PEBBLE_WINDOW_AI_HEIGHT: f64 = 540.0;
const PEBBLE_WINDOW_MIN_WIDTH: f64 = 300.0;
const PEBBLE_WINDOW_MIN_HEIGHT: f64 = 240.0;
const PEBBLE_WINDOW_OUTER_HEIGHT: f64 = 380.0;
const PEBBLE_WINDOW_MARGIN: f64 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PebbleWindowPosition {
    pub logical_x: f64,
    pub logical_y: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PebbleSessionSnapshot {
    pub region: Option<PhysicalRegion>,
    pub window_open: bool,
    pub privacy_blank_active: bool,
    pub revision: u64,
}

#[derive(Debug, Clone, Default)]
pub struct PebbleSessionState {
    data: Arc<Mutex<PebbleSessionData>>,
}

#[derive(Debug, Clone, Default)]
struct PebbleSessionData {
    snapshot: PebbleSessionSnapshot,
    capture_scale_factor: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct AuthorizedAiCapture {
    region: PhysicalRegion,
    scale_factor: f64,
    session_revision: u64,
}

impl AuthorizedAiCapture {
    pub(crate) fn region(&self) -> &PhysicalRegion {
        &self.region
    }

    pub(crate) fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub(crate) fn session_revision(&self) -> u64 {
        self.session_revision
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PebbleSessionError {
    pub code: PebbleSessionErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PebbleSessionErrorCode {
    InvalidRegion,
    NoActivePebble,
    UnauthorizedWindow,
    WindowUnavailable,
    StateUnavailable,
}

impl PebbleSessionState {
    pub fn snapshot(&self) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
        self.data
            .lock()
            .map(|data| data.snapshot.clone())
            .map_err(|_| PebbleSessionError::state_unavailable())
    }

    pub fn select_region(
        &self,
        request: RegionSelectionRequest,
    ) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
        let scale_factor = request.monitor.scale_factor;
        let selection = region_selection::select_region(request).map_err(|issue| {
            PebbleSessionError::new(PebbleSessionErrorCode::InvalidRegion, issue.message)
        })?;
        let mut data = self
            .data
            .lock()
            .map_err(|_| PebbleSessionError::state_unavailable())?;

        data.capture_scale_factor = Some(scale_factor);
        data.snapshot.region = Some(selection.region);
        data.snapshot.privacy_blank_active = false;
        increment_revision(&mut data.snapshot);
        Ok(data.snapshot.clone())
    }

    pub fn set_window_open(
        &self,
        window_open: bool,
    ) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| PebbleSessionError::state_unavailable())?;

        if data.snapshot.window_open != window_open {
            data.snapshot.window_open = window_open;
            increment_revision(&mut data.snapshot);
        }
        Ok(data.snapshot.clone())
    }

    pub fn set_privacy_blank(
        &self,
        active: bool,
    ) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| PebbleSessionError::state_unavailable())?;

        if data.snapshot.privacy_blank_active != active {
            data.snapshot.privacy_blank_active = active;
            increment_revision(&mut data.snapshot);
        }
        Ok(data.snapshot.clone())
    }

    pub fn clear(&self) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| PebbleSessionError::state_unavailable())?;

        let revision = data.snapshot.revision.saturating_add(1);
        data.snapshot = PebbleSessionSnapshot {
            revision,
            ..PebbleSessionSnapshot::default()
        };
        data.capture_scale_factor = None;
        Ok(data.snapshot.clone())
    }

    pub fn authorize_capture(
        &self,
        request: LiveTileCaptureRequest,
        monitors: &[MonitorGeometry],
    ) -> Result<AuthorizedLiveTileCapture, CaptureError> {
        let data = self.data.lock().map_err(|_| {
            capture_error(
                CaptureErrorCode::CaptureUnavailable,
                MAIN_LIVE_TILE_ID,
                "Pebble session state is unavailable.",
            )
        })?;
        let region = data.snapshot.region.as_ref().ok_or_else(|| {
            capture_error(
                CaptureErrorCode::CaptureUnavailable,
                MAIN_LIVE_TILE_ID,
                "No selected region is active.",
            )
        })?;

        if !data.snapshot.window_open
            || request.tile_id != MAIN_LIVE_TILE_ID
            || request.region != *region
            || request.request_id.len() > 256
            || matches!(
                request.mode,
                CaptureTileMode::Hidden | CaptureTileMode::Closed | CaptureTileMode::Deleted
            )
        {
            return Err(capture_error(
                CaptureErrorCode::UnauthorizedWindow,
                MAIN_LIVE_TILE_ID,
                "Capture request does not match the active selected region.",
            ));
        }

        let scale_factor = data.capture_scale_factor.ok_or_else(|| {
            capture_error(
                CaptureErrorCode::CaptureUnavailable,
                MAIN_LIVE_TILE_ID,
                "Capture scale factor is unavailable.",
            )
        })?;
        validate_current_monitor(region, scale_factor, monitors)?;
        let sanitized = LiveTileCaptureRequest {
            request_id: request.request_id,
            blank_generation: request.blank_generation,
            tile_id: MAIN_LIVE_TILE_ID.to_string(),
            region: region.clone(),
            fps: request.fps,
            mode: if data.snapshot.privacy_blank_active {
                CaptureTileMode::Blanked
            } else {
                request.mode
            },
        };

        Ok(AuthorizedLiveTileCapture::new(
            sanitized,
            scale_factor,
            data.snapshot.revision,
        ))
    }

    pub fn frame_delivery_is_current(
        &self,
        expected_revision: u64,
        monitors: &[MonitorGeometry],
    ) -> Result<bool, PebbleSessionError> {
        let data = self
            .data
            .lock()
            .map_err(|_| PebbleSessionError::state_unavailable())?;

        if !frame_delivery_is_current(&data.snapshot, expected_revision) {
            return Ok(false);
        }

        let Some(region) = data.snapshot.region.as_ref() else {
            return Ok(false);
        };
        let Some(scale_factor) = data.capture_scale_factor else {
            return Ok(false);
        };

        Ok(validate_current_monitor(region, scale_factor, monitors).is_ok())
    }

    pub fn authorize_ai_capture(
        &self,
        monitors: &[MonitorGeometry],
    ) -> Result<AuthorizedAiCapture, CaptureError> {
        let data = self.data.lock().map_err(|_| {
            capture_error(
                CaptureErrorCode::CaptureUnavailable,
                MAIN_LIVE_TILE_ID,
                "Pebble session state is unavailable.",
            )
        })?;
        let region = data.snapshot.region.as_ref().ok_or_else(|| {
            capture_error(
                CaptureErrorCode::CaptureUnavailable,
                MAIN_LIVE_TILE_ID,
                "No selected region is active.",
            )
        })?;
        if data.snapshot.privacy_blank_active {
            return Err(capture_error(
                CaptureErrorCode::UnauthorizedWindow,
                MAIN_LIVE_TILE_ID,
                "Image questions stop while the selected region is hidden.",
            ));
        }

        let scale_factor = data.capture_scale_factor.ok_or_else(|| {
            capture_error(
                CaptureErrorCode::CaptureUnavailable,
                MAIN_LIVE_TILE_ID,
                "Capture scale factor is unavailable.",
            )
        })?;
        validate_current_monitor(region, scale_factor, monitors)?;

        Ok(AuthorizedAiCapture {
            region: region.clone(),
            scale_factor,
            session_revision: data.snapshot.revision,
        })
    }

    pub fn ai_capture_is_current(
        &self,
        expected_revision: u64,
        monitors: &[MonitorGeometry],
    ) -> Result<bool, PebbleSessionError> {
        let data = self
            .data
            .lock()
            .map_err(|_| PebbleSessionError::state_unavailable())?;
        if data.snapshot.revision != expected_revision || data.snapshot.privacy_blank_active {
            return Ok(false);
        }

        let Some(region) = data.snapshot.region.as_ref() else {
            return Ok(false);
        };
        let Some(scale_factor) = data.capture_scale_factor else {
            return Ok(false);
        };

        Ok(validate_current_monitor(region, scale_factor, monitors).is_ok())
    }
}

fn increment_revision(snapshot: &mut PebbleSessionSnapshot) {
    snapshot.revision = snapshot.revision.saturating_add(1);
}

pub(crate) fn validate_current_monitor(
    region: &PhysicalRegion,
    scale_factor: f64,
    monitors: &[MonitorGeometry],
) -> Result<(), CaptureError> {
    let monitor = monitors
        .iter()
        .find(|monitor| monitor.id == region.monitor_id)
        .ok_or_else(|| {
            capture_error(
                CaptureErrorCode::MonitorUnavailable,
                &region.monitor_id,
                "The selected display configuration has changed.",
            )
        })?;

    if monitor.scale_factor.to_bits() != scale_factor.to_bits() {
        return Err(capture_error(
            CaptureErrorCode::MonitorUnavailable,
            &region.monitor_id,
            "The selected display scale has changed.",
        ));
    }

    let monitor_width = (monitor.logical_size.width * scale_factor).round() as i64;
    let monitor_height = (monitor.logical_size.height * scale_factor).round() as i64;
    let left = i64::from(region.x);
    let top = i64::from(region.y);
    let right = left + i64::from(region.width);
    let bottom = top + i64::from(region.height);
    let monitor_left = i64::from(monitor.physical_origin.x);
    let monitor_top = i64::from(monitor.physical_origin.y);

    if left < monitor_left
        || top < monitor_top
        || right > monitor_left + monitor_width
        || bottom > monitor_top + monitor_height
    {
        return Err(capture_error(
            CaptureErrorCode::RegionOutOfBounds,
            &region.monitor_id,
            "The selected region no longer belongs to the active display.",
        ));
    }

    Ok(())
}

pub(crate) fn frame_delivery_is_current(
    snapshot: &PebbleSessionSnapshot,
    expected_revision: u64,
) -> bool {
    snapshot.revision == expected_revision && snapshot.window_open && !snapshot.privacy_blank_active
}

pub fn confirm_region_selection(
    app: &AppHandle,
    window: &WebviewWindow,
    state: &PebbleSessionState,
    request: RegionSelectionRequest,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    ensure_window(window, REGION_SELECTOR_LABEL)?;
    let monitor = region_selector_monitor_geometry(window)
        .map_err(|error| PebbleSessionError::window_unavailable(error.message))?;
    let selected = state.select_region(trusted_selection_request(request, monitor))?;
    let live_tile = app.state::<LiveTileState>();
    live_tile.close_tile(MAIN_LIVE_TILE_ID, selected.revision.saturating_sub(1));
    live_tile.set_privacy_blank(false, selected.revision);
    let snapshot = open_active_pebble_window(app, state)?;
    emit_session(app, &snapshot);
    window
        .close()
        .map_err(|error| PebbleSessionError::window_unavailable(error.to_string()))?;

    Ok(snapshot)
}

pub(crate) fn trusted_selection_request(
    request: RegionSelectionRequest,
    monitor: MonitorGeometry,
) -> RegionSelectionRequest {
    RegionSelectionRequest {
        monitor,
        start: request.start,
        end: request.end,
    }
}

pub fn show_active_pebble_window(
    app: &AppHandle,
    source_window: &WebviewWindow,
    state: &PebbleSessionState,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    ensure_window(source_window, PEBBLE_TILE_LABEL)?;
    let snapshot = open_active_pebble_window(app, state)?;
    emit_session(app, &snapshot);
    Ok(snapshot)
}

pub fn show_pebble_shell(
    app: &AppHandle,
    state: &PebbleSessionState,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    let snapshot = open_pebble_window(app, state, false)?;
    emit_session(app, &snapshot);
    Ok(snapshot)
}

pub fn set_privacy_blank(
    app: &AppHandle,
    source_window: &WebviewWindow,
    state: &PebbleSessionState,
    active: bool,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    ensure_window(source_window, PEBBLE_TILE_LABEL)?;
    let snapshot = state.set_privacy_blank(active)?;
    app.state::<LiveTileState>()
        .set_privacy_blank(active, snapshot.revision);
    emit_session(app, &snapshot);
    Ok(snapshot)
}

pub fn remove_active_pebble(
    app: &AppHandle,
    source_window: &WebviewWindow,
    state: &PebbleSessionState,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    ensure_window(source_window, PEBBLE_TILE_LABEL)?;
    let snapshot = state.clear()?;
    clear_live_tile(app, snapshot.revision);
    if let Some(window) = app.get_webview_window(PEBBLE_TILE_LABEL) {
        window
            .close()
            .map_err(|error| PebbleSessionError::window_unavailable(error.to_string()))?;
    }
    emit_session(app, &snapshot);
    Ok(snapshot)
}

pub fn close_pebble_window(
    app: &AppHandle,
    window: &WebviewWindow,
    state: &PebbleSessionState,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    ensure_window(window, PEBBLE_TILE_LABEL)?;
    let snapshot = mark_pebble_window_hidden(app, state)?;
    window
        .hide()
        .map_err(|error| PebbleSessionError::window_unavailable(error.to_string()))?;
    Ok(snapshot)
}

pub fn set_ai_panel_expanded(
    window: &WebviewWindow,
    expanded: bool,
) -> Result<(), PebbleSessionError> {
    ensure_window(window, PEBBLE_TILE_LABEL)?;
    let scale_factor = window
        .scale_factor()
        .map_err(|error| PebbleSessionError::window_unavailable(error.to_string()))?;
    let inner_size = window
        .inner_size()
        .map_err(|error| PebbleSessionError::window_unavailable(error.to_string()))?;
    let logical_width = f64::from(inner_size.width) / scale_factor;
    let logical_height = if expanded {
        PEBBLE_WINDOW_AI_HEIGHT
    } else {
        PEBBLE_WINDOW_HEIGHT
    };

    window
        .set_size(LogicalSize::new(
            logical_width.max(PEBBLE_WINDOW_MIN_WIDTH),
            logical_height,
        ))
        .map_err(|error| PebbleSessionError::window_unavailable(error.to_string()))
}

fn open_active_pebble_window(
    app: &AppHandle,
    state: &PebbleSessionState,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    open_pebble_window(app, state, true)
}

fn open_pebble_window(
    app: &AppHandle,
    state: &PebbleSessionState,
    require_region: bool,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    let snapshot = state.snapshot()?;
    if require_region && snapshot.region.is_none() {
        return Err(PebbleSessionError::new(
            PebbleSessionErrorCode::NoActivePebble,
            "Select a region before opening a pebble.",
        ));
    }

    if let Some(window) = app.get_webview_window(PEBBLE_TILE_LABEL) {
        show_existing_window(&window)
            .map_err(|error| PebbleSessionError::window_unavailable(error.message))?;
        return state.set_window_open(true);
    }

    let mut builder = WebviewWindowBuilder::new(
        app,
        PEBBLE_TILE_LABEL,
        WebviewUrl::App("index.html#tile".into()),
    )
    .title("")
    .inner_size(PEBBLE_WINDOW_WIDTH, PEBBLE_WINDOW_HEIGHT)
    .min_inner_size(PEBBLE_WINDOW_MIN_WIDTH, PEBBLE_WINDOW_MIN_HEIGHT)
    .resizable(true)
    .minimizable(false)
    .always_on_top(true)
    .content_protected(true);

    #[cfg(target_os = "macos")]
    {
        builder = builder
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .hidden_title(true)
            .traffic_light_position(tauri::LogicalPosition::new(12.0, 12.0));
    }

    if let Some(region) = snapshot.region.as_ref() {
        if let Some(position) = pebble_window_position(app, region) {
            builder = builder.position(position.logical_x, position.logical_y);
        }
    } else {
        builder = builder.center();
    }

    let window = builder
        .build()
        .map_err(|error| PebbleSessionError::window_unavailable(error.to_string()))?;

    let close_app = app.clone();
    let close_state = state.clone();
    let close_capture = app.state::<LiveTileState>().inner().clone();
    let close_window = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            match close_state.set_window_open(false) {
                Ok(snapshot) => {
                    close_capture.close_tile(MAIN_LIVE_TILE_ID, snapshot.revision);
                    emit_session(&close_app, &snapshot);
                }
                Err(_) => close_capture.close_tile(MAIN_LIVE_TILE_ID, u64::MAX),
            }
            let _ = close_window.hide();
        } else if matches!(event, WindowEvent::Destroyed) {
            close_capture.close_tile(MAIN_LIVE_TILE_ID, u64::MAX);
        }
    });

    state.set_window_open(true)
}

fn clear_live_tile(app: &AppHandle, session_revision: u64) {
    app.state::<LiveTileState>()
        .close_tile(MAIN_LIVE_TILE_ID, session_revision);
}

fn mark_pebble_window_hidden(
    app: &AppHandle,
    state: &PebbleSessionState,
) -> Result<PebbleSessionSnapshot, PebbleSessionError> {
    let snapshot = state.set_window_open(false)?;
    clear_live_tile(app, snapshot.revision);
    emit_session(app, &snapshot);
    Ok(snapshot)
}

fn pebble_window_position(
    app: &AppHandle,
    region: &PhysicalRegion,
) -> Option<PebbleWindowPosition> {
    let monitors = app.available_monitors().ok()?;
    let monitor = monitors
        .iter()
        .find(|monitor| monitor_identifier(monitor) == region.monitor_id)?;
    let origin = monitor.position();
    let size = monitor.size();

    position_pebble_away_from_region(
        region,
        origin.x,
        origin.y,
        i32::try_from(size.width).ok()?,
        i32::try_from(size.height).ok()?,
        monitor.scale_factor(),
    )
}

pub fn position_pebble_away_from_region(
    region: &PhysicalRegion,
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: i32,
    monitor_height: i32,
    scale_factor: f64,
) -> Option<PebbleWindowPosition> {
    if monitor_width <= 0 || monitor_height <= 0 || !scale_factor.is_finite() || scale_factor <= 0.0
    {
        return None;
    }

    let left = i64::from(monitor_x);
    let top = i64::from(monitor_y);
    let right = left + i64::from(monitor_width);
    let bottom = top + i64::from(monitor_height);
    let window_width = (PEBBLE_WINDOW_WIDTH * scale_factor).ceil() as i64;
    let window_height = (PEBBLE_WINDOW_OUTER_HEIGHT * scale_factor).ceil() as i64;
    let margin = (PEBBLE_WINDOW_MARGIN * scale_factor).ceil() as i64;
    let region_left = i64::from(region.x);
    let region_top = i64::from(region.y);
    let region_right = region_left + i64::from(region.width);
    let region_bottom = region_top + i64::from(region.height);

    if window_width > right - left || window_height > bottom - top {
        return Some(PebbleWindowPosition {
            logical_x: left as f64 / scale_factor,
            logical_y: top as f64 / scale_factor,
        });
    }

    let aligned_x = region_left.clamp(left, (right - window_width).max(left));
    let aligned_y = region_top.clamp(top, (bottom - window_height).max(top));
    let candidates = [
        (region_right + margin, aligned_y),
        (region_left - window_width - margin, aligned_y),
        (aligned_x, region_bottom + margin),
        (aligned_x, region_top - window_height - margin),
    ];

    let (physical_x, physical_y) = candidates
        .into_iter()
        .find(|(x, y)| {
            *x >= left && *y >= top && *x + window_width <= right && *y + window_height <= bottom
        })
        .unwrap_or_else(|| {
            [
                (left, top),
                (right - window_width, top),
                (left, bottom - window_height),
                (right - window_width, bottom - window_height),
            ]
            .into_iter()
            .min_by_key(|(x, y)| {
                overlap_area(
                    *x,
                    *y,
                    window_width,
                    window_height,
                    region_left,
                    region_top,
                    i64::from(region.width),
                    i64::from(region.height),
                )
            })
            .unwrap_or((left, top))
        });

    Some(PebbleWindowPosition {
        logical_x: physical_x as f64 / scale_factor,
        logical_y: physical_y as f64 / scale_factor,
    })
}

#[allow(clippy::too_many_arguments)]
fn overlap_area(
    left_a: i64,
    top_a: i64,
    width_a: i64,
    height_a: i64,
    left_b: i64,
    top_b: i64,
    width_b: i64,
    height_b: i64,
) -> i64 {
    let width = (left_a + width_a).min(left_b + width_b) - left_a.max(left_b);
    let height = (top_a + height_a).min(top_b + height_b) - top_a.max(top_b);

    width.max(0) * height.max(0)
}

fn emit_session(app: &AppHandle, snapshot: &PebbleSessionSnapshot) {
    let _ = app.emit_to(PEBBLE_TILE_LABEL, PEBBLE_SESSION_UPDATED_EVENT, snapshot);
}

fn ensure_window(window: &WebviewWindow, expected_label: &str) -> Result<(), PebbleSessionError> {
    if window.label() == expected_label {
        return Ok(());
    }

    Err(PebbleSessionError::new(
        PebbleSessionErrorCode::UnauthorizedWindow,
        "This action is not available from the current window.",
    ))
}

impl PebbleSessionError {
    fn new(code: PebbleSessionErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn window_unavailable(message: impl Into<String>) -> Self {
        Self::new(PebbleSessionErrorCode::WindowUnavailable, message)
    }

    fn state_unavailable() -> Self {
        Self::new(
            PebbleSessionErrorCode::StateUnavailable,
            "Pebble session state is unavailable.",
        )
    }
}
