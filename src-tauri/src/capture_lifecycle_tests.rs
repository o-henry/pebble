use std::cell::Cell;

use crate::{
    capture_backend::{CaptureBackend, CaptureResult, FakeCaptureBackend},
    capture_lifecycle::{CaptureLifecycle, CaptureTileMode},
    capture_scheduler::CaptureScheduler,
    region_selection_types::PhysicalRegion,
};

#[test]
fn capture_starts_only_when_live_and_visible() {
    let backend = CountingBackend::default();
    let mut lifecycle = lifecycle_with_mode(CaptureTileMode::Live);
    let mut scheduler = CaptureScheduler::default();

    let results = scheduler.capture_all_once(&lifecycle, &backend);

    assert_eq!(results.len(), 1);
    assert!(results[0].as_ref().expect("capture result").frame.width > 0);
    assert_eq!(backend.calls(), 1);
    assert_eq!(scheduler.task_count(), 1);
    assert_eq!(
        scheduler.task("tile").expect("task").buffered_frame_bytes,
        Some(24 * 24 * 4)
    );

    lifecycle.transition("tile", CaptureTileMode::Paused);
    let paused_results = scheduler.capture_all_once(&lifecycle, &backend);

    assert!(paused_results.is_empty());
    assert_eq!(backend.calls(), 1);
}

#[test]
fn paused_hidden_and_blanked_tiles_do_not_capture() {
    for mode in [
        CaptureTileMode::Paused,
        CaptureTileMode::Hidden,
        CaptureTileMode::Blanked,
    ] {
        let backend = CountingBackend::default();
        let lifecycle = lifecycle_with_mode(mode);
        let mut scheduler = CaptureScheduler::default();

        let results = scheduler.capture_all_once(&lifecycle, &backend);

        assert!(results.is_empty());
        assert_eq!(backend.calls(), 0);
        assert_eq!(scheduler.task_count(), 1);
        assert!(scheduler
            .task("tile")
            .expect("task")
            .buffered_frame_bytes
            .is_none());
    }
}

#[test]
fn privacy_blank_stops_all_capture_and_clears_buffers() {
    let backend = CountingBackend::default();
    let mut lifecycle = lifecycle_with_mode(CaptureTileMode::Live);
    let mut scheduler = CaptureScheduler::default();

    assert_eq!(scheduler.capture_all_once(&lifecycle, &backend).len(), 1);
    assert!(scheduler
        .task("tile")
        .expect("task")
        .buffered_frame_bytes
        .is_some());

    lifecycle.blank_all();
    let results = scheduler.capture_all_once(&lifecycle, &backend);

    assert!(results.is_empty());
    assert_eq!(backend.calls(), 1);
    assert!(lifecycle.privacy_blank_active());
    assert_eq!(lifecycle.tile_mode("tile"), Some(CaptureTileMode::Blanked));
    assert!(scheduler
        .task("tile")
        .expect("task")
        .buffered_frame_bytes
        .is_none());
}

#[test]
fn blank_changes_lifecycle_state_for_visible_tiles() {
    let mut lifecycle = CaptureLifecycle::default();
    lifecycle.upsert_tile("live", region(), CaptureTileMode::Live);
    lifecycle.upsert_tile("paused", region(), CaptureTileMode::Paused);
    lifecycle.upsert_tile("hidden", region(), CaptureTileMode::Hidden);
    lifecycle.upsert_tile("closed", region(), CaptureTileMode::Closed);

    lifecycle.blank_all();

    assert!(lifecycle.privacy_blank_active());
    assert_eq!(lifecycle.tile_mode("live"), Some(CaptureTileMode::Blanked));
    assert_eq!(
        lifecycle.tile_mode("paused"),
        Some(CaptureTileMode::Blanked)
    );
    assert_eq!(
        lifecycle.tile_mode("hidden"),
        Some(CaptureTileMode::Blanked)
    );
    assert_eq!(lifecycle.tile_mode("closed"), Some(CaptureTileMode::Closed));
}

#[test]
fn restore_restarts_only_previously_active_tiles() {
    let backend = CountingBackend::default();
    let mut lifecycle = CaptureLifecycle::default();
    let mut scheduler = CaptureScheduler::default();
    lifecycle.upsert_tile("live", region(), CaptureTileMode::Live);
    lifecycle.upsert_tile("paused", region(), CaptureTileMode::Paused);
    lifecycle.upsert_tile("hidden", region(), CaptureTileMode::Hidden);

    assert_eq!(scheduler.capture_all_once(&lifecycle, &backend).len(), 1);
    lifecycle.blank_all();
    assert!(scheduler.capture_all_once(&lifecycle, &backend).is_empty());
    lifecycle.restore_after_blank();
    let restored = scheduler.capture_all_once(&lifecycle, &backend);

    assert!(!lifecycle.privacy_blank_active());
    assert_eq!(lifecycle.tile_mode("live"), Some(CaptureTileMode::Live));
    assert_eq!(lifecycle.tile_mode("paused"), Some(CaptureTileMode::Paused));
    assert_eq!(lifecycle.tile_mode("hidden"), Some(CaptureTileMode::Hidden));
    assert_eq!(restored.len(), 1);
    assert_eq!(
        restored[0].as_ref().expect("restored capture").tile_id,
        "live"
    );
    assert_eq!(backend.calls(), 2);
}

