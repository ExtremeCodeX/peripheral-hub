//! Manifest validation, encoding and interpreter behavior without hardware.

use peripheral_hub_device_sdk::*;

use crate::*;

const BASE: &str = r#"
manifest_version = 1

[model]
id = "declarative:test"
display_name = "Fictitious test model"
sdk_major = 1
sdk_minor = 0

[[match]]
vendor_id = 0xF0F0
product_id = 0x0001
transport = "hid"
interface_number = 3
usage = { page = 0xFF00, usage = 1 }

[capabilities.lighting]
zones = ["whole-device"]
brightness = true
effects = ["static"]

[[operations]]
kind = "set_lighting"
zone = "whole-device"

[operations.report]
kind = "output"
report_id = 0
length = 5
timeout_ms = 100
max_rate_per_second = 20

[[operations.report.fields]]
offset = 0
fill = 0x11

[[operations.report.fields]]
offset = 1
source = "red"

[[operations.report.fields]]
offset = 2
source = "green"

[[operations.report.fields]]
offset = 3
source = "blue"

[operations.report.effect]
offset = 4
values = [{ effect = "static", value = 1 }]
"#;

fn command() -> LightingCommand {
    LightingCommand {
        zone: "whole-device".into(),
        color: RgbColor {
            red: 10,
            green: 20,
            blue: 30,
        },
        brightness: None,
        effect: LightingEffect::Static,
    }
}

#[test]
fn valid_manifest_parses_and_builds_factory_metadata() {
    let manifest = parse_manifest(BASE).unwrap();
    assert_eq!(manifest.manifest_version, MANIFEST_VERSION);
    let factory = DeclarativeFactory::new(manifest, ExecutionMode::Simulated).unwrap();
    let metadata = factory.metadata();
    assert_eq!(metadata.model_id.as_str(), "declarative:test");
    assert_eq!(metadata.sdk_version, SDK_CONTRACT_VERSION);
    assert_eq!(metadata.match_rules.len(), 1);
}

#[test]
fn encoding_is_deterministic_and_bounded() {
    let manifest = parse_manifest(BASE).unwrap();
    let report = manifest.operations[0].report.encode(&command()).unwrap();
    assert_eq!(report.report_id, 0);
    assert_eq!(report.payload, vec![0x11, 10, 20, 30, 1]);
}

#[test]
fn unsupported_versions_and_weak_matches_are_rejected() {
    assert!(matches!(
        parse_manifest(&BASE.replace("manifest_version = 1", "manifest_version = 2")),
        Err(ManifestError::UnsupportedVersion {
            found: 2,
            supported: 1
        })
    ));
    let weak = BASE.replace(
        "interface_number = 3\nusage = { page = 0xFF00, usage = 1 }",
        "",
    );
    assert!(matches!(
        parse_manifest(&weak),
        Err(ManifestError::Device(_))
    ));
    let wrong_sdk = BASE.replace("sdk_minor = 0", "sdk_minor = 9");
    assert!(matches!(
        parse_manifest(&wrong_sdk),
        Err(ManifestError::Device(DeviceError::IncompatibleVersion))
    ));
}

#[test]
fn unknown_fields_and_operations_are_rejected() {
    assert!(matches!(
        parse_manifest(&format!("{BASE}\nunexpected = 1\n")),
        Err(ManifestError::Toml(_))
    ));
    assert!(matches!(
        parse_manifest(&BASE.replace("kind = \"set_lighting\"", "kind = \"write_flash\"")),
        Err(ManifestError::Toml(_))
    ));
}

#[test]
fn report_layout_violations_are_rejected() {
    // Input reports cannot be written.
    assert!(matches!(
        parse_manifest(&BASE.replace("kind = \"output\"", "kind = \"input\"")),
        Err(ManifestError::Invalid(_))
    ));
    // Zero/oversized report length.
    assert!(matches!(
        parse_manifest(&BASE.replace("length = 5", "length = 0")),
        Err(ManifestError::Invalid(_))
    ));
    assert!(matches!(
        parse_manifest(&BASE.replace("length = 5", "length = 65")),
        Err(ManifestError::Invalid(_))
    ));
    // Field offset beyond the report.
    assert!(matches!(
        parse_manifest(&BASE.replace(
            "offset = 3\nsource = \"blue\"",
            "offset = 9\nsource = \"blue\""
        )),
        Err(ManifestError::Invalid(_))
    ));
    // The effect byte cannot overlap a field byte.
    assert!(matches!(
        parse_manifest(&BASE.replace("offset = 4\nvalues", "offset = 3\nvalues")),
        Err(ManifestError::Invalid(_))
    ));
}

#[test]
fn brightness_and_effect_declarations_are_checked() {
    let with_brightness = BASE
        .replace("length = 5", "length = 6")
        .replace("offset = 4\nvalues", "offset = 5\nvalues")
        .replace(
            "[operations.report.effect]",
            "[[operations.report.fields]]\noffset = 4\nsource = \"brightness\"\n\n[operations.report.effect]",
        );
    assert!(parse_manifest(&with_brightness).is_ok());
    // Same encoding with brightness support disabled is rejected.
    let no_brightness = with_brightness.replace("brightness = true", "brightness = false");
    assert!(matches!(
        parse_manifest(&no_brightness),
        Err(ManifestError::Invalid(_))
    ));
    // A non-static effect whose encoding does not map it is rejected.
    let breathing = BASE.replace(
        "effects = [\"static\"]",
        "effects = [\"static\", \"breathing\"]",
    );
    assert!(matches!(
        parse_manifest(&breathing),
        Err(ManifestError::Invalid(_))
    ));
}

#[test]
fn duplicate_or_undeclared_zones_are_rejected() {
    let undeclared = BASE.replace("zone = \"whole-device\"", "zone = \"side-panel\"");
    assert!(matches!(
        parse_manifest(&undeclared),
        Err(ManifestError::Invalid(_))
    ));
    let ops_start = BASE.find("[[operations]]").unwrap();
    let duplicated = format!("{BASE}{}", &BASE[ops_start..]);
    assert!(matches!(
        parse_manifest(&duplicated),
        Err(ManifestError::Invalid(_))
    ));
}
