//! End-to-end synthetic evidence for the standalone DRA.

use std::path::PathBuf;

use device_research_assistant::analysis::{self, Confidence};
use device_research_assistant::capture;
use device_research_assistant::cli;
use device_research_assistant::error::DraError;
use device_research_assistant::inventory;
use device_research_assistant::model::ResearchSession;
use device_research_assistant::paths::DraPaths;
use device_research_assistant::report;
use device_research_assistant::scaffold;
use device_research_assistant::session::{self, SessionSpec};
use peripheral_hub_device_sdk::{
    DeviceDescriptor, DeviceInstanceId, DeviceInterfaceId, HidUsage, TransportKind,
};
use serde_json::json;
use tempfile::TempDir;

const NOW: u64 = 1_700_000_000;

fn paths(temp: &TempDir) -> DraPaths {
    DraPaths::new(Some(temp.path().to_path_buf()))
}

fn spec(id: &str) -> SessionSpec {
    SessionSpec {
        id: Some(id.to_owned()),
        name: "Synthetic RGB study".into(),
        device: "Fictitious Pad".into(),
        firmware: Some("1.0".into()),
        software: Some("Synthetic configurator".into()),
        sha256: Some("a".repeat(64)),
        goal: "Identify which payload offset selects red, green and blue.".into(),
        consent: true,
    }
}

fn create(paths: &DraPaths, id: &str) -> ResearchSession {
    session::create(paths, spec(id), NOW).expect("session")
}

fn write_capture(temp: &TempDir, name: &str, value: &serde_json::Value) -> PathBuf {
    let path = temp.path().join(name);
    std::fs::write(&path, serde_json::to_vec_pretty(value).expect("json")).expect("write");
    path
}

fn synthetic_capture() -> serde_json::Value {
    json!({
        "capture_schema_version": 1,
        "source": "synthetic",
        "description": "baseline then one safe RGB UI change at a time",
        "events": [
            {"timestamp_us": 10, "direction": "out", "transfer": "interrupt", "interface": 0, "report_id": 1, "payload_hex": "1100aa00", "action": "baseline"},
            {"timestamp_us": 20, "direction": "out", "transfer": "interrupt", "interface": 0, "report_id": 1, "payload_hex": "1101aa00", "action": "set-red"},
            {"timestamp_us": 30, "direction": "out", "transfer": "interrupt", "interface": 0, "report_id": 1, "payload_hex": "1102aa00", "action": "set-green"},
            {"timestamp_us": 40, "direction": "out", "transfer": "interrupt", "interface": 0, "report_id": 1, "payload_hex": "1103aa00", "action": "set-blue"},
            {"timestamp_us": 50, "direction": "out", "transfer": "control", "interface": 0, "report_id": 2, "payload_hex": "aa"},
            {"timestamp_us": 60, "direction": "out", "transfer": "control", "interface": 0, "report_id": 2, "payload_hex": "aa"},
            {"timestamp_us": 70, "direction": "in", "transfer": "interrupt", "interface": 0, "report_id": 1, "payload_hex": "00"}
        ]
    })
}

