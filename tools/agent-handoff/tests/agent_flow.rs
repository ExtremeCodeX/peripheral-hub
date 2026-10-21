//! End-to-end synthetic tests using the real DRA CLI and real manifest validator.

use std::path::{Path, PathBuf};

use device_research_assistant::cli as dra_cli;
use peripheral_hub_agent_handoff::error::HandoffError;
use peripheral_hub_agent_handoff::handoff;
use peripheral_hub_agent_handoff::manifest::validate_manifest;
use peripheral_hub_agent_handoff::model::{HandoffDocument, Readiness};
use peripheral_hub_agent_handoff::report_gate::{load_report, verify_handoff};
use serde_json::{Value, json};
use tempfile::TempDir;

const NOW: u64 = 1_700_000_000;

fn synthetic_capture() -> Value {
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

fn write_capture(temp: &TempDir, name: &str, value: &Value) -> PathBuf {
    let path = temp.path().join(name);
    std::fs::write(&path, serde_json::to_vec_pretty(value).expect("json")).expect("write");
    path
}

fn dra_report(temp: &TempDir, id: &str, capture: &Value) -> PathBuf {
    let data = temp.path().join(format!("dra-{id}"));
    let input = write_capture(temp, &format!("{id}.json"), capture);
    dra_cli::run(vec![
        "--data-dir".into(),
        data.to_string_lossy().into_owned(),
        "session".into(),
        "create".into(),
        "--id".into(),
        id.into(),
        "--name".into(),
        "Synthetic RGB study".into(),
        "--device".into(),
        "Fictitious Pad".into(),
        "--firmware".into(),
        "1.0".into(),
        "--goal".into(),
        "Identify which payload offset selects each color.".into(),
        "--consent".into(),
    ])
    .expect("session create");
    dra_cli::run(vec![
        "--data-dir".into(),
        data.to_string_lossy().into_owned(),
        "capture".into(),
        "import".into(),
        "--session".into(),
        id.into(),
        "--input".into(),
        input.to_string_lossy().into_owned(),
    ])
    .expect("capture import");
    let output = temp.path().join(format!("report-{id}"));
    dra_cli::run(vec![
        "--data-dir".into(),
        data.to_string_lossy().into_owned(),
        "report".into(),
        "--session".into(),
        id.into(),
        "--output".into(),
        output.to_string_lossy().into_owned(),
    ])
    .expect("report");
    output.join("report.json")
}

fn rewrite_report(report: &Path, mutate: impl FnOnce(&mut Value)) -> PathBuf {
    let mut value: Value =
        serde_json::from_slice(&std::fs::read(report).expect("report")).expect("json");
    mutate(&mut value);
    let path = report.with_file_name("tampered.json");
    std::fs::write(&path, serde_json::to_vec_pretty(&value).expect("json")).expect("write");
    path
}

fn example_manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/declarative-driver/manifest.toml")
}

