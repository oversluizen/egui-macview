//! macOS Semantic System Color Tokens (NSColor Aqua and DarkAqua).

use serde::{Deserialize, Serialize};
use super::dtcg::ColorValue;

/// Appearance semantic colors for light or dark mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacAppearanceColors {
    pub window_background: ColorValue,
    pub control_background: ColorValue,
    pub control: ColorValue,
    pub control_accent: ColorValue,
    pub selected_control: ColorValue,
    pub selected_control_inactive: ColorValue,
    pub separator: ColorValue,
    pub placeholder_text: ColorValue,
    pub keyboard_focus_indicator: ColorValue,
}

/// Semantic system color tokens for Aqua and DarkAqua.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacColorTokens {
    pub aqua: MacAppearanceColors,
    pub dark_aqua: MacAppearanceColors,
}
