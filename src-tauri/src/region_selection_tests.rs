use crate::region_selection::select_region;
use crate::region_selection_types::{
    LogicalPoint, LogicalSize, MonitorGeometry, PhysicalPoint, RegionSelectionIssueCode,
    RegionSelectionRequest, WindowCaptureTarget,
};

#[test]
fn normal_selection_maps_logical_bounds_to_physical_region() {
    let selection = select_region(request(point(10.0, 20.0), point(210.0, 170.0), monitor()))
        .expect("region selection");

    assert_eq!(selection.region.monitor_id, "main");
    assert_eq!(selection.region.x, 10);
    assert_eq!(selection.region.y, 20);
    assert_eq!(selection.region.width, 200);
    assert_eq!(selection.region.height, 150);
    assert!(selection.warnings.is_empty());
}

#[test]
fn source_window_identity_is_never_serialized_with_region_geometry() {
    let mut selection = select_region(request(point(10.0, 20.0), point(210.0, 170.0), monitor()))
        .expect("region selection");
    selection.region.source_window = Some(WindowCaptureTarget {
        window_id: 42,
        relative_x_millipoints: 10_000,
        relative_y_millipoints: 20_000,
        width_millipoints: 200_000,
        height_millipoints: 150_000,
    });

    let serialized = serde_json::to_string(&selection.region).expect("serialized region");

    assert!(!serialized.contains("window"));
    assert!(!serialized.contains("42"));
}

#[test]
fn reversed_drag_direction_is_normalized() {
    let selection = select_region(request(point(210.0, 170.0), point(10.0, 20.0), monitor()))
        .expect("region selection");

    assert_eq!(selection.region.x, 10);
    assert_eq!(selection.region.y, 20);
    assert_eq!(selection.region.width, 200);
    assert_eq!(selection.region.height, 150);
}

#[test]
fn any_non_empty_region_is_accepted() {
    let selection = select_region(request(point(10.0, 20.0), point(20.0, 42.0), monitor()))
        .expect("small region");

    assert_eq!(selection.region.width, 10);
    assert_eq!(selection.region.height, 22);
}

#[test]
fn full_display_region_is_accepted() {
    let selection = select_region(request(point(0.0, 0.0), point(1920.0, 1080.0), monitor()))
        .expect("full display region");

    assert_eq!(selection.region.width, 1920);
    assert_eq!(selection.region.height, 1080);
}

#[test]
fn out_of_range_physical_coordinates_are_rejected_before_casting() {
    let error = select_region(request(
        point(0.0, 0.0),
        point(100.0, 100.0),
        MonitorGeometry {
            id: "extreme".to_string(),
            logical_origin: point(0.0, 0.0),
            logical_size: logical_size(1_000.0, 800.0),
            physical_origin: PhysicalPoint { x: i32::MAX, y: 0 },
            scale_factor: 1.0,
        },
    ))
    .expect_err("out of range coordinate error");

    assert_eq!(
        error.code,
        RegionSelectionIssueCode::RegionCoordinateOutOfRange
    );
}

#[test]
fn scale_factor_converts_logical_selection_to_physical_pixels() {
    let selection = select_region(request(
        point(100.5, 40.5),
        point(300.25, 190.25),
        MonitorGeometry {
            id: "retina".to_string(),
            logical_origin: point(100.0, 40.0),
            logical_size: logical_size(1_000.0, 800.0),
            physical_origin: PhysicalPoint { x: 1200, y: 400 },
            scale_factor: 2.0,
        },
    ))
    .expect("scaled selection");

    assert_eq!(selection.region.x, 1201);
    assert_eq!(selection.region.y, 401);
    assert_eq!(selection.region.width, 400);
    assert_eq!(selection.region.height, 300);
}

#[test]
fn multi_monitor_offsets_are_preserved_in_physical_space() {
    let selection = select_region(request(
        point(-1180.0, 90.0),
        point(-980.0, 210.0),
        MonitorGeometry {
            id: "left-display".to_string(),
            logical_origin: point(-1280.0, 40.0),
            logical_size: logical_size(1_280.0, 540.0),
            physical_origin: PhysicalPoint { x: -2560, y: 80 },
            scale_factor: 2.0,
        },
    ))
    .expect("offset selection");

    assert_eq!(selection.region.monitor_id, "left-display");
    assert_eq!(selection.region.x, -2360);
    assert_eq!(selection.region.y, 180);
    assert_eq!(selection.region.width, 400);
    assert_eq!(selection.region.height, 240);
}

#[test]
fn selection_size_does_not_return_warnings() {
    let selection = select_region(request(point(0.0, 0.0), point(650.0, 320.0), monitor()))
        .expect("recommended warning selection");

    assert_eq!(selection.region.width, 650);
    assert_eq!(selection.region.height, 320);
    assert!(selection.warnings.is_empty());
}

#[test]
fn selection_outside_monitor_bounds_is_rejected() {
    let error = select_region(request(point(10.0, 10.0), point(1_930.0, 200.0), monitor()))
        .expect_err("outside monitor");

    assert_eq!(
        error.code,
        RegionSelectionIssueCode::SelectionOutsideMonitor
    );
}

fn request(
    start: LogicalPoint,
    end: LogicalPoint,
    monitor: MonitorGeometry,
) -> RegionSelectionRequest {
    RegionSelectionRequest {
        monitor,
        start,
        end,
    }
}

fn monitor() -> MonitorGeometry {
    MonitorGeometry {
        id: "main".to_string(),
        logical_origin: point(0.0, 0.0),
        logical_size: logical_size(1_920.0, 1_080.0),
        physical_origin: PhysicalPoint { x: 0, y: 0 },
        scale_factor: 1.0,
    }
}

fn point(x: f64, y: f64) -> LogicalPoint {
    LogicalPoint { x, y }
}

fn logical_size(width: f64, height: f64) -> LogicalSize {
    LogicalSize { width, height }
}
