//! Create, load and list local research sessions.

use std::path::PathBuf;

use crate::error::{DraError, DraResult};
use crate::model::{
    MAX_CAPTURES, MAX_NAME, MAX_SESSION_ID, ResearchSession, SESSION_SCHEMA_VERSION, SessionDevice,
    SessionSoftware, validate_optional_text, validate_sha256, validate_text, validate_token,
};
use crate::paths::{DraPaths, read_json, to_pretty_json, write_atomic};

const SESSION_FILE_BYTES: u64 = 1024 * 1024;

/// User-supplied metadata for a new session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSpec {
    /// Optional explicit id; generated from the name when absent.
    pub id: Option<String>,
    /// Human-readable session name.
    pub name: String,
    /// Device model name.
    pub device: String,
    /// Firmware or revision, only when known.
    pub firmware: Option<String>,
    /// Original software name/version, only when known.
    pub software: Option<String>,
    /// SHA-256 of the locally held software artifact, only when known.
    pub sha256: Option<String>,
    /// Interoperability question.
    pub goal: String,
    /// Explicit consent to conduct authorized research.
    pub consent: bool,
}

impl SessionSpec {
    fn validate(&self) -> DraResult<()> {
        validate_text(&self.name, "session name", MAX_NAME)?;
        validate_text(&self.device, "device model", MAX_NAME)?;
        validate_optional_text(self.firmware.as_deref(), "firmware", MAX_NAME)?;
        validate_optional_text(self.software.as_deref(), "software name", MAX_NAME)?;
        validate_sha256(self.sha256.as_deref())?;
        if self.sha256.is_some() && self.software.is_none() {
            return Err(DraError::InvalidArgument(
                "--sha256 requires --software".into(),
            ));
        }
        if let Some(id) = &self.id {
            validate_token(id, "session id", MAX_SESSION_ID)?;
        }
        validate_text(&self.goal, "goal", crate::model::MAX_GOAL)?;
        if !self.consent {
            return Err(DraError::ConsentRequired);
        }
        Ok(())
    }
}

/// Path of the versioned session document.
pub fn session_file(paths: &DraPaths, id: &str) -> DraResult<PathBuf> {
    Ok(paths.checked_session_dir(id)?.join("session.json"))
}

/// Path of one normalized capture file, after validating its stored name.
pub fn capture_file(paths: &DraPaths, id: &str, file: &str) -> DraResult<PathBuf> {
    crate::model::validate_capture_file_name(file)?;
    Ok(paths.checked_session_dir(id)?.join(file))
}

/// Create a new session directory and its `session.json`.
///
/// `now` is injected so tests can produce deterministic documents.
pub fn create(paths: &DraPaths, spec: SessionSpec, now: u64) -> DraResult<ResearchSession> {
    spec.validate()?;
    paths.ensure_sessions_dir()?;
    let id = match &spec.id {
        Some(id) => {
            let directory = paths.checked_session_dir(id)?;
            if directory.exists() {
                return Err(DraError::SessionExists(id.clone()));
            }
            id.clone()
        }
        None => unique_generated_id(paths, &spec.name, now)?,
    };
    let directory = paths.checked_session_dir(&id)?;
    std::fs::create_dir(&directory).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            DraError::SessionExists(id.clone())
        } else {
            DraError::Io(error)
        }
    })?;
    let session = ResearchSession {
        session_schema_version: SESSION_SCHEMA_VERSION,
        id,
        name: spec.name,
        created_at_unix: now,
        device: SessionDevice {
            model: spec.device,
            firmware: spec.firmware,
        },
        software: spec.software.map(|name| SessionSoftware {
            name,
            sha256: spec.sha256,
        }),
        goal: spec.goal,
        consent: spec.consent,
        captures: Vec::new(),
    };
    if let Err(error) = write_session(paths, &session) {
        let _ = std::fs::remove_dir_all(&directory);
        return Err(error);
    }
    Ok(session)
}

/// Load and validate a session by id.
pub fn load(paths: &DraPaths, id: &str) -> DraResult<ResearchSession> {
    let directory = paths.checked_session_dir(id)?;
    if !directory.is_dir() {
        return Err(DraError::SessionNotFound(id.to_owned()));
    }
    let session: ResearchSession = read_json(&directory.join("session.json"), SESSION_FILE_BYTES)?;
    session.validate()?;
    if session.id != id {
        return Err(DraError::InvalidArgument(format!(
            "session document id '{}' does not match directory '{id}'",
            session.id
        )));
    }
    Ok(session)
}

/// Validate and atomically write a session document.
pub fn save(paths: &DraPaths, session: &ResearchSession) -> DraResult<()> {
    session.validate()?;
    write_session(paths, session)
}

fn write_session(paths: &DraPaths, session: &ResearchSession) -> DraResult<()> {
    let file = session_file(paths, &session.id)?;
    write_atomic(&file, to_pretty_json(session)?.as_bytes())
}

/// List valid sessions ordered by creation time and id.
pub fn list(paths: &DraPaths) -> DraResult<Vec<ResearchSession>> {
    let directory = paths.sessions_dir();
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(&directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        if validate_token(&id, "session id", MAX_SESSION_ID).is_err() {
            continue;
        }
        if !entry.path().join("session.json").is_file() {
            continue;
        }
        sessions.push(load(paths, &id)?);
    }
    sessions.sort_by(|left, right| {
        (left.created_at_unix, left.id.as_str()).cmp(&(right.created_at_unix, right.id.as_str()))
    });
    Ok(sessions)
}

/// Number of captures still available for a session.
pub fn remaining_captures(session: &ResearchSession) -> usize {
    MAX_CAPTURES.saturating_sub(session.captures.len())
}

fn unique_generated_id(paths: &DraPaths, name: &str, now: u64) -> DraResult<String> {
    let stamp = now.to_string();
    let max_slug = MAX_SESSION_ID.saturating_sub(stamp.len() + 1 + 4);
    let mut slug = slug(name);
    if slug.len() > max_slug {
        slug.truncate(max_slug);
        slug = slug.trim_end_matches('-').to_owned();
    }
    if slug.is_empty() {
        slug.push_str("session");
    }
    let base = format!("{slug}-{stamp}");
    let mut candidate = base.clone();
    let mut suffix = 2u32;
    while paths.session_dir(&candidate).exists() {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
        if suffix > 1000 {
            return Err(DraError::InvalidArgument(
                "could not allocate a unique session id".into(),
            ));
        }
    }
    validate_token(&candidate, "session id", MAX_SESSION_ID)?;
    Ok(candidate)
}

fn slug(name: &str) -> String {
    let mut output = String::new();
    for character in name.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            output.push(character);
        } else if (character.is_ascii_whitespace() || matches!(character, '-' | '_' | '.'))
            && !output.ends_with('-')
            && !output.is_empty()
        {
            output.push('-');
        }
        if output.len() >= 40 {
            break;
        }
    }
    let slug = output.trim_matches('-');
    if slug.is_empty() {
        "session".to_owned()
    } else {
        slug.to_owned()
    }
}
