//! macOS Finder & AppKit Pro-Inspector Semantic Tag Label Color Tokens (`color.tag_labels`).

use super::dtcg::ColorValue;
use egui::Color32;
use serde::{Deserialize, Serialize};

/// Semantic color tokens for the 12-slot macOS Finder / AppKit tag color swatch picker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacTagLabelColors {
    /// Red tag swatch color (`#FF3B30`).
    pub red: ColorValue,
    /// Orange tag swatch color (`#FF9500`).
    pub orange: ColorValue,
    /// Brown tag swatch color (`#A2845E`).
    pub brown: ColorValue,
    /// Yellow tag swatch color (`#FFCC00`).
    pub yellow: ColorValue,
    /// Light green tag swatch color (`#34C759`).
    pub light_green: ColorValue,
    /// Dark green tag swatch color (`#248A3D`).
    pub dark_green: ColorValue,
    /// Cyan tag swatch color (`#32ADE6`).
    pub cyan: ColorValue,
    /// Blue tag swatch color (`#007AFF`).
    pub blue: ColorValue,
    /// Purple tag swatch color (`#AF52DE`).
    pub purple: ColorValue,
    /// Magenta / pink tag swatch color (`#FF2D55`).
    pub magenta: ColorValue,
    /// Neutral gray tag swatch color (`#8E8E93`).
    pub gray: ColorValue,
    /// Transparent fill for the clear (`None`) tag swatch (`rgba(0, 0, 0, 0.0)`).
    pub clear: ColorValue,
    /// Subtle hairline border around each swatch dot (`rgba(0, 0, 0, 0.18)`).
    pub swatch_border: ColorValue,
}

impl Default for MacTagLabelColors {
    fn default() -> Self {
        crate::tokens::MacDesignTokens::default_shared()
            .colors()
            .tag_labels
            .clone()
    }
}

impl MacTagLabelColors {
    /// Returns the 11 standard semantic color swatches (`Red` through `Gray`) as [`Color32`].
    pub fn palette_11(&self) -> [Color32; 11] {
        [
            self.red.to_color32(),
            self.orange.to_color32(),
            self.brown.to_color32(),
            self.yellow.to_color32(),
            self.light_green.to_color32(),
            self.dark_green.to_color32(),
            self.cyan.to_color32(),
            self.blue.to_color32(),
            self.purple.to_color32(),
            self.magenta.to_color32(),
            self.gray.to_color32(),
        ]
    }
}
