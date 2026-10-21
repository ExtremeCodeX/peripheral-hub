use peripheral_hub_device_sdk::*;
use peripheral_hub_macros::*;
use peripheral_hub_profiles::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Implemented IPC wire contract; independent from SDK 1.0.
///
/// Version 2 extends version 1 with typed profile operations.
/// Version 3 adds the profile-application operation.
/// Version 4 adds the generic multi-zone lighting-scene operation.
/// Version 5 adds the persistent macro management operations.
pub const IPC_VERSION: u16 = 5;
/// Maximum live interfaces exposed by this bounded daemon.
pub const MAX_INTERFACES: usize = 256;
/// Maximum zones in one generic lighting scene (matches the capability ceiling).
pub const MAX_SCENE_ZONES: usize = 256;

/// Caller request with explicit version and correlation token.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// Must equal the implemented IPC version.
    pub version: u16,
    /// Caller-chosen correlation token; not an authentication credential.
    pub request_id: u64,
    /// Typed operation. No raw reports or executable commands are accepted.
    pub operation: Operation,
}

/// A generic multi-zone lighting request over the SDK `LightingCommand`.
///
/// Shape validation (count and unique zones) runs before capability gating; the
/// daemon then checks every zone, effect and brightness against the open
/// driver's declared capabilities before any report is written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightingScene {
    /// One command per target zone; zones must be unique and 1..=256 entries.
    pub commands: Vec<LightingCommand>,
}

impl LightingScene {
    /// Validate only the wire shape, without device capabilities.
    pub fn validate(&self) -> Result<(), Fault> {
        if self.commands.is_empty() || self.commands.len() > MAX_SCENE_ZONES {
            return Err(Fault::InvalidArgument);
        }
        let mut zones = BTreeSet::new();
        for command in &self.commands {
            if !zones.insert(command.zone.as_str()) {
                return Err(Fault::InvalidArgument);
            }
        }
        Ok(())
    }
}

/// Allowed user-session operations. No raw reports or executable commands exist.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "operation",
    content = "parameters",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Operation {
    /// Daemon version, mode and last enumeration result.
    Health,
    /// Sanitized summary of live discovered interfaces.
    ListDevices,
    /// Inspect one live interface.
    GetDevice {
        /// Opaque endpoint token.
        interface_id: DeviceInterfaceId,
    },
    /// Query an open driver's capabilities.
    GetCapabilities {
        /// Opaque endpoint token.
        interface_id: DeviceInterfaceId,
    },
    /// Open only a matched reviewed driver; unknown devices are never opened.
    Connect {
        /// Opaque endpoint token.
        interface_id: DeviceInterfaceId,
    },
    /// Close a managed handle.
    Disconnect {
        /// Opaque endpoint token.
        interface_id: DeviceInterfaceId,
    },
    /// Typed, capability-gated volatile lighting request.
    SetLighting {
        /// Opaque endpoint token.
        interface_id: DeviceInterfaceId,
        /// Lighting parameters.
        command: LightingCommand,
    },
    /// Generic multi-zone lighting scene applied to one open driver.
    ///
    /// Every entry is capability-validated before any report is written, so an
    /// unsupported zone rejects the whole scene instead of partially applying it.
    SetLightingScene {
        /// Opaque endpoint token of an already connected device.
        interface_id: DeviceInterfaceId,
        /// Per-zone lighting commands.
        scene: LightingScene,
    },
    /// Query actual driver state.
    ReadState {
        /// Opaque endpoint token.
        interface_id: DeviceInterfaceId,
    },
    /// Bounded connection-event journal; poll with the last sequence number.
    Events {
        /// Last consumed sequence; zero requests retained history.
        after: u64,
    },
    /// Change presence only in explicitly enabled simulation mode.
    SimulatePresence {
        /// Desired software-only presence.
        present: bool,
    },
    /// List stored profile summaries; no full payloads are returned.
    ListProfiles,
    /// Fetch one stored profile by validated identifier.
    GetProfile {
        /// Validated profile identifier.
        profile_id: ProfileId,
    },
    /// Insert or replace a validated profile; no raw report payload is accepted.
    SaveProfile {
        /// Fully validated profile payload.
        profile: Profile,
    },
    /// Remove one stored profile.
    DeleteProfile {
        /// Validated profile identifier.
        profile_id: ProfileId,
    },
    /// Apply a stored profile's supported preferences to an open driver.
    ///
    /// The manager enforces the optional model binding and capability-gates the
    /// embedded lighting request; a profile with no applicable preference or a
    /// model mismatch yields `NotSupported`, never a fabricated success.
    ApplyProfile {
        /// Opaque endpoint token of an already connected device.
        interface_id: DeviceInterfaceId,
        /// Validated profile identifier.
        profile_id: ProfileId,
    },
    /// List stored macro summaries; no full payloads are returned.
    ListMacros,
    /// Fetch one stored macro by validated identifier.
    GetMacro {
        /// Validated macro identifier.
        macro_id: MacroId,
    },
    /// Insert or replace a validated macro; persistence never executes it.
    ///
    /// The manager validates the macro and, when a profile link is present,
    /// confirms the referenced profile exists before writing.
    SaveMacro {
        /// Fully validated macro payload (wire key `macro`; the Rust field is
        /// `macro_value` because `macro` is a reserved keyword).
        #[serde(rename = "macro")]
        macro_value: Macro,
    },
    /// Remove one stored macro.
    DeleteMacro {
        /// Validated macro identifier.
        macro_id: MacroId,
    },
}

