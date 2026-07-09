use crate::{
    capture_backend::{
        capture_error, validate_region_size, CaptureBackend, CaptureError, CaptureErrorCode,
        CaptureResult,
    },
    performance_limits::{PerformanceLimitErrorCode, PerformanceLimits, RegionSize},
    region_selection_types::PhysicalRegion,
};

#[cfg(target_os = "macos")]
mod platform_capture_macos;

#[derive(Debug, Clone, Copy, Default)]
pub struct PlatformCaptureBackend;

impl CaptureBackend for PlatformCaptureBackend {
    fn capture_region(&self, region: &PhysicalRegion) -> CaptureResult {
        validate_platform_region(region)?;
        capture_region_platform(region)
    }
}

pub fn capture_real_region_once(region: PhysicalRegion) -> CaptureResult {
    PlatformCaptureBackend.capture_region(&region)
}

#[cfg(target_os = "macos")]
fn capture_region_platform(region: &PhysicalRegion) -> CaptureResult {
    platform_capture_macos::capture_region(region)
}

#[cfg(not(target_os = "macos"))]
fn capture_region_platform(region: &PhysicalRegion) -> CaptureResult {
    Err(capture_error(
        CaptureErrorCode::PlatformUnavailable,
        &region.monitor_id,
        "Real capture is available only on macOS in this build.",
    ))
}

fn validate_platform_region(region: &PhysicalRegion) -> Result<(), CaptureError> {
    validate_region_size(region)?;
    PerformanceLimits::default()
        .validate_region_size(RegionSize {
            width: region.width,
            height: region.height,
        })
        .map_err(|error| match error.code {
            PerformanceLimitErrorCode::RegionWidthTooLarge
            | PerformanceLimitErrorCode::RegionHeightTooLarge => capture_error(
                CaptureErrorCode::RegionTooLarge,
                &region.monitor_id,
                format!(
                    "Capture region is too large: actual {}, limit {}.",
                    error.actual, error.limit
                ),
            ),
            _ => capture_error(
                CaptureErrorCode::InvalidRegion,
                &region.monitor_id,
                "Capture region dimensions must be positive.",
            ),
        })
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
            })
            .expect_err("invalid region should fail before platform capture")
    }

    pub fn oversized_region_error() -> CaptureError {
        PlatformCaptureBackend
            .capture_region(&PhysicalRegion {
                monitor_id: "main".to_string(),
                x: 0,
                y: 0,
                width: 801,
                height: 24,
            })
            .expect_err("oversized region should fail before platform capture")
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
    pub fn capture_rect(region: &PhysicalRegion) -> (f64, f64, f64, f64) {
        platform_capture_macos::test_capture_rect(region)
    }

    #[cfg(target_os = "macos")]
    pub fn is_supported_bgra_bitmap_info(bitmap_info: u32) -> bool {
        platform_capture_macos::test_is_supported_bgra_bitmap_info(bitmap_info)
    }
}
