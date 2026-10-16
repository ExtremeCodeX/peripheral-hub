//! Bounded declarative interpreter: a manifest becomes a reviewed factory.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use peripheral_hub_device_sdk::{
    Capabilities, CommandOutcome, DeviceCommand, DeviceDescriptor, DeviceDriver, DeviceError,
    DeviceResult, DriverFactory, DriverMetadata, ExecutionMode, Transport,
};

use crate::{Manifest, ManifestError, OperationKind};

/// A validated manifest exposed as an SDK factory with an explicit provenance mode.
///
/// Registration is explicit; nothing is loaded at runtime. The provenance mode
/// must match the daemon's transport source, and the daemon rejects a mismatch
/// rather than reporting a simulated outcome as physical.
pub struct DeclarativeFactory {
    manifest: Arc<Manifest>,
    metadata: DriverMetadata,
    mode: ExecutionMode,
}

impl DeclarativeFactory {
    /// Validate a manifest and build its metadata for one provenance mode.
    pub fn new(manifest: Manifest, mode: ExecutionMode) -> Result<Self, ManifestError> {
        manifest.validate()?;
        let metadata = DriverMetadata {
            model_id: manifest.model.id.clone(),
            display_name: manifest.model.display_name.clone(),
            sdk_version: manifest.model.contract_version(),
            match_rules: manifest.device_match.clone(),
        };
        metadata.validate()?;
        Ok(Self {
            manifest: Arc::new(manifest),
            metadata,
            mode,
        })
    }

    /// The interpreted manifest, for diagnostics and tests.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}

impl DriverFactory for DeclarativeFactory {
    fn metadata(&self) -> &DriverMetadata {
        &self.metadata
    }

    fn create(&self, _descriptor: &DeviceDescriptor) -> DeviceResult<Box<dyn DeviceDriver>> {
        Ok(Box::new(DeclarativeDriver {
            manifest: Arc::clone(&self.manifest),
            mode: self.mode,
            transport: None,
            last_sent: BTreeMap::new(),
        }))
    }
}

struct DeclarativeDriver {
    manifest: Arc<Manifest>,
    mode: ExecutionMode,
    transport: Option<Box<dyn Transport>>,
    /// Last accepted write per zone; each zone has its own declared report rate.
    last_sent: BTreeMap<String, Instant>,
}

impl DeviceDriver for DeclarativeDriver {
    fn open(&mut self, transport: Box<dyn Transport>) -> DeviceResult<()> {
        if self.transport.is_some() {
            return Err(DeviceError::InvalidArgument(
                "declarative driver is already open".into(),
            ));
        }
        let limits = transport.limits();
        limits.validate()?;
        for operation in &self.manifest.operations {
            let report = &operation.report;
            if report.length > report.capacity(limits) {
                return Err(DeviceError::NotSupported);
            }
            if report.timeout() > limits.max_timeout {
                return Err(DeviceError::InvalidArgument(
                    "declarative timeout exceeds transport limits".into(),
                ));
            }
        }
        self.transport = Some(transport);
        self.last_sent.clear();
        Ok(())
    }

    fn close(&mut self) -> DeviceResult<()> {
        self.transport.take().ok_or(DeviceError::NotConnected)?;
        self.last_sent.clear();
        Ok(())
    }

    fn capabilities(&self) -> DeviceResult<Capabilities> {
        Ok(self.manifest.capabilities.clone())
    }

    fn execute(&mut self, command: &DeviceCommand) -> DeviceResult<CommandOutcome> {
        let DeviceCommand::SetLighting(request) = command;
        // Capability gating happens before any transport access.
        self.manifest.capabilities.validate_command(command)?;
        let operation = self
            .manifest
            .operations
            .iter()
            .find(|operation| {
                operation.kind == OperationKind::SetLighting && operation.zone == request.zone
            })
            .ok_or(DeviceError::NotSupported)?;
        let report = &operation.report;
        let minimum_interval =
            std::time::Duration::from_secs_f64(1.0 / f64::from(report.max_rate_per_second));
        if let Some(last) = self.last_sent.get(&request.zone)
            && last.elapsed() < minimum_interval
        {
            return Err(DeviceError::ProtocolError(
                "declarative rate limit exceeded".into(),
            ));
        }
        let encoded = report.encode(request)?;
        let transport = self.transport.as_mut().ok_or(DeviceError::NotConnected)?;
        let written = transport.write_report(report.kind, &encoded, report.timeout())?;
        if written != report.length {
            return Err(DeviceError::TransportError(
                "incomplete declarative report write".into(),
            ));
        }
        self.last_sent.insert(request.zone.clone(), Instant::now());
        Ok(CommandOutcome { mode: self.mode })
    }
}
