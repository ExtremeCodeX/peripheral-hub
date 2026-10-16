//! Daemon integration for declarative manifests over the real management path.
//!
//! No hardware is touched: a synthetic `DeviceSource` and the bounded in-memory
//! transport stand in for a device, proving identification, capability gating,
//! connection state and provenance without any physical claim.

use std::path::PathBuf;

use peripheral_hub_daemon::*;
use peripheral_hub_device_registry::DriverRegistry;
use peripheral_hub_device_sdk::*;
use peripheral_hub_ipc::*;
use peripheral_hub_transport_hid::{DeviceSource, simulated::SimulatedTransport};

fn example_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/declarative-driver")
}

fn descriptor() -> DeviceDescriptor {
    DeviceDescriptor {
        instance_id: DeviceInstanceId::new("declarative-from-mem-unit").unwrap(),
        interface_id: DeviceInterfaceId::new("declarative-from-mem-interface").unwrap(),
        vendor_id: 0xf0f0,
        product_id: 0x0001,
        transport: TransportKind::Hid,
        path: b"synthetic-declarative-locator".to_vec(),
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

struct ExampleSource {
    present: bool,
}

impl ExampleSource {
    fn new() -> Self {
        Self { present: true }
    }
}

impl DeviceSource for ExampleSource {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Simulated
    }
    fn scan(&mut self) -> DeviceResult<Vec<DeviceDescriptor>> {
        Ok(if self.present {
            vec![descriptor()]
        } else {
            vec![]
        })
    }
    fn open(&mut self, _: &DeviceDescriptor) -> DeviceResult<Box<dyn Transport>> {
        Ok(Box::new(CheckedTransport::new(
            SimulatedTransport::default(),
        )?))
    }
    fn set_simulated_presence(&mut self, present: bool) -> DeviceResult<()> {
        self.present = present;
        Ok(())
    }
}

fn lighting(zone: &str) -> LightingCommand {
    LightingCommand {
        zone: zone.into(),
        color: RgbColor {
            red: 4,
            green: 5,
            blue: 6,
        },
        brightness: Some(70),
        effect: LightingEffect::Static,
    }
}

#[test]
fn manifests_load_and_flow_through_the_manager() {
    let mut registry = DriverRegistry::new();
    assert_eq!(
        register_manifest_dir(&mut registry, &example_dir(), ExecutionMode::Simulated).unwrap(),
        1
    );
    let mut manager = DeviceManager::try_new(registry, Box::new(ExampleSource::new())).unwrap();
    manager.refresh().unwrap();

    let devices = match manager.handle(Operation::ListDevices).unwrap() {
        Reply::Devices(devices) => devices,
        _ => panic!("unexpected reply"),
    };
    assert_eq!(devices.len(), 1);
    assert_eq!(
        devices[0].model.as_ref().map(DeviceId::as_str),
        Some("declarative:example-lighting")
    );
    assert_eq!(devices[0].mode, ExecutionMode::Simulated);
    let interface_id = devices[0].interface_id.clone();

    manager
        .handle(Operation::Connect {
            interface_id: interface_id.clone(),
        })
        .unwrap();
    let capabilities = match manager
        .handle(Operation::GetCapabilities {
            interface_id: interface_id.clone(),
        })
        .unwrap()
    {
        Reply::Capabilities(capabilities) => capabilities,
        _ => panic!("unexpected reply"),
    };
    assert_eq!(
        capabilities.lighting.as_ref().unwrap().zones,
        vec!["whole-device".to_string()]
    );

    // A valid request reports the explicit simulation provenance.
    let outcome = match manager
        .handle(Operation::SetLighting {
            interface_id: interface_id.clone(),
            command: lighting("whole-device"),
        })
        .unwrap()
    {
        Reply::Outcome(outcome) => outcome,
        _ => panic!("unexpected reply"),
    };
    assert_eq!(outcome.mode, ExecutionMode::Simulated);

    // An undeclared zone is a structured device fault and must not tear down the
    // live session.
    let fault = manager
        .handle(Operation::SetLighting {
            interface_id: interface_id.clone(),
            command: lighting("side-panel"),
        })
        .unwrap_err();
    assert!(matches!(
        fault,
        Fault::Device(DeviceError::InvalidArgument(_))
    ));

    let summary = match manager
        .handle(Operation::GetDevice {
            interface_id: interface_id.clone(),
        })
        .unwrap()
    {
        Reply::Device(summary) => summary,
        _ => panic!("unexpected reply"),
    };
    assert_eq!(summary.status, ConnectionStatus::Connected);
    manager.shutdown().unwrap();
}

#[test]
fn invalid_manifest_aborts_loading() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("broken.toml"),
        "manifest_version = 1\n[model]\nid = \"declarative:bad\"\n",
    )
    .unwrap();
    let mut registry = DriverRegistry::new();
    let error =
        register_manifest_dir(&mut registry, dir.path(), ExecutionMode::Simulated).unwrap_err();
    assert_eq!(error.code(), "manifest_invalid");
    assert_eq!(registry.len(), 0);
}
