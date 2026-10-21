//! Versioned local research-session and capture documents.
//!
//! All fields are bounded and validated before persistence. Raw payloads and
//! private device locators are never part of a session document; captures hold
//! only the normalized textual events that the user explicitly imported.

use serde::{Deserialize, Serialize};

use crate::error::{DraError, DraResult};

/// Version of the local `session.json` document.
pub const SESSION_SCHEMA_VERSION: u32 = 1;
/// Version of an imported, normalized capture document.
pub const CAPTURE_SCHEMA_VERSION: u32 = 1;
/// Maximum accepted characters in a session id.
pub const MAX_SESSION_ID: usize = 64;
/// Maximum accepted characters in a session or device name.
pub const MAX_NAME: usize = 128;
/// Maximum accepted characters in a free-text goal.
pub const MAX_GOAL: usize = 512;
/// Maximum accepted characters in a capture description.
pub const MAX_DESCRIPTION: usize = 512;
/// Maximum accepted characters in a correlated action label.
pub const MAX_ACTION: usize = 128;
/// Maximum captures stored in one session.
pub const MAX_CAPTURES: usize = 32;
/// Maximum events in one capture document.
pub const MAX_CAPTURE_EVENTS: usize = 20_000;
/// Maximum events analyzed across all captures of one session.
pub const MAX_TOTAL_EVENTS: usize = 100_000;
/// Maximum decoded payload size in bytes.
pub const MAX_PAYLOAD_BYTES: usize = 4096;
/// Maximum accepted capture file size in bytes.
pub const MAX_CAPTURE_FILE_BYTES: u64 = 16 * 1024 * 1024;

/// Validate a lowercase, path-safe token such as a session id or source name.
///
/// Tokens start with an ASCII letter or digit, contain only lowercase letters,
/// digits, `-`, `_` and `.`, and never contain `..`.
pub fn validate_token(value: &str, field: &str, max: usize) -> DraResult<()> {
    let mut chars = value.chars();
    let starts_safely = chars
        .next()
        .is_some_and(|first| first.is_ascii_lowercase() || first.is_ascii_digit());
    if !starts_safely
        || value.chars().count() > max
        || value.contains("..")
        || !value.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '-' | '_' | '.')
        })
    {
        return Err(DraError::InvalidArgument(format!(
            "{field} must be 1..={max} lowercase token characters (letters, digits, '-', '_', '.'), start alphanumerically and avoid '..'"
        )));
    }
    Ok(())
}

/// Validate required human-readable text without control characters.
pub fn validate_text(value: &str, field: &str, max: usize) -> DraResult<()> {
    if value.trim().is_empty() {
        return Err(DraError::InvalidArgument(format!(
            "{field} must not be empty"
        )));
    }
    if value.chars().count() > max {
        return Err(DraError::LimitExceeded {
            kind: field.to_owned(),
            limit: max,
        });
    }
    if value.chars().any(char::is_control) {
        return Err(DraError::InvalidArgument(format!(
            "{field} must not contain control characters or newlines"
        )));
    }
    Ok(())
}

/// Validate an optional text field, treating `None` as absent.
pub fn validate_optional_text(value: Option<&str>, field: &str, max: usize) -> DraResult<()> {
    match value {
        Some(value) => validate_text(value, field, max),
        None => Ok(()),
    }
}

/// Validate an optional lowercase or uppercase SHA-256 hex digest.
pub fn validate_sha256(value: Option<&str>) -> DraResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.len() != 64 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(DraError::InvalidArgument(
            "software SHA-256 must be exactly 64 hexadecimal characters".into(),
        ));
    }
    Ok(())
}

/// Validate a capture file name stored by the session (`capture-0001.json`).
pub fn validate_capture_file_name(value: &str) -> DraResult<()> {
    let bytes = value.as_bytes();
    let valid = bytes.len() == 17
        && value.starts_with("capture-")
        && value.ends_with(".json")
        && bytes[8..12].iter().all(u8::is_ascii_digit);
    if !valid {
        return Err(DraError::InvalidArgument(
            "capture file must use the capture-NNNN.json form".into(),
        ));
    }
    Ok(())
}

/// Device identity recorded for a research session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionDevice {
    /// User-supplied model name; not proof of a physical identity.
    pub model: String,
    /// Firmware/revision only when actually known.
    #[serde(default)]
    pub firmware: Option<String>,
}

impl SessionDevice {
    /// Validate bounded fields without accessing hardware.
    pub fn validate(&self) -> DraResult<()> {
        validate_text(&self.model, "device model", MAX_NAME)?;
        validate_optional_text(self.firmware.as_deref(), "firmware", MAX_NAME)
    }
}

/// Original software artifact metadata; never an executable copy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSoftware {
    /// Descriptive software name or version.
    pub name: String,
    /// Optional SHA-256 of the locally held artifact.
    #[serde(default)]
    pub sha256: Option<String>,
}

