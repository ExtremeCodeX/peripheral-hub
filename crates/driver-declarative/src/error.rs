use peripheral_hub_device_sdk::DeviceError;

/// Structured manifest failures; no manifest content is echoed in messages.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    /// The declared manifest schema version is newer than this interpreter.
    #[error("manifest version {found} is not supported (expected {supported})")]
    UnsupportedVersion {
        /// Version declared in the manifest.
        found: u16,
        /// Version implemented by this interpreter.
        supported: u16,
    },
    /// A semantic constraint on the manifest was violated.
    #[error("manifest is invalid: {0}")]
    Invalid(&'static str),
    /// TOML did not parse into the typed manifest model.
    #[error("manifest TOML is invalid")]
    Toml(#[from] toml::de::Error),
    /// The manifest file could not be read.
    #[error("manifest file could not be read")]
    Io(#[from] std::io::Error),
    /// An embedded SDK contract (identity, match rule or capability) failed validation.
    #[error(transparent)]
    Device(#[from] DeviceError),
}
