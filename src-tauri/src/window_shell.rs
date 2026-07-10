use serde::Serialize;
use tauri::WebviewWindow;

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

impl WindowShellError {
    pub(crate) fn unavailable(message: impl Into<String>) -> Self {
        Self {
            code: WindowShellErrorCode::TileWindowUnavailable,
            message: message.into(),
        }
    }
}

pub(crate) fn show_existing_window(window: &WebviewWindow) -> Result<(), WindowShellError> {
    window
        .show()
        .map_err(|error| WindowShellError::unavailable(error.to_string()))?;
    window
        .set_focus()
        .map_err(|error| WindowShellError::unavailable(error.to_string()))
}
