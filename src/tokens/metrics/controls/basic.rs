//! Basic control geometry metrics tokens (focus ring, text field, button, segmented).

use crate::tokens::dtcg::DimensionValue;
use serde::{Deserialize, Serialize};

/// Returns default 1.0 pt hairline border stroke width.
pub fn default_border_width() -> DimensionValue {
    DimensionValue::from_pt(1.0)
}

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
    #[serde(default = "default_text_field_padding_horizontal")]
    pub padding_horizontal: DimensionValue,
    /// Vertical content padding.
    #[serde(default)]
    pub padding_vertical: Option<DimensionValue>,
    /// AppKit `firstBaselineOffsetFromTop` in points.
    #[serde(default = "default_text_field_first_baseline")]
    pub first_baseline_offset_from_top: DimensionValue,
    /// AppKit `baselineOffsetFromBottom` in points.
    #[serde(default = "default_text_field_bottom_baseline")]
    pub baseline_offset_from_bottom: DimensionValue,
    /// Default fallback width.
    #[serde(default = "default_text_field_default_width")]
    pub default_width: DimensionValue,
    /// Border hairline stroke width.
    #[serde(default = "default_border_width")]
    pub border_width: DimensionValue,
}

fn default_text_field_padding_horizontal() -> DimensionValue {
    DimensionValue::from_pt(4.0)
}
fn default_text_field_first_baseline() -> DimensionValue {
    DimensionValue::from_pt(15.0)
}
fn default_text_field_bottom_baseline() -> DimensionValue {
    DimensionValue::from_pt(6.0)
}
fn default_text_field_default_width() -> DimensionValue {
    DimensionValue::from_pt(44.0)
}
fn default_auto_size_regular_min() -> DimensionValue {
    DimensionValue::from_pt(21.0)
}
fn default_auto_size_small_min() -> DimensionValue {
    DimensionValue::from_pt(18.5)
}
fn default_auto_size_compact_min() -> DimensionValue {
    DimensionValue::from_pt(17.0)
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
    #[serde(default = "default_auto_size_regular_min")]
    pub auto_size_regular_min_height: DimensionValue,
    /// Minimum `fixed_rect` height threshold (`18.5pt`) resolving to `Small`.
    #[serde(default = "default_auto_size_small_min")]
    pub auto_size_small_min_height: DimensionValue,
    /// Minimum `fixed_rect` height threshold (`17.0pt`) resolving to `CompactInfobox`.
    #[serde(default = "default_auto_size_compact_min")]
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
    #[serde(default = "default_button_font_size")]
    pub font_size: DimensionValue,
    /// Border hairline stroke width.
    #[serde(default = "default_border_width")]
    pub border_width: DimensionValue,
}

fn default_button_font_size() -> DimensionValue {
    DimensionValue::from_pt(13.0)
}

/// Push button metrics tiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonMetrics {
    /// Regular push button size metrics.
    pub regular: ButtonSizeMetrics,
    /// Small push button size metrics.
    pub small: ButtonSizeMetrics,
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

fn default_segmented_item_width() -> DimensionValue {
    DimensionValue::from_pt(36.0)
}

fn default_segmented_indicator_inset() -> DimensionValue {
    DimensionValue::from_pt(2.0)
}

fn default_segmented_font_size() -> DimensionValue {
    DimensionValue::from_pt(11.0)
}

fn default_segmented_padding_horizontal() -> DimensionValue {
    DimensionValue::from_pt(10.0)
}

fn default_segmented_animation_duration() -> DimensionValue {
    DimensionValue::from_pt(0.18)
}

/// NSSegmentedControl metrics tiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentedMetrics {
    /// Regular segmented control size tier.
    pub regular: SegmentedSizeMetrics,
    /// Small segmented control size tier.
    pub small: SegmentedSizeMetrics,
    /// Minimum width of each segment item.
    #[serde(default = "default_segmented_item_width")]
    pub item_width: DimensionValue,
    /// Inset of the active selection pill from the outer container edge.
    #[serde(default = "default_segmented_indicator_inset")]
    pub indicator_inset: DimensionValue,
    /// Border hairline stroke width.
    #[serde(default = "default_border_width")]
    pub border_width: DimensionValue,
    /// Segment label font size.
    #[serde(default = "default_segmented_font_size")]
    pub font_size: DimensionValue,
    /// Horizontal padding on left and right sides of segment label.
    #[serde(default = "default_segmented_padding_horizontal")]
    pub padding_horizontal: DimensionValue,
    /// Duration of the sliding pill transition animation in seconds.
    #[serde(default = "default_segmented_animation_duration")]
    pub animation_duration: DimensionValue,
}
