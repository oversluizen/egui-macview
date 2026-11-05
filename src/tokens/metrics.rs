//! AppKit Control, Toolbar, and Popover geometry metrics tokens.

use serde::{Deserialize, Serialize};
use super::dtcg::DimensionValue;

/// Control focus ring metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FocusRingMetrics {
    pub stroke_width: DimensionValue,
    pub offset: DimensionValue,
    pub corner_radius_delta: DimensionValue,
}

/// Text field metrics for a specific size tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextFieldSizeMetrics {
    pub height: DimensionValue,
    pub font_size: DimensionValue,
    pub corner_radius: DimensionValue,
    #[serde(default)]
    pub padding_horizontal: Option<DimensionValue>,
    #[serde(default)]
    pub padding_vertical: Option<DimensionValue>,
}

/// All NSTextField size tiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextFieldMetrics {
    pub regular: TextFieldSizeMetrics,
    pub small: TextFieldSizeMetrics,
    #[serde(alias = "compact")]
    pub compact_infobox: TextFieldSizeMetrics,
    pub mini: TextFieldSizeMetrics,
    pub large: TextFieldSizeMetrics,
}

impl TextFieldMetrics {
    #[inline]
    pub fn compact(&self) -> &TextFieldSizeMetrics {
        &self.compact_infobox
    }
}

/// Push button metrics for a specific size tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonSizeMetrics {
    pub height: DimensionValue,
    pub corner_radius: DimensionValue,
    #[serde(default)]
    pub padding_horizontal: Option<DimensionValue>,
}

/// Push button metrics tiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonMetrics {
    pub regular: ButtonSizeMetrics,
    pub small: ButtonSizeMetrics,
}

/// NSSegmentedControl metrics tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentedSizeMetrics {
    pub height: DimensionValue,
    pub corner_radius: DimensionValue,
    pub indicator_radius: DimensionValue,
}

/// NSSegmentedControl metrics tiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentedMetrics {
    pub regular: SegmentedSizeMetrics,
    pub small: SegmentedSizeMetrics,
}

/// Control metrics collection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacControlMetrics {
    pub focus_ring: FocusRingMetrics,
    pub text_field: TextFieldMetrics,
    pub button: ButtonMetrics,
    pub segmented: SegmentedMetrics,
}

/// macOS Unified and Compact Toolbar metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolbarMetrics {
    pub traffic_lights_inset: DimensionValue,
    pub height_unified: DimensionValue,
    pub height_compact: DimensionValue,
    pub icon_size: DimensionValue,
    pub hit_target_width: DimensionValue,
    pub hit_target_height: DimensionValue,
    pub hit_target_min: DimensionValue,
    pub item_spacing: DimensionValue,
    pub item_spacing_compact: DimensionValue,
    pub edge_padding: DimensionValue,
}

/// NSPopover and floating HUD panel metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopoverMetrics {
    pub arrow_base_width: DimensionValue,
    pub arrow_height: DimensionValue,
    pub arrow_tip_radius: DimensionValue,
    pub arrow_base_radius: DimensionValue,
    pub corner_radius: DimensionValue,
    pub border_width: DimensionValue,
    pub compact_panel_width: DimensionValue,
    pub compact_panel_height: DimensionValue,
    pub wing_width: DimensionValue,
    pub wing_height: DimensionValue,
    pub full_popover_width: DimensionValue,
    pub full_popover_height: DimensionValue,
    pub shadow_blur: DimensionValue,
    pub shadow_y_offset: DimensionValue,
}

/// Top-level metrics container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacMetricsTokens {
    pub control: MacControlMetrics,
    pub toolbar: ToolbarMetrics,
    pub popover: PopoverMetrics,
}
