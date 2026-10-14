use crate::{IPC_VERSION, IpcError, Operation, Request, Response, read_frame, write_frame};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};

#[cfg(unix)]
#[path = "unix.rs"]
mod platform;
#[cfg(windows)]
#[path = "windows.rs"]
mod platform;

/// Async local byte stream; deliberately no network endpoint support.
pub trait LocalStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> LocalStream for T {}
/// Platform-specific protected listener.
pub use platform::Listener;

/// Local endpoint: Unix socket path or local Windows pipe name.
#[derive(Debug, Clone)]
pub struct Endpoint(PathBuf);
impl Endpoint {
    /// Per-user default location. Does not create or connect a server.
    pub fn default_for_user() -> std::io::Result<Self> {
        platform::default_endpoint().map(Self)
    }
    /// Select a custom local endpoint; authorization is still checked at bind/connect.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }
    /// Inspect endpoint path locally; do not put it into public diagnostic logs.
    pub fn path(&self) -> &Path {
        &self.0
    }
}

/// Deadline-bounded typed client. A fresh connection per call permits reconnects.
/// Mutations are never automatically retried after a timeout or disconnection.
pub struct Client {
    endpoint: Endpoint,
    next_id: u64,
    timeout: Duration,
}
impl Client {
    /// Configure client deadlines between 1 ms and 30 seconds.
    pub fn new(endpoint: Endpoint, timeout: Duration) -> Result<Self, IpcError> {
        if timeout < Duration::from_millis(1) || timeout > Duration::from_secs(30) {
            return Err(IpcError::InvalidMessage);
        }
        Ok(Self {
            endpoint,
            next_id: 0,
            timeout,
        })
    }
    /// Send a typed request, preserving remote Fault instead of treating it as success.
    pub async fn call(&mut self, operation: Operation) -> Result<Response, IpcError> {
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(IpcError::UnexpectedResponse)?;
        let request = Request {
            version: IPC_VERSION,
            request_id: self.next_id,
            operation,
        };
        let result = tokio::time::timeout(self.timeout, async {
            let mut stream = platform::connect(&self.endpoint).await?;
            write_frame(&mut stream, &request).await?;
            let response: Response = read_frame(&mut stream).await?;
            if response.version != IPC_VERSION || response.request_id != request.request_id {
                return Err(IpcError::UnexpectedResponse);
            }
            Ok(response)
        })
        .await;
        result.map_err(|_| IpcError::Timeout)?
    }
}
