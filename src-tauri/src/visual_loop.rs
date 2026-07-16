use std::collections::VecDeque;

use crate::capture_backend::{CroppedFramePayload, FramePixelFormat, RGBA_BYTES_PER_PIXEL};

const GRID_SIDE: usize = 8;
const CELL_COUNT: usize = GRID_SIDE * GRID_SIDE;
const SAMPLES_PER_AXIS: usize = 4;
const HISTORY_LIMIT: usize = 12;
const MIN_PERIOD: usize = 2;
const MAX_PERIOD: usize = 4;
const REQUIRED_CYCLES: usize = 3;
const MAX_NORMALIZED_DISTANCE: f64 = 0.08;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct VisualFingerprint {
    cells: [u8; CELL_COUNT],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VisualLoopPattern {
    pub period: usize,
}

#[derive(Debug, Default)]
pub(crate) struct VisualLoopDetector {
    history: VecDeque<VisualFingerprint>,
    last_alert: Option<Vec<VisualFingerprint>>,
}

impl VisualFingerprint {
    pub(crate) fn from_frame(frame: &CroppedFramePayload) -> Option<Self> {
        if frame.pixel_format != FramePixelFormat::Rgba8
            || frame.bytes_per_pixel != RGBA_BYTES_PER_PIXEL as i32
        {
            return None;
        }
        let width = usize::try_from(frame.width).ok()?;
        let height = usize::try_from(frame.height).ok()?;
        if width == 0 || height == 0 {
            return None;
        }
        let expected = width
            .checked_mul(height)?
            .checked_mul(RGBA_BYTES_PER_PIXEL)?;
        if frame.bytes.len() != expected {
            return None;
        }

        let mut cells = [0_u8; CELL_COUNT];
        for cell_y in 0..GRID_SIDE {
            for cell_x in 0..GRID_SIDE {
                let mut red = 0_u32;
                let mut green = 0_u32;
                let mut blue = 0_u32;
                let mut samples = 0_u32;
                let mut minimum_luma = u8::MAX;
                let mut maximum_luma = u8::MIN;
                for sample_y in 0..SAMPLES_PER_AXIS {
                    let y = sample_coordinate(cell_y, sample_y, height);
                    for sample_x in 0..SAMPLES_PER_AXIS {
                        let x = sample_coordinate(cell_x, sample_x, width);
                        let offset = (y * width + x) * RGBA_BYTES_PER_PIXEL;
                        red += u32::from(frame.bytes[offset]);
                        green += u32::from(frame.bytes[offset + 1]);
                        blue += u32::from(frame.bytes[offset + 2]);
                        let luma = approximate_luma(
                            frame.bytes[offset],
                            frame.bytes[offset + 1],
                            frame.bytes[offset + 2],
                        );
                        minimum_luma = minimum_luma.min(luma);
                        maximum_luma = maximum_luma.max(luma);
                        samples += 1;
                    }
                }
                let red = (red / samples) as u8 >> 6;
                let green = (green / samples) as u8 >> 6;
                let blue = (blue / samples) as u8 >> 6;
                let texture = maximum_luma.saturating_sub(minimum_luma) >> 6;
                cells[cell_y * GRID_SIDE + cell_x] =
                    (texture << 6) | (red << 4) | (green << 2) | blue;
            }
        }
        Some(Self { cells })
    }

    fn similar_to(&self, other: &Self) -> bool {
        let distance = self
            .cells
            .iter()
            .zip(other.cells.iter())
            .map(|(left, right)| {
                channel_distance(*left >> 6, *right >> 6)
                    + channel_distance((*left >> 4) & 0b11, (*right >> 4) & 0b11)
                    + channel_distance((*left >> 2) & 0b11, (*right >> 2) & 0b11)
                    + channel_distance(*left & 0b11, *right & 0b11)
            })
            .sum::<u32>();
        distance as f64 / (CELL_COUNT as f64 * 12.0) <= MAX_NORMALIZED_DISTANCE
    }
}

impl VisualLoopDetector {
    pub(crate) fn reset(&mut self) {
        self.history.clear();
        self.last_alert = None;
    }

    pub(crate) fn seed(&mut self, fingerprint: VisualFingerprint) {
        self.reset();
        self.history.push_back(fingerprint);
    }

    pub(crate) fn observe(&mut self, fingerprint: VisualFingerprint) -> Option<VisualLoopPattern> {
        if self
            .history
            .back()
            .is_some_and(|previous| previous.similar_to(&fingerprint))
        {
            return None;
        }
        self.history.push_back(fingerprint);
        while self.history.len() > HISTORY_LIMIT {
            self.history.pop_front();
        }

        let Some((period, pattern)) = detect_pattern(&self.history) else {
            self.last_alert = None;
            return None;
        };
        let pattern = canonical_pattern(pattern);
        if self.last_alert.as_ref() == Some(&pattern) {
            return None;
        }
        self.last_alert = Some(pattern);
        Some(VisualLoopPattern { period })
    }
}

fn sample_coordinate(cell: usize, sample: usize, length: usize) -> usize {
    let numerator = (cell * SAMPLES_PER_AXIS + sample) * length;
    let denominator = GRID_SIDE * SAMPLES_PER_AXIS;
    (numerator / denominator).min(length - 1)
}

fn channel_distance(left: u8, right: u8) -> u32 {
    u32::from(left.abs_diff(right))
}

fn approximate_luma(red: u8, green: u8, blue: u8) -> u8 {
    ((u32::from(red) * 54 + u32::from(green) * 183 + u32::from(blue) * 19) >> 8) as u8
}

fn detect_pattern(
    history: &VecDeque<VisualFingerprint>,
) -> Option<(usize, Vec<VisualFingerprint>)> {
    for period in MIN_PERIOD..=MAX_PERIOD {
        let required = period * REQUIRED_CYCLES;
        if history.len() < required {
            continue;
        }
        let tail = history
            .iter()
            .skip(history.len() - required)
            .collect::<Vec<_>>();
        let first_cycle = &tail[..period];
        let distinct_states = first_cycle.iter().enumerate().all(|(index, state)| {
            first_cycle
                .iter()
                .skip(index + 1)
                .all(|other| !state.similar_to(other))
        });
        if !distinct_states {
            continue;
        }
        let repeated = tail
            .iter()
            .enumerate()
            .all(|(index, state)| state.similar_to(first_cycle[index % period]));
        if repeated {
            return Some((period, first_cycle.iter().cloned().cloned().collect()));
        }
    }
    None
}

fn canonical_pattern(pattern: Vec<VisualFingerprint>) -> Vec<VisualFingerprint> {
    (0..pattern.len())
        .map(|offset| {
            pattern[offset..]
                .iter()
                .chain(pattern[..offset].iter())
                .cloned()
                .collect::<Vec<_>>()
        })
        .min()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::{capture_backend::cropped_frame, region_selection_types::PhysicalRegion};

    use super::{VisualFingerprint, VisualLoopDetector};

    #[test]
    fn fingerprint_is_compact_and_rejects_invalid_frame_bytes() {
        let mut frame = solid_frame([20, 30, 40, 255]);
        let fingerprint = VisualFingerprint::from_frame(&frame).expect("fingerprint");
        assert_eq!(std::mem::size_of_val(&fingerprint), 64);
        frame.bytes.pop();
        assert!(VisualFingerprint::from_frame(&frame).is_none());
    }

    #[test]
    fn fingerprint_distinguishes_texture_with_a_similar_average_color() {
        let solid = patterned_frame(32, |_, _| [128, 128, 128, 255]);
        let checker = patterned_frame(32, |x, y| {
            if (x + y) % 2 == 0 {
                [0, 0, 0, 255]
            } else {
                [255, 255, 255, 255]
            }
        });
        let solid = VisualFingerprint::from_frame(&solid).expect("solid fingerprint");
        let checker = VisualFingerprint::from_frame(&checker).expect("checker fingerprint");
        assert!(!solid.similar_to(&checker));
    }

    #[test]
    fn two_state_cycle_alerts_once_after_three_complete_cycles() {
        let mut detector = VisualLoopDetector::default();
        detector.seed(fingerprint([20, 20, 20, 255]));
        for color in [
            [220, 30, 30, 255],
            [20, 20, 20, 255],
            [220, 30, 30, 255],
            [20, 20, 20, 255],
        ] {
            assert!(detector.observe(fingerprint(color)).is_none());
        }
        let matched = detector
            .observe(fingerprint([220, 30, 30, 255]))
            .expect("three cycles");
        assert_eq!(matched.period, 2);
        assert!(detector.observe(fingerprint([20, 20, 20, 255])).is_none());
    }

    #[test]
    fn broken_cycle_rearms_the_same_pattern() {
        let mut detector = VisualLoopDetector::default();
        for color in repeating_colors() {
            let _ = detector.observe(fingerprint(color));
        }
        assert!(detector.observe(fingerprint([20, 220, 30, 255])).is_none());
        let mut matched = None;
        for color in repeating_colors() {
            matched = detector.observe(fingerprint(color)).or(matched);
        }
        assert_eq!(matched.expect("rearmed cycle").period, 2);
    }

    #[test]
    fn three_and_four_state_cycles_are_supported() {
        for states in [
            vec![[20, 20, 20, 255], [220, 30, 30, 255], [30, 220, 30, 255]],
            vec![
                [20, 20, 20, 255],
                [220, 30, 30, 255],
                [30, 220, 30, 255],
                [30, 30, 220, 255],
            ],
        ] {
            let mut detector = VisualLoopDetector::default();
            let mut matched = None;
            for _ in 0..3 {
                for color in &states {
                    matched = detector.observe(fingerprint(*color)).or(matched);
                }
            }
            assert_eq!(matched.expect("multi-state cycle").period, states.len());
        }
    }

    #[test]
    fn non_repeating_changes_do_not_alert() {
        let mut detector = VisualLoopDetector::default();
        for color in [
            [10, 20, 30, 255],
            [40, 50, 60, 255],
            [70, 80, 90, 255],
            [100, 110, 120, 255],
            [130, 140, 150, 255],
            [160, 170, 180, 255],
        ] {
            assert!(detector.observe(fingerprint(color)).is_none());
        }
    }

    fn repeating_colors() -> [[u8; 4]; 6] {
        [
            [20, 20, 20, 255],
            [220, 30, 30, 255],
            [20, 20, 20, 255],
            [220, 30, 30, 255],
            [20, 20, 20, 255],
            [220, 30, 30, 255],
        ]
    }

    fn fingerprint(color: [u8; 4]) -> VisualFingerprint {
        VisualFingerprint::from_frame(&solid_frame(color)).expect("fingerprint")
    }

    fn solid_frame(color: [u8; 4]) -> crate::capture_backend::CroppedFramePayload {
        patterned_frame(8, |_, _| color)
    }

    fn patterned_frame(
        side: i32,
        pixel: impl Fn(i32, i32) -> [u8; 4],
    ) -> crate::capture_backend::CroppedFramePayload {
        let region = PhysicalRegion {
            monitor_id: "main".into(),
            source_window: None,
            x: 0,
            y: 0,
            width: side,
            height: side,
        };
        let mut bytes = Vec::with_capacity((side * side * 4) as usize);
        for y in 0..side {
            for x in 0..side {
                bytes.extend_from_slice(&pixel(x, y));
            }
        }
        cropped_frame(&region, bytes)
    }
}
