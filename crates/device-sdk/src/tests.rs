use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use super::*;

// All IDs/locators below are synthetic contract-test data, not device support.
fn descriptor() -> DeviceDescriptor {
    DeviceDescriptor {
        instance_id: DeviceInstanceId::new("test-unit-1").unwrap(),
        interface_id: DeviceInterfaceId::new("test-interface-1").unwrap(),
        vendor_id: 0xffff,
        product_id: 0xfffe,
        transport: TransportKind::Hid,
        path: vec![0xff, 0x01],
        interface_number: Some(2),
        usage: Some(HidUsage {
            page: 0xff00,
            usage: 1,
        }),
        manufacturer: Some("Synthetic fixture".into()),
        product: Some("Test only".into()),
        serial_number: Some("PRIVATE-SERIAL".into()),
        firmware_version: Some("test-v1".into()),
    }
}

fn rule() -> DeviceMatchRule {
    let descriptor = descriptor();
    DeviceMatchRule {
        vendor_id: descriptor.vendor_id,
        product_id: descriptor.product_id,
        transport: descriptor.transport,
        interface_number: descriptor.interface_number,
        usage: descriptor.usage,
        product: None,
        firmware_version: None,
    }
}

fn capabilities() -> Capabilities {
    Capabilities {
        lighting: Some(LightingCapabilities {
            zones: vec!["whole-device".into()],
            brightness: true,
            effects: vec![LightingEffect::Static],
        }),
        ..Capabilities::default()
    }
}

fn command() -> DeviceCommand {
    DeviceCommand::SetLighting(LightingCommand {
        zone: "whole-device".into(),
        color: RgbColor {
            red: 255,
            green: 0,
            blue: 0,
        },
        brightness: Some(50),
        effect: LightingEffect::Static,
    })
}

#[test]
fn identifiers_reject_empty_paths_unicode_and_excessive_lengths_even_via_serde() {
    for value in ["", "a/b", "with space", "á", "a\n"] {
        assert!(DeviceId::new(value).is_err());
        let json = serde_json::to_string(value).unwrap();
        assert!(serde_json::from_str::<DeviceId>(&json).is_err());
        assert!(serde_json::from_str::<DeviceInstanceId>(&json).is_err());
        assert!(serde_json::from_str::<DeviceInterfaceId>(&json).is_err());
    }
    assert!(DeviceId::new("a".repeat(129)).is_err());
    assert!(DeviceId::new("a".repeat(128)).is_ok());
    assert_eq!(
        serde_json::from_str::<DeviceId>("\"test:model-1\"")
            .unwrap()
            .as_str(),
        "test:model-1"
    );
}

#[test]
fn physical_instances_and_interfaces_are_distinct_from_model_identity() {
    let first = descriptor();
    let mut second = first.clone();
    second.instance_id = DeviceInstanceId::new("test-unit-2").unwrap();
    assert_ne!(first.instance_id, second.instance_id);
    second.instance_id = first.instance_id.clone();
    second.interface_id = DeviceInterfaceId::new("test-interface-2").unwrap();
    assert_eq!(first.instance_id, second.instance_id);
    assert_ne!(first.interface_id, second.interface_id);
}

#[test]
fn descriptor_roundtrips_non_utf8_locator_and_debug_redacts_private_data() {
    let original = descriptor();
    original.validate().unwrap();
    let restored: DeviceDescriptor =
        serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap();
    assert_eq!(original, restored);
    let debug = format!("{original:?}");
    assert!(!debug.contains("PRIVATE-SERIAL"));
    assert!(!debug.contains("test-unit-1"));
    assert!(!debug.contains("path"));
}

#[test]
fn descriptor_rejects_invalid_lengths_and_transport_usage_combinations() {
    let mut value = descriptor();
    value.path.clear();
    assert!(value.validate().is_err());
    value.path = vec![1; 4097];
    assert!(value.validate().is_err());
    value = descriptor();
    value.firmware_version = Some("bad\nversion".into());
    assert!(value.validate().is_err());
    value = descriptor();
    value.transport = TransportKind::Usb;
    assert!(value.validate().is_err());
}

