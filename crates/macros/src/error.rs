use peripheral_hub_storage::{StorageError, rusqlite};

/// Structured macro failures; wire layers project these onto safe codes.
#[derive(Debug, thiserror::Error)]
pub enum MacroError {
    /// A field failed contract validation. The message is static, not user data.
    #[error("macro is invalid: {0}")]
    Invalid(&'static str),
    /// The stored or imported schema version is not understood.
    #[error("unsupported macro schema version")]
    UnsupportedVersion,
    /// No macro with the requested id exists.
    #[error("macro not found")]
    NotFound,
    /// Imported JSON could not be parsed into the macro contract.
    #[error("macro JSON is malformed")]
    InvalidJson,
    /// Underlying persistence failed.
    #[error(transparent)]
    Storage(#[from] StorageError),
}

impl From<rusqlite::Error> for MacroError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(StorageError::from(error))
    }
}
