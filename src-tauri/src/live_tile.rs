use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::{
    capture_backend::{
        capture_error, CaptureBackend, CaptureError, CaptureErrorCode, CroppedFramePayload,
        FakeCaptureBackend,
    },
    capture_lifecycle::{CaptureLifecycle, CaptureTileMode},
    capture_scheduler::CaptureScheduler,
    performance_limits::PerformanceLimits,
    region_selection_types::PhysicalRegion,
};

const FRAME_UPDATED_EVENT: &str = "pebble://frame-updated";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveTileCaptureRequest {
    pub request_id: String,
    pub blank_generation: u64,
    pub tile_id: String,
    pub region: PhysicalRegion,
    pub fps: i32,
    pub mode: CaptureTileMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveTileCaptureResponse {
    pub request_id: String,
    pub blank_generation: u64,
    pub tile_id: String,
    pub mode: CaptureTileMode,
    pub effective_fps: i32,
    pub capture_active: bool,
    pub frame_sequence: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveTileFrameEvent {
    pub event_name: &'static str,
    pub request_id: String,
    pub tile_id: String,
    pub sequence: u64,
    pub frame: CroppedFramePayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveTileCaptureOutcome {
    pub response: LiveTileCaptureResponse,
    pub frame_event: Option<LiveTileFrameEvent>,
}

#[derive(Debug)]
pub struct LiveTileService<B> {
    backend: B,
    lifecycle: CaptureLifecycle,
    scheduler: CaptureScheduler,
    latest_frames: BTreeMap<String, LiveTileFrameEvent>,
    privacy_blank_generation: u64,
    next_sequence: u64,
}

#[derive(Debug, Clone)]
pub struct LiveTileState {
    service: Arc<Mutex<LiveTileService<FakeCaptureBackend>>>,
}

impl Default for LiveTileState {
    fn default() -> Self {
        Self {
            service: Arc::new(Mutex::new(LiveTileService::new(
                FakeCaptureBackend::default(),
            ))),
        }
    }
}

impl LiveTileState {
    pub fn capture_once(
        &self,
        request: LiveTileCaptureRequest,
    ) -> Result<LiveTileCaptureOutcome, CaptureError> {
        self.service
            .lock()
            .expect("live tile state lock")
            .capture_once(request)
    }
}

impl<B: CaptureBackend> LiveTileService<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            lifecycle: CaptureLifecycle::default(),
            scheduler: CaptureScheduler::default(),
            latest_frames: BTreeMap::new(),
            privacy_blank_generation: 0,
            next_sequence: 1,
        }
    }

    pub fn capture_once(
        &mut self,
        request: LiveTileCaptureRequest,
    ) -> Result<LiveTileCaptureOutcome, CaptureError> {
        let tile_id = request.tile_id.clone();
        if request.mode == CaptureTileMode::Blanked {
            self.privacy_blank_generation = self.privacy_blank_generation.saturating_add(1);
        }
        let effective_mode = self.effective_mode(request.mode, request.blank_generation);
        self.validate_active_tile_limit(&tile_id, effective_mode)?;
        let effective_fps = clamp_live_tile_fps(request.fps);
        self.lifecycle
            .upsert_tile(tile_id.clone(), request.region, effective_mode);
        let frame_event =
            self.capture_frame_event(&request.request_id, &tile_id, effective_mode)?;
        let capture_active = self.lifecycle.should_capture(&tile_id);

        if should_drop_latest_frame(effective_mode) {
            self.latest_frames.remove(&tile_id);
        }

        let response = LiveTileCaptureResponse {
            request_id: request.request_id,
            blank_generation: self.privacy_blank_generation,
            tile_id,
            mode: effective_mode,
            effective_fps,
            capture_active,
            frame_sequence: frame_event.as_ref().map(|event| event.sequence),
        };

        Ok(LiveTileCaptureOutcome {
            response,
            frame_event,
        })
    }

    pub fn task_count(&self) -> usize {
        self.scheduler.task_count()
    }

    pub fn latest_frame_count(&self) -> usize {
        self.latest_frames.len()
    }

    pub fn latest_frame(&self, tile_id: &str) -> Option<&LiveTileFrameEvent> {
        self.latest_frames.get(tile_id)
    }

    fn capture_frame_event(
        &mut self,
        request_id: &str,
        tile_id: &str,
        mode: CaptureTileMode,
    ) -> Result<Option<LiveTileFrameEvent>, CaptureError> {
        if mode != CaptureTileMode::Live {
            self.scheduler.sync_lifecycle(&self.lifecycle);
            return Ok(None);
        }

        self.scheduler
            .capture_tile_once(&self.lifecycle, &self.backend, tile_id)
            .transpose()
            .map(|captured| {
                captured.map(|captured| {
                    self.next_frame_event(request_id.to_string(), captured.tile_id, captured.frame)
                })
            })
    }

    fn next_frame_event(
        &mut self,
        request_id: String,
        tile_id: String,
        frame: CroppedFramePayload,
    ) -> LiveTileFrameEvent {
        let event = LiveTileFrameEvent {
            event_name: FRAME_UPDATED_EVENT,
            request_id,
            tile_id,
            sequence: self.next_sequence,
            frame,
        };
        self.next_sequence += 1;
        self.latest_frames
            .insert(event.tile_id.clone(), event.clone());
        event
    }

    fn validate_active_tile_limit(
        &self,
        tile_id: &str,
        mode: CaptureTileMode,
    ) -> Result<(), CaptureError> {
        if mode != CaptureTileMode::Live {
            return Ok(());
        }

        let active_count = self
            .lifecycle
            .tiles()
            .filter(|tile| tile.id != tile_id && tile.mode == CaptureTileMode::Live)
            .count()
            + 1;
        let limit = PerformanceLimits::default().max_active_tiles as usize;

        if active_count > limit {
            return Err(capture_error(
                CaptureErrorCode::ActiveTileLimitExceeded,
                tile_id,
                format!("Active live tile limit exceeded: limit {limit}, actual {active_count}."),
            ));
        }

        Ok(())
    }

    fn effective_mode(
        &self,
        requested_mode: CaptureTileMode,
        request_blank_generation: u64,
    ) -> CaptureTileMode {
        if requested_mode == CaptureTileMode::Live
            && request_blank_generation < self.privacy_blank_generation
        {
            CaptureTileMode::Blanked
        } else {
            requested_mode
        }
    }
}

pub fn live_tile_frame_event_name() -> &'static str {
    FRAME_UPDATED_EVENT
}

pub fn clamp_live_tile_fps(fps: i32) -> i32 {
    let limits = PerformanceLimits::default();

    fps.clamp(1, limits.max_fps)
}

fn should_drop_latest_frame(mode: CaptureTileMode) -> bool {
    matches!(
        mode,
        CaptureTileMode::Blanked | CaptureTileMode::Closed | CaptureTileMode::Deleted
    )
}