#[test]
fn serde_rejects_out_of_range_vid_and_unknown_descriptor_fields() {
    let mut json = serde_json::to_value(descriptor()).unwrap();
    json["vendor_id"] = 65536.into();
    assert!(serde_json::from_value::<DeviceDescriptor>(json).is_err());
    let mut json = serde_json::to_value(descriptor()).unwrap();
    json["unknown_field"] = true.into();
    assert!(serde_json::from_value::<DeviceDescriptor>(json).is_err());
}

#[test]
fn version_envelope_refuses_future_or_different_contracts() {
    let current = Versioned::new(descriptor());
    let json = serde_json::to_string(&current).unwrap();
    let decoded: Versioned<DeviceDescriptor> = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.into_supported().unwrap(), descriptor());
    for version in [
        ContractVersion { major: 0, minor: 0 },
        ContractVersion { major: 2, minor: 0 },
        ContractVersion { major: 1, minor: 1 },
    ] {
        assert_eq!(
            version.ensure_supported(),
            Err(DeviceError::IncompatibleVersion)
        );
    }
}

#[test]
fn matching_checks_each_constraint_and_never_accepts_unknown_required_firmware() {
    let mut match_rule = rule();
    match_rule.firmware_version = Some("test-v1".into());
    match_rule.product = Some("Test only".into());
    assert!(match_rule.matches(&descriptor()).unwrap());
    let mut variants = Vec::new();
    let mut value = descriptor();
    value.vendor_id = 1;
    variants.push(value);
    let mut value = descriptor();
    value.product_id = 1;
    variants.push(value);
    let mut value = descriptor();
    value.interface_number = None;
    variants.push(value);
    let mut value = descriptor();
    value.usage = None;
    variants.push(value);
    let mut value = descriptor();
    value.product = None;
    variants.push(value);
    let mut value = descriptor();
    value.firmware_version = None;
    variants.push(value);
    let mut value = descriptor();
    value.firmware_version = Some("test-v2".into());
    variants.push(value);
    for value in variants {
        assert!(!match_rule.matches(&value).unwrap());
    }
}

#[test]
fn vid_pid_only_rule_and_non_hid_usage_are_rejected() {
    let mut value = rule();
    value.interface_number = None;
    value.usage = None;
    assert!(value.validate().is_err());
    let mut value = rule();
    value.transport = TransportKind::Usb;
    assert!(value.validate().is_err());
}

#[test]
fn lighting_is_capability_gated_with_explicit_unsupported_errors() {
    assert_eq!(
        Capabilities::default().validate_command(&command()),
        Err(DeviceError::NotSupported)
    );
    capabilities().validate_command(&command()).unwrap();
    let mut caps = capabilities();
    caps.lighting.as_mut().unwrap().brightness = false;
    assert_eq!(
        caps.validate_command(&command()),
        Err(DeviceError::NotSupported)
    );
    let mut request = command();
    let DeviceCommand::SetLighting(light) = &mut request;
    light.brightness = None;
    light.effect = LightingEffect::Breathing;
    assert_eq!(
        caps.validate_command(&request),
        Err(DeviceError::NotSupported)
    );
}

#[test]
fn lighting_parameter_boundaries_and_zones_are_enforced() {
    let caps = capabilities();
    for brightness in [0, 100] {
        let mut request = command();
        let DeviceCommand::SetLighting(light) = &mut request;
        light.brightness = Some(brightness);
        caps.validate_command(&request).unwrap();
    }
    let mut request = command();
    let DeviceCommand::SetLighting(light) = &mut request;
    light.brightness = Some(101);
    assert!(matches!(
        caps.validate_command(&request),
        Err(DeviceError::InvalidArgument(_))
    ));
    let DeviceCommand::SetLighting(light) = &mut request;
    light.brightness = None;
    light.zone = "undeclared".into();
    assert!(caps.validate_command(&request).is_err());
}

