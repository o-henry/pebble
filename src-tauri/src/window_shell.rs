use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

pub const TEST_TILE_LABEL: &str = "screenpebble-test-tile";

const TEST_TILE_TITLE: &str = "Test Pebble";
const TEST_TILE_WIDTH: f64 = 320.0;
const TEST_TILE_HEIGHT: f64 = 220.0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TileWindowShell {
    pub id: &'static str,
    pub label: &'static str,
    pub title: &'static str,
    pub mode: TileMode,
    pub always_on_top: bool,
    pub capture_active: bool,
    pub placeholder: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TileMode {
    Live,
    Paused,
    Hidden,
    Blanked,
    Error,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowShellSnapshot {
    pub test_tile: TileWindowShell,
    pub supported_modes: Vec<TileMode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowShellError {
    pub code: WindowShellErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowShellErrorCode {
    TileWindowUnavailable,
}

#[derive(Debug, Clone)]
pub struct WindowShellState {
    test_tile: Arc<Mutex<TileWindowShell>>,
}

impl Default for WindowShellState {
    fn default() -> Self {
        Self {
            test_tile: Arc::new(Mutex::new(TileWindowShell::closed())),
        }
    }
}

impl WindowShellState {
    pub fn snapshot(&self) -> Result<WindowShellSnapshot, WindowShellError> {
        Ok(WindowShellSnapshot {
            test_tile: self.test_tile()?,
            supported_modes: TileMode::all().to_vec(),
        })
    }

    pub fn mark_test_tile_live(&self) -> Result<TileWindowShell, WindowShellError> {
        self.update_test_tile(TileMode::Live)
    }

    pub fn mark_test_tile_closed(&self) -> Result<TileWindowShell, WindowShellError> {
        self.update_test_tile(TileMode::Closed)
    }

    pub fn test_tile(&self) -> Result<TileWindowShell, WindowShellError> {
        self.test_tile
            .lock()
            .map(|tile| tile.clone())
            .map_err(|_| WindowShellError::unavailable("test tile state lock failed"))
    }

    fn update_test_tile(&self, mode: TileMode) -> Result<TileWindowShell, WindowShellError> {
        let mut tile = self
            .test_tile
            .lock()
            .map_err(|_| WindowShellError::unavailable("test tile state lock failed"))?;
        tile.mode = mode;
        tile.capture_active = false;

        Ok(tile.clone())
    }
}

impl TileWindowShell {
    pub fn closed() -> Self {
        Self {
            id: "test-tile",
            label: TEST_TILE_LABEL,
            title: TEST_TILE_TITLE,
            mode: TileMode::Closed,
            always_on_top: true,
            capture_active: false,
            placeholder: "Fake tile placeholder. Capture is not implemented.",
        }
    }
}

impl TileMode {
    fn all() -> [Self; 6] {
        [
            Self::Live,
            Self::Paused,
            Self::Hidden,
            Self::Blanked,
            Self::Error,
            Self::Closed,
        ]
    }
}

impl WindowShellError {
    pub(crate) fn unavailable(message: impl Into<String>) -> Self {
        Self {
            code: WindowShellErrorCode::TileWindowUnavailable,
            message: message.into(),
        }
    }
}

pub fn open_test_tile_window(
    app: &AppHandle,
    state: &WindowShellState,
) -> Result<TileWindowShell, WindowShellError> {
    if let Some(window) = app.get_webview_window(TEST_TILE_LABEL) {
        show_existing_window(&window)?;
        return state.mark_test_tile_live();
    }

    let window = WebviewWindowBuilder::new(
        app,
        TEST_TILE_LABEL,
        WebviewUrl::App("index.html#tile".into()),
    )
    .title(TEST_TILE_TITLE)
    .inner_size(TEST_TILE_WIDTH, TEST_TILE_HEIGHT)
    .resizable(false)
    .always_on_top(true)
    .build()
    .map_err(|error| WindowShellError::unavailable(error.to_string()))?;

    let close_state = state.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            let _ = close_state.mark_test_tile_closed();
        }
    });

    state.mark_test_tile_live()
}

pub(crate) fn show_existing_window(window: &WebviewWindow) -> Result<(), WindowShellError> {
    window
        .show()
        .map_err(|error| WindowShellError::unavailable(error.to_string()))?;
    window
        .set_focus()
        .map_err(|error| WindowShellError::unavailable(error.to_string()))
}
