use crate::{
    capture_backend::{CaptureErrorCode, FakeCaptureBackend},
    capture_lifecycle::CaptureTileMode,
    live_tile::{
        clamp_live_tile_fps, AuthorizedLiveTileCapture, LiveTileCaptureRequest, LiveTileService,
    },
    region_selection_types::PhysicalRegion,
};

#[test]
fn live_tile_keeps_only_latest_frame_event() {
    let mut service = LiveTileService::new(FakeCaptureBackend::default());

    let first = service
        .capture_once(request(CaptureTileMode::Live, region(10, 20, 24, 24), 1))
        .expect("first frame")
        .frame_event
        .expect("first event");
    let second = service
        .capture_once(request(CaptureTileMode::Live, region(30, 20, 12, 12), 1))
        .expect("second frame")
        .frame_event
        .expect("second event");

    assert_eq!(first.sequence, 1);
    assert_eq!(second.sequence, 2);
    assert_eq!(service.latest_frame_count(), 1);
    assert_eq!(
        service.latest_frame("tile").expect("latest frame").sequence,
        second.sequence
    );
    assert_eq!(second.frame.bytes.len(), 12 * 12 * 4);
}

#[test]
fn pause_stops_live_tile_scheduler_capture() {
    let mut service = LiveTileService::new(FakeCaptureBackend::default());

    let live = service
        .capture_once(request(CaptureTileMode::Live, region(10, 20, 24, 24), 1))
        .expect("live frame");
    let paused = service
        .capture_once(request(CaptureTileMode::Paused, region(10, 20, 24, 24), 1))
        .expect("paused capture");

    assert!(live.response.capture_active);
    assert!(live.frame_event.is_some());
    assert!(!paused.response.capture_active);
    assert!(paused.frame_event.is_none());
    assert_eq!(service.task_count(), 1);
}

#[test]
fn close_removes_live_tile_task_and_latest_frame() {
    let mut service = LiveTileService::new(FakeCaptureBackend::default());

    service
        .capture_once(request(CaptureTileMode::Live, region(10, 20, 24, 24), 1))
        .expect("live frame");
    let closed = service
        .capture_once(request(CaptureTileMode::Closed, region(10, 20, 24, 24), 1))
        .expect("closed tile");

    assert!(!closed.response.capture_active);
    assert!(closed.frame_event.is_none());
    assert_eq!(service.task_count(), 0);
    assert_eq!(service.latest_frame_count(), 0);
}

#[test]
fn native_window_cleanup_removes_live_tile_task_and_latest_frame() {
    let mut service = LiveTileService::new(FakeCaptureBackend::default());

    service
        .capture_once(request(CaptureTileMode::Live, region(10, 20, 24, 24), 1))
        .expect("live frame");
    service.close_tile("tile", 2);

    assert_eq!(service.task_count(), 0);
    assert_eq!(service.latest_frame_count(), 0);
}

#[test]
fn stale_frame_can_be_discarded_before_delivery() {
    let mut service = LiveTileService::new(FakeCaptureBackend::default());

    service
        .capture_once(request(CaptureTileMode::Live, region(10, 20, 24, 24), 1))
        .expect("live frame");
    service.discard_frame("tile");

    assert_eq!(service.latest_frame_count(), 0);
}

#[test]
fn privacy_blank_stops_capture_and_drops_latest_frame() {
    let mut service = LiveTileService::new(FakeCaptureBackend::default());

    service
        .capture_once(request(CaptureTileMode::Live, region(10, 20, 24, 24), 1))
        .expect("live frame");
    service.set_privacy_blank(true, 2);

    assert_eq!(service.task_count(), 1);
    assert_eq!(service.latest_frame_count(), 0);
}

#[test]
fn stale_live_request_after_blank_does_not_capture() {
    let mut service = LiveTileService::new(FakeCaptureBackend::default());
    let stale_live = request_with_revision(CaptureTileMode::Live, region(10, 20, 24, 24), 1, 1);

    service
        .capture_once(request(CaptureTileMode::Live, region(10, 20, 24, 24), 1))
        .expect("live frame");
    service.set_privacy_blank(true, 2);
    service.set_privacy_blank(false, 3);
    let error = service
        .capture_once(stale_live)
        .expect_err("stale live request");

    assert_eq!(error.code, CaptureErrorCode::CaptureUnavailable);
    assert_eq!(service.latest_frame_count(), 0);
}

#[test]
fn late_request_after_close_cannot_recreate_capture_state() {
    let mut service = LiveTileService::new(FakeCaptureBackend::default());
    let late_request = request_with_revision(CaptureTileMode::Live, region(10, 20, 24, 24), 1, 1);

    service
        .capture_once(request(CaptureTileMode::Live, region(10, 20, 24, 24), 1))
        .expect("live frame");
    service.close_tile("tile", 2);
    let error = service
        .capture_once(late_request)
        .expect_err("late request");

    assert_eq!(error.code, CaptureErrorCode::CaptureUnavailable);
    assert_eq!(service.task_count(), 0);
    assert_eq!(service.latest_frame_count(), 0);
}

#[test]
fn fourth_live_tile_is_rejected_by_backend_limit() {
    let mut service = LiveTileService::new(FakeCaptureBackend::default());

    for index in 0..3 {
        service
            .capture_once(request_for_tile(
                &format!("tile-{index}"),
                CaptureTileMode::Live,
                region(10, 20, 24, 24),
                1,
            ))
            .expect("allowed live tile");
    }

    let error = service
        .capture_once(request_for_tile(
            "tile-4",
            CaptureTileMode::Live,
            region(10, 20, 24, 24),
            1,
        ))
        .expect_err("active tile limit");

    assert_eq!(error.code, CaptureErrorCode::ActiveTileLimitExceeded);
}

#[test]
fn live_tile_fps_is_clamped_to_backend_limits() {
    let mut service = LiveTileService::new(FakeCaptureBackend::default());
    let low = service
        .capture_once(request(CaptureTileMode::Paused, region(10, 20, 24, 24), 0))
        .expect("low fps");
    let high = service
        .capture_once(request(CaptureTileMode::Paused, region(10, 20, 24, 24), 99))
        .expect("high fps");

    assert_eq!(clamp_live_tile_fps(0), 1);
    assert_eq!(clamp_live_tile_fps(99), 5);
    assert_eq!(low.response.effective_fps, 1);
    assert_eq!(high.response.effective_fps, 5);
}

fn request(mode: CaptureTileMode, region: PhysicalRegion, fps: i32) -> AuthorizedLiveTileCapture {
    request_with_revision(mode, region, fps, 1)
}

fn request_with_revision(
    mode: CaptureTileMode,
    region: PhysicalRegion,
    fps: i32,
    session_revision: u64,
) -> AuthorizedLiveTileCapture {
    AuthorizedLiveTileCapture::new(
        raw_request_for_tile("tile", mode, region, fps),
        1.0,
        session_revision,
    )
}

fn request_for_tile(
    tile_id: &str,
    mode: CaptureTileMode,
    region: PhysicalRegion,
    fps: i32,
) -> AuthorizedLiveTileCapture {
    AuthorizedLiveTileCapture::new(raw_request_for_tile(tile_id, mode, region, fps), 1.0, 1)
}

fn raw_request_for_tile(
    tile_id: &str,
    mode: CaptureTileMode,
    region: PhysicalRegion,
    fps: i32,
) -> LiveTileCaptureRequest {
    LiveTileCaptureRequest {
        request_id: format!("{tile_id}-request"),
        blank_generation: 0,
        tile_id: tile_id.to_string(),
        region,
        fps,
        mode,
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
