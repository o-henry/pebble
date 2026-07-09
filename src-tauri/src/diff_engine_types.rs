const DEFAULT_SAMPLE_WIDTH: usize = 64;
const DEFAULT_SAMPLE_HEIGHT: usize = 64;
const DEFAULT_CHANGE_THRESHOLD: f64 = 0.18;
const DEFAULT_COOLDOWN_TICKS: u64 = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct DiffEngineConfig {
    pub sample_width: usize,
    pub sample_height: usize,
    pub change_threshold: f64,
    pub cooldown_ticks: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmallFrame {
    pub width: usize,
    pub height: usize,
    pub grayscale: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffObservation {
    pub tile_id: String,
    pub score: f64,
    pub changed: bool,
    pub event: Option<ChangedEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChangedEvent {
    pub tile_id: String,
    pub score: f64,
    pub tick: u64,
    pub sample_width: usize,
    pub sample_height: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffEngineError {
    pub code: DiffEngineErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffEngineErrorCode {
    InvalidConfig,
    InvalidFrameDimensions,
    UnsupportedPixelFormat,
    ByteLengthMismatch,
    SampleSizeMismatch,
}

impl Default for DiffEngineConfig {
    fn default() -> Self {
        Self {
            sample_width: DEFAULT_SAMPLE_WIDTH,
            sample_height: DEFAULT_SAMPLE_HEIGHT,
            change_threshold: DEFAULT_CHANGE_THRESHOLD,
            cooldown_ticks: DEFAULT_COOLDOWN_TICKS,
        }
    }
}

impl DiffEngineError {
    pub(crate) fn new(code: DiffEngineErrorCode) -> Self {
        Self {
            code,
            message: message_for(code).to_string(),
        }
    }
}

fn message_for(code: DiffEngineErrorCode) -> &'static str {
    match code {
        DiffEngineErrorCode::InvalidConfig => "Diff engine configuration is invalid.",
        DiffEngineErrorCode::InvalidFrameDimensions => "Frame dimensions must be positive.",
        DiffEngineErrorCode::UnsupportedPixelFormat => "Only RGBA8 frames are supported.",
        DiffEngineErrorCode::ByteLengthMismatch => "Frame byte length does not match dimensions.",
        DiffEngineErrorCode::SampleSizeMismatch => "Small frame sample sizes do not match.",
    }
}
