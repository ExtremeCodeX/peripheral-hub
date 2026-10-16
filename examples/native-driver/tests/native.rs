//! Software test for the fictitious native example; no hardware is touched.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use peripheral_hub_device_registry::DriverRegistry;
use peripheral_hub_device_sdk::*;
use peripheral_hub_example_native_driver::ExampleNativeFactory;

fn descriptor() -> DeviceDescriptor {
    DeviceDescriptor {
        instance_id: DeviceInstanceId::new("native-example-unit").unwrap(),
        interface_id: DeviceInterfaceId::new("native-example-interface").unwrap(),
        vendor_id: 0xf0f1,
        product_id: 0x0002,
        transport: TransportKind::Hid,
        path: b"synthetic-native-locator".to_vec(),
        interface_number: Some(4),
        usage: Some(HidUsage {
            page: 0xff01,
            usage: 2,
        }),
        manufacturer: Some("Fictitious example".into()),
        product: Some("Fictitious native keyboard".into()),
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
            feature_bytes: 0,
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
        _: Duration,
    ) -> DeviceResult<usize> {
        if kind == ReportKind::Input {
            return Err(DeviceError::NotSupported);
        }
        *self.last.lock().unwrap() = Some(report.clone());
        Ok(report.payload.len())
    }
}

#[test]
fn native_example_matches_and_writes_its_bounded_report() {
    let mut registry = DriverRegistry::new();
    registry
        .register(Box::new(ExampleNativeFactory::new().unwrap()))
        .unwrap();
    assert_eq!(
        registry
            .resolve(&descriptor())
            .unwrap()
            .unwrap()
            .model_id
            .as_str(),
        "example-native:lighting"
    );

    let mut driver = registry.create_driver(&descriptor()).unwrap();
    let last = Arc::new(Mutex::new(None));
    driver
        .open(Box::new(RecordingTransport {
            last: Arc::clone(&last),
        }))
        .unwrap();
    let outcome = driver
        .execute(&DeviceCommand::SetLighting(LightingCommand {
            zone: "whole-device".into(),
            color: RgbColor {
                red: 1,
                green: 2,
                blue: 3,
            },
            brightness: None,
            effect: LightingEffect::Static,
        }))
        .unwrap();
    assert_eq!(outcome.mode, ExecutionMode::Physical);
    assert_eq!(
        last.lock().unwrap().clone().unwrap().payload,
        vec![0x22, 1, 2, 3]
    );
}
