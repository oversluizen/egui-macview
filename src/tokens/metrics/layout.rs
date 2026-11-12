//! AppKit Toolbar, Popover, SplitView, and Layout geometry metrics tokens.

use crate::tokens::dtcg::DimensionValue;
use crate::tokens::metrics::controls::MacControlMetrics;
use crate::tokens::metrics::navigation::NavigationMetrics;
use crate::tokens::MacDesignTokens;
use serde::{Deserialize, Serialize};

/// macOS Unified and Compact Toolbar metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolbarMetrics {
    /// Inset from window leading edge to the first traffic light button.
    pub traffic_lights_inset: DimensionValue,
    /// Height of a unified macOS window titlebar/toolbar.
    pub height_unified: DimensionValue,
    /// Height of a compact macOS window toolbar.
    pub height_compact: DimensionValue,
    /// Standard toolbar item icon size.
    pub icon_size: DimensionValue,
    /// Width of interactive toolbar button hit targets.
    pub hit_target_width: DimensionValue,
    /// Height of interactive toolbar button hit targets.
    pub hit_target_height: DimensionValue,
    /// Minimum dimension of toolbar hit targets.
    pub hit_target_min: DimensionValue,
    /// Standard spacing between adjacent toolbar items.
    pub item_spacing: DimensionValue,
    /// Compact spacing between adjacent toolbar items.
    pub item_spacing_compact: DimensionValue,
    /// Outer edge padding of toolbar content.
    pub edge_padding: DimensionValue,
    /// Width of Windows 11 window caption buttons.
    pub caption_button_width: DimensionValue,
    /// Height of Windows 11 window caption buttons.
    pub caption_button_height: DimensionValue,
    /// Icon size for Windows 11 window caption buttons.
    pub caption_icon_size: DimensionValue,
    /// Radius of macOS window traffic light buttons.
    pub traffic_light_radius: DimensionValue,
    /// Spacing between centers of adjacent traffic light buttons.
    pub traffic_light_spacing: DimensionValue,
    /// Window top corner radius in points.
    pub window_corner_radius: DimensionValue,
    /// Interactive hit box size for each traffic light circle.
    pub traffic_light_hit_size: DimensionValue,
    /// Leading inset from left toolbar edge to first traffic light center.
    pub traffic_light_leading_inset: DimensionValue,
    /// Half-span of interior close/zoom glyph inside traffic light circle.
    pub traffic_light_glyph_half: DimensionValue,
    /// Minimum available width to render window title in toolbar center zone.
    pub min_title_width: DimensionValue,
}

/// NSPopover and floating HUD panel metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopoverMetrics {
    /// Width of the popover arrow base.
    pub arrow_base_width: DimensionValue,
    /// Altitude/height of the popover arrow triangle.
    pub arrow_height: DimensionValue,
    /// Corner radius at the tip of the popover arrow.
    pub arrow_tip_radius: DimensionValue,
    /// Blend radius at the base where the arrow merges into the body.
    pub arrow_base_radius: DimensionValue,
    /// Corner radius of the popover bubble body.
    pub corner_radius: DimensionValue,
    /// Border hairline stroke width.
    pub border_width: DimensionValue,
    /// Width of compact floating HUD panels.
    pub compact_panel_width: DimensionValue,
    /// Height of compact floating HUD panels.
    pub compact_panel_height: DimensionValue,
    /// Width of lateral wing extensions.
    pub wing_width: DimensionValue,
    /// Height of lateral wing extensions.
    pub wing_height: DimensionValue,
    /// Default width of full popover surfaces.
    pub full_popover_width: DimensionValue,
    /// Default height of full popover surfaces.
    pub full_popover_height: DimensionValue,
    /// Gaussian shadow blur radius.
    pub shadow_blur: DimensionValue,
    /// Drop shadow vertical offset.
    pub shadow_y_offset: DimensionValue,
    /// Height of popover header bar.
    pub header_height: DimensionValue,
    /// Font size of popover header title text.
    pub header_font_size: DimensionValue,
    /// Inner padding around popover body content.
    pub inner_padding: DimensionValue,
    /// Standard corner radius for AppKit `NSPopover` container surface (`8.0pt`).
    pub surface_corner_radius: DimensionValue,
    /// Subtle perimeter hairline border width for AppKit `NSPopover` container surface (`1.0pt`).
    pub surface_border_width: DimensionValue,
    /// Corner radius for compact floating HUD palette (`6.0pt`).
    pub hud_corner_radius: DimensionValue,
    /// Row height inside compact floating HUD palette (`18.0pt`).
    pub hud_row_height: DimensionValue,
    /// Vertical gap between rows inside compact floating HUD palette (`1.0pt`).
    pub hud_row_gap: DimensionValue,
    /// Horizontal padding inside compact floating HUD palette (`6.0pt`).
    pub hud_padding_x: DimensionValue,
    /// Top padding below header inside compact floating HUD palette (`2.0pt`).
    pub hud_padding_top: DimensionValue,
    /// Status indicator dot radius in HUD header (`3.0pt`).
    pub hud_dot_radius: DimensionValue,
    /// Leading X offset of HUD header dot (`10.0pt`).
    pub hud_dot_offset_x: DimensionValue,
    /// Leading X offset of HUD header title (`20.0pt`).
    pub hud_title_offset_x: DimensionValue,
}

