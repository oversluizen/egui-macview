//! SearchField, Stepper, ColorWell, and Toggle geometry metrics tokens.

use crate::tokens::dtcg::DimensionValue;
use crate::tokens::metrics::controls::basic::default_border_width;
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
    #[serde(default = "default_search_icon_stroke_width")]
    pub icon_stroke_width: DimensionValue,
    /// Horizontal padding on left and right edges.
    pub padding_horizontal: DimensionValue,
    /// Size of the circular clear ('x') button.
    pub clear_button_size: DimensionValue,
    /// Default fallback width in points.
    #[serde(default = "default_search_field_default_width")]
    pub default_width: DimensionValue,
    /// Padding inside the clear button circle.
    #[serde(default = "default_search_clear_icon_padding")]
    pub clear_button_icon_padding: DimensionValue,
    /// Spacing between search icon, text, and clear button.
    #[serde(default = "default_search_content_spacing")]
    pub content_spacing: DimensionValue,
    /// Vertical padding within search text input.
    #[serde(default = "default_search_padding_vertical")]
    pub padding_vertical: DimensionValue,
    /// Border hairline stroke width.
    #[serde(default = "default_border_width")]
    pub border_width: DimensionValue,
    /// Outer radius of magnifying glass lens.
    #[serde(default = "default_search_glass_radius")]
    pub glass_radius: DimensionValue,
    /// Font size of search query text.
    #[serde(default = "default_search_font_size")]
    pub font_size: DimensionValue,
}

fn default_search_icon_stroke_width() -> DimensionValue {
    DimensionValue::from_pt(1.2)
}

fn default_search_field_default_width() -> DimensionValue {
    DimensionValue::from_pt(160.0)
}

fn default_search_clear_icon_padding() -> DimensionValue {
    DimensionValue::from_pt(3.5)
}

fn default_search_content_spacing() -> DimensionValue {
    DimensionValue::from_pt(4.0)
}

fn default_search_padding_vertical() -> DimensionValue {
    DimensionValue::from_pt(2.0)
}

fn default_search_glass_radius() -> DimensionValue {
    DimensionValue::from_pt(3.0)
}

fn default_search_font_size() -> DimensionValue {
    DimensionValue::from_pt(12.0)
}

