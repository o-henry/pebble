use serde::{Deserialize, Serialize};

use crate::performance_limits::RegionSize;

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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalRegion {
    pub monitor_id: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
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
    RegionWidthAboveRecommended,
    RegionHeightAboveRecommended,
    RegionWidthTooLarge,
    RegionHeightTooLarge,
}

pub type RegionSelectionResult = Result<RegionSelection, RegionSelectionIssue>;
