//! Typed TOML manifest model, semantic validation and bounded report encoding.
//!
//! A manifest describes only operations already modeled by this interpreter. It
//! is untrusted input: it can never name a shell command, a raw transport call,
//! a firmware/bootloader operation or arbitrary bytes. The interpreter writes at
//! most one bounded output/feature HID report per typed `SetLighting` command.

use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

use peripheral_hub_device_sdk::{
    Capabilities, ContractVersion, DeviceError, DeviceId, DeviceMatchRule, HidReport,
    LightingCommand, LightingEffect, ReportKind, TransportLimits,
};
use serde::{Deserialize, Serialize};

use crate::ManifestError;

/// Manifest schema version implemented by this interpreter.
pub const MANIFEST_VERSION: u16 = 1;
/// Maximum encoded report payload, excluding the report ID.
pub const MAX_REPORT_BYTES: usize = 64;
/// Maximum declared report timeout, matching the SDK transport ceiling.
pub const MAX_TIMEOUT_MS: u32 = 30_000;
/// Maximum declared report rate.
pub const MAX_RATE_PER_SECOND: u16 = 1_000;

/// A declarative driver manifest: identity, capabilities and bounded operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Must equal [`MANIFEST_VERSION`].
    pub manifest_version: u16,
    /// Model and SDK declaration.
    pub model: ModelDeclaration,
    /// Explicit matching evidence; converted to [`DeviceMatchRule`].
    #[serde(rename = "match")]
    pub device_match: Vec<DeviceMatchRule>,
    /// Declared capabilities, reused from the SDK.
    pub capabilities: Capabilities,
    /// Modeled operations; at least one is required.
    pub operations: Vec<OperationDeclaration>,
}

/// Model/driver identity and the SDK contract it implements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelDeclaration {
    /// Stable model identifier token.
    pub id: DeviceId,
    /// Human-readable driver name.
    pub display_name: String,
    /// SDK contract major version implemented by the manifest.
    pub sdk_major: u16,
    /// SDK contract minor version implemented by the manifest.
    pub sdk_minor: u16,
}

impl ModelDeclaration {
    /// Contract version to declare to the SDK.
    pub fn contract_version(&self) -> ContractVersion {
        ContractVersion {
            major: self.sdk_major,
            minor: self.sdk_minor,
        }
    }
}

/// Modeled operation kinds. Only volatile lighting is executable today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    /// Volatile static lighting for one declared zone.
    SetLighting,
}

/// One modeled operation and its single bounded report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationDeclaration {
    /// Operation kind; must be a kind the interpreter models.
    pub kind: OperationKind,
    /// Declared lighting zone this operation applies to.
    pub zone: String,
    /// The bounded report encoding for this operation.
    pub report: ReportDeclaration,
}

/// A bounded output/feature report layout. Input reports cannot be written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportDeclaration {
    /// Report direction; `input` is rejected.
    pub kind: ReportKind,
    /// HID report ID, separate from the payload.
    pub report_id: u8,
    /// Exact payload length in bytes, excluding the report ID.
    pub length: usize,
    /// Operation deadline in milliseconds.
    pub timeout_ms: u32,
    /// Maximum executed operations per second.
    pub max_rate_per_second: u16,
    /// Byte layout; each entry sets exactly one byte.
    pub fields: Vec<FieldDeclaration>,
    /// Optional mapping of declared lighting effects to a byte.
    #[serde(default)]
    pub effect: Option<EffectEncoding>,
}

/// One encoded payload byte: a constant or a value from the typed command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldDeclaration {
    /// Byte offset within the payload.
    pub offset: usize,
    /// Constant byte value.
    #[serde(default)]
    pub fill: Option<u8>,
    /// Typed command source.
    #[serde(default)]
    pub source: Option<FieldSource>,
}

