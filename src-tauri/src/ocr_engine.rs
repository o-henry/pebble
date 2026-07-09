use std::{
    collections::BTreeMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use serde::Serialize;

use crate::capture_backend::CroppedFramePayload;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OcrTrigger {
    ExplicitRequest,
    ChangedFrame,
    ScheduledTick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OcrRunStatus {
    Disabled,
    NotRequested,
    Unchanged,
    TextReady,
    Deduped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OcrStoragePolicy {
    EphemeralOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OcrErrorCode {
    AdapterUnavailable,
    RecognitionFailed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrStatus {
    pub enabled_by_default: bool,
    pub local_adapter_available: bool,
    pub storage_policy: OcrStoragePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrRunOutcome {
    pub tile_id: String,
    pub status: OcrRunStatus,
    pub text: Option<String>,
    pub storage_policy: OcrStoragePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrError {
    pub code: OcrErrorCode,
    pub message: String,
    pub recoverable: bool,
}

pub struct OcrFrameRequest<'a> {
    pub tile_id: &'a str,
    pub frame: &'a CroppedFramePayload,
    pub trigger: OcrTrigger,
    pub diff_changed: bool,
}

pub trait OcrEngine {
    fn recognize_text(&self, frame: &CroppedFramePayload) -> Result<String, OcrError>;
}

#[derive(Debug, Default)]
pub struct LocalOcrAdapter;

#[derive(Debug)]
pub struct OcrService<E> {
    config: OcrConfig,
    engine: E,
    last_text_fingerprints: BTreeMap<String, u64>,
}

impl OcrStatus {
    pub fn default_local() -> Self {
        Self {
            enabled_by_default: OcrConfig::default().enabled,
            local_adapter_available: false,
            storage_policy: OcrStoragePolicy::EphemeralOnly,
        }
    }
}

impl<E: OcrEngine> OcrService<E> {
    pub fn new(config: OcrConfig, engine: E) -> Self {
        Self {
            config,
            engine,
            last_text_fingerprints: BTreeMap::new(),
        }
    }

    pub fn run(&mut self, request: OcrFrameRequest<'_>) -> Result<OcrRunOutcome, OcrError> {
        if !self.config.enabled {
            return Ok(skipped(request.tile_id, OcrRunStatus::Disabled));
        }

        if request.trigger == OcrTrigger::ScheduledTick {
            return Ok(skipped(request.tile_id, OcrRunStatus::NotRequested));
        }

        if request.trigger == OcrTrigger::ChangedFrame && !request.diff_changed {
            return Ok(skipped(request.tile_id, OcrRunStatus::Unchanged));
        }

        let text = normalize_ocr_text(self.engine.recognize_text(request.frame)?);
        let fingerprint = text_fingerprint(&text);
        if self
            .last_text_fingerprints
            .get(request.tile_id)
            .is_some_and(|previous| *previous == fingerprint)
        {
            return Ok(skipped(request.tile_id, OcrRunStatus::Deduped));
        }

        self.last_text_fingerprints
            .insert(request.tile_id.to_string(), fingerprint);

        Ok(OcrRunOutcome {
            tile_id: request.tile_id.to_string(),
            status: OcrRunStatus::TextReady,
            text: Some(text),
            storage_policy: OcrStoragePolicy::EphemeralOnly,
        })
    }

    pub fn remembered_text_count(&self) -> usize {
        0
    }

    pub fn fingerprint_count(&self) -> usize {
        self.last_text_fingerprints.len()
    }
}

impl OcrEngine for LocalOcrAdapter {
    fn recognize_text(&self, _frame: &CroppedFramePayload) -> Result<String, OcrError> {
        Err(OcrError {
            code: OcrErrorCode::AdapterUnavailable,
            message: "Local OCR adapter is not available in this build.".to_string(),
            recoverable: true,
        })
    }
}

pub fn local_ocr_status() -> OcrStatus {
    OcrStatus::default_local()
}

fn skipped(tile_id: &str, status: OcrRunStatus) -> OcrRunOutcome {
    OcrRunOutcome {
        tile_id: tile_id.to_string(),
        status,
        text: None,
        storage_policy: OcrStoragePolicy::EphemeralOnly,
    }
}

fn normalize_ocr_text(text: String) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn text_fingerprint(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}
