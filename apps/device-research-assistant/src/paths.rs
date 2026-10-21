//! Local data layout and bounded, atomic file helpers.

use std::path::{Path, PathBuf};

use crate::error::{DraError, DraResult};
use crate::model::validate_token;

/// Resolve the per-user DRA root without writing inside the repository.
///
/// `PERIPHERAL_HUB_DRA_DATA` overrides the exact root (used by tests).
/// Otherwise `PERIPHERAL_HUB_DATA`, `LOCALAPPDATA` or `XDG_DATA_HOME`/`HOME`
/// provide a parent and the root becomes `<parent>/peripheral-hub/dra`.
pub fn default_data_root() -> PathBuf {
    if let Some(exact) = std::env::var_os("PERIPHERAL_HUB_DRA_DATA") {
        return PathBuf::from(exact);
    }
    let base = std::env::var_os("PERIPHERAL_HUB_DATA")
        .map(PathBuf::from)
        .or_else(|| {
            #[cfg(windows)]
            {
                std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
            }
            #[cfg(not(windows))]
            {
                std::env::var_os("XDG_DATA_HOME")
                    .map(PathBuf::from)
                    .or_else(|| {
                        std::env::var_os("HOME")
                            .map(|home| PathBuf::from(home).join(".local/share"))
                    })
            }
        })
        .unwrap_or_else(std::env::temp_dir);
    base.join("peripheral-hub").join("dra")
}

/// Session storage rooted at a DRA data directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraPaths {
    root: PathBuf,
}

impl DraPaths {
    /// Use an explicit root or the resolved per-user default.
    pub fn new(root: Option<PathBuf>) -> Self {
        Self {
            root: root.unwrap_or_else(default_data_root),
        }
    }

    /// Configured DRA root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Directory containing all local sessions.
    pub fn sessions_dir(&self) -> PathBuf {
        self.root.join("sessions")
    }

    /// Directory for one validated session id.
    pub fn session_dir(&self, id: &str) -> PathBuf {
        self.sessions_dir().join(id)
    }

    /// Create the sessions root if needed.
    pub fn ensure_sessions_dir(&self) -> DraResult<()> {
        std::fs::create_dir_all(self.sessions_dir())?;
        Ok(())
    }

    /// Ensure an id cannot select another directory.
    pub fn checked_session_dir(&self, id: &str) -> DraResult<PathBuf> {
        validate_token(id, "session id", crate::model::MAX_SESSION_ID)?;
        Ok(self.session_dir(id))
    }
}

/// Read a UTF-8 file only after checking its size against a hard limit.
pub fn read_limited(path: &Path, limit: u64) -> DraResult<String> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > limit {
        return Err(DraError::LimitExceeded {
            kind: "input file bytes".to_owned(),
            limit: usize::try_from(limit).unwrap_or(usize::MAX),
        });
    }
    Ok(std::fs::read_to_string(path)?)
}

/// Write a file through a same-directory temporary name, then rename it.
///
/// On platforms where rename does not replace an existing file, the old file is
/// removed immediately before the rename. Derived reports may be regenerated.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> DraResult<()> {
    let parent = path.parent().ok_or_else(|| {
        DraError::InvalidArgument(format!("path '{}' has no parent directory", path.display()))
    })?;
    std::fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .ok_or_else(|| DraError::InvalidArgument("output path has no file name".into()))?
        .to_string_lossy();
    let temporary = parent.join(format!(".{file_name}.tmp-{}", std::process::id()));
    std::fs::write(&temporary, bytes)?;
    if path.exists()
        && let Err(error) = std::fs::remove_file(path)
    {
        let _ = std::fs::remove_file(&temporary);
        return Err(error.into());
    }
    if let Err(error) = std::fs::rename(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(error.into());
    }
    Ok(())
}

/// Current Unix time in seconds, clamped to zero for pre-epoch clocks.
pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// Serialize a document as pretty JSON with a trailing newline.
pub fn to_pretty_json<T: serde::Serialize>(value: &T) -> DraResult<String> {
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    Ok(text)
}

/// Read and deserialize a bounded UTF-8 JSON document.
pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path, limit: u64) -> DraResult<T> {
    let text = read_limited(path, limit)?;
    Ok(serde_json::from_str(&text)?)
}
