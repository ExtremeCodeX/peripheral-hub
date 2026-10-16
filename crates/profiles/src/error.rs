use peripheral_hub_storage::{StorageError, rusqlite};

/// Structured profile failures; wire layers project these onto safe codes.
#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    /// A field failed contract validation. The message is static, not user data.
    #[error("profile is invalid: {0}")]
    Invalid(&'static str),
    /// The stored or imported schema version is not understood.
    #[error("unsupported profile schema version")]
    UnsupportedVersion,
    /// No profile with the requested id exists.
    #[error("profile not found")]
    NotFound,
    /// Imported JSON could not be parsed into the profile contract.
    #[error("profile JSON is malformed")]
    InvalidJson,
    /// Underlying persistence failed.
    #[error(transparent)]
    Storage(#[from] StorageError),
}

impl From<rusqlite::Error> for ProfileError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(StorageError::from(error))
    }
}
