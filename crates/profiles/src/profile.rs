use std::fmt;

use peripheral_hub_device_sdk::{DeviceId, LightingCommand};
use serde::{Deserialize, Serialize};

use crate::ProfileError;

/// Profile interchange schema version, independent from the SDK contract.
pub const PROFILE_SCHEMA_VERSION: u16 = 1;

/// Stable, validated profile identifier token.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ProfileId(String);

impl ProfileId {
    /// Validate a nonempty, bounded ASCII token (no paths or whitespace).
    pub fn new(value: impl Into<String>) -> Result<Self, ProfileError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 128
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
        {
            return Err(ProfileError::Invalid(
                "profile id must be a 1..128 byte token",
            ));
        }
        Ok(Self(value))
    }

    /// Access the validated token.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ProfileId {
    type Error = ProfileError;
    fn try_from(value: String) -> Result<Self, ProfileError> {
        Self::new(value)
    }
}

impl From<ProfileId> for String {
    fn from(value: ProfileId) -> Self {
        value.0
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A user profile: a name plus optional model binding and lighting request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Stable identifier; saving an existing id replaces that profile.
    pub id: ProfileId,
    /// Human-readable name (1..64 bytes, no control characters).
    pub name: String,
    /// Optional bound device model; not a physical-unit identity.
    pub device_model: Option<DeviceId>,
    /// Optional typed lighting preference applied by supported devices.
    pub lighting: Option<LightingCommand>,
}

/// Condensed profile view for listings, without full payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileSummary {
    /// Stable identifier.
    pub id: ProfileId,
    /// Human-readable name.
    pub name: String,
    /// Optional bound device model.
    pub device_model: Option<DeviceId>,
}

/// Versioned JSON envelope used for import/export and stored payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileFile {
    /// Schema version of the enclosed profile.
    pub profile_schema_version: u16,
    /// The profile payload.
    pub profile: Profile,
}

impl Profile {
    /// Validate all fields before persistence or export.
    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.name.is_empty() || self.name.len() > 64 || self.name.chars().any(char::is_control) {
            return Err(ProfileError::Invalid(
                "profile name must be 1..64 bytes without control characters",
            ));
        }
        if let Some(lighting) = &self.lighting {
            DeviceId::new(lighting.zone.clone())
                .map_err(|_| ProfileError::Invalid("lighting zone is not a valid token"))?;
            if lighting
                .brightness
                .is_some_and(|brightness| brightness > 100)
            {
                return Err(ProfileError::Invalid("brightness must be 0..100"));
            }
        }
        Ok(())
    }

    /// Condense to a listing summary.
    pub fn summary(&self) -> ProfileSummary {
        ProfileSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            device_model: self.device_model.clone(),
        }
    }

    /// Export the profile as a schema-versioned JSON document.
    pub fn to_json(&self) -> Result<String, ProfileError> {
        self.validate()?;
        serde_json::to_string_pretty(&ProfileFile::new(self.clone()))
            .map_err(|_| ProfileError::Invalid("profile serialization failed"))
    }

    /// Parse and validate a schema-versioned JSON document.
    pub fn from_json(text: &str) -> Result<Self, ProfileError> {
        let file: ProfileFile =
            serde_json::from_str(text).map_err(|_| ProfileError::InvalidJson)?;
        file.into_supported()
    }
}

impl ProfileFile {
    /// Wrap a profile using the implemented schema version.
    pub fn new(profile: Profile) -> Self {
        Self {
            profile_schema_version: PROFILE_SCHEMA_VERSION,
            profile,
        }
    }

    /// Extract the profile only when its schema version is understood.
    pub fn into_supported(self) -> Result<Profile, ProfileError> {
        if self.profile_schema_version != PROFILE_SCHEMA_VERSION {
            return Err(ProfileError::UnsupportedVersion);
        }
        self.profile.validate()?;
        Ok(self.profile)
    }
}
