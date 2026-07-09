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

    lifecycle.set_privacy_blank(true);
    let results = scheduler.capture_all_once(&lifecycle, &backend);

    assert!(results.is_empty());
    assert_eq!(backend.calls(), 1);
    assert!(scheduler
        .task("tile")
        .expect("task")
        .buffered_frame_bytes
        .is_none());
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
