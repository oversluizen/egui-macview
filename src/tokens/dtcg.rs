//! Core W3C DTCG value wrapper primitives.

use serde::{Deserialize, Serialize};

/// Generic DTCG dimension token representation supporting `"22.0pt"` or `{ "value": "22.0pt" }`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionValue {
    #[serde(alias = "$value", alias = "value")]
    pub value: String,
    #[serde(rename = "$type", default)]
    pub token_type: Option<String>,
    #[serde(rename = "$description", default)]
    pub description: Option<String>,
}

impl DimensionValue {
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
    #[serde(alias = "$value", alias = "value")]
    pub value: String,
    #[serde(rename = "$type", default)]
    pub token_type: Option<String>,
    #[serde(rename = "$description", default)]
    pub description: Option<String>,
}

impl ColorValue {
    pub fn hex_or_rgba(&self) -> &str {
        &self.value
    }
}