#[test]
fn synthetic_session_produces_inferred_report_and_safe_scaffold() {
    let temp = TempDir::new().expect("temp");
    let paths = paths(&temp);
    let session = create(&paths, "synthetic-rgb");
    let input = write_capture(&temp, "capture.json", &synthetic_capture());

    let (session, meta) = capture::import(&paths, &session.id, &input, NOW + 1).expect("import");
    assert_eq!(meta.file, "capture-0001.json");
    assert_eq!(meta.event_count, 7);
    assert_eq!(session.captures.len(), 1);
    assert!(
        paths
            .session_dir("synthetic-rgb")
            .join(&meta.file)
            .is_file()
    );

    let analysis_doc = analysis::analyze(&paths, &session.id, NOW + 2).expect("analyze");
    assert_eq!(analysis_doc.capture_count, 1);
    assert_eq!(analysis_doc.event_count, 7);
    assert!(
        analysis_doc
            .variable_offsets
            .iter()
            .any(|offset| offset.offset == 1
                && offset.values == vec![0, 1, 2, 3]
                && offset.action_values.len() == 4)
    );
    let inferred = analysis_doc
        .hypotheses
        .iter()
        .find(|hypothesis| hypothesis.confidence == Confidence::Inferred)
        .expect("inferred offset hypothesis");
    assert_eq!(inferred.offset, Some(1));
    assert!(inferred.statement.contains("set-red"));
    assert!(
        analysis_doc
            .hypotheses
            .iter()
            .any(
                |hypothesis| hypothesis.kind == analysis::HypothesisKind::RepeatedPayload
                    && hypothesis.confidence == Confidence::Unverified
            )
    );
    assert!(
        analysis_doc
            .hypotheses
            .iter()
            .all(|hypothesis| hypothesis.confidence != Confidence::ConfirmedOnHardware)
    );

    let (report_doc, report_output) =
        report::generate(&paths, &session.id, None, NOW + 3).expect("report");
    assert_eq!(report_doc.analysis.session_id, session.id);
    let markdown = std::fs::read_to_string(&report_output.markdown).expect("markdown");
    let report_json = std::fs::read_to_string(&report_output.json).expect("report json");
    assert!(markdown.contains("inferred"));
    assert!(!markdown.contains("- **confirmed-on-hardware**"));
    assert!(!markdown.contains("secret"));
    assert!(!markdown.contains(input.to_string_lossy().as_ref()));
    assert!(report_json.contains("\"report_schema_version\": 1"));
    assert!(report_json.contains("\"confidence\": \"inferred\""));

    let (scaffold_analysis, scaffold_output) =
        scaffold::generate(&paths, &session.id, None, false, NOW + 4).expect("scaffold");
    assert_eq!(
        scaffold_analysis.hypotheses.len(),
        analysis_doc.hypotheses.len()
    );
    let manifest =
        std::fs::read_to_string(&scaffold_output.manifest_example).expect("manifest template");
    for line in manifest.lines().filter(|line| !line.trim().is_empty()) {
        assert!(
            line.trim_start().starts_with('#'),
            "template line is loadable: {line}"
        );
    }
    let notes = std::fs::read_to_string(&scaffold_output.readme).expect("notes");
    assert!(notes.contains("not a driver"));
    assert!(notes.contains("inferred"));
    assert!(!notes.contains("1101aa00"));
    assert!(
        !std::fs::read_to_string(&scaffold_output.hypotheses)
            .expect("hypotheses")
            .contains("1101aa00")
    );
}

#[test]
fn invalid_captures_fail_before_the_session_is_updated() {
    let temp = TempDir::new().expect("temp");
    let paths = paths(&temp);
    let session = create(&paths, "reject-invalid");

    let invalid = [
        json!({"capture_schema_version": 2, "source": "synthetic", "events": []}),
        json!({"capture_schema_version": 1, "source": "synthetic", "events": [
            {"timestamp_us": 1, "direction": "out", "transfer": "interrupt", "interface": 0, "payload_hex": "abc"}
        ]}),
        json!({"capture_schema_version": 1, "source": "synthetic", "events": [
            {"timestamp_us": 1, "direction": "out", "transfer": "interrupt", "interface": 0, "payload_hex": "aa", "private": true}
        ]}),
        json!({"capture_schema_version": 1, "source": "Synthetic", "events": [
            {"timestamp_us": 1, "direction": "out", "transfer": "interrupt", "interface": 0, "payload_hex": "aa"}
        ]}),
        json!({"capture_schema_version": 1, "source": "synthetic", "events": []}),
    ];
    for (index, value) in invalid.iter().enumerate() {
        let input = write_capture(&temp, &format!("invalid-{index}.json"), value);
        assert!(
            capture::import(&paths, &session.id, &input, NOW + 1).is_err(),
            "case {index} should fail"
        );
        assert!(
            session::load(&paths, &session.id)
                .expect("session")
                .captures
                .is_empty(),
            "case {index} mutated the session"
        );
    }

    let too_large = json!({
        "capture_schema_version": 1,
        "source": "synthetic",
        "events": [{
            "timestamp_us": 1,
            "direction": "out",
            "transfer": "interrupt",
            "interface": 0,
            "payload_hex": "aa".repeat(4097)
        }]
    });
    let input = write_capture(&temp, "too-large.json", &too_large);
    assert!(capture::import(&paths, &session.id, &input, NOW + 1).is_err());
    assert!(
        session::load(&paths, &session.id)
            .expect("session")
            .captures
            .is_empty()
    );
}

#[test]
fn session_ids_and_consent_are_enforced() {
    let temp = TempDir::new().expect("temp");
    let paths = paths(&temp);
    let mut invalid = spec("../escape");
    invalid.id = Some("../escape".into());
    assert!(matches!(
        session::create(&paths, invalid, NOW),
        Err(DraError::InvalidArgument(_))
    ));

    let no_consent = SessionSpec {
        consent: false,
        ..spec("no-consent")
    };
    assert!(matches!(
        session::create(&paths, no_consent, NOW),
        Err(DraError::ConsentRequired)
    ));

    create(&paths, "duplicate");
    assert!(matches!(
        session::create(&paths, spec("duplicate"), NOW),
        Err(DraError::SessionExists(_))
    ));
}

