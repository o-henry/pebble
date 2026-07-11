use std::sync::{Arc, Mutex};

use crate::{
    capture_backend::CroppedFramePayload,
    diff_engine::{DiffEngine, DiffEngineConfig},
};

const CHANGE_THRESHOLD: f64 = 0.06;
const AI_COOLDOWN_TICKS: u64 = 300;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MonitoringDecision {
    Baseline,
    Changed { score: f64 },
}

#[derive(Debug, Clone)]
pub struct MonitoringState {
    data: Arc<Mutex<MonitoringData>>,
}

#[derive(Debug)]
struct MonitoringData {
    revision: Option<u64>,
    engine: DiffEngine,
    baseline_sent: bool,
}

impl Default for MonitoringState {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(MonitoringData::new())),
        }
    }
}

impl MonitoringState {
    pub fn observe(
        &self,
        revision: u64,
        frame: &CroppedFramePayload,
        tick: u64,
    ) -> Option<MonitoringDecision> {
        let mut data = self.data.lock().ok()?;
        if data.revision != Some(revision) {
            *data = MonitoringData::new();
            data.revision = Some(revision);
        }
        let observation = data
            .engine
            .observe_frame("active-region", frame, tick)
            .ok()?;
        if !data.baseline_sent {
            data.baseline_sent = true;
            return Some(MonitoringDecision::Baseline);
        }
        observation.changed.then_some(MonitoringDecision::Changed {
            score: observation.score,
        })
    }
}

impl MonitoringData {
    fn new() -> Self {
        let config = DiffEngineConfig {
            sample_width: 64,
            sample_height: 64,
            change_threshold: CHANGE_THRESHOLD,
            cooldown_ticks: AI_COOLDOWN_TICKS,
        };
        Self {
            revision: None,
            engine: DiffEngine::new(config).expect("valid monitoring diff configuration"),
            baseline_sent: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MonitoringDecision, MonitoringState};
    use crate::{
        capture_backend::{CroppedFramePayload, FramePixelFormat, FrameStoragePolicy},
        region_selection_types::PhysicalRegion,
    };

    #[test]
    fn sends_one_baseline_then_only_material_changes() {
        let state = MonitoringState::default();
        assert_eq!(
            state.observe(1, &frame(0), 1),
            Some(MonitoringDecision::Baseline)
        );
        assert_eq!(state.observe(1, &frame(1), 2), None);
        assert!(matches!(
            state.observe(1, &frame(255), 3),
            Some(MonitoringDecision::Changed { .. })
        ));
        assert_eq!(state.observe(1, &frame(0), 4), None);
    }

    #[test]
    fn new_session_gets_a_new_baseline() {
        let state = MonitoringState::default();
        assert_eq!(
            state.observe(1, &frame(0), 1),
            Some(MonitoringDecision::Baseline)
        );
        assert_eq!(
            state.observe(2, &frame(0), 2),
            Some(MonitoringDecision::Baseline)
        );
    }

    fn frame(value: u8) -> CroppedFramePayload {
        CroppedFramePayload {
            monitor_id: "main".into(),
            region: PhysicalRegion {
                monitor_id: "main".into(),
                x: 0,
                y: 0,
                width: 64,
                height: 64,
            },
            width: 64,
            height: 64,
            pixel_format: FramePixelFormat::Rgba8,
            bytes_per_pixel: 4,
            storage_policy: FrameStoragePolicy::MemoryOnly,
            bytes: vec![value; 64 * 64 * 4],
        }
    }
}
