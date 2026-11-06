//! Extended control geometry metrics tokens (slider, thumbnail, outline, popup, search, stepper, color well, toggle).

use crate::tokens::dtcg::DimensionValue;
use crate::tokens::metrics::controls::basic::default_border_width;
use serde::{Deserialize, Serialize};

/// NSSlider metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SliderMetrics {
    /// Height of the slider groove track.
    pub track_height: DimensionValue,
    /// Diameter of the circular thumb knob.
    pub thumb_size: DimensionValue,
    /// Stroke width of the thumb knob border.
    #[serde(default = "default_slider_thumb_stroke_width")]
    pub thumb_stroke_width: DimensionValue,
    /// Height of track tick mark indicators.
    pub tick_height: DimensionValue,
    /// Width/thickness of tick mark lines.
    pub tick_width: DimensionValue,
    /// Vertical distance from slider track to tick marks.
    pub tick_offset: DimensionValue,
    /// Default fallback width in points.
    #[serde(default = "default_slider_default_width")]
    pub default_width: DimensionValue,
    /// Step fraction traversed by keyboard arrow key presses.
    #[serde(default = "default_slider_key_step_fraction")]
    pub key_step_fraction: DimensionValue,
    /// Vertical offset of the thumb ambient drop shadow.
    #[serde(default = "default_slider_shadow_offset_y")]
    pub shadow_offset_y: DimensionValue,
}

fn default_slider_thumb_stroke_width() -> DimensionValue {
    DimensionValue::from_pt(1.0)
}

fn default_slider_shadow_offset_y() -> DimensionValue {
    DimensionValue::from_pt(1.5)
}

fn default_slider_default_width() -> DimensionValue {
    DimensionValue::from_pt(120.0)
}

fn default_slider_key_step_fraction() -> DimensionValue {
    DimensionValue::from_pt(0.05)
}

impl Default for SliderMetrics {
    fn default() -> Self {
        Self {
            track_height: DimensionValue::from_pt(3.0),
            thumb_size: DimensionValue::from_pt(16.0),
            thumb_stroke_width: default_slider_thumb_stroke_width(),
            tick_height: DimensionValue::from_pt(4.0),
            tick_width: DimensionValue::from_pt(1.0),
            tick_offset: DimensionValue::from_pt(3.0),
            default_width: default_slider_default_width(),
            key_step_fraction: default_slider_key_step_fraction(),
            shadow_offset_y: default_slider_shadow_offset_y(),
        }
    }
}

/// NSImageCell / thumbnail preview canvas metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThumbnailCellMetrics {
    /// Default width and height dimension in points (24x24 pt).
    pub size: DimensionValue,
    /// Corner rounding radius of the thumbnail preview canvas.
    pub corner_radius: DimensionValue,
    /// Outer border stroke thickness.
    pub border_width: DimensionValue,
}

impl Default for ThumbnailCellMetrics {
    fn default() -> Self {
        Self {
            size: DimensionValue::from_pt(24.0),
            corner_radius: DimensionValue::from_pt(4.0),
            border_width: DimensionValue::from_pt(1.0),
        }
    }
}

/// NSOutlineView metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutlineViewMetrics {
    /// Height of each outline row in points.
    pub row_height: DimensionValue,
    /// Horizontal indentation distance per depth level.
    pub indent_step: DimensionValue,
    /// Dimension of the disclosure chevron button area.
    pub disclosure_size: DimensionValue,
    /// Margin between disclosure chevron and row edge.
    pub disclosure_margin: DimensionValue,
    /// Spacing between items inside the row.
    pub content_spacing: DimensionValue,
    /// Size of trailing accessory button icons.
    pub accessory_size: DimensionValue,
    /// Spacing between adjacent trailing accessory icons.
    pub accessory_spacing: DimensionValue,
    /// Corner radius of the row selection highlight pill.
    pub corner_radius: DimensionValue,
    /// Font size of primary row label text.
    pub font_size: DimensionValue,
    /// Stroke width of the vector disclosure chevron.
    #[serde(default = "default_outline_chevron_stroke_width")]
    pub chevron_stroke_width: DimensionValue,
    /// Span size of the disclosure chevron legs.
    #[serde(default = "default_outline_chevron_size")]
    pub chevron_size: DimensionValue,
    /// Forward depth of the disclosure chevron arrow tip.
    #[serde(default = "default_outline_chevron_depth")]
    pub chevron_depth: DimensionValue,
    /// Spacing between adjacent outline rows.
    #[serde(default = "default_outline_item_spacing")]
    pub item_spacing: DimensionValue,
    /// Outer padding around outline view content.
    #[serde(default = "default_outline_edge_padding")]
    pub edge_padding: DimensionValue,
    /// Font size for section headers (11.0pt AppKit Small Caps).
    #[serde(default = "default_outline_header_font_size")]
    pub header_font_size: DimensionValue,
}

fn default_outline_header_font_size() -> DimensionValue {
    DimensionValue::from_pt(11.0)
}