/// Actual connection status, separate from physical validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    /// Enumerated, no open handle.
    Discovered,
    /// Driver owns an open transport.
    Connected,
    /// Opening or an operation failed; no trusted live handle.
    Failed,
    /// Removed from enumeration or explicitly closed.
    Disconnected,
}

/// Local IPC device view deliberately excludes raw locator and serial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceSummary {
    /// Physical-unit token with discovery limitations documented separately.
    pub instance_id: DeviceInstanceId,
    /// Distinct endpoint token.
    pub interface_id: DeviceInterfaceId,
    /// Observed vendor ID.
    pub vendor_id: u16,
    /// Observed product ID.
    pub product_id: u16,
    /// Optional observed manufacturer.
    pub manufacturer: Option<String>,
    /// Optional observed product.
    pub product: Option<String>,
    /// Optional interface number.
    pub interface_number: Option<u8>,
    /// Optional HID usage.
    pub usage: Option<HidUsage>,
    /// Unique selected model; None means unknown/unsupported.
    pub model: Option<DeviceId>,
    /// Hardware or explicit software simulation.
    pub mode: ExecutionMode,
    /// Actual lifecycle status.
    pub status: ConnectionStatus,
    /// Sanitized typed error, never a raw backend message.
    pub last_error: Option<DeviceError>,
}

/// Diagnostic health even when HID enumeration is unavailable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Health {
    /// Fresh daemon epoch; reset event cursors when this token changes.
    pub session_id: String,
    /// Application package version.
    pub daemon_version: String,
    /// SDK contract supported by this daemon.
    pub sdk_version: ContractVersion,
    /// Actual backend mode.
    pub mode: ExecutionMode,
    /// Last sanitized enumeration error, if any; never converted to an empty success.
    pub discovery_error: Option<DeviceError>,
    /// Last emitted event sequence.
    pub sequence: u64,
}

/// One lifecycle transition in a bounded journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionEvent {
    /// Monotonically increasing sequence within this daemon process.
    pub sequence: u64,
    /// Changed endpoint token.
    pub interface_id: DeviceInterfaceId,
    /// Actual new status.
    pub status: ConnectionStatus,
    /// Explicit simulation/physical provenance.
    pub mode: ExecutionMode,
}

/// Journal reply; a gap requires refreshing the current device list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventBatch {
    /// Newest sequence for the next poll.
    pub sequence: u64,
    /// True when requested history was evicted.
    pub gap: bool,
    /// At most 128 retained lifecycle transitions.
    pub events: Vec<ConnectionEvent>,
}

/// Typed server result; library errors remain explicit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "result",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Reply {
    /// Daemon diagnostics.
    Health(Health),
    /// Live summaries.
    Devices(Vec<DeviceSummary>),
    /// One summary.
    Device(DeviceSummary),
    /// Validated driver declarations.
    Capabilities(Capabilities),
    /// Observed state.
    State(DeviceState),
    /// Successful command with explicit provenance.
    Outcome(CommandOutcome),
    /// Lifecycle transitions.
    Events(EventBatch),
    /// Stored profile summaries.
    Profiles(Vec<ProfileSummary>),
    /// One stored profile.
    Profile(Profile),
    /// Stored macro summaries.
    Macros(Vec<MacroSummary>),
    /// One stored macro.
    Macro(Macro),
    /// Completed non-hardware administrative action.
    Ack,
}

