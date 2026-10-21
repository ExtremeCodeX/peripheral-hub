use std::fmt;

use peripheral_hub_profiles::ProfileId;
use serde::{Deserialize, Serialize};

use crate::MacroError;

/// Macro interchange schema version, independent from the SDK and profiles.
pub const MACRO_SCHEMA_VERSION: u16 = 1;
/// Maximum number of actions in one macro iteration.
pub const MAX_MACRO_ACTIONS: usize = 4096;
/// Maximum byte length of a macro name.
pub const MAX_MACRO_NAME: usize = 64;
/// Maximum duration of a single delay action in milliseconds.
pub const MAX_DELAY_MS: u32 = 60_000;
/// Maximum total delay duration of one macro iteration in milliseconds.
pub const MAX_MACRO_ITERATION_MS: u64 = 600_000;
/// Maximum repetition count of one macro; repetition is always finite.
pub const MAX_MACRO_REPEAT: u16 = 1_000;
/// Maximum pause between iterations in milliseconds.
pub const MAX_MACRO_INTERVAL_MS: u32 = 60_000;
/// Maximum total (repeated) duration of one macro in milliseconds.
pub const MAX_MACRO_TOTAL_MS: u64 = 3_600_000;

/// A symbolic token: 1..=128 ASCII bytes with no whitespace or path separators.
fn is_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
}

/// Stable, validated macro identifier token.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MacroId(String);

impl MacroId {
    /// Validate a nonempty, bounded ASCII token (no paths or whitespace).
    pub fn new(value: impl Into<String>) -> Result<Self, MacroError> {
        let value = value.into();
        if !is_token(&value) {
            return Err(MacroError::Invalid("macro id must be a 1..128 byte token"));
        }
        Ok(Self(value))
    }

    /// Access the validated token.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for MacroId {
    type Error = MacroError;
    fn try_from(value: String) -> Result<Self, MacroError> {
        Self::new(value)
    }
}

impl From<MacroId> for String {
    fn from(value: MacroId) -> Self {
        value.0
    }
}

impl fmt::Display for MacroId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Key transition semantics. The macro stores a symbolic key name, never the
/// user's actual typed input, so no keystroke logging is possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyPress {
    /// Hold the key down.
    Down,
    /// Release the key.
    Up,
    /// Press and release the key.
    Tap,
}

/// One bounded macro action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MacroAction {
    /// Press/release a named key by symbolic token.
    Key {
        /// Symbolic key name (for example `f13`); not captured input.
        key: String,
        /// Transition to perform.
        press: KeyPress,
    },
    /// Wait for 1..=60000 milliseconds.
    Delay {
        /// Delay in milliseconds.
        ms: u32,
    },
}

/// Bounded repetition of a macro sequence; never an unbounded loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacroRepeat {
    /// Number of iterations, 1..=1000.
    pub times: u16,
    /// Pause between iterations in milliseconds, 0..=60000.
    pub interval_ms: u32,
}

impl Default for MacroRepeat {
    fn default() -> Self {
        Self {
            times: 1,
            interval_ms: 0,
        }
    }
}

/// A stored macro: named, bounded action sequence with an optional profile link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Macro {
    /// Stable identifier; saving an existing id replaces that macro.
    pub id: MacroId,
    /// Human-readable name (1..64 bytes, no control characters).
    pub name: String,
    /// Ordered, bounded actions; at least one is required.
    pub actions: Vec<MacroAction>,
    /// Finite repetition; defaults to a single iteration.
    #[serde(default)]
    pub repeat: MacroRepeat,
    /// Optional associated stored profile; an advisory link, not execution.
    #[serde(default)]
    pub profile: Option<ProfileId>,
}

/// Condensed macro view for listings, without the full action sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacroSummary {
    /// Stable identifier.
    pub id: MacroId,
    /// Human-readable name.
    pub name: String,
    /// Number of actions in one iteration.
    pub action_count: usize,
    /// Optional associated stored profile.
    pub profile: Option<ProfileId>,
}

