use serde::{Deserialize, Serialize};

const DEFAULT_FPS: i32 = 1;
const MAX_FPS: i32 = 5;
const MAX_ACTIVE_TILES: i32 = 3;
const UNBOUNDED_REGION_DIMENSION: i32 = i32::MAX;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceLimits {
    pub default_fps: i32,
    pub max_fps: i32,
    pub max_active_tiles: i32,
    pub recommended_region: RegionSize,
    pub max_region: RegionSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceLimitRequest {
    pub fps: i32,
    pub active_tile_count: i32,
    pub region: RegionSize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceLimitError {
    pub code: PerformanceLimitErrorCode,
    pub limit: i32,
    pub actual: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PerformanceLimitErrorCode {
    FpsTooLow,
    FpsTooHigh,
    ActiveTileCountTooLow,
    ActiveTileLimitExceeded,
    RegionWidthTooLarge,
    RegionHeightTooLarge,
    RegionEmpty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceValidation {
    pub valid: bool,
    pub error: Option<PerformanceLimitError>,
}

pub type PerformanceLimitResult = Result<(), PerformanceLimitError>;

impl Default for PerformanceLimits {
    fn default() -> Self {
        Self {
            default_fps: DEFAULT_FPS,
            max_fps: MAX_FPS,
            max_active_tiles: MAX_ACTIVE_TILES,
            recommended_region: RegionSize {
                width: UNBOUNDED_REGION_DIMENSION,
                height: UNBOUNDED_REGION_DIMENSION,
            },
            max_region: RegionSize {
                width: UNBOUNDED_REGION_DIMENSION,
                height: UNBOUNDED_REGION_DIMENSION,
            },
        }
    }
}

impl PerformanceLimits {
    pub fn validate(&self, request: PerformanceLimitRequest) -> PerformanceLimitResult {
        self.validate_fps(request.fps)?;
        self.validate_active_tile_count(request.active_tile_count)?;
        self.validate_region_size(request.region)
    }

    pub fn validate_fps(&self, fps: i32) -> PerformanceLimitResult {
        if fps < 1 {
            return Err(PerformanceLimitError {
                code: PerformanceLimitErrorCode::FpsTooLow,
                limit: 1,
                actual: fps,
            });
        }

        if fps > self.max_fps {
            return Err(PerformanceLimitError {
                code: PerformanceLimitErrorCode::FpsTooHigh,
                limit: self.max_fps,
                actual: fps,
            });
        }

        Ok(())
    }

    pub fn validate_active_tile_count(&self, active_tile_count: i32) -> PerformanceLimitResult {
        if active_tile_count < 0 {
            return Err(PerformanceLimitError {
                code: PerformanceLimitErrorCode::ActiveTileCountTooLow,
                limit: 0,
                actual: active_tile_count,
            });
        }

        if active_tile_count > self.max_active_tiles {
            return Err(PerformanceLimitError {
                code: PerformanceLimitErrorCode::ActiveTileLimitExceeded,
                limit: self.max_active_tiles,
                actual: active_tile_count,
            });
        }

        Ok(())
    }

    pub fn validate_region_size(&self, region: RegionSize) -> PerformanceLimitResult {
        if region.width < 1 {
            return Err(PerformanceLimitError {
                code: PerformanceLimitErrorCode::RegionEmpty,
                limit: 1,
                actual: region.width,
            });
        }

        if region.height < 1 {
            return Err(PerformanceLimitError {
                code: PerformanceLimitErrorCode::RegionEmpty,
                limit: 1,
                actual: region.height,
            });
        }

        if region.width > self.max_region.width {
            return Err(PerformanceLimitError {
                code: PerformanceLimitErrorCode::RegionWidthTooLarge,
                limit: self.max_region.width,
                actual: region.width,
            });
        }

        if region.height > self.max_region.height {
            return Err(PerformanceLimitError {
                code: PerformanceLimitErrorCode::RegionHeightTooLarge,
                limit: self.max_region.height,
                actual: region.height,
            });
        }

        Ok(())
    }
}

impl From<PerformanceLimitResult> for PerformanceValidation {
    fn from(result: PerformanceLimitResult) -> Self {
        match result {
            Ok(()) => Self {
                valid: true,
                error: None,
            },
            Err(error) => Self {
                valid: false,
                error: Some(error),
            },
        }
    }
}
