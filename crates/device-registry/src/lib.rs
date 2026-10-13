//! Explicit, reviewed driver factories and conservative interface matching.
//!
//! No dynamic loading, physical probing, or registry population occurs here.

use std::collections::BTreeMap;

use peripheral_hub_device_sdk::{
    DeviceDescriptor, DeviceDriver, DeviceError, DeviceId, DriverFactory, DriverMetadata,
};

#[cfg(test)]
mod tests;

/// Registry failures are distinct from device-operation failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegistryError {
    /// A model is already registered; existing factories remain unchanged.
    #[error("model already registered: {0}")]
    DuplicateModel(DeviceId),
    /// Multiple factories satisfy the evidence; no winner is guessed.
    #[error("ambiguous driver match")]
    AmbiguousMatch(Vec<DeviceId>),
    /// No registered model supports this interface.
    #[error("no matching driver")]
    NoMatch,
    /// Metadata, descriptor, or factory contract failure.
    #[error(transparent)]
    Device(#[from] DeviceError),
}

impl RegistryError {
    /// Privacy-safe structured logging category.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DuplicateModel(_) => "duplicate_model",
            Self::AmbiguousMatch(_) => "ambiguous_match",
            Self::NoMatch => "no_match",
            Self::Device(error) => error.code(),
        }
    }
}

struct Registration {
    metadata: DriverMetadata,
    factory: Box<dyn DriverFactory>,
}

/// Explicit factory registry; native/declarative producers use the same contract.
#[derive(Default)]
pub struct DriverRegistry {
    entries: BTreeMap<DeviceId, Registration>,
}

impl DriverRegistry {
    /// Create an empty registry, with no presumed device support.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register valid metadata once; validation occurs before any mutation.
    pub fn register(&mut self, factory: Box<dyn DriverFactory>) -> Result<(), RegistryError> {
        let metadata = factory.metadata().clone();
        metadata.validate()?;
        if self.entries.contains_key(&metadata.model_id) {
            return Err(RegistryError::DuplicateModel(metadata.model_id));
        }
        tracing::info!(event = "driver_registered", model = %metadata.model_id,
            sdk_major = metadata.sdk_version.major, sdk_minor = metadata.sdk_version.minor);
        self.entries.insert(
            metadata.model_id.clone(),
            Registration { metadata, factory },
        );
        Ok(())
    }

    /// Number of explicitly registered models.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no model is registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Inspect metadata in stable model-ID order; never expose factory internals.
    pub fn models(&self) -> impl Iterator<Item = &DriverMetadata> {
        self.entries.values().map(|entry| &entry.metadata)
    }

    /// Resolve without opening hardware; multiple rules in one model count once.
    pub fn resolve(
        &self,
        descriptor: &DeviceDescriptor,
    ) -> Result<Option<&DriverMetadata>, RegistryError> {
        descriptor.validate()?;
        let mut candidates = Vec::new();
        for entry in self.entries.values() {
            for rule in &entry.metadata.match_rules {
                if rule.matches(descriptor)? {
                    candidates.push(&entry.metadata);
                    break;
                }
            }
        }
        if candidates.len() > 1 {
            tracing::warn!(
                event = "driver_match_rejected",
                error_code = "ambiguous_match",
                candidates = candidates.len()
            );
            return Err(RegistryError::AmbiguousMatch(
                candidates.iter().map(|m| m.model_id.clone()).collect(),
            ));
        }
        Ok(candidates.first().copied())
    }

    /// Construct a uniquely selected driver, never a connection or hardware probe.
    pub fn create_driver(
        &self,
        descriptor: &DeviceDescriptor,
    ) -> Result<Box<dyn DeviceDriver>, RegistryError> {
        let metadata = self.resolve(descriptor)?.ok_or(RegistryError::NoMatch)?;
        // The selected ID belongs to this immutable registry, so indexing is safe.
        let entry = &self.entries[&metadata.model_id];
        let result = entry.factory.create(descriptor);
        if let Err(error) = &result {
            tracing::warn!(event = "driver_creation_failed", error_code = error.code());
        }
        result.map_err(RegistryError::from)
    }
}