/// Versioned JSON envelope used for import/export and stored payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacroFile {
    /// Schema version of the enclosed macro.
    pub macro_schema_version: u16,
    /// The macro payload (wire key `macro`).
    #[serde(rename = "macro")]
    pub macro_value: Macro,
}

impl Macro {
    /// Validate every field and bound before persistence, export or import.
    pub fn validate(&self) -> Result<(), MacroError> {
        if self.name.is_empty()
            || self.name.len() > MAX_MACRO_NAME
            || self.name.chars().any(char::is_control)
        {
            return Err(MacroError::Invalid(
                "macro name must be 1..64 bytes without control characters",
            ));
        }
        if self.actions.is_empty() || self.actions.len() > MAX_MACRO_ACTIONS {
            return Err(MacroError::Invalid("macro must have 1..=4096 actions"));
        }
        for action in &self.actions {
            match action {
                MacroAction::Key { key, .. } => {
                    if !is_token(key) {
                        return Err(MacroError::Invalid("macro key must be a 1..128 byte token"));
                    }
                }
                MacroAction::Delay { ms } => {
                    if *ms == 0 || *ms > MAX_DELAY_MS {
                        return Err(MacroError::Invalid("macro delay must be 1..=60000 ms"));
                    }
                }
            }
        }
        if self.repeat.times == 0 || self.repeat.times > MAX_MACRO_REPEAT {
            return Err(MacroError::Invalid("macro repeat must be 1..=1000"));
        }
        if self.repeat.interval_ms > MAX_MACRO_INTERVAL_MS {
            return Err(MacroError::Invalid("macro interval must be 0..=60000 ms"));
        }
        if self.iteration_ms() > MAX_MACRO_ITERATION_MS {
            return Err(MacroError::Invalid(
                "macro iteration duration exceeds the limit",
            ));
        }
        if self.total_ms() > MAX_MACRO_TOTAL_MS {
            return Err(MacroError::Invalid(
                "macro total duration exceeds the limit",
            ));
        }
        Ok(())
    }

    /// Total delay duration of a single iteration, in milliseconds.
    pub fn iteration_ms(&self) -> u64 {
        self.actions
            .iter()
            .map(|action| match action {
                MacroAction::Delay { ms } => u64::from(*ms),
                MacroAction::Key { .. } => 0,
            })
            .sum()
    }

    /// Total repeated duration of the whole macro, in milliseconds (saturating).
    pub fn total_ms(&self) -> u64 {
        let times = u64::from(self.repeat.times);
        let interval = u64::from(self.repeat.interval_ms);
        self.iteration_ms()
            .saturating_mul(times)
            .saturating_add(interval.saturating_mul(times.saturating_sub(1)))
    }

    /// Condense to a listing summary.
    pub fn summary(&self) -> MacroSummary {
        MacroSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            action_count: self.actions.len(),
            profile: self.profile.clone(),
        }
    }

    /// Export the macro as a schema-versioned JSON document.
    pub fn to_json(&self) -> Result<String, MacroError> {
        self.validate()?;
        serde_json::to_string_pretty(&MacroFile::new(self.clone()))
            .map_err(|_| MacroError::Invalid("macro serialization failed"))
    }

    /// Parse and validate a schema-versioned JSON document.
    pub fn from_json(text: &str) -> Result<Self, MacroError> {
        let file: MacroFile = serde_json::from_str(text).map_err(|_| MacroError::InvalidJson)?;
        file.into_supported()
    }
}

impl MacroFile {
    /// Wrap a macro using the implemented schema version.
    pub fn new(macro_value: Macro) -> Self {
        Self {
            macro_schema_version: MACRO_SCHEMA_VERSION,
            macro_value,
        }
    }

    /// Extract the macro only when its schema version is understood.
    pub fn into_supported(self) -> Result<Macro, MacroError> {
        if self.macro_schema_version != MACRO_SCHEMA_VERSION {
            return Err(MacroError::UnsupportedVersion);
        }
        self.macro_value.validate()?;
        Ok(self.macro_value)
    }
}
