//! Presets, Profile Management, and Shareable Codes

use serde::{Deserialize, Serialize};
use crate::color_engine::DaltonismMode;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CrosshairConfig {
    pub enabled: bool,
    pub style: CrosshairStyle,
    pub size: f32,          // Length / radius
    pub thickness: f32,     // Thickness in subpixels
    pub gap: f32,           // Center gap
    pub dot: bool,          // Center dot
    pub dot_size: f32,      // Dot radius
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    pub color_a: u8,
    pub outline: bool,
    pub outline_thickness: f32,
    pub outline_color_r: u8,
    pub outline_color_g: u8,
    pub outline_color_b: u8,
}

impl Default for CrosshairConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            style: CrosshairStyle::Cross,
            size: 4.0,
            thickness: 1.5,
            gap: 2.0,
            dot: false,
            dot_size: 1.5,
            color_r: 0,
            color_g: 255,
            color_b: 255, // Radiant Cyan
            color_a: 255,
            outline: true,
            outline_thickness: 1.0,
            outline_color_r: 0,
            outline_color_g: 0,
            outline_color_b: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum CrosshairStyle {
    Cross,
    Dot,
    Circle,
    TStyle,
    Diamond,
    Chevron,
    Box,
}

impl CrosshairStyle {
    pub fn all() -> &'static [CrosshairStyle] {
        &[
            CrosshairStyle::Cross,
            CrosshairStyle::Dot,
            CrosshairStyle::Circle,
            CrosshairStyle::TStyle,
            CrosshairStyle::Diamond,
            CrosshairStyle::Chevron,
            CrosshairStyle::Box,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            CrosshairStyle::Cross => "Cross (+)",
            CrosshairStyle::Dot => "Subpixel Dot (•)",
            CrosshairStyle::Circle => "Anti-Aliased Ring (○)",
            CrosshairStyle::TStyle => "T-Shape (T)",
            CrosshairStyle::Diamond => "Diamond (◇)",
            CrosshairStyle::Chevron => "Chevron Apex (^)",
            CrosshairStyle::Box => "Demon1 Box (□)",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DisplayProfile {
    pub id: String,
    pub name: String,
    pub process_target: String, // Comma separated or single process name

    // Color controls
    pub saturation: f32,        // 0.0 to 3.0 (1.0 = 100%, 3.0 = 300%)
    pub digital_vibrance: f32,  // 0.0 to 1.0
    pub brightness: f32,        // -0.5 to 0.5
    pub contrast: f32,          // 0.5 to 2.0
    pub red_gain: f32,          // 0.5 to 2.0
    pub green_gain: f32,        // 0.5 to 2.0
    pub blue_gain: f32,         // 0.5 to 2.0
    pub daltonism: DaltonismMode,

    // Gamma & Shadow Lift (Black Equalizer)
    pub gamma: f32,             // 0.5 to 2.5 (1.0 = normal)
    pub black_equalizer: f32,   // 0.0 to 1.0 (0.0 = off, 1.0 = max shadow lift)

    // Crosshair overlay
    pub crosshair: CrosshairConfig,
}

impl DisplayProfile {
    pub fn neutral() -> Self {
        Self {
            id: "neutral".into(),
            name: "Desktop (Neutral Calibrated)".into(),
            process_target: "".into(),
            saturation: 1.0,
            digital_vibrance: 0.0,
            brightness: 0.0,
            contrast: 1.0,
            red_gain: 1.0,
            green_gain: 1.0,
            blue_gain: 1.0,
            daltonism: DaltonismMode::None,
            gamma: 1.0,
            black_equalizer: 0.0,
            crosshair: CrosshairConfig::default(),
        }
    }

    /// 1. Valorant: Optimal Tournament-Clean Profile
    pub fn valorant() -> Self {
        Self {
            id: "valorant".into(),
            name: "Valorant (Tournament Clean)".into(),
            process_target: "valorant-win64-shipping.exe".into(),
            saturation: 1.35,
            digital_vibrance: 0.00,
            brightness: 0.00,
            contrast: 1.10,
            red_gain: 1.00,
            green_gain: 1.00,
            blue_gain: 1.00,
            daltonism: DaltonismMode::None,
            gamma: 1.00,
            black_equalizer: 0.15,
            crosshair: CrosshairConfig {
                enabled: false,
                style: CrosshairStyle::Dot,
                size: 2.5,
                thickness: 1.5,
                gap: 0.0,
                dot: true,
                dot_size: 2.0,
                color_r: 0,
                color_g: 255,
                color_b: 255, // Radiant Cyan
                color_a: 255,
                outline: true,
                outline_thickness: 1.0,
                outline_color_r: 0,
                outline_color_g: 0,
                outline_color_b: 0,
            },
        }
    }

    /// 2. Overwatch 2: Anti-Particle Clutter & Red Silhouette Boost
    pub fn overwatch2() -> Self {
        Self {
            id: "overwatch2".into(),
            name: "Overwatch 2 (Anti-Particle Silhouette)".into(),
            process_target: "overwatch.exe".into(),
            saturation: 1.50,
            digital_vibrance: 0.15,
            brightness: 0.00,
            contrast: 1.15,
            red_gain: 1.15,
            green_gain: 1.00,
            blue_gain: 0.90,
            daltonism: DaltonismMode::Protanopia,
            gamma: 0.98,
            black_equalizer: 0.20,
            crosshair: CrosshairConfig {
                enabled: false,
                style: CrosshairStyle::Circle,
                size: 4.5,
                thickness: 1.5,
                gap: 2.0,
                dot: true,
                dot_size: 1.5,
                color_r: 0,
                color_g: 255,
                color_b: 128, // Neon Green
                color_a: 255,
                outline: true,
                outline_thickness: 1.0,
                outline_color_r: 0,
                outline_color_g: 0,
                outline_color_b: 0,
            },
        }
    }

    /// 3. Marvel Rivals: UE5 Destruction Piercing & Aerial Tracker
    pub fn marvel_rivals() -> Self {
        Self {
            id: "marvel_rivals".into(),
            name: "Marvel Rivals (UE5 Debris & Sky Tracker)".into(),
            process_target: "marvel-win64-shipping.exe,marvelrivals.exe".into(),
            saturation: 1.60,
            digital_vibrance: 0.20,
            brightness: 0.02,
            contrast: 1.18,
            red_gain: 1.10,
            green_gain: 1.05,
            blue_gain: 0.95,
            daltonism: DaltonismMode::PvPHighVis,
            gamma: 0.94,
            black_equalizer: 0.35,
            crosshair: CrosshairConfig {
                enabled: false,
                style: CrosshairStyle::Diamond,
                size: 4.0,
                thickness: 1.5,
                gap: 1.5,
                dot: true,
                dot_size: 1.5,
                color_r: 255,
                color_g: 0,
                color_b: 180, // Magenta
                color_a: 255,
                outline: true,
                outline_thickness: 1.0,
                outline_color_r: 0,
                outline_color_g: 0,
                outline_color_b: 0,
            },
        }
    }

    /// Check if a process matches this profile's targets
    pub fn matches_process(&self, process_name: &str) -> bool {
        if self.process_target.is_empty() {
            return false;
        }
        for target in self.process_target.split(',') {
            let t = target.trim();
            if !t.is_empty() && process_name.eq_ignore_ascii_case(t) {
                return true;
            }
        }
        false
    }

    /// Export profile to base64 share code
    pub fn to_share_code(&self) -> Result<String, String> {
        let json = serde_json::to_string(self).map_err(|e| e.to_string())?;
        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.encode(json))
    }

    /// Import profile from base64 share code
    pub fn from_share_code(code: &str) -> Result<Self, String> {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(code.trim())
            .map_err(|e| format!("Invalid base64: {}", e))?;
        let json = String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8: {}", e))?;
        serde_json::from_str(&json).map_err(|e| format!("Invalid profile JSON: {}", e))
    }
}