/// NSSplitView metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitViewMetrics {
    /// Visible divider hairline thickness in points.
    pub divider_thickness: DimensionValue,
    /// Expanded interactive mouse hit testing thickness around the divider.
    pub hit_thickness: DimensionValue,
    /// Minimum allowed size for any pane in points.
    pub min_pane_size: DimensionValue,
    /// Length of the center divider drag handle pill.
    pub handle_length: DimensionValue,
    /// Thickness of the center divider drag handle pill.
    pub handle_thickness: DimensionValue,
    /// Default split proportion between first and second pane.
    pub default_ratio: DimensionValue,
}

impl Default for SplitViewMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().split_view_metrics().clone()
    }
}

/// NSScrollView overlay metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScrollViewMetrics {
    /// Thickness of floating overlay scrollbar track/thumb.
    pub scrollbar_thickness: DimensionValue,
    /// Minimum length of the draggable scrollbar thumb.
    pub thumb_min_length: DimensionValue,
    /// Corner rounding radius of the scrollbar thumb pill.
    pub corner_radius: DimensionValue,
    /// Inactivity timeout before scrollbars fade out, in seconds.
    pub fade_timeout: DimensionValue,
    /// Inset from the viewport edge to the scrollbar.
    pub edge_inset: DimensionValue,
    /// Expanded mouse hit zone padding around the scrollbar.
    pub hit_zone_padding: DimensionValue,
    /// Minimum visible thumb length when content is excessively large.
    pub min_visible_thumb_length: DimensionValue,
    /// Expanded overlay scrollbar thickness when hovered or dragged (`10.0pt`).
    pub scrollbar_hover_thickness: DimensionValue,
}

impl Default for ScrollViewMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().scroll_view_metrics().clone()
    }
}

/// NSStackView layout metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackViewMetrics {
    /// Default spacing between adjacent items in the stack.
    pub default_spacing: DimensionValue,
    /// Default outer edge padding around the stack items.
    pub edge_padding: DimensionValue,
    /// Hairline thickness for inter-item separators.
    pub separator_thickness: DimensionValue,
    /// Font size of section header labels.
    pub section_header_font_size: DimensionValue,
    /// Default item height for horizontal stack separator alignment.
    pub default_item_height: DimensionValue,
}

impl Default for StackViewMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().stack_view_metrics().clone()
    }
}

/// Layout engines metrics container.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LayoutMetrics {
    /// Scroll view overlay metrics.
    pub scroll_view: ScrollViewMetrics,
    /// Stack view layout metrics.
    pub stack_view: StackViewMetrics,
}

/// Top-level metrics container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacMetricsTokens {
    /// Control component metrics.
    pub control: MacControlMetrics,
    /// Window toolbar metrics.
    pub toolbar: ToolbarMetrics,
    /// Popover and panel surface metrics.
    pub popover: PopoverMetrics,
    /// Split view divider metrics.
    pub split_view: SplitViewMetrics,
    /// Layout container metrics.
    pub layout: LayoutMetrics,
    /// Navigation and feedback metrics.
    pub navigation: NavigationMetrics,
}
