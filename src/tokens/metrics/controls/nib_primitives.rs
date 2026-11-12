//! AppKit NIB primitive control tokens (`MacCheckbox`, `MacLabel`, `MacHelpButton`, `MacInlineActionButton`).

use crate::tokens::dtcg::DimensionValue;
use crate::tokens::MacDesignTokens;
use serde::{Deserialize, Serialize};

/// Size-specific geometry tokens for `MacCheckbox` (`NSButtonTypeSwitch`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckboxSizeMetrics {
    /// Square checkbox indicator width/height in points (`14.0`, `12.0`, `10.0`).
    pub box_size: DimensionValue,
    /// Rounded squircle corner radius (`3.5`, `3.0`, `2.5`).
    pub corner_radius: DimensionValue,
    /// Horizontal gap between indicator box and label (`6.0`, `5.0`, `4.0`).
    pub label_gap: DimensionValue,
    /// Label font size in points (`13.0`, `11.0`, `9.0`).
    pub font_size: DimensionValue,
    /// Vector checkmark stroke width in points (`1.6`, `1.4`, `1.2`).
    pub checkmark_stroke: DimensionValue,
    /// AppKit `firstBaselineOffsetFromTop` in points (`13.0`, `11.0`, `9.0`).
    pub first_baseline_offset_from_top: DimensionValue,
}

/// Complete token set for `MacCheckbox` (`NSButtonTypeSwitch`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckboxMetrics {
    /// Regular size tier (`14x14pt` box, `13pt` font).
    pub regular: CheckboxSizeMetrics,
    /// Small size tier (`12x12pt` box, `11pt` font).
    pub small: CheckboxSizeMetrics,
    /// Mini size tier (`10x10pt` box, `9pt` font).
    pub mini: CheckboxSizeMetrics,
    /// Unchecked box border stroke width (`1.0pt`).
    pub border_width: DimensionValue,
    /// Checkmark start vertex normalized X (`0.24`).
    pub check_p1_x: DimensionValue,
    /// Checkmark start vertex normalized Y (`0.52`).
    pub check_p1_y: DimensionValue,
    /// Checkmark knee vertex normalized X (`0.43`).
    pub check_p2_x: DimensionValue,
    /// Checkmark knee vertex normalized Y (`0.72`).
    pub check_p2_y: DimensionValue,
    /// Checkmark end vertex normalized X (`0.76`).
    pub check_p3_x: DimensionValue,
    /// Checkmark end vertex normalized Y (`0.28`).
    pub check_p3_y: DimensionValue,
    /// Horizontal inset ratio for mixed-state (`-`) dash (`0.24`).
    pub mixed_bar_inset_ratio: DimensionValue,
}

impl Default for CheckboxMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().checkbox_metrics().clone()
    }
}

/// Typography and baseline metrics for `MacLabel` (`NSTextField.labelWithString:`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelMetrics {
    /// Regular font size (`13.0pt`).
    pub regular_font_size: DimensionValue,
    /// Regular intrinsic cell height (`16.0pt`).
    pub regular_height: DimensionValue,
    /// Regular `firstBaselineOffsetFromTop` (`13.0pt`).
    pub regular_baseline_from_top: DimensionValue,
    /// Regular `baselineOffsetFromBottom` (`3.0pt`).
    pub regular_baseline_from_bottom: DimensionValue,
    /// Regular `NSFont.ascender` (`12.57pt`).
    pub regular_ascender: DimensionValue,
    /// Regular `NSFont.descender` (`-2.74pt`).
    pub regular_descender: DimensionValue,
    /// Regular `NSFont.capHeight` (`9.16pt`).
    pub regular_cap_height: DimensionValue,
    /// Small font size (`11.0pt`).
    pub small_font_size: DimensionValue,
    /// Small intrinsic cell height (`14.0pt`).
    pub small_height: DimensionValue,
    /// Small `firstBaselineOffsetFromTop` (`11.0pt`).
    pub small_baseline_from_top: DimensionValue,
    /// Small `baselineOffsetFromBottom` (`3.0pt`).
    pub small_baseline_from_bottom: DimensionValue,
    /// Small `NSFont.ascender` (`10.63pt`).
    pub small_ascender: DimensionValue,
    /// Small `NSFont.descender` (`-2.32pt`).
    pub small_descender: DimensionValue,
    /// Small `NSFont.capHeight` (`7.75pt`).
    pub small_cap_height: DimensionValue,
    /// Mini font size (`9.0pt`).
    pub mini_font_size: DimensionValue,
    /// Mini intrinsic cell height (`11.0pt`).
    pub mini_height: DimensionValue,
    /// Mini `firstBaselineOffsetFromTop` (`10.0pt`).
    pub mini_baseline_from_top: DimensionValue,
    /// Mini `baselineOffsetFromBottom` (`2.0pt`).
    pub mini_baseline_from_bottom: DimensionValue,
    /// Mini `NSFont.ascender` (`8.70pt`).
    pub mini_ascender: DimensionValue,
    /// Mini `NSFont.descender` (`-1.90pt`).
    pub mini_descender: DimensionValue,
    /// Mini `NSFont.capHeight` (`6.34pt`).
    pub mini_cap_height: DimensionValue,
    /// Subpixel faux-bold offset for Semibold weight (`0.35pt`).
    pub semibold_stroke_boost: DimensionValue,
    /// Subpixel faux-bold offset for Bold weight (`0.55pt`).
    pub bold_stroke_boost: DimensionValue,
    /// WCAG 2.1 Level AA minimum contrast ratio for primary and secondary labels (`4.5`).
    pub wcag_aa_normal_ratio: DimensionValue,
    /// WCAG 2.1 Level AA minimum contrast ratio for tertiary and disabled labels (`3.0`).
    pub wcag_aa_large_ratio: DimensionValue,
    /// AppKit `PlusDarker` / `PlusLighter` surface delta gain factor (`1.0`).
    pub vibrancy_gain: DimensionValue,
}

impl Default for LabelMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().label_metrics().clone()
    }
}

/// Geometry tokens for `MacHelpButton` (`NSBezelStyleHelpButton`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HelpButtonMetrics {
    /// Regular circular diameter (`20.0pt`).
    pub diameter: DimensionValue,
    /// Small circular diameter (`16.0pt`).
    pub small_diameter: DimensionValue,
    /// Mini circular diameter (`13.0pt`).
    pub mini_diameter: DimensionValue,
    /// Regular question mark glyph font size (`12.0pt`).
    pub font_size: DimensionValue,
    /// Small question mark glyph font size (`10.0pt`).
    pub small_font_size: DimensionValue,
    /// Mini question mark glyph font size (`9.0pt`).
    pub mini_font_size: DimensionValue,
    /// Circular bezel border stroke width (`1.0pt`).
    pub border_width: DimensionValue,
    /// Subtle bottom drop-shadow offset (`1.0pt`).
    pub shadow_offset_y: DimensionValue,
}

impl Default for HelpButtonMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared()
            .help_button_metrics()
            .clone()
    }
}

/// Geometry tokens for `MacInlineActionButton` (`16x16pt` circular inline badge).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InlineActionButtonMetrics {
    /// Circular badge diameter (`16.0pt`).
    pub diameter: DimensionValue,
    /// Interior icon or symbol font size (`10.0pt`).
    pub icon_size: DimensionValue,
    /// Bezel border stroke width (`1.0pt`).
    pub border_width: DimensionValue,
    /// Vector icon stroke width (`1.3pt`).
    pub icon_stroke_width: DimensionValue,
}

impl Default for InlineActionButtonMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared()
            .inline_action_button_metrics()
            .clone()
    }
}
