use peripheral_hub_profiles::ProfileId;
use peripheral_hub_storage::Database;

use crate::*;

fn key(name: &str, press: KeyPress) -> MacroAction {
    MacroAction::Key {
        key: name.into(),
        press,
    }
}

fn macro_value(id: &str, name: &str) -> Macro {
    Macro {
        id: MacroId::new(id).unwrap(),
        name: name.into(),
        actions: vec![key("f13", KeyPress::Tap), MacroAction::Delay { ms: 25 }],
        repeat: MacroRepeat {
            times: 2,
            interval_ms: 10,
        },
        profile: None,
    }
}

#[test]
fn macro_ids_are_bounded_tokens() {
    assert!(MacroId::new("open-browser").is_ok());
    assert!(MacroId::new("").is_err());
    assert!(MacroId::new("has space").is_err());
    assert!(MacroId::new("path/with/slash").is_err());
    assert!(MacroId::new("x".repeat(129)).is_err());
}

#[test]
fn validation_rejects_malformed_fields() {
    let mut value = macro_value("m1", "Combo");
    assert!(value.validate().is_ok());

    value.name = String::new();
    assert!(matches!(value.validate(), Err(MacroError::Invalid(_))));
    value.name = "bad\nname".into();
    assert!(value.validate().is_err());
    value.name = "x".repeat(65);
    assert!(value.validate().is_err());

    value = macro_value("m1", "Combo");
    value.actions.clear();
    assert!(value.validate().is_err());

    value = macro_value("m1", "Combo");
    value.actions = vec![MacroAction::Delay { ms: 0 }];
    assert!(value.validate().is_err());
    value.actions = vec![MacroAction::Delay {
        ms: MAX_DELAY_MS + 1,
    }];
    assert!(value.validate().is_err());

    value = macro_value("m1", "Combo");
    value.actions = vec![key("not a token", KeyPress::Tap)];
    assert!(value.validate().is_err());
}

#[test]
fn validation_bounds_actions_repetition_and_duration() {
    let mut value = macro_value("m1", "Combo");

    value.actions = vec![MacroAction::Delay { ms: 1 }; MAX_MACRO_ACTIONS + 1];
    assert!(value.validate().is_err());

    value = macro_value("m1", "Combo");
    value.repeat.times = 0;
    assert!(value.validate().is_err());
    value.repeat.times = MAX_MACRO_REPEAT + 1;
    assert!(value.validate().is_err());
    value.repeat.times = 1;
    value.repeat.interval_ms = MAX_MACRO_INTERVAL_MS + 1;
    assert!(value.validate().is_err());

    // One iteration beyond the per-iteration ceiling is rejected.
    value = macro_value("m1", "Combo");
    value.repeat = MacroRepeat {
        times: 1,
        interval_ms: 0,
    };
    value.actions = vec![MacroAction::Delay { ms: MAX_DELAY_MS }; 11];
    assert!(value.iteration_ms() > MAX_MACRO_ITERATION_MS);
    assert!(value.validate().is_err());

    // Finite but excessive total duration across repeats is rejected.
    value = macro_value("m1", "Combo");
    value.actions = vec![MacroAction::Delay { ms: MAX_DELAY_MS }];
    value.repeat = MacroRepeat {
        times: MAX_MACRO_REPEAT,
        interval_ms: MAX_MACRO_INTERVAL_MS,
    };
    assert!(value.total_ms() > MAX_MACRO_TOTAL_MS);
    assert!(value.validate().is_err());
}

#[test]
fn json_export_roundtrips_with_explicit_version() {
    let mut value = macro_value("m1", "Combo");
    value.profile = Some(ProfileId::new("gaming").unwrap());
    let json = value.to_json().unwrap();
    assert!(json.contains("\"macro_schema_version\": 1"));
    assert_eq!(Macro::from_json(&json).unwrap(), value);
}

