use serde::{Serialize, de::DeserializeOwned};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Hard bound checked before allocating incoming JSON frames.
pub const MAX_FRAME_BYTES: usize = 256 * 1024;

/// Local codec/transport errors (not device-operation success).
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    /// OS transport failed, including disconnected/truncated streams.
    #[error("local IPC transport failed: {0}")]
    Io(#[from] std::io::Error),
    /// Frame is empty or exceeds the allocation bound.
    #[error("IPC frame length is invalid")]
    InvalidLength,
    /// JSON or typed contracts do not parse.
    #[error("malformed IPC message")]
    InvalidMessage,
    /// Caller deadline expired; mutation outcome may be unknown.
    #[error("IPC deadline exceeded")]
    Timeout,
    /// Version/correlation does not match the outstanding request.
    #[error("unexpected IPC response")]
    UnexpectedResponse,
}

/// Read a 4-byte big-endian length then bounded typed UTF-8 JSON.
pub async fn read_frame<T: DeserializeOwned>(
    stream: &mut (impl AsyncRead + Unpin),
) -> Result<T, IpcError> {
    let length = stream.read_u32().await? as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(IpcError::InvalidLength);
    }
    let mut bytes = vec![0; length];
    stream.read_exact(&mut bytes).await?;
    serde_json::from_slice(&bytes).map_err(|_| IpcError::InvalidMessage)
}

/// Write one bounded frame; no truncation or partial success is accepted.
pub async fn write_frame<T: Serialize>(
    stream: &mut (impl AsyncWrite + Unpin),
    message: &T,
) -> Result<(), IpcError> {
    let bytes = serde_json::to_vec(message).map_err(|_| IpcError::InvalidMessage)?;
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err(IpcError::InvalidLength);
    }
    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    Ok(())
}