#[test]
fn real_dra_report_produces_traceable_non_loadable_handoff() {
    let temp = TempDir::new().expect("temp");
    let report = dra_report(&temp, "synthetic-rgb", &synthetic_capture());
    let loaded = load_report(&report).expect("load report");
    assert_eq!(loaded.assessment.readiness, Readiness::ScaffoldReady);
    assert!(loaded.assessment.counts.inferred > 0);
    assert_eq!(loaded.assessment.counts.confirmed_on_hardware, 0);
    assert_eq!(loaded.assessment.report_sha256.len(), 64);

    let directory = temp.path().join("handoff");
    let (document, output) = handoff::create(
        &report,
        &directory,
        Some(&example_manifest()),
        false,
        NOW + 5,
    )
    .expect("create handoff");
    assert_eq!(document.assessment.readiness, Readiness::ScaffoldReady);
    assert!(!document.safety.driver_code_generated);
    assert!(!document.safety.loadable_manifest_generated);
    assert!(!document.safety.hardware_access_performed);
    assert!(!document.safety.vendor_execution_performed);
    assert!(!document.safety.packet_replay_performed);
    assert!(!document.safety.raw_capture_included);
    assert_eq!(
        std::fs::read(&output.evidence_report_json).expect("copied report"),
        std::fs::read(&report).expect("source report")
    );
    let manifest = document.manifest.as_ref().expect("manifest evidence");
    assert_eq!(manifest.operation_count, 2);
    assert_eq!(manifest.sdk_major, 1);
    assert!(manifest.validation_note.contains("not a report match"));

    let handoff_json: HandoffDocument =
        serde_json::from_slice(&std::fs::read(&output.handoff_json).expect("handoff json"))
            .expect("typed handoff");
    assert_eq!(handoff_json, document);
    let agent = std::fs::read_to_string(&output.agent_markdown).expect("agent");
    assert!(agent.contains("untrusted"));
    assert!(agent.contains("Required before implementation"));
    assert!(!agent.contains("1101aa00"));
    let protocol = std::fs::read_to_string(&output.protocol_markdown).expect("protocol");
    assert!(protocol.contains("SCAFFOLD ONLY"));
    assert!(protocol.contains("set-red"));
    assert!(!protocol.contains("1101aa00"));
    let validation = std::fs::read_to_string(&output.validation_markdown).expect("validation");
    assert!(validation.contains("not run"));
    assert!(validation.contains("Report integrity"));
    let evidence_markdown =
        std::fs::read_to_string(&output.evidence_report_markdown).expect("report md");
    assert!(evidence_markdown.contains("# Research report"));
    let template = std::fs::read_to_string(&output.manifest_example).expect("template");
    for line in template.lines().filter(|line| !line.trim().is_empty()) {
        assert!(
            line.trim_start().starts_with('#'),
            "manifest template line is loadable: {line}"
        );
    }

    let verified = verify_handoff(&directory).expect("verify");
    assert_eq!(verified.session_id, "synthetic-rgb");
    assert_eq!(verified.report_sha256, loaded.sha256);
    assert_eq!(verified.readiness, Readiness::ScaffoldReady);
}

#[test]
fn invalid_and_tampered_reports_are_rejected() {
    let temp = TempDir::new().expect("temp");
    let report = dra_report(&temp, "tamper-source", &synthetic_capture());

    let unknown = rewrite_report(&report, |value| {
        value["unexpected"] = json!(true);
    });
    assert!(matches!(
        load_report(&unknown),
        Err(HandoffError::InvalidReport(_))
    ));

    let event_total = rewrite_report(&report, |value| {
        value["session"]["captures"][0]["event_count"] = json!(1);
    });
    let error = load_report(&event_total).expect_err("event total mismatch");
    assert!(error.to_string().contains("event total"));

    let unknown_group = rewrite_report(&report, |value| {
        value["analysis"]["hypotheses"][0]["group"]["interface"] = json!(99);
    });
    let error = load_report(&unknown_group).expect_err("unknown group");
    assert!(error.to_string().contains("unknown group"));

    let duplicate = rewrite_report(&report, |value| {
        let hypothesis = value["analysis"]["hypotheses"][0].clone();
        value["analysis"]["hypotheses"]
            .as_array_mut()
            .expect("hypotheses")
            .push(hypothesis);
    });
    let error = load_report(&duplicate).expect_err("duplicate hypothesis");
    assert!(error.to_string().contains("duplicate or contradictory"));

    let output = temp.path().join("handoff");
    handoff::create(&report, &output, None, false, NOW + 5).expect("create");
    let copied = output.join("evidence/report.json");
    let mut bytes = std::fs::read(&copied).expect("copied");
    let position = bytes
        .windows(4)
        .position(|window| window == b"1.0\"")
        .expect("firmware bytes");
    bytes[position] = b'2';
    std::fs::write(&copied, bytes).expect("tamper");
    assert!(matches!(
        verify_handoff(&output),
        Err(HandoffError::Verification(_))
    ));
}