fn default_outline_chevron_stroke_width() -> DimensionValue {
    DimensionValue::from_pt(1.2)
}

fn default_outline_chevron_size() -> DimensionValue {
    DimensionValue::from_pt(7.0)
}

fn default_outline_chevron_depth() -> DimensionValue {
    DimensionValue::from_pt(1.75)
}

fn default_outline_item_spacing() -> DimensionValue {
    DimensionValue::from_pt(6.0)
}

fn default_outline_edge_padding() -> DimensionValue {
    DimensionValue::from_pt(4.0)
}

impl Default for OutlineViewMetrics {
    fn default() -> Self {
        Self {
            row_height: DimensionValue::from_pt(24.0),
            indent_step: DimensionValue::from_pt(16.0),
            disclosure_size: DimensionValue::from_pt(10.0),
            disclosure_margin: DimensionValue::from_pt(4.0),
            content_spacing: DimensionValue::from_pt(6.0),
            accessory_size: DimensionValue::from_pt(16.0),
            accessory_spacing: DimensionValue::from_pt(4.0),
            corner_radius: DimensionValue::from_pt(4.0),
            font_size: DimensionValue::from_pt(12.0),
            chevron_stroke_width: default_outline_chevron_stroke_width(),
            chevron_size: default_outline_chevron_size(),
            chevron_depth: default_outline_chevron_depth(),
            item_spacing: default_outline_item_spacing(),
            edge_padding: default_outline_edge_padding(),
            header_font_size: default_outline_header_font_size(),
        }
    }
}

/// NSPopUpButton metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopUpButtonMetrics {
    /// Height of the pop-up button.
    pub height: DimensionValue,
    /// Corner rounding radius.
    pub corner_radius: DimensionValue,
    /// Horizontal content padding.
    pub padding_horizontal: DimensionValue,
    /// Size of the dual stacked chevron indicators.
    pub chevron_size: DimensionValue,
    /// Stroke width of the chevron arrows.
    #[serde(default = "default_popup_chevron_stroke_width")]
    pub chevron_stroke_width: DimensionValue,
    /// Spacing between label text and chevron arrows.
    #[serde(default = "default_popup_chevron_spacing")]
    pub chevron_spacing: DimensionValue,
    /// Font size of selected item label.
    #[serde(default = "default_popup_font_size")]
    pub font_size: DimensionValue,
    /// Border hairline stroke width.
    #[serde(default = "default_border_width")]
    pub border_width: DimensionValue,
    /// Vertical offset of chevrons from center.
    #[serde(default = "default_popup_chevron_offset_y")]
    pub chevron_offset_y: DimensionValue,
    /// Height of each dropdown menu item.
    #[serde(default = "default_popup_menu_item_height")]
    pub menu_item_height: DimensionValue,
    /// Padding around menu item text.
    #[serde(default = "default_popup_menu_item_padding")]
    pub menu_item_padding: DimensionValue,
    /// Minimum width of the popup menu panel.
    #[serde(default = "default_popup_menu_min_width")]
    pub menu_min_width: DimensionValue,
    /// Gap between popup button and menu surface.
    #[serde(default = "default_popup_gap")]
    pub popup_gap: DimensionValue,
}

fn default_popup_chevron_stroke_width() -> DimensionValue {
    DimensionValue::from_pt(1.2)
}

fn default_popup_chevron_spacing() -> DimensionValue {
    DimensionValue::from_pt(6.0)
}

fn default_popup_font_size() -> DimensionValue {
    DimensionValue::from_pt(13.0)
}

fn default_popup_chevron_offset_y() -> DimensionValue {
    DimensionValue::from_pt(2.0)
}

fn default_popup_menu_item_height() -> DimensionValue {
    DimensionValue::from_pt(20.0)
}

fn default_popup_menu_item_padding() -> DimensionValue {
    DimensionValue::from_pt(6.0)
}

fn default_popup_menu_min_width() -> DimensionValue {
    DimensionValue::from_pt(120.0)
}

fn default_popup_gap() -> DimensionValue {
    DimensionValue::from_pt(2.0)
}

impl Default for PopUpButtonMetrics {
    fn default() -> Self {
        Self {
            height: DimensionValue::from_pt(22.0),
            corner_radius: DimensionValue::from_pt(5.0),
            padding_horizontal: DimensionValue::from_pt(8.0),
            chevron_size: DimensionValue::from_pt(8.0),
            chevron_stroke_width: default_popup_chevron_stroke_width(),
            chevron_spacing: default_popup_chevron_spacing(),
            font_size: default_popup_font_size(),
            border_width: default_border_width(),
            chevron_offset_y: default_popup_chevron_offset_y(),
            menu_item_height: default_popup_menu_item_height(),
            menu_item_padding: default_popup_menu_item_padding(),
            menu_min_width: default_popup_menu_min_width(),
            popup_gap: default_popup_gap(),
        }
    }
}

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
