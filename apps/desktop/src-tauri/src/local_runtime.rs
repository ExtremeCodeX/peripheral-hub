//! Descriptor-based local development loader. No renderer paths or remote sources.
pub fn valid_dev_id(id: &str) -> bool {
    id.starts_with("dev.")
        && id.len() > 4
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
        && !id.contains("..")
        && !id.ends_with('.')
}

#[cfg(all(debug_assertions, target_os = "linux"))]
pub fn plugin_source(id: &str) -> Result<String, String> {
    if !valid_dev_id(id) || matches!(id, "dev.fake" | "dev.starter") {
        return Err("UNAUTHORIZED_PLUGIN".into());
    }
    load_plugin(&data_home()?, id)
}

#[cfg(all(debug_assertions, target_os = "linux"))]
fn load_plugin(base: &std::path::Path, id: &str) -> Result<String, String> {
    if !valid_dev_id(id) {
        return Err("UNAUTHORIZED_PLUGIN".into());
    }
    Directory::open(base)?
        .child("dev-plugins")?
        .child(id)?
        .read("plugin.py", "")
}

#[cfg(all(debug_assertions, not(target_os = "linux")))]
pub fn plugin_source(_id: &str) -> Result<String, String> {
    Err("LOCAL_INTEGRATION_UNAVAILABLE".into())
}

#[cfg(all(test, debug_assertions, target_os = "linux"))]
mod tests {
    use super::*;
    use std::os::unix::fs::{symlink, PermissionsExt};

    #[test]
    fn local_snapshots_reload_and_refuse_symlinks_permissions_and_traversal() {
        let base = std::env::temp_dir().join(format!("hub-loader-{}", std::process::id()));
        std::fs::create_dir(&base).unwrap();
        let root = base.join("dev-plugins/dev.mouse");
        std::fs::create_dir_all(&root).unwrap();
        let file = root.join("plugin.py");
        std::fs::write(&file, "# first").unwrap();
        assert_eq!(load_plugin(&base, "dev.mouse").unwrap(), "# first");
        std::fs::write(&file, "# changed").unwrap();
        assert_eq!(load_plugin(&base, "dev.mouse").unwrap(), "# changed");
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o666)).unwrap();
        assert!(load_plugin(&base, "dev.mouse").is_err());
        std::fs::remove_file(&file).unwrap();
        symlink("/etc/passwd", &file).unwrap();
        assert!(load_plugin(&base, "dev.mouse").is_err());
        for id in [
            "dev../mouse",
            "dev.mouse/../../bin",
            "dev.foo.",
            "dev.foo..bar",
            "file:///bin/sh",
            "dev.",
        ] {
            assert!(load_plugin(&base, id).is_err());
        }
        std::fs::remove_dir_all(base).unwrap();
    }
}
pub fn source(_hardware: bool, _wire: &str) -> Result<String, String> {
    Err("R1_DISTRIBUTION_BLOCKED".into())
}

#[cfg(debug_assertions)]
fn data_home() -> Result<std::path::PathBuf, String> {
    let path = match std::env::var_os("XDG_DATA_HOME") {
        Some(p) => std::path::PathBuf::from(p),
        None => std::path::PathBuf::from(
            std::env::var_os("HOME").ok_or("LOCAL_INTEGRATION_UNAVAILABLE")?,
        )
        .join(".local/share"),
    };
    if !path.is_absolute() || path.to_string_lossy().starts_with("//") {
        return Err("LOCAL_INTEGRATION_UNSAFE".into());
    }
    Ok(path.join("peripheral-hub"))
}

/// Only a host-selected local DEV registration, never a path supplied over IPC.
#[cfg(all(debug_assertions, target_os = "linux"))]
pub fn starter_source() -> Result<String, String> {
    load_starter(&data_home()?)
}

#[cfg(all(debug_assertions, target_os = "linux"))]
fn load_starter(base: &std::path::Path) -> Result<String, String> {
    Directory::open(base)?
        .child("dev-starter")?
        .read("plugin.py", "")
}

#[cfg(all(debug_assertions, not(target_os = "linux")))]
pub fn starter_source() -> Result<String, String> {
    Err("LOCAL_INTEGRATION_UNAVAILABLE".into())
}

#[cfg(all(debug_assertions, target_os = "linux"))]
struct Directory(std::fs::File);

