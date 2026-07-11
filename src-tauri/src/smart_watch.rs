use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::Emitter;

pub const SMART_WATCH_CONSENT_VERSION: u16 = 2;
pub const SMART_WATCH_SESSION_LIMIT: u16 = 24;
pub const SMART_WATCH_STATUS_EVENT: &str = "pebble://smart-watch-status";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartWatchStatus {
    pub enabled: bool,
    pub notifications_sent: u16,
    pub session_limit: u16,
    pub remaining: u16,
}

#[derive(Debug, Clone, Default)]
pub struct SmartWatchState {
    data: Arc<Mutex<SmartWatchData>>,
}

#[derive(Debug, Default)]
struct SmartWatchData {
    enabled: bool,
    revision: Option<u64>,
    notifications_sent: u16,
}

impl SmartWatchState {
    pub fn configure(
        &self,
        enabled: bool,
        revision: u64,
        consent_version: u16,
    ) -> Result<SmartWatchStatus, SmartWatchError> {
        if enabled && consent_version != SMART_WATCH_CONSENT_VERSION {
            return Err(SmartWatchError::consent_required());
        }

        let mut data = self
            .data
            .lock()
            .map_err(|_| SmartWatchError::unavailable())?;
        data.enabled = enabled;
        data.revision = enabled.then_some(revision);
        Ok(data.status())
    }

    pub fn disable(&self) -> SmartWatchStatus {
        let mut data = self.data.lock().expect("smart watch state lock");
        data.enabled = false;
        data.revision = None;
        data.status()
    }

    pub fn status(&self) -> SmartWatchStatus {
        let data = self.data.lock().expect("smart watch state lock");
        data.status()
    }

    pub fn claim_notification(&self, revision: u64) -> Option<SmartWatchStatus> {
        let mut data = self.data.lock().ok()?;
        if !data.enabled {
            return None;
        }
        if data.revision != Some(revision) {
            data.enabled = false;
            data.revision = None;
            return None;
        }
        if data.notifications_sent >= SMART_WATCH_SESSION_LIMIT {
            return None;
        }

        data.notifications_sent = data.notifications_sent.saturating_add(1);
        Some(data.status())
    }
}

impl SmartWatchData {
    fn status(&self) -> SmartWatchStatus {
        SmartWatchStatus {
            enabled: self.enabled,
            notifications_sent: self.notifications_sent,
            session_limit: SMART_WATCH_SESSION_LIMIT,
            remaining: SMART_WATCH_SESSION_LIMIT.saturating_sub(self.notifications_sent),
        }
    }
}

pub fn emit_status(app: &tauri::AppHandle, status: SmartWatchStatus) {
    let _ = app.emit_to(
        crate::pebble_session::PEBBLE_TILE_LABEL,
        SMART_WATCH_STATUS_EVENT,
        status,
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SmartWatchErrorCode {
    ConsentRequired,
    InvalidSession,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartWatchError {
    pub code: SmartWatchErrorCode,
    pub message: &'static str,
}

impl SmartWatchError {
    fn consent_required() -> Self {
        Self {
            code: SmartWatchErrorCode::ConsentRequired,
            message: "REVIEW AND ACCEPT THE SMART WATCH NOTICE BEFORE ENABLING IT.",
        }
    }

    pub fn invalid_session() -> Self {
        Self {
            code: SmartWatchErrorCode::InvalidSession,
            message: "SMART WATCH NEEDS A VISIBLE, ACTIVE SELECTED REGION.",
        }
    }

    pub fn unavailable() -> Self {
        Self {
            code: SmartWatchErrorCode::Unavailable,
            message: "SMART WATCH STATE IS UNAVAILABLE.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SmartWatchErrorCode, SmartWatchState, SMART_WATCH_CONSENT_VERSION,
        SMART_WATCH_SESSION_LIMIT,
    };

    #[test]
    fn watch_is_off_until_current_consent_is_supplied() {
        let state = SmartWatchState::default();
        assert!(!state.status().enabled);
        assert_eq!(
            state.configure(true, 7, 0).unwrap_err().code,
            SmartWatchErrorCode::ConsentRequired
        );
        assert!(
            state
                .configure(true, 7, SMART_WATCH_CONSENT_VERSION)
                .unwrap()
                .enabled
        );
    }

    #[test]
    fn region_change_disables_watch() {
        let state = SmartWatchState::default();
        state
            .configure(true, 7, SMART_WATCH_CONSENT_VERSION)
            .unwrap();

        assert!(state.claim_notification(8).is_none());
        assert!(!state.status().enabled);
    }

    #[test]
    fn watch_exposes_a_bounded_session_notification_budget() {
        let state = SmartWatchState::default();
        state
            .configure(true, 2, SMART_WATCH_CONSENT_VERSION)
            .unwrap();

        for _ in 0..SMART_WATCH_SESSION_LIMIT {
            assert!(state.claim_notification(2).is_some());
        }
        assert!(state.claim_notification(2).is_none());
        assert_eq!(state.status().remaining, 0);
    }

    #[test]
    fn disable_stops_notifications_immediately() {
        let state = SmartWatchState::default();
        state
            .configure(true, 3, SMART_WATCH_CONSENT_VERSION)
            .unwrap();
        state.disable();

        assert!(state.claim_notification(3).is_none());
    }
}