#[test]
fn out_of_order_captures_are_normalized_and_flagged() {
    let temp = TempDir::new().expect("temp");
    let paths = paths(&temp);
    let session = create(&paths, "reordered");
    let value = json!({
        "capture_schema_version": 1,
        "source": "synthetic",
        "events": [
            {"timestamp_us": 30, "direction": "out", "transfer": "interrupt", "interface": 0, "payload_hex": "aa"},
            {"timestamp_us": 10, "direction": "out", "transfer": "interrupt", "interface": 0, "payload_hex": "bb"}
        ]
    });
    let input = write_capture(&temp, "out-of-order.json", &value);
    capture::import(&paths, &session.id, &input, NOW + 1).expect("import");
    let stored: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(paths.session_dir(&session.id).join("capture-0001.json"))
            .expect("capture"),
    )
    .expect("json");
    let timestamps: Vec<u64> = stored["events"]
        .as_array()
        .expect("events")
        .iter()
        .map(|event| event["timestamp_us"].as_u64().expect("timestamp"))
        .collect();
    assert_eq!(timestamps, vec![10, 30]);

    let analysis_doc = analysis::analyze(&paths, &session.id, NOW + 2).expect("analyze");
    assert!(analysis_doc.reordered);
    assert!(
        analysis_doc
            .limitations
            .iter()
            .any(|limitation| limitation.contains("reordered"))
    );
    assert!(
        analysis_doc
            .hypotheses
            .iter()
            .all(|hypothesis| hypothesis.confidence != Confidence::Inferred)
    );
}

#[test]
fn inventory_omits_serials_and_private_paths() {
    let descriptor = DeviceDescriptor {
        instance_id: DeviceInstanceId::new("unit-1").expect("instance"),
        interface_id: DeviceInterfaceId::new("iface-1").expect("interface"),
        vendor_id: 0x1234,
        product_id: 0xabcd,
        transport: TransportKind::Hid,
        path: b"/dev/hidraw-private".to_vec(),
        interface_number: Some(0),
        usage: Some(HidUsage {
            page: 0xff00,
            usage: 1,
        }),
        manufacturer: Some("ACME".into()),
        product: Some("Pad".into()),
        serial_number: Some("secret-serial".into()),
        firmware_version: Some("1.2.3".into()),
    };
    let entries = inventory::from_descriptors(&[descriptor]);
    let encoded = serde_json::to_string(&entries).expect("json");
    let value: serde_json::Value = serde_json::from_str(&encoded).expect("json");
    assert_eq!(value[0]["vendor_id"], 0x1234);
    assert_eq!(value[0]["manufacturer"], "ACME");
    assert_eq!(value[0]["product"], "Pad");
    assert!(!encoded.contains("secret-serial"));
    assert!(!encoded.contains("hidraw-private"));
}

#[test]
fn cli_requires_consent_and_rejects_unknown_commands() {
    let temp = TempDir::new().expect("temp");
    let base = temp.path().to_string_lossy().into_owned();
    let consent_error = cli::run(vec![
        "--data-dir".into(),
        base.clone(),
        "session".into(),
        "create".into(),
        "--name".into(),
        "No consent".into(),
        "--device".into(),
        "Pad".into(),
        "--goal".into(),
        "Study".into(),
    ])
    .expect_err("consent");
    assert!(matches!(consent_error, DraError::ConsentRequired));
    assert!(cli::run(vec!["unknown".into()]).is_err());
    assert!(cli::run(vec!["--help".into()]).is_ok());
}

#[test]
fn analysis_rejects_a_session_without_captures() {
    let temp = TempDir::new().expect("temp");
    let paths = paths(&temp);
    let session = create(&paths, "empty");
    assert!(matches!(
        analysis::analyze(&paths, &session.id, NOW),
        Err(DraError::NoCapture(_))
    ));
}

#[test]
fn documented_research_schemas_parse_and_declare_the_current_versions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas");
    for (name, version) in [
        ("research-capture.schema.json", 1),
        ("research-report.schema.json", 1),
    ] {
        let text = std::fs::read_to_string(root.join(name)).expect("schema file");
        let value: serde_json::Value = serde_json::from_str(&text).expect("schema JSON");
        assert!(value["$schema"].as_str().is_some(), "{name}");
        if name.contains("capture") {
            assert_eq!(
                value["properties"]["capture_schema_version"]["const"],
                version
            );
        } else {
            assert_eq!(
                value["properties"]["report_schema_version"]["const"],
                version
            );
        }
    }
}
