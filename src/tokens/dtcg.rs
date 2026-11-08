//! Core W3C DTCG value wrapper primitives.

use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct RawDimensionValue {
    #[serde(alias = "$value", alias = "value")]
    value: String,
    #[serde(rename = "$type", default)]
    token_type: Option<String>,
    #[serde(rename = "$description", default)]
    description: Option<String>,
}

/// Generic DTCG dimension token representation supporting `"22.0pt"` or `{ "value": "22.0pt" }`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "RawDimensionValue")]
pub struct DimensionValue {
    /// String representation of the dimension (e.g. `"22.0pt"`).
    pub value: String,
    /// DTCG token type (defaults to `"dimension"`).
    #[serde(rename = "$type", default)]
    pub token_type: Option<String>,
    /// Optional human-readable description.
    #[serde(rename = "$description", default)]
    pub description: Option<String>,
    #[serde(skip)]
    cached_points: f32,
}

impl From<RawDimensionValue> for DimensionValue {
    fn from(raw: RawDimensionValue) -> Self {
        let cached_points = parse_points(&raw.value);
        Self {
            value: raw.value,
            token_type: raw.token_type,
            description: raw.description,
            cached_points,
        }
    }
}

impl DimensionValue {
    /// Constructs a `DimensionValue` from a point value (e.g. `22.0pt`).
    pub fn from_pt(pt: f32) -> Self {
        Self {
            value: format!("{:.1}pt", pt),
            token_type: Some("dimension".to_string()),
            description: None,
            cached_points: pt,
        }
    }

    /// Extracts numeric value in logical points (AppKit pt) in O(1) from the pre-parsed cache.
    #[inline]
    pub fn points(&self) -> f32 {
        self.cached_points
    }
}

/// Parses a DTCG dimension string (e.g. `"22.0pt"`) into logical points (`f32`).
pub fn parse_points(s: &str) -> f32 {
    s.trim_end_matches("pt")
        .trim()
        .parse::<f32>()
        .unwrap_or(0.0)
}

#[derive(Deserialize)]
struct RawColorValue {
    #[serde(alias = "$value", alias = "value")]
    value: String,
    #[serde(rename = "$type", default)]
    token_type: Option<String>,
    #[serde(rename = "$description", default)]
    description: Option<String>,
}

/// Generic DTCG color token representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "RawColorValue")]
pub struct ColorValue {
    /// String representation of color (`#RRGGBB`, `#RRGGBBAA`, or `rgba(...)`).
    pub value: String,
    /// DTCG token type (defaults to `"color"`).
    #[serde(rename = "$type", default)]
    pub token_type: Option<String>,
    /// Optional human-readable description.
    #[serde(rename = "$description", default)]
    pub description: Option<String>,
    #[serde(skip)]
    cached_color: Color32,
}

impl From<RawColorValue> for ColorValue {
    fn from(raw: RawColorValue) -> Self {
        let cached_color = parse_color32(&raw.value);
        Self {
            value: raw.value,
            token_type: raw.token_type,
            description: raw.description,
            cached_color,
        }
    }
}

impl ColorValue {
    /// Constructs a `ColorValue` from a hex or CSS `rgba(...)` string, pre-parsing `Color32`.
    pub fn new(val: &str) -> Self {
        Self {
            value: val.to_string(),
            token_type: Some("color".to_string()),
            description: None,
            cached_color: parse_color32(val),
        }
    }

    /// Returns the underlying hex or rgba string slice.
    #[inline]
    pub fn hex_or_rgba(&self) -> &str {
        &self.value
    }

    /// Returns the pre-parsed `egui::Color32` in O(1).
    #[inline]
    pub fn to_color32(&self) -> Color32 {
        self.cached_color
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
                    return Color32::from_rgba_unmultiplied(r, g, b, a);
                }
            }
            _ => {}
        }
    } else if let Some(inside) = trimmed
        .strip_prefix("rgba(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let mut parts = inside.split(',').map(str::trim);
        if let (Some(r_s), Some(g_s), Some(b_s), Some(a_s), None) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            let r = r_s.parse::<u8>().unwrap_or(0);
            let g = g_s.parse::<u8>().unwrap_or(0);
            let b = b_s.parse::<u8>().unwrap_or(0);
            let a_f = a_s.parse::<f32>().unwrap_or(1.0);
            let a = (a_f.clamp(0.0, 1.0) * 255.0).round() as u8;
            return Color32::from_rgba_unmultiplied(r, g, b, a);
        }
    }
    Color32::BLACK
}
