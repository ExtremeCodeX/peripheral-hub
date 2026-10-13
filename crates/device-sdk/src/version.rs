use serde::{Deserialize, Serialize};

use crate::{DeviceError, DeviceResult};

/// SDK contract version, independent from Cargo package and future IPC versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractVersion {
    /// Breaking changes increment the major version.
    pub major: u16,
    /// Additive changes increment the minor version.
    pub minor: u16,
}

/// The implemented contract: version 1.0.
pub const SDK_CONTRACT_VERSION: ContractVersion = ContractVersion { major: 1, minor: 0 };

impl ContractVersion {
    /// Accept the same major and no newer minor; reject unknown future contracts.
    pub fn ensure_supported(self) -> DeviceResult<()> {
        if self.major == SDK_CONTRACT_VERSION.major
            && self.minor.cmp(&SDK_CONTRACT_VERSION.minor).is_le()
        {
            Ok(())
        } else {
            Err(DeviceError::IncompatibleVersion)
        }
    }
}

/// Versioned SDK data, not an IPC envelope or a profile schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Versioned<T> {
    /// Explicit SDK version; consumers must check it before using the payload.
    pub version: ContractVersion,
    /// Contract-specific data, requiring its own semantic validation.
    pub data: T,
}

impl<T> Versioned<T> {
    /// Wrap locally generated data using the implemented contract.
    pub fn new(data: T) -> Self {
        Self {
            version: SDK_CONTRACT_VERSION,
            data,
        }
    }

    /// Extract data only when the envelope's version is understood.
    pub fn into_supported(self) -> DeviceResult<T> {
        self.version.ensure_supported()?;
        Ok(self.data)
    }
}
