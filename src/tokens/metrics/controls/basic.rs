//! Basic control geometry metrics tokens (focus ring, text field, button, segmented).

use crate::tokens::dtcg::DimensionValue;
use serde::{Deserialize, Serialize};

/// Control focus ring metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FocusRingMetrics {
    /// Focus ring stroke thickness in points.
    pub stroke_width: DimensionValue,
    /// Offset distance outside the control boundary.
    pub offset: DimensionValue,
    /// Additional corner radius delta added to match concentric expansion.
    pub corner_radius_delta: DimensionValue,
}

/// Text field metrics for a specific size tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextFieldSizeMetrics {
    /// Height of the text field container.
    pub height: DimensionValue,
    /// Font size of input text.
    pub font_size: DimensionValue,
    /// Outer corner radius.
    pub corner_radius: DimensionValue,
    /// Horizontal content padding.
    pub padding_horizontal: DimensionValue,
    /// Vertical content padding.
    #[serde(default)]
    pub padding_vertical: Option<DimensionValue>,
    /// AppKit `firstBaselineOffsetFromTop` in points.
    pub first_baseline_offset_from_top: DimensionValue,
    /// AppKit `baselineOffsetFromBottom` in points.
    pub baseline_offset_from_bottom: DimensionValue,
    /// Default fallback width.
    pub default_width: DimensionValue,
    /// Border hairline stroke width.
    pub border_width: DimensionValue,
}

/// All NSTextField size tiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextFieldMetrics {
    /// Regular (default) size tier.
    pub regular: TextFieldSizeMetrics,
    /// Small size tier.
    pub small: TextFieldSizeMetrics,
    /// Compact inspector infobox size tier.
    #[serde(alias = "compact")]
    pub compact_infobox: TextFieldSizeMetrics,
    /// Mini size tier.
    pub mini: TextFieldSizeMetrics,
    /// Large size tier.
    pub large: TextFieldSizeMetrics,
    /// Minimum `fixed_rect` height threshold (`21.0pt`) resolving to `Regular`.
    pub auto_size_regular_min_height: DimensionValue,
    /// Minimum `fixed_rect` height threshold (`18.5pt`) resolving to `Small`.
    pub auto_size_small_min_height: DimensionValue,
    /// Minimum `fixed_rect` height threshold (`17.0pt`) resolving to `CompactInfobox`.
    pub auto_size_compact_min_height: DimensionValue,
}

impl TextFieldMetrics {
    /// Alias to access the compact inspector infobox size metrics.
    #[inline]
    pub fn compact(&self) -> &TextFieldSizeMetrics {
        &self.compact_infobox
    }
}

/// Push button metrics for a specific size tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonSizeMetrics {
    /// Height of the button.
    pub height: DimensionValue,
    /// Outer corner radius.
    pub corner_radius: DimensionValue,
    /// Horizontal padding on left and right sides of label.
    #[serde(default)]
    pub padding_horizontal: Option<DimensionValue>,
    /// Font size of button text.
    pub font_size: DimensionValue,
    /// Border hairline stroke width.
    pub border_width: DimensionValue,
}

/// Push button metrics tiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonMetrics {
    /// Regular push button size metrics.
    pub regular: ButtonSizeMetrics,
    /// Small push button size metrics.
    pub small: ButtonSizeMetrics,
    /// Mini push button size metrics.
    pub mini: ButtonSizeMetrics,
}

/// NSSegmentedControl metrics tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentedSizeMetrics {
    /// Overall height of the segmented bar.
    pub height: DimensionValue,
    /// Outer corner radius of the container.
    pub corner_radius: DimensionValue,
    /// Corner radius of the sliding active segment indicator pill.
    pub indicator_radius: DimensionValue,
}

/// NSSegmentedControl metrics tiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentedMetrics {
    /// Regular segmented control size tier.
    pub regular: SegmentedSizeMetrics,
    /// Small segmented control size tier.
    pub small: SegmentedSizeMetrics,
    /// Mini segmented control size tier.
    pub mini: SegmentedSizeMetrics,
    /// Minimum width of each segment item.
    pub item_width: DimensionValue,
    /// Inset of the active selection pill from the outer container edge.
    pub indicator_inset: DimensionValue,
    /// Border hairline stroke width.
    pub border_width: DimensionValue,
    /// Segment label font size.
    pub font_size: DimensionValue,
    /// Small segment label font size (`11.0pt`).
    pub small_font_size: DimensionValue,
    /// Mini segment label font size (`9.0pt`).
    pub mini_font_size: DimensionValue,
    /// Outer corner radius for `.smallSquare` segmented style (`2.0pt`).
    pub small_square_corner_radius: DimensionValue,
    /// Horizontal padding on left and right sides of segment label.
    pub padding_horizontal: DimensionValue,
    /// Duration of the sliding pill transition animation in seconds.
    pub animation_duration: DimensionValue,
    /// Interior padding inset around custom template icons inside a segment cell.
    pub icon_padding: DimensionValue,
    /// Top and bottom vertical inset for rounded-style interior segment dividers.
    pub divider_vertical_inset: DimensionValue,
    /// Horizontal half-width of the trailing downward menu chevron indicator.
    pub menu_chevron_half_width: DimensionValue,
    /// Vertical span of the trailing downward menu chevron indicator.
    pub menu_chevron_height: DimensionValue,
    /// Stroke thickness of the trailing downward menu chevron indicator.
    pub menu_chevron_stroke_width: DimensionValue,
    /// Trailing horizontal inset from segment right edge to the menu chevron apex.
    pub menu_chevron_inset_trailing: DimensionValue,
    /// Bottom vertical inset from segment bottom edge to the menu chevron apex.
    pub menu_chevron_inset_bottom: DimensionValue,
    /// Press-and-hold duration threshold in seconds before opening a segment menu.
    pub menu_press_hold_delay: DimensionValue,
    /// Downward pointer drag distance in points that immediately opens a segment menu.
    pub menu_drag_threshold: DimensionValue,
    /// Opacity multiplier for the depressed `isHighlighted` backdrop on an inactive segment.
    pub pressed_highlight_opacity: DimensionValue,
}
