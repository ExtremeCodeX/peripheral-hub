use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CString;

use hidapi::HidApi;
use peripheral_hub_device_sdk::*;

/// Discovery/open boundary used by the daemon; enumeration must not send reports.
pub trait DeviceSource: Send {
    /// Whether observations originate from hardware or explicit simulation.
    fn mode(&self) -> ExecutionMode;
    /// Return a complete observed snapshot or an explicit failure.
    fn scan(&mut self) -> DeviceResult<Vec<DeviceDescriptor>>;
    /// Open only a previously selected descriptor using reviewed transport limits.
    fn open(&mut self, descriptor: &DeviceDescriptor) -> DeviceResult<Box<dyn Transport>>;
    /// Optional simulation control; hardware sources must reject it.
    fn set_simulated_presence(&mut self, _present: bool) -> DeviceResult<()> {
        Err(DeviceError::NotSupported)
    }
}

/// Persistent hidapi context and conservative session-local identity allocator.
pub struct HidDiscovery {
    api: HidApi,
    ids: IdentityAllocator,
}

impl HidDiscovery {
    /// Initialize enumeration, without opening device handles or sending reports.
    pub fn new() -> DeviceResult<Self> {
        Ok(Self {
            api: HidApi::new().map_err(crate::io::map_error)?,
            ids: IdentityAllocator::new()?,
        })
    }

    /// Open a specific validated, still-observed interface with reviewed bounds.
    /// No report is sent during opening. Unknown protocols must never call this.
    pub fn open_reviewed(
        &self,
        descriptor: &DeviceDescriptor,
        policy: crate::HidPolicy,
    ) -> DeviceResult<Box<dyn Transport>> {
        descriptor.validate()?;
        policy.validate()?;
        let info = self
            .api
            .device_list()
            .find(|info| {
                info.path().to_bytes() == descriptor.path
                    && info.vendor_id() == descriptor.vendor_id
                    && info.product_id() == descriptor.product_id
                    && u8::try_from(info.interface_number()).ok() == descriptor.interface_number
                    && observed_usage(info.usage_page(), info.usage()) == descriptor.usage
            })
            .ok_or(DeviceError::NotConnected)?;
        let path = CString::new(info.path().to_bytes())
            .map_err(|_| DeviceError::InvalidArgument("invalid HID locator".into()))?;
        let device = self.api.open_path(&path).map_err(crate::io::map_error)?;
        Ok(Box::new(CheckedTransport::new(crate::HidTransport::new(
            device, policy,
        )?)?))
    }
}

impl DeviceSource for HidDiscovery {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Physical
    }

    fn scan(&mut self) -> DeviceResult<Vec<DeviceDescriptor>> {
        self.api.refresh_devices().map_err(crate::io::map_error)?;
        let observations: Vec<_> = self
            .api
            .device_list()
            .map(|info| Observation {
                path: info.path().to_bytes().to_vec(),
                vendor_id: info.vendor_id(),
                product_id: info.product_id(),
                interface_number: u8::try_from(info.interface_number()).ok(),
                usage: observed_usage(info.usage_page(), info.usage()),
                manufacturer: bounded_text(info.manufacturer_string()),
                product: bounded_text(info.product_string()),
                serial: bounded_text(info.serial_number()),
            })
            .collect();
        self.ids.snapshot(observations)
    }

    fn open(&mut self, _descriptor: &DeviceDescriptor) -> DeviceResult<Box<dyn Transport>> {
        // No real model/policy is registered in Phase 2. Never guess report sizes.
        Err(DeviceError::NotSupported)
    }
}

fn observed_usage(page: u16, usage: u16) -> Option<HidUsage> {
    if page == 0 && usage == 0 {
        None
    } else {
        Some(HidUsage { page, usage })
    }
}

fn bounded_text(value: Option<&str>) -> Option<String> {
    value
        .filter(|s| !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control))
        .map(str::to_owned)
}

struct Observation {
    path: Vec<u8>,
    vendor_id: u16,
    product_id: u16,
    interface_number: Option<u8>,
    usage: Option<HidUsage>,
    manufacturer: Option<String>,
    product: Option<String>,
    serial: Option<String>,
}

