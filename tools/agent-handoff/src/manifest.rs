//! Bounded structural validation of candidate declarative manifests.
//!
//! The real `peripheral-hub-driver-declarative` parser and semantic validator
//! are used directly. Success means the manifest is well formed and models only
//! supported bounded operations; it never proves that the manifest matches a
//! report, a physical model or a firmware version.

use std::path::Path;

use peripheral_hub_driver_declarative::{Manifest, parse_manifest};
use sha2::{Digest, Sha256};

use crate::error::{HandoffError, HandoffResult};
use crate::model::{MAX_MANIFEST_BYTES, ManifestEvidence};

/// Validate a manifest file and summarize its typed, semantic result.
pub fn validate_manifest(path: &Path) -> HandoffResult<ManifestEvidence> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_MANIFEST_BYTES {
        return Err(HandoffError::LimitExceeded {
            kind: "manifest bytes".to_owned(),
            limit: usize::try_from(MAX_MANIFEST_BYTES).unwrap_or(usize::MAX),
        });
    }
    let bytes = std::fs::read(path)?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(HandoffError::LimitExceeded {
            kind: "manifest bytes".to_owned(),
            limit: usize::try_from(MAX_MANIFEST_BYTES).unwrap_or(usize::MAX),
        });
    }
    let text = std::str::from_utf8(&bytes).map_err(|error| {
        HandoffError::InvalidManifest(format!("manifest is not UTF-8: {error}"))
    })?;
    let manifest =
        parse_manifest(text).map_err(|error| HandoffError::InvalidManifest(error.to_string()))?;
    Ok(summary(&manifest, &bytes))
}

fn summary(manifest: &Manifest, bytes: &[u8]) -> ManifestEvidence {
    ManifestEvidence {
        sha256: format!("{:x}", Sha256::digest(bytes)),
        model_id: manifest.model.id.to_string(),
        display_name: manifest.model.display_name.clone(),
        match_rule_count: manifest.device_match.len(),
        operation_count: manifest.operations.len(),
        sdk_major: manifest.model.sdk_major,
        sdk_minor: manifest.model.sdk_minor,
        validation_note: "Structurally valid for the declarative interpreter; not a report match, hardware test or capability claim."
            .to_owned(),
    }
}