#[test]
fn tile_added_during_blank_does_not_restore_as_live() {
    let backend = CountingBackend::default();
    let mut lifecycle = lifecycle_with_mode(CaptureTileMode::Live);
    let mut scheduler = CaptureScheduler::default();

    assert_eq!(scheduler.capture_all_once(&lifecycle, &backend).len(), 1);
    lifecycle.blank_all();
    lifecycle.upsert_tile("new", region(), CaptureTileMode::Live);

    assert_eq!(lifecycle.tile_mode("new"), Some(CaptureTileMode::Blanked));
    assert!(scheduler.capture_all_once(&lifecycle, &backend).is_empty());

    lifecycle.restore_after_blank();
    let restored = scheduler.capture_all_once(&lifecycle, &backend);

    assert_eq!(lifecycle.tile_mode("tile"), Some(CaptureTileMode::Live));
    assert_eq!(lifecycle.tile_mode("new"), Some(CaptureTileMode::Paused));
    assert_eq!(restored.len(), 1);
    assert_eq!(
        restored[0].as_ref().expect("restored capture").tile_id,
        "tile"
    );
    assert_eq!(backend.calls(), 2);
}

#[test]
fn close_and_delete_remove_tasks_and_buffers() {
    let backend = CountingBackend::default();
    let mut lifecycle = lifecycle_with_mode(CaptureTileMode::Live);
    let mut scheduler = CaptureScheduler::default();

    scheduler.capture_all_once(&lifecycle, &backend);
    lifecycle.transition("tile", CaptureTileMode::Closed);
    scheduler.sync_lifecycle(&lifecycle);

    assert_eq!(scheduler.task_count(), 0);
    assert!(scheduler.task("tile").is_none());

    lifecycle.upsert_tile("tile", region(), CaptureTileMode::Live);
    scheduler.capture_all_once(&lifecycle, &backend);
    lifecycle.transition("tile", CaptureTileMode::Deleted);
    scheduler.sync_lifecycle(&lifecycle);

    assert_eq!(scheduler.task_count(), 0);
    assert!(scheduler.task("tile").is_none());
}

#[test]
fn repeated_pause_resume_does_not_leak_tasks() {
    let backend = CountingBackend::default();
    let mut lifecycle = lifecycle_with_mode(CaptureTileMode::Live);
    let mut scheduler = CaptureScheduler::default();

    for _ in 0..3 {
        assert_eq!(scheduler.capture_all_once(&lifecycle, &backend).len(), 1);
        assert_eq!(scheduler.task_count(), 1);

        lifecycle.transition("tile", CaptureTileMode::Paused);
        assert!(scheduler.capture_all_once(&lifecycle, &backend).is_empty());
        assert_eq!(scheduler.task_count(), 1);

        lifecycle.transition("tile", CaptureTileMode::Live);
        scheduler.sync_lifecycle(&lifecycle);
        assert_eq!(scheduler.task_count(), 1);
    }

    let task = scheduler.task("tile").expect("task");
    assert_eq!(task.capture_count, 3);
    assert_eq!(backend.calls(), 3);
}

fn lifecycle_with_mode(mode: CaptureTileMode) -> CaptureLifecycle {
    let mut lifecycle = CaptureLifecycle::default();
    lifecycle.upsert_tile("tile", region(), mode);
    lifecycle
}

fn region() -> PhysicalRegion {
    PhysicalRegion {
        monitor_id: "main".to_string(),
        x: 10,
        y: 20,
        width: 24,
        height: 24,
    }
}

struct CountingBackend {
    inner: FakeCaptureBackend,
    calls: Cell<usize>,
}

impl Default for CountingBackend {
    fn default() -> Self {
        Self {
            inner: FakeCaptureBackend::default(),
            calls: Cell::new(0),
        }
    }
}

impl CountingBackend {
    fn calls(&self) -> usize {
        self.calls.get()
    }
}

impl CaptureBackend for CountingBackend {
    fn capture_region(&self, region: &PhysicalRegion) -> CaptureResult {
        self.calls.set(self.calls.get() + 1);
        self.inner.capture_region(region)
    }
}
