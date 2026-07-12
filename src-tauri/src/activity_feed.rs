use std::{
    collections::VecDeque,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::Serialize;
use tauri::{Emitter, Manager};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use url::Url;

const MAX_MEMORY_ENTRIES: usize = 100;
const MAX_SUMMARY_CHARS: usize = 500;
const MAX_JOURNAL_BYTES: u64 = 25 * 1024 * 1024;
const JOURNAL_FOLDER: &str = "Pebble";
const JOURNAL_FILE: &str = "pebble-updates.md";
pub const UPDATE_FEED_EVENT: &str = "pebble://update-feed";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateKind {
    Watch,
    Source,
}

impl UpdateKind {
    fn label(self) -> &'static str {
        match self {
            Self::Watch => "WATCH",
            Self::Source => "SOURCE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEntry {
    pub id: u64,
    pub kind: UpdateKind,
    pub summary: String,
    pub source_url: Option<String>,
    pub occurred_at: String,
    pub saved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFeedSnapshot {
    pub entries: Vec<UpdateEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct ActivityFeedState {
    data: Arc<Mutex<ActivityFeedData>>,
}

#[derive(Debug, Default)]
struct ActivityFeedData {
    entries: VecDeque<UpdateEntry>,
    next_id: u64,
}

impl ActivityFeedState {
    pub fn snapshot(&self) -> UpdateFeedSnapshot {
        let entries = self
            .data
            .lock()
            .map(|data| data.entries.iter().cloned().collect())
            .unwrap_or_default();
        UpdateFeedSnapshot { entries }
    }

    pub(crate) fn record(
        &self,
        kind: UpdateKind,
        summary: &str,
        source_url: Option<&str>,
        occurred_at: String,
        journal_path: Option<&Path>,
    ) -> Option<UpdateEntry> {
        let summary = sanitize_summary(summary)?;
        let source_url = sanitize_source_url(source_url)?;
        let mut data = self.data.lock().ok()?;
        let saved = journal_path.is_some_and(|path| {
            append_journal(path, kind, &summary, source_url.as_deref(), &occurred_at).is_ok()
        });
        data.next_id = data.next_id.saturating_add(1);
        let entry = UpdateEntry {
            id: data.next_id,
            kind,
            summary,
            source_url,
            occurred_at,
            saved,
        };
        data.entries.push_front(entry.clone());
        data.entries.truncate(MAX_MEMORY_ENTRIES);
        Some(entry)
    }
}

pub fn snapshot(state: &ActivityFeedState) -> UpdateFeedSnapshot {
    state.snapshot()
}

pub fn record_watch(app: &tauri::AppHandle, state: &ActivityFeedState, summary: &str) {
    record_and_emit(app, state, UpdateKind::Watch, summary, None);
}

pub fn record_source(
    app: &tauri::AppHandle,
    state: &ActivityFeedState,
    summary: &str,
    source_url: &str,
) {
    record_and_emit(app, state, UpdateKind::Source, summary, Some(source_url));
}

fn record_and_emit(
    app: &tauri::AppHandle,
    state: &ActivityFeedState,
    kind: UpdateKind,
    summary: &str,
    source_url: Option<&str>,
) {
    let occurred_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "UNKNOWN".to_string());
    let path = journal_path(app);
    if let Some(entry) = state.record(kind, summary, source_url, occurred_at, path.as_deref()) {
        let _ = app.emit_to(
            crate::pebble_session::PEBBLE_TILE_LABEL,
            UPDATE_FEED_EVENT,
            entry,
        );
    }
}

fn journal_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path()
        .download_dir()
        .ok()
        .map(|downloads| downloads.join(JOURNAL_FOLDER).join(JOURNAL_FILE))
}

fn append_journal(
    path: &Path,
    kind: UpdateKind,
    summary: &str,
    source_url: Option<&str>,
    occurred_at: &str,
) -> std::io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("missing parent"))?;
    fs::create_dir_all(parent)?;
    if fs::symlink_metadata(parent)?.file_type().is_symlink() {
        return Err(std::io::Error::other(
            "journal directory cannot be a symlink",
        ));
    }
    secure_directory(parent)?;
    let current_size = fs::symlink_metadata(path)
        .map(|meta| {
            if meta.file_type().is_symlink() || !meta.is_file() {
                Err(std::io::Error::other("journal path must be a regular file"))
            } else {
                Ok(meta.len())
            }
        })
        .unwrap_or(Ok(0))?;
    if current_size >= MAX_JOURNAL_BYTES {
        return Err(std::io::Error::other("journal size limit reached"));
    }
    let needs_header = current_size == 0;
    let mut file = secure_append_file(path)?;
    if needs_header {
        file.write_all(b"# Pebble Updates\n\n")?;
    }
    let source = source_url
        .map(|url| format!(" | <{url}>"))
        .unwrap_or_default();
    writeln!(
        file,
        "- {} | {} | {}{}",
        occurred_at,
        kind.label(),
        escape_markdown(summary),
        source
    )
}

#[cfg(unix)]
fn secure_directory(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(unix)]
fn secure_append_file(path: &Path) -> std::io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

#[cfg(not(unix))]
fn secure_append_file(path: &Path) -> std::io::Result<fs::File> {
    OpenOptions::new().create(true).append(true).open(path)
}

#[cfg(not(unix))]
fn secure_directory(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

fn sanitize_summary(value: &str) -> Option<String> {
    if value.chars().any(char::is_control) {
        return None;
    }
    let summary = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!summary.is_empty() && summary.chars().count() <= MAX_SUMMARY_CHARS).then_some(summary)
}

fn sanitize_source_url(value: Option<&str>) -> Option<Option<String>> {
    let Some(value) = value else {
        return Some(None);
    };
    let url = Url::parse(value).ok()?;
    let allowed = url.scheme() == "https"
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.port_or_known_default() == Some(443);
    allowed.then(|| Some(url.to_string()))
}

fn escape_markdown(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