#[cfg(all(debug_assertions, target_os = "linux"))]
impl Directory {
    fn open(path: &std::path::Path) -> Result<Self, String> {
        use std::path::Component;
        if !path.is_absolute() {
            return Err("LOCAL_INTEGRATION_UNSAFE".into());
        }
        // Traverse every ancestor by descriptor. Renames cannot redirect the
        // descriptor and symlinks cannot redirect any component to another tree.
        let mut directory =
            Self(std::fs::File::open("/").map_err(|_| "LOCAL_INTEGRATION_UNAVAILABLE")?);
        for part in path.components() {
            match part {
                Component::RootDir => (),
                Component::Normal(name) => {
                    directory = Self(directory.open_at(name, true)?);
                }
                _ => return Err("LOCAL_INTEGRATION_UNSAFE".into()),
            }
        }
        directory.trusted()?;
        Ok(directory)
    }

    fn open_at(&self, name: &std::ffi::OsStr, directory: bool) -> Result<std::fs::File, String> {
        use std::os::{
            fd::{AsRawFd, FromRawFd},
            unix::ffi::OsStrExt,
        };
        let bytes = name.as_bytes();
        if bytes.is_empty() || bytes.contains(&b'/') || bytes == b"." || bytes == b".." {
            return Err("LOCAL_INTEGRATION_UNSAFE".into());
        }
        let name = std::ffi::CString::new(bytes).map_err(|_| "LOCAL_INTEGRATION_UNSAFE")?;
        let flags = libc::O_RDONLY
            | libc::O_CLOEXEC
            | libc::O_NOFOLLOW
            | libc::O_NONBLOCK
            | if directory { libc::O_DIRECTORY } else { 0 };
        // SAFETY: valid owned descriptor and NUL-terminated single component;
        // returned descriptor is exclusively owned by File on success.
        let fd = unsafe { libc::openat(self.0.as_raw_fd(), name.as_ptr(), flags) };
        if fd < 0 {
            return Err("LOCAL_INTEGRATION_UNAVAILABLE".into());
        }
        Ok(unsafe { std::fs::File::from_raw_fd(fd) })
    }

    fn trusted_file(file: &std::fs::File) -> Result<(), String> {
        use std::os::unix::fs::MetadataExt;
        let metadata = file
            .metadata()
            .map_err(|_| "LOCAL_INTEGRATION_UNAVAILABLE")?;
        // SAFETY: geteuid has no arguments or side effects.
        let uid = unsafe { libc::geteuid() };
        // Reject network and FUSE-backed mounts (including gvfs/sshfs). This is
        // deliberately conservative: development code must live on local storage.
        use std::os::fd::AsRawFd;
        let mut fs = std::mem::MaybeUninit::<libc::statfs>::uninit();
        if unsafe { libc::fstatfs(file.as_raw_fd(), fs.as_mut_ptr()) } != 0 {
            return Err("LOCAL_INTEGRATION_UNSAFE".into());
        }
        let kind = unsafe { fs.assume_init() }.f_type as u32;
        if matches!(
            kind,
            0x6969
                | 0xff534d42
                | 0xfe534d42
                | 0x65735546
                | 0x517b
                | 0x73757245
                | 0x01021997
                | 0x5346414f
                | 0x6b414653
                | 0x00c36400
                | 0x9fa0
        ) {
            return Err("LOCAL_INTEGRATION_UNSAFE".into());
        }
        if (metadata.uid() != uid && metadata.uid() != 0) || metadata.mode() & 0o022 != 0 {
            return Err("LOCAL_INTEGRATION_UNSAFE".into());
        }
        Ok(())
    }

    fn trusted(&self) -> Result<(), String> {
        Self::trusted_file(&self.0)
    }

    fn child(&self, name: &str) -> Result<Self, String> {
        let directory = Self(self.open_at(std::ffi::OsStr::new(name), true)?);
        directory.trusted()?;
        Ok(directory)
    }

    fn read(&self, name: &str, pin: &str) -> Result<String, String> {
        use sha2::{Digest, Sha256};
        use std::io::Read;
        let file = self.open_at(std::ffi::OsStr::new(name), false)?;
        Self::trusted_file(&file)?;
        let metadata = file
            .metadata()
            .map_err(|_| "LOCAL_INTEGRATION_UNAVAILABLE")?;
        if !metadata.is_file() || metadata.len() > 65536 {
            return Err("LOCAL_SOURCE_INTEGRITY".into());
        }
        let mut bytes = Vec::new();
        file.take(65537)
            .read_to_end(&mut bytes)
            .map_err(|_| "LOCAL_SOURCE_INTEGRITY")?;
        if bytes.len() > 65536
            || (!pin.is_empty() && format!("{:x}", Sha256::digest(&bytes)) != pin)
        {
            return Err("LOCAL_SOURCE_INTEGRITY".into());
        }
        // Execute this snapshot, never reopen verified paths in the worker.
        String::from_utf8(bytes).map_err(|_| "LOCAL_SOURCE_INTEGRITY".into())
    }
}
