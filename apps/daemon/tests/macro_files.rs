//! Local, schema-versioned macro JSON export/import; no hardware, network or
//! execution. Importing a macro only validates and stores it.
use peripheral_hub_daemon::{export_macro, import_macro};
use peripheral_hub_macros::{
    KeyPress, MACRO_SCHEMA_VERSION, Macro, MacroAction, MacroId, MacroRepeat, MacroRepository,
};
use peripheral_hub_profiles::ProfileId;

fn macro_value() -> Macro {
    Macro {
        id: MacroId::new("combo").unwrap(),
        name: "Combo".into(),
        actions: vec![
            MacroAction::Key {
                key: "f13".into(),
                press: KeyPress::Tap,
            },
            MacroAction::Delay { ms: 25 },
        ],
        repeat: MacroRepeat {
            times: 3,
            interval_ms: 10,
        },
        profile: Some(ProfileId::new("gaming").unwrap()),
    }
}

#[test]
fn export_then_import_roundtrips_into_a_repository() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("combo.json");
    let value = macro_value();

    export_macro(&path, &value).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains(&format!("\"macro_schema_version\": {MACRO_SCHEMA_VERSION}")));

    let imported = import_macro(&path).unwrap();
    assert_eq!(imported, value);

    // The imported document is a fully validated macro that persists unchanged.
    let mut repository = MacroRepository::open_in_memory().unwrap();
    repository.save(&imported).unwrap();
    assert_eq!(repository.get(&value.id).unwrap(), value);
}

#[test]
fn import_rejects_future_versions_malformed_and_missing_files() {
    let directory = tempfile::tempdir().unwrap();
    let valid = macro_value().to_json().unwrap();

    let future = directory.path().join("future.json");
    let bumped = valid.replace(
        &format!("\"macro_schema_version\": {MACRO_SCHEMA_VERSION}"),
        &format!("\"macro_schema_version\": {}", MACRO_SCHEMA_VERSION + 1),
    );
    std::fs::write(&future, bumped).unwrap();
    assert!(import_macro(&future).is_err());

    let malformed = directory.path().join("malformed.json");
    std::fs::write(&malformed, b"not json").unwrap();
    assert!(import_macro(&malformed).is_err());

    assert!(import_macro(&directory.path().join("missing.json")).is_err());
}
