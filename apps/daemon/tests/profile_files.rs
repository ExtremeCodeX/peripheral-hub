//! Local, schema-versioned profile JSON export/import; no hardware or network.
use peripheral_hub_daemon::{export_profile, import_profile};
use peripheral_hub_device_sdk::{DeviceId, LightingCommand, LightingEffect, RgbColor};
use peripheral_hub_profiles::{PROFILE_SCHEMA_VERSION, Profile, ProfileId, ProfileRepository};

fn profile() -> Profile {
    Profile {
        id: ProfileId::new("night").unwrap(),
        name: "Night".into(),
        device_model: Some(DeviceId::new("model:keyboard").unwrap()),
        lighting: Some(LightingCommand {
            zone: "whole-device".into(),
            color: RgbColor {
                red: 10,
                green: 20,
                blue: 30,
            },
            brightness: Some(40),
            effect: LightingEffect::Static,
        }),
    }
}

#[test]
fn export_then_import_roundtrips_into_a_repository() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("night.json");
    let value = profile();

    export_profile(&path, &value).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains(&format!(
        "\"profile_schema_version\": {PROFILE_SCHEMA_VERSION}"
    )));

    let imported = import_profile(&path).unwrap();
    assert_eq!(imported, value);

    // The imported document is a fully validated profile that persists unchanged.
    let mut repository = ProfileRepository::open_in_memory().unwrap();
    repository.save(&imported).unwrap();
    assert_eq!(repository.get(&value.id).unwrap(), value);
}

#[test]
fn import_rejects_future_versions_malformed_and_missing_files() {
    let directory = tempfile::tempdir().unwrap();
    let valid = profile().to_json().unwrap();

    let future = directory.path().join("future.json");
    let bumped = valid.replace(
        &format!("\"profile_schema_version\": {PROFILE_SCHEMA_VERSION}"),
        &format!("\"profile_schema_version\": {}", PROFILE_SCHEMA_VERSION + 1),
    );
    std::fs::write(&future, bumped).unwrap();
    assert!(import_profile(&future).is_err());

    let malformed = directory.path().join("malformed.json");
    std::fs::write(&malformed, b"not json").unwrap();
    assert!(import_profile(&malformed).is_err());

    assert!(import_profile(&directory.path().join("missing.json")).is_err());
}
