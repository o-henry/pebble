use std::{
    fs::{self, File, OpenOptions},
    io::{self, ErrorKind, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{Deserialize, Serialize};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use crate::{
    performance_limits::{
        PerformanceLimitErrorCode, PerformanceLimitRequest, PerformanceLimits, RegionSize,
    },
    region_selection_types::PhysicalRegion,
};

pub const PEBBLE_STORE_SCHEMA_VERSION: u32 = 1;
const PEBBLE_STORE_FILE_NAME: &str = "pebbles.json";
const MAX_PEBBLE_STORE_BYTES: u64 = 1_048_576;
static TEMP_FILE_NONCE: AtomicU64 = AtomicU64::new(0);

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
        match open_config_for_read(&self.path) {
            Ok(file) => {
                let metadata = file.metadata().map_err(|error| {
                    PebbleStoreError::io(
                        PebbleStoreErrorCode::ReadFailed,
                        "Could not inspect pebble configuration.",
                        error,
                    )
                })?;
                if !metadata.is_file() || metadata.len() > MAX_PEBBLE_STORE_BYTES {
                    return Err(PebbleStoreError::invalid_store_file());
                }
                secure_loaded_store(&file, &self.path)?;

                let mut raw = String::new();
                file.take(MAX_PEBBLE_STORE_BYTES + 1)
                    .read_to_string(&mut raw)
                    .map_err(|error| {
                        PebbleStoreError::io(
                            PebbleStoreErrorCode::ReadFailed,
                            "Could not read pebble configuration.",
                            error,
                        )
                    })?;
                if raw.len() as u64 > MAX_PEBBLE_STORE_BYTES {
                    return Err(PebbleStoreError::invalid_store_file());
                }

                parse_document(&raw)
            }
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
        if raw.len() as u64 > MAX_PEBBLE_STORE_BYTES {
            return Err(PebbleStoreError::invalid_store_file());
        }

        let parent = self.path.parent().ok_or_else(|| {
            PebbleStoreError::io(
                PebbleStoreErrorCode::WriteFailed,
                "Pebble configuration has no parent directory.",
                io::Error::new(ErrorKind::InvalidInput, "missing parent directory"),
            )
        })?;
        prepare_private_directory(parent)?;
        reject_unsafe_destination(&self.path)?;

        let (temp_path, mut temp_file) = create_private_temp_file(parent)?;
        let write_result = (|| {
            temp_file.write_all(raw.as_bytes()).map_err(|error| {
                PebbleStoreError::io(
                    PebbleStoreErrorCode::WriteFailed,
                    "Could not write the private pebble configuration file.",
                    error,
                )
            })?;
            temp_file.sync_all().map_err(|error| {
                PebbleStoreError::io(
                    PebbleStoreErrorCode::WriteFailed,
                    "Could not synchronize the private pebble configuration file.",
                    error,
                )
            })?;
            drop(temp_file);
            reject_unsafe_destination(&self.path)?;
            fs::rename(&temp_path, &self.path).map_err(|error| {
                PebbleStoreError::io(
                    PebbleStoreErrorCode::WriteFailed,
                    "Could not atomically replace pebble configuration.",
                    error,
                )
            })?;
            sync_directory(parent).map_err(|error| {
                PebbleStoreError::io(
                    PebbleStoreErrorCode::WriteFailed,
                    "Could not synchronize pebble configuration directory.",
                    error,
                )
            })?;
            Ok::<(), PebbleStoreError>(())
        })();

        if let Err(error) = write_result {
            let _ = fs::remove_file(&temp_path);
            return Err(error);
        }

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

    fn invalid_store_file() -> Self {
        Self {
            code: PebbleStoreErrorCode::InvalidConfig,
            message: "Pebble configuration must be a regular file no larger than 1 MiB."
                .to_string(),
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

fn open_config_for_read(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW);
    options.open(path)
}

fn secure_loaded_store(file: &File, path: &Path) -> Result<(), PebbleStoreError> {
    let parent = path.parent().ok_or_else(|| {
        PebbleStoreError::io(
            PebbleStoreErrorCode::ReadFailed,
            "Pebble configuration has no parent directory.",
            io::Error::new(ErrorKind::InvalidInput, "missing parent directory"),
        )
    })?;
    let parent_metadata = fs::symlink_metadata(parent).map_err(|error| {
        PebbleStoreError::io(
            PebbleStoreErrorCode::ReadFailed,
            "Could not inspect pebble configuration directory.",
            error,
        )
    })?;
    if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
        return Err(PebbleStoreError::io(
            PebbleStoreErrorCode::ReadFailed,
            "Pebble configuration directory is not a private directory.",
            io::Error::new(ErrorKind::InvalidInput, "unsafe configuration directory"),
        ));
    }

    #[cfg(unix)]
    {
        let parent_file = File::open(parent).map_err(|error| {
            PebbleStoreError::io(
                PebbleStoreErrorCode::ReadFailed,
                "Could not open pebble configuration directory.",
                error,
            )
        })?;
        parent_file
            .set_permissions(fs::Permissions::from_mode(0o700))
            .map_err(|error| {
                PebbleStoreError::io(
                    PebbleStoreErrorCode::ReadFailed,
                    "Could not secure pebble configuration directory.",
                    error,
                )
            })?;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| {
                PebbleStoreError::io(
                    PebbleStoreErrorCode::ReadFailed,
                    "Could not secure pebble configuration file.",
                    error,
                )
            })?;
    }

    Ok(())
}

fn prepare_private_directory(path: &Path) -> Result<(), PebbleStoreError> {
    fs::create_dir_all(path).map_err(|error| {
        PebbleStoreError::io(
            PebbleStoreErrorCode::WriteFailed,
            "Could not create pebble configuration directory.",
            error,
        )
    })?;

    let metadata = fs::symlink_metadata(path).map_err(|error| {
        PebbleStoreError::io(
            PebbleStoreErrorCode::WriteFailed,
            "Could not inspect pebble configuration directory.",
            error,
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(PebbleStoreError::io(
            PebbleStoreErrorCode::WriteFailed,
            "Pebble configuration directory is not a private directory.",
            io::Error::new(ErrorKind::InvalidInput, "unsafe configuration directory"),
        ));
    }

    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|error| {
        PebbleStoreError::io(
            PebbleStoreErrorCode::WriteFailed,
            "Could not secure pebble configuration directory.",
            error,
        )
    })?;

    Ok(())
}

fn reject_unsafe_destination(path: &Path) -> Result<(), PebbleStoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(PebbleStoreError::io(
                PebbleStoreErrorCode::WriteFailed,
                "Pebble configuration destination is not a regular file.",
                io::Error::new(ErrorKind::InvalidInput, "unsafe configuration destination"),
            ))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(PebbleStoreError::io(
            PebbleStoreErrorCode::WriteFailed,
            "Could not inspect pebble configuration destination.",
            error,
        )),
    }
}

fn create_private_temp_file(parent: &Path) -> Result<(PathBuf, File), PebbleStoreError> {
    for _ in 0..16 {
        let nonce = TEMP_FILE_NONCE.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(
            ".{PEBBLE_STORE_FILE_NAME}.{}.{}.tmp",
            std::process::id(),
            nonce
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);

        match options.open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(PebbleStoreError::io(
                    PebbleStoreErrorCode::WriteFailed,
                    "Could not create a private pebble configuration file.",
                    error,
                ));
            }
        }
    }

    Err(PebbleStoreError::io(
        PebbleStoreErrorCode::WriteFailed,
        "Could not allocate a private pebble configuration file.",
        io::Error::new(ErrorKind::AlreadyExists, "temporary file collision"),
    ))
}

fn sync_directory(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}

fn parse_document(raw: &str) -> Result<PebbleStoreDocument, PebbleStoreError> {
    let document: PebbleStoreDocument =
        serde_json::from_str(raw).map_err(PebbleStoreError::corrupt)?;
    document.validate_schema()?;
    document.validate_safe_config()?;

    Ok(document)
}
