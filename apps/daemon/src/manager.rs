use peripheral_hub_core::{Identification, PeripheralCore};
use peripheral_hub_device_registry::{DriverRegistry, RegistryError};
use peripheral_hub_device_sdk::*;
use peripheral_hub_ipc::*;
use peripheral_hub_profiles::ProfileRepository;
use peripheral_hub_transport_hid::DeviceSource;
use std::collections::{BTreeMap, VecDeque};

struct Session {
    driver: Option<Box<dyn DeviceDriver>>,
    status: ConnectionStatus,
    error: Option<DeviceError>,
}

/// User-session device lifecycle over unchanged Phase 1 Core/Registry contracts.
pub struct DeviceManager {
    session_id: String,
    core: PeripheralCore,
    source: Box<dyn DeviceSource>,
    sessions: BTreeMap<DeviceInterfaceId, Session>,
    events: VecDeque<ConnectionEvent>,
    sequence: u64,
    discovery_error: Option<DeviceError>,
    profiles: ProfileRepository,
}
impl DeviceManager {
    /// Own an explicit registry/backend; physical mode never registers simulation.
    pub fn new(registry: DriverRegistry, source: Box<dyn DeviceSource>) -> Self {
        Self::try_new(registry, source).expect("daemon initialization failed")
    }
    /// Fallible initialization with an ephemeral in-memory profile store.
    pub fn try_new(registry: DriverRegistry, source: Box<dyn DeviceSource>) -> DeviceResult<Self> {
        let profiles = ProfileRepository::open_in_memory().map_err(|_| {
            DeviceError::ProtocolError("profile storage initialization failed".into())
        })?;
        Self::with_profiles(registry, source, profiles)
    }
    /// Initialization with an explicit profile repository (for example file-backed).
    pub fn with_profiles(
        registry: DriverRegistry,
        source: Box<dyn DeviceSource>,
        profiles: ProfileRepository,
    ) -> DeviceResult<Self> {
        Ok(Self {
            session_id: peripheral_hub_transport_hid::session_epoch()?,
            core: PeripheralCore::new(registry),
            source,
            sessions: BTreeMap::new(),
            events: VecDeque::new(),
            sequence: 0,
            discovery_error: None,
            profiles,
        })
    }
    /// Actual source mode, never an automatic fallback from failed hardware discovery.
    pub fn mode(&self) -> ExecutionMode {
        self.source.mode()
    }

