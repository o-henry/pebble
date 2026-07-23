use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::fs::{symlink, PermissionsExt};

use crate::{
    pebble_store::{
        PebbleStore, PebbleStoreDocument, PebbleStoreErrorCode, PebbleStoreMigration,
        StoredCaptureConfig, StoredPebbleRegion, PEBBLE_STORE_SCHEMA_VERSION,
    },
    region_selection_types::PhysicalRegion,
};

#[test]
fn store_serializes_safe_config_fields_only() {
    let document = sample_document();
    let raw = serde_json::to_string(&document).expect("serialize config");

    assert!(raw.contains("schemaVersion"));
    assert!(raw.contains("regions"));
    assert!(raw.contains("Dashboard total"));
    assert!(raw.contains("fps"));
    for forbidden in [
        "bytes",
        "frame",
        "preview",
        "image",
        "screenshot",
        "ocr",
        "history",
        "prompt",
        "token",
        "cookie",
    ] {
        assert!(!raw.to_ascii_lowercase().contains(forbidden), "{forbidden}");
    }
}

#[test]
fn save_and_load_restore_named_regions_without_frame_data() {
    let path = test_store_path("roundtrip");
    let store = PebbleStore::new(path.clone());
    let saved = store.save(&sample_document()).expect("save config");
    let loaded = store.load_or_default().expect("load config");

    assert_eq!(saved, loaded);
    assert_eq!(loaded.restore_named_regions(), sample_document().regions);
    assert!(!fs::read_to_string(path)
        .expect("store file")
        .to_ascii_lowercase()
        .contains("frame"));
}

#[cfg(unix)]
#[test]
fn load_migrates_existing_store_to_private_permissions() {
    let path = test_store_path("migrate-permissions");
    let parent = path.parent().expect("parent");
    fs::create_dir_all(parent).expect("test dir");
    fs::write(
        &path,
        serde_json::to_string(&sample_document()).expect("json"),
    )
    .expect("write config");
    fs::set_permissions(parent, fs::Permissions::from_mode(0o755)).expect("directory mode");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("file mode");

    PebbleStore::new(path.clone())
        .load_or_default()
        .expect("load config");

    assert_eq!(
        fs::metadata(parent)
            .expect("directory")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(path).expect("file").permissions().mode() & 0o777,
        0o600
    );
}

