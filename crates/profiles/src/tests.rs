use peripheral_hub_device_sdk::{DeviceId, LightingCommand, LightingEffect, RgbColor};
use peripheral_hub_storage::Database;

use crate::*;

fn lighting(brightness: Option<u8>) -> LightingCommand {
    LightingCommand {
        zone: "whole-device".into(),
        color: RgbColor {
            red: 10,
            green: 20,
            blue: 30,
        },
        brightness,
        effect: LightingEffect::Static,
    }
}

fn profile(id: &str, name: &str) -> Profile {
    Profile {
        id: ProfileId::new(id).unwrap(),
        name: name.into(),
        device_model: Some(DeviceId::new("model:keyboard").unwrap()),
        lighting: Some(lighting(None)),
    }
}

#[test]
fn profile_ids_are_bounded_tokens() {
    assert!(ProfileId::new("dark-mode").is_ok());
    assert!(ProfileId::new("").is_err());
    assert!(ProfileId::new("has space").is_err());
    assert!(ProfileId::new("path/with/slash").is_err());
    assert!(ProfileId::new("x".repeat(129)).is_err());
}

#[test]
fn validation_rejects_malformed_fields() {
    let mut value = profile("p1", "Gaming");
    assert!(value.validate().is_ok());

    value.name = String::new();
    assert!(matches!(value.validate(), Err(ProfileError::Invalid(_))));
    value.name = "bad\nname".into();
    assert!(value.validate().is_err());
    value.name = "x".repeat(65);
    assert!(value.validate().is_err());

    value = profile("p1", "Gaming");
    value.lighting = Some(lighting(Some(101)));
    assert!(value.validate().is_err());

    value.lighting = Some(LightingCommand {
        zone: "not a token".into(),
        ..lighting(None)
    });
    assert!(value.validate().is_err());
}

#[test]
fn json_export_roundtrips_with_explicit_version() {
    let value = profile("p1", "Gaming");
    let json = value.to_json().unwrap();
    assert!(json.contains("\"profile_schema_version\": 1"));
    assert_eq!(Profile::from_json(&json).unwrap(), value);
}

#[test]
fn import_rejects_unknown_version_and_unknown_fields() {
    let future = format!(
        "{{\"profile_schema_version\": {}, \"profile\": {}}}",
        PROFILE_SCHEMA_VERSION + 1,
        serde_json::to_string(&profile("p1", "Gaming")).unwrap()
    );
    assert!(matches!(
        Profile::from_json(&future),
        Err(ProfileError::UnsupportedVersion)
    ));
    assert!(matches!(
        Profile::from_json("{\"profile_schema_version\": 1, \"extra\": 1}"),
        Err(ProfileError::InvalidJson)
    ));
    assert!(matches!(
        Profile::from_json("not json"),
        Err(ProfileError::InvalidJson)
    ));
}

#[test]
fn repository_saves_lists_replaces_and_deletes() {
    let mut repository = ProfileRepository::open_in_memory().unwrap();
    assert_eq!(repository.count().unwrap(), 0);

    repository.save(&profile("p1", "Gaming")).unwrap();
    repository.save(&profile("p2", "Work")).unwrap();
    assert_eq!(repository.count().unwrap(), 2);
    assert_eq!(
        repository.get(&ProfileId::new("p1").unwrap()).unwrap().name,
        "Gaming"
    );

    // Saving an existing id replaces it rather than duplicating.
    let mut renamed = profile("p1", "Gaming Pro");
    renamed.lighting = None;
    repository.save(&renamed).unwrap();
    assert_eq!(repository.count().unwrap(), 2);
    assert_eq!(
        repository.get(&ProfileId::new("p1").unwrap()).unwrap(),
        renamed
    );

    let summaries = repository.list().unwrap();
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].name, "Gaming Pro");
    assert_eq!(summaries[1].name, "Work");

    repository.delete(&ProfileId::new("p2").unwrap()).unwrap();
    assert_eq!(repository.count().unwrap(), 1);
    assert!(matches!(
        repository.delete(&ProfileId::new("p2").unwrap()),
        Err(ProfileError::NotFound)
    ));
    assert!(matches!(
        repository.get(&ProfileId::new("missing").unwrap()),
        Err(ProfileError::NotFound)
    ));
}

#[test]
fn repository_rejects_invalid_profiles_before_writing() {
    let mut repository = ProfileRepository::open_in_memory().unwrap();
    let mut invalid = profile("p1", "Gaming");
    invalid.name = String::new();
    assert!(matches!(
        repository.save(&invalid),
        Err(ProfileError::Invalid(_))
    ));
    assert_eq!(repository.count().unwrap(), 0);
}

#[test]
fn file_repository_persists_across_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("profiles.sqlite");
    {
        let mut repository = ProfileRepository::open_file(&path).unwrap();
        repository.save(&profile("p1", "Gaming")).unwrap();
    }
    let _ = Database::open(&path).unwrap(); // database file exists and reopens
    let repository = ProfileRepository::open_file(&path).unwrap();
    assert_eq!(repository.count().unwrap(), 1);
}
