//! Typed failures for the agent handoff tool.

/// Failure returned by an agent handoff command.
#[derive(Debug, thiserror::Error)]
pub enum HandoffError {
    /// A command argument or generated field failed validation.
    #[error("{0}")]
    InvalidArgument(String),
    /// A bounded input or output limit was exceeded.
    #[error("{kind} exceeds the limit of {limit}")]
    LimitExceeded {
        /// Human-readable resource name.
        kind: String,
        /// Maximum accepted value.
        limit: usize,
    },
    /// A DRA report did not pass typed or cross-document validation.
    #[error("DRA report is invalid: {0}")]
    InvalidReport(String),
    /// A declarative manifest did not pass the real typed/semantic validator.
    #[error("declarative manifest is invalid: {0}")]
    InvalidManifest(String),
    /// The report does not contain enough evidence for a driver handoff.
    #[error("insufficient evidence: {0}")]
    InsufficientEvidence(String),
    /// The output directory contains files and `--force` was not supplied.
    #[error("output directory '{0}' is not empty; pass --force to overwrite known files")]
    OutputNotEmpty(String),
    /// A handoff directory or provenance digest is inconsistent.
    #[error("handoff verification failed: {0}")]
    Verification(String),
    /// A filesystem operation failed.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// A JSON document could not be parsed or serialized.
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type used by agent handoff commands and tests.
pub type HandoffResult<T> = Result<T, HandoffError>;
