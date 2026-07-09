use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};

use crate::{
    capture_backend::{cropped_frame, CroppedFramePayload},
    ocr_engine::{
        local_ocr_status, OcrConfig, OcrEngine, OcrError, OcrFrameRequest, OcrRunStatus,
        OcrService, OcrStoragePolicy, OcrTrigger,
    },
    pebble_store::PebbleStoreDocument,
    region_selection_types::PhysicalRegion,
};

#[test]
fn ocr_is_disabled_by_default() {
    let engine = FakeOcrEngine::with_texts(["should not run"]);
    let call_count = engine.call_count();
    let mut service = OcrService::new(OcrConfig::default(), engine);

    let outcome = service
        .run(request(OcrTrigger::ExplicitRequest, true, &frame()))
        .expect("disabled outcome");

    assert_eq!(outcome.status, OcrRunStatus::Disabled);
    assert!(outcome.text.is_none());
    assert_eq!(call_count.get(), 0);
    assert!(!local_ocr_status().enabled_by_default);
}

#[test]
fn ocr_adapter_can_be_faked_for_local_extraction() {
    let engine = FakeOcrEngine::with_texts(["  build failed  \n\n missing dep "]);
    let call_count = engine.call_count();
    let mut service = OcrService::new(OcrConfig { enabled: true }, engine);

    let outcome = service
        .run(request(OcrTrigger::ExplicitRequest, false, &frame()))
        .expect("ocr outcome");

    assert_eq!(outcome.status, OcrRunStatus::TextReady);
    assert_eq!(outcome.text.as_deref(), Some("build failed\nmissing dep"));
    assert_eq!(outcome.storage_policy, OcrStoragePolicy::EphemeralOnly);
    assert_eq!(call_count.get(), 1);
}

#[test]
fn ocr_does_not_run_on_scheduled_tick_or_unchanged_diff() {
    let engine = FakeOcrEngine::with_texts(["hidden"]);
    let call_count = engine.call_count();
    let mut service = OcrService::new(OcrConfig { enabled: true }, engine);
    let frame = frame();

    let scheduled = service
        .run(request(OcrTrigger::ScheduledTick, true, &frame))
        .expect("scheduled");
    let unchanged = service
        .run(request(OcrTrigger::ChangedFrame, false, &frame))
        .expect("unchanged");

    assert_eq!(scheduled.status, OcrRunStatus::NotRequested);
    assert_eq!(unchanged.status, OcrRunStatus::Unchanged);
    assert_eq!(call_count.get(), 0);
}

#[test]
fn changed_diff_runs_local_ocr() {
    let engine = FakeOcrEngine::with_texts(["new error"]);
    let mut service = OcrService::new(OcrConfig { enabled: true }, engine);

    let outcome = service
        .run(request(OcrTrigger::ChangedFrame, true, &frame()))
        .expect("changed ocr");

    assert_eq!(outcome.status, OcrRunStatus::TextReady);
    assert_eq!(outcome.text.as_deref(), Some("new error"));
}

#[test]
fn dedupe_suppresses_unchanged_text_without_storing_text() {
    let engine = FakeOcrEngine::with_texts(["same text", "same text"]);
    let mut service = OcrService::new(OcrConfig { enabled: true }, engine);
    let frame = frame();

    let first = service
        .run(request(OcrTrigger::ExplicitRequest, false, &frame))
        .expect("first ocr");
    let second = service
        .run(request(OcrTrigger::ExplicitRequest, false, &frame))
        .expect("second ocr");

    assert_eq!(first.status, OcrRunStatus::TextReady);
    assert_eq!(second.status, OcrRunStatus::Deduped);
    assert!(second.text.is_none());
    assert_eq!(service.remembered_text_count(), 0);
    assert_eq!(service.fingerprint_count(), 1);
}

#[test]
fn ocr_results_are_not_part_of_persisted_config_schema() {
    let raw = serde_json::to_string(&PebbleStoreDocument::default()).expect("store json");

    assert!(!raw.to_ascii_lowercase().contains("ocr"));
    assert!(!raw.to_ascii_lowercase().contains("observed text"));
}

#[derive(Debug)]
struct FakeOcrEngine {
    responses: RefCell<VecDeque<String>>,
    call_count: Rc<Cell<usize>>,
}

impl FakeOcrEngine {
    fn with_texts<const N: usize>(texts: [&str; N]) -> Self {
        Self {
            responses: RefCell::new(texts.into_iter().map(str::to_string).collect()),
            call_count: Rc::new(Cell::new(0)),
        }
    }

    fn call_count(&self) -> Rc<Cell<usize>> {
        Rc::clone(&self.call_count)
    }
}

impl OcrEngine for FakeOcrEngine {
    fn recognize_text(&self, _frame: &CroppedFramePayload) -> Result<String, OcrError> {
        self.call_count.set(self.call_count.get() + 1);

        Ok(self.responses.borrow_mut().pop_front().unwrap_or_default())
    }
}

fn request<'a>(
    trigger: OcrTrigger,
    diff_changed: bool,
    frame: &'a CroppedFramePayload,
) -> OcrFrameRequest<'a> {
    OcrFrameRequest {
        tile_id: "tile",
        frame,
        trigger,
        diff_changed,
    }
}

fn frame() -> CroppedFramePayload {
    let region = PhysicalRegion {
        monitor_id: "main".to_string(),
        x: 0,
        y: 0,
        width: 2,
        height: 1,
    };

    cropped_frame(&region, vec![255, 255, 255, 255, 0, 0, 0, 255])
}
