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
