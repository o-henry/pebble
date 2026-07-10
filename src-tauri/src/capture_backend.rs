use serde::Serialize;

use crate::{
    performance_limits::RegionSize,
    region_selection_types::{PhysicalPoint, PhysicalRegion},
};

pub(crate) const RGBA_BYTES_PER_PIXEL: usize = 4;
const FAKE_MONITOR_ID: &str = "main";
const FAKE_MONITOR_WIDTH: i32 = 800;
const FAKE_MONITOR_HEIGHT: i32 = 600;

pub trait CaptureBackend {
    fn capture_region(&self, region: &PhysicalRegion) -> CaptureResult;

    fn capture_region_at_scale(
        &self,
        region: &PhysicalRegion,
        _scale_factor: f64,
    ) -> CaptureResult {
        self.capture_region(region)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureMonitor {
    pub id: String,
    pub physical_origin: PhysicalPoint,
    pub size: RegionSize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CroppedFramePayload {
    pub monitor_id: String,
    pub region: PhysicalRegion,
    pub width: i32,
    pub height: i32,
    pub pixel_format: FramePixelFormat,
    pub bytes_per_pixel: i32,
    pub storage_policy: FrameStoragePolicy,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FramePixelFormat {
    Rgba8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FrameStoragePolicy {
    MemoryOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureError {
    pub code: CaptureErrorCode,
    pub monitor_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureErrorCode {
    ActiveTileLimitExceeded,
    CaptureUnavailable,
    InvalidRegion,
    MonitorUnavailable,
    PermissionDenied,
    PlatformUnavailable,
    RegionTooLarge,
    RegionOutOfBounds,
    UnsupportedPixelFormat,
    UnauthorizedWindow,
}

pub type CaptureResult = Result<CroppedFramePayload, CaptureError>;

#[derive(Debug, Clone)]
pub struct FakeCaptureBackend {
    monitors: Vec<CaptureMonitor>,
}

impl Default for FakeCaptureBackend {
    fn default() -> Self {
        Self::with_monitors(vec![CaptureMonitor {
            id: FAKE_MONITOR_ID.to_string(),
            physical_origin: PhysicalPoint { x: 0, y: 0 },
            size: RegionSize {
                width: FAKE_MONITOR_WIDTH,
                height: FAKE_MONITOR_HEIGHT,
            },
        }])
    }
}

impl FakeCaptureBackend {
    pub fn with_monitors(monitors: Vec<CaptureMonitor>) -> Self {
        Self { monitors }
    }

    fn monitor_for(&self, monitor_id: &str) -> Option<&CaptureMonitor> {
        self.monitors
            .iter()
            .find(|monitor| monitor.id == monitor_id)
    }
}

impl CaptureBackend for FakeCaptureBackend {
    fn capture_region(&self, region: &PhysicalRegion) -> CaptureResult {
        validate_region_size(region)?;
        let monitor = self.monitor_for(&region.monitor_id).ok_or_else(|| {
            capture_error(
                CaptureErrorCode::MonitorUnavailable,
                &region.monitor_id,
                "Capture monitor is not available.",
            )
        })?;

        if !region_is_inside_monitor(region, monitor) {
            return Err(capture_error(
                CaptureErrorCode::RegionOutOfBounds,
                &region.monitor_id,
                "Selected region is outside the capture monitor.",
            ));
        }

        Ok(cropped_frame(region, fake_region_bytes(region)?))
    }
}

pub fn capture_region_once(region: PhysicalRegion) -> CaptureResult {
    FakeCaptureBackend::default().capture_region(&region)
}

pub(crate) fn validate_region_size(region: &PhysicalRegion) -> Result<(), CaptureError> {
    if region.width < 1 || region.height < 1 {
        return Err(capture_error(
            CaptureErrorCode::InvalidRegion,
            &region.monitor_id,
            "Capture region dimensions must be positive.",
        ));
    }

    Ok(())
}

fn region_is_inside_monitor(region: &PhysicalRegion, monitor: &CaptureMonitor) -> bool {
    let left = i64::from(region.x);
    let top = i64::from(region.y);
    let right = left + i64::from(region.width);
    let bottom = top + i64::from(region.height);
    let monitor_left = i64::from(monitor.physical_origin.x);
    let monitor_top = i64::from(monitor.physical_origin.y);
    let monitor_right = monitor_left + i64::from(monitor.size.width);
    let monitor_bottom = monitor_top + i64::from(monitor.size.height);

    left >= monitor_left && top >= monitor_top && right <= monitor_right && bottom <= monitor_bottom
}

fn fake_region_bytes(region: &PhysicalRegion) -> Result<Vec<u8>, CaptureError> {
    let mut bytes = Vec::with_capacity(byte_len(region)?);
    let bottom = i64::from(region.y) + i64::from(region.height);
    let right = i64::from(region.x) + i64::from(region.width);

    for y in i64::from(region.y)..bottom {
        for x in i64::from(region.x)..right {
            bytes.extend_from_slice(&fake_pixel(x, y));
        }
    }

    Ok(bytes)
}

pub(crate) fn byte_len(region: &PhysicalRegion) -> Result<usize, CaptureError> {
    let width = usize::try_from(region.width).map_err(|_| {
        capture_error(
            CaptureErrorCode::InvalidRegion,
            &region.monitor_id,
            "Capture region width must be positive.",
        )
    })?;
    let height = usize::try_from(region.height).map_err(|_| {
        capture_error(
            CaptureErrorCode::InvalidRegion,
            &region.monitor_id,
            "Capture region height must be positive.",
        )
    })?;
    width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(RGBA_BYTES_PER_PIXEL))
        .ok_or_else(|| {
            capture_error(
                CaptureErrorCode::InvalidRegion,
                &region.monitor_id,
                "Capture region byte length is too large.",
            )
        })
}

pub(crate) fn cropped_frame(region: &PhysicalRegion, bytes: Vec<u8>) -> CroppedFramePayload {
    CroppedFramePayload {
        monitor_id: region.monitor_id.clone(),
        region: region.clone(),
        width: region.width,
        height: region.height,
        pixel_format: FramePixelFormat::Rgba8,
        bytes_per_pixel: RGBA_BYTES_PER_PIXEL as i32,
        storage_policy: FrameStoragePolicy::MemoryOnly,
        bytes,
    }
}

fn fake_pixel(x: i64, y: i64) -> [u8; RGBA_BYTES_PER_PIXEL] {
    [
        x.rem_euclid(256) as u8,
        y.rem_euclid(256) as u8,
        (x + y).rem_euclid(256) as u8,
        255,
    ]
}

pub(crate) fn capture_error(
    code: CaptureErrorCode,
    monitor_id: &str,
    message: impl Into<String>,
) -> CaptureError {
    CaptureError {
        code,
        monitor_id: monitor_id.to_string(),
        message: message.into(),
    }
}
