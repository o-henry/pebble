use crate::performance_limits::{PerformanceLimits, RegionSize};
use crate::region_selection_types::{
    LogicalPoint, PhysicalRegion, RegionSelection, RegionSelectionIssue, RegionSelectionIssueCode,
    RegionSelectionLimits, RegionSelectionRequest, RegionSelectionResult,
};

const MIN_REGION_WIDTH: i32 = 24;
const MIN_REGION_HEIGHT: i32 = 24;

impl Default for RegionSelectionLimits {
    fn default() -> Self {
        let performance_limits = PerformanceLimits::default();

        Self {
            minimum_region: RegionSize {
                width: MIN_REGION_WIDTH,
                height: MIN_REGION_HEIGHT,
            },
            recommended_region: performance_limits.recommended_region,
            max_region: performance_limits.max_region,
        }
    }
}

pub fn select_region(request: RegionSelectionRequest) -> RegionSelectionResult {
    validate_geometry(&request)?;

    let limits = RegionSelectionLimits::default();
    let region = map_logical_selection_to_physical(&request)?;

    validate_minimum_region(&region, &limits)?;
    validate_hard_limits(&region, &limits)?;

    Ok(RegionSelection {
        warnings: recommended_warnings(&region, &limits),
        region,
        limits,
    })
}

fn validate_geometry(request: &RegionSelectionRequest) -> Result<(), RegionSelectionIssue> {
    let points = [
        request.monitor.logical_origin,
        LogicalPoint {
            x: request.monitor.physical_origin.x as f64,
            y: request.monitor.physical_origin.y as f64,
        },
        request.start,
        request.end,
    ];

    if points
        .iter()
        .any(|point| !point.x.is_finite() || !point.y.is_finite())
    {
        return Err(issue(RegionSelectionIssueCode::InvalidCoordinate, 0.0, 0.0));
    }

    if !request.monitor.scale_factor.is_finite() || request.monitor.scale_factor <= 0.0 {
        return Err(issue(
            RegionSelectionIssueCode::InvalidScaleFactor,
            0.0,
            finite_or_zero(request.monitor.scale_factor),
        ));
    }

    Ok(())
}

fn map_logical_selection_to_physical(
    request: &RegionSelectionRequest,
) -> Result<PhysicalRegion, RegionSelectionIssue> {
    let left = request.start.x.min(request.end.x);
    let right = request.start.x.max(request.end.x);
    let top = request.start.y.min(request.end.y);
    let bottom = request.start.y.max(request.end.y);

    let (x, width) = map_axis(
        left,
        right,
        request.monitor.logical_origin.x,
        request.monitor.physical_origin.x,
        request.monitor.scale_factor,
    )?;
    let (y, height) = map_axis(
        top,
        bottom,
        request.monitor.logical_origin.y,
        request.monitor.physical_origin.y,
        request.monitor.scale_factor,
    )?;

    Ok(PhysicalRegion {
        monitor_id: request.monitor.id.clone(),
        x,
        y,
        width,
        height,
    })
}

fn map_axis(
    min: f64,
    max: f64,
    logical_origin: f64,
    physical_origin: i32,
    scale: f64,
) -> Result<(i32, i32), RegionSelectionIssue> {
    let start = checked_i32(((min - logical_origin) * scale).floor() + physical_origin as f64)?;
    let end = checked_i32(((max - logical_origin) * scale).ceil() + physical_origin as f64)?;
    let width = i64::from(end) - i64::from(start);

    if width < 0 || width > i64::from(i32::MAX) {
        return Err(issue(
            RegionSelectionIssueCode::RegionCoordinateOutOfRange,
            i32::MAX as f64,
            width as f64,
        ));
    }

    Ok((start, width as i32))
}

fn checked_i32(value: f64) -> Result<i32, RegionSelectionIssue> {
    if !value.is_finite() || value < i32::MIN as f64 || value > i32::MAX as f64 {
        return Err(issue(
            RegionSelectionIssueCode::RegionCoordinateOutOfRange,
            i32::MAX as f64,
            value,
        ));
    }

    Ok(value as i32)
}

fn validate_minimum_region(
    region: &PhysicalRegion,
    limits: &RegionSelectionLimits,
) -> Result<(), RegionSelectionIssue> {
    if region.width < limits.minimum_region.width {
        return Err(issue(
            RegionSelectionIssueCode::RegionTooNarrow,
            limits.minimum_region.width as f64,
            region.width as f64,
        ));
    }

    if region.height < limits.minimum_region.height {
        return Err(issue(
            RegionSelectionIssueCode::RegionTooShort,
            limits.minimum_region.height as f64,
            region.height as f64,
        ));
    }

    Ok(())
}

fn validate_hard_limits(
    region: &PhysicalRegion,
    limits: &RegionSelectionLimits,
) -> Result<(), RegionSelectionIssue> {
    if region.width > limits.max_region.width {
        return Err(issue(
            RegionSelectionIssueCode::RegionWidthTooLarge,
            limits.max_region.width as f64,
            region.width as f64,
        ));
    }

    if region.height > limits.max_region.height {
        return Err(issue(
            RegionSelectionIssueCode::RegionHeightTooLarge,
            limits.max_region.height as f64,
            region.height as f64,
        ));
    }

    Ok(())
}

fn recommended_warnings(
    region: &PhysicalRegion,
    limits: &RegionSelectionLimits,
) -> Vec<RegionSelectionIssue> {
    let mut warnings = Vec::with_capacity(2);

    if region.width > limits.recommended_region.width {
        warnings.push(issue(
            RegionSelectionIssueCode::RegionWidthAboveRecommended,
            limits.recommended_region.width as f64,
            region.width as f64,
        ));
    }

    if region.height > limits.recommended_region.height {
        warnings.push(issue(
            RegionSelectionIssueCode::RegionHeightAboveRecommended,
            limits.recommended_region.height as f64,
            region.height as f64,
        ));
    }

    warnings
}

fn issue(code: RegionSelectionIssueCode, limit: f64, actual: f64) -> RegionSelectionIssue {
    RegionSelectionIssue {
        code,
        limit: finite_or_zero(limit),
        actual: finite_or_zero(actual),
        message: message_for(code).to_string(),
    }
}

fn finite_or_zero(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

fn message_for(code: RegionSelectionIssueCode) -> &'static str {
    match code {
        RegionSelectionIssueCode::InvalidCoordinate => "Selection coordinates must be finite.",
        RegionSelectionIssueCode::InvalidScaleFactor => {
            "Monitor scale factor must be greater than zero."
        }
        RegionSelectionIssueCode::RegionTooNarrow => "Selected region is too narrow.",
        RegionSelectionIssueCode::RegionTooShort => "Selected region is too short.",
        RegionSelectionIssueCode::RegionCoordinateOutOfRange => {
            "Selected region is outside the supported coordinate range."
        }
        RegionSelectionIssueCode::RegionWidthAboveRecommended => {
            "Selected region is wider than recommended."
        }
        RegionSelectionIssueCode::RegionHeightAboveRecommended => {
            "Selected region is taller than recommended."
        }
        RegionSelectionIssueCode::RegionWidthTooLarge => "Selected region is wider than allowed.",
        RegionSelectionIssueCode::RegionHeightTooLarge => "Selected region is taller than allowed.",
    }
}
