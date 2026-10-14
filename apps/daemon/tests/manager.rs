//! Lifecycle and failure evidence using explicit in-memory simulation only.
use peripheral_hub_daemon::*;
use peripheral_hub_device_registry::DriverRegistry;
use peripheral_hub_device_sdk::*;
use peripheral_hub_ipc::*;
use peripheral_hub_transport_hid::{DeviceSource, simulated::*};

fn first_id(manager: &mut DeviceManager) -> DeviceInterfaceId {
    let Reply::Devices(devices) = manager.handle(Operation::ListDevices).unwrap() else {
        panic!("device reply")
    };
    devices[0].interface_id.clone()
}
fn lighting() -> LightingCommand {
    LightingCommand {
        zone: "whole-device".into(),
        color: RgbColor {
            red: 255,
            green: 0,
            blue: 0,
        },
        brightness: Some(25),
        effect: LightingEffect::Static,
    }
}

#[test]
fn simulated_disconnect_reconnect_and_bounded_event_history() {
    let mut manager =
        DeviceManager::new(simulated_registry(), Box::new(SimulatedSource::default()));
    manager.refresh().unwrap();
    let original = first_id(&mut manager);
    manager
        .handle(Operation::Connect {
            interface_id: original.clone(),
        })
        .unwrap();
    manager
        .handle(Operation::SetLighting {
            interface_id: original.clone(),
            command: lighting(),
        })
        .unwrap();
    let Reply::State(state) = manager
        .handle(Operation::ReadState {
            interface_id: original.clone(),
        })
        .unwrap()
    else {
        panic!("state")
    };
    assert_eq!(state.mode, ExecutionMode::Simulated);
    assert_eq!(state.lighting, Some(lighting()));
    manager
        .handle(Operation::SimulatePresence { present: false })
        .unwrap();
    assert!(matches!(
        manager.handle(Operation::ReadState {
            interface_id: original.clone()
        }),
        Err(Fault::Device(DeviceError::NotConnected))
    ));
    manager
        .handle(Operation::SimulatePresence { present: true })
        .unwrap();
    assert_ne!(first_id(&mut manager), original);
    for _ in 0..70 {
        manager
            .handle(Operation::SimulatePresence { present: false })
            .unwrap();
        manager
            .handle(Operation::SimulatePresence { present: true })
            .unwrap();
    }
    let Reply::Events(batch) = manager.handle(Operation::Events { after: 0 }).unwrap() else {
        panic!("events")
    };
    assert_eq!(batch.events.len(), 128);
    assert!(batch.gap);
    assert!(matches!(
        manager.handle(Operation::Events {
            after: batch.sequence + 1
        }),
        Err(Fault::InvalidArgument)
    ));
}

struct ErrorSource {
    inner: SimulatedSource,
    error: DeviceError,
    opening: bool,
}
impl DeviceSource for ErrorSource {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Simulated
    }
    fn scan(&mut self) -> DeviceResult<Vec<DeviceDescriptor>> {
        self.inner.scan()
    }
    fn open(&mut self, _: &DeviceDescriptor) -> DeviceResult<Box<dyn Transport>> {
        if self.opening {
            return Err(self.error.clone());
        }
        Ok(Box::new(SimulatedTransport {
            error: Some(self.error.clone()),
            last_report: None,
        }))
    }
}

#[test]
fn transport_failures_never_update_state_or_keep_a_connected_handle() {
    for error in [
        DeviceError::Timeout,
        DeviceError::NotConnected,
        DeviceError::PermissionDenied,
        DeviceError::TransportError("private-path".into()),
    ] {
        let mut manager = DeviceManager::new(
            simulated_registry(),
            Box::new(ErrorSource {
                inner: SimulatedSource::default(),
                error: error.clone(),
                opening: false,
            }),
        );
        manager.refresh().unwrap();
        let id = first_id(&mut manager);
        manager
            .handle(Operation::Connect {
                interface_id: id.clone(),
            })
            .unwrap();
        assert_eq!(
            manager
                .handle(Operation::SetLighting {
                    interface_id: id.clone(),
                    command: lighting()
                })
                .unwrap_err(),
            Fault::from(error)
        );
        let Reply::Device(summary) = manager
            .handle(Operation::GetDevice {
                interface_id: id.clone(),
            })
            .unwrap()
        else {
            panic!("summary")
        };
        assert_eq!(summary.status, ConnectionStatus::Failed);
        assert!(matches!(
            manager.handle(Operation::ReadState { interface_id: id }),
            Err(Fault::Device(DeviceError::NotConnected))
        ));
    }
}

#[test]
fn permission_denied_on_open_is_visible_and_unsupported_commands_do_not_succeed() {
    let mut manager = DeviceManager::new(
        simulated_registry(),
        Box::new(ErrorSource {
            inner: SimulatedSource::default(),
            error: DeviceError::PermissionDenied,
            opening: true,
        }),
    );
    manager.refresh().unwrap();
    let id = first_id(&mut manager);
    assert!(matches!(
        manager.handle(Operation::Connect { interface_id: id }),
        Err(Fault::Device(DeviceError::PermissionDenied))
    ));
    for operation in [Operation::GetProfile, Operation::SaveProfile] {
        assert!(matches!(
            manager.handle(operation),
            Err(Fault::Device(DeviceError::NotSupported))
        ));
    }
}

#[test]
fn unknown_interfaces_are_not_opened_and_initialization_failure_stays_physical() {
    let mut manager =
        DeviceManager::new(DriverRegistry::new(), Box::new(SimulatedSource::default()));
    manager.refresh().unwrap();
    let id = first_id(&mut manager);
    assert!(matches!(
        manager.handle(Operation::Connect { interface_id: id }),
        Err(Fault::Device(DeviceError::NotSupported))
    ));
    let mut unavailable = DeviceManager::new(
        DriverRegistry::new(),
        Box::new(UnavailableSource::new(DeviceError::PermissionDenied)),
    );
    assert!(unavailable.refresh().is_err());
    let Reply::Health(health) = unavailable.handle(Operation::Health).unwrap() else {
        panic!("health")
    };
    assert_eq!(health.mode, ExecutionMode::Physical);
    assert_eq!(health.discovery_error, Some(DeviceError::PermissionDenied));
    assert!(unavailable.handle(Operation::ListDevices).is_err());
    assert!(matches!(
        unavailable.handle(Operation::SimulatePresence { present: true }),
        Err(Fault::Device(DeviceError::NotSupported))
    ));
}