    /// Poll enumeration and reconcile handles. Failed snapshots retain last-known state.
    pub fn refresh(&mut self) -> DeviceResult<()> {
        let result = self.refresh_inner();
        self.discovery_error = result.as_ref().err().cloned().map(sanitize_error);
        if let Err(error) = &result {
            tracing::warn!(event = "discovery_failed", error_code = error.code());
        }
        result
    }
    fn refresh_inner(&mut self) -> DeviceResult<()> {
        let descriptors = self.source.scan()?;
        if descriptors.len() > MAX_INTERFACES {
            return Err(DeviceError::InvalidArgument(
                "inventory interface budget exceeded".into(),
            ));
        }
        let previous: BTreeMap<_, _> = self
            .core
            .devices()
            .map(|record| {
                (
                    record.descriptor.interface_id.clone(),
                    record.descriptor.clone(),
                )
            })
            .collect();
        self.core.replace_inventory(descriptors).map_err(|_| {
            DeviceError::ProtocolError("inventory validation rejected snapshot".into())
        })?;
        let current: BTreeMap<_, _> = self
            .core
            .devices()
            .map(|record| {
                (
                    record.descriptor.interface_id.clone(),
                    record.descriptor.clone(),
                )
            })
            .collect();
        for (id, descriptor) in &previous {
            if current.get(id) != Some(descriptor) {
                if let Some(mut session) = self.sessions.remove(id)
                    && let Some(mut driver) = session.driver.take()
                    && let Err(error) = driver.close()
                {
                    tracing::warn!(event = "device_cleanup_failed", error_code = error.code());
                }
                self.emit(id.clone(), ConnectionStatus::Disconnected);
            }
        }
        for id in current.keys() {
            if !self.sessions.contains_key(id) {
                self.sessions.insert(
                    id.clone(),
                    Session {
                        driver: None,
                        status: ConnectionStatus::Discovered,
                        error: None,
                    },
                );
                self.emit(id.clone(), ConnectionStatus::Discovered);
            }
        }
        Ok(())
    }
    fn emit(&mut self, id: DeviceInterfaceId, status: ConnectionStatus) {
        self.sequence = self.sequence.saturating_add(1);
        if self.events.len() == 128 {
            self.events.pop_front();
        }
        self.events.push_back(ConnectionEvent {
            sequence: self.sequence,
            interface_id: id,
            status,
            mode: self.mode(),
        });
    }
    fn summary(&self, id: &DeviceInterfaceId) -> Result<DeviceSummary, Fault> {
        let record = self.core.get(id).ok_or(DeviceError::NotConnected)?;
        let session = self.sessions.get(id).ok_or(Fault::Internal)?;
        Ok(DeviceSummary {
            instance_id: record.descriptor.instance_id.clone(),
            interface_id: id.clone(),
            vendor_id: record.descriptor.vendor_id,
            product_id: record.descriptor.product_id,
            manufacturer: record.descriptor.manufacturer.clone(),
            product: record.descriptor.product.clone(),
            interface_number: record.descriptor.interface_number,
            usage: record.descriptor.usage,
            model: match &record.identification {
                Identification::Compatible(model) => Some(model.clone()),
                Identification::Unknown => None,
            },
            mode: self.mode(),
            status: session.status,
            last_error: session.error.clone(),
        })
    }
    fn driver(&mut self, id: &DeviceInterfaceId) -> Result<&mut Box<dyn DeviceDriver>, Fault> {
        if let Some(error) = &self.discovery_error {
            return Err(error.clone().into());
        }
        let record = self.core.get(id).ok_or(DeviceError::NotConnected)?;
        if record.identification == Identification::Unknown {
            return Err(DeviceError::NotSupported.into());
        }
        self.sessions
            .get_mut(id)
            .and_then(|session| session.driver.as_mut())
            .ok_or_else(|| DeviceError::NotConnected.into())
    }
    fn failed(&mut self, id: &DeviceInterfaceId, error: DeviceError) -> Fault {
        let error = sanitize_error(error);
        if let Some(session) = self.sessions.get_mut(id) {
            if let Some(mut driver) = session.driver.take() {
                let _ = driver.close();
            }
            session.status = ConnectionStatus::Failed;
            session.error = Some(error.clone());
            self.emit(id.clone(), ConnectionStatus::Failed);
        }
        Fault::Device(error)
    }
    fn connect(&mut self, id: &DeviceInterfaceId) -> Result<Reply, Fault> {
        if let Some(error) = &self.discovery_error {
            return Err(error.clone().into());
        }
        let summary = self.summary(id)?;
        if summary.model.is_none() {
            return Err(DeviceError::NotSupported.into());
        }
        if summary.status == ConnectionStatus::Connected {
            return Ok(Reply::Device(summary));
        }
        let descriptor = self
            .core
            .get(id)
            .ok_or(DeviceError::NotConnected)?
            .descriptor
            .clone();
        let result =
            (|| -> DeviceResult<Box<dyn DeviceDriver>> {
                let mut driver = self.core.registry().create_driver(&descriptor).map_err(
                    |error| match error {
                        RegistryError::Device(error) => error,
                        _ => DeviceError::ProtocolError("driver selection rejected".into()),
                    },
                )?;
                let transport = self.source.open(&descriptor)?;
                if let Err(error) = driver.open(transport) {
                    let _ = driver.close();
                    return Err(error);
                }
                match driver
                    .capabilities()
                    .and_then(|capabilities| capabilities.validate())
                {
                    Ok(()) => Ok(driver),
                    Err(error) => {
                        let _ = driver.close();
                        Err(error)
                    }
                }
            })();
        match result {
            Ok(driver) => {
                let session = self.sessions.get_mut(id).ok_or(Fault::Internal)?;
                session.driver = Some(driver);
                session.status = ConnectionStatus::Connected;
                session.error = None;
                self.emit(id.clone(), ConnectionStatus::Connected);
                Ok(Reply::Device(self.summary(id)?))
            }
            Err(error) => Err(self.failed(id, error)),
        }
    }
    /// Handle a typed request on the serialized manager worker.
    pub fn handle(&mut self, operation: Operation) -> Result<Reply, Fault> {
        match operation {
            Operation::Health => Ok(Reply::Health(Health {
                session_id: self.session_id.clone(),
                daemon_version: env!("CARGO_PKG_VERSION").into(),
                sdk_version: SDK_CONTRACT_VERSION,
                mode: self.mode(),
                discovery_error: self.discovery_error.clone(),
                sequence: self.sequence,
            })),
            Operation::ListDevices => {
                if let Some(error) = &self.discovery_error {
                    return Err(error.clone().into());
                }
                Ok(Reply::Devices(
                    self.core
                        .devices()
                        .map(|record| self.summary(&record.descriptor.interface_id))
                        .collect::<Result<_, _>>()?,
                ))
            }
            Operation::GetDevice { interface_id } => {
                Ok(Reply::Device(self.summary(&interface_id)?))
            }
            Operation::Connect { interface_id } => self.connect(&interface_id),
            Operation::Disconnect { interface_id } => {
                self.summary(&interface_id)?;
                if let Some(mut driver) = self
                    .sessions
                    .get_mut(&interface_id)
                    .ok_or(Fault::Internal)?
                    .driver
                    .take()
                    && let Err(error) = driver.close()
                {
                    return Err(self.failed(&interface_id, error));
                }
                self.sessions
                    .get_mut(&interface_id)
                    .ok_or(Fault::Internal)?
                    .status = ConnectionStatus::Disconnected;
                self.emit(interface_id, ConnectionStatus::Disconnected);
                Ok(Reply::Ack)
            }
            Operation::GetCapabilities { interface_id } => {
                let capabilities = self.driver(&interface_id)?.capabilities()?;
                capabilities.validate()?;
                Ok(Reply::Capabilities(capabilities))
            }
            Operation::ReadState { interface_id } => {
                let result = self.driver(&interface_id)?.read_state();
                match result {
                    Ok(state) if state.mode == self.mode() => Ok(Reply::State(state)),
                    Ok(_) => Err(self.failed(
                        &interface_id,
                        DeviceError::ProtocolError("driver mode mismatch".into()),
                    )),
                    Err(error) => self.operation_error(&interface_id, error),
                }
            }
            Operation::SetLighting {
                interface_id,
                command,
            } => self.execute_lighting(&interface_id, command),
            Operation::Events { after } => {
                if after > self.sequence {
                    return Err(Fault::InvalidArgument);
                }
                let gap = self
                    .events
                    .front()
                    .is_some_and(|event| after.saturating_add(1) < event.sequence);
                Ok(Reply::Events(EventBatch {
                    sequence: self.sequence,
                    gap,
                    events: self
                        .events
                        .iter()
                        .filter(|event| event.sequence > after)
                        .cloned()
                        .collect(),
                }))
            }
            Operation::SimulatePresence { present } => {
                self.source.set_simulated_presence(present)?;
                self.refresh()?;
                Ok(Reply::Ack)
            }
            Operation::ListProfiles => Ok(Reply::Profiles(self.profiles.list()?)),
            Operation::GetProfile { profile_id } => {
                Ok(Reply::Profile(self.profiles.get(&profile_id)?))
            }
            Operation::SaveProfile { profile } => {
                self.profiles.save(&profile)?;
                Ok(Reply::Profile(profile))
            }
            Operation::DeleteProfile { profile_id } => {
                self.profiles.delete(&profile_id)?;
                Ok(Reply::Ack)
            }
            Operation::ApplyProfile {
                interface_id,
                profile_id,
            } => {
                let profile = self.profiles.get(&profile_id)?;
                let summary = self.summary(&interface_id)?;
                if let Some(bound) = &profile.device_model
                    && summary.model.as_ref() != Some(bound)
                {
                    return Err(DeviceError::NotSupported.into());
                }
                let Some(command) = profile.lighting else {
                    return Err(DeviceError::NotSupported.into());
                };
                self.execute_lighting(&interface_id, command)
            }
        }
    }
    fn execute_lighting(
        &mut self,
        id: &DeviceInterfaceId,
        command: LightingCommand,
    ) -> Result<Reply, Fault> {
        let request = DeviceCommand::SetLighting(command);
        let driver = self.driver(id)?;
        driver.capabilities()?.validate_command(&request)?;
        let result = driver.execute(&request);
        match result {
            Ok(outcome) if outcome.mode == self.mode() => Ok(Reply::Outcome(outcome)),
            Ok(_) => Err(self.failed(
                id,
                DeviceError::ProtocolError("driver mode mismatch".into()),
            )),
            Err(error) => self.operation_error(id, error),
        }
    }
    fn operation_error(
        &mut self,
        id: &DeviceInterfaceId,
        error: DeviceError,
    ) -> Result<Reply, Fault> {
        if matches!(
            error,
            DeviceError::NotSupported | DeviceError::InvalidArgument(_)
        ) {
            Err(error.into())
        } else {
            Err(self.failed(id, error))
        }
    }
    /// Close every managed driver on shutdown, retaining the first real cleanup error.
    pub fn shutdown(&mut self) -> DeviceResult<()> {
        let mut first_error = None;
        for session in self.sessions.values_mut() {
            if let Some(mut driver) = session.driver.take()
                && let Err(error) = driver.close()
            {
                first_error.get_or_insert(error);
            }
            session.status = ConnectionStatus::Disconnected;
        }
        first_error.map_or(Ok(()), Err)
    }
}

/// Backend startup failure that preserves physical mode and reports the real error.
pub struct UnavailableSource {
    error: DeviceError,
}
impl UnavailableSource {
    /// Represent a failed HID initialization, never a fake empty inventory.
    pub fn new(error: DeviceError) -> Self {
        Self {
            error: sanitize_error(error),
        }
    }
}
impl DeviceSource for UnavailableSource {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Physical
    }
    fn scan(&mut self) -> DeviceResult<Vec<DeviceDescriptor>> {
        Err(self.error.clone())
    }
    fn open(&mut self, _: &DeviceDescriptor) -> DeviceResult<Box<dyn Transport>> {
        Err(self.error.clone())
    }
}