/// Structured wire failures. No sensitive free-form backend error is exported.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(
    tag = "code",
    content = "detail",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Fault {
    /// Unsupported wire version.
    #[error("incompatible IPC version")]
    IncompatibleVersion,
    /// Invalid input or ambiguous identity.
    #[error("invalid request")]
    InvalidArgument,
    /// Request queue/concurrency budget unavailable.
    #[error("daemon busy")]
    Busy,
    /// Deadline exceeded; never automatically replay a mutation.
    #[error("request timed out")]
    Timeout,
    /// Sanitized SDK failure.
    #[error("device operation failed: {0}")]
    Device(DeviceError),
    /// Profile contract or persistence failure, projected to a stable code.
    #[error("profile operation failed: {0}")]
    Profile(ProfileFault),
    /// Macro contract or persistence failure, projected to a stable code.
    #[error("macro operation failed: {0}")]
    Macro(MacroFault),
    /// Internal infrastructure failure, without exception detail.
    #[error("internal daemon error")]
    Internal,
}

/// Stable, wire-safe projection of profile failures; no storage detail leaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileFault {
    /// A field failed contract validation.
    Invalid,
    /// The schema version is not understood.
    UnsupportedVersion,
    /// The requested profile does not exist.
    NotFound,
    /// Local persistence failed.
    Storage,
}

impl std::fmt::Display for ProfileFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "invalid",
            Self::UnsupportedVersion => "unsupported_version",
            Self::NotFound => "not_found",
            Self::Storage => "storage",
        })
    }
}

/// Stable, wire-safe projection of macro failures; no storage detail leaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MacroFault {
    /// A field, limit or referenced profile failed contract validation.
    Invalid,
    /// The schema version is not understood.
    UnsupportedVersion,
    /// The requested macro does not exist.
    NotFound,
    /// Local persistence failed.
    Storage,
}

impl std::fmt::Display for MacroFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "invalid",
            Self::UnsupportedVersion => "unsupported_version",
            Self::NotFound => "not_found",
            Self::Storage => "storage",
        })
    }
}

/// Replace backend detail strings before logs or IPC exports.
pub fn sanitize_error(error: DeviceError) -> DeviceError {
    match error {
        DeviceError::InvalidArgument(_) => {
            DeviceError::InvalidArgument("contract rejected input".into())
        }
        DeviceError::TransportError(_) => {
            DeviceError::TransportError("backend operation failed".into())
        }
        DeviceError::ProtocolError(_) => {
            DeviceError::ProtocolError("protocol validation failed".into())
        }
        other => other,
    }
}
impl From<DeviceError> for Fault {
    fn from(error: DeviceError) -> Self {
        Self::Device(sanitize_error(error))
    }
}
impl From<ProfileError> for Fault {
    fn from(error: ProfileError) -> Self {
        let fault = match error {
            ProfileError::Invalid(_) | ProfileError::InvalidJson => ProfileFault::Invalid,
            ProfileError::UnsupportedVersion => ProfileFault::UnsupportedVersion,
            ProfileError::NotFound => ProfileFault::NotFound,
            ProfileError::Storage(_) => ProfileFault::Storage,
        };
        Self::Profile(fault)
    }
}
impl From<MacroError> for Fault {
    fn from(error: MacroError) -> Self {
        let fault = match error {
            MacroError::Invalid(_) | MacroError::InvalidJson => MacroFault::Invalid,
            MacroError::UnsupportedVersion => MacroFault::UnsupportedVersion,
            MacroError::NotFound => MacroFault::NotFound,
            MacroError::Storage(_) => MacroFault::Storage,
        };
        Self::Macro(fault)
    }
}

/// Correlated versioned reply; malformed frames may be closed without a reply.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    /// Server's supported wire version.
    pub version: u16,
    /// Echo of request correlation token.
    pub request_id: u64,
    /// Actual outcome or structured error.
    pub result: Result<Reply, Fault>,
}
