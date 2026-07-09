use std::collections::BTreeMap;

use crate::capture_backend::{CroppedFramePayload, FramePixelFormat};

const RGBA_BYTES_PER_PIXEL: usize = 4;

pub use crate::diff_engine_types::{
    ChangedEvent, DiffEngineConfig, DiffEngineError, DiffEngineErrorCode, DiffObservation,
    SmallFrame,
};

#[derive(Debug, Default)]
pub struct DiffEngine {
    config: DiffEngineConfig,
    tiles: BTreeMap<String, TileDiffState>,
}

#[derive(Debug, Clone, PartialEq)]
struct TileDiffState {
    previous_small_frame: SmallFrame,
    last_changed_tick: Option<u64>,
}

impl DiffEngine {
    pub fn new(config: DiffEngineConfig) -> Result<Self, DiffEngineError> {
        validate_config(&config)?;

        Ok(Self {
            config,
            tiles: BTreeMap::new(),
        })
    }

    pub fn observe_frame(
        &mut self,
        tile_id: impl Into<String>,
        frame: &CroppedFramePayload,
        tick: u64,
    ) -> Result<DiffObservation, DiffEngineError> {
        let tile_id = tile_id.into();
        let current = downsample_to_grayscale(frame, &self.config)?;
        let Some(previous_state) = self.tiles.get_mut(&tile_id) else {
            self.tiles.insert(
                tile_id.clone(),
                TileDiffState {
                    previous_small_frame: current,
                    last_changed_tick: None,
                },
            );
            return Ok(observation(tile_id, 0.0, false, None));
        };

        let score = mean_absolute_difference(&previous_state.previous_small_frame, &current)?;
        previous_state.previous_small_frame = current;
        let should_emit = score >= self.config.change_threshold
            && cooldown_ready(
                previous_state.last_changed_tick,
                tick,
                self.config.cooldown_ticks,
            );
        let event = should_emit.then(|| ChangedEvent {
            tile_id: tile_id.clone(),
            score,
            tick,
            sample_width: self.config.sample_width,
            sample_height: self.config.sample_height,
        });

        if should_emit {
            previous_state.last_changed_tick = Some(tick);
        }

        Ok(observation(tile_id, score, should_emit, event))
    }

    pub fn tracked_tile_count(&self) -> usize {
        self.tiles.len()
    }

    pub fn previous_sample_len(&self, tile_id: &str) -> Option<usize> {
        self.tiles
            .get(tile_id)
            .map(|state| state.previous_small_frame.grayscale.len())
    }
}

pub fn downsample_to_grayscale(
    frame: &CroppedFramePayload,
    config: &DiffEngineConfig,
) -> Result<SmallFrame, DiffEngineError> {
    validate_config(config)?;
    validate_frame(frame)?;

    let source_width = usize::try_from(frame.width)
        .map_err(|_| DiffEngineError::new(DiffEngineErrorCode::InvalidFrameDimensions))?;
    let source_height = usize::try_from(frame.height)
        .map_err(|_| DiffEngineError::new(DiffEngineErrorCode::InvalidFrameDimensions))?;
    let mut grayscale = Vec::with_capacity(config.sample_width * config.sample_height);

    for sample_y in 0..config.sample_height {
        let source_y = sample_y * source_height / config.sample_height;
        for sample_x in 0..config.sample_width {
            let source_x = sample_x * source_width / config.sample_width;
            let index = (source_y * source_width + source_x) * RGBA_BYTES_PER_PIXEL;
            grayscale.push(luma(
                frame.bytes[index],
                frame.bytes[index + 1],
                frame.bytes[index + 2],
            ));
        }
    }

    Ok(SmallFrame {
        width: config.sample_width,
        height: config.sample_height,
        grayscale,
    })
}

pub fn mean_absolute_difference(
    previous: &SmallFrame,
    current: &SmallFrame,
) -> Result<f64, DiffEngineError> {
    if previous.width != current.width
        || previous.height != current.height
        || previous.grayscale.len() != current.grayscale.len()
    {
        return Err(DiffEngineError::new(
            DiffEngineErrorCode::SampleSizeMismatch,
        ));
    }

    if previous.grayscale.is_empty() {
        return Ok(0.0);
    }

    let total_delta: u64 = previous
        .grayscale
        .iter()
        .zip(current.grayscale.iter())
        .map(|(previous, current)| previous.abs_diff(*current) as u64)
        .sum();

    Ok(total_delta as f64 / (previous.grayscale.len() as f64 * 255.0))
}

fn validate_config(config: &DiffEngineConfig) -> Result<(), DiffEngineError> {
    if config.sample_width == 0
        || config.sample_height == 0
        || !config.change_threshold.is_finite()
        || !(0.0..=1.0).contains(&config.change_threshold)
    {
        return Err(DiffEngineError::new(DiffEngineErrorCode::InvalidConfig));
    }

    Ok(())
}

fn validate_frame(frame: &CroppedFramePayload) -> Result<(), DiffEngineError> {
    if frame.pixel_format != FramePixelFormat::Rgba8 || frame.bytes_per_pixel != 4 {
        return Err(DiffEngineError::new(
            DiffEngineErrorCode::UnsupportedPixelFormat,
        ));
    }

    if frame.width < 1 || frame.height < 1 {
        return Err(DiffEngineError::new(
            DiffEngineErrorCode::InvalidFrameDimensions,
        ));
    }

    let width = usize::try_from(frame.width)
        .map_err(|_| DiffEngineError::new(DiffEngineErrorCode::InvalidFrameDimensions))?;
    let height = usize::try_from(frame.height)
        .map_err(|_| DiffEngineError::new(DiffEngineErrorCode::InvalidFrameDimensions))?;
    let expected_len = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(RGBA_BYTES_PER_PIXEL))
        .ok_or_else(|| DiffEngineError::new(DiffEngineErrorCode::ByteLengthMismatch))?;

    if frame.bytes.len() != expected_len {
        return Err(DiffEngineError::new(
            DiffEngineErrorCode::ByteLengthMismatch,
        ));
    }

    Ok(())
}

fn cooldown_ready(last_changed_tick: Option<u64>, tick: u64, cooldown_ticks: u64) -> bool {
    last_changed_tick
        .map(|last_tick| tick.saturating_sub(last_tick) >= cooldown_ticks)
        .unwrap_or(true)
}

fn observation(
    tile_id: String,
    score: f64,
    changed: bool,
    event: Option<ChangedEvent>,
) -> DiffObservation {
    DiffObservation {
        tile_id,
        score,
        changed,
        event,
    }
}

fn luma(red: u8, green: u8, blue: u8) -> u8 {
    ((77 * u16::from(red) + 150 * u16::from(green) + 29 * u16::from(blue)) / 256) as u8
}
