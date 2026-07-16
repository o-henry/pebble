use crate::watch_intent::{
    CompiledWatchIntent, CrossRegionState, LocalWatchDecision, WatchEvaluationMode,
    WatchLocalEngine,
};

#[test]
fn compiles_text_appearance_and_disappearance_rules() {
    let appears = CompiledWatchIntent::compile("Tell me when 'FAILED' appears".into());
    assert_eq!(appears.mode(), WatchEvaluationMode::Local);
    assert_eq!(appears.rule_summary(), "TEXT APPEARS: failed");
    assert!(matches!(
        appears.evaluate(Some("RUNNING"), Some("BUILD FAILED"), "en-US"),
        LocalWatchDecision::Matched(_)
    ));
    assert_eq!(
        appears.evaluate(Some("BUILD FAILED"), Some("BUILD FAILED"), "en-US"),
        LocalWatchDecision::NotMatched
    );

    let disappears = CompiledWatchIntent::compile("Tell me when READY disappears".into());
    assert!(matches!(
        disappears.evaluate(Some("READY"), Some("WAITING"), "en-US"),
        LocalWatchDecision::Matched(_)
    ));
}

#[test]
fn compiles_korean_text_and_numeric_rules() {
    let text = CompiledWatchIntent::compile("'오류'가 나타나면 알려줘".into());
    assert!(matches!(
        text.evaluate(Some("정상"), Some("오류 발생"), "ko-KR"),
        LocalWatchDecision::Matched(_)
    ));

    let number = CompiledWatchIntent::compile("숫자가 100 이상이면 알려줘".into());
    assert_eq!(number.rule_summary(), "NUMBER >= 100");
    assert!(matches!(
        number.evaluate(Some("현재 99"), Some("현재 100"), "ko-KR"),
        LocalWatchDecision::Matched(_)
    ));
}

#[test]
fn numeric_rules_match_only_when_the_threshold_is_crossed() {
    let above = CompiledWatchIntent::compile("Tell me when the number goes above 100".into());
    assert_eq!(above.rule_summary(), "NUMBER > 100");
    assert_eq!(
        above.evaluate(Some("101"), Some("102"), "en-US"),
        LocalWatchDecision::NotMatched
    );
    assert!(matches!(
        above.evaluate(Some("99"), Some("101"), "en-US"),
        LocalWatchDecision::Matched(_)
    ));

    let below = CompiledWatchIntent::compile("Alert me when the value is below 5".into());
    assert!(matches!(
        below.evaluate(Some("5"), Some("4"), "en-US"),
        LocalWatchDecision::Matched(_)
    ));
}

#[test]
fn ambiguous_numeric_screens_require_semantic_ai_instead_of_guessing() {
    let intent = CompiledWatchIntent::compile("Tell me when the number goes above 100".into());
    assert_eq!(
        intent.evaluate(
            Some("PRICE 99 VOLUME 20"),
            Some("PRICE 101 VOLUME 21"),
            "en-US"
        ),
        LocalWatchDecision::NeedsAi
    );
}

#[test]
fn progress_and_state_rules_stay_local() {
    let progress = CompiledWatchIntent::compile("Tell me when progress reaches 100%".into());
    assert_eq!(progress.rule_summary(), "PROGRESS REACHES 100%");
    assert!(matches!(
        progress.evaluate(Some("PROGRESS 92%"), Some("PROGRESS 100%"), "en-US"),
        LocalWatchDecision::Matched(_)
    ));

    let state = CompiledWatchIntent::compile("Tell me when the build fails".into());
    assert_eq!(state.rule_summary(), "ERROR STATE APPEARS");
    assert!(matches!(
        state.evaluate(Some("RUNNING"), Some("FAILED"), "en-US"),
        LocalWatchDecision::Matched(_)
    ));
}

