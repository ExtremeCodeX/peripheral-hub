//! Import, normalize and reload bounded research captures.

use std::path::Path;

use crate::error::{DraError, DraResult};
use crate::model::{
    CAPTURE_SCHEMA_VERSION, CaptureFile, CaptureMeta, MAX_CAPTURE_FILE_BYTES, MAX_CAPTURES,
    NormalizedCapture, ResearchSession,
};
use crate::paths::{DraPaths, read_json, read_limited, to_pretty_json, write_atomic};
use crate::session;

/// Import one JSON capture into a session and persist its normalized form.
///
/// The input file is read only after a size check; the session is updated only
/// after the normalized capture was written. On a session-write failure the
/// just-written capture file is removed.
pub fn import(
    paths: &DraPaths,
    session_id: &str,
    input: &Path,
    now: u64,
) -> DraResult<(ResearchSession, CaptureMeta)> {
    let mut session = session::load(paths, session_id)?;
    if session.captures.len() >= MAX_CAPTURES {
        return Err(DraError::LimitExceeded {
            kind: "captures".to_owned(),
            limit: MAX_CAPTURES,
        });
    }
    let text = read_limited(input, MAX_CAPTURE_FILE_BYTES)?;
    let parsed: CaptureFile = serde_json::from_str(&text)?;
    let normalized = parsed.normalize()?;
    let file = format!("capture-{:04}.json", session.captures.len() + 1);
    let destination = session::capture_file(paths, session_id, &file)?;
    write_atomic(
        &destination,
        to_pretty_json(&normalized.document)?.as_bytes(),
    )?;
    let meta = CaptureMeta {
        file,
        source: normalized.document.source.clone(),
        description: normalized.document.description.clone(),
        event_count: normalized.events.len(),
        reordered: normalized.reordered,
        imported_at_unix: now,
    };
    meta.validate()?;
    session.captures.push(meta.clone());
    if let Err(error) = session::save(paths, &session) {
        let _ = std::fs::remove_file(&destination);
        return Err(error);
    }
    Ok((session, meta))
}

/// Load and re-validate one imported capture, checking session metadata.
pub fn load_capture(
    paths: &DraPaths,
    session_id: &str,
    meta: &CaptureMeta,
) -> DraResult<NormalizedCapture> {
    meta.validate()?;
    let path = session::capture_file(paths, session_id, &meta.file)?;
    let document: CaptureFile = read_json(&path, MAX_CAPTURE_FILE_BYTES)?;
    if document.capture_schema_version != CAPTURE_SCHEMA_VERSION {
        return Err(DraError::UnsupportedVersion {
            kind: "capture",
            found: document.capture_schema_version,
            expected: CAPTURE_SCHEMA_VERSION,
        });
    }
    let normalized = document.normalize()?;
    if normalized.events.len() != meta.event_count {
        return Err(DraError::InvalidArgument(format!(
            "capture '{}' has {} events but the session records {}",
            meta.file,
            normalized.events.len(),
            meta.event_count
        )));
    }
    if normalized.document.source != meta.source {
        return Err(DraError::InvalidArgument(format!(
            "capture '{}' source does not match the session record",
            meta.file
        )));
    }
    Ok(normalized)
}

/// Load every capture referenced by a session, in import order.
pub fn load_all(
    paths: &DraPaths,
    session: &ResearchSession,
) -> DraResult<Vec<(CaptureMeta, NormalizedCapture)>> {
    session
        .captures
        .iter()
        .map(|meta| Ok((meta.clone(), load_capture(paths, &session.id, meta)?)))
        .collect()
}
