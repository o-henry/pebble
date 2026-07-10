use crate::{
    capture_backend::CaptureErrorCode,
    capture_lifecycle::CaptureTileMode,
    live_tile::{LiveTileCaptureRequest, MAIN_LIVE_TILE_ID},
    pebble_session::{
        position_pebble_away_from_region, PebbleSessionErrorCode, PebbleSessionState,
    },
    region_selection_types::{
        LogicalPoint, LogicalSize, MonitorGeometry, PhysicalPoint, RegionSelectionRequest,
    },
};

#[test]
fn session_starts_without_selected_or_persisted_capture_data() {
    let snapshot = PebbleSessionState::default()
        .snapshot()
        .expect("session snapshot");

    assert!(snapshot.region.is_none());
    assert!(!snapshot.window_open);
    assert!(!snapshot.privacy_blank_active);
    assert_eq!(snapshot.revision, 0);
}

#[test]
fn session_accepts_a_valid_region_and_tracks_window_state() {
    let state = PebbleSessionState::default();
    let selected = state
        .select_region(selection_request(10.0, 20.0, 310.0, 200.0))
        .expect("selected region");
    let opened = state.set_window_open(true).expect("opened window");

    assert_eq!(selected.region.as_ref().expect("region").width, 300);
    assert_eq!(selected.revision, 1);
    assert!(opened.window_open);
    assert_eq!(opened.revision, 2);
}

#[test]
fn session_rejects_regions_outside_hard_limits() {
    let error = PebbleSessionState::default()
        .select_region(selection_request(0.0, 0.0, 801.0, 200.0))
        .expect_err("oversized region");

    assert_eq!(error.code, PebbleSessionErrorCode::InvalidRegion);
}

#[test]
fn privacy_blank_and_clear_are_global_session_state() {
    let state = PebbleSessionState::default();
    state
        .select_region(selection_request(0.0, 0.0, 240.0, 160.0))
        .expect("selected region");
    let blanked = state.set_privacy_blank(true).expect("privacy blank");
    let cleared = state.clear().expect("cleared session");

    assert!(blanked.privacy_blank_active);
    assert!(cleared.region.is_none());
    assert!(!cleared.window_open);
    assert!(!cleared.privacy_blank_active);
    assert!(cleared.revision > blanked.revision);
}

#[test]
fn capture_authorization_rejects_a_region_not_selected_by_the_backend() {
    let state = PebbleSessionState::default();
    let selected = state
        .select_region(selection_request(10.0, 20.0, 310.0, 200.0))
        .expect("selected region");
    state.set_window_open(true).expect("opened window");
    let mut forged_region = selected.region.expect("region");
    forged_region.x += 20;

    let error = state
        .authorize_capture(LiveTileCaptureRequest {
            request_id: "forged-request".to_string(),
            blank_generation: 0,
            tile_id: MAIN_LIVE_TILE_ID.to_string(),
            region: forged_region,
            fps: 1,
            mode: CaptureTileMode::Live,
        })
        .expect_err("forged region");

    assert_eq!(error.code, CaptureErrorCode::UnauthorizedWindow);
}

#[test]
fn pebble_window_is_placed_beside_the_selected_region_when_space_exists() {
    let region = physical_region(-1100, 100, 340, 220);
    let position = position_pebble_away_from_region(&region, -1920, -180, 1920, 1080, 1.0)
        .expect("window position");

    assert_eq!(position.logical_x, -744.0);
    assert_eq!(position.logical_y, 100.0);
}

#[test]
fn pebble_window_moves_left_when_the_region_is_near_the_right_edge() {
    let region = physical_region(1500, 100, 300, 200);
    let position =
        position_pebble_away_from_region(&region, 0, 0, 1920, 1080, 1.0).expect("window position");

    assert_eq!(position.logical_x, 1044.0);
    assert_eq!(position.logical_y, 100.0);
}

fn selection_request(start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> RegionSelectionRequest {
    RegionSelectionRequest {
        monitor: MonitorGeometry {
            id: "main".to_string(),
            logical_origin: LogicalPoint { x: 0.0, y: 0.0 },
            logical_size: LogicalSize {
                width: 1_920.0,
                height: 1_080.0,
            },
            physical_origin: PhysicalPoint { x: 0, y: 0 },
            scale_factor: 1.0,
        },
        start: LogicalPoint {
            x: start_x,
            y: start_y,
        },
        end: LogicalPoint { x: end_x, y: end_y },
    }
}

fn physical_region(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> crate::region_selection_types::PhysicalRegion {
    crate::region_selection_types::PhysicalRegion {
        monitor_id: "main".to_string(),
        x,
        y,
        width,
        height,
    }
}
