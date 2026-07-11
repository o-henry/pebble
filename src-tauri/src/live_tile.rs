use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::{
    capture_backend::{
        capture_error, CaptureBackend, CaptureError, CaptureErrorCode, CroppedFramePayload,
    },
    capture_lifecycle::{CaptureLifecycle, CaptureTileMode},
    capture_scheduler::CaptureScheduler,
    performance_limits::PerformanceLimits,
    platform_capture::PlatformCaptureBackend,
    region_selection_types::PhysicalRegion,
};

const FRAME_UPDATED_EVENT: &str = "pebble://frame-updated";
const PREVIEW_MAX_WIDTH: usize = 960;
const PREVIEW_MAX_HEIGHT: usize = 540;
pub const MAIN_LIVE_TILE_ID: &str = "main-live-tile";

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

#[derive(Debug, Clone)]
pub struct AuthorizedLiveTileCapture {
    request: LiveTileCaptureRequest,
    scale_factor: f64,
    session_revision: u64,
}

impl AuthorizedLiveTileCapture {
    pub(crate) fn new(
        request: LiveTileCaptureRequest,
        scale_factor: f64,
        session_revision: u64,
    ) -> Self {
        Self {
            request,
            scale_factor,
            session_revision,
        }
    }

    pub(crate) fn session_revision(&self) -> u64 {
        self.session_revision
    }
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
    blocked_through_revision: BTreeMap<String, u64>,
    privacy_blank_generation: u64,
    next_sequence: u64,
}

#[derive(Debug, Clone)]
pub struct LiveTileState {
    service: Arc<Mutex<LiveTileService<PlatformCaptureBackend>>>,
}

impl Default for LiveTileState {
    fn default() -> Self {
        Self {
            service: Arc::new(Mutex::new(LiveTileService::new(PlatformCaptureBackend))),
        }
    }
}

impl LiveTileState {
    pub fn capture_once(
        &self,
        request: AuthorizedLiveTileCapture,
    ) -> Result<LiveTileCaptureOutcome, CaptureError> {
        self.service
            .lock()
            .expect("live tile state lock")
            .capture_once(request)
    }

    pub fn close_tile(&self, tile_id: &str, session_revision: u64) {
        self.service
            .lock()
            .expect("live tile state lock")
            .close_tile(tile_id, session_revision);
    }

    pub fn set_privacy_blank(&self, active: bool, session_revision: u64) {
        self.service
            .lock()
            .expect("live tile state lock")
            .set_privacy_blank(active, session_revision);
    }

    pub fn discard_frame(&self, tile_id: &str) {
        self.service
            .lock()
            .expect("live tile state lock")
            .discard_frame(tile_id);
    }
}

impl<B: CaptureBackend> LiveTileService<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            lifecycle: CaptureLifecycle::default(),
            scheduler: CaptureScheduler::default(),
            latest_frames: BTreeMap::new(),
            blocked_through_revision: BTreeMap::new(),
            privacy_blank_generation: 0,
            next_sequence: 1,
        }
    }

    pub fn capture_once(
        &mut self,
        authorized: AuthorizedLiveTileCapture,
    ) -> Result<LiveTileCaptureOutcome, CaptureError> {
        let AuthorizedLiveTileCapture {
            request,
            scale_factor,
            session_revision,
        } = authorized;
        let tile_id = request.tile_id.clone();
        if self.request_is_blocked(&tile_id, session_revision) {
            return Err(capture_error(
                CaptureErrorCode::CaptureUnavailable,
                &tile_id,
                "Capture request belongs to an inactive Pebble session.",
            ));
        }
        let effective_mode = self.effective_mode(request.mode);
        self.validate_active_tile_limit(&tile_id, effective_mode)?;
        let effective_fps = clamp_live_tile_fps(request.fps);
        self.lifecycle
            .upsert_tile(tile_id.clone(), request.region, effective_mode);
        let frame_event =
            self.capture_frame_event(&request.request_id, &tile_id, effective_mode, scale_factor)?;
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

    pub fn close_tile(&mut self, tile_id: &str, session_revision: u64) {
        self.lifecycle.transition(tile_id, CaptureTileMode::Closed);
        self.scheduler.sync_lifecycle(&self.lifecycle);
        self.latest_frames.remove(tile_id);
        self.block_through(tile_id, session_revision);
    }

    pub fn set_privacy_blank(&mut self, active: bool, session_revision: u64) {
        if active {
            self.privacy_blank_generation = self.privacy_blank_generation.saturating_add(1);
            self.lifecycle.blank_all();
            let tile_ids = self
                .lifecycle
                .tiles()
                .map(|tile| tile.id.clone())
                .collect::<Vec<_>>();
            for tile_id in tile_ids {
                self.block_through(&tile_id, session_revision);
            }
            self.block_through(MAIN_LIVE_TILE_ID, session_revision);
        } else {
            self.lifecycle.restore_after_blank();
        }

        self.scheduler.sync_lifecycle(&self.lifecycle);
        self.latest_frames.clear();
    }

    pub fn discard_frame(&mut self, tile_id: &str) {
        self.latest_frames.remove(tile_id);
    }

    fn capture_frame_event(
        &mut self,
        request_id: &str,
        tile_id: &str,
        mode: CaptureTileMode,
        scale_factor: f64,
    ) -> Result<Option<LiveTileFrameEvent>, CaptureError> {
        if mode != CaptureTileMode::Live {
            self.scheduler.sync_lifecycle(&self.lifecycle);
            return Ok(None);
        }

        self.scheduler
            .capture_tile_once(&self.lifecycle, &self.backend, tile_id, scale_factor)
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
        let frame = preview_frame(frame);
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

    fn effective_mode(&self, requested_mode: CaptureTileMode) -> CaptureTileMode {
        if self.lifecycle.privacy_blank_active() {
            CaptureTileMode::Blanked
        } else {
            requested_mode
        }
    }

    fn request_is_blocked(&self, tile_id: &str, session_revision: u64) -> bool {
        self.blocked_through_revision
            .get(tile_id)
            .is_some_and(|blocked| session_revision <= *blocked)
    }

    fn block_through(&mut self, tile_id: &str, session_revision: u64) {
        self.blocked_through_revision
            .entry(tile_id.to_string())
            .and_modify(|blocked| *blocked = (*blocked).max(session_revision))
            .or_insert(session_revision);
    }
}

fn preview_frame(mut frame: CroppedFramePayload) -> CroppedFramePayload {
    let Ok(source_width) = usize::try_from(frame.width) else {
        return frame;
    };
    let Ok(source_height) = usize::try_from(frame.height) else {
        return frame;
    };
    if source_width <= PREVIEW_MAX_WIDTH && source_height <= PREVIEW_MAX_HEIGHT {
        return frame;
    }
    let scale = (PREVIEW_MAX_WIDTH as f64 / source_width as f64)
        .min(PREVIEW_MAX_HEIGHT as f64 / source_height as f64);
    let target_width = ((source_width as f64 * scale).round() as usize).max(1);
    let target_height = ((source_height as f64 * scale).round() as usize).max(1);
    let mut bytes = Vec::with_capacity(target_width * target_height * 4);
    for y in 0..target_height {
        let source_y = y * source_height / target_height;
        for x in 0..target_width {
            let source_x = x * source_width / target_width;
            let offset = (source_y * source_width + source_x) * 4;
            bytes.extend_from_slice(&frame.bytes[offset..offset + 4]);
        }
    }
    frame.width = i32::try_from(target_width).unwrap_or(i32::MAX);
    frame.height = i32::try_from(target_height).unwrap_or(i32::MAX);
    frame.bytes = bytes;
    frame
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
