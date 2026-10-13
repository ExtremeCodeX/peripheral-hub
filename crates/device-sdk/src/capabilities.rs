use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{DeviceError, DeviceResult, error::invalid};

/// RGB channel values, independent from any proprietary packet format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RgbColor {
    /// Red channel (0..255).
    pub red: u8,
    /// Green channel (0..255).
    pub green: u8,
    /// Blue channel (0..255).
    pub blue: u8,
}

/// Generic lighting effects; drivers declare only those they can implement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LightingEffect {
    /// Constant color.
    Static,
    /// Pulsing brightness.
    Breathing,
    /// Device-native color cycling.
    ColorCycle,
}

/// Lighting support for explicit named zones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightingCapabilities {
    /// Nonempty unique zone tokens; whole-device support uses an explicit token.
    pub zones: Vec<String>,
    /// Whether an independent brightness percentage is supported.
    pub brightness: bool,
    /// Nonempty unique effect list.
    pub effects: Vec<LightingEffect>,
}

/// Generic frame format, not a transport packet encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PixelFormat {
    /// Three bytes per pixel.
    Rgb888,
    /// Two bytes per pixel; concrete byte order belongs in protocol documentation.
    Rgb565,
}

/// Display declaration only; rendering/transmission are later domain work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisplayCapabilities {
    /// Width in pixels (1..4096).
    pub width: u16,
    /// Height in pixels (1..4096).
    pub height: u16,
    /// Unique supported uncompressed frame formats.
    pub formats: Vec<PixelFormat>,
    /// Maximum frame rate (1..240).
    pub max_frames_per_second: u16,
    /// Maximum uncompressed frame bytes (at most 16 MiB).
    pub max_frame_bytes: u32,
}

/// Button support declaration, not a platform input-injection implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ButtonCapabilities {
    /// Number of addressable buttons (1..1024).
    pub count: u16,
    /// Whether evidence-backed device-side remapping is supported.
    pub remapping: bool,
}

/// Bounds for future macro editing/validation/simulation only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacroCapabilities {
    /// Maximum actions in an edited macro (1..4096).
    pub max_actions: u16,
    /// Maximum delay per action (1..60000 ms).
    pub max_delay_ms: u32,
}

/// Profile declaration; persistence is not implemented by the SDK.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileCapabilities {
    /// Number of modeled profiles (1..1024).
    pub max_profiles: u16,
}

/// Granular optional capabilities; the default means no supported operations.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    /// Supported lighting operations and bounds, if known.
    pub lighting: Option<LightingCapabilities>,
    /// Display declaration, if known.
    pub display: Option<DisplayCapabilities>,
    /// Button declaration, if known.
    pub buttons: Option<ButtonCapabilities>,
    /// Safe macro model bounds, if supported by later software.
    pub macros: Option<MacroCapabilities>,
    /// Profile declaration, if known.
    pub profiles: Option<ProfileCapabilities>,
}

/// Typed volatile lighting request; no raw report or permanent-memory command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightingCommand {
    /// Exact declared zone.
    pub zone: String,
    /// Generic RGB value.
    pub color: RgbColor,
    /// Optional brightness percentage (0..100).
    pub brightness: Option<u8>,
    /// Exact declared effect.
    pub effect: LightingEffect,
}

/// SDK 1.0 command vocabulary; later commands require contract version review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "operation",
    content = "parameters",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum DeviceCommand {
    /// Set volatile lighting only when supported and validated by a driver.
    SetLighting(LightingCommand),
}

impl Capabilities {
    /// Validate declarations before exposing them or accepting commands.
    pub fn validate(&self) -> DeviceResult<()> {
        if let Some(light) = &self.lighting {
            if light.zones.is_empty() || light.zones.len() > 256 {
                return Err(invalid("lighting requires 1..256 zones"));
            }
            let mut zones = BTreeSet::new();
            for zone in &light.zones {
                crate::DeviceId::new(zone.clone())?;
                if !zones.insert(zone) {
                    return Err(invalid("duplicate lighting zone"));
                }
            }
            if light.effects.is_empty()
                || light.effects.len() > 3
                || light.effects.iter().collect::<BTreeSet<_>>().len() != light.effects.len()
            {
                return Err(invalid("lighting effects must be nonempty and unique"));
            }
        }
        if let Some(display) = &self.display {
            if !(1..=4096).contains(&display.width)
                || !(1..=4096).contains(&display.height)
                || !(1..=240).contains(&display.max_frames_per_second)
                || display.max_frame_bytes == 0
                || display.max_frame_bytes > 16 * 1024 * 1024
                || display.formats.is_empty()
                || display.formats.len() > 2
                || display.formats.iter().collect::<BTreeSet<_>>().len() != display.formats.len()
            {
                return Err(invalid(
                    "display dimensions, formats, or bounds are invalid",
                ));
            }
            for format in &display.formats {
                let bytes_per_pixel = match format {
                    PixelFormat::Rgb888 => 3,
                    PixelFormat::Rgb565 => 2,
                };
                let bytes = u32::from(display.width) * u32::from(display.height) * bytes_per_pixel;
                if bytes > display.max_frame_bytes {
                    return Err(invalid("declared display format exceeds frame byte limit"));
                }
            }
        }
        if self
            .buttons
            .as_ref()
            .is_some_and(|b| !(1..=1024).contains(&b.count))
        {
            return Err(invalid("button count must be 1..1024"));
        }
        if self.macros.as_ref().is_some_and(|m| {
            !(1..=4096).contains(&m.max_actions) || !(1..=60000).contains(&m.max_delay_ms)
        }) {
            return Err(invalid("macro bounds are invalid"));
        }
        if self
            .profiles
            .as_ref()
            .is_some_and(|p| !(1..=1024).contains(&p.max_profiles))
        {
            return Err(invalid("profile count must be 1..1024"));
        }
        Ok(())
    }

    /// Gate a typed request before a driver executes it. Does not perform I/O.
    pub fn validate_command(&self, command: &DeviceCommand) -> DeviceResult<()> {
        self.validate()?;
        match command {
            DeviceCommand::SetLighting(request) => {
                let light = self.lighting.as_ref().ok_or(DeviceError::NotSupported)?;
                if !light.zones.contains(&request.zone) {
                    return Err(invalid("lighting zone is not declared"));
                }
                if !light.effects.contains(&request.effect) {
                    return Err(DeviceError::NotSupported);
                }
                if let Some(brightness) = request.brightness {
                    if !light.brightness {
                        return Err(DeviceError::NotSupported);
                    }
                    if brightness > 100 {
                        return Err(invalid("brightness must be 0..100"));
                    }
                }
                Ok(())
            }
        }
    }
}