#[test]
fn capabilities_reject_duplicates_and_invalid_domain_limits() {
    let mut caps = capabilities();
    caps.lighting
        .as_mut()
        .unwrap()
        .zones
        .push("whole-device".into());
    assert!(caps.validate().is_err());
    caps = capabilities();
    caps.lighting
        .as_mut()
        .unwrap()
        .effects
        .push(LightingEffect::Static);
    assert!(caps.validate().is_err());
    caps = Capabilities::default();
    caps.buttons = Some(ButtonCapabilities {
        count: 0,
        remapping: false,
    });
    assert!(caps.validate().is_err());
    caps.buttons = None;
    caps.macros = Some(MacroCapabilities {
        max_actions: 1,
        max_delay_ms: 60001,
    });
    assert!(caps.validate().is_err());
    caps.macros = None;
    caps.profiles = Some(ProfileCapabilities { max_profiles: 0 });
    assert!(caps.validate().is_err());
}

#[test]
fn display_frame_limits_must_cover_each_declared_format() {
    let mut caps = Capabilities {
        display: Some(DisplayCapabilities {
            width: 16,
            height: 16,
            formats: vec![PixelFormat::Rgb888],
            max_frames_per_second: 30,
            max_frame_bytes: 16 * 16 * 3,
        }),
        ..Capabilities::default()
    };
    caps.validate().unwrap();
    caps.display.as_mut().unwrap().max_frame_bytes -= 1;
    assert!(caps.validate().is_err());
    caps.display.as_mut().unwrap().width = 0;
    assert!(caps.validate().is_err());
}

#[test]
fn typed_commands_and_capabilities_roundtrip_without_unmodeled_operations() {
    let json = serde_json::to_string(&command()).unwrap();
    assert_eq!(
        serde_json::from_str::<DeviceCommand>(&json).unwrap(),
        command()
    );
    let json = serde_json::to_string(&capabilities()).unwrap();
    assert_eq!(
        serde_json::from_str::<Capabilities>(&json).unwrap(),
        capabilities()
    );
    assert!(
        serde_json::from_str::<DeviceCommand>(r#"{"operation":"firmware_update","parameters":{}}"#)
            .is_err()
    );
}

#[test]
fn errors_roundtrip_with_static_log_codes() {
    for error in [
        DeviceError::NotSupported,
        DeviceError::NotConnected,
        DeviceError::PermissionDenied,
        DeviceError::InvalidArgument("bound".into()),
        DeviceError::Timeout,
        DeviceError::TransportError("backend".into()),
        DeviceError::ProtocolError("frame".into()),
        DeviceError::IncompatibleVersion,
    ] {
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(serde_json::from_str::<DeviceError>(&json).unwrap(), error);
        assert!(!error.code().contains("backend"));
    }
    assert!(
        serde_json::from_str::<DeviceError>(r#"{"code":"timeout","unexpected":true}"#).is_err()
    );
}

// Test-only transport double: no OS device access and no peripheral protocol.
struct StubTransport {
    calls: Arc<AtomicUsize>,
    read: DeviceResult<HidReport>,
    write: DeviceResult<usize>,
}

impl StubTransport {
    fn new() -> Self {
        Self {
            calls: Arc::new(AtomicUsize::new(0)),
            read: Ok(HidReport {
                report_id: 0,
                payload: vec![1, 2],
            }),
            write: Ok(2),
        }
    }
}

impl Transport for StubTransport {
    fn limits(&self) -> TransportLimits {
        TransportLimits {
            input_bytes: 4,
            output_bytes: 4,
            feature_bytes: 0,
            max_timeout: Duration::from_millis(100),
        }
    }
    fn read_report(&mut self, _: ReportKind, _: u8, _: Duration) -> DeviceResult<HidReport> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.read.clone()
    }
    fn write_report(&mut self, _: ReportKind, _: &HidReport, _: Duration) -> DeviceResult<usize> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.write.clone()
    }
}

