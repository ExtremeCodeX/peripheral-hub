use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use peripheral_hub_device_sdk::*;

use super::*;

// Synthetic IDs and factories exist only in tests, never a device catalog.
fn descriptor() -> DeviceDescriptor {
    DeviceDescriptor {
        instance_id: DeviceInstanceId::new("test-unit").unwrap(),
        interface_id: DeviceInterfaceId::new("test-endpoint").unwrap(),
        vendor_id: 0xffff,
        product_id: 0xfffe,
        transport: TransportKind::Hid,
        path: b"test-only-locator".to_vec(),
        interface_number: Some(2),
        usage: Some(HidUsage {
            page: 0xff00,
            usage: 1,
        }),
        manufacturer: None,
        product: None,
        serial_number: None,
        firmware_version: None,
    }
}

struct Factory {
    metadata: DriverMetadata,
    error: Option<DeviceError>,
    calls: Arc<AtomicUsize>,
}

fn factory(name: &str) -> Factory {
    Factory {
        metadata: DriverMetadata {
            model_id: DeviceId::new(name).unwrap(),
            display_name: "Synthetic fixture".into(),
            sdk_version: SDK_CONTRACT_VERSION,
            match_rules: vec![DeviceMatchRule {
                vendor_id: 0xffff,
                product_id: 0xfffe,
                transport: TransportKind::Hid,
                interface_number: Some(2),
                usage: Some(HidUsage {
                    page: 0xff00,
                    usage: 1,
                }),
                product: None,
                firmware_version: None,
            }],
        },
        error: None,
        calls: Arc::new(AtomicUsize::new(0)),
    }
}

// No connection or success simulation: unsupported default commands are tested.
struct UnsupportedDriver;
impl DeviceDriver for UnsupportedDriver {
    fn open(&mut self, _: Box<dyn Transport>) -> DeviceResult<()> {
        Err(DeviceError::NotSupported)
    }
    fn close(&mut self) -> DeviceResult<()> {
        Err(DeviceError::NotConnected)
    }
    fn capabilities(&self) -> DeviceResult<Capabilities> {
        Ok(Capabilities::default())
    }
}

impl DriverFactory for Factory {
    fn metadata(&self) -> &DriverMetadata {
        &self.metadata
    }
    fn create(&self, _: &DeviceDescriptor) -> DeviceResult<Box<dyn DeviceDriver>> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if let Some(error) = &self.error {
            Err(error.clone())
        } else {
            Ok(Box::new(UnsupportedDriver))
        }
    }
}