/// Typed command values a field may serialize.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldSource {
    /// Red channel.
    Red,
    /// Green channel.
    Green,
    /// Blue channel.
    Blue,
    /// Brightness percentage (requires a brightness value in the command).
    Brightness,
}

/// Mapping of every declared effect to one byte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectEncoding {
    /// Byte offset within the payload.
    pub offset: usize,
    /// One entry per declared effect; values must be unique.
    pub values: Vec<EffectValue>,
}

/// One declared effect and its encoded byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectValue {
    /// Declared lighting effect.
    pub effect: LightingEffect,
    /// Byte written for this effect.
    pub value: u8,
}

/// Parse TOML text into a validated manifest.
pub fn parse_manifest(text: &str) -> Result<Manifest, ManifestError> {
    let manifest: Manifest = toml::from_str(text)?;
    manifest.validate()?;
    Ok(manifest)
}

/// Read and validate a manifest file from disk.
pub fn load_manifest_file(path: &Path) -> Result<Manifest, ManifestError> {
    parse_manifest(&std::fs::read_to_string(path)?)
}

impl Manifest {
    /// Validate schema version, identity, capabilities and every operation.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.manifest_version != MANIFEST_VERSION {
            return Err(ManifestError::UnsupportedVersion {
                found: self.manifest_version,
                supported: MANIFEST_VERSION,
            });
        }
        if self.model.display_name.is_empty()
            || self.model.display_name.len() > 128
            || self.model.display_name.chars().any(char::is_control)
        {
            return Err(ManifestError::Invalid(
                "display name must be 1..128 bytes without control characters",
            ));
        }
        self.model.contract_version().ensure_supported()?;
        if self.device_match.is_empty() || self.device_match.len() > 256 {
            return Err(ManifestError::Invalid(
                "manifest requires 1..256 match rules",
            ));
        }
        for rule in &self.device_match {
            rule.validate()?;
        }
        self.capabilities.validate()?;
        if self.operations.is_empty() {
            return Err(ManifestError::Invalid(
                "manifest requires at least one operation",
            ));
        }
        let lighting = self.capabilities.lighting.as_ref();
        let mut zones = BTreeSet::new();
        for operation in &self.operations {
            match operation.kind {
                OperationKind::SetLighting => {
                    let lighting = lighting.ok_or(ManifestError::Invalid(
                        "set_lighting requires declared lighting capabilities",
                    ))?;
                    if !lighting.zones.contains(&operation.zone) {
                        return Err(ManifestError::Invalid(
                            "operation zone is not declared in capabilities",
                        ));
                    }
                    if !zones.insert(operation.zone.as_str()) {
                        return Err(ManifestError::Invalid("duplicate set_lighting zone"));
                    }
                    operation.report.validate(lighting, &lighting.effects)?;
                }
            }
        }
        Ok(())
    }
}

