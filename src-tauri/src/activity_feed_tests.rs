use std::{fs, path::PathBuf};

use crate::activity_feed::{
    ActivityFeedState, UpdateKind, WatchSignal, WatchSignalConfidence, WatchSignalEngine,
    WatchSignalKind,
};

#[test]
fn journal_appends_entries_to_one_markdown_document() {
    let path = test_path("append");
    let state = ActivityFeedState::default();
    let first = state
        .record(
            UpdateKind::Watch,
            "Material change detected",
            "2026-07-11T10:00:00Z".into(),
            Some(&path),
        )
        .expect("first entry");
    let second = state
        .record(
            UpdateKind::Watch,
            "A second material change",
            "2026-07-11T10:15:00Z".into(),
            Some(&path),
        )
        .expect("second entry");

    assert!(first.saved);
    assert!(second.saved);
    let document = fs::read_to_string(&path).expect("journal");
    assert_eq!(document.matches("# Pebble Updates").count(), 1);
    assert!(document.contains("WATCH | Material change detected"));
    assert!(document.contains("WATCH | A second material change"));
    let _ = fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn feed_rejects_control_characters() {
    let state = ActivityFeedState::default();
    assert!(state
        .record(UpdateKind::Watch, "unsafe\0summary", "now".into(), None,)
        .is_none());
}

#[test]
fn snapshot_keeps_newest_entries_first_without_private_frames() {
    let state = ActivityFeedState::default();
    state
        .record(UpdateKind::Watch, "first", "one".into(), None)
        .expect("first");
    state
        .record(UpdateKind::Watch, "second", "two".into(), None)
        .expect("second");

    let snapshot = state.snapshot();
    assert_eq!(snapshot.entries[0].summary, "second");
    let serialized = serde_json::to_string(&snapshot).expect("snapshot json");
    assert!(!serialized.contains("frame"));
    assert!(!serialized.contains("bytes"));
}

#[test]
fn structured_signal_keeps_safe_metadata_separate_from_the_summary() {
    let path = test_path("signal");
    let state = ActivityFeedState::default();
    let signal = WatchSignal::new(
        WatchSignalKind::Match,
        "REGION 1",
        WatchSignalEngine::OpenAi,
        Some("gpt-5.6-terra"),
        Some(WatchSignalConfidence::High),
        Some(1_240),
    )
    .expect("valid signal");
    let entry = state
        .record_signal(
            "The visible status changed to failed.",
            "2026-07-11T10:00:00Z".into(),
            Some(&path),
            signal,
        )
        .expect("signal entry");

    assert_eq!(entry.summary, "The visible status changed to failed.");
    assert_eq!(
        entry.signal.as_ref().map(|signal| signal.region.as_str()),
        Some("REGION 1")
    );
    let document = fs::read_to_string(&path).expect("journal");
    assert!(document.contains(
        "WATCH | REGION 1 | MATCH | OPENAI | GPT-5.6-TERRA | HIGH | 1240MS | The visible status changed to failed."
    ));
    let serialized = serde_json::to_string(&entry).expect("signal json");
    for private_field in ["frame", "bytes", "sourceWindow", "monitorId", "ocrText"] {
        assert!(!serialized.contains(private_field));
    }
    let _ = fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn rich_ai_summary_stays_in_memory_while_the_journal_is_redacted() {
    let path = test_path("redacted-ai-summary");
    let state = ActivityFeedState::default();
    let signal = WatchSignal::new(
        WatchSignalKind::Match,
        "REGION 1",
        WatchSignalEngine::OpenAi,
        Some("gpt-5.6-terra"),
        Some(WatchSignalConfidence::High),
        Some(900),
    )
    .expect("valid signal");
    let private_summary = "ACME changed from 123.45 to 129.80 (+5.1%).";
    let journal_summary =
        "A selected-region Watch condition matched. Detailed screen-derived values were omitted from the Pebble journal.";
    let entry = state
        .record_signal_with_journal_summary(
            private_summary,
            journal_summary,
            "2026-07-22T10:00:00Z".into(),
            Some(&path),
            signal,
        )
        .expect("redacted signal entry");

    assert_eq!(entry.summary, private_summary);
    assert!(entry.saved);
    let document = fs::read_to_string(&path).expect("journal");
    assert!(document.contains(journal_summary));
    assert!(!document.contains("ACME"));
    assert!(!document.contains("123.45"));
    assert!(!document.contains("129.80"));
    let _ = fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn rich_ai_summary_is_redacted_even_without_structured_metadata() {
    let path = test_path("redacted-ai-summary-without-signal");
    let state = ActivityFeedState::default();
    let private_summary = "Private row changed from pending to approved.";
    let journal_summary = "A selected-region Watch condition matched.";
    let entry = state
        .record_with_journal_summary(
            UpdateKind::Watch,
            private_summary,
            journal_summary,
            "2026-07-22T10:00:00Z".into(),
            Some(&path),
        )
        .expect("redacted entry without metadata");

    assert_eq!(entry.summary, private_summary);
    assert!(entry.signal.is_none());
    let document = fs::read_to_string(&path).expect("journal");
    assert!(document.contains(journal_summary));
    assert!(!document.contains("Private row"));
    assert!(!document.contains("approved"));
    let _ = fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn structured_signal_rejects_untrusted_metadata() {
    assert!(WatchSignal::new(
        WatchSignalKind::Waiting,
        "REGION 1\nINJECTED",
        WatchSignalEngine::System,
        None,
        None,
        None,
    )
    .is_none());
    assert!(WatchSignal::new(
        WatchSignalKind::Match,
        "REGION 1",
        WatchSignalEngine::Claude,
        Some("model | injected"),
        None,
        None,
    )
    .is_none());
}

#[test]
fn cross_region_signal_records_only_safe_region_labels() {
    let path = test_path("cross-region");
    let state = ActivityFeedState::default();
    let related = vec!["REGION 2".to_string()];
    let signal = WatchSignal::new(
        WatchSignalKind::Conflict,
        "REGION 1",
        WatchSignalEngine::LocalCrossCheck,
        None,
        Some(WatchSignalConfidence::High),
        None,
    )
    .and_then(|signal| signal.with_related_regions(&related))
    .expect("safe cross-region signal");
    let entry = state
        .record_signal(
            "Opposing states remain visible.",
            "2026-07-16T10:00:00Z".into(),
            Some(&path),
            signal,
        )
        .expect("cross-region entry");

    assert_eq!(
        entry
            .signal
            .as_ref()
            .map(|signal| signal.related_regions.as_slice()),
        Some(related.as_slice())
    );
    let document = fs::read_to_string(&path).expect("journal");
    assert!(document.contains(
        "REGION 1 + REGION 2 | CONFLICT | LOCAL CROSS-CHECK | HIGH | Opposing states remain visible."
    ));
    let serialized = serde_json::to_string(&entry).expect("entry json");
    for private_field in ["frame", "bytes", "ocrText", "monitorId", "sourceWindow"] {
        assert!(!serialized.contains(private_field));
    }
    let _ = fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn cross_region_signal_rejects_unsafe_or_excessive_related_regions() {
    let signal = WatchSignal::new(
        WatchSignalKind::Conflict,
        "REGION 1",
        WatchSignalEngine::LocalCrossCheck,
        None,
        None,
        None,
    )
    .expect("base signal");
    assert!(signal
        .clone()
        .with_related_regions(&["REGION 2\nINJECTED".to_string()])
        .is_none());
    assert!(signal
        .with_related_regions(&[
            "REGION 2".to_string(),
            "REGION 3".to_string(),
            "REGION 4".to_string(),
        ])
        .is_none());
}

#[test]
fn follow_through_signal_keeps_only_safe_linked_region_labels() {
    let path = test_path("follow-through");
    let state = ActivityFeedState::default();
    let related = vec!["REGION 2".to_string()];
    let signal = WatchSignal::new(
        WatchSignalKind::NoFollowThrough,
        "REGION 1",
        WatchSignalEngine::LocalFollowThrough,
        None,
        Some(WatchSignalConfidence::High),
        None,
    )
    .and_then(|signal| signal.with_related_regions(&related))
    .expect("safe follow-through signal");
    let entry = state
        .record_signal(
            "REGION 2 did not change after REGION 1.",
            "2026-07-16T11:00:00Z".into(),
            Some(&path),
            signal,
        )
        .expect("follow-through entry");

    assert_eq!(
        entry.signal.as_ref().map(|signal| signal.kind),
        Some(WatchSignalKind::NoFollowThrough)
    );
    let document = fs::read_to_string(&path).expect("journal");
    assert!(
        document.contains("REGION 1 + REGION 2 | NO FOLLOW-THROUGH | LOCAL FOLLOW-THROUGH | HIGH")
    );
    let serialized = serde_json::to_string(&entry).expect("entry json");
    for private_field in ["frame", "bytes", "ocrText", "monitorId", "sourceWindow"] {
        assert!(!serialized.contains(private_field));
    }
    let _ = fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn visual_loop_signal_contains_no_fingerprint_or_frame_data() {
    let path = test_path("visual-loop");
    let state = ActivityFeedState::default();
    let signal = WatchSignal::new(
        WatchSignalKind::Loop,
        "REGION 1",
        WatchSignalEngine::LocalVisualLoop,
        None,
        Some(WatchSignalConfidence::High),
        None,
    )
    .expect("safe visual loop signal");
    let entry = state
        .record_signal(
            "The region repeated a 2-step visual cycle.",
            "2026-07-16T12:00:00Z".into(),
            Some(&path),
            signal,
        )
        .expect("visual loop entry");

    let document = fs::read_to_string(&path).expect("journal");
    assert!(document.contains("REGION 1 | LOOP | LOCAL VISUAL LOOP | HIGH"));
    let serialized = serde_json::to_string(&entry).expect("entry json");
    for private_field in [
        "frame",
        "bytes",
        "fingerprint",
        "cells",
        "loopHistory",
        "monitorId",
        "sourceWindow",
    ] {
        assert!(!serialized.contains(private_field));
    }
    let _ = fs::remove_dir_all(path.parent().expect("parent"));
}

#[cfg(unix)]
#[test]
fn journal_refuses_a_symlink_target() {
    use std::os::unix::fs::symlink;

    let path = test_path("symlink");
    let parent = path.parent().expect("parent");
    fs::create_dir_all(parent).expect("test directory");
    let target = parent.join("target.md");
    fs::write(&target, "do not touch").expect("target");
    symlink(&target, &path).expect("symlink");
    let state = ActivityFeedState::default();
    let entry = state
        .record(UpdateKind::Watch, "change", "now".into(), Some(&path))
        .expect("memory entry");

    assert!(!entry.saved);
    assert_eq!(fs::read_to_string(&target).unwrap(), "do not touch");
    let _ = fs::remove_dir_all(parent);
}

fn test_path(label: &str) -> PathBuf {
    std::env::temp_dir()
        .join(format!(
            "pebble-activity-feed-{}-{label}",
            std::process::id()
        ))
        .join("pebble-updates.md")
}
