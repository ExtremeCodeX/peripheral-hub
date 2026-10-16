//! Lifecycle and failure evidence using explicit in-memory simulation only.
use peripheral_hub_daemon::*;
use peripheral_hub_device_registry::DriverRegistry;
use peripheral_hub_device_sdk::*;
use peripheral_hub_ipc::*;
use peripheral_hub_profiles::{Profile, ProfileId};
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
}

#[test]
fn profiles_are_validated_and_persisted_through_the_manager() {
    let mut manager =
        DeviceManager::new(simulated_registry(), Box::new(SimulatedSource::default()));
    let profile = Profile {
        id: ProfileId::new("gaming").unwrap(),
        name: "Gaming".into(),
        device_model: None,
        lighting: Some(LightingCommand {
            zone: "whole-device".into(),
            color: RgbColor {
                red: 255,
                green: 0,
                blue: 0,
            },
            brightness: Some(80),
            effect: LightingEffect::Static,
        }),
    };
    let Reply::Profile(saved) = manager
        .handle(Operation::SaveProfile {
            profile: profile.clone(),
        })
        .unwrap()
    else {
        panic!("save profile")
    };
    assert_eq!(saved, profile);
    let Reply::Profiles(profiles) = manager.handle(Operation::ListProfiles).unwrap() else {
        panic!("list profiles")
    };
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].id.as_str(), "gaming");
    let Reply::Profile(fetched) = manager
        .handle(Operation::GetProfile {
            profile_id: ProfileId::new("gaming").unwrap(),
        })
        .unwrap()
    else {
        panic!("get profile")
    };
    assert_eq!(fetched, profile);

    // Invalid profiles are rejected by contract, not silently stored.
    let mut invalid = profile.clone();
    invalid.name = String::new();
    assert!(matches!(
        manager.handle(Operation::SaveProfile { profile: invalid }),
        Err(Fault::Profile(ProfileFault::Invalid))
    ));

    manager
        .handle(Operation::DeleteProfile {
            profile_id: ProfileId::new("gaming").unwrap(),
        })
        .unwrap();
    assert!(matches!(
        manager.handle(Operation::GetProfile {
            profile_id: ProfileId::new("gaming").unwrap()
        }),
        Err(Fault::Profile(ProfileFault::NotFound))
    ));
}

#[test]
fn apply_profile_enforces_binding_capability_and_live_connection() {
    let mut manager =
        DeviceManager::new(simulated_registry(), Box::new(SimulatedSource::default()));
    manager.refresh().unwrap();
    let id = first_id(&mut manager);
    manager
        .handle(Operation::Connect {
            interface_id: id.clone(),
        })
        .unwrap();

    let mut bound = Profile {
        id: ProfileId::new("bound").unwrap(),
        name: "Bound".into(),
        device_model: Some(DeviceId::new("simulation:peripheral").unwrap()),
        lighting: Some(lighting()),
    };
    manager
        .handle(Operation::SaveProfile {
            profile: bound.clone(),
        })
        .unwrap();
    let Reply::Outcome(outcome) = manager
        .handle(Operation::ApplyProfile {
            interface_id: id.clone(),
            profile_id: ProfileId::new("bound").unwrap(),
        })
        .unwrap()
    else {
        panic!("outcome")
    };
    assert_eq!(outcome.mode, ExecutionMode::Simulated);
    let Reply::State(state) = manager
        .handle(Operation::ReadState {
            interface_id: id.clone(),
        })
        .unwrap()
    else {
        panic!("state")
    };
    assert_eq!(state.lighting, Some(lighting()));

    // An unbound profile applies to any device that supports the request.
    bound.id = ProfileId::new("unbound").unwrap();
    bound.device_model = None;
    manager
        .handle(Operation::SaveProfile { profile: bound })
        .unwrap();
    assert!(matches!(
        manager.handle(Operation::ApplyProfile {
            interface_id: id.clone(),
            profile_id: ProfileId::new("unbound").unwrap(),
        }),
        Ok(Reply::Outcome(_))
    ));

    // A profile bound to another model never applies to this device.
    manager
        .handle(Operation::SaveProfile {
            profile: Profile {
                id: ProfileId::new("foreign").unwrap(),
                name: "Foreign".into(),
                device_model: Some(DeviceId::new("example:other").unwrap()),
                lighting: Some(lighting()),
            },
        })
        .unwrap();
    assert!(matches!(
        manager.handle(Operation::ApplyProfile {
            interface_id: id.clone(),
            profile_id: ProfileId::new("foreign").unwrap(),
        }),
        Err(Fault::Device(DeviceError::NotSupported))
    ));

    // A profile without an applicable preference is not silently applied.
    manager
        .handle(Operation::SaveProfile {
            profile: Profile {
                id: ProfileId::new("empty").unwrap(),
                name: "Empty".into(),
                device_model: None,
                lighting: None,
            },
        })
        .unwrap();
    assert!(matches!(
        manager.handle(Operation::ApplyProfile {
            interface_id: id.clone(),
            profile_id: ProfileId::new("empty").unwrap(),
        }),
        Err(Fault::Device(DeviceError::NotSupported))
    ));

    // Applying to an interface that is not open fails without fabricating success.
    assert!(matches!(
        manager.handle(Operation::ApplyProfile {
            interface_id: DeviceInterfaceId::new("not-live").unwrap(),
            profile_id: ProfileId::new("bound").unwrap(),
        }),
        Err(Fault::Device(DeviceError::NotConnected))
    ));
}

