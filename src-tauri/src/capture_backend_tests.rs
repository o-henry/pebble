use crate::{
    capture_backend::{
        capture_region_once, CaptureBackend, CaptureErrorCode, FakeCaptureBackend,
        FramePixelFormat, FrameStoragePolicy,
    },
    region_selection_types::PhysicalRegion,
};

#[test]
fn fake_backend_returns_deterministic_frames() {
    let backend = FakeCaptureBackend::default();
    let region = region(10, 20, 3, 2);

    let first = backend.capture_region(&region).expect("first fake frame");
    let second = backend.capture_region(&region).expect("second fake frame");

    assert_eq!(first, second);
    assert_eq!(first.pixel_format, FramePixelFormat::Rgba8);
    assert_eq!(first.bytes_per_pixel, 4);
}

#[test]
fn out_of_bounds_region_returns_typed_error() {
    let error = capture_region_once(region(790, 590, 20, 20)).expect_err("capture error");

    assert_eq!(error.code, CaptureErrorCode::RegionOutOfBounds);
    assert_eq!(error.monitor_id, "main");
}

#[test]
fn missing_monitor_returns_typed_error() {
    let error = capture_region_once(PhysicalRegion {
        monitor_id: "missing".to_string(),
        x: 0,
        y: 0,
        width: 24,
        height: 24,
    })
    .expect_err("missing monitor error");

    assert_eq!(error.code, CaptureErrorCode::MonitorUnavailable);
    assert_eq!(error.monitor_id, "missing");
}

#[test]
fn fake_capture_payload_is_memory_only() {
    let frame = capture_region_once(region(0, 0, 24, 24)).expect("fake frame");

    assert_eq!(frame.storage_policy, FrameStoragePolicy::MemoryOnly);
    assert!(!frame.bytes.is_empty());
}

#[test]
fn payload_contains_cropped_content_only() {
    let frame = capture_region_once(region(10, 20, 3, 2)).expect("cropped frame");

    assert_eq!(frame.monitor_id, "main");
    assert_eq!(frame.width, 3);
    assert_eq!(frame.height, 2);
    assert_eq!(frame.region.width, 3);
    assert_eq!(frame.region.height, 2);
    assert_eq!(frame.bytes.len(), 3 * 2 * 4);
    assert_ne!(frame.bytes.len(), 800 * 600 * 4);
    assert_eq!(&frame.bytes[0..4], &[10, 20, 30, 255]);
    assert_eq!(&frame.bytes[20..24], &[12, 21, 33, 255]);
}

#[test]
fn invalid_region_size_returns_typed_error() {
    let error = capture_region_once(region(0, 0, 0, 24)).expect_err("invalid region");

    assert_eq!(error.code, CaptureErrorCode::InvalidRegion);
}

fn region(x: i32, y: i32, width: i32, height: i32) -> PhysicalRegion {
    PhysicalRegion {
        monitor_id: "main".to_string(),
        x,
        y,
        width,
        height,
    }
}
