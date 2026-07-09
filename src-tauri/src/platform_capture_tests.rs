use crate::{
    capture_backend::{CaptureErrorCode, RGBA_BYTES_PER_PIXEL},
    platform_capture::{capture_real_region_once, platform_capture_test_support},
    region_selection_types::PhysicalRegion,
};

#[test]
fn platform_backend_rejects_invalid_region_before_capture() {
    let error = platform_capture_test_support::invalid_region_error();

    assert_eq!(error.code, CaptureErrorCode::InvalidRegion);
}

#[test]
fn platform_backend_rejects_oversized_region_before_capture() {
    let error = platform_capture_test_support::oversized_region_error();

    assert_eq!(error.code, CaptureErrorCode::RegionTooLarge);
}

#[test]
fn real_capture_adapter_maps_invalid_region_to_recoverable_error() {
    let error = capture_real_region_once(region(0, 0, 0, 24)).expect_err("invalid region");

    assert_eq!(error.code, CaptureErrorCode::InvalidRegion);
    assert_eq!(error.monitor_id, "main");
}

#[cfg(target_os = "macos")]
#[test]
fn macos_capture_rect_uses_selected_region_dimensions() {
    let rect = platform_capture_test_support::capture_rect(&region(12, 34, 56, 78));

    assert_eq!(rect, (12.0, 34.0, 56.0, 78.0));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_bitmap_info_must_be_supported_bgra_layout() {
    let byte_order_32_little = 2 << 12;
    let premultiplied_first = 2;
    let premultiplied_last = 1;
    let byte_order_32_big = 4 << 12;

    assert!(
        platform_capture_test_support::is_supported_bgra_bitmap_info(
            byte_order_32_little | premultiplied_first
        )
    );
    assert!(
        !platform_capture_test_support::is_supported_bgra_bitmap_info(
            byte_order_32_big | premultiplied_first
        )
    );
    assert!(
        !platform_capture_test_support::is_supported_bgra_bitmap_info(
            byte_order_32_little | premultiplied_last
        )
    );
}

#[cfg(target_os = "macos")]
#[test]
fn macos_pixel_copy_emits_cropped_rgba_rows_only() {
    let bytes_per_row = 12;
    let source = [
        1, 2, 3, 255, 4, 5, 6, 255, 99, 99, 99, 99, 7, 8, 9, 255, 10, 11, 12, 255, 88, 88, 88, 88,
    ];
    let expected_len = 2 * 2 * RGBA_BYTES_PER_PIXEL;

    let rgba = platform_capture_test_support::copy_bgra_rows_to_rgba(
        &source,
        bytes_per_row,
        2,
        2,
        expected_len,
    )
    .expect("cropped rgba bytes");

    assert_eq!(rgba.len(), expected_len);
    assert_eq!(
        rgba,
        vec![3, 2, 1, 255, 6, 5, 4, 255, 9, 8, 7, 255, 12, 11, 10, 255]
    );
}

#[test]
fn platform_capture_source_does_not_use_file_backed_capture() {
    let shared_source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/platform_capture.rs"
    ));
    let macos_source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/platform_capture/platform_capture_macos.rs"
    ));
    let macos_sys_source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/platform_capture/platform_capture_macos_sys.rs"
    ));

    for forbidden in [
        "std::fs",
        "File::create",
        "write_all",
        "tempfile",
        "screencapture",
    ] {
        assert!(
            !shared_source.contains(forbidden)
                && !macos_source.contains(forbidden)
                && !macos_sys_source.contains(forbidden),
            "platform capture must not use file-backed capture: {forbidden}"
        );
    }
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
