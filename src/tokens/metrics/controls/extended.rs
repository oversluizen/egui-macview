//! Extended control geometry metrics tokens (slider, thumbnail, outline, popup).

use crate::tokens::dtcg::DimensionValue;
use crate::tokens::metrics::controls::basic::default_border_width;
use serde::{Deserialize, Serialize};

pub use super::selection_and_value::{
    ColorWellMetrics, SearchFieldMetrics, StepperMetrics, ToggleMetrics,
};

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
    /// Scale multiplier for the thumb knob when pressed or dragged (`1.15pt`).
    #[serde(default = "default_slider_thumb_press_scale")]
    pub thumb_press_scale: DimensionValue,
}

fn default_slider_thumb_press_scale() -> DimensionValue { DimensionValue::from_pt(1.15) }
fn default_slider_thumb_stroke_width() -> DimensionValue { DimensionValue::from_pt(1.0) }
fn default_slider_shadow_offset_y() -> DimensionValue { DimensionValue::from_pt(1.5) }
fn default_slider_default_width() -> DimensionValue { DimensionValue::from_pt(120.0) }
fn default_slider_key_step_fraction() -> DimensionValue { DimensionValue::from_pt(0.05) }

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
            thumb_press_scale: default_slider_thumb_press_scale(),
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

fn default_outline_header_font_size() -> DimensionValue { DimensionValue::from_pt(11.0) }
fn default_outline_chevron_stroke_width() -> DimensionValue { DimensionValue::from_pt(1.2) }
fn default_outline_chevron_size() -> DimensionValue { DimensionValue::from_pt(7.0) }
fn default_outline_chevron_depth() -> DimensionValue { DimensionValue::from_pt(1.75) }
fn default_outline_item_spacing() -> DimensionValue { DimensionValue::from_pt(6.0) }
fn default_outline_edge_padding() -> DimensionValue { DimensionValue::from_pt(4.0) }

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
    /// Half-gap vertical separation from vertical center to each chevron base (`1.75pt`).
    #[serde(default = "default_popup_chevron_gap_vertical")]
    pub chevron_gap_vertical: DimensionValue,
    /// Horizontal half-width of each chevron (`2.5pt`).
    #[serde(default = "default_popup_chevron_half_width")]
    pub chevron_half_width: DimensionValue,
    /// Vertical height of each chevron from base to apex (`2.25pt`).
    #[serde(default = "default_popup_chevron_height")]
    pub chevron_height: DimensionValue,
    /// AppKit `firstBaselineOffsetFromTop` for Regular popup buttons (`14pt`).
    #[serde(default = "default_popup_regular_baseline")]
    pub regular_first_baseline_offset: DimensionValue,
    /// AppKit `firstBaselineOffsetFromTop` for Small popup buttons (`12pt`).
    #[serde(default = "default_popup_small_baseline")]
    pub small_first_baseline_offset: DimensionValue,
    /// AppKit `firstBaselineOffsetFromTop` for Mini popup buttons (`10pt`).
    #[serde(default = "default_popup_mini_baseline")]
    pub mini_first_baseline_offset: DimensionValue,
    /// AppKit visual bezel height for Regular popup buttons inside a 25pt NIB frame (`20pt`).
    #[serde(default = "default_popup_regular_visual_h")]
    pub regular_visual_height: DimensionValue,
    /// AppKit visual bezel height for Small popup buttons inside a 22pt NIB frame (`16pt`).
    #[serde(default = "default_popup_small_visual_h")]
    pub small_visual_height: DimensionValue,
    /// AppKit visual bezel height for Mini popup buttons inside a 16pt NIB frame (`13pt`).
    #[serde(default = "default_popup_mini_visual_h")]
    pub mini_visual_height: DimensionValue,
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

fn default_popup_chevron_stroke_width() -> DimensionValue { DimensionValue::from_pt(1.25) }
fn default_popup_chevron_gap_vertical() -> DimensionValue { DimensionValue::from_pt(1.75) }
fn default_popup_chevron_half_width() -> DimensionValue { DimensionValue::from_pt(2.5) }
fn default_popup_chevron_height() -> DimensionValue { DimensionValue::from_pt(2.25) }
fn default_popup_regular_baseline() -> DimensionValue { DimensionValue::from_pt(14.0) }
fn default_popup_small_baseline() -> DimensionValue { DimensionValue::from_pt(12.0) }
fn default_popup_mini_baseline() -> DimensionValue { DimensionValue::from_pt(10.0) }
fn default_popup_regular_visual_h() -> DimensionValue { DimensionValue::from_pt(20.0) }
fn default_popup_small_visual_h() -> DimensionValue { DimensionValue::from_pt(16.0) }
fn default_popup_mini_visual_h() -> DimensionValue { DimensionValue::from_pt(13.0) }
fn default_popup_chevron_spacing() -> DimensionValue { DimensionValue::from_pt(6.0) }
fn default_popup_font_size() -> DimensionValue { DimensionValue::from_pt(13.0) }
fn default_popup_chevron_offset_y() -> DimensionValue { DimensionValue::from_pt(2.0) }
fn default_popup_menu_item_height() -> DimensionValue { DimensionValue::from_pt(20.0) }
fn default_popup_menu_item_padding() -> DimensionValue { DimensionValue::from_pt(6.0) }
fn default_popup_menu_min_width() -> DimensionValue { DimensionValue::from_pt(120.0) }
fn default_popup_gap() -> DimensionValue { DimensionValue::from_pt(2.0) }

impl Default for PopUpButtonMetrics {
    fn default() -> Self {
        Self {
            height: DimensionValue::from_pt(22.0),
            corner_radius: DimensionValue::from_pt(5.0),
            padding_horizontal: DimensionValue::from_pt(8.0),
            chevron_size: DimensionValue::from_pt(8.0),
            chevron_stroke_width: default_popup_chevron_stroke_width(),
            chevron_gap_vertical: default_popup_chevron_gap_vertical(),
            chevron_half_width: default_popup_chevron_half_width(),
            chevron_height: default_popup_chevron_height(),
            regular_first_baseline_offset: default_popup_regular_baseline(),
            small_first_baseline_offset: default_popup_small_baseline(),
            mini_first_baseline_offset: default_popup_mini_baseline(),
            regular_visual_height: default_popup_regular_visual_h(),
            small_visual_height: default_popup_small_visual_h(),
            mini_visual_height: default_popup_mini_visual_h(),
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
