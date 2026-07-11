use std::{fs, path::PathBuf};

use crate::activity_feed::{ActivityFeedState, UpdateKind};

#[test]
fn journal_appends_entries_to_one_markdown_document() {
    let path = test_path("append");
    let state = ActivityFeedState::default();
    let first = state
        .record(
            UpdateKind::Watch,
            "Material change detected",
            None,
            "2026-07-11T10:00:00Z".into(),
            Some(&path),
        )
        .expect("first entry");
    let second = state
        .record(
            UpdateKind::Watch,
            "A second material change",
            None,
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
fn feed_rejects_controls_and_non_https_sources() {
    let state = ActivityFeedState::default();
    assert!(state
        .record(
            UpdateKind::Watch,
            "unsafe\0summary",
            None,
            "now".into(),
            None,
        )
        .is_none());
    assert!(state
        .record(
            UpdateKind::Watch,
            "local",
            Some("http://127.0.0.1/private"),
            "now".into(),
            None,
        )
        .is_none());
}

#[test]
fn snapshot_keeps_newest_entries_first_without_private_frames() {
    let state = ActivityFeedState::default();
    state
        .record(UpdateKind::Watch, "first", None, "one".into(), None)
        .expect("first");
    state
        .record(UpdateKind::Watch, "second", None, "two".into(), None)
        .expect("second");

    let snapshot = state.snapshot();
    assert_eq!(snapshot.entries[0].summary, "second");
    let serialized = serde_json::to_string(&snapshot).expect("snapshot json");
    assert!(!serialized.contains("frame"));
    assert!(!serialized.contains("bytes"));
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
        .record(UpdateKind::Watch, "change", None, "now".into(), Some(&path))
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
