use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{DeviceResult, error::invalid};

macro_rules! identifier {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            /// Validate a nonempty, bounded ASCII token (no paths or whitespace).
            pub fn new(value: impl Into<String>) -> DeviceResult<Self> {
                let value = value.into();
                if value.is_empty()
                    || value.len() > 128
                    || !value.bytes().all(|b| {
                        b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':')
                    })
                {
                    return Err(invalid("identifier must be a 1..128 byte ASCII token"));
                }
                Ok(Self(value))
            }

            /// Access the validated token. Instance/interface tokens remain local.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = crate::DeviceError;
            fn try_from(value: String) -> DeviceResult<Self> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

identifier!(
    DeviceId,
    "Stable model/driver-family identifier, never a physical unit key."
);
identifier!(
    DeviceInstanceId,
    "Opaque physical-unit token assigned by discovery; not VID/PID or a serial."
);
identifier!(
    DeviceInterfaceId,
    "Opaque endpoint token; multiple interfaces can share one physical instance."
);

/// Backend category; no backend is implemented by the SDK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportKind {
    /// HID reports.
    Hid,
    /// Non-HID USB interfaces, reserved for a future transport backend.
    Usb,
}

/// Observed HID usage; absence is represented by `None`, not invented values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HidUsage {
    /// HID usage page.
    pub page: u16,
    /// Usage within that page.
    pub usage: u16,
}

/// Observed interface descriptor. Paths/serials are private local data.
///
/// Discovery must assign unit and endpoint tokens; the SDK cannot infer physical
/// grouping from VID/PID alone. Serde parsing must be followed by `validate()`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceDescriptor {
    /// Physical unit identity, shared by that unit's interfaces.
    pub instance_id: DeviceInstanceId,
    /// Distinct discovered endpoint identity.
    pub interface_id: DeviceInterfaceId,
    /// Observed USB vendor identifier.
    pub vendor_id: u16,
    /// Observed USB product identifier.
    pub product_id: u16,
    /// Transport exposed by this interface.
    pub transport: TransportKind,
    /// Non-UTF-8-safe OS endpoint locator, never a public device identifier.
    pub path: Vec<u8>,
    /// Optional observed interface number.
    pub interface_number: Option<u8>,
    /// Optional observed HID usage.
    pub usage: Option<HidUsage>,
    /// Optional manufacturer string.
    pub manufacturer: Option<String>,
    /// Optional product string.
    pub product: Option<String>,
    /// Optional private serial; never emitted by default logs/Debug.
    pub serial_number: Option<String>,
    /// Firmware version only when actually known.
    pub firmware_version: Option<String>,
}

impl DeviceDescriptor {
    /// Check bounded observed fields without accessing hardware.
    pub fn validate(&self) -> DeviceResult<()> {
        if self.path.is_empty() || self.path.len() > 4096 {
            return Err(invalid("endpoint path must contain 1..4096 bytes"));
        }
        for text in [
            &self.manufacturer,
            &self.product,
            &self.serial_number,
            &self.firmware_version,
        ]
        .into_iter()
        .flatten()
        {
            if text.is_empty() || text.len() > 256 || text.chars().any(char::is_control) {
                return Err(invalid(
                    "descriptor text must contain 1..256 bytes without control characters",
                ));
            }
        }
        if self.transport != TransportKind::Hid && self.usage.is_some() {
            return Err(invalid("HID usage cannot describe a non-HID interface"));
        }
        Ok(())
    }
}

impl fmt::Debug for DeviceDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceDescriptor")
            .field("vendor_id", &self.vendor_id)
            .field("product_id", &self.product_id)
            .field("transport", &self.transport)
            .field("interface_number", &self.interface_number)
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
