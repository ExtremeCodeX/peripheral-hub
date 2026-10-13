use serde::{Deserialize, Serialize};

use crate::{
    Capabilities, ContractVersion, DeviceCommand, DeviceDescriptor, DeviceError, DeviceId,
    DeviceMatchRule, DeviceResult, LightingCommand, Transport, error::invalid,
};

/// Explicitly registered model metadata, independent from Core implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverMetadata {
    /// Model/driver-family ID; one factory per model.
    pub model_id: DeviceId,
    /// Human-readable public driver name.
    pub display_name: String,
    /// SDK version the factory implements.
    pub sdk_version: ContractVersion,
    /// Alternatives; each rule is an exact conjunction of constraints.
    pub match_rules: Vec<DeviceMatchRule>,
}

impl DriverMetadata {
    /// Reject incompatible versions, empty rules, and weak matching evidence.
    pub fn validate(&self) -> DeviceResult<()> {
        self.sdk_version.ensure_supported()?;
        if self.display_name.is_empty()
            || self.display_name.len() > 128
            || self.display_name.chars().any(char::is_control)
            || self.match_rules.is_empty()
            || self.match_rules.len() > 256
        {
            return Err(invalid("driver metadata name or match rules are invalid"));
        }
        for rule in &self.match_rules {
            rule.validate()?;
        }
        Ok(())
    }
}

/// Explicit result provenance, never inferred from a successful software test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionMode {
    /// Software-only simulated behavior.
    Simulated,
    /// Real backend operation; not automatically a hardware validation claim.
    Physical,
}

/// Optional observed state; unknown lighting is `None`, not a successful update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceState {
    /// Origin of the state observation.
    pub mode: ExecutionMode,
    /// Last actually observed lighting when query support exists.
    pub lighting: Option<LightingCommand>,
}

/// Successful command outcome, returned only after the driver's operation succeeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandOutcome {
    /// The operation's actual mode; clients must expose simulation explicitly.
    pub mode: ExecutionMode,
}

/// Synchronous connection contract; implementations own their supplied transport.
///
/// No implementations ship in Phase 1. Drivers must validate capabilities and
/// commands, keep connection state truthful, and propagate transport failures.
pub trait DeviceDriver: Send {
    /// Attach/open a reviewed transport for this selected descriptor.
    fn open(&mut self, transport: Box<dyn Transport>) -> DeviceResult<()>;
    /// Release transport resources; return actual cleanup errors if any.
    fn close(&mut self) -> DeviceResult<()>;
    /// Query capabilities; callers must run semantic validation on the result.
    fn capabilities(&self) -> DeviceResult<Capabilities>;
    /// Query observed state. Absent query support is not simulated success.
    fn read_state(&mut self) -> DeviceResult<DeviceState> {
        Err(DeviceError::NotSupported)
    }
    /// Execute a supported typed command. Default is explicitly unsupported.
    fn execute(&mut self, _command: &DeviceCommand) -> DeviceResult<CommandOutcome> {
        Err(DeviceError::NotSupported)
    }
}

/// Reviewed factory registered at build/startup time; no runtime native loading.
pub trait DriverFactory: Send + Sync {
    /// Immutable model metadata; Registry snapshots and validates it at registration.
    fn metadata(&self) -> &DriverMetadata;
    /// Construct a driver for a matched descriptor without physical I/O.
    fn create(&self, descriptor: &DeviceDescriptor) -> DeviceResult<Box<dyn DeviceDriver>>;
}
