use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::region_selection_types::PhysicalRegion;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureLifecycle {
    privacy_blank_active: bool,
    pre_blank_modes: BTreeMap<String, CaptureTileMode>,
    tiles: BTreeMap<String, CaptureTile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureTile {
    pub id: String,
    pub mode: CaptureTileMode,
    pub region: PhysicalRegion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
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
        let mode = self.capture_mode_for_upsert(&id, mode);

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

    pub fn blank_all(&mut self) {
        if !self.privacy_blank_active {
            self.pre_blank_modes = self
                .tiles
                .iter()
                .filter(|(_, tile)| !terminal_mode(tile.mode))
                .map(|(id, tile)| (id.clone(), tile.mode))
                .collect();
        }

        self.privacy_blank_active = true;
        for tile in self.tiles.values_mut() {
            if !terminal_mode(tile.mode) {
                tile.mode = CaptureTileMode::Blanked;
            }
        }
    }

    pub fn restore_after_blank(&mut self) {
        if !self.privacy_blank_active {
            return;
        }

        for (id, mode) in std::mem::take(&mut self.pre_blank_modes) {
            if let Some(tile) = self.tiles.get_mut(&id) {
                if !terminal_mode(tile.mode) {
                    tile.mode = mode;
                }
            }
        }
        self.privacy_blank_active = false;
    }

    pub fn privacy_blank_active(&self) -> bool {
        self.privacy_blank_active
    }

    pub fn tile_mode(&self, id: &str) -> Option<CaptureTileMode> {
        self.tiles.get(id).map(|tile| tile.mode)
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
            .map(|tile| !terminal_mode(tile.mode))
            .unwrap_or(false)
    }

    pub fn tiles(&self) -> impl Iterator<Item = &CaptureTile> {
        self.tiles.values()
    }

    fn capture_mode_for_upsert(&mut self, id: &str, mode: CaptureTileMode) -> CaptureTileMode {
        if !self.privacy_blank_active || terminal_mode(mode) {
            return mode;
        }

        self.pre_blank_modes
            .entry(id.to_string())
            .or_insert(CaptureTileMode::Paused);
        CaptureTileMode::Blanked
    }
}

fn terminal_mode(mode: CaptureTileMode) -> bool {
    matches!(mode, CaptureTileMode::Closed | CaptureTileMode::Deleted)
}
