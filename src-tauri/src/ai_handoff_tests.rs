use std::{cell::RefCell, rc::Rc};

use crate::{
    ai_handoff::{
        AiConnector, AiConnectorError, AiHandoffConfig, AiHandoffPayload, AiHandoffRequest,
        AiHandoffService, AiHandoffStatus, AiHandoffTrigger, AiPayloadKind, AiRegionConfig,
        AiRegionMode, LocalAiConnector,
    },
    capture_lifecycle::CaptureTileMode,
};

#[test]
fn ai_handoff_is_disabled_by_default() {
    let connector = FakeAiConnector::default();
    let sent = connector.sent_payloads();
    let mut service = AiHandoffService::new(AiHandoffConfig::default(), connector);

    let outcome = service.handoff(text_request()).expect("disabled outcome");

    assert_eq!(outcome.status, AiHandoffStatus::Disabled);
    assert!(!outcome.indicator_visible);
    assert!(sent.borrow().is_empty());
}

#[test]
fn unauthorized_region_cannot_handoff_data() {
    let mut service = AiHandoffService::new(enabled_config([]), FakeAiConnector::default());

    let outcome = service.handoff(text_request()).expect("unauthorized");

    assert_eq!(outcome.status, AiHandoffStatus::RegionUnauthorized);
}

#[test]
fn privacy_blank_and_inactive_tile_block_handoff() {
    let mut service = AiHandoffService::new(
        enabled_config([region("tile", AiRegionMode::TextOnChange)]),
        FakeAiConnector::default(),
    );

    let blanked = service
        .handoff(AiHandoffRequest {
            privacy_blank_active: true,
            ..text_request()
        })
        .expect("blanked");
    let paused = service
        .handoff(AiHandoffRequest {
            tile_mode: CaptureTileMode::Paused,
            ..text_request()
        })
        .expect("paused");

    assert_eq!(blanked.status, AiHandoffStatus::PrivacyBlanked);
    assert_eq!(paused.status, AiHandoffStatus::InactiveTile);
}

#[test]
fn text_payload_excludes_unrelated_screen_content() {
    let connector = FakeAiConnector::default();
    let sent = connector.sent_payloads();
    let mut service = AiHandoffService::new(
        enabled_config([region("tile", AiRegionMode::TextOnChange)]),
        connector,
    );

    let outcome = service.handoff(text_request()).expect("sent");
    let payload = sent.borrow().first().expect("payload").clone();

    assert_eq!(outcome.status, AiHandoffStatus::Sent);
    assert!(outcome.indicator_visible);
    assert_eq!(payload.kind, AiPayloadKind::Text);
    assert!(payload.text.contains("Observed text:\nerror: missing dep"));
    assert!(!payload.text.contains("other monitor"));
    assert_eq!(service.remembered_payload_text_count(), 0);
}

#[test]
fn image_handoff_requires_explicit_region_setting() {
    let mut service = AiHandoffService::new(
        enabled_config([region("tile", AiRegionMode::TextOnChange)]),
        FakeAiConnector::default(),
    );

    let outcome = service
        .handoff(AiHandoffRequest {
            requested_payload_kind: AiPayloadKind::Image,
            ..text_request()
        })
        .expect("image blocked");

    assert_eq!(
        outcome.status,
        AiHandoffStatus::ImageRequiresExplicitRegionSetting
    );
}

#[test]
fn cooldown_and_dedupe_reduce_repeated_calls() {
    let connector = FakeAiConnector::default();
    let sent = connector.sent_payloads();
    let mut service = AiHandoffService::new(
        enabled_config([region("tile", AiRegionMode::TextOnChange)]),
        connector,
    );

    let first = service.handoff(text_request()).expect("first");
    let cooldown = service
        .handoff(AiHandoffRequest {
            tick: 2,
            ocr_text: Some("different"),
            ..text_request()
        })
        .expect("cooldown");
    let deduped = service
        .handoff(AiHandoffRequest {
            tick: 5,
            ..text_request()
        })
        .expect("deduped");

    assert_eq!(first.status, AiHandoffStatus::Sent);
    assert_eq!(cooldown.status, AiHandoffStatus::Cooldown);
    assert_eq!(deduped.status, AiHandoffStatus::Deduped);
    assert_eq!(sent.borrow().len(), 1);
}

