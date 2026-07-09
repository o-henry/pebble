use crate::{
    region_selector_window::{RegionSelectorWindowShell, REGION_SELECTOR_LABEL},
    window_shell::{TileMode, TileWindowShell, WindowShellState, TEST_TILE_LABEL},
};

#[test]
fn test_tile_shell_starts_closed_and_always_on_top() {
    let tile = TileWindowShell::closed();

    assert_eq!(tile.id, "test-tile");
    assert_eq!(tile.label, TEST_TILE_LABEL);
    assert_eq!(tile.mode, TileMode::Closed);
    assert!(tile.always_on_top);
    assert!(!tile.capture_active);
}

#[test]
fn window_shell_snapshot_exposes_the_test_tile() {
    let state = WindowShellState::default();
    let snapshot = state.snapshot().expect("window shell snapshot");

    assert_eq!(snapshot.test_tile.label, TEST_TILE_LABEL);
    assert_eq!(snapshot.test_tile.mode, TileMode::Closed);
    assert_eq!(snapshot.supported_modes.len(), 6);
}

#[test]
fn region_selector_shell_is_transparent_and_capture_free() {
    let shell = RegionSelectorWindowShell::transparent_overlay();

    assert_eq!(shell.label, REGION_SELECTOR_LABEL);
    assert!(shell.visual_overlay);
    assert!(shell.native_transparent);
    assert!(shell.always_on_top);
    assert!(!shell.capture_active);
}

#[test]
fn live_tile_keeps_capture_disabled_in_the_window_shell_phase() {
    let state = WindowShellState::default();
    let live = state.mark_test_tile_live().expect("live tile");

    assert_eq!(live.mode, TileMode::Live);
    assert!(!live.capture_active);

    let closed = state.mark_test_tile_closed().expect("closed tile");

    assert_eq!(closed.mode, TileMode::Closed);
    assert!(!closed.capture_active);
}

#[test]
fn close_cleanup_updates_the_test_tile_state() {
    let state = WindowShellState::default();

    state.mark_test_tile_live().expect("live tile");
    let closed = state.mark_test_tile_closed().expect("closed tile");

    assert_eq!(closed.mode, TileMode::Closed);
    assert_eq!(
        state
            .snapshot()
            .expect("window shell snapshot")
            .test_tile
            .mode,
        TileMode::Closed
    );
}
