use crate::{
    capture_backend::{capture_region_once, CroppedFramePayload},
    diff_engine::{
        downsample_to_grayscale, mean_absolute_difference, DiffEngine, DiffEngineConfig,
    },
    region_selection_types::PhysicalRegion,
};

#[test]
fn identical_frames_score_zero() {
    let config = DiffEngineConfig::default();
    let frame = solid_frame(24, 24, 80);
    let previous = downsample_to_grayscale(&frame, &config).expect("previous frame");
    let current = downsample_to_grayscale(&frame, &config).expect("current frame");

    assert_eq!(
        mean_absolute_difference(&previous, &current).expect("diff score"),
        0.0
    );
}

#[test]
fn small_changes_stay_below_default_threshold() {
    let mut engine = DiffEngine::default();

    engine
        .observe_frame("tile", &solid_frame(64, 64, 40), 0)
        .expect("initial frame");
    let observation = engine
        .observe_frame("tile", &solid_frame(64, 64, 50), 1)
        .expect("small change observation");

    assert!(observation.score > 0.0);
    assert!(!observation.changed);
    assert!(observation.event.is_none());
}

#[test]
fn large_changes_cross_default_threshold() {
    let mut engine = DiffEngine::default();

    engine
        .observe_frame("tile", &solid_frame(64, 64, 0), 0)
        .expect("initial frame");
    let observation = engine
        .observe_frame("tile", &solid_frame(64, 64, 255), 1)
        .expect("large change observation");

    assert!(observation.score >= DiffEngineConfig::default().change_threshold);
    assert!(observation.changed);
    assert_eq!(observation.event.expect("changed event").tile_id, "tile");
}

#[test]
fn changed_event_can_be_created_from_fake_frames() {
    let mut engine = DiffEngine::default();
    let first = capture_region_once(region(0, 0, 64, 64)).expect("first fake frame");
    let second = capture_region_once(region(192, 0, 64, 64)).expect("second fake frame");

    engine
        .observe_frame("tile", &first, 0)
        .expect("initial fake frame");
    let observation = engine
        .observe_frame("tile", &second, 1)
        .expect("fake frame diff");

    assert!(observation.changed);
    assert!(observation.event.is_some());
}

#[test]
fn cooldown_suppresses_repeated_alerts() {
    let mut engine = DiffEngine::default();

    engine
        .observe_frame("tile", &solid_frame(64, 64, 0), 0)
        .expect("initial frame");
    assert!(
        engine
            .observe_frame("tile", &solid_frame(64, 64, 255), 1)
            .expect("first change")
            .changed
    );
    let suppressed = engine
        .observe_frame("tile", &solid_frame(64, 64, 0), 2)
        .expect("cooldown change");

    assert!(suppressed.score >= DiffEngineConfig::default().change_threshold);
    assert!(!suppressed.changed);
    assert!(suppressed.event.is_none());

    let emitted = engine
        .observe_frame("tile", &solid_frame(64, 64, 255), 4)
        .expect("post cooldown change");
    assert!(emitted.changed);
}

#[test]
fn only_one_previous_small_frame_is_retained_per_tile() {
    let mut engine = DiffEngine::default();

    for tick in 0..5 {
        engine
            .observe_frame("tile", &solid_frame(64, 64, tick as u8 * 20), tick)
            .expect("observed frame");
    }

    assert_eq!(engine.tracked_tile_count(), 1);
    assert_eq!(
        engine.previous_sample_len("tile"),
        Some(DiffEngineConfig::default().sample_width * DiffEngineConfig::default().sample_height)
    );
    assert_ne!(engine.previous_sample_len("tile"), Some(64 * 64 * 4));
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

fn solid_frame(width: i32, height: i32, value: u8) -> CroppedFramePayload {
    let mut bytes = Vec::with_capacity(width as usize * height as usize * 4);
    for _ in 0..(width * height) {
        bytes.extend_from_slice(&[value, value, value, 255]);
    }

    CroppedFramePayload {
        monitor_id: "test".to_string(),
        region: PhysicalRegion {
            monitor_id: "test".to_string(),
            x: 0,
            y: 0,
            width,
            height,
        },
        width,
        height,
        pixel_format: crate::capture_backend::FramePixelFormat::Rgba8,
        bytes_per_pixel: 4,
        storage_policy: crate::capture_backend::FrameStoragePolicy::MemoryOnly,
        bytes,
    }
}