#[cfg(unix)]
#[test]
fn save_uses_private_permissions_and_leaves_no_temp_file() {
    let path = test_store_path("private-permissions");
    let store = PebbleStore::new(path.clone());

    store.save(&sample_document()).expect("save config");

    let file_mode = fs::metadata(&path)
        .expect("store metadata")
        .permissions()
        .mode()
        & 0o777;
    let directory_mode = fs::metadata(path.parent().expect("parent"))
        .expect("directory metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(file_mode, 0o600);
    assert_eq!(directory_mode, 0o700);
    assert_eq!(
        fs::read_dir(path.parent().expect("parent"))
            .expect("read directory")
            .count(),
        1
    );
}

#[cfg(unix)]
#[test]
fn load_rejects_symbolic_link_store() {
    let path = test_store_path("symlink-load");
    let parent = path.parent().expect("parent");
    fs::create_dir_all(parent).expect("test dir");
    let target = parent.join("outside.json");
    fs::write(
        &target,
        serde_json::to_string(&sample_document()).expect("json"),
    )
    .expect("write target");
    symlink(&target, &path).expect("create symlink");
    let store = PebbleStore::new(path);

    let error = store.load_or_default().expect_err("reject symlink");

    assert_eq!(error.code, PebbleStoreErrorCode::ReadFailed);
}

#[cfg(unix)]
#[test]
fn save_rejects_symbolic_link_destination_without_touching_target() {
    let path = test_store_path("symlink-save");
    let parent = path.parent().expect("parent");
    fs::create_dir_all(parent).expect("test dir");
    let target = parent.join("outside.json");
    fs::write(&target, "unchanged").expect("write target");
    symlink(&target, &path).expect("create symlink");
    let store = PebbleStore::new(path);

    let error = store.save(&sample_document()).expect_err("reject symlink");

    assert_eq!(error.code, PebbleStoreErrorCode::WriteFailed);
    assert_eq!(fs::read_to_string(target).expect("target"), "unchanged");
}

#[test]
fn load_rejects_store_larger_than_one_mibibyte() {
    let path = test_store_path("oversized");
    fs::create_dir_all(path.parent().expect("parent")).expect("test dir");
    fs::write(&path, vec![b' '; 1_048_577]).expect("write oversized config");
    let store = PebbleStore::new(path);

    let error = store.load_or_default().expect_err("reject oversized");

    assert_eq!(error.code, PebbleStoreErrorCode::InvalidConfig);
}

#[test]
fn load_missing_store_returns_default_document() {
    let store = PebbleStore::new(test_store_path("missing"));

    let document = store.load_or_default().expect("default config");

    assert_eq!(document, PebbleStoreDocument::default());
}

#[test]
fn corrupted_store_returns_recoverable_error() {
    let path = test_store_path("corrupt");
    fs::create_dir_all(path.parent().expect("parent")).expect("test dir");
    fs::write(&path, "{not-json").expect("write corrupt config");
    let store = PebbleStore::new(path);

    let error = store.load_or_default().expect_err("corrupt data");

    assert_eq!(error.code, PebbleStoreErrorCode::CorruptData);
    assert!(error.recoverable);
}

#[test]
fn newer_schema_returns_recoverable_error() {
    let path = test_store_path("newer-schema");
    let store = PebbleStore::new(path.clone());
    fs::create_dir_all(path.parent().expect("parent")).expect("test dir");
    fs::write(
        &path,
        format!(
            r#"{{"schemaVersion":{},"migration":{{"latestSupportedSchema":{},"lastMigratedFrom":null}},"regions":[]}}"#,
            PEBBLE_STORE_SCHEMA_VERSION + 1,
            PEBBLE_STORE_SCHEMA_VERSION + 1
        ),
    )
    .expect("write newer schema");

    let error = store.load_or_default().expect_err("newer schema");

    assert_eq!(error.code, PebbleStoreErrorCode::UnsupportedSchema);
    assert!(error.recoverable);
}

#[test]
fn save_rejects_newer_schema_before_overwriting_store() {
    let store = PebbleStore::new(test_store_path("reject-newer-save"));
    let mut document = sample_document();
    document.schema_version = PEBBLE_STORE_SCHEMA_VERSION + 1;

    let error = store.save(&document).expect_err("newer schema");

    assert_eq!(error.code, PebbleStoreErrorCode::UnsupportedSchema);
    assert!(error.recoverable);
}

#[test]
fn save_rejects_out_of_contract_fps() {
    let store = PebbleStore::new(test_store_path("invalid-fps"));
    let mut document = sample_document();
    document.regions[0].capture.fps = 99;

    let error = store.save(&document).expect_err("invalid fps");

    assert_eq!(error.code, PebbleStoreErrorCode::InvalidConfig);
    assert!(error.recoverable);
}

#[test]
fn save_rejects_empty_region() {
    let store = PebbleStore::new(test_store_path("invalid-region"));
    let mut document = sample_document();
    document.regions[0].region.width = 0;

    let error = store.save(&document).expect_err("invalid region");

    assert_eq!(error.code, PebbleStoreErrorCode::InvalidConfig);
    assert!(error.recoverable);
}

#[test]
fn save_rejects_more_regions_than_active_tile_limit() {
    let store = PebbleStore::new(test_store_path("too-many-regions"));
    let mut document = sample_document();
    for index in 1..4 {
        let mut region = document.regions[0].clone();
        region.id = format!("region-{index}");
        region.name = format!("Region {index}");
        document.regions.push(region);
    }

    let error = store.save(&document).expect_err("too many regions");

    assert_eq!(error.code, PebbleStoreErrorCode::InvalidConfig);
    assert!(error.recoverable);
}

#[test]
fn load_rejects_out_of_contract_stored_config() {
    let path = test_store_path("invalid-load");
    let mut document = sample_document();
    document.regions[0].capture.fps = 99;
    fs::create_dir_all(path.parent().expect("parent")).expect("test dir");
    fs::write(
        &path,
        serde_json::to_string(&document).expect("invalid config json"),
    )
    .expect("write invalid config");
    let store = PebbleStore::new(path);

    let error = store.load_or_default().expect_err("invalid stored config");

    assert_eq!(error.code, PebbleStoreErrorCode::InvalidConfig);
    assert!(error.recoverable);
}

fn sample_document() -> PebbleStoreDocument {
    PebbleStoreDocument {
        schema_version: PEBBLE_STORE_SCHEMA_VERSION,
        migration: PebbleStoreMigration::default(),
        regions: vec![StoredPebbleRegion {
            id: "dashboard-total".to_string(),
            name: "Dashboard total".to_string(),
            region: PhysicalRegion {
                monitor_id: "main".to_string(),
                x: 10,
                y: 20,
                width: 600,
                height: 300,
                source_window: None,
            },
            capture: StoredCaptureConfig { fps: 1 },
        }],
    }
}

fn test_store_path(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir()
        .join("pebble-store-tests")
        .join(format!("{name}-{nonce}"))
        .join("pebbles.json")
}
