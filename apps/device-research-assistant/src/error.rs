//! Typed errors for the Device Research Assistant.
//!
//! Messages never include payload contents, serial numbers or private device
//! locators. Callers may print the path they supplied themselves.

/// Failure returned by a DRA command or library operation.
#[derive(Debug, thiserror::Error)]
pub enum DraError {
    /// A command argument or imported field failed semantic validation.
    #[error("{0}")]
    InvalidArgument(String),
    /// The document declares a schema version this build does not understand.
    #[error("unsupported {kind} schema version {found}; expected {expected}")]
    UnsupportedVersion {
        /// Document kind (session, capture, analysis or report).
        kind: &'static str,
        /// Version read from the document.
        found: u32,
        /// Version implemented by this build.
        expected: u32,
    },
    /// A bounded resource limit was exceeded before parsing or writing.
    #[error("{kind} exceeds the limit of {limit}")]
    LimitExceeded {
        /// Human-readable resource name.
        kind: String,
        /// Maximum accepted value.
        limit: usize,
    },
    /// The requested session directory does not exist.
    #[error("session '{0}' was not found")]
    SessionNotFound(String),
    /// A session with the requested id already exists.
    #[error("session '{0}' already exists")]
    SessionExists(String),
    /// The session has no normalized capture to analyze.
    #[error("session '{0}' has no imported capture; run capture import first")]
    NoCapture(String),
    /// The session has no analysis or it is not usable for a scaffold.
    #[error("session '{0}' has no usable analysis; run analyze first")]
    NoAnalysis(String),
    /// The target directory already contains files and `--force` was not set.
    #[error("output directory '{0}' is not empty; pass --force to overwrite")]
    OutputNotEmpty(String),
    /// Session creation requires a recorded consent flag.
    #[error("creating a research session requires --consent")]
    ConsentRequired,
    /// HID enumeration failed; only the category code is exposed.
    #[error("HID inventory failed ({0}); no device was opened or written")]
    Inventory(String),
    /// A filesystem operation failed.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// A JSON document could not be parsed or serialized.
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type used by DRA commands and tests.
pub type DraResult<T> = Result<T, DraError>;
