//! Versioned profile JSON file import/export for the diagnostic client.
//!
//! These helpers only move a validated, schema-versioned profile document to and
//! from a local path explicitly chosen by the caller. They never read raw
//! reports, vendor files or captures, and the path is never a wire field.

use peripheral_hub_profiles::{Profile, ProfileError};
use std::path::Path;

/// Write a validated profile as a schema-versioned JSON document, replacing `path`.
pub fn export_profile(path: &Path, profile: &Profile) -> Result<(), ProfileError> {
    let json = profile.to_json()?;
    std::fs::write(path, json.as_bytes())
        .map_err(|_| ProfileError::Invalid("profile export failed"))
}

/// Read, parse and validate a schema-versioned profile JSON document from `path`.
pub fn import_profile(path: &Path) -> Result<Profile, ProfileError> {
    let text = std::fs::read_to_string(path)
        .map_err(|_| ProfileError::Invalid("profile import failed"))?;
    Profile::from_json(&text)
}
