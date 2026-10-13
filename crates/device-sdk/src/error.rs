use serde::{Deserialize, Serialize};

/// Stable, structured categories; never infer success from a missing operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(
    tag = "code",
    content = "detail",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum DeviceError {
    /// The device/driver does not implement the requested operation.
    #[error("operation not supported")]
    NotSupported,
    /// No live connection is available.
    #[error("device not connected")]
    NotConnected,
    /// The OS or device access policy denied the operation.
    #[error("permission denied")]
    PermissionDenied,
    /// A contract constraint was violated. Do not include private input here.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    /// The operation did not finish before its deadline.
    #[error("operation timed out")]
    Timeout,
    /// An I/O operation failed. Callers must sanitize backend messages.
    #[error("transport error: {0}")]
    TransportError(String),
    /// Device framing, responses, or sequencing violated the protocol.
    #[error("protocol error: {0}")]
    ProtocolError(String),
    /// The supplied SDK contract is not understood by this SDK.
    #[error("incompatible SDK contract version")]
    IncompatibleVersion,
}

impl DeviceError {
    /// Safe static category for logs; excludes backend detail and private data.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotSupported => "not_supported",
            Self::NotConnected => "not_connected",
            Self::PermissionDenied => "permission_denied",
            Self::InvalidArgument(_) => "invalid_argument",
            Self::Timeout => "timeout",
            Self::TransportError(_) => "transport_error",
            Self::ProtocolError(_) => "protocol_error",
            Self::IncompatibleVersion => "incompatible_version",
        }
    }
}

/// Fallible SDK operation result.
pub type DeviceResult<T> = Result<T, DeviceError>;

pub(crate) fn invalid(message: &str) -> DeviceError {
    DeviceError::InvalidArgument(message.to_owned())
}
