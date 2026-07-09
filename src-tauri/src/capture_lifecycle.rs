use std::collections::BTreeMap;

use serde::Serialize;

use crate::region_selection_types::PhysicalRegion;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureLifecycle {
    privacy_blank_active: bool,
    tiles: BTreeMap<String, CaptureTile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureTile {
    pub id: String,
    pub mode: CaptureTileMode,
    pub region: PhysicalRegion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureTileMode {
    Live,
    Paused,
    Hidden,
    Blanked,
    Closed,
    Deleted,
}

impl CaptureLifecycle {
    pub fn upsert_tile(
        &mut self,
        id: impl Into<String>,
        region: PhysicalRegion,
        mode: CaptureTileMode,
    ) {
        let id = id.into();

        self.tiles
            .insert(id.clone(), CaptureTile { id, mode, region });
    }

    pub fn transition(&mut self, id: &str, mode: CaptureTileMode) {
        if let Some(tile) = self.tiles.get_mut(id) {
            tile.mode = mode;
        }
    }

    pub fn set_privacy_blank(&mut self, active: bool) {
        self.privacy_blank_active = active;
    }

    pub fn should_capture(&self, id: &str) -> bool {
        if self.privacy_blank_active {
            return false;
        }

        self.tiles
            .get(id)
            .map(|tile| tile.mode == CaptureTileMode::Live)
            .unwrap_or(false)
    }

    pub fn should_keep_task(&self, id: &str) -> bool {
        self.tiles
            .get(id)
            .map(|tile| {
                !matches!(
                    tile.mode,
                    CaptureTileMode::Closed | CaptureTileMode::Deleted
                )
            })
            .unwrap_or(false)
    }

    pub fn tiles(&self) -> impl Iterator<Item = &CaptureTile> {
        self.tiles.values()
    }
}
