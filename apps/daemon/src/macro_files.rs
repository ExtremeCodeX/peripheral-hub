//! Versioned macro JSON file import/export for the diagnostic client.
//!
//! These helpers only move a validated, schema-versioned macro document to and
//! from a local path explicitly chosen by the caller. They never read captured
//! input, and importing a macro never executes it; persistence is a separate
//! operation the daemon performs without any device access.

use peripheral_hub_macros::{Macro, MacroError};
use std::path::Path;

/// Write a validated macro as a schema-versioned JSON document, replacing `path`.
pub fn export_macro(path: &Path, macro_value: &Macro) -> Result<(), MacroError> {
    let json = macro_value.to_json()?;
    std::fs::write(path, json.as_bytes()).map_err(|_| MacroError::Invalid("macro export failed"))
}

/// Read, parse and validate a schema-versioned macro JSON document from `path`.
pub fn import_macro(path: &Path) -> Result<Macro, MacroError> {
    let text =
        std::fs::read_to_string(path).map_err(|_| MacroError::Invalid("macro import failed"))?;
    Macro::from_json(&text)
}
