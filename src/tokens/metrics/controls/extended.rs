//! Extended control geometry metrics tokens (slider, thumbnail, outline, popup).

use crate::tokens::dtcg::DimensionValue;
use crate::tokens::MacDesignTokens;
use serde::{Deserialize, Serialize};

pub use super::selection_and_value::{
    ColorWellMetrics, SearchFieldMetrics, StepperMetrics, ToggleMetrics,
};

/// NSSlider metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SliderMetrics {
    /// Height of the slider groove track.
    pub track_height: DimensionValue,
    /// Neutral track height when `numberOfTickMarks > 0` (`4.0pt`).
    pub ticked_track_height: DimensionValue,
    /// Neutral track border stroke width when `numberOfTickMarks > 0` (`1.0pt`).
    pub ticked_track_border_width: DimensionValue,
    /// Diameter of the circular thumb knob.
    pub thumb_size: DimensionValue,
    /// Stroke width of the thumb knob border.
    pub thumb_stroke_width: DimensionValue,
    /// Height of track tick mark indicators.
    pub tick_height: DimensionValue,
    /// Width/thickness of tick mark lines.
    pub tick_width: DimensionValue,
    /// Vertical distance from slider track to tick marks.
    pub tick_offset: DimensionValue,
    /// Default fallback width in points.
    pub default_width: DimensionValue,
    /// Step fraction traversed by keyboard arrow key presses.
    pub key_step_fraction: DimensionValue,
    /// Vertical offset of the thumb ambient drop shadow.
    pub shadow_offset_y: DimensionValue,
    /// Scale multiplier for the thumb knob when pressed or dragged (`1.15pt`).
    pub thumb_press_scale: DimensionValue,
    /// Width of the vertical pill thumb when `numberOfTickMarks > 0` (`7.0pt`).
    pub ticked_thumb_width: DimensionValue,
    /// Height of the vertical pill thumb when `numberOfTickMarks > 0` (`15.0pt`).
    pub ticked_thumb_height: DimensionValue,
    /// Corner radius of the vertical pill thumb when `numberOfTickMarks > 0` (`3.5pt`).
    pub ticked_thumb_corner_radius: DimensionValue,
}

impl Default for SliderMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().slider_metrics().clone()
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
        MacDesignTokens::default_shared().thumbnail_metrics().clone()
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
    pub chevron_stroke_width: DimensionValue,
    /// Span size of the disclosure chevron legs.
    pub chevron_size: DimensionValue,
    /// Forward depth of the disclosure chevron arrow tip.
    pub chevron_depth: DimensionValue,
    /// Spacing between adjacent outline rows.
    pub item_spacing: DimensionValue,
    /// Outer padding around outline view content.
    pub edge_padding: DimensionValue,
    /// Font size for section headers (11.0pt AppKit Small Caps).
    pub header_font_size: DimensionValue,
}

impl Default for OutlineViewMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().outline_view_metrics().clone()
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
    pub chevron_stroke_width: DimensionValue,
    /// Half-gap vertical separation from vertical center to each chevron base (`1.75pt`).
    pub chevron_gap_vertical: DimensionValue,
    /// Horizontal half-width of each chevron (`2.5pt`).
    pub chevron_half_width: DimensionValue,
    /// Vertical height of each chevron from base to apex (`2.25pt`).
    pub chevron_height: DimensionValue,
    /// AppKit `firstBaselineOffsetFromTop` for Regular popup buttons (`14pt`).
    pub regular_first_baseline_offset: DimensionValue,
    /// AppKit `firstBaselineOffsetFromTop` for Small popup buttons (`12pt`).
    pub small_first_baseline_offset: DimensionValue,
    /// AppKit `firstBaselineOffsetFromTop` for Mini popup buttons (`10pt`).
    pub mini_first_baseline_offset: DimensionValue,
    /// AppKit visual bezel height for Regular popup buttons inside a 25pt NIB frame (`20pt`).
    pub regular_visual_height: DimensionValue,
    /// AppKit visual bezel height for Small popup buttons inside a 22pt NIB frame (`16pt`).
    pub small_visual_height: DimensionValue,
    /// AppKit visual bezel height for Mini popup buttons inside a 16pt NIB frame (`13pt`).
    pub mini_visual_height: DimensionValue,
    /// Spacing between label text and chevron arrows.
    pub chevron_spacing: DimensionValue,
    /// Font size of selected item label.
    pub font_size: DimensionValue,
    /// Small intrinsic height for `NSPopUpButton` (`18.0pt`).
    pub small_height: DimensionValue,
    /// Mini intrinsic height for `NSPopUpButton` (`15.0pt`).
    pub mini_height: DimensionValue,
    /// Small font size for `NSPopUpButton` (`11.0pt`).
    pub small_font_size: DimensionValue,
    /// Mini font size for `NSPopUpButton` (`9.0pt`).
    pub mini_font_size: DimensionValue,
    /// Border hairline stroke width.
    pub border_width: DimensionValue,
    /// Vertical offset of chevrons from center.
    pub chevron_offset_y: DimensionValue,
    /// Height of each dropdown menu item.
    pub menu_item_height: DimensionValue,
    /// Padding around menu item text.
    pub menu_item_padding: DimensionValue,
    /// Minimum width of the popup menu panel.
    pub menu_min_width: DimensionValue,
    /// Gap between popup button and menu surface.
    pub popup_gap: DimensionValue,
}

impl Default for PopUpButtonMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().popup_button_metrics().clone()
    }
}
