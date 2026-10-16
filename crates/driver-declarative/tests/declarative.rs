//! End-to-end declarative example: manifest -> registry -> core -> transport.
//!
//! Software only: a recording in-memory transport stands in for hardware. This
//! proves identification, capability gating and bounded encoding, not physical
//! device support.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use peripheral_hub_core::{Identification, PeripheralCore};
use peripheral_hub_device_registry::DriverRegistry;
use peripheral_hub_device_sdk::*;
use peripheral_hub_driver_declarative::{DeclarativeFactory, load_manifest_file};

fn example_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/declarative-driver/manifest.toml")
}

fn descriptor() -> DeviceDescriptor {
    DeviceDescriptor {
        instance_id: DeviceInstanceId::new("example-unit").unwrap(),
        interface_id: DeviceInterfaceId::new("example-interface").unwrap(),
        vendor_id: 0xf0f0,
        product_id: 0x0001,
        transport: TransportKind::Hid,
        path: b"synthetic-example-locator".to_vec(),
        interface_number: Some(3),
        usage: Some(HidUsage {
            page: 0xff00,
            usage: 1,
        }),
        manufacturer: Some("Fictitious example".into()),
        product: Some("Fictitious example keyboard".into()),
        serial_number: None,
        firmware_version: None,
    }
}

struct RecordingTransport {
    last: Arc<Mutex<Option<HidReport>>>,
}

impl Transport for RecordingTransport {
    fn limits(&self) -> TransportLimits {
        TransportLimits {
            input_bytes: 0,
            output_bytes: 64,
            feature_bytes: 64,
            max_timeout: Duration::from_secs(1),
        }
    }
    fn read_report(&mut self, _: ReportKind, _: u8, _: Duration) -> DeviceResult<HidReport> {
        Err(DeviceError::NotSupported)
    }
    fn write_report(
        &mut self,
        kind: ReportKind,
        report: &HidReport,
        timeout: Duration,
    ) -> DeviceResult<usize> {
        if kind == ReportKind::Input || timeout.is_zero() {
            return Err(DeviceError::NotSupported);
        }
        *self.last.lock().unwrap() = Some(report.clone());
        Ok(report.payload.len())
    }
}

fn registry() -> DriverRegistry {
    let manifest = load_manifest_file(&example_manifest_path()).unwrap();
    let factory = DeclarativeFactory::new(manifest, ExecutionMode::Simulated).unwrap();
    let mut registry = DriverRegistry::new();
    registry.register(Box::new(factory)).unwrap();
    registry
}

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

#[test]
fn shipped_example_identifies_and_executes_a_bounded_report() {
    let mut core = PeripheralCore::new(registry());
    core.replace_inventory(vec![descriptor()]).unwrap();
    assert_eq!(
        core.get(&descriptor().interface_id).unwrap().identification,
        Identification::Compatible(DeviceId::new("declarative:example-lighting").unwrap())
    );

    let mut driver = core.registry().create_driver(&descriptor()).unwrap();
    let last = Arc::new(Mutex::new(None));
    driver
        .open(Box::new(RecordingTransport {
            last: Arc::clone(&last),
        }))
        .unwrap();
    assert_eq!(
        driver
            .capabilities()
            .unwrap()
            .lighting
            .as_ref()
            .unwrap()
            .zones
            .len(),
        1
    );

    let outcome = driver
        .execute(&DeviceCommand::SetLighting(lighting(Some(40))))
        .unwrap();
    assert_eq!(outcome.mode, ExecutionMode::Simulated);
    let report = last.lock().unwrap().clone().unwrap();
    assert_eq!(report.report_id, 0);
    assert_eq!(report.payload, vec![0x11, 10, 20, 30, 40, 1]);
}

#[test]
fn capabilities_gate_commands_and_rate_limits_are_enforced() {
    let mut driver = registry().create_driver(&descriptor()).unwrap();
    driver
        .open(Box::new(RecordingTransport {
            last: Arc::new(Mutex::new(None)),
        }))
        .unwrap();

    // An undeclared zone is rejected by capability gating before any I/O.
    let mut wrong_zone = lighting(None);
    wrong_zone.zone = "side-panel".into();
    assert!(matches!(
        driver.execute(&DeviceCommand::SetLighting(wrong_zone)),
        Err(DeviceError::InvalidArgument(_))
    ));

    // A valid command succeeds once, then the declared rate limit rejects an
    // immediate second write instead of silently dropping or replaying it.
    driver
        .execute(&DeviceCommand::SetLighting(lighting(Some(50))))
        .unwrap();
    assert!(matches!(
        driver.execute(&DeviceCommand::SetLighting(lighting(Some(50)))),
        Err(DeviceError::ProtocolError(_))
    ));
}

#[test]
fn non_matching_descriptors_are_unknown() {
    let mut core = PeripheralCore::new(registry());
    let mut other = descriptor();
    other.interface_number = Some(9);
    core.replace_inventory(vec![other.clone()]).unwrap();
    assert_eq!(
        core.get(&other.interface_id).unwrap().identification,
        Identification::Unknown
    );
}
