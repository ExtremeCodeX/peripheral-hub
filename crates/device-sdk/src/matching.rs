use serde::{Deserialize, Serialize};

use crate::{DeviceDescriptor, DeviceResult, HidUsage, TransportKind, error::invalid};

/// A conjunction of exact evidence constraints, never a VID/PID-only wildcard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceMatchRule {
    /// Exact observed vendor ID.
    pub vendor_id: u16,
    /// Exact observed product ID.
    pub product_id: u16,
    /// Required transport.
    pub transport: TransportKind,
    /// Required interface number, if used as evidence.
    pub interface_number: Option<u8>,
    /// Required HID usage pair, if used as evidence.
    pub usage: Option<HidUsage>,
    /// Optional exact product discriminator.
    pub product: Option<String>,
    /// Optional exact supported firmware; unknown firmware does not match.
    pub firmware_version: Option<String>,
}

impl DeviceMatchRule {
    /// Require interface/usage evidence and reject incompatible constraints.
    pub fn validate(&self) -> DeviceResult<()> {
        if self.interface_number.is_none() && self.usage.is_none() {
            return Err(invalid(
                "matching requires interface or HID usage evidence beyond VID/PID",
            ));
        }
        if self.transport != TransportKind::Hid && self.usage.is_some() {
            return Err(invalid("non-HID matching cannot use HID usage"));
        }
        for text in [&self.product, &self.firmware_version]
            .into_iter()
            .flatten()
        {
            if text.is_empty() || text.len() > 256 || text.chars().any(char::is_control) {
                return Err(invalid("match discriminator is invalid"));
            }
        }
        Ok(())
    }

    /// Match all constraints; absent descriptor evidence fails a required check.
    pub fn matches(&self, descriptor: &DeviceDescriptor) -> DeviceResult<bool> {
        self.validate()?;
        descriptor.validate()?;
        Ok(self.vendor_id == descriptor.vendor_id
            && self.product_id == descriptor.product_id
            && self.transport == descriptor.transport
            && self
                .interface_number
                .is_none_or(|n| descriptor.interface_number == Some(n))
            && self
                .usage
                .is_none_or(|usage| descriptor.usage == Some(usage))
            && self
                .product
                .as_ref()
                .is_none_or(|s| descriptor.product.as_ref() == Some(s))
            && self
                .firmware_version
                .as_ref()
                .is_none_or(|s| descriptor.firmware_version.as_ref() == Some(s)))
    }
}
