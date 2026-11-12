//! SearchField, Stepper, ColorWell, and Toggle geometry metrics tokens.

use crate::tokens::dtcg::DimensionValue;
use crate::tokens::MacDesignTokens;
use serde::{Deserialize, Serialize};

/// NSSearchField metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchFieldMetrics {
    /// Height of the search field.
    pub height: DimensionValue,
    /// Corner rounding radius (fully rounded pill).
    pub corner_radius: DimensionValue,
    /// Size of the magnifying glass search icon.
    pub icon_size: DimensionValue,
    /// Stroke width of the magnifying glass icon.
    pub icon_stroke_width: DimensionValue,
    /// Horizontal padding on left and right edges.
    pub padding_horizontal: DimensionValue,
    /// Size of the circular clear ('x') button.
    pub clear_button_size: DimensionValue,
    /// Default fallback width in points.
    pub default_width: DimensionValue,
    /// Padding inside the clear button circle.
    pub clear_button_icon_padding: DimensionValue,
    /// Spacing between search icon, text, and clear button.
    pub content_spacing: DimensionValue,
    /// Vertical padding within search text input.
    pub padding_vertical: DimensionValue,
    /// Border hairline stroke width.
    pub border_width: DimensionValue,
    /// Outer radius of magnifying glass lens.
    pub glass_radius: DimensionValue,
    /// Font size of search query text.
    pub font_size: DimensionValue,
}

impl Default for SearchFieldMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().search_field_metrics().clone()
    }
}

/// NSStepper metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepperMetrics {
    /// Width of the vertical stepper button.
    pub width: DimensionValue,
    /// Height of the vertical stepper button.
    pub height: DimensionValue,
    /// Corner rounding radius.
    pub corner_radius: DimensionValue,
    /// Thickness of horizontal divider dividing up/down halves.
    pub divider_thickness: DimensionValue,
    /// Span size of arrow indicator chevrons.
    pub arrow_size: DimensionValue,
    /// Stroke width of arrow chevrons.
    pub arrow_stroke_width: DimensionValue,
    /// Vertical offset delta of arrow chevrons from half centers.
    pub arrow_delta: DimensionValue,
    /// Outer border hairline stroke width.
    pub border_width: DimensionValue,
    /// Initial delay in seconds before hold-to-repeat starts (`0.4s`).
    pub auto_repeat_delay: DimensionValue,
    /// Repeat interval in seconds while stepper is held down (`0.075s`).
    pub auto_repeat_interval: DimensionValue,
    /// Step multiplier when Shift modifier is held (`10.0`).
    pub shift_multiplier: DimensionValue,
    /// Step multiplier when Command or Control modifier is held (`100.0`).
    pub cmd_multiplier: DimensionValue,
}

impl Default for StepperMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().stepper_metrics().clone()
    }
}

/// NSColorWell metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorWellMetrics {
    /// Width of the color well button.
    pub width: DimensionValue,
    /// Height of the color well button.
    pub height: DimensionValue,
    /// Outer corner rounding radius.
    pub corner_radius: DimensionValue,
    /// Outer border stroke thickness.
    pub border_width: DimensionValue,
    /// Inset from border to interior color swatch.
    pub swatch_inset: DimensionValue,
    /// Corner radius of the interior color swatch.
    pub swatch_radius: DimensionValue,
    /// Square grid cell size for alpha transparency checkerboard.
    pub checker_grid_size: DimensionValue,
    /// Dimension of popup color picker panel.
    pub picker_size: DimensionValue,
    /// Gap between well button and popup picker surface.
    pub popup_gap: DimensionValue,
    /// Corner radius of popup color picker panel.
    pub popup_corner_radius: DimensionValue,
}

impl Default for ColorWellMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().color_well_metrics().clone()
    }
}

/// NSSwitch / toggle metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleMetrics {
    /// Total width of the toggle capsule pill.
    pub width: DimensionValue,
    /// Total height of the toggle capsule pill.
    pub height: DimensionValue,
    /// Corner rounding radius (half of height for capsule ends).
    pub corner_radius: DimensionValue,
    /// Diameter of sliding circular knob.
    pub knob_size: DimensionValue,
    /// Knob horizontal inset in 'off' state.
    pub knob_offset_off: DimensionValue,
    /// Knob horizontal inset in 'on' state.
    pub knob_offset_on: DimensionValue,
    /// Outer border stroke width.
    pub border_width: DimensionValue,
    /// Animation duration for sliding knob transition in seconds.
    pub animation_duration: DimensionValue,
}

impl Default for ToggleMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().toggle_metrics().clone()
    }
}
