//! Typed W3C DTCG-compliant Design Token models for `egui-macview`.
//!
//! Grounded in Apple AppKit, macOS Human Interface Guidelines,
//! Apple Typography specifications, and CoreAnimation spring physics.

pub mod color;
pub mod dtcg;
pub mod metrics;
pub mod motion;
pub mod typography;

pub use color::{MacAppearanceColors, MacColorTokens};
pub use dtcg::{ColorValue, DimensionValue};
pub use metrics::{
    ButtonMetrics, ButtonSizeMetrics, FocusRingMetrics, MacControlMetrics, MacMetricsTokens,
    PopoverMetrics, SegmentedMetrics, SegmentedSizeMetrics, TextFieldMetrics, TextFieldSizeMetrics,
    ToolbarMetrics,
};
pub use motion::{MacMotionTokens, MotionPresetToken};
pub use typography::{
    sf_pro_tracking, CubicCoefficients, CubicFit, MacTypographyTokens, PolynomialSpec001,
    QuadraticCoefficients, QuadraticFit, TrackingPoint, TrackingTableEntry,
};

use serde::{Deserialize, Serialize};

/// Inner container grouping the token subcategories.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacViewTokens {
    pub metrics: MacMetricsTokens,
    pub color: MacColorTokens,
    pub typography: MacTypographyTokens,
    pub motion: MacMotionTokens,
}

/// Root token catalog deserialized from `tokens/design_tokens.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacDesignTokens {
    #[serde(rename = "$name", default)]
    pub name: String,
    #[serde(rename = "$version", default)]
    pub version: String,
    #[serde(rename = "$description", default)]
    pub description: String,
    pub macview: MacViewTokens,
}

impl MacDesignTokens {
    /// Loads default design tokens from embedded `tokens/design_tokens.json`.
    pub fn load_default() -> Self {
        Self::from_json(include_str!("../../tokens/design_tokens.json"))
            .expect("Failed to parse embedded tokens/design_tokens.json")
    }

    /// Deserializes `MacDesignTokens` from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[inline]
    pub fn control_metrics(&self) -> &MacControlMetrics {
        &self.macview.metrics.control
    }

    #[inline]
    pub fn toolbar_metrics(&self) -> &ToolbarMetrics {
        &self.macview.metrics.toolbar
    }

    #[inline]
    pub fn popover_metrics(&self) -> &PopoverMetrics {
        &self.macview.metrics.popover
    }

    #[inline]
    pub fn colors(&self) -> &MacColorTokens {
        &self.macview.color
    }

    #[inline]
    pub fn typography(&self) -> &MacTypographyTokens {
        &self.macview.typography
    }

    #[inline]
    pub fn motion(&self) -> &MacMotionTokens {
        &self.macview.motion
    }
}
