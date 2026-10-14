use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use super::{Endpoint, LocalStream};
use nix::fcntl::{Flock, FlockArg, OFlag};
use nix::unistd::Uid;
use tokio::net::{UnixListener, UnixStream};

fn denied() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "local endpoint ownership or permissions rejected",
    )
}
fn validate_directory(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.uid() != Uid::effective().as_raw()
        || metadata.mode() & 0o777 != 0o700
    {
        return Err(denied());
    }
    Ok(())
}
fn validate_socket(path: &Path) -> io::Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_socket()
        || metadata.uid() != Uid::effective().as_raw()
        || metadata.mode() & 0o777 != 0o600
    {
        return Err(denied());
    }
    Ok(metadata)
}

pub(super) fn default_endpoint() -> io::Result<PathBuf> {
    let base = if let Some(path) = std::env::var_os("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(path);
        validate_directory(&path)?;
        path
    } else {
        std::env::temp_dir()
    };
    Ok(base
        .join(format!("peripheral-hub-{}", Uid::effective()))
        .join("daemon.sock"))
}

/// Unix listener protected by private directory, permissions, credentials and flock.
pub struct Listener {
    listener: UnixListener,
    path: PathBuf,
    identity: (u64, u64),
    _lock: Flock<File>,
}
impl Listener {
    /// Bind once per endpoint. Recover only verified stale sockets while holding flock.
    pub async fn bind(endpoint: &Endpoint) -> io::Result<Self> {
        let parent = endpoint
            .path()
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .ok_or_else(denied)?;
        match fs::DirBuilder::new().mode(0o700).create(parent) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        validate_directory(parent)?;
        let name = endpoint
            .path()
            .file_name()
            .ok_or_else(denied)?
            .to_string_lossy();
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(OFlag::O_NOFOLLOW.bits())
            .open(parent.join(format!("{name}.lock")))?;
        let metadata = lock.metadata()?;
        if !metadata.is_file()
            || metadata.uid() != Uid::effective().as_raw()
            || metadata.mode() & 0o777 != 0o600
            || metadata.nlink() != 1
        {
            return Err(denied());
        }
        let lock = Flock::lock(lock, FlockArg::LockExclusiveNonblock)
            .map_err(|(_, _)| io::Error::new(io::ErrorKind::AddrInUse, "daemon already running"))?;
        if fs::symlink_metadata(endpoint.path()).is_ok() {
            validate_socket(endpoint.path())?;
            match UnixStream::connect(endpoint.path()).await {
                Ok(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::AddrInUse,
                        "daemon already running",
                    ));
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::ConnectionRefused | io::ErrorKind::NotFound
                    ) =>
                {
                    fs::remove_file(endpoint.path())?
                }
                Err(error) => return Err(error),
            }
        }
        let listener = UnixListener::bind(endpoint.path())?;
        fs::set_permissions(endpoint.path(), fs::Permissions::from_mode(0o600))?;
        let metadata = validate_socket(endpoint.path())?;
        Ok(Self {
            listener,
            path: endpoint.path().to_owned(),
            identity: (metadata.dev(), metadata.ino()),
            _lock: lock,
        })
    }
    /// Accept only same-effective-UID peers before reading a request.
    pub async fn accept(&self) -> io::Result<Box<dyn LocalStream>> {
        let (stream, _) = self.listener.accept().await?;
        if stream.peer_cred()?.uid() != Uid::effective().as_raw() {
            return Err(denied());
        }
        Ok(Box::new(stream))
    }
}
impl Drop for Listener {
    fn drop(&mut self) {
        if let Ok(metadata) = validate_socket(&self.path)
            && (metadata.dev(), metadata.ino()) == self.identity
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(super) async fn connect(endpoint: &Endpoint) -> io::Result<Box<dyn LocalStream>> {
    validate_directory(endpoint.path().parent().ok_or_else(denied)?)?;
    validate_socket(endpoint.path())?;
    let stream = UnixStream::connect(endpoint.path()).await?;
    if stream.peer_cred()?.uid() != Uid::effective().as_raw() {
        return Err(denied());
    }
    Ok(Box::new(stream))
}
