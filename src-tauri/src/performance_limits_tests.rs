use crate::performance_limits::{
    PerformanceLimitErrorCode, PerformanceLimitRequest, PerformanceLimits, PerformanceValidation,
    RegionSize,
};

#[test]
fn default_limits_match_the_product_contract() {
    let limits = PerformanceLimits::default();

    assert_eq!(limits.default_fps, 1);
    assert_eq!(limits.max_fps, 5);
    assert_eq!(limits.max_active_tiles, 3);
    assert_eq!(limits.recommended_region.width, 600);
    assert_eq!(limits.recommended_region.height, 300);
    assert_eq!(limits.max_region.width, 800);
    assert_eq!(limits.max_region.height, 600);
}

#[test]
fn valid_request_passes() {
    let limits = PerformanceLimits::default();
    let request = PerformanceLimitRequest {
        fps: 1,
        active_tile_count: 3,
        region: RegionSize {
            width: 800,
            height: 600,
        },
    };

    assert_eq!(limits.validate(request), Ok(()));
    assert_eq!(
        PerformanceValidation::from(limits.validate(request)),
        PerformanceValidation {
            valid: true,
            error: None
        }
    );
}

#[test]
fn fps_must_be_at_least_one() {
    let error = PerformanceLimits::default().validate_fps(0).unwrap_err();

    assert_eq!(error.code, PerformanceLimitErrorCode::FpsTooLow);
    assert_eq!(error.limit, 1);
    assert_eq!(error.actual, 0);

    let negative_error = PerformanceLimits::default().validate_fps(-1).unwrap_err();

    assert_eq!(negative_error.code, PerformanceLimitErrorCode::FpsTooLow);
    assert_eq!(negative_error.limit, 1);
    assert_eq!(negative_error.actual, -1);
}

#[test]
fn fps_must_not_exceed_maximum() {
    let error = PerformanceLimits::default().validate_fps(6).unwrap_err();

    assert_eq!(error.code, PerformanceLimitErrorCode::FpsTooHigh);
    assert_eq!(error.limit, 5);
    assert_eq!(error.actual, 6);
}

#[test]
fn active_tile_count_must_not_exceed_maximum() {
    let error = PerformanceLimits::default()
        .validate_active_tile_count(4)
        .unwrap_err();

    assert_eq!(
        error.code,
        PerformanceLimitErrorCode::ActiveTileLimitExceeded
    );
    assert_eq!(error.limit, 3);
    assert_eq!(error.actual, 4);
}

#[test]
fn active_tile_count_must_not_be_negative() {
    let error = PerformanceLimits::default()
        .validate_active_tile_count(-1)
        .unwrap_err();

    assert_eq!(error.code, PerformanceLimitErrorCode::ActiveTileCountTooLow);
    assert_eq!(error.limit, 0);
    assert_eq!(error.actual, -1);
}

#[test]
fn region_width_must_not_exceed_maximum() {
    let error = PerformanceLimits::default()
        .validate_region_size(RegionSize {
            width: 801,
            height: 600,
        })
        .unwrap_err();

    assert_eq!(error.code, PerformanceLimitErrorCode::RegionWidthTooLarge);
    assert_eq!(error.limit, 800);
    assert_eq!(error.actual, 801);
}

#[test]
fn region_height_must_not_exceed_maximum() {
    let error = PerformanceLimits::default()
        .validate_region_size(RegionSize {
            width: 800,
            height: 601,
        })
        .unwrap_err();

    assert_eq!(error.code, PerformanceLimitErrorCode::RegionHeightTooLarge);
    assert_eq!(error.limit, 600);
    assert_eq!(error.actual, 601);
}

#[test]
fn empty_regions_are_rejected() {
    let error = PerformanceLimits::default()
        .validate_region_size(RegionSize {
            width: 0,
            height: 300,
        })
        .unwrap_err();

    assert_eq!(error.code, PerformanceLimitErrorCode::RegionEmpty);
    assert_eq!(error.limit, 1);
    assert_eq!(error.actual, 0);

    let negative_error = PerformanceLimits::default()
        .validate_region_size(RegionSize {
            width: 600,
            height: -1,
        })
        .unwrap_err();

    assert_eq!(negative_error.code, PerformanceLimitErrorCode::RegionEmpty);
    assert_eq!(negative_error.limit, 1);
    assert_eq!(negative_error.actual, -1);
}