#[test]
fn unsupported_or_missing_ocr_evidence_requires_ai() {
    let semantic = CompiledWatchIntent::compile("Tell me when this looks unusual".into());
    assert_eq!(semantic.mode(), WatchEvaluationMode::Ai);
    assert_eq!(
        semantic.evaluate(Some("before"), Some("after"), "en-US"),
        LocalWatchDecision::NeedsAi
    );

    let local = CompiledWatchIntent::compile("Tell me when ERROR appears".into());
    assert_eq!(
        local.evaluate(None, Some("ERROR"), "en-US"),
        LocalWatchDecision::NeedsAi
    );
}

#[test]
fn local_summaries_follow_the_user_locale() {
    let intent = CompiledWatchIntent::compile("Tell me when ERROR appears".into());
    let LocalWatchDecision::Matched(event) = intent.evaluate(Some("READY"), Some("ERROR"), "ko-KR")
    else {
        panic!("expected local match");
    };
    assert!(event.summary.starts_with("조건이 충족되었습니다."));
    assert!(event.fingerprint.starts_with("local:"));
}

#[test]
fn compiles_stuck_intents_as_zero_token_visual_rules() {
    for intent in [
        "Tell me when this region stops changing after activity",
        "Notify me if progress gets stuck",
        "진행이 멈추면 알려줘",
        "변화가 없으면 알려줘",
    ] {
        let compiled = CompiledWatchIntent::compile(intent.into());
        assert_eq!(compiled.mode(), WatchEvaluationMode::Local);
        assert_eq!(
            compiled.local_engine(),
            Some(WatchLocalEngine::VisualStability)
        );
        assert!(compiled.detects_stuck_after_activity());
        assert_eq!(compiled.rule_summary(), "NO PROGRESS AFTER ACTIVITY");
        assert_eq!(
            compiled.evaluate(Some("10%"), Some("20%"), "en-US"),
            LocalWatchDecision::NotMatched
        );
    }
}

#[test]
fn ordinary_stop_language_does_not_accidentally_enable_stuck_watch() {
    let compiled = CompiledWatchIntent::compile("Tell me when the service stops failing".into());
    assert!(!compiled.detects_stuck_after_activity());
}

#[test]
fn compiles_cross_region_conflicts_as_local_ocr_state_rules() {
    for intent in [
        "Tell me when watched regions show opposing success and error states",
        "Notify me when regions disagree",
        "영역 상태 불일치가 생기면 알려줘",
    ] {
        let compiled = CompiledWatchIntent::compile(intent.into());
        assert_eq!(compiled.mode(), WatchEvaluationMode::Local);
        assert_eq!(
            compiled.local_engine(),
            Some(WatchLocalEngine::CrossRegionOcr)
        );
        assert!(compiled.detects_cross_region_conflict());
        assert_eq!(compiled.rule_summary(), "CROSS-REGION STATUS CONFLICT");
    }
}

#[test]
fn cross_region_state_classifier_is_high_precision_and_local() {
    let compiled = CompiledWatchIntent::compile(
        "Tell me when watched regions show opposing success and error states".into(),
    );
    for text in [
        "DEPLOY SUCCESS",
        "ALL SYSTEMS HEALTHY",
        "작업 완료",
        "서비스 정상",
    ] {
        assert_eq!(
            compiled.classify_cross_region_state(text),
            Some(CrossRegionState::Positive)
        );
    }
    for text in [
        "BUILD FAILED",
        "SERVICE OFFLINE",
        "NOT HEALTHY",
        "NOT READY",
        "NOT COMPLETE",
        "NOT PASSED",
        "배포 오류",
        "서비스 비정상",
        "작업 미완료",
    ] {
        assert_eq!(
            compiled.classify_cross_region_state(text),
            Some(CrossRegionState::Negative)
        );
    }
    for text in ["STILL RUNNING", "ALREADY CHECKED", "WARNING", "42%"] {
        assert_eq!(compiled.classify_cross_region_state(text), None);
    }
}

#[test]
fn ordinary_semantic_intents_do_not_join_cross_region_watch() {
    let compiled = CompiledWatchIntent::compile("Tell me whether these screens agree".into());
    assert!(!compiled.detects_cross_region_conflict());
    assert_eq!(compiled.classify_cross_region_state("SUCCESS"), None);
}