#[test]
fn checked_transport_blocks_invalid_requests_before_backend_calls() {
    let stub = StubTransport::new();
    let calls = Arc::clone(&stub.calls);
    let mut transport = CheckedTransport::new(stub).unwrap();
    let report = HidReport {
        report_id: 0,
        payload: vec![1, 2],
    };
    let timeout = Duration::from_millis(10);
    assert!(
        transport
            .write_report(ReportKind::Input, &report, timeout)
            .is_err()
    );
    assert!(
        transport
            .read_report(ReportKind::Output, 0, timeout)
            .is_err()
    );
    assert_eq!(
        transport.read_report(ReportKind::Feature, 0, timeout),
        Err(DeviceError::NotSupported)
    );
    assert!(
        transport
            .write_report(ReportKind::Output, &report, Duration::ZERO)
            .is_err()
    );
    assert!(
        transport
            .write_report(ReportKind::Output, &report, Duration::from_secs(1))
            .is_err()
    );
    assert!(
        transport
            .write_report(
                ReportKind::Output,
                &HidReport {
                    report_id: 0,
                    payload: vec![0; 5]
                },
                timeout
            )
            .is_err()
    );
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}

#[test]
fn checked_transport_preserves_errors_and_rejects_short_writes() {
    let report = HidReport {
        report_id: 0,
        payload: vec![1, 2],
    };
    for error in [
        DeviceError::Timeout,
        DeviceError::NotConnected,
        DeviceError::PermissionDenied,
        DeviceError::TransportError("failure".into()),
    ] {
        let mut stub = StubTransport::new();
        stub.write = Err(error.clone());
        stub.read = Err(error.clone());
        let mut checked = CheckedTransport::new(stub).unwrap();
        assert_eq!(
            checked.write_report(ReportKind::Output, &report, Duration::from_millis(10)),
            Err(error.clone())
        );
        assert_eq!(
            checked.read_report(ReportKind::Input, 0, Duration::from_millis(10)),
            Err(error)
        );
    }
    let mut stub = StubTransport::new();
    stub.write = Ok(1);
    assert!(matches!(
        CheckedTransport::new(stub).unwrap().write_report(
            ReportKind::Output,
            &report,
            Duration::from_millis(10)
        ),
        Err(DeviceError::TransportError(_))
    ));
}

#[test]
fn checked_transport_validates_received_id_and_length_and_keeps_zero_id_separate() {
    let mut checked = CheckedTransport::new(StubTransport::new()).unwrap();
    let report = checked
        .read_report(ReportKind::Input, 0, Duration::from_millis(10))
        .unwrap();
    assert_eq!(report.payload, vec![1, 2]);
    assert_eq!(
        checked
            .write_report(ReportKind::Output, &report, Duration::from_millis(10))
            .unwrap(),
        2
    );
    for report in [
        HidReport {
            report_id: 1,
            payload: vec![1],
        },
        HidReport {
            report_id: 0,
            payload: vec![1; 5],
        },
    ] {
        let mut stub = StubTransport::new();
        stub.read = Ok(report);
        assert!(matches!(
            CheckedTransport::new(stub).unwrap().read_report(
                ReportKind::Input,
                0,
                Duration::from_millis(10)
            ),
            Err(DeviceError::ProtocolError(_))
        ));
    }
}

#[test]
fn transport_limits_reject_unbounded_timeouts_and_payloads() {
    let mut limits = StubTransport::new().limits();
    limits.input_bytes = 65536;
    assert!(limits.validate().is_err());
    limits.input_bytes = 4;
    limits.max_timeout = Duration::ZERO;
    assert!(limits.validate().is_err());
    limits.max_timeout = Duration::from_secs(31);
    assert!(limits.validate().is_err());
    assert!(
        !format!(
            "{:?}",
            HidReport {
                report_id: 0,
                payload: vec![99; 5]
            }
        )
        .contains("99")
    );
}
