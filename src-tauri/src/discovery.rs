use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::Serialize;
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::time::sleep;

use crate::{
    activity_feed::ActivityFeedState,
    discovery_fetch::{fetch_discovery, DiscoveryFetch},
    public_source::PublicSourceError,
};

const REFRESH_INTERVAL: Duration = Duration::from_secs(30 * 60);
const REFRESH_INTERVAL_MINUTES: u16 = 30;
pub const DISCOVERY_STATUS_EVENT: &str = "pebble://discovery-status";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DiscoveryCategory {
    News,
    Community,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryItem {
    pub id: String,
    pub category: DiscoveryCategory,
    pub title: String,
    pub source: String,
    pub url: String,
    pub score: Option<u32>,
    pub comments: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryStatus {
    pub enabled: bool,
    pub interval_minutes: u16,
    pub last_checked_at: Option<String>,
    pub items: Vec<DiscoveryItem>,
    pub warnings: Vec<&'static str>,
    pub error: Option<&'static str>,
}

#[derive(Debug, Clone, Default)]
pub struct DiscoveryState {
    data: Arc<Mutex<DiscoveryData>>,
}

#[derive(Debug, Clone)]
struct DiscoveryData {
    generation: u64,
    fingerprint: Option<u64>,
    status: DiscoveryStatus,
}

impl Default for DiscoveryData {
    fn default() -> Self {
        Self {
            generation: 0,
            fingerprint: None,
            status: DiscoveryStatus {
                enabled: false,
                interval_minutes: REFRESH_INTERVAL_MINUTES,
                last_checked_at: None,
                items: Vec::new(),
                warnings: Vec::new(),
                error: None,
            },
        }
    }
}

impl DiscoveryState {
    pub fn status(&self) -> DiscoveryStatus {
        self.data
            .lock()
            .map(|data| data.status.clone())
            .unwrap_or_else(|_| DiscoveryData::default().status)
    }

    fn begin(&self) -> Result<u64, PublicSourceError> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| PublicSourceError::unavailable())?;
        data.generation = data.generation.saturating_add(1);
        data.status.enabled = true;
        data.status.error = None;
        Ok(data.generation)
    }

    pub fn disable(&self) -> DiscoveryStatus {
        let mut data = self.data.lock().expect("discovery state lock");
        data.generation = data.generation.saturating_add(1);
        data.fingerprint = None;
        data.status.enabled = false;
        data.status.error = None;
        data.status.clone()
    }

    fn is_current(&self, generation: u64) -> bool {
        self.data
            .lock()
            .is_ok_and(|data| data.generation == generation && data.status.enabled)
    }

    fn apply(
        &self,
        generation: u64,
        fetched: DiscoveryFetch,
    ) -> Option<(DiscoveryStatus, bool, bool)> {
        let mut data = self.data.lock().ok()?;
        if data.generation != generation || !data.status.enabled {
            return None;
        }
        let fingerprint = fingerprint(&fetched.items);
        let initial = data.fingerprint.is_none();
        let changed = data
            .fingerprint
            .is_some_and(|previous| previous != fingerprint);
        data.fingerprint = Some(fingerprint);
        data.status.last_checked_at = Some(now());
        data.status.items = fetched.items;
        data.status.warnings = fetched.warnings;
        data.status.error = None;
        Some((data.status.clone(), initial, changed))
    }

    fn mark_error(&self, generation: u64) -> Option<DiscoveryStatus> {
        let mut data = self.data.lock().ok()?;
        if data.generation != generation || !data.status.enabled {
            return None;
        }
        data.status.last_checked_at = Some(now());
        data.status.error = Some("DISCOVERY CHECK FAILED");
        Some(data.status.clone())
    }
}

pub async fn enable(
    app: tauri::AppHandle,
    state: DiscoveryState,
) -> Result<DiscoveryStatus, PublicSourceError> {
    if state.status().enabled {
        return Ok(state.status());
    }
    let generation = state.begin()?;
    emit_status(&app, state.status());
    if let Err(error) = refresh_generation(&app, &state, generation).await {
        emit_status(&app, state.disable());
        return Err(error);
    }

    let loop_app = app.clone();
    let loop_state = state.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            sleep(REFRESH_INTERVAL).await;
            if !loop_state.is_current(generation) {
                break;
            }
            let _ = refresh_generation(&loop_app, &loop_state, generation).await;
        }
    });
    Ok(state.status())
}

pub async fn refresh(
    app: &tauri::AppHandle,
    state: &DiscoveryState,
) -> Result<DiscoveryStatus, PublicSourceError> {
    let status = state.status();
    if !status.enabled {
        return Err(PublicSourceError {
            code: crate::public_source::PublicSourceErrorCode::Unavailable,
            message: "START DISCOVERY BEFORE REFRESHING.",
        });
    }
    let generation = state
        .data
        .lock()
        .map_err(|_| PublicSourceError::unavailable())?
        .generation;
    refresh_generation(app, state, generation).await?;
    Ok(state.status())
}

async fn refresh_generation(
    app: &tauri::AppHandle,
    state: &DiscoveryState,
    generation: u64,
) -> Result<(), PublicSourceError> {
    match fetch_discovery().await {
        Ok(fetched) => {
            apply_fetch(app, state, generation, fetched);
            Ok(())
        }
        Err(error) => {
            if let Some(status) = state.mark_error(generation) {
                emit_status(app, status);
            }
            Err(error)
        }
    }
}

fn apply_fetch(
    app: &tauri::AppHandle,
    state: &DiscoveryState,
    generation: u64,
    fetched: DiscoveryFetch,
) {
    let Some((status, initial, changed)) = state.apply(generation, fetched) else {
        return;
    };
    emit_status(app, status.clone());
    if !(initial || changed) {
        return;
    }
    let summary = digest_summary(&status.items);
    crate::activity_feed::record_discovery(app, app.state::<ActivityFeedState>().inner(), &summary);
    if changed {
        crate::menu_bar::set_attention(app, true);
        let _ = app
            .notification()
            .builder()
            .title("PEBBLE DISCOVER")
            .body(&summary)
            .show();
    }
}

pub fn emit_status(app: &tauri::AppHandle, status: DiscoveryStatus) {
    let _ = app.emit_to(
        crate::pebble_session::PEBBLE_TILE_LABEL,
        DISCOVERY_STATUS_EVENT,
        status,
    );
}

fn digest_summary(items: &[DiscoveryItem]) -> String {
    let news = items
        .iter()
        .find(|item| item.category == DiscoveryCategory::News)
        .map(|item| item.title.as_str())
        .unwrap_or("NO NEWS UPDATE");
    let community = items
        .iter()
        .find(|item| item.category == DiscoveryCategory::Community)
        .map(|item| item.title.as_str())
        .unwrap_or("NO COMMUNITY UPDATE");
    format!("NEWS: {news} · COMMUNITY: {community}")
        .chars()
        .take(500)
        .collect()
}

fn fingerprint(items: &[DiscoveryItem]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for item in items {
        item.id.hash(&mut hasher);
    }
    hasher.finish()
}

fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "UNKNOWN".to_string())
}
