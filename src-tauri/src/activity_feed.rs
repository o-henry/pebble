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
}

impl UpdateKind {
    fn label(self) -> &'static str {
        match self {
            Self::Watch => "WATCH",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEntry {
    pub id: u64,
    pub kind: UpdateKind,
    pub summary: String,
    pub occurred_at: String,
    pub saved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<WatchSignal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WatchSignalKind {
    Match,
    Stuck,
    Conflict,
    NoFollowThrough,
    Loop,
    Waiting,
    AnalysisSkipped,
}

impl WatchSignalKind {
    fn label(self) -> &'static str {
        match self {
            Self::Match => "MATCH",
            Self::Stuck => "STUCK",
            Self::Conflict => "CONFLICT",
            Self::NoFollowThrough => "NO FOLLOW-THROUGH",
            Self::Loop => "LOOP",
            Self::Waiting => "WAITING",
            Self::AnalysisSkipped => "ANALYSIS SKIPPED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WatchSignalEngine {
    System,
    LocalOcr,
    LocalVisual,
    LocalCrossCheck,
    LocalFollowThrough,
    LocalVisualLoop,
    OpenAi,
    Claude,
}

impl WatchSignalEngine {
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "SYSTEM",
            Self::LocalOcr => "LOCAL OCR",
            Self::LocalVisual => "LOCAL VISUAL",
            Self::LocalCrossCheck => "LOCAL CROSS-CHECK",
            Self::LocalFollowThrough => "LOCAL FOLLOW-THROUGH",
            Self::LocalVisualLoop => "LOCAL VISUAL LOOP",
            Self::OpenAi => "OPENAI",
            Self::Claude => "CLAUDE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WatchSignalConfidence {
    Low,
    Medium,
    High,
}

impl WatchSignalConfidence {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchSignal {
    pub kind: WatchSignalKind,
    pub region: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_regions: Vec<String>,
    pub engine: WatchSignalEngine,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<WatchSignalConfidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl WatchSignal {
    pub fn new(
        kind: WatchSignalKind,
        region: &str,
        engine: WatchSignalEngine,
        model: Option<&str>,
        confidence: Option<WatchSignalConfidence>,
        duration_ms: Option<u64>,
    ) -> Option<Self> {
        let region = sanitized_metadata(region, 80)?;
        let model = match model {
            Some(value) => Some(sanitized_metadata(value, 100)?),
            None => None,
        };
        Some(Self {
            kind,
            region,
            related_regions: Vec::new(),
            engine,
            model,
            confidence,
            duration_ms,
        })
    }

    pub fn with_related_regions(mut self, regions: &[String]) -> Option<Self> {
        if regions.len() > 2 {
            return None;
        }
        let mut related = Vec::with_capacity(regions.len());
        for region in regions {
            let region = sanitized_metadata(region, 80)?;
            if region != self.region && !related.contains(&region) {
                related.push(region);
            }
        }
        self.related_regions = related;
        Some(self)
    }

    fn journal_fields(&self) -> Vec<String> {
        let region_label = std::iter::once(&self.region)
            .chain(self.related_regions.iter())
            .cloned()
            .collect::<Vec<_>>()
            .join(" + ");
        let mut fields = vec![
            region_label,
            self.kind.label().to_string(),
            self.engine.label().to_string(),
        ];
        if let Some(model) = &self.model {
            fields.push(model.to_ascii_uppercase());
        }
        if let Some(confidence) = self.confidence {
            fields.push(confidence.label().to_string());
        }
        if let Some(duration_ms) = self.duration_ms {
            fields.push(format!("{duration_ms}MS"));
        }
        fields
    }
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
        occurred_at: String,
        journal_path: Option<&Path>,
    ) -> Option<UpdateEntry> {
        self.record_entry(kind, summary, None, occurred_at, journal_path, None)
    }

    pub(crate) fn record_signal(
        &self,
        summary: &str,
        occurred_at: String,
        journal_path: Option<&Path>,
        signal: WatchSignal,
    ) -> Option<UpdateEntry> {
        self.record_entry(
            UpdateKind::Watch,
            summary,
            None,
            occurred_at,
            journal_path,
            Some(signal),
        )
    }

    pub(crate) fn record_signal_with_journal_summary(
        &self,
        summary: &str,
        journal_summary: &str,
        occurred_at: String,
        journal_path: Option<&Path>,
        signal: WatchSignal,
    ) -> Option<UpdateEntry> {
        self.record_entry(
            UpdateKind::Watch,
            summary,
            Some(journal_summary),
            occurred_at,
            journal_path,
            Some(signal),
        )
    }

    pub(crate) fn record_with_journal_summary(
        &self,
        kind: UpdateKind,
        summary: &str,
        journal_summary: &str,
        occurred_at: String,
        journal_path: Option<&Path>,
    ) -> Option<UpdateEntry> {
        self.record_entry(
            kind,
            summary,
            Some(journal_summary),
            occurred_at,
            journal_path,
            None,
        )
    }

    fn record_entry(
        &self,
        kind: UpdateKind,
        summary: &str,
        journal_summary: Option<&str>,
        occurred_at: String,
        journal_path: Option<&Path>,
        signal: Option<WatchSignal>,
    ) -> Option<UpdateEntry> {
        let summary = sanitize_summary(summary)?;
        let journal_summary = match journal_summary {
            Some(value) => sanitize_summary(value)?,
            None => summary.clone(),
        };
        let mut data = self.data.lock().ok()?;
        let saved = journal_path.is_some_and(|path| {
            append_journal(path, kind, &journal_summary, &occurred_at, signal.as_ref()).is_ok()
        });
        data.next_id = data.next_id.saturating_add(1);
        let entry = UpdateEntry {
            id: data.next_id,
            kind,
            summary,
            occurred_at,
            saved,
            signal,
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

pub fn record_watch_signal(
    app: &tauri::AppHandle,
    state: &ActivityFeedState,
    summary: &str,
    signal: WatchSignal,
) {
    record_and_emit(app, state, UpdateKind::Watch, summary, Some(signal));
}

pub fn record_watch_signal_with_journal_summary(
    app: &tauri::AppHandle,
    state: &ActivityFeedState,
    summary: &str,
    journal_summary: &str,
    signal: WatchSignal,
) {
    let occurred_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "UNKNOWN".to_string());
    let path = journal_path(app);
    let entry = state.record_signal_with_journal_summary(
        summary,
        journal_summary,
        occurred_at,
        path.as_deref(),
        signal,
    );
    emit_entry(app, entry);
}

pub fn record_watch_with_journal_summary(
    app: &tauri::AppHandle,
    state: &ActivityFeedState,
    summary: &str,
    journal_summary: &str,
) {
    let occurred_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "UNKNOWN".to_string());
    let path = journal_path(app);
    let entry = state.record_with_journal_summary(
        UpdateKind::Watch,
        summary,
        journal_summary,
        occurred_at,
        path.as_deref(),
    );
    emit_entry(app, entry);
}

fn record_and_emit(
    app: &tauri::AppHandle,
    state: &ActivityFeedState,
    kind: UpdateKind,
    summary: &str,
    signal: Option<WatchSignal>,
) {
    let occurred_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "UNKNOWN".to_string());
    let path = journal_path(app);
    let entry = match signal {
        Some(signal) => state.record_signal(summary, occurred_at, path.as_deref(), signal),
        None => state.record(kind, summary, occurred_at, path.as_deref()),
    };
    emit_entry(app, entry);
}

fn emit_entry(app: &tauri::AppHandle, entry: Option<UpdateEntry>) {
    if let Some(entry) = entry {
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
    occurred_at: &str,
    signal: Option<&WatchSignal>,
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
    let mut fields = vec![occurred_at.to_string(), kind.label().to_string()];
    if let Some(signal) = signal {
        fields.extend(signal.journal_fields());
    }
    fields.push(summary.to_string());
    writeln!(
        file,
        "- {}",
        fields
            .iter()
            .map(|field| escape_markdown(field))
            .collect::<Vec<_>>()
            .join(" | ")
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

fn sanitized_metadata(value: &str, max_chars: usize) -> Option<String> {
    let value = value.trim();
    let valid = !value.is_empty()
        && value.chars().count() <= max_chars
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b'-' | b'_' | b'.'));
    valid.then(|| value.to_string())
}

fn escape_markdown(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