impl SessionSoftware {
    /// Validate the descriptive name and optional digest.
    pub fn validate(&self) -> DraResult<()> {
        validate_text(&self.name, "software name", MAX_NAME)?;
        validate_sha256(self.sha256.as_deref())
    }
}

/// Reference to one normalized capture stored inside the session directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureMeta {
    /// Safe relative file name (`capture-NNNN.json`).
    pub file: String,
    /// Declared origin token, such as `synthetic` or `tshark-json`.
    pub source: String,
    /// Optional short description of the captured scenario.
    #[serde(default)]
    pub description: Option<String>,
    /// Number of events in the normalized file.
    pub event_count: usize,
    /// Whether the imported source order differed from normalized timestamp order.
    #[serde(default)]
    pub reordered: bool,
    /// Unix timestamp of the local import.
    pub imported_at_unix: u64,
}

impl CaptureMeta {
    /// Validate the file name, origin and bounds.
    pub fn validate(&self) -> DraResult<()> {
        validate_capture_file_name(&self.file)?;
        validate_token(&self.source, "capture source", 64)?;
        validate_optional_text(
            self.description.as_deref(),
            "capture description",
            MAX_DESCRIPTION,
        )?;
        if self.event_count == 0 || self.event_count > MAX_CAPTURE_EVENTS {
            return Err(DraError::InvalidArgument(format!(
                "capture event count must be 1..={MAX_CAPTURE_EVENTS}"
            )));
        }
        Ok(())
    }
}

/// Local, versioned research session. It contains metadata only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchSession {
    /// Version of this document.
    pub session_schema_version: u32,
    /// Path-safe session identifier.
    pub id: String,
    /// Human-readable session name.
    pub name: String,
    /// Local creation time in Unix seconds.
    pub created_at_unix: u64,
    /// Recorded device identity.
    pub device: SessionDevice,
    /// Original software metadata, if supplied.
    #[serde(default)]
    pub software: Option<SessionSoftware>,
    /// Interoperability question for this session.
    pub goal: String,
    /// Explicit consent to conduct authorized research.
    pub consent: bool,
    /// Imported, normalized captures in import order.
    #[serde(default)]
    pub captures: Vec<CaptureMeta>,
}

impl ResearchSession {
    /// Validate every field before loading or writing the document.
    pub fn validate(&self) -> DraResult<()> {
        if self.session_schema_version != SESSION_SCHEMA_VERSION {
            return Err(DraError::UnsupportedVersion {
                kind: "session",
                found: self.session_schema_version,
                expected: SESSION_SCHEMA_VERSION,
            });
        }
        validate_token(&self.id, "session id", MAX_SESSION_ID)?;
        validate_text(&self.name, "session name", MAX_NAME)?;
        validate_text(&self.goal, "goal", MAX_GOAL)?;
        self.device.validate()?;
        if let Some(software) = &self.software {
            software.validate()?;
        }
        if !self.consent {
            return Err(DraError::ConsentRequired);
        }
        if self.captures.len() > MAX_CAPTURES {
            return Err(DraError::LimitExceeded {
                kind: "captures".to_owned(),
                limit: MAX_CAPTURES,
            });
        }
        let mut files = std::collections::BTreeSet::new();
        for capture in &self.captures {
            capture.validate()?;
            if !files.insert(capture.file.as_str()) {
                return Err(DraError::InvalidArgument(format!(
                    "duplicate capture file '{}'",
                    capture.file
                )));
            }
        }
        Ok(())
    }
}

/// Direction of an observed USB/HID transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Device-to-host traffic.
    In,
    /// Host-to-device traffic.
    Out,
}

impl Direction {
    /// Stable lowercase label used in reports.
    pub fn label(self) -> &'static str {
        match self {
            Self::In => "in",
            Self::Out => "out",
        }
    }
}

/// Generic transfer category recorded by an import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transfer {
    /// Control transfer.
    Control,
    /// Interrupt transfer.
    Interrupt,
    /// Bulk transfer.
    Bulk,
}

impl Transfer {
    /// Stable lowercase label used in reports.
    pub fn label(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Interrupt => "interrupt",
            Self::Bulk => "bulk",
        }
    }
}

/// One textual event in a stable import document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureEvent {
    /// Monotonic timestamp in microseconds as exported by the source.
    pub timestamp_us: u64,
    /// Transfer direction.
    pub direction: Direction,
    /// Transfer category.
    pub transfer: Transfer,
    /// Observed interface number.
    pub interface: u8,
    /// Optional report id, distinct from an unnumbered report.
    #[serde(default)]
    pub report_id: Option<u8>,
    /// Hex-encoded payload; ASCII whitespace between digits is ignored.
    pub payload_hex: String,
    /// Optional descriptive action correlated by the researcher.
    #[serde(default)]
    pub action: Option<String>,
}

