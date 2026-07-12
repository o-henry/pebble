use crate::{
    capture_backend::{
        capture_error, validate_region_size, CaptureBackend, CaptureError, CaptureErrorCode,
        CaptureResult,
    },
    region_selection_types::{PhysicalRegion, WindowCaptureTarget},
};
use serde::Serialize;
use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
mod platform_capture_macos;

#[derive(Debug, Clone, Copy, Default)]
pub struct PlatformCaptureBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackdropColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl CaptureBackend for PlatformCaptureBackend {
    fn capture_region(&self, region: &PhysicalRegion) -> CaptureResult {
        self.capture_region_at_scale(region, 1.0)
    }

    fn capture_region_at_scale(&self, region: &PhysicalRegion, scale_factor: f64) -> CaptureResult {
        validate_platform_region(region)?;
        validate_scale_factor(region, scale_factor)?;
        capture_region_platform(region, scale_factor)
    }
}

pub fn capture_real_region_once(region: PhysicalRegion) -> CaptureResult {
    PlatformCaptureBackend.capture_region(&region)
}

#[cfg(target_os = "macos")]
pub fn bind_region_to_source_window(
    region: &PhysicalRegion,
    scale_factor: f64,
) -> Option<WindowCaptureTarget> {
    platform_capture_macos::source_window_for_region(region, scale_factor)
}

#[cfg(not(target_os = "macos"))]
pub fn bind_region_to_source_window(
    _region: &PhysicalRegion,
    _scale_factor: f64,
) -> Option<WindowCaptureTarget> {
    None
}

#[cfg(target_os = "macos")]
pub fn capture_window_backdrop_color(window: &WebviewWindow) -> Option<BackdropColor> {
    platform_capture_macos::capture_window_backdrop_color(window)
}

#[cfg(not(target_os = "macos"))]
pub fn capture_window_backdrop_color(_window: &WebviewWindow) -> Option<BackdropColor> {
    None
}

#[cfg(target_os = "macos")]
pub fn request_screen_capture_access() -> bool {
    platform_capture_macos::request_screen_capture_access()
}

#[cfg(not(target_os = "macos"))]
pub fn request_screen_capture_access() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn capture_region_platform(region: &PhysicalRegion, scale_factor: f64) -> CaptureResult {
    platform_capture_macos::capture_region(region, scale_factor)
}

#[cfg(not(target_os = "macos"))]
fn capture_region_platform(region: &PhysicalRegion, _scale_factor: f64) -> CaptureResult {
    Err(capture_error(
        CaptureErrorCode::PlatformUnavailable,
        &region.monitor_id,
        "Real capture is available only on macOS in this build.",
    ))
}

fn validate_scale_factor(region: &PhysicalRegion, scale_factor: f64) -> Result<(), CaptureError> {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        return Ok(());
    }

    Err(capture_error(
        CaptureErrorCode::InvalidRegion,
        &region.monitor_id,
        "Capture scale factor must be finite and positive.",
    ))
}

fn validate_platform_region(region: &PhysicalRegion) -> Result<(), CaptureError> {
    validate_region_size(region)
}

#[cfg(test)]
pub(crate) mod platform_capture_test_support {
    use super::*;
    use crate::capture_backend::CaptureErrorCode;

    pub fn invalid_region_error() -> CaptureError {
        PlatformCaptureBackend
            .capture_region(&PhysicalRegion {
                monitor_id: "main".to_string(),
                x: 0,
                y: 0,
                width: 0,
                height: 24,
                source_window: None,
            })
            .expect_err("invalid region should fail before platform capture")
    }

    pub fn large_region_is_valid() -> bool {
        validate_platform_region(&PhysicalRegion {
            monitor_id: "main".to_string(),
            x: 0,
            y: 0,
            width: 7680,
            height: 4320,
            source_window: None,
        })
        .is_ok()
    }

    #[cfg(target_os = "macos")]
    pub fn copy_bgra_rows_to_rgba(
        source: &[u8],
        bytes_per_row: usize,
        width: usize,
        height: usize,
        expected_len: usize,
    ) -> Result<Vec<u8>, CaptureErrorCode> {
        platform_capture_macos::test_copy_bgra_rows_to_rgba(
            source,
            bytes_per_row,
            width,
            height,
            expected_len,
        )
    }

    #[cfg(target_os = "macos")]
    pub fn capture_rect(region: &PhysicalRegion, scale_factor: f64) -> (f64, f64, f64, f64) {
        platform_capture_macos::test_capture_rect(region, scale_factor)
    }

    #[cfg(target_os = "macos")]
    pub fn is_supported_bgra_bitmap_info(bitmap_info: u32) -> bool {
        platform_capture_macos::test_is_supported_bgra_bitmap_info(bitmap_info)
    }

    #[cfg(target_os = "macos")]
    pub fn backdrop_rect(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        scale_factor: f64,
    ) -> (f64, f64, f64, f64) {
        platform_capture_macos::test_backdrop_rect(x, y, width, height, scale_factor)
    }
}
