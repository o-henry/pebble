use std::sync::{Arc, Mutex};

use crate::capture_backend::CroppedFramePayload;

const SAMPLE_EDGE: usize = 128;
const TILE_EDGE: usize = 8;
const PIXEL_DELTA_THRESHOLD: u8 = 24;
const MIN_CHANGED_SAMPLES: usize = 6;
const MIN_CHANGED_SAMPLES_PER_TILE: usize = 4;
const GLOBAL_MEAN_DELTA_THRESHOLD: f64 = 0.06;
const GLOBAL_CHANGED_RATIO_THRESHOLD: f64 = 0.012;
const LOCAL_CHANGED_RATIO_THRESHOLD: f64 = 0.12;
const LOCAL_MEAN_DELTA_THRESHOLD: f64 = 0.08;

#[derive(Debug, Clone, PartialEq)]
pub enum MonitoringDecision {
    Baseline,
    Stable,
    Activity,
    Changed {
        score: f64,
        kind: VisualChangeKind,
        previous_frame: CroppedFramePayload,
    },
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
    baseline: Option<VisualBaseline>,
    candidate_tiles: Option<Vec<bool>>,
    candidate_sample: Option<VisualSample>,
}

#[derive(Debug)]
struct VisualBaseline {
    sample: VisualSample,
    profile: VisualProfile,
    frame: CroppedFramePayload,
}

#[derive(Debug, Clone)]
struct VisualSample {
    width: usize,
    height: usize,
    rgb: Vec<[u8; 3]>,
}

#[derive(Debug)]
struct ChangeEvidence {
    score: f64,
    changed_tiles: Vec<bool>,
    meaningful: bool,
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
    pub fn reset(&self) {
        if let Ok(mut data) = self.data.lock() {
            *data = MonitoringData::new();
        }
    }

    pub fn observe(
        &self,
        revision: u64,
        frame: &CroppedFramePayload,
        _tick: u64,
    ) -> Option<MonitoringDecision> {
        let mut data = self.data.lock().ok()?;
        if data.revision != Some(revision) {
            *data = MonitoringData::new();
            data.revision = Some(revision);
        }
        let current_sample = VisualSample::from_frame(frame)?;
        let current_profile = VisualProfile::from_frame(frame)?;
        let Some(baseline) = data.baseline.as_ref() else {
            data.baseline = Some(VisualBaseline {
                sample: current_sample,
                profile: current_profile,
                frame: frame.clone(),
            });
            return Some(MonitoringDecision::Baseline);
        };
        let evidence = compare_samples(&baseline.sample, &current_sample)?;
        if !evidence.meaningful {
            data.candidate_tiles = None;
            data.candidate_sample = None;
            return Some(MonitoringDecision::Stable);
        }

        let stable_candidate = data
            .candidate_sample
            .as_ref()
            .and_then(|candidate| compare_samples(candidate, &current_sample))
            .is_some_and(|candidate_change| !candidate_change.meaningful);
        let same_area = data
            .candidate_tiles
            .as_ref()
            .is_some_and(|candidate| tiles_overlap(candidate, &evidence.changed_tiles));
        if !stable_candidate || !same_area {
            data.candidate_tiles = Some(evidence.changed_tiles);
            data.candidate_sample = Some(current_sample);
            return Some(MonitoringDecision::Activity);
        }

        let previous_frame = baseline.frame.clone();
        let kind = classify_change(baseline.profile, current_profile);
        data.baseline = Some(VisualBaseline {
            sample: current_sample,
            profile: current_profile,
            frame: frame.clone(),
        });
        data.candidate_tiles = None;
        data.candidate_sample = None;
        Some(MonitoringDecision::Changed {
            score: evidence.score,
            kind,
            previous_frame,
        })
    }
}

impl MonitoringData {
    fn new() -> Self {
        Self {
            revision: None,
            baseline: None,
            candidate_tiles: None,
            candidate_sample: None,
        }
    }
}