#[test]
fn handoff_metadata_cannot_drift_from_the_copied_report() {
    let temp = TempDir::new().expect("temp");
    let report = dra_report(&temp, "metadata-drift", &synthetic_capture());
    let output = temp.path().join("handoff");
    handoff::create(&report, &output, None, false, NOW + 5).expect("create");
    let handoff_path = output.join("handoff.json");
    let mut value: Value =
        serde_json::from_slice(&std::fs::read(&handoff_path).expect("handoff")).expect("json");
    value["captures"][0]["event_count"] = json!(99);
    std::fs::write(
        &handoff_path,
        serde_json::to_vec_pretty(&value).expect("json"),
    )
    .expect("write");
    let error = verify_handoff(&output).expect_err("metadata drift");
    assert!(error.to_string().contains("capture metadata"));
}

#[test]
fn insufficient_evidence_is_refused_before_any_output() {
    let temp = TempDir::new().expect("temp");
    let capture = json!({
        "capture_schema_version": 1,
        "source": "synthetic",
        "events": [
            {"timestamp_us": 10, "direction": "out", "transfer": "interrupt", "interface": 0, "payload_hex": "aa"}
        ]
    });
    let report = dra_report(&temp, "too-thin", &capture);
    let loaded = load_report(&report).expect("load");
    assert_eq!(loaded.assessment.readiness, Readiness::Insufficient);
    let output = temp.path().join("refused");
    let error =
        handoff::create(&report, &output, None, false, NOW + 5).expect_err("insufficient evidence");
    assert!(matches!(error, HandoffError::InsufficientEvidence(_)));
    assert!(!output.exists());
}

#[test]
fn unverified_only_evidence_gets_a_scaffold_with_more_evidence_gate() {
    let temp = TempDir::new().expect("temp");
    let capture = json!({
        "capture_schema_version": 1,
        "source": "synthetic",
        "events": [
            {"timestamp_us": 10, "direction": "out", "transfer": "interrupt", "interface": 0, "payload_hex": "aa"},
            {"timestamp_us": 20, "direction": "out", "transfer": "interrupt", "interface": 0, "payload_hex": "aa"}
        ]
    });
    let report = dra_report(&temp, "unverified", &capture);
    let (document, output) =
        handoff::create(&report, &temp.path().join("handoff"), None, false, NOW + 5)
            .expect("create");
    assert_eq!(document.assessment.readiness, Readiness::NeedsMoreEvidence);
    let protocol = std::fs::read_to_string(output.protocol_markdown).expect("protocol");
    assert!(protocol.contains("polling"));
    assert!(!protocol.contains("1101aa00"));
}

#[test]
fn output_requires_force_and_manifest_validation_is_structural() {
    let temp = TempDir::new().expect("temp");
    let report = dra_report(&temp, "force-output", &synthetic_capture());
    let output = temp.path().join("handoff");
    std::fs::create_dir_all(&output).expect("dir");
    std::fs::write(output.join("unrelated.txt"), b"keep").expect("marker");
    assert!(matches!(
        handoff::create(&report, &output, None, false, NOW + 5),
        Err(HandoffError::OutputNotEmpty(_))
    ));
    handoff::create(&report, &output, None, true, NOW + 5).expect("forced create");
    assert_eq!(
        std::fs::read_to_string(output.join("unrelated.txt")).expect("marker"),
        "keep"
    );

    let valid = example_manifest();
    let evidence = validate_manifest(&valid).expect("valid manifest");
    assert_eq!(evidence.model_id, "declarative:example-lighting");
    assert_eq!(evidence.match_rule_count, 1);
    assert!(
        evidence
            .validation_note
            .to_lowercase()
            .contains("structurally")
    );

    let text = std::fs::read_to_string(&valid).expect("manifest");
    let invalid = temp.path().join("invalid.toml");
    std::fs::write(
        &invalid,
        text.replace("manifest_version = 1", "manifest_version = 2"),
    )
    .expect("write");
    assert!(matches!(
        validate_manifest(&invalid),
        Err(HandoffError::InvalidManifest(_))
    ));
}
