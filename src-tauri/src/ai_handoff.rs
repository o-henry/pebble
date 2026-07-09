use std::{
    collections::BTreeMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::capture_lifecycle::CaptureTileMode;

pub use crate::ai_handoff_types::{
    AiConnectorError, AiConnectorErrorCode, AiHandoffConfig, AiHandoffOutcome, AiHandoffPayload,
    AiHandoffRequest, AiHandoffStatus, AiHandoffTrigger, AiPayloadKind, AiRegionConfig,
    AiRegionMode,
};

const DEFAULT_COOLDOWN_TICKS: u64 = 3;

pub trait AiConnector {
    fn send_text(&self, payload: &AiHandoffPayload) -> Result<(), AiConnectorError>;

    fn send_image(&self, _payload: &AiHandoffPayload) -> Result<(), AiConnectorError> {
        Err(AiConnectorError::unavailable(
            "Image handoff connector is not available.",
        ))
    }
}

#[derive(Debug, Default)]
pub struct LocalAiConnector;

#[derive(Debug)]
pub struct AiHandoffService<C> {
    config: AiHandoffConfig,
    connector: C,
    last_attempt_by_region: BTreeMap<String, AttemptFingerprint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttemptFingerprint {
    fingerprint: u64,
    tick: u64,
}

impl Default for AiHandoffConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cooldown_ticks: DEFAULT_COOLDOWN_TICKS,
            regions: Vec::new(),
        }
    }
}

impl<C: AiConnector> AiHandoffService<C> {
    pub fn new(config: AiHandoffConfig, connector: C) -> Self {
        Self {
            config,
            connector,
            last_attempt_by_region: BTreeMap::new(),
        }
    }

    pub fn handoff(
        &mut self,
        request: AiHandoffRequest<'_>,
    ) -> Result<AiHandoffOutcome, AiConnectorError> {
        if !self.config.enabled {
            return Ok(skipped(request.region_id, AiHandoffStatus::Disabled));
        }

        if request.privacy_blank_active {
            return Ok(skipped(request.region_id, AiHandoffStatus::PrivacyBlanked));
        }

        if request.tile_mode != CaptureTileMode::Live {
            return Ok(skipped(request.region_id, AiHandoffStatus::InactiveTile));
        }

        let Some(region_mode) = self.region_mode(request.region_id) else {
            return Ok(skipped(
                request.region_id,
                AiHandoffStatus::RegionUnauthorized,
            ));
        };

        if !mode_allows_trigger(region_mode, request.trigger) {
            return Ok(skipped(request.region_id, AiHandoffStatus::NotRequested));
        }

        if !mode_allows_payload(region_mode, request.requested_payload_kind) {
            return Ok(skipped(
                request.region_id,
                AiHandoffStatus::ImageRequiresExplicitRegionSetting,
            ));
        }

        let Some(text) = request
            .ocr_text
            .map(str::trim)
            .filter(|text| !text.is_empty())
        else {
            return Ok(skipped(request.region_id, AiHandoffStatus::NoText));
        };

        let payload = build_text_payload(
            request.region_id,
            request.trigger,
            request.requested_payload_kind,
            text,
        );
        let fingerprint = payload_fingerprint(&payload);

        if !self.cooldown_ready(request.region_id, request.tick) {
            return Ok(skipped(request.region_id, AiHandoffStatus::Cooldown));
        }

        if self.is_duplicate(request.region_id, fingerprint) {
            return Ok(skipped(request.region_id, AiHandoffStatus::Deduped));
        }

        self.last_attempt_by_region.insert(
            request.region_id.to_string(),
            AttemptFingerprint {
                fingerprint,
                tick: request.tick,
            },
        );

        match payload.kind {
            AiPayloadKind::Text => self.connector.send_text(&payload)?,
            AiPayloadKind::Image => self.connector.send_image(&payload)?,
        }

        Ok(AiHandoffOutcome {
            region_id: request.region_id.to_string(),
            status: AiHandoffStatus::Sent,
            indicator_visible: true,
            sent_payload_kind: Some(payload.kind),
        })
    }

    pub fn remembered_payload_text_count(&self) -> usize {
        0
    }

    fn region_mode(&self, region_id: &str) -> Option<AiRegionMode> {
        self.config
            .regions
            .iter()
            .find(|region| region.region_id == region_id)
            .map(|region| region.mode)
            .filter(|mode| *mode != AiRegionMode::Off)
    }

    fn cooldown_ready(&self, region_id: &str, tick: u64) -> bool {
        self.last_attempt_by_region
            .get(region_id)
            .map(|sent| tick.saturating_sub(sent.tick) >= self.config.cooldown_ticks)
            .unwrap_or(true)
    }

    fn is_duplicate(&self, region_id: &str, fingerprint: u64) -> bool {
        self.last_attempt_by_region
            .get(region_id)
            .is_some_and(|sent| sent.fingerprint == fingerprint)
    }
}

impl AiConnector for LocalAiConnector {
    fn send_text(&self, _payload: &AiHandoffPayload) -> Result<(), AiConnectorError> {
        Err(AiConnectorError::unavailable(
            "AI connector is not available in this build.",
        ))
    }
}

impl AiConnectorError {
    fn unavailable(message: &str) -> Self {
        Self {
            code: AiConnectorErrorCode::ConnectorUnavailable,
            message: message.to_string(),
            recoverable: true,
        }
    }
}

fn mode_allows_trigger(mode: AiRegionMode, trigger: AiHandoffTrigger) -> bool {
    matches!(
        (mode, trigger),
        (AiRegionMode::TextOnChange, AiHandoffTrigger::ChangedFrame)
            | (AiRegionMode::ImageOnChange, AiHandoffTrigger::ChangedFrame)
            | (
                AiRegionMode::TextOnRequest,
                AiHandoffTrigger::ExplicitRequest
            )
            | (
                AiRegionMode::ImageOnRequest,
                AiHandoffTrigger::ExplicitRequest
            )
    )
}

fn mode_allows_payload(mode: AiRegionMode, kind: AiPayloadKind) -> bool {
    matches!(
        (mode, kind),
        (AiRegionMode::TextOnChange, AiPayloadKind::Text)
            | (AiRegionMode::TextOnRequest, AiPayloadKind::Text)
            | (AiRegionMode::ImageOnChange, AiPayloadKind::Image)
            | (AiRegionMode::ImageOnRequest, AiPayloadKind::Image)
    )
}

fn build_text_payload(
    region_id: &str,
    trigger: AiHandoffTrigger,
    kind: AiPayloadKind,
    text: &str,
) -> AiHandoffPayload {
    AiHandoffPayload {
        region_id: region_id.to_string(),
        trigger,
        kind,
        text: format!("Region: {region_id}\nEvent: {trigger:?}\nObserved text:\n{text}"),
    }
}

fn payload_fingerprint(payload: &AiHandoffPayload) -> u64 {
    let mut hasher = DefaultHasher::new();
    payload.region_id.hash(&mut hasher);
    payload.kind.hash(&mut hasher);
    payload.text.hash(&mut hasher);
    hasher.finish()
}

fn skipped(region_id: &str, status: AiHandoffStatus) -> AiHandoffOutcome {
    AiHandoffOutcome {
        region_id: region_id.to_string(),
        status,
        indicator_visible: false,
        sent_payload_kind: None,
    }
}