#[test]
fn import_rejects_unknown_version_and_unknown_fields() {
    let future = format!(
        "{{\"macro_schema_version\": {}, \"macro\": {}}}",
        MACRO_SCHEMA_VERSION + 1,
        serde_json::to_string(&macro_value("m1", "Combo")).unwrap()
    );
    assert!(matches!(
        Macro::from_json(&future),
        Err(MacroError::UnsupportedVersion)
    ));
    assert!(matches!(
        Macro::from_json("{\"macro_schema_version\": 1, \"extra\": 1}"),
        Err(MacroError::InvalidJson)
    ));
    // An out-of-range action is rejected during import, not stored as success.
    let oversized = "{\"macro_schema_version\": 1, \"macro\": {\"id\": \"m1\", \"name\": \"Combo\", \"actions\": [{\"kind\": \"delay\", \"ms\": 0}]}}";
    assert!(matches!(
        Macro::from_json(oversized),
        Err(MacroError::Invalid(_))
    ));
    assert!(matches!(
        Macro::from_json("not json"),
        Err(MacroError::InvalidJson)
    ));
    // Unknown fields are rejected by the strict structures and actions.
    let unknown_macro = "{\"macro_schema_version\": 1, \"macro\": {\"id\": \"m1\", \"name\": \"Combo\", \"actions\": [{\"kind\": \"delay\", \"ms\": 10}], \"bogus\": 1}}";
    assert!(matches!(
        Macro::from_json(unknown_macro),
        Err(MacroError::InvalidJson)
    ));
    let unknown_action = "{\"macro_schema_version\": 1, \"macro\": {\"id\": \"m1\", \"name\": \"Combo\", \"actions\": [{\"kind\": \"delay\", \"ms\": 10, \"bogus\": true}]}}";
    assert!(matches!(
        Macro::from_json(unknown_action),
        Err(MacroError::InvalidJson)
    ));
}

#[test]
fn repository_saves_lists_replaces_and_deletes() {
    let mut repository = MacroRepository::open_in_memory().unwrap();
    assert_eq!(repository.count().unwrap(), 0);

    repository.save(&macro_value("m1", "Combo")).unwrap();
    repository.save(&macro_value("m2", "Alpha")).unwrap();
    assert_eq!(repository.count().unwrap(), 2);
    assert_eq!(
        repository.get(&MacroId::new("m1").unwrap()).unwrap().name,
        "Combo"
    );

    // Saving an existing id replaces it rather than duplicating.
    let mut renamed = macro_value("m1", "Combo Pro");
    renamed.actions = vec![key("f14", KeyPress::Down)];
    repository.save(&renamed).unwrap();
    assert_eq!(repository.count().unwrap(), 2);
    assert_eq!(
        repository.get(&MacroId::new("m1").unwrap()).unwrap(),
        renamed
    );

    let summaries = repository.list().unwrap();
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].name, "Alpha");
    assert_eq!(summaries[0].action_count, 2);

    repository.delete(&MacroId::new("m2").unwrap()).unwrap();
    assert_eq!(repository.count().unwrap(), 1);
    assert!(matches!(
        repository.delete(&MacroId::new("m2").unwrap()),
        Err(MacroError::NotFound)
    ));
    assert!(matches!(
        repository.get(&MacroId::new("missing").unwrap()),
        Err(MacroError::NotFound)
    ));
}

#[test]
fn repository_rejects_invalid_macros_before_writing() {
    let mut repository = MacroRepository::open_in_memory().unwrap();
    let mut invalid = macro_value("m1", "Combo");
    invalid.actions.clear();
    assert!(matches!(
        repository.save(&invalid),
        Err(MacroError::Invalid(_))
    ));
    assert_eq!(repository.count().unwrap(), 0);
}

#[test]
fn file_repository_persists_across_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("macros.sqlite");
    {
        let mut repository = MacroRepository::open_file(&path).unwrap();
        repository.save(&macro_value("m1", "Combo")).unwrap();
    }
    let _ = Database::open(&path).unwrap(); // database file exists and reopens
    let repository = MacroRepository::open_file(&path).unwrap();
    assert_eq!(repository.count().unwrap(), 1);
}