/// Stable, versioned JSON document accepted by `capture import`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureFile {
    /// Version of this document.
    pub capture_schema_version: u32,
    /// Lowercase origin token, not a private path.
    pub source: String,
    /// Optional short description.
    #[serde(default)]
    pub description: Option<String>,
    /// Events in source order; normalization sorts them deterministically.
    pub events: Vec<CaptureEvent>,
}

impl CaptureFile {
    /// Validate the document shape and every event without writing anything.
    pub fn validate(&self) -> DraResult<()> {
        if self.capture_schema_version != CAPTURE_SCHEMA_VERSION {
            return Err(DraError::UnsupportedVersion {
                kind: "capture",
                found: self.capture_schema_version,
                expected: CAPTURE_SCHEMA_VERSION,
            });
        }
        validate_token(&self.source, "capture source", 64)?;
        validate_optional_text(
            self.description.as_deref(),
            "capture description",
            MAX_DESCRIPTION,
        )?;
        if self.events.is_empty() {
            return Err(DraError::InvalidArgument(
                "capture must contain at least one event".into(),
            ));
        }
        if self.events.len() > MAX_CAPTURE_EVENTS {
            return Err(DraError::LimitExceeded {
                kind: "capture events".to_owned(),
                limit: MAX_CAPTURE_EVENTS,
            });
        }
        for (index, event) in self.events.iter().enumerate() {
            if let Some(action) = &event.action {
                validate_text(action, "action label", MAX_ACTION)?;
            }
            let payload = decode_hex(&event.payload_hex).map_err(|message| {
                DraError::InvalidArgument(format!("event {index}: {message}"))
            })?;
            if payload.len() > MAX_PAYLOAD_BYTES {
                return Err(DraError::LimitExceeded {
                    kind: "payload bytes".to_owned(),
                    limit: MAX_PAYLOAD_BYTES,
                });
            }
        }
        Ok(())
    }
}

/// Decode a whitespace-tolerant hexadecimal payload.
///
/// The input must contain an even number of hex digits. Empty payloads are
/// accepted because some control transfers carry no data.
pub fn decode_hex(value: &str) -> Result<Vec<u8>, &'static str> {
    let mut digits = Vec::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_whitespace() {
            continue;
        }
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return Err("payload_hex contains a non-hexadecimal character"),
        };
        digits.push(digit);
    }
    if digits.len() % 2 != 0 {
        return Err("payload_hex must contain an even number of hex digits");
    }
    let mut bytes = Vec::with_capacity(digits.len() / 2);
    for index in 0..digits.len() / 2 {
        bytes.push((digits[index * 2] << 4) | digits[index * 2 + 1]);
    }
    Ok(bytes)
}

/// Encode bytes as lowercase hexadecimal.
pub fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

/// One validated, decoded event used by the analysis pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// Original position inside the capture file.
    pub index: usize,
    /// Source timestamp.
    pub timestamp_us: u64,
    /// Transfer direction.
    pub direction: Direction,
    /// Transfer category.
    pub transfer: Transfer,
    /// Interface number.
    pub interface: u8,
    /// Optional report id.
    pub report_id: Option<u8>,
    /// Decoded payload.
    pub payload: Vec<u8>,
    /// Optional descriptive action.
    pub action: Option<String>,
}

/// A validated capture plus its normalized events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedCapture {
    /// Document rewritten with sorted events and lowercase hex.
    pub document: CaptureFile,
    /// Decoded events sorted by `(timestamp_us, original index)`.
    pub events: Vec<Event>,
    /// Whether the input order differed from the normalized order.
    pub reordered: bool,
}

impl CaptureFile {
    /// Validate, decode and sort a capture deterministically.
    pub fn normalize(&self) -> DraResult<NormalizedCapture> {
        self.validate()?;
        let mut events: Vec<Event> = self
            .events
            .iter()
            .enumerate()
            .map(|(index, event)| {
                let payload = decode_hex(&event.payload_hex).map_err(|message| {
                    DraError::InvalidArgument(format!("event {index}: {message}"))
                })?;
                Ok(Event {
                    index,
                    timestamp_us: event.timestamp_us,
                    direction: event.direction,
                    transfer: event.transfer,
                    interface: event.interface,
                    report_id: event.report_id,
                    payload,
                    action: event.action.clone(),
                })
            })
            .collect::<DraResult<Vec<_>>>()?;
        events.sort_by_key(|event| (event.timestamp_us, event.index));
        let reordered = events
            .iter()
            .enumerate()
            .any(|(position, event)| position != event.index);
        let document = CaptureFile {
            capture_schema_version: self.capture_schema_version,
            source: self.source.clone(),
            description: self.description.clone(),
            events: events
                .iter()
                .map(|event| CaptureEvent {
                    timestamp_us: event.timestamp_us,
                    direction: event.direction,
                    transfer: event.transfer,
                    interface: event.interface,
                    report_id: event.report_id,
                    payload_hex: encode_hex(&event.payload),
                    action: event.action.clone(),
                })
                .collect(),
        };
        Ok(NormalizedCapture {
            document,
            events,
            reordered,
        })
    }
}