struct IdentityAllocator {
    epoch: String,
    next: u64,
    endpoints: BTreeMap<Vec<u8>, (DeviceInstanceId, DeviceInterfaceId)>,
}

impl IdentityAllocator {
    fn new() -> DeviceResult<Self> {
        Ok(Self {
            epoch: session_epoch()?,
            next: 0,
            endpoints: BTreeMap::new(),
        })
    }
    fn snapshot(&mut self, observations: Vec<Observation>) -> DeviceResult<Vec<DeviceDescriptor>> {
        // Serial alone is not proof of a unique unit (duplicate serials exist).
        // Until OS parent identity is implemented, group only identical paths.
        // Collapse hidapi duplicate usage collections for a path to unknown usage.
        let mut unique: BTreeMap<Vec<u8>, Observation> = BTreeMap::new();
        for observation in observations {
            if let Some(previous) = unique.get_mut(&observation.path) {
                if previous.vendor_id != observation.vendor_id
                    || previous.product_id != observation.product_id
                {
                    return Err(DeviceError::ProtocolError(
                        "conflicting HID enumeration identity".into(),
                    ));
                }
                if previous.usage != observation.usage {
                    previous.usage = None;
                }
            } else {
                unique.insert(observation.path.clone(), observation);
            }
        }
        let live: BTreeSet<_> = unique.keys().cloned().collect();
        self.endpoints.retain(|path, _| live.contains(path));
        let mut result = Vec::new();
        for (path, observation) in unique {
            let ids = if let Some(ids) = self.endpoints.get(&path) {
                ids.clone()
            } else {
                self.next = self.next.checked_add(1).ok_or_else(|| {
                    DeviceError::ProtocolError("identity sequence exhausted".into())
                })?;
                let ids = (
                    DeviceInstanceId::new(format!("hid-{}-unit-{}", self.epoch, self.next))?,
                    DeviceInterfaceId::new(format!("hid-{}-interface-{}", self.epoch, self.next))?,
                );
                self.endpoints.insert(path.clone(), ids.clone());
                ids
            };
            // Locator hashes are deliberately not exported as permanent identity.
            let descriptor = DeviceDescriptor {
                instance_id: ids.0,
                interface_id: ids.1,
                vendor_id: observation.vendor_id,
                product_id: observation.product_id,
                transport: TransportKind::Hid,
                path,
                interface_number: observation.interface_number,
                usage: observation.usage,
                manufacturer: observation.manufacturer,
                product: observation.product,
                serial_number: observation.serial,
                firmware_version: None,
            };
            descriptor.validate()?;
            result.push(descriptor);
        }
        Ok(result)
    }
}

/// Fresh namespace epoch; not an authentication secret or hardware identifier.
pub fn session_epoch() -> DeviceResult<String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)
        .map_err(|_| DeviceError::TransportError("session entropy unavailable".into()))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn observation(path: u8) -> Observation {
        Observation {
            path: vec![path],
            vendor_id: 0xffff,
            product_id: 0xfffe,
            interface_number: Some(1),
            usage: Some(HidUsage {
                page: 0xff00,
                usage: 1,
            }),
            manufacturer: None,
            product: None,
            serial: Some("duplicate-test-serial".into()),
        }
    }
    #[test]
    fn duplicate_vid_pid_serials_do_not_merge_units_and_reconnect_gets_new_tokens() {
        let mut allocator = IdentityAllocator::new().unwrap();
        let first = allocator
            .snapshot(vec![observation(1), observation(2)])
            .unwrap();
        assert_ne!(first[0].instance_id, first[1].instance_id);
        assert_eq!(
            allocator.snapshot(vec![observation(1)]).unwrap()[0].interface_id,
            first[0].interface_id
        );
        allocator.snapshot(vec![]).unwrap();
        assert_ne!(
            allocator.snapshot(vec![observation(1)]).unwrap()[0].interface_id,
            first[0].interface_id
        );
    }
    #[test]
    fn duplicate_collections_collapse_usage_without_guessing_a_driver() {
        let mut other = observation(1);
        other.usage = Some(HidUsage { page: 1, usage: 2 });
        let result = IdentityAllocator::new()
            .unwrap()
            .snapshot(vec![observation(1), other])
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].usage, None);
    }
}