impl VisualSample {
    fn from_frame(frame: &CroppedFramePayload) -> Option<Self> {
        let width = usize::try_from(frame.width).ok()?;
        let height = usize::try_from(frame.height).ok()?;
        let expected_len = width.checked_mul(height)?.checked_mul(4)?;
        if width == 0 || height == 0 || frame.bytes.len() != expected_len {
            return None;
        }

        let sample_width = width.min(SAMPLE_EDGE);
        let sample_height = height.min(SAMPLE_EDGE);
        let mut rgb = Vec::with_capacity(sample_width.checked_mul(sample_height)?);
        for sample_y in 0..sample_height {
            let source_y = sample_y * height / sample_height;
            for sample_x in 0..sample_width {
                let source_x = sample_x * width / sample_width;
                let index = (source_y * width + source_x) * 4;
                rgb.push([
                    frame.bytes[index],
                    frame.bytes[index + 1],
                    frame.bytes[index + 2],
                ]);
            }
        }
        Some(Self {
            width: sample_width,
            height: sample_height,
            rgb,
        })
    }
}

fn compare_samples(baseline: &VisualSample, current: &VisualSample) -> Option<ChangeEvidence> {
    if baseline.width != current.width
        || baseline.height != current.height
        || baseline.rgb.len() != current.rgb.len()
        || baseline.rgb.is_empty()
    {
        return None;
    }

    let tiles_x = baseline.width.div_ceil(TILE_EDGE);
    let tiles_y = baseline.height.div_ceil(TILE_EDGE);
    let tile_count = tiles_x.checked_mul(tiles_y)?;
    let mut tile_pixels = vec![0_usize; tile_count];
    let mut tile_changed = vec![0_usize; tile_count];
    let mut tile_delta = vec![0_u64; tile_count];
    let mut changed_samples = 0_usize;
    let mut total_delta = 0_u64;

    for (index, (before, after)) in baseline.rgb.iter().zip(&current.rgb).enumerate() {
        let delta = before
            .iter()
            .zip(after.iter())
            .map(|(before, after)| before.abs_diff(*after))
            .max()?;
        let x = index % baseline.width;
        let y = index / baseline.width;
        let tile = (y / TILE_EDGE) * tiles_x + (x / TILE_EDGE);
        tile_pixels[tile] += 1;
        tile_delta[tile] += u64::from(delta);
        total_delta += u64::from(delta);
        if delta >= PIXEL_DELTA_THRESHOLD {
            changed_samples += 1;
            tile_changed[tile] += 1;
        }
    }

    let sample_count = baseline.rgb.len() as f64;
    let mean_delta = total_delta as f64 / (sample_count * 255.0);
    let changed_ratio = changed_samples as f64 / sample_count;
    let mut max_tile_changed_ratio = 0.0_f64;
    let mut max_tile_mean_delta = 0.0_f64;
    let changed_tiles = tile_pixels
        .iter()
        .zip(tile_changed.iter())
        .zip(tile_delta.iter())
        .map(|((pixels, changed), delta)| {
            let changed_ratio = *changed as f64 / *pixels as f64;
            let mean_delta = *delta as f64 / (*pixels as f64 * 255.0);
            max_tile_changed_ratio = max_tile_changed_ratio.max(changed_ratio);
            max_tile_mean_delta = max_tile_mean_delta.max(mean_delta);
            *changed >= MIN_CHANGED_SAMPLES_PER_TILE
                && (changed_ratio >= LOCAL_CHANGED_RATIO_THRESHOLD
                    || mean_delta >= LOCAL_MEAN_DELTA_THRESHOLD)
        })
        .collect::<Vec<_>>();

    let meaningful = changed_samples >= MIN_CHANGED_SAMPLES
        && (mean_delta >= GLOBAL_MEAN_DELTA_THRESHOLD
            || changed_ratio >= GLOBAL_CHANGED_RATIO_THRESHOLD
            || max_tile_changed_ratio >= LOCAL_CHANGED_RATIO_THRESHOLD
            || max_tile_mean_delta >= LOCAL_MEAN_DELTA_THRESHOLD);
    Some(ChangeEvidence {
        score: mean_delta
            .max(changed_ratio)
            .max(max_tile_changed_ratio)
            .max(max_tile_mean_delta),
        changed_tiles,
        meaningful,
    })
}

