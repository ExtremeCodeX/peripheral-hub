//! Sanitized, non-destructive HID inventory for research scoping.

use peripheral_hub_device_sdk::DeviceDescriptor;
use peripheral_hub_transport_hid::{DeviceSource, HidDiscovery};
use serde::{Deserialize, Serialize};

use crate::error::{DraError, DraResult};

/// Observed HID usage, kept separate from a model claim.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryUsage {
    /// Usage page.
    pub page: u16,
    /// Usage id.
    pub usage: u16,
}

/// Public inventory row; deliberately excludes serial numbers and OS paths.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryEntry {
    /// Observed USB vendor id.
    pub vendor_id: u16,
    /// Observed USB product id.
    pub product_id: u16,
    /// Observed manufacturer string, when provided.
    #[serde(default)]
    pub manufacturer: Option<String>,
    /// Observed product string, when provided.
    #[serde(default)]
    pub product: Option<String>,
    /// Observed interface number.
    #[serde(default)]
    pub interface_number: Option<u8>,
    /// Observed HID usage.
    #[serde(default)]
    pub usage: Option<InventoryUsage>,
    /// Observed transport category.
    pub transport: String,
    /// Firmware string only when actually observed.
    #[serde(default)]
    pub firmware_version: Option<String>,
}

/// Convert discovery descriptors into sanitized, deterministically ordered rows.
pub fn from_descriptors(descriptors: &[DeviceDescriptor]) -> Vec<InventoryEntry> {
    let mut entries: Vec<InventoryEntry> = descriptors
        .iter()
        .map(|descriptor| InventoryEntry {
            vendor_id: descriptor.vendor_id,
            product_id: descriptor.product_id,
            manufacturer: descriptor.manufacturer.clone(),
            product: descriptor.product.clone(),
            interface_number: descriptor.interface_number,
            usage: descriptor.usage.map(|usage| InventoryUsage {
                page: usage.page,
                usage: usage.usage,
            }),
            transport: match descriptor.transport {
                peripheral_hub_device_sdk::TransportKind::Hid => "hid".to_owned(),
                peripheral_hub_device_sdk::TransportKind::Usb => "usb".to_owned(),
            },
            firmware_version: descriptor.firmware_version.clone(),
        })
        .collect();
    entries.sort();
    entries
}

/// Enumerate physical HID interfaces without opening a handle or writing reports.
///
/// Only the SDK error category is returned; backend messages can contain private
/// device paths and are not surfaced by this tool.
pub fn enumerate() -> DraResult<Vec<InventoryEntry>> {
    let mut discovery =
        HidDiscovery::new().map_err(|error| DraError::Inventory(error.code().to_owned()))?;
    let descriptors = discovery
        .scan()
        .map_err(|error| DraError::Inventory(error.code().to_owned()))?;
    Ok(from_descriptors(&descriptors))
}
