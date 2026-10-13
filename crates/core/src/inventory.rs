use std::collections::{BTreeMap, BTreeSet};

use peripheral_hub_device_registry::{DriverRegistry, RegistryError};
use peripheral_hub_device_sdk::{
    DeviceDescriptor, DeviceError, DeviceId, DeviceInstanceId, DeviceInterfaceId,
};

/// Core errors retain their typed source; an invalid snapshot is not committed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoreError {
    /// Descriptor or SDK validation failed.
    #[error(transparent)]
    Device(#[from] DeviceError),
    /// Registry refused the available matching evidence.
    #[error(transparent)]
    Registry(#[from] RegistryError),
    /// Two descriptors use the same endpoint identity or locator.
    #[error("duplicate endpoint in inventory snapshot")]
    DuplicateInterface,
    /// One physical instance has inconsistent VID/PID observations.
    #[error("inconsistent physical instance identity")]
    InconsistentInstance,
}

impl CoreError {
    /// Static category suitable for logs without private descriptor/error details.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Device(error) => error.code(),
            Self::Registry(error) => error.code(),
            Self::DuplicateInterface => "duplicate_interface",
            Self::InconsistentInstance => "inconsistent_instance",
        }
    }
}

/// Identification only: neither case means a driver connection is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identification {
    /// Unique registered model selected from observed evidence.
    Compatible(DeviceId),
    /// No registered model matches; descriptor is still visible.
    Unknown,
}

/// One observed interface and its matching result; descriptor Debug is redacted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceRecord {
    /// Local observed hardware identity and endpoint.
    pub descriptor: DeviceDescriptor,
    /// Conservative matching result.
    pub identification: Identification,
}

/// Changes between supplied snapshots, not an OS hotplug event stream.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InventoryChange {
    /// Interfaces not in the previous snapshot.
    pub added: usize,
    /// Interfaces absent from the new complete snapshot.
    pub removed: usize,
    /// Existing interfaces whose descriptor/identification changed.
    pub updated: usize,
}

/// In-memory inventory with atomic replacement and an immutable explicit registry.
pub struct PeripheralCore {
    registry: DriverRegistry,
    records: BTreeMap<DeviceInterfaceId, DeviceRecord>,
}

impl PeripheralCore {
    /// Own a caller-built registry; no built-in support is assumed.
    pub fn new(registry: DriverRegistry) -> Self {
        Self {
            registry,
            records: BTreeMap::new(),
        }
    }

    /// Inspect the registry without replacing rules underneath a snapshot.
    pub fn registry(&self) -> &DriverRegistry {
        &self.registry
    }

    /// Inspect records in endpoint-token order; callers must sanitize exports.
    pub fn devices(&self) -> impl Iterator<Item = &DeviceRecord> {
        self.records.values()
    }

    /// Obtain one endpoint's current identification.
    pub fn get(&self, interface_id: &DeviceInterfaceId) -> Option<&DeviceRecord> {
        self.records.get(interface_id)
    }

    /// Inspect all interfaces belonging to one observed physical unit.
    pub fn interfaces_for<'a>(
        &'a self,
        instance: &'a DeviceInstanceId,
    ) -> impl Iterator<Item = &'a DeviceRecord> {
        self.records
            .values()
            .filter(move |r| &r.descriptor.instance_id == instance)
    }

    /// Validate and atomically replace a complete externally supplied snapshot.
    /// Failed input/matching preserves the entire previous inventory.
    pub fn replace_inventory(
        &mut self,
        descriptors: Vec<DeviceDescriptor>,
    ) -> Result<InventoryChange, CoreError> {
        let result = self.prepare_inventory(descriptors);
        let next = match result {
            Ok(next) => next,
            Err(error) => {
                tracing::warn!(event = "inventory_rejected", error_code = error.code());
                return Err(error);
            }
        };
        let mut change = InventoryChange::default();
        for (id, record) in &next {
            match self.records.get(id) {
                None => change.added += 1,
                Some(previous) if previous != record => change.updated += 1,
                Some(_) => {}
            }
        }
        change.removed = self
            .records
            .keys()
            .filter(|id| !next.contains_key(*id))
            .count();
        self.records = next;
        let compatible = self
            .records
            .values()
            .filter(|r| matches!(r.identification, Identification::Compatible(_)))
            .count();
        tracing::info!(
            event = "inventory_replaced",
            added = change.added,
            removed = change.removed,
            updated = change.updated,
            interfaces = self.records.len(),
            compatible,
            unknown = self.records.len() - compatible
        );
        Ok(change)
    }

    fn prepare_inventory(
        &self,
        descriptors: Vec<DeviceDescriptor>,
    ) -> Result<BTreeMap<DeviceInterfaceId, DeviceRecord>, CoreError> {
        let mut next = BTreeMap::new();
        let mut locators = BTreeSet::new();
        let mut units: BTreeMap<DeviceInstanceId, (u16, u16)> = BTreeMap::new();
        for descriptor in descriptors {
            descriptor.validate()?;
            if next.contains_key(&descriptor.interface_id)
                || !locators.insert((descriptor.transport as u8, descriptor.path.clone()))
            {
                return Err(CoreError::DuplicateInterface);
            }
            let identity = (descriptor.vendor_id, descriptor.product_id);
            if units
                .insert(descriptor.instance_id.clone(), identity)
                .is_some_and(|previous| previous != identity)
            {
                return Err(CoreError::InconsistentInstance);
            }
            let identification = self
                .registry
                .resolve(&descriptor)?
                .map_or(Identification::Unknown, |metadata| {
                    Identification::Compatible(metadata.model_id.clone())
                });
            next.insert(
                descriptor.interface_id.clone(),
                DeviceRecord {
                    descriptor,
                    identification,
                },
            );
        }
        Ok(next)
    }
}
