use serde::Serialize;
use tauri::{
    window::Color, AppHandle, Manager, Monitor, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use crate::{
    region_selection_types::{LogicalPoint, LogicalSize, MonitorGeometry, PhysicalPoint},
    window_shell::{show_existing_window, WindowShellError},
};

pub const REGION_SELECTOR_LABEL: &str = "screenpebble-region-selector";

const REGION_SELECTOR_TITLE: &str = "Select Region";
const REGION_SELECTOR_FALLBACK_WIDTH: f64 = 960.0;
const REGION_SELECTOR_FALLBACK_HEIGHT: f64 = 640.0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionSelectorWindowShell {
    pub label: &'static str,
    pub title: &'static str,
    pub visual_overlay: bool,
    pub native_transparent: bool,
    pub always_on_top: bool,
    pub capture_active: bool,
}

pub fn open_region_selector_window(
    app: &AppHandle,
    source_window: Option<&WebviewWindow>,
) -> Result<RegionSelectorWindowShell, WindowShellError> {
    if let Some(window) = app.get_webview_window(REGION_SELECTOR_LABEL) {
        show_existing_window(&window)?;
        return Ok(RegionSelectorWindowShell::transparent_overlay());
    }

    let geometry = selector_window_geometry(app, source_window)?;
    WebviewWindowBuilder::new(
        app,
        REGION_SELECTOR_LABEL,
        WebviewUrl::App("index.html#selector".into()),
    )
    .title(REGION_SELECTOR_TITLE)
    .position(geometry.logical_x, geometry.logical_y)
    .inner_size(geometry.logical_width, geometry.logical_height)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .background_color(Color(0, 0, 0, 0))
    .always_on_top(true)
    .skip_taskbar(true)
    .build()
    .map_err(|error| WindowShellError::unavailable(error.to_string()))?;

    Ok(RegionSelectorWindowShell::transparent_overlay())
}

pub fn region_selector_monitor_geometry(
    window: &WebviewWindow,
) -> Result<MonitorGeometry, WindowShellError> {
    ensure_region_selector_window(window)?;

    let monitor = window
        .current_monitor()
        .map_err(|error| WindowShellError::unavailable(error.to_string()))?
        .ok_or_else(|| WindowShellError::unavailable("Active display is unavailable."))?;

    Ok(monitor_geometry(&monitor))
}

pub(crate) fn available_monitor_geometries(
    app: &AppHandle,
) -> Result<Vec<MonitorGeometry>, WindowShellError> {
    app.available_monitors()
        .map(|monitors| monitors.iter().map(monitor_geometry).collect())
        .map_err(|error| WindowShellError::unavailable(error.to_string()))
}

fn monitor_geometry(monitor: &Monitor) -> MonitorGeometry {
    let position = monitor.position();
    let size = monitor.size();
    let scale_factor = monitor.scale_factor();

    MonitorGeometry {
        id: monitor_identifier(monitor),
        logical_origin: LogicalPoint { x: 0.0, y: 0.0 },
        logical_size: LogicalSize {
            width: size.width as f64 / scale_factor,
            height: size.height as f64 / scale_factor,
        },
        physical_origin: PhysicalPoint {
            x: position.x,
            y: position.y,
        },
        scale_factor,
    }
}

pub(crate) fn monitor_identifier(monitor: &Monitor) -> String {
    let position = monitor.position();
    let size = monitor.size();

    format!(
        "display:{}:{}:{}:{}:{:x}",
        position.x,
        position.y,
        size.width,
        size.height,
        monitor.scale_factor().to_bits()
    )
}

pub fn close_region_selector_window(window: &WebviewWindow) -> Result<(), WindowShellError> {
    ensure_region_selector_window(window)?;
    window
        .close()
        .map_err(|error| WindowShellError::unavailable(error.to_string()))
}

impl RegionSelectorWindowShell {
    pub fn transparent_overlay() -> Self {
        Self {
            label: REGION_SELECTOR_LABEL,
            title: REGION_SELECTOR_TITLE,
            visual_overlay: true,
            native_transparent: true,
            always_on_top: true,
            capture_active: false,
        }
    }
}

struct SelectorWindowGeometry {
    logical_x: f64,
    logical_y: f64,
    logical_width: f64,
    logical_height: f64,
}

fn selector_window_geometry(
    app: &AppHandle,
    source_window: Option<&WebviewWindow>,
) -> Result<SelectorWindowGeometry, WindowShellError> {
    let source_monitor = match source_window {
        Some(window) => window
            .current_monitor()
            .map_err(|error| WindowShellError::unavailable(error.to_string()))?,
        None => None,
    };
    let primary_monitor = app
        .primary_monitor()
        .map_err(|error| WindowShellError::unavailable(error.to_string()))?;
    let Some(monitor) = source_monitor.or(primary_monitor) else {
        return Ok(SelectorWindowGeometry {
            logical_x: 0.0,
            logical_y: 0.0,
            logical_width: REGION_SELECTOR_FALLBACK_WIDTH,
            logical_height: REGION_SELECTOR_FALLBACK_HEIGHT,
        });
    };
    let scale_factor = monitor.scale_factor();
    let position = monitor.position();
    let size = monitor.size();

    Ok(SelectorWindowGeometry {
        logical_x: position.x as f64 / scale_factor,
        logical_y: position.y as f64 / scale_factor,
        logical_width: size.width as f64 / scale_factor,
        logical_height: size.height as f64 / scale_factor,
    })
}

fn ensure_region_selector_window(window: &WebviewWindow) -> Result<(), WindowShellError> {
    if window.label() == REGION_SELECTOR_LABEL {
        return Ok(());
    }

    Err(WindowShellError::unavailable(
        "command is only available from the region selector window",
    ))
}