#[test]
fn scheduled_tick_and_empty_text_do_not_handoff() {
    let mut service = AiHandoffService::new(
        enabled_config([region("tile", AiRegionMode::TextOnChange)]),
        FakeAiConnector::default(),
    );

    let scheduled = service
        .handoff(AiHandoffRequest {
            trigger: AiHandoffTrigger::ScheduledTick,
            ..text_request()
        })
        .expect("scheduled");
    let empty = service
        .handoff(AiHandoffRequest {
            ocr_text: Some("   "),
            ..text_request()
        })
        .expect("empty");

    assert_eq!(scheduled.status, AiHandoffStatus::NotRequested);
    assert_eq!(empty.status, AiHandoffStatus::NoText);
}

#[test]
fn local_connector_errors_are_recoverable_and_do_not_send() {
    let mut service = AiHandoffService::new(
        enabled_config([region("tile", AiRegionMode::TextOnChange)]),
        LocalAiConnector,
    );

    let error = service
        .handoff(text_request())
        .expect_err("connector closed");

    assert!(error.recoverable);
}

#[test]
fn failed_connector_attempts_are_deduped_before_retrying() {
    let connector = FailingAiConnector::default();
    let attempts = connector.attempt_count();
    let mut service = AiHandoffService::new(
        enabled_config([region("tile", AiRegionMode::TextOnChange)]),
        connector,
    );

    let first = service.handoff(text_request()).expect_err("first failure");
    let duplicate = service
        .handoff(AiHandoffRequest {
            tick: 5,
            ..text_request()
        })
        .expect("deduped failure retry");

    assert!(first.recoverable);
    assert_eq!(duplicate.status, AiHandoffStatus::Deduped);
    assert_eq!(attempts.get(), 1);
}

#[derive(Debug, Default)]
struct FakeAiConnector {
    sent: Rc<RefCell<Vec<AiHandoffPayload>>>,
}

impl FakeAiConnector {
    fn sent_payloads(&self) -> Rc<RefCell<Vec<AiHandoffPayload>>> {
        Rc::clone(&self.sent)
    }
}

impl AiConnector for FakeAiConnector {
    fn send_text(&self, payload: &AiHandoffPayload) -> Result<(), AiConnectorError> {
        self.sent.borrow_mut().push(payload.clone());

        Ok(())
    }
}

#[derive(Debug, Default)]
struct FailingAiConnector {
    attempts: Rc<std::cell::Cell<usize>>,
}

impl FailingAiConnector {
    fn attempt_count(&self) -> Rc<std::cell::Cell<usize>> {
        Rc::clone(&self.attempts)
    }
}

impl AiConnector for FailingAiConnector {
    fn send_text(&self, _payload: &AiHandoffPayload) -> Result<(), AiConnectorError> {
        self.attempts.set(self.attempts.get() + 1);

        Err(AiConnectorError {
            code: crate::ai_handoff::AiConnectorErrorCode::ConnectorUnavailable,
            message: "closed".to_string(),
            recoverable: true,
        })
    }
}

fn enabled_config<const N: usize>(regions: [AiRegionConfig; N]) -> AiHandoffConfig {
    AiHandoffConfig {
        enabled: true,
        cooldown_ticks: 3,
        regions: regions.into(),
    }
}

fn region(region_id: &str, mode: AiRegionMode) -> AiRegionConfig {
    AiRegionConfig {
        region_id: region_id.to_string(),
        mode,
    }
}

fn text_request() -> AiHandoffRequest<'static> {
    AiHandoffRequest {
        region_id: "tile",
        tile_mode: CaptureTileMode::Live,
        privacy_blank_active: false,
        trigger: AiHandoffTrigger::ChangedFrame,
        requested_payload_kind: AiPayloadKind::Text,
        ocr_text: Some("error: missing dep"),
        tick: 1,
    }
}
