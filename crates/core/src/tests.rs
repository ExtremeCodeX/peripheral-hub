use peripheral_hub_device_registry::{DriverRegistry, RegistryError};
use peripheral_hub_device_sdk::*;

use super::*;

// All descriptors below are synthetic. No devices are enumerated or accessed.
fn descriptor(unit: &str, endpoint: &str, interface: u8) -> DeviceDescriptor {
    DeviceDescriptor {
        instance_id: DeviceInstanceId::new(unit).unwrap(),
        interface_id: DeviceInterfaceId::new(endpoint).unwrap(),
        vendor_id: 0xffff,
        product_id: 0xfffe,
        transport: TransportKind::Hid,
        path: format!("PRIVATE-PATH-{endpoint}").into_bytes(),
        interface_number: Some(interface),
        usage: Some(HidUsage {
            page: 0xff00,
            usage: 1,
        }),
        manufacturer: None,
        product: None,
        serial_number: Some("PRIVATE-SERIAL".into()),
        firmware_version: None,
    }
}

struct Factory(DriverMetadata);
impl DriverFactory for Factory {
    fn metadata(&self) -> &DriverMetadata {
        &self.0
    }
    fn create(&self, _: &DeviceDescriptor) -> DeviceResult<Box<dyn DeviceDriver>> {
        Err(DeviceError::NotSupported)
    }
}

fn register(registry: &mut DriverRegistry, name: &str, interface: u8) {
    registry
        .register(Box::new(Factory(DriverMetadata {
            model_id: DeviceId::new(name).unwrap(),
            display_name: "Synthetic test factory".into(),
            sdk_version: SDK_CONTRACT_VERSION,
            match_rules: vec![DeviceMatchRule {
                vendor_id: 0xffff,
                product_id: 0xfffe,
                transport: TransportKind::Hid,
                interface_number: Some(interface),
                usage: Some(HidUsage {
                    page: 0xff00,
                    usage: 1,
                }),
                product: None,
                firmware_version: None,
            }],
        })))
        .unwrap();
}

fn core() -> PeripheralCore {
    let mut registry = DriverRegistry::new();
    register(&mut registry, "test:model", 2);
    PeripheralCore::new(registry)
}

#[test]
fn inventory_preserves_multiple_physical_units_of_the_same_model() {
    let mut core = core();
    let change = core
        .replace_inventory(vec![
            descriptor("test:unit1", "test:end1", 2),
            descriptor("test:unit2", "test:end2", 2),
        ])
        .unwrap();
    assert_eq!(
        change,
        InventoryChange {
            added: 2,
            removed: 0,
            updated: 0
        }
    );
    assert_eq!(core.devices().count(), 2);
    for device in core.devices() {
        assert_eq!(
            device.identification,
            Identification::Compatible(DeviceId::new("test:model").unwrap())
        );
    }
}

#[test]
fn composite_interfaces_share_a_physical_unit_without_collapsing_endpoints() {
    let mut core = core();
    core.replace_inventory(vec![
        descriptor("test:unit", "test:end1", 2),
        descriptor("test:unit", "test:end2", 3),
    ])
    .unwrap();
    assert_eq!(
        core.interfaces_for(&DeviceInstanceId::new("test:unit").unwrap())
            .count(),
        2
    );
    assert_eq!(
        core.get(&DeviceInterfaceId::new("test:end2").unwrap())
            .unwrap()
            .identification,
        Identification::Unknown
    );
}

#[test]
fn snapshots_report_changes_without_claiming_real_hotplug_detection() {
    let mut core = core();
    let initial = descriptor("test:unit", "test:end", 2);
    core.replace_inventory(vec![initial.clone()]).unwrap();
    assert_eq!(
        core.replace_inventory(vec![initial.clone()]).unwrap(),
        InventoryChange::default()
    );
    let mut updated = initial;
    updated.firmware_version = Some("test-v2".into());
    assert_eq!(core.replace_inventory(vec![updated]).unwrap().updated, 1);
    assert_eq!(core.replace_inventory(vec![]).unwrap().removed, 1);
    assert_eq!(core.devices().count(), 0);
}

#[test]
fn invalid_descriptor_rejects_snapshot_atomically() {
    let mut core = core();
    let original = descriptor("test:unit", "test:end", 2);
    core.replace_inventory(vec![original.clone()]).unwrap();
    let mut invalid = descriptor("test:other", "test:other-end", 2);
    invalid.path.clear();
    assert!(matches!(
        core.replace_inventory(vec![invalid]),
        Err(CoreError::Device(DeviceError::InvalidArgument(_)))
    ));
    assert_eq!(core.devices().count(), 1);
    assert_eq!(core.devices().next().unwrap().descriptor, original);
}

#[test]
fn duplicate_interface_tokens_or_locators_are_rejected_atomically() {
    let mut core = core();
    let original = descriptor("test:unit", "test:end", 2);
    core.replace_inventory(vec![original.clone()]).unwrap();
    let mut same_token = original.clone();
    same_token.path = b"other-path".to_vec();
    assert_eq!(
        core.replace_inventory(vec![original.clone(), same_token]),
        Err(CoreError::DuplicateInterface)
    );
    let mut same_path = original.clone();
    same_path.interface_id = DeviceInterfaceId::new("test:other-end").unwrap();
    assert_eq!(
        core.replace_inventory(vec![original.clone(), same_path]),
        Err(CoreError::DuplicateInterface)
    );
    assert_eq!(core.devices().next().unwrap().descriptor, original);
}

#[test]
fn conflicting_vid_pid_for_one_unit_is_not_silently_merged() {
    let mut core = core();
    let original = descriptor("test:unit", "test:end1", 2);
    let mut conflict = descriptor("test:unit", "test:end2", 3);
    conflict.product_id = 0;
    assert_eq!(
        core.replace_inventory(vec![original, conflict]),
        Err(CoreError::InconsistentInstance)
    );
    assert_eq!(core.devices().count(), 0);
}

#[test]
fn registry_ambiguity_is_preserved_and_prior_inventory_is_retained() {
    let mut registry = DriverRegistry::new();
    register(&mut registry, "test:a", 2);
    register(&mut registry, "test:b", 2);
    let mut core = PeripheralCore::new(registry);
    let unknown = descriptor("test:unit", "test:end", 3);
    core.replace_inventory(vec![unknown.clone()]).unwrap();
    assert!(matches!(
        core.replace_inventory(vec![descriptor("test:other", "test:other-end", 2)]),
        Err(CoreError::Registry(RegistryError::AmbiguousMatch(_)))
    ));
    assert_eq!(core.devices().next().unwrap().descriptor, unknown);
}

#[test]
fn one_unit_can_expose_hid_and_non_hid_interfaces() {
    let mut core = core();
    let hid = descriptor("test:unit", "test:hid", 2);
    let mut usb = descriptor("test:unit", "test:usb", 3);
    usb.transport = TransportKind::Usb;
    usb.usage = None;
    core.replace_inventory(vec![hid, usb]).unwrap();
    assert_eq!(
        core.interfaces_for(&DeviceInstanceId::new("test:unit").unwrap())
            .count(),
        2
    );
}
