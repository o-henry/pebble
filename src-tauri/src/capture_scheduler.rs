use std::collections::BTreeMap;

use crate::{
    capture_backend::{CaptureBackend, CaptureError, CroppedFramePayload},
    capture_lifecycle::{CaptureLifecycle, CaptureTileMode},
    region_selection_types::PhysicalRegion,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureTask {
    pub tile_id: String,
    pub mode: CaptureTileMode,
    pub region: PhysicalRegion,
    pub capture_count: u64,
    pub buffered_frame_bytes: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedFrame {
    pub tile_id: String,
    pub frame: CroppedFramePayload,
}

pub type SchedulerCaptureResult = Result<CapturedFrame, CaptureError>;

#[derive(Debug, Default)]
pub struct CaptureScheduler {
    tasks: BTreeMap<String, CaptureTask>,
}

impl CaptureScheduler {
    pub fn sync_lifecycle(&mut self, lifecycle: &CaptureLifecycle) {
        self.tasks
            .retain(|tile_id, _| lifecycle.should_keep_task(tile_id));

        for tile in lifecycle.tiles() {
            if !lifecycle.should_keep_task(&tile.id) {
                continue;
            }

            let task = self
                .tasks
                .entry(tile.id.clone())
                .or_insert_with(|| CaptureTask::new(tile.id.clone(), tile.region.clone()));
            task.mode = tile.mode;
            task.region = tile.region.clone();

            if !lifecycle.should_capture(&tile.id) {
                task.clear_buffer();
            }
        }
    }

    pub fn capture_all_once<B: CaptureBackend>(
        &mut self,
        lifecycle: &CaptureLifecycle,
        backend: &B,
    ) -> Vec<SchedulerCaptureResult> {
        self.sync_lifecycle(lifecycle);

        let mut results = Vec::new();

        for task in self.tasks.values_mut() {
            if lifecycle.should_capture(&task.tile_id) {
                results.push(capture_task_once(task, backend));
            } else {
                task.clear_buffer();
            }
        }

        results
    }

    pub fn capture_tile_once<B: CaptureBackend>(
        &mut self,
        lifecycle: &CaptureLifecycle,
        backend: &B,
        tile_id: &str,
        scale_factor: f64,
    ) -> Option<SchedulerCaptureResult> {
        self.sync_lifecycle(lifecycle);

        let task = self.tasks.get_mut(tile_id)?;
        if lifecycle.should_capture(tile_id) {
            Some(capture_task_once_at_scale(task, backend, scale_factor))
        } else {
            task.clear_buffer();
            None
        }
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn task(&self, tile_id: &str) -> Option<&CaptureTask> {
        self.tasks.get(tile_id)
    }
}

fn capture_task_once_at_scale<B: CaptureBackend>(
    task: &mut CaptureTask,
    backend: &B,
    scale_factor: f64,
) -> SchedulerCaptureResult {
    let frame = backend.capture_region_at_scale(&task.region, scale_factor)?;
    task.capture_count += 1;
    task.buffered_frame_bytes = Some(frame.bytes.len());

    Ok(CapturedFrame {
        tile_id: task.tile_id.clone(),
        frame,
    })
}

impl CaptureTask {
    fn new(tile_id: String, region: PhysicalRegion) -> Self {
        Self {
            tile_id,
            mode: CaptureTileMode::Paused,
            region,
            capture_count: 0,
            buffered_frame_bytes: None,
        }
    }

    fn clear_buffer(&mut self) {
        self.buffered_frame_bytes = None;
    }
}

fn capture_task_once<B: CaptureBackend>(
    task: &mut CaptureTask,
    backend: &B,
) -> SchedulerCaptureResult {
    let frame = backend.capture_region(&task.region)?;
    task.capture_count += 1;
    task.buffered_frame_bytes = Some(frame.bytes.len());

    Ok(CapturedFrame {
        tile_id: task.tile_id.clone(),
        frame,
    })
}