fn tiles_overlap(previous: &[bool], current: &[bool]) -> bool {
    previous.len() == current.len()
        && previous
            .iter()
            .zip(current)
            .any(|(previous, current)| *previous && *current)
}

impl VisualProfile {
    fn from_frame(frame: &CroppedFramePayload) -> Option<Self> {
        let width = usize::try_from(frame.width).ok()?;
        let height = usize::try_from(frame.height).ok()?;
        let expected_len = width.checked_mul(height)?.checked_mul(4)?;
        if width == 0 || height == 0 || frame.bytes.len() != expected_len {
            return None;
        }

        let sample_width = width.min(SAMPLE_EDGE);
        let sample_height = height.min(SAMPLE_EDGE);
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
    fn local_detection_keeps_collecting_material_changes_without_ai_cooldown() {
        let state = MonitoringState::default();
        assert_eq!(
            state.observe(1, &frame(0), 1),
            Some(MonitoringDecision::Baseline)
        );
        assert_eq!(
            state.observe(1, &frame(1), 2),
            Some(MonitoringDecision::Stable)
        );
        assert_eq!(
            state.observe(1, &frame(255), 3),
            Some(MonitoringDecision::Activity)
        );
        assert!(matches!(
            state.observe(1, &frame(255), 4),
            Some(MonitoringDecision::Changed { .. })
        ));
        assert_eq!(
            state.observe(1, &frame(0), 5),
            Some(MonitoringDecision::Activity)
        );
        assert!(matches!(
            state.observe(1, &frame(0), 6),
            Some(MonitoringDecision::Changed { .. })
        ));
    }

    #[test]
    fn localized_text_sized_change_triggers_below_global_average_threshold() {
        let state = MonitoringState::default();
        let baseline = frame(0);
        let changed = frame_with_patch(0, 255, 16, 16, 8, 8);
        state.observe(1, &baseline, 1);

        assert_eq!(
            state.observe(1, &changed, 2),
            Some(MonitoringDecision::Activity)
        );
        let decision = state.observe(1, &changed, 3);
        assert!(matches!(
            &decision,
            Some(MonitoringDecision::Changed { .. })
        ));
        let Some(MonitoringDecision::Changed { score, .. }) = decision else {
            unreachable!();
        };
        assert!(score >= 0.99);
    }

    #[test]
    fn cumulative_change_uses_the_stable_baseline_instead_of_each_previous_frame() {
        let state = MonitoringState::default();
        let baseline = frame(0);
        let small_change = frame_with_patch(0, 255, 16, 16, 4, 4);
        let larger_change = frame_with_patch(0, 255, 16, 16, 8, 8);
        state.observe(1, &baseline, 1);

        assert_eq!(
            state.observe(1, &small_change, 2),
            Some(MonitoringDecision::Activity)
        );
        assert_eq!(
            state.observe(1, &larger_change, 3),
            Some(MonitoringDecision::Activity)
        );
        assert!(matches!(
            state.observe(1, &larger_change, 4),
            Some(MonitoringDecision::Changed { previous_frame, .. })
                if previous_frame == baseline
        ));
    }

    #[test]
    fn moving_animation_is_ignored_until_the_new_content_settles() {
        let state = MonitoringState::default();
        let baseline = frame(0);
        let first = frame_with_patch(0, 255, 8, 16, 8, 8);
        let second = frame_with_patch(0, 255, 16, 16, 8, 8);
        let third = frame_with_patch(0, 255, 24, 16, 8, 8);
        state.observe(1, &baseline, 1);

        assert_eq!(
            state.observe(1, &first, 2),
            Some(MonitoringDecision::Activity)
        );
        assert_eq!(
            state.observe(1, &second, 3),
            Some(MonitoringDecision::Activity)
        );
        assert_eq!(
            state.observe(1, &third, 4),
            Some(MonitoringDecision::Activity)
        );
        assert!(matches!(
            state.observe(1, &third, 5),
            Some(MonitoringDecision::Changed { previous_frame, .. })
                if previous_frame == baseline
        ));
    }

    #[test]
    fn one_poll_transient_noise_does_not_trigger_analysis() {
        let state = MonitoringState::default();
        let baseline = frame(0);
        let transient = frame_with_patch(0, 255, 16, 16, 8, 8);
        state.observe(1, &baseline, 1);

        assert_eq!(
            state.observe(1, &transient, 2),
            Some(MonitoringDecision::Activity)
        );
        assert_eq!(
            state.observe(1, &baseline, 3),
            Some(MonitoringDecision::Stable)
        );
        assert_eq!(
            state.observe(1, &baseline, 4),
            Some(MonitoringDecision::Stable)
        );
    }

    #[test]
    fn equal_luma_color_change_is_detected_from_rgb_content() {
        let state = MonitoringState::default();
        let red = rgba_frame([255, 0, 0, 255]);
        let green_with_similar_luma = rgba_frame([0, 131, 0, 255]);
        state.observe(1, &red, 1);

        assert_eq!(
            state.observe(1, &green_with_similar_luma, 2),
            Some(MonitoringDecision::Activity)
        );
        assert!(matches!(
            state.observe(1, &green_with_similar_luma, 3),
            Some(MonitoringDecision::Changed { .. })
        ));
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
    fn resetting_watch_discards_the_previous_baseline() {
        let state = MonitoringState::default();
        state.observe(1, &frame(0), 1);
        state.reset();

        assert_eq!(
            state.observe(1, &frame(255), 2),
            Some(MonitoringDecision::Baseline)
        );
    }

    #[test]
    fn classifies_local_brightness_and_warning_color_changes() {
        let brightness = MonitoringState::default();
        brightness.observe(1, &rgba_frame([0, 0, 0, 255]), 1);
        assert_eq!(
            brightness.observe(1, &rgba_frame([255, 255, 255, 255]), 2),
            Some(MonitoringDecision::Activity)
        );
        assert!(matches!(
            brightness.observe(1, &rgba_frame([255, 255, 255, 255]), 3),
            Some(MonitoringDecision::Changed {
                kind: VisualChangeKind::Brighter,
                ..
            })
        ));

        let warning = MonitoringState::default();
        warning.observe(2, &rgba_frame([255, 255, 255, 255]), 1);
        assert_eq!(
            warning.observe(2, &rgba_frame([220, 70, 40, 255]), 2),
            Some(MonitoringDecision::Activity)
        );
        assert!(matches!(
            warning.observe(2, &rgba_frame([220, 70, 40, 255]), 3),
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
                source_window: None,
            },
            width: 64,
            height: 64,
            pixel_format: FramePixelFormat::Rgba8,
            bytes_per_pixel: 4,
            storage_policy: FrameStoragePolicy::MemoryOnly,
            bytes: pixel.into_iter().cycle().take(64 * 64 * 4).collect(),
        }
    }

    fn frame_with_patch(
        background: u8,
        foreground: u8,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> CroppedFramePayload {
        let mut frame = frame(background);
        for pixel_y in y..y + height {
            for pixel_x in x..x + width {
                let index = (pixel_y * 64 + pixel_x) * 4;
                frame.bytes[index..index + 3].fill(foreground);
            }
        }
        frame
    }
}
