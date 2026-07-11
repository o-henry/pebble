use std::sync::{Arc, Mutex};

use crate::{
    capture_backend::CroppedFramePayload,
    diff_engine::{DiffEngine, DiffEngineConfig},
};

const CHANGE_THRESHOLD: f64 = 0.06;
const CHANGE_COOLDOWN_TICKS: u64 = 300;
const PROFILE_SAMPLE_EDGE: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MonitoringDecision {
    Baseline,
    Changed { score: f64, kind: VisualChangeKind },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualChangeKind {
    Brighter,
    Darker,
    WarningColor,
    PositiveColor,
    Material,
}

impl VisualChangeKind {
    pub fn summary(self) -> &'static str {
        match self {
            Self::Brighter => "THE REGION BECAME MARKEDLY BRIGHTER",
            Self::Darker => "THE REGION BECAME MARKEDLY DARKER",
            Self::WarningColor => "RED OR AMBER CONTENT INCREASED",
            Self::PositiveColor => "GREEN CONTENT INCREASED",
            Self::Material => "MATERIAL VISUAL CHANGE DETECTED",
        }
    }
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
    previous_profile: Option<VisualProfile>,
}

#[derive(Debug, Clone, Copy)]
struct VisualProfile {
    mean_luma: f64,
    warning_ratio: f64,
    positive_ratio: f64,
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
        let current_profile = VisualProfile::from_frame(frame)?;
        let previous_profile = data.previous_profile.replace(current_profile);
        if !data.baseline_sent {
            data.baseline_sent = true;
            return Some(MonitoringDecision::Baseline);
        }
        observation.changed.then(|| MonitoringDecision::Changed {
            score: observation.score,
            kind: previous_profile
                .map(|previous| classify_change(previous, current_profile))
                .unwrap_or(VisualChangeKind::Material),
        })
    }
}

impl MonitoringData {
    fn new() -> Self {
        let config = DiffEngineConfig {
            sample_width: 64,
            sample_height: 64,
            change_threshold: CHANGE_THRESHOLD,
            cooldown_ticks: CHANGE_COOLDOWN_TICKS,
        };
        Self {
            revision: None,
            engine: DiffEngine::new(config).expect("valid monitoring diff configuration"),
            baseline_sent: false,
            previous_profile: None,
        }
    }
}

impl VisualProfile {
    fn from_frame(frame: &CroppedFramePayload) -> Option<Self> {
        let width = usize::try_from(frame.width).ok()?;
        let height = usize::try_from(frame.height).ok()?;
        let expected_len = width.checked_mul(height)?.checked_mul(4)?;
        if width == 0 || height == 0 || frame.bytes.len() != expected_len {
            return None;
        }

        let sample_width = width.min(PROFILE_SAMPLE_EDGE);
        let sample_height = height.min(PROFILE_SAMPLE_EDGE);
        let pixel_count = sample_width.checked_mul(sample_height)?;
        let mut luma_total = 0_u64;
        let mut warning = 0_usize;
        let mut positive = 0_usize;
        for sample_y in 0..sample_height {
            let source_y = sample_y * height / sample_height;
            for sample_x in 0..sample_width {
                let source_x = sample_x * width / sample_width;
                let index = (source_y * width + source_x) * 4;
                let red = frame.bytes[index];
                let green = frame.bytes[index + 1];
                let blue = frame.bytes[index + 2];
                luma_total += u64::from(
                    (77 * u16::from(red) + 150 * u16::from(green) + 29 * u16::from(blue)) / 256,
                );
                warning += usize::from(
                    red >= 150 && red > green.saturating_add(35) && red > blue.saturating_add(25),
                );
                positive += usize::from(
                    green >= 120
                        && green > red.saturating_add(30)
                        && green > blue.saturating_add(20),
                );
            }
        }

        Some(Self {
            mean_luma: luma_total as f64 / (pixel_count as f64 * 255.0),
            warning_ratio: warning as f64 / pixel_count as f64,
            positive_ratio: positive as f64 / pixel_count as f64,
        })
    }
}

fn classify_change(previous: VisualProfile, current: VisualProfile) -> VisualChangeKind {
    if current.warning_ratio - previous.warning_ratio >= 0.12 {
        VisualChangeKind::WarningColor
    } else if current.positive_ratio - previous.positive_ratio >= 0.12 {
        VisualChangeKind::PositiveColor
    } else if current.mean_luma - previous.mean_luma >= 0.18 {
        VisualChangeKind::Brighter
    } else if previous.mean_luma - current.mean_luma >= 0.18 {
        VisualChangeKind::Darker
    } else {
        VisualChangeKind::Material
    }
}

#[cfg(test)]
mod tests {
    use super::{MonitoringDecision, MonitoringState, VisualChangeKind};
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

    #[test]
    fn classifies_local_brightness_and_warning_color_changes() {
        let brightness = MonitoringState::default();
        brightness.observe(1, &rgba_frame([0, 0, 0, 255]), 1);
        assert!(matches!(
            brightness.observe(1, &rgba_frame([255, 255, 255, 255]), 2),
            Some(MonitoringDecision::Changed {
                kind: VisualChangeKind::Brighter,
                ..
            })
        ));

        let warning = MonitoringState::default();
        warning.observe(2, &rgba_frame([255, 255, 255, 255]), 1);
        assert!(matches!(
            warning.observe(2, &rgba_frame([220, 70, 40, 255]), 2),
            Some(MonitoringDecision::Changed {
                kind: VisualChangeKind::WarningColor,
                ..
            })
        ));
    }

    fn frame(value: u8) -> CroppedFramePayload {
        rgba_frame([value, value, value, 255])
    }

    fn rgba_frame(pixel: [u8; 4]) -> CroppedFramePayload {
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
            bytes: pixel.into_iter().cycle().take(64 * 64 * 4).collect(),
        }
    }
}