#[test]
fn lighting_scenes_are_capability_gated_and_shape_validated() {
    let mut manager =
        DeviceManager::new(simulated_registry(), Box::new(SimulatedSource::default()));
    manager.refresh().unwrap();
    let id = first_id(&mut manager);
    manager
        .handle(Operation::Connect {
            interface_id: id.clone(),
        })
        .unwrap();

    // One-zone scene succeeds with explicit simulated provenance.
    assert!(matches!(
        manager.handle(Operation::SetLightingScene {
            interface_id: id.clone(),
            scene: LightingScene {
                commands: vec![lighting()]
            },
        }),
        Ok(Reply::Outcome(outcome)) if outcome.mode == ExecutionMode::Simulated
    ));

    // Empty and duplicate-zone scenes are wire-shape rejections.
    assert!(matches!(
        manager.handle(Operation::SetLightingScene {
            interface_id: id.clone(),
            scene: LightingScene { commands: vec![] },
        }),
        Err(Fault::InvalidArgument)
    ));
    assert!(matches!(
        manager.handle(Operation::SetLightingScene {
            interface_id: id.clone(),
            scene: LightingScene {
                commands: vec![lighting(), lighting()]
            },
        }),
        Err(Fault::InvalidArgument)
    ));

    // Undeclared zones and out-of-range brightness fail capability gating.
    let mut wrong_zone = lighting();
    wrong_zone.zone = "side-panel".into();
    assert!(matches!(
        manager.handle(Operation::SetLightingScene {
            interface_id: id.clone(),
            scene: LightingScene {
                commands: vec![wrong_zone]
            },
        }),
        Err(Fault::Device(DeviceError::InvalidArgument(_)))
    ));
    let mut too_bright = lighting();
    too_bright.brightness = Some(101);
    assert!(matches!(
        manager.handle(Operation::SetLightingScene {
            interface_id: id.clone(),
            scene: LightingScene {
                commands: vec![too_bright]
            },
        }),
        Err(Fault::Device(DeviceError::InvalidArgument(_)))
    ));

    // An undeclared effect is NotSupported, never a fabricated success.
    let mut breathing = lighting();
    breathing.effect = LightingEffect::Breathing;
    assert!(matches!(
        manager.handle(Operation::SetLightingScene {
            interface_id: id.clone(),
            scene: LightingScene {
                commands: vec![breathing]
            },
        }),
        Err(Fault::Device(DeviceError::NotSupported))
    ));

    // An oversized scene is rejected by shape validation before any I/O.
    let oversized = (0..=MAX_SCENE_ZONES)
        .map(|index| {
            let mut command = lighting();
            command.zone = format!("zone-{index}");
            command
        })
        .collect();
    assert!(matches!(
        manager.handle(Operation::SetLightingScene {
            interface_id: id.clone(),
            scene: LightingScene {
                commands: oversized
            },
        }),
        Err(Fault::InvalidArgument)
    ));

    // A scene on a non-open interface fails without a fabricated success.
    assert!(matches!(
        manager.handle(Operation::SetLightingScene {
            interface_id: DeviceInterfaceId::new("not-live").unwrap(),
            scene: LightingScene {
                commands: vec![lighting()]
            },
        }),
        Err(Fault::Device(DeviceError::NotConnected))
    ));
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
