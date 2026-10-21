use peripheral_hub_device_registry::DriverRegistry;
use peripheral_hub_device_sdk::*;
use peripheral_hub_transport_hid::simulated::{SIMULATED_PID, SIMULATED_VID};
use std::time::Duration;

/// Register the fictitious software-only model, exclusively in simulation mode.
pub fn simulated_registry() -> DriverRegistry {
    let mut registry = DriverRegistry::new();
    registry
        .register(Box::new(SimulatedFactory {
            metadata: DriverMetadata {
                model_id: DeviceId::new("simulation:peripheral").expect("static model token"),
                display_name: "Fictitious software-only peripheral".into(),
                sdk_version: SDK_CONTRACT_VERSION,
                match_rules: vec![DeviceMatchRule {
                    vendor_id: SIMULATED_VID,
                    product_id: SIMULATED_PID,
                    transport: TransportKind::Hid,
                    interface_number: Some(0),
                    usage: Some(HidUsage {
                        page: 0xff00,
                        usage: 1,
                    }),
                    product: Some("Software-only peripheral".into()),
                    firmware_version: None,
                }],
            },
        }))
        .expect("static simulation metadata");
    registry
}

struct SimulatedFactory {
    metadata: DriverMetadata,
}
impl DriverFactory for SimulatedFactory {
    fn metadata(&self) -> &DriverMetadata {
        &self.metadata
    }
    fn create(&self, _: &DeviceDescriptor) -> DeviceResult<Box<dyn DeviceDriver>> {
        Ok(Box::new(SimulatedDriver::default()))
    }
}

/// Bounded demonstration driver. It serializes synthetic JSON to memory only.
#[derive(Default)]
pub struct SimulatedDriver {
    transport: Option<Box<dyn Transport>>,
    lighting: Option<LightingCommand>,
}
impl DeviceDriver for SimulatedDriver {
    fn open(&mut self, transport: Box<dyn Transport>) -> DeviceResult<()> {
        if self.transport.is_some() {
            return Err(DeviceError::InvalidArgument("driver already open".into()));
        }
        transport.limits().validate()?;
        self.transport = Some(transport);
        Ok(())
    }
    fn close(&mut self) -> DeviceResult<()> {
        self.transport.take().ok_or(DeviceError::NotConnected)?;
        self.lighting = None;
        Ok(())
    }
    fn capabilities(&self) -> DeviceResult<Capabilities> {
        Ok(Capabilities {
            lighting: Some(LightingCapabilities {
                zones: vec!["whole-device".into()],
                brightness: true,
                effects: vec![LightingEffect::Static],
            }),
            display: Some(DisplayCapabilities {
                width: 64,
                height: 32,
                formats: vec![PixelFormat::Rgb888, PixelFormat::Rgb565],
                max_frames_per_second: 30,
                max_frame_bytes: 64 * 32 * 3,
            }),
            ..Capabilities::default()
        })
    }
    fn read_state(&mut self) -> DeviceResult<DeviceState> {
        if self.transport.is_none() {
            return Err(DeviceError::NotConnected);
        }
        Ok(DeviceState {
            mode: ExecutionMode::Simulated,
            lighting: self.lighting.clone(),
        })
    }
    fn execute(&mut self, command: &DeviceCommand) -> DeviceResult<CommandOutcome> {
        self.capabilities()?.validate_command(command)?;
        let transport = self.transport.as_mut().ok_or(DeviceError::NotConnected)?;
        let bytes = serde_json::to_vec(command)
            .map_err(|_| DeviceError::ProtocolError("simulation encoding failed".into()))?;
        let count = transport.write_report(
            ReportKind::Output,
            &HidReport {
                report_id: 0,
                payload: bytes.clone(),
            },
            Duration::from_millis(100),
        )?;
        if count != bytes.len() {
            return Err(DeviceError::TransportError("short simulation write".into()));
        }
        let DeviceCommand::SetLighting(lighting) = command;
        self.lighting = Some(lighting.clone());
        Ok(CommandOutcome {
            mode: ExecutionMode::Simulated,
        })
    }
}