#[test]
fn registration_rejects_duplicates_without_replacing_original_factory() {
    let original = factory("test:model");
    let calls = Arc::clone(&original.calls);
    let mut registry = DriverRegistry::new();
    assert!(registry.is_empty());
    registry.register(Box::new(original)).unwrap();
    assert!(matches!(
        registry.register(Box::new(factory("test:model"))),
        Err(RegistryError::DuplicateModel(_))
    ));
    assert_eq!(registry.len(), 1);
    registry.create_driver(&descriptor()).unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[test]
fn registration_rejects_weak_rules_and_incompatible_sdk_before_mutation() {
    let mut registry = DriverRegistry::new();
    let mut weak = factory("test:weak");
    weak.metadata.match_rules[0].interface_number = None;
    weak.metadata.match_rules[0].usage = None;
    assert!(registry.register(Box::new(weak)).is_err());
    let mut future = factory("test:future");
    future.metadata.sdk_version.minor = 1;
    assert_eq!(
        registry.register(Box::new(future)),
        Err(RegistryError::Device(DeviceError::IncompatibleVersion))
    );
    let mut empty = factory("test:empty");
    empty.metadata.match_rules.clear();
    assert!(registry.register(Box::new(empty)).is_err());
    assert!(registry.is_empty());
}

#[test]
fn wrong_interface_or_usage_is_unknown_despite_equal_vid_pid() {
    let mut registry = DriverRegistry::new();
    registry.register(Box::new(factory("test:model"))).unwrap();
    assert!(registry.resolve(&descriptor()).unwrap().is_some());
    let mut wrong = descriptor();
    wrong.interface_number = Some(3);
    assert!(registry.resolve(&wrong).unwrap().is_none());
    wrong = descriptor();
    wrong.usage = Some(HidUsage {
        page: 0xff00,
        usage: 2,
    });
    assert!(registry.resolve(&wrong).unwrap().is_none());
    assert!(matches!(
        registry.create_driver(&wrong),
        Err(RegistryError::NoMatch)
    ));
}

#[test]
fn duplicate_matching_rules_do_not_make_one_driver_ambiguous() {
    let mut candidate = factory("test:model");
    candidate
        .metadata
        .match_rules
        .push(candidate.metadata.match_rules[0].clone());
    let mut registry = DriverRegistry::new();
    registry.register(Box::new(candidate)).unwrap();
    assert_eq!(
        registry
            .resolve(&descriptor())
            .unwrap()
            .unwrap()
            .model_id
            .as_str(),
        "test:model"
    );
}

#[test]
fn ambiguity_is_order_independent_and_never_constructs_a_driver() {
    for order in [["test:a", "test:b"], ["test:b", "test:a"]] {
        let mut registry = DriverRegistry::new();
        let mut counters = Vec::new();
        for name in order {
            let candidate = factory(name);
            counters.push(Arc::clone(&candidate.calls));
            registry.register(Box::new(candidate)).unwrap();
        }
        let expected = RegistryError::AmbiguousMatch(vec![
            DeviceId::new("test:a").unwrap(),
            DeviceId::new("test:b").unwrap(),
        ]);
        assert_eq!(registry.resolve(&descriptor()), Err(expected.clone()));
        assert!(matches!(registry.create_driver(&descriptor()), Err(error) if error == expected));
        for calls in counters {
            assert_eq!(calls.load(Ordering::Relaxed), 0);
        }
    }
}

#[test]
fn factory_errors_are_preserved_without_success_fallback() {
    for error in [
        DeviceError::PermissionDenied,
        DeviceError::NotConnected,
        DeviceError::Timeout,
        DeviceError::TransportError("test failure".into()),
    ] {
        let mut candidate = factory("test:model");
        candidate.error = Some(error.clone());
        let mut registry = DriverRegistry::new();
        registry.register(Box::new(candidate)).unwrap();
        assert!(
            matches!(registry.create_driver(&descriptor()), Err(RegistryError::Device(actual)) if actual == error)
        );
    }
}

#[test]
fn resolution_validates_input_without_constructing_or_opening_a_driver() {
    let candidate = factory("test:model");
    let calls = Arc::clone(&candidate.calls);
    let mut registry = DriverRegistry::new();
    registry.register(Box::new(candidate)).unwrap();
    registry.resolve(&descriptor()).unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    let mut invalid = descriptor();
    invalid.path.clear();
    assert!(matches!(
        registry.resolve(&invalid),
        Err(RegistryError::Device(DeviceError::InvalidArgument(_)))
    ));
}

#[test]
fn default_driver_state_and_commands_are_explicitly_unsupported() {
    let mut registry = DriverRegistry::new();
    registry.register(Box::new(factory("test:model"))).unwrap();
    let mut driver = registry.create_driver(&descriptor()).unwrap();
    assert_eq!(driver.read_state(), Err(DeviceError::NotSupported));
    assert_eq!(
        driver.execute(&DeviceCommand::SetLighting(LightingCommand {
            zone: "whole-device".into(),
            color: RgbColor {
                red: 0,
                green: 0,
                blue: 0
            },
            brightness: None,
            effect: LightingEffect::Static,
        })),
        Err(DeviceError::NotSupported)
    );
    assert_eq!(driver.capabilities().unwrap(), Capabilities::default());
}
