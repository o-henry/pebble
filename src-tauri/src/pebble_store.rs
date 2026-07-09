use std::{
    fs,
    io::{self, ErrorKind},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{
    performance_limits::{
        PerformanceLimitErrorCode, PerformanceLimitRequest, PerformanceLimits, RegionSize,
    },
    region_selection_types::PhysicalRegion,
};

pub const PEBBLE_STORE_SCHEMA_VERSION: u32 = 1;
const PEBBLE_STORE_FILE_NAME: &str = "pebbles.json";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PebbleStoreDocument {
    pub schema_version: u32,
    pub migration: PebbleStoreMigration,
    pub regions: Vec<StoredPebbleRegion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PebbleStoreMigration {
    pub latest_supported_schema: u32,
    pub last_migrated_from: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredPebbleRegion {
    pub id: String,
    pub name: String,
    pub region: PhysicalRegion,
    pub capture: StoredCaptureConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredCaptureConfig {
    pub fps: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PebbleStoreErrorCode {
    ConfigPathUnavailable,
    ReadFailed,
    WriteFailed,
    CorruptData,
    InvalidConfig,
    UnsupportedSchema,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PebbleStoreError {
    pub code: PebbleStoreErrorCode,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone)]
pub struct PebbleStore {
    path: PathBuf,
}

impl Default for PebbleStoreDocument {
    fn default() -> Self {
        Self {
            schema_version: PEBBLE_STORE_SCHEMA_VERSION,
            migration: PebbleStoreMigration::default(),
            regions: Vec::new(),
        }
    }
}

impl Default for PebbleStoreMigration {
    fn default() -> Self {
        Self {
            latest_supported_schema: PEBBLE_STORE_SCHEMA_VERSION,
            last_migrated_from: None,
        }
    }
}

impl PebbleStoreDocument {
    pub fn restore_named_regions(&self) -> Vec<StoredPebbleRegion> {
        self.regions.clone()
    }

    pub fn into_current_schema(mut self) -> Self {
        self.schema_version = PEBBLE_STORE_SCHEMA_VERSION;
        self.migration.latest_supported_schema = PEBBLE_STORE_SCHEMA_VERSION;
        self
    }

    fn validate_schema(&self) -> Result<(), PebbleStoreError> {
        if self.schema_version > PEBBLE_STORE_SCHEMA_VERSION {
            return Err(PebbleStoreError::unsupported_schema());
        }

        Ok(())
    }

    fn validate_safe_config(&self) -> Result<(), PebbleStoreError> {
        let limits = PerformanceLimits::default();

        for stored_region in &self.regions {
            let validation = limits.validate(PerformanceLimitRequest {
                fps: stored_region.capture.fps,
                active_tile_count: self.regions.len() as i32,
                region: RegionSize {
                    width: stored_region.region.width,
                    height: stored_region.region.height,
                },
            });

            if let Err(error) = validation {
                return Err(PebbleStoreError::invalid_config(
                    &stored_region.id,
                    error.code,
                ));
            }
        }

        Ok(())
    }
}

impl PebbleStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path_for_config_dir(config_dir: PathBuf) -> PathBuf {
        config_dir.join(PEBBLE_STORE_FILE_NAME)
    }

    pub fn load_or_default(&self) -> Result<PebbleStoreDocument, PebbleStoreError> {
        match fs::read_to_string(&self.path) {
            Ok(raw) => parse_document(&raw),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(PebbleStoreDocument::default()),
            Err(error) => Err(PebbleStoreError::io(
                PebbleStoreErrorCode::ReadFailed,
                "Could not read pebble configuration.",
                error,
            )),
        }
    }

    pub fn save(
        &self,
        document: &PebbleStoreDocument,
    ) -> Result<PebbleStoreDocument, PebbleStoreError> {
        document.validate_schema()?;
        document.validate_safe_config()?;
        let document = document.clone().into_current_schema();
        let raw = serde_json::to_string_pretty(&document).map_err(PebbleStoreError::corrupt)?;

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                PebbleStoreError::io(
                    PebbleStoreErrorCode::WriteFailed,
                    "Could not create pebble configuration directory.",
                    error,
                )
            })?;
        }

        fs::write(&self.path, raw).map_err(|error| {
            PebbleStoreError::io(
                PebbleStoreErrorCode::WriteFailed,
                "Could not write pebble configuration.",
                error,
            )
        })?;

        Ok(document)
    }
}

impl PebbleStoreError {
    pub fn config_path_unavailable() -> Self {
        Self {
            code: PebbleStoreErrorCode::ConfigPathUnavailable,
            message: "Could not resolve the app configuration directory.".to_string(),
            recoverable: true,
        }
    }

    fn corrupt(error: serde_json::Error) -> Self {
        Self {
            code: PebbleStoreErrorCode::CorruptData,
            message: format!("Pebble configuration is not valid JSON: {error}."),
            recoverable: true,
        }
    }

    fn unsupported_schema() -> Self {
        Self {
            code: PebbleStoreErrorCode::UnsupportedSchema,
            message: "Pebble configuration was written by a newer app version.".to_string(),
            recoverable: true,
        }
    }

    fn invalid_config(region_id: &str, code: PerformanceLimitErrorCode) -> Self {
        Self {
            code: PebbleStoreErrorCode::InvalidConfig,
            message: format!(
                "Pebble configuration for region '{region_id}' violates the performance contract: {code:?}."
            ),
            recoverable: true,
        }
    }

    fn io(code: PebbleStoreErrorCode, message: &str, error: io::Error) -> Self {
        Self {
            code,
            message: format!("{message} {error}."),
            recoverable: true,
        }
    }
}

fn parse_document(raw: &str) -> Result<PebbleStoreDocument, PebbleStoreError> {
    let document: PebbleStoreDocument =
        serde_json::from_str(raw).map_err(PebbleStoreError::corrupt)?;
    document.validate_schema()?;
    document.validate_safe_config()?;

    Ok(document)
}
