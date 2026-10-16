//! Explicit, fail-fast loading of local declarative manifests.
//!
//! Manifests are only loaded from a directory the operator names on the command
//! line. Invalid or conflicting manifests abort startup; nothing is fetched,
//! executed, or loaded at runtime beyond reviewed, bounded report encodings.

use std::path::{Path, PathBuf};

use peripheral_hub_device_registry::{DriverRegistry, RegistryError};
use peripheral_hub_device_sdk::ExecutionMode;
use peripheral_hub_driver_declarative::{DeclarativeFactory, ManifestError, load_manifest_file};

/// Failures while loading a manifest directory; categories are privacy-safe.
#[derive(Debug, thiserror::Error)]
pub enum ManifestLoadError {
    /// The manifest directory could not be listed.
    #[error("manifest directory could not be read: {0}")]
    Directory(std::io::Error),
    /// One manifest failed parsing or semantic validation.
    #[error("manifest {path} is invalid: {source}")]
    Invalid {
        /// Local path of the offending manifest.
        path: String,
        /// Underlying manifest failure.
        #[source]
        source: ManifestError,
    },
    /// A manifest's model conflicts with an already registered factory.
    #[error("manifest {path} conflicts with a registered model: {source}")]
    Conflict {
        /// Local path of the offending manifest.
        path: String,
        /// Underlying registry failure.
        #[source]
        source: RegistryError,
    },
}

impl ManifestLoadError {
    /// Static category for logs without exposing paths or manifest content.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Directory(_) => "manifest_directory",
            Self::Invalid { .. } => "manifest_invalid",
            Self::Conflict { .. } => "manifest_conflict",
        }
    }
}

/// Register every `*.toml` manifest in `dir`, in stable path order.
///
/// The provenance `mode` is explicit and must agree with the daemon's transport
/// source; a manifest never selects simulated success on its own. Any failure
/// leaves earlier registrations in place and aborts startup.
pub fn register_manifest_dir(
    registry: &mut DriverRegistry,
    dir: &Path,
    mode: ExecutionMode,
) -> Result<usize, ManifestLoadError> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(ManifestLoadError::Directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(std::ffi::OsStr::to_str) == Some("toml"))
        .collect();
    paths.sort();
    for path in &paths {
        let manifest = load_manifest_file(path).map_err(|source| ManifestLoadError::Invalid {
            path: path.display().to_string(),
            source,
        })?;
        let factory = DeclarativeFactory::new(manifest, mode).map_err(|source| {
            ManifestLoadError::Invalid {
                path: path.display().to_string(),
                source,
            }
        })?;
        registry
            .register(Box::new(factory))
            .map_err(|source| ManifestLoadError::Conflict {
                path: path.display().to_string(),
                source,
            })?;
    }
    tracing::info!(event = "declarative_manifests_loaded", count = paths.len());
    Ok(paths.len())
}
