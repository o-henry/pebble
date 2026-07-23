use serde::{Deserialize, Serialize};

use crate::performance_limits::RegionSize;

#[cfg(target_os = "macos")]
use std::sync::Arc;

#[cfg(target_os = "macos")]
use screencapturekit::shareable_content::SCWindow;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogicalPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogicalSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorGeometry {
    pub id: String,
    pub logical_origin: LogicalPoint,
    pub logical_size: LogicalSize,
    pub physical_origin: PhysicalPoint,
    pub scale_factor: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionSelectionRequest {
    pub monitor: MonitorGeometry,
    pub start: LogicalPoint,
    pub end: LogicalPoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowCaptureTarget {
    pub window_id: u32,
    pub owner_pid: i32,
    pub source_width_millipoints: u64,
    pub source_height_millipoints: u64,
    #[cfg(target_os = "macos")]
    pub native_window: Option<Arc<SCWindow>>,
    pub relative_x_millipoints: i64,
    pub relative_y_millipoints: i64,
    pub width_millipoints: u64,
    pub height_millipoints: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalRegion {
    pub monitor_id: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    #[serde(skip)]
    pub source_window: Option<WindowCaptureTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionSelectionLimits {
    pub minimum_region: RegionSize,
    pub recommended_region: RegionSize,
    pub max_region: RegionSize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionSelection {
    pub region: PhysicalRegion,
    pub warnings: Vec<RegionSelectionIssue>,
    pub limits: RegionSelectionLimits,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionSelectionIssue {
    pub code: RegionSelectionIssueCode,
    pub limit: f64,
    pub actual: f64,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RegionSelectionIssueCode {
    InvalidCoordinate,
    InvalidScaleFactor,
    SelectionOutsideMonitor,
    RegionTooNarrow,
    RegionTooShort,
    RegionCoordinateOutOfRange,
}

pub type RegionSelectionResult = Result<RegionSelection, RegionSelectionIssue>;
