//! Fictitious native driver example built on the Peripheral Hub SDK.
//!
//! This is NOT real hardware support and names no real vendor, device, or
//! protocol. It exists to show the compiled-driver contract, matching evidence,
//! capability gating and bounded single-report I/O. It never performs firmware,
//! bootloader or persistent-memory operations, and it is not registered in the
//! daemon's runtime; it is compiled and tested here only as a reference.

use std::time::Duration;

use peripheral_hub_device_sdk::{
    Capabilities, CommandOutcome, DeviceCommand, DeviceDescriptor, DeviceDriver, DeviceError,
    DeviceId, DeviceMatchRule, DeviceResult, DriverFactory, DriverMetadata, ExecutionMode,
    HidReport, HidUsage, LightingCapabilities, LightingEffect, ReportKind, SDK_CONTRACT_VERSION,
    Transport, TransportKind,
};

const REPORT_ID: u8 = 0;
const REPORT_LENGTH: usize = 4;
const REPORT_TIMEOUT: Duration = Duration::from_millis(100);

/// Reviewed factory for the fictitious native example model.
pub struct ExampleNativeFactory {
    metadata: DriverMetadata,
}

impl ExampleNativeFactory {
    /// Build the factory with explicit, evidence-backed match constraints.
    pub fn new() -> Result<Self, DeviceError> {
        let metadata = DriverMetadata {
            model_id: DeviceId::new("example-native:lighting")?,
            display_name: "Fictitious native lighting example".into(),
            sdk_version: SDK_CONTRACT_VERSION,
            match_rules: vec![DeviceMatchRule {
                vendor_id: 0xf0f1,
                product_id: 0x0002,
                transport: TransportKind::Hid,
                interface_number: Some(4),
                usage: Some(HidUsage {
                    page: 0xff01,
                    usage: 2,
                }),
                product: None,
                firmware_version: None,
            }],
        };
        metadata.validate()?;
        Ok(Self { metadata })
    }
}

impl DriverFactory for ExampleNativeFactory {
    fn metadata(&self) -> &DriverMetadata {
        &self.metadata
    }

    fn create(&self, _descriptor: &DeviceDescriptor) -> DeviceResult<Box<dyn DeviceDriver>> {
        Ok(Box::new(ExampleNativeDriver { transport: None }))
    }
}

struct ExampleNativeDriver {
    transport: Option<Box<dyn Transport>>,
}

impl DeviceDriver for ExampleNativeDriver {
    fn open(&mut self, transport: Box<dyn Transport>) -> DeviceResult<()> {
        if self.transport.is_some() {
            return Err(DeviceError::InvalidArgument(
                "example native driver is already open".into(),
            ));
        }
        let limits = transport.limits();
        limits.validate()?;
        if limits.output_bytes < REPORT_LENGTH || limits.max_timeout < REPORT_TIMEOUT {
            return Err(DeviceError::NotSupported);
        }
        self.transport = Some(transport);
        Ok(())
    }

    fn close(&mut self) -> DeviceResult<()> {
        self.transport.take().ok_or(DeviceError::NotConnected)?;
        Ok(())
    }

    fn capabilities(&self) -> DeviceResult<Capabilities> {
        Ok(Capabilities {
            lighting: Some(LightingCapabilities {
                zones: vec!["whole-device".into()],
                brightness: false,
                effects: vec![LightingEffect::Static],
            }),
            ..Capabilities::default()
        })
    }

    fn execute(&mut self, command: &DeviceCommand) -> DeviceResult<CommandOutcome> {
        let DeviceCommand::SetLighting(request) = command;
        self.capabilities()?.validate_command(command)?;
        if request.zone != "whole-device" {
            return Err(DeviceError::NotSupported);
        }
        let report = HidReport {
            report_id: REPORT_ID,
            payload: vec![
                0x22,
                request.color.red,
                request.color.green,
                request.color.blue,
            ],
        };
        let transport = self.transport.as_mut().ok_or(DeviceError::NotConnected)?;
        let written = transport.write_report(ReportKind::Output, &report, REPORT_TIMEOUT)?;
        if written != REPORT_LENGTH {
            return Err(DeviceError::TransportError(
                "incomplete example native report write".into(),
            ));
        }
        Ok(CommandOutcome {
            mode: ExecutionMode::Physical,
        })
    }
}
