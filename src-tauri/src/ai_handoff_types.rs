use std::hash::Hash;

use serde::Serialize;

use crate::capture_lifecycle::CaptureTileMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AiRegionMode {
    Off,
    TextOnChange,
    TextOnRequest,
    ImageOnRequest,
    ImageOnChange,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AiPayloadKind {
    Text,
    Image,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AiHandoffTrigger {
    ExplicitRequest,
    ChangedFrame,
    ScheduledTick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AiHandoffStatus {
    Disabled,
    RegionUnauthorized,
    InactiveTile,
    PrivacyBlanked,
    NotRequested,
    NoText,
    ImageRequiresExplicitRegionSetting,
    Cooldown,
    Deduped,
    Sent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AiConnectorErrorCode {
    ConnectorUnavailable,
    SendFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRegionConfig {
    pub region_id: String,
    pub mode: AiRegionMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiHandoffConfig {
    pub enabled: bool,
    pub cooldown_ticks: u64,
    pub regions: Vec<AiRegionConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiHandoffPayload {
    pub region_id: String,
    pub trigger: AiHandoffTrigger,
    pub kind: AiPayloadKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiHandoffOutcome {
    pub region_id: String,
    pub status: AiHandoffStatus,
    pub indicator_visible: bool,
    pub sent_payload_kind: Option<AiPayloadKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConnectorError {
    pub code: AiConnectorErrorCode,
    pub message: String,
    pub recoverable: bool,
}

pub struct AiHandoffRequest<'a> {
    pub region_id: &'a str,
    pub tile_mode: CaptureTileMode,
    pub privacy_blank_active: bool,
    pub trigger: AiHandoffTrigger,
    pub requested_payload_kind: AiPayloadKind,
    pub ocr_text: Option<&'a str>,
    pub tick: u64,
}