impl ReportDeclaration {
    /// Validate bounds, byte layout and effect mapping against capabilities.
    pub fn validate(
        &self,
        lighting: &peripheral_hub_device_sdk::LightingCapabilities,
        declared_effects: &[LightingEffect],
    ) -> Result<(), ManifestError> {
        if self.kind == ReportKind::Input {
            return Err(ManifestError::Invalid("input reports cannot be written"));
        }
        if self.length == 0 || self.length > MAX_REPORT_BYTES {
            return Err(ManifestError::Invalid("report length must be 1..64 bytes"));
        }
        if self.timeout_ms == 0 || self.timeout_ms > MAX_TIMEOUT_MS {
            return Err(ManifestError::Invalid("report timeout is out of range"));
        }
        if self.max_rate_per_second == 0 || self.max_rate_per_second > MAX_RATE_PER_SECOND {
            return Err(ManifestError::Invalid("report rate is out of range"));
        }
        let mut used = vec![false; self.length];
        let mut sources = BTreeSet::new();
        for field in &self.fields {
            if field.offset >= self.length {
                return Err(ManifestError::Invalid("field offset exceeds report length"));
            }
            if used[field.offset] {
                return Err(ManifestError::Invalid("report fields overlap"));
            }
            used[field.offset] = true;
            match (field.fill, field.source) {
                (Some(_), None) | (None, Some(_)) => {}
                _ => {
                    return Err(ManifestError::Invalid(
                        "field must set exactly one of fill or source",
                    ));
                }
            }
            if let Some(source) = field.source {
                if source == FieldSource::Brightness && !lighting.brightness {
                    return Err(ManifestError::Invalid(
                        "brightness field requires declared brightness support",
                    ));
                }
                if !sources.insert(source) {
                    return Err(ManifestError::Invalid("duplicate field source"));
                }
            }
        }
        let non_static = declared_effects
            .iter()
            .any(|effect| *effect != LightingEffect::Static);
        match &self.effect {
            Some(effect) => {
                if effect.offset >= self.length {
                    return Err(ManifestError::Invalid(
                        "effect offset exceeds report length",
                    ));
                }
                if used[effect.offset] {
                    return Err(ManifestError::Invalid("effect byte conflicts with a field"));
                }
                if effect.values.len() != declared_effects.len() {
                    return Err(ManifestError::Invalid(
                        "effect encoding must map every declared effect exactly once",
                    ));
                }
                let mut seen_effects = BTreeSet::new();
                let mut seen_values = BTreeSet::new();
                for entry in &effect.values {
                    if !declared_effects.contains(&entry.effect) {
                        return Err(ManifestError::Invalid(
                            "effect encoding names an undeclared effect",
                        ));
                    }
                    if !seen_effects.insert(entry.effect) || !seen_values.insert(entry.value) {
                        return Err(ManifestError::Invalid(
                            "effect encoding must be unique per effect and byte",
                        ));
                    }
                }
            }
            None if non_static => {
                return Err(ManifestError::Invalid(
                    "non-static effects require an effect encoding",
                ));
            }
            None => {}
        }
        Ok(())
    }

    /// Build the bounded report for a validated command. Performs no I/O.
    pub fn encode(&self, command: &LightingCommand) -> Result<HidReport, DeviceError> {
        let mut payload = vec![0u8; self.length];
        for field in &self.fields {
            let value = match (field.fill, field.source) {
                (Some(value), None) => value,
                (None, Some(source)) => match source {
                    FieldSource::Red => command.color.red,
                    FieldSource::Green => command.color.green,
                    FieldSource::Blue => command.color.blue,
                    FieldSource::Brightness => {
                        command.brightness.ok_or(DeviceError::InvalidArgument(
                            "brightness is required by this report encoding".into(),
                        ))?
                    }
                },
                _ => {
                    return Err(DeviceError::ProtocolError(
                        "declarative field encoding is invalid".into(),
                    ));
                }
            };
            *payload.get_mut(field.offset).ok_or_else(|| {
                DeviceError::ProtocolError("declarative field offset is invalid".into())
            })? = value;
        }
        if let Some(effect) = &self.effect {
            let value = effect
                .values
                .iter()
                .find(|entry| entry.effect == command.effect)
                .map(|entry| entry.value)
                .ok_or_else(|| {
                    DeviceError::ProtocolError("declarative effect encoding is missing".into())
                })?;
            *payload.get_mut(effect.offset).ok_or_else(|| {
                DeviceError::ProtocolError("declarative effect offset is invalid".into())
            })? = value;
        }
        Ok(HidReport {
            report_id: self.report_id,
            payload,
        })
    }

    /// Deadline for this report.
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(u64::from(self.timeout_ms))
    }

    /// Capacity available for this report direction, from transport limits.
    pub fn capacity(&self, limits: TransportLimits) -> usize {
        match self.kind {
            ReportKind::Input => limits.input_bytes,
            ReportKind::Output => limits.output_bytes,
            ReportKind::Feature => limits.feature_bytes,
        }
    }
}
