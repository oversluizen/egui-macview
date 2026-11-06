//! Core W3C DTCG value wrapper primitives.

use egui::Color32;
use serde::{Deserialize, Serialize};

/// Generic DTCG dimension token representation supporting `"22.0pt"` or `{ "value": "22.0pt" }`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionValue {
    /// String representation of the dimension (e.g. `"22.0pt"`).
    #[serde(alias = "$value", alias = "value")]
    pub value: String,
    /// DTCG token type (defaults to `"dimension"`).
    #[serde(rename = "$type", default)]
    pub token_type: Option<String>,
    /// Optional human-readable description.
    #[serde(rename = "$description", default)]
    pub description: Option<String>,
}

impl DimensionValue {
    /// Constructs a `DimensionValue` from a point value (e.g. `22.0pt`).
    pub fn from_pt(pt: f32) -> Self {
        Self {
            value: format!("{:.1}pt", pt),
            token_type: Some("dimension".to_string()),
            description: None,
        }
    }

    /// Extracts numeric value in logical points (AppKit pt).
    pub fn points(&self) -> f32 {
        self.value
            .trim_end_matches("pt")
            .trim()
            .parse::<f32>()
            .unwrap_or(0.0)
    }
}

/// Generic DTCG color token representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorValue {
    /// String representation of color (`#RRGGBB`, `#RRGGBBAA`, or `rgba(...)`).
    #[serde(alias = "$value", alias = "value")]
    pub value: String,
    /// DTCG token type (defaults to `"color"`).
    #[serde(rename = "$type", default)]
    pub token_type: Option<String>,
    /// Optional human-readable description.
    #[serde(rename = "$description", default)]
    pub description: Option<String>,
}

impl ColorValue {
    /// Returns the underlying hex or rgba string slice.
    pub fn hex_or_rgba(&self) -> &str {
        &self.value
    }

    /// Converts DTCG color token string (`#RRGGBB`, `#RRGGBBAA`, or `rgba(r, g, b, a)`) to `egui::Color32`.
    pub fn to_color32(&self) -> Color32 {
        parse_color32(&self.value)
    }
}

/// Parses Hex or CSS `rgba(...)` string into `egui::Color32`.
pub fn parse_color32(s: &str) -> Color32 {
    let trimmed = s.trim();
    if let Some(hex) = trimmed.strip_prefix('#') {
        match hex.len() {
            6 => {
                if let Ok(v) = u32::from_str_radix(hex, 16) {
                    let r = ((v >> 16) & 0xFF) as u8;
                    let g = ((v >> 8) & 0xFF) as u8;
                    let b = (v & 0xFF) as u8;
                    return Color32::from_rgb(r, g, b);
                }
            }
            8 => {
                if let Ok(v) = u32::from_str_radix(hex, 16) {
                    let r = ((v >> 24) & 0xFF) as u8;
                    let g = ((v >> 16) & 0xFF) as u8;
                    let b = ((v >> 8) & 0xFF) as u8;
                    let a = (v & 0xFF) as u8;
                    return Color32::from_rgba_premultiplied(r, g, b, a);
                }
            }
            _ => {}
        }
    } else if trimmed.starts_with("rgba(") && trimmed.ends_with(')') {
        let inside = &trimmed[5..trimmed.len() - 1];
        let parts: Vec<&str> = inside.split(',').map(|p| p.trim()).collect();
        if parts.len() == 4 {
            let r = parts[0].parse::<u8>().unwrap_or(0);
            let g = parts[1].parse::<u8>().unwrap_or(0);
            let b = parts[2].parse::<u8>().unwrap_or(0);
            let a_f = parts[3].parse::<f32>().unwrap_or(1.0);
            let a = (a_f.clamp(0.0, 1.0) * 255.0).round() as u8;
            return Color32::from_rgba_premultiplied(r, g, b, a);
        }
    }
    Color32::BLACK
}
