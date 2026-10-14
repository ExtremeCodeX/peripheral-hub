use super::{Endpoint, LocalStream};
use interprocess::os::windows::{
    named_pipe::{
        PipeListenerOptions, pipe_mode,
        tokio::{DuplexPipeStream, PipeListener},
    },
    security_descriptor::SecurityDescriptor,
};
use std::io;
use std::path::PathBuf;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use widestring::U16CString;

type Pipe = DuplexPipeStream<pipe_mode::Bytes>;
fn denied() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "named-pipe user identity rejected",
    )
}

fn process_sid(pid: u32) -> io::Result<String> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        true,
        ProcessRefreshKind::nothing().with_user(UpdateKind::Always),
    );
    let sid = system
        .process(Pid::from_u32(pid))
        .and_then(|process| process.user_id())
        .ok_or_else(denied)?
        .to_string();
    if !sid.starts_with("S-1-")
        || !sid
            .bytes()
            .all(|b| b.is_ascii_digit() || b == b'-' || b == b'S')
    {
        return Err(denied());
    }
    Ok(sid)
}

pub(super) fn default_endpoint() -> io::Result<PathBuf> {
    Ok(PathBuf::from(format!(
        r"\\.\pipe\peripheral-hub-v1-{}",
        process_sid(std::process::id())?
    )))
}

fn validate_name(endpoint: &Endpoint, sid: &str) -> io::Result<()> {
    let name = endpoint.path().to_str().ok_or_else(denied)?;
    if !name.starts_with(r"\\.\pipe\peripheral-hub-")
        || !name.ends_with(&format!("-{sid}"))
        || name.len() > 240
    {
        return Err(denied());
    }
    Ok(())
}

/// Local-only Windows pipe with a protected current-user SID DACL.
pub struct Listener {
    listener: PipeListener<pipe_mode::Bytes, pipe_mode::Bytes>,
    sid: String,
}
impl Listener {
    /// Create first instance atomically; no inherited broad/default DACL is used.
    pub async fn bind(endpoint: &Endpoint) -> io::Result<Self> {
        let sid = process_sid(std::process::id())?;
        validate_name(endpoint, &sid)?;
        let sddl =
            U16CString::from_str(format!("O:{sid}D:P(A;;GA;;;{sid})")).map_err(|_| denied())?;
        let descriptor = SecurityDescriptor::deserialize(&sddl)?;
        let listener = PipeListenerOptions::new()
            .path(endpoint.path().as_os_str())
            .accept_remote(false)
            .inheritable(false)
            .security_descriptor(Some(descriptor))
            .create_tokio_duplex::<pipe_mode::Bytes>()?;
        Ok(Self { listener, sid })
    }
    /// Require actual peer process ownership in addition to the pipe DACL.
    pub async fn accept(&self) -> io::Result<Box<dyn LocalStream>> {
        let stream = self.listener.accept().await?;
        if process_sid(stream.peer_process_id()?)? != self.sid {
            return Err(denied());
        }
        Ok(Box::new(stream))
    }
}
pub(super) async fn connect(endpoint: &Endpoint) -> io::Result<Box<dyn LocalStream>> {
    let sid = process_sid(std::process::id())?;
    validate_name(endpoint, &sid)?;
    let stream = Pipe::connect_by_path(endpoint.path().as_os_str()).await?;
    if process_sid(stream.peer_process_id()?)? != sid {
        return Err(denied());
    }
    Ok(Box::new(stream))
}
