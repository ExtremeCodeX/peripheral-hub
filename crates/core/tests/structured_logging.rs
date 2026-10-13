//! Isolated subscriber test so parallel unit tests cannot alter callsite interest.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use peripheral_hub_core::PeripheralCore;
use peripheral_hub_device_registry::DriverRegistry;
use peripheral_hub_device_sdk::*;

#[derive(Clone)]
struct Buffer(Arc<Mutex<Vec<u8>>>);
impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn structured_logs_expose_counts_and_codes_without_private_descriptors() {
    let output = Arc::new(Mutex::new(Vec::new()));
    let writer = Buffer(Arc::clone(&output));
    let subscriber = tracing_subscriber::fmt()
        .json()
        .without_time()
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        let mut core = PeripheralCore::new(DriverRegistry::new());
        // Synthetic contract data, never enumerated or physically operated.
        let descriptor = DeviceDescriptor {
            instance_id: DeviceInstanceId::new("test:unit").unwrap(),
            interface_id: DeviceInterfaceId::new("test:end").unwrap(),
            vendor_id: 0xffff,
            product_id: 0xfffe,
            transport: TransportKind::Hid,
            path: b"PRIVATE-PATH".to_vec(),
            interface_number: Some(2),
            usage: None,
            manufacturer: None,
            product: None,
            serial_number: Some("PRIVATE-SERIAL".into()),
            firmware_version: None,
        };
        core.replace_inventory(vec![descriptor.clone()]).unwrap();
        let mut invalid = descriptor;
        invalid.path.clear();
        assert!(core.replace_inventory(vec![invalid]).is_err());
    });
    let text = String::from_utf8(output.lock().unwrap().clone()).unwrap();
    for private in ["PRIVATE-PATH", "PRIVATE-SERIAL", "test:unit", "test:end"] {
        assert!(!text.contains(private));
    }
    let events: Vec<serde_json::Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let replaced = events
        .iter()
        .find(|e| e["fields"]["event"] == "inventory_replaced")
        .expect("structured inventory event");
    assert_eq!(replaced["fields"]["unknown"], 1);
    assert_eq!(replaced["fields"]["interfaces"], 1);
    let rejected = events
        .iter()
        .find(|e| e["fields"]["event"] == "inventory_rejected")
        .expect("structured rejection event");
    assert_eq!(rejected["fields"]["error_code"], "invalid_argument");
}
