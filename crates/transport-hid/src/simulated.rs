//! Explicit software-only source/transport, never used as a physical fallback.

use crate::DeviceSource;
use peripheral_hub_device_sdk::*;
use std::time::Duration;

/// Synthetic VID, used only in the explicitly selected simulation backend.
pub const SIMULATED_VID: u16 = 0xffff;
/// Synthetic PID, not a real compatibility claim.
pub const SIMULATED_PID: u16 = 0xfffe;

/// Controllable simulation source; absent devices produce an empty snapshot.
pub struct SimulatedSource {
    present: bool,
    generation: u64,
    epoch: String,
}
impl SimulatedSource {
    /// Initialize explicit simulation with fresh session-local identities.
    pub fn new() -> DeviceResult<Self> {
        Ok(Self {
            present: true,
            generation: 1,
            epoch: crate::session_epoch()?,
        })
    }
}
impl Default for SimulatedSource {
    fn default() -> Self {
        Self::new().expect("OS session entropy unavailable")
    }
}
impl DeviceSource for SimulatedSource {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Simulated
    }
    fn scan(&mut self) -> DeviceResult<Vec<DeviceDescriptor>> {
        if !self.present {
            return Ok(vec![]);
        }
        Ok(vec![DeviceDescriptor {
            instance_id: DeviceInstanceId::new(format!(
                "simulated-{}-unit-{}",
                self.epoch, self.generation
            ))?,
            interface_id: DeviceInterfaceId::new(format!(
                "simulated-{}-interface-{}",
                self.epoch, self.generation
            ))?,
            vendor_id: SIMULATED_VID,
            product_id: SIMULATED_PID,
            transport: TransportKind::Hid,
            path: format!("simulation-only-{}", self.generation).into_bytes(),
            interface_number: Some(0),
            usage: Some(HidUsage {
                page: 0xff00,
                usage: 1,
            }),
            manufacturer: Some("Fictitious simulation".into()),
            product: Some("Software-only peripheral".into()),
            serial_number: None,
            firmware_version: None,
        }])
    }
    fn open(&mut self, descriptor: &DeviceDescriptor) -> DeviceResult<Box<dyn Transport>> {
        if !self.scan()?.iter().any(|current| current == descriptor) {
            return Err(DeviceError::NotConnected);
        }
        Ok(Box::new(CheckedTransport::new(
            SimulatedTransport::default(),
        )?))
    }
    fn set_simulated_presence(&mut self, present: bool) -> DeviceResult<()> {
        if present && !self.present {
            self.generation += 1;
        }
        self.present = present;
        Ok(())
    }
}

/// Bounded in-memory transport with explicit configurable failure injection.
#[derive(Default)]
pub struct SimulatedTransport {
    /// Injected error, propagated without pretending a successful operation.
    pub error: Option<DeviceError>,
    /// Last software-only written report. Not a physical packet capture.
    pub last_report: Option<HidReport>,
}
impl Transport for SimulatedTransport {
    fn limits(&self) -> TransportLimits {
        TransportLimits {
            input_bytes: 0,
            output_bytes: 1024,
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
        timeout: Duration,
    ) -> DeviceResult<usize> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        if kind != ReportKind::Output || report.report_id != 0 {
            return Err(DeviceError::NotSupported);
        }
        if timeout.is_zero()
            || timeout > self.limits().max_timeout
            || report.payload.is_empty()
            || report.payload.len() > self.limits().output_bytes
        {
            return Err(DeviceError::InvalidArgument(
                "invalid simulated report bounds".into(),
            ));
        }
        self.last_report = Some(report.clone());
        Ok(report.payload.len())
    }
}
