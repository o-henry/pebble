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
fn platform_backend_accepts_large_positive_regions() {
    assert!(platform_capture_test_support::large_region_is_valid());
}

#[test]
fn real_capture_adapter_maps_invalid_region_to_recoverable_error() {
    let error = capture_real_region_once(region(0, 0, 0, 24)).expect_err("invalid region");

    assert_eq!(error.code, CaptureErrorCode::InvalidRegion);
    assert_eq!(error.monitor_id, "main");
}

#[cfg(target_os = "macos")]
#[test]
fn macos_capture_never_falls_back_to_the_current_screen_coordinates() {
    let error = capture_real_region_once(region(10, 20, 300, 180))
        .expect_err("unbound source window must fail closed");

    assert_eq!(error.code, CaptureErrorCode::CaptureUnavailable);
    assert!(error.message.contains("not pinned"));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_capture_rect_uses_selected_region_dimensions() {
    let rect = platform_capture_test_support::capture_rect(&region(12, 34, 56, 78), 1.0);

    assert_eq!(rect, (12.0, 34.0, 56.0, 78.0));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_capture_rect_converts_retina_pixels_to_screen_points() {
    let rect = platform_capture_test_support::capture_rect(&region(12, 34, 56, 78), 2.0);

    assert_eq!(rect, (6.0, 17.0, 28.0, 39.0));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_backdrop_sample_tracks_the_window_center_in_screen_points() {
    let rect = platform_capture_test_support::backdrop_rect(3_456.0, 200.0, 440.0, 340.0);

    assert_eq!(rect, (3_612.0, 306.0, 128.0, 128.0));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_backdrop_capture_reads_only_windows_below_pebble() {
    let (below_window, including_window) = platform_capture_test_support::window_list_options();

    assert_eq!(below_window, 1 << 2);
    assert_eq!(including_window, 1 << 3);
}

#[cfg(target_os = "macos")]
#[test]
fn macos_source_binding_skips_a_small_overlay_and_pins_the_containing_window() {
    let source = platform_capture_test_support::select_source_window(
        (100.0, 100.0, 300.0, 180.0),
        55,
        &[
            (10, 77, 0, 180.0, 140.0, 80.0, 80.0),
            (20, 88, 0, 20.0, 20.0, 900.0, 700.0),
        ],
    );

    assert_eq!(source, Some(20));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_source_binding_ignores_pebble_and_nonstandard_window_layers() {
    let source = platform_capture_test_support::select_source_window(
        (100.0, 100.0, 300.0, 180.0),
        55,
        &[
            (10, 55, 0, 20.0, 20.0, 900.0, 700.0),
            (20, 88, 2, 20.0, 20.0, 900.0, 700.0),
            (30, 99, 0, 40.0, 40.0, 800.0, 600.0),
        ],
    );

    assert_eq!(source, Some(30));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_source_binding_rejects_regions_that_span_multiple_windows() {
    let source = platform_capture_test_support::select_source_window(
        (100.0, 100.0, 500.0, 300.0),
        55,
        &[
            (10, 77, 0, 100.0, 100.0, 250.0, 300.0),
            (20, 88, 0, 350.0, 100.0, 250.0, 300.0),
        ],
    );

    assert_eq!(source, None);
}

#[cfg(target_os = "macos")]
#[test]
fn macos_capture_rejects_changed_source_window_identity() {
    assert!(
        platform_capture_test_support::source_window_identity_matches(
            42,
            700,
            (900.0, 700.0),
            42,
            700,
            (900.0, 700.0)
        )
    );
    assert!(
        !platform_capture_test_support::source_window_identity_matches(
            42,
            700,
            (900.0, 700.0),
            42,
            701,
            (900.0, 700.0)
        )
    );
    assert!(
        !platform_capture_test_support::source_window_identity_matches(
            42,
            700,
            (900.0, 700.0),
            43,
            700,
            (900.0, 700.0)
        )
    );
    assert!(
        !platform_capture_test_support::source_window_identity_matches(
            42,
            700,
            (900.0, 700.0),
            42,
            700,
            (800.0, 700.0)
        )
    );
}

#[cfg(target_os = "macos")]
#[test]
fn macos_source_binding_requires_the_same_frontmost_window_before_retaining() {
    let original = (42, 700, 100.0, 80.0, 900.0, 700.0);

    assert!(platform_capture_test_support::same_window_info(
        original, original
    ));
    assert!(!platform_capture_test_support::same_window_info(
        original,
        (43, 700, 100.0, 80.0, 900.0, 700.0)
    ));
    assert!(!platform_capture_test_support::same_window_info(
        original,
        (42, 700, 130.0, 80.0, 900.0, 700.0)
    ));
}

#[test]
fn capture_bindings_do_not_expose_input_injection() {
    let bindings = include_str!("platform_capture/platform_capture_macos_sys.rs");
    let manifest = include_str!("../Cargo.toml");

    for forbidden in [
        "CGEventPost",
        "CGEventCreateKeyboardEvent",
        "AXUIElementPerformAction",
    ] {
        assert!(!bindings.contains(forbidden));
    }
    for forbidden_dependency in ["enigo", "rdev", "autopilot"] {
        assert!(!manifest.contains(forbidden_dependency));
    }
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
        "Command::new(\"screencapture\")",
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
        source_window: None,
    }
}