impl Default for SearchFieldMetrics {
    fn default() -> Self {
        Self {
            height: DimensionValue::from_pt(22.0),
            corner_radius: DimensionValue::from_pt(6.0),
            icon_size: DimensionValue::from_pt(12.0),
            icon_stroke_width: default_search_icon_stroke_width(),
            padding_horizontal: DimensionValue::from_pt(8.0),
            clear_button_size: DimensionValue::from_pt(14.0),
            default_width: default_search_field_default_width(),
            clear_button_icon_padding: default_search_clear_icon_padding(),
            content_spacing: default_search_content_spacing(),
            padding_vertical: default_search_padding_vertical(),
            border_width: default_border_width(),
            glass_radius: default_search_glass_radius(),
            font_size: default_search_font_size(),
        }
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
    #[serde(default = "default_stepper_divider_thickness")]
    pub divider_thickness: DimensionValue,
    /// Span size of arrow indicator chevrons.
    pub arrow_size: DimensionValue,
    /// Stroke width of arrow chevrons.
    #[serde(default = "default_stepper_arrow_stroke_width")]
    pub arrow_stroke_width: DimensionValue,
    /// Vertical offset delta of arrow chevrons from half centers.
    #[serde(default = "default_stepper_arrow_delta")]
    pub arrow_delta: DimensionValue,
    /// Outer border hairline stroke width.
    #[serde(default = "default_border_width")]
    pub border_width: DimensionValue,
    /// Initial delay in seconds before hold-to-repeat starts (`0.4s`).
    #[serde(default = "default_stepper_auto_repeat_delay")]
    pub auto_repeat_delay: DimensionValue,
    /// Repeat interval in seconds while stepper is held down (`0.075s`).
    #[serde(default = "default_stepper_auto_repeat_interval")]
    pub auto_repeat_interval: DimensionValue,
    /// Step multiplier when Shift modifier is held (`10.0`).
    #[serde(default = "default_stepper_shift_multiplier")]
    pub shift_multiplier: DimensionValue,
    /// Step multiplier when Command or Control modifier is held (`100.0`).
    #[serde(default = "default_stepper_cmd_multiplier")]
    pub cmd_multiplier: DimensionValue,
}

fn default_stepper_auto_repeat_delay() -> DimensionValue {
    DimensionValue::from_pt(0.4)
}

fn default_stepper_auto_repeat_interval() -> DimensionValue {
    DimensionValue::from_pt(0.075)
}

fn default_stepper_shift_multiplier() -> DimensionValue {
    DimensionValue::from_pt(10.0)
}

fn default_stepper_cmd_multiplier() -> DimensionValue {
    DimensionValue::from_pt(100.0)
}

fn default_stepper_divider_thickness() -> DimensionValue {
    DimensionValue::from_pt(1.0)
}

fn default_stepper_arrow_stroke_width() -> DimensionValue {
    DimensionValue::from_pt(1.2)
}

fn default_stepper_arrow_delta() -> DimensionValue {
    DimensionValue::from_pt(1.5)
}

impl Default for StepperMetrics {
    fn default() -> Self {
        Self {
            width: DimensionValue::from_pt(14.0),
            height: DimensionValue::from_pt(22.0),
            corner_radius: DimensionValue::from_pt(4.0),
            divider_thickness: default_stepper_divider_thickness(),
            arrow_size: DimensionValue::from_pt(6.0),
            arrow_stroke_width: default_stepper_arrow_stroke_width(),
            arrow_delta: default_stepper_arrow_delta(),
            border_width: default_border_width(),
            auto_repeat_delay: default_stepper_auto_repeat_delay(),
            auto_repeat_interval: default_stepper_auto_repeat_interval(),
            shift_multiplier: default_stepper_shift_multiplier(),
            cmd_multiplier: default_stepper_cmd_multiplier(),
        }
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
    #[serde(default = "default_color_well_checker_grid_size")]
    pub checker_grid_size: DimensionValue,
    /// Dimension of popup color picker panel.
    #[serde(default = "default_color_picker_size")]
    pub picker_size: DimensionValue,
    /// Gap between well button and popup picker surface.
    #[serde(default = "default_color_well_popup_gap")]
    pub popup_gap: DimensionValue,
    /// Corner radius of popup color picker panel.
    #[serde(default = "default_color_well_popup_corner_radius")]
    pub popup_corner_radius: DimensionValue,
}

fn default_color_well_checker_grid_size() -> DimensionValue {
    DimensionValue::from_pt(4.0)
}

fn default_color_picker_size() -> DimensionValue {
    DimensionValue::from_pt(220.0)
}

fn default_color_well_popup_gap() -> DimensionValue {
    DimensionValue::from_pt(4.0)
}

fn default_color_well_popup_corner_radius() -> DimensionValue {
    DimensionValue::from_pt(8.0)
}

impl Default for ColorWellMetrics {
    fn default() -> Self {
        Self {
            width: DimensionValue::from_pt(44.0),
            height: DimensionValue::from_pt(23.0),
            corner_radius: DimensionValue::from_pt(5.0),
            border_width: DimensionValue::from_pt(1.0),
            swatch_inset: DimensionValue::from_pt(3.0),
            swatch_radius: DimensionValue::from_pt(3.0),
            checker_grid_size: default_color_well_checker_grid_size(),
            picker_size: default_color_picker_size(),
            popup_gap: default_color_well_popup_gap(),
            popup_corner_radius: default_color_well_popup_corner_radius(),
        }
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
    #[serde(default = "default_border_width")]
    pub border_width: DimensionValue,
    /// Animation duration for sliding knob transition in seconds.
    #[serde(default = "default_toggle_animation_duration")]
    pub animation_duration: DimensionValue,
}

fn default_toggle_animation_duration() -> DimensionValue {
    DimensionValue::from_pt(0.20)
}

impl Default for ToggleMetrics {
    fn default() -> Self {
        Self {
            width: DimensionValue::from_pt(26.0),
            height: DimensionValue::from_pt(15.0),
            corner_radius: DimensionValue::from_pt(7.5),
            knob_size: DimensionValue::from_pt(13.0),
            knob_offset_off: DimensionValue::from_pt(1.0),
            knob_offset_on: DimensionValue::from_pt(12.0),
            border_width: default_border_width(),
            animation_duration: default_toggle_animation_duration(),
        }
    }
}
