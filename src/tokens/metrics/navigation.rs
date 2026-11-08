//! Navigation, Menu Bar, Canvas Footer, Command Palette, Context Menu, and Toast Metrics.
//!
//! Grounded in `SPEC-007` and `SPEC-008` via W3C DTCG tokens (`tokens/design_tokens.json`).

use crate::tokens::dtcg::DimensionValue;
use serde::{Deserialize, Serialize};

/// Deterministic canvas footer height (`24.0pt` per `SPEC-007 §3`).
pub const CANVAS_FOOTER_HEIGHT: f32 = 24.0;
/// Default Spotlight command palette modal width (`480.0pt` per `SPEC-007 §4`).
pub const COMMAND_PALETTE_WIDTH: f32 = 480.0;
/// Default Spotlight command palette modal height (`280.0pt` per `SPEC-007 §4`).
pub const COMMAND_PALETTE_HEIGHT: f32 = 280.0;
/// Default toast duration in seconds (`2.5s` per `SPEC-008 §4`).
pub const DEFAULT_TOAST_DURATION: f32 = 2.5;

/// SPEC-007 Menu Bar and Dropdown metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MenuBarMetrics {
    /// Height of the top-level menu bar in points.
    pub bar_height: DimensionValue,
    /// Horizontal padding inside each top-level menu title button.
    pub item_padding_x: DimensionValue,
    /// Horizontal spacing between top-level menu titles.
    pub item_spacing: DimensionValue,
    /// Minimum dropdown menu width.
    pub dropdown_min_width: DimensionValue,
    /// Maximum dropdown menu width.
    pub dropdown_max_width: DimensionValue,
    /// Standard height for dropdown menu item row.
    pub dropdown_item_height: DimensionValue,
    /// Vertical span of menu separator row.
    pub dropdown_separator_height: DimensionValue,
    /// Corner radius for dropdown menu container.
    pub dropdown_corner_radius: DimensionValue,
    /// Top and bottom padding inside dropdown menu.
    pub dropdown_padding_y: DimensionValue,
    /// Horizontal padding inside dropdown menu item.
    pub dropdown_padding_x: DimensionValue,
    /// Minimum horizontal clearance between item title and shortcut text.
    pub shortcut_clearance: DimensionValue,
    /// Horizontal gap between parent menu and cascading submenu.
    pub submenu_gap: DimensionValue,
    /// Corner radius for active top-level menu bar pill.
    pub bar_corner_radius: DimensionValue,
    /// Menu item title font size.
    pub font_size: DimensionValue,
    /// Menu item shortcut accelerator font size.
    pub shortcut_font_size: DimensionValue,
}

impl Default for MenuBarMetrics {
    fn default() -> Self {
        Self {
            bar_height: DimensionValue::from_pt(24.0),
            item_padding_x: DimensionValue::from_pt(8.0),
            item_spacing: DimensionValue::from_pt(2.0),
            dropdown_min_width: DimensionValue::from_pt(180.0),
            dropdown_max_width: DimensionValue::from_pt(360.0),
            dropdown_item_height: DimensionValue::from_pt(22.0),
            dropdown_separator_height: DimensionValue::from_pt(9.0),
            dropdown_corner_radius: DimensionValue::from_pt(6.0),
            dropdown_padding_y: DimensionValue::from_pt(4.0),
            dropdown_padding_x: DimensionValue::from_pt(10.0),
            shortcut_clearance: DimensionValue::from_pt(24.0),
            submenu_gap: DimensionValue::from_pt(2.0),
            bar_corner_radius: DimensionValue::from_pt(4.0),
            font_size: DimensionValue::from_pt(13.0),
            shortcut_font_size: DimensionValue::from_pt(12.0),
        }
    }
}

/// SPEC-007 Canvas Footer metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FooterMetrics {
    /// Deterministic SPEC-007 canvas footer height (`24.0pt`).
    pub height: DimensionValue,
    /// Horizontal edge inset for footer bar.
    pub edge_padding_x: DimensionValue,
    /// Width of zoom preset pill button.
    pub preset_width: DimensionValue,
    /// Height of zoom preset pill button.
    pub preset_height: DimensionValue,
    /// Horizontal gap between zoom preset buttons.
    pub preset_spacing: DimensionValue,
    /// Corner radius of zoom preset pill button.
    pub preset_corner_radius: DimensionValue,
    /// Breadcrumb and coordinate status font size.
    pub font_size: DimensionValue,
    /// Zoom preset button font size.
    pub preset_font_size: DimensionValue,
    /// Horizontal gap around vertical footer dividers.
    pub separator_gap: DimensionValue,
    /// Reserved width for cursor coordinate readout.
    pub coords_width: DimensionValue,
    /// Trailing offset for zoom presets cluster.
    pub presets_zone_offset: DimensionValue,
    /// Trailing offset for divider before zoom presets.
    pub divider_trailing_offset: DimensionValue,
    /// Top/bottom inset for vertical footer divider line.
    pub divider_vertical_inset: DimensionValue,
}

impl Default for FooterMetrics {
    fn default() -> Self {
        Self {
            height: DimensionValue::from_pt(CANVAS_FOOTER_HEIGHT),
            edge_padding_x: DimensionValue::from_pt(10.0),
            preset_width: DimensionValue::from_pt(36.0),
            preset_height: DimensionValue::from_pt(18.0),
            preset_spacing: DimensionValue::from_pt(2.0),
            preset_corner_radius: DimensionValue::from_pt(3.0),
            font_size: DimensionValue::from_pt(10.5),
            preset_font_size: DimensionValue::from_pt(10.0),
            separator_gap: DimensionValue::from_pt(10.0),
            coords_width: DimensionValue::from_pt(75.0),
            presets_zone_offset: DimensionValue::from_pt(150.0),
            divider_trailing_offset: DimensionValue::from_pt(155.0),
            divider_vertical_inset: DimensionValue::from_pt(5.0),
        }
    }
}

/// SPEC-007 Spotlight Command Palette metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaletteMetrics {
    /// Spotlight command palette modal width (`480.0pt`).
    pub width: DimensionValue,
    /// Maximum command palette modal height (`280.0pt`).
    pub max_height: DimensionValue,
    /// Command palette outer squircle corner radius.
    pub corner_radius: DimensionValue,
    /// Top query input bar height.
    pub search_bar_height: DimensionValue,
    /// Horizontal padding inside query input bar.
    pub search_padding_x: DimensionValue,
    /// Leading magnifying glass icon size in palette.
    pub search_icon_size: DimensionValue,
    /// Height of each command result row.
    pub row_height: DimensionValue,
    /// Horizontal padding inside each result row.
    pub row_padding_x: DimensionValue,
    /// Corner radius of highlighted result row.
    pub row_corner_radius: DimensionValue,
    /// Horizontal inset of result rows from palette edge.
    pub row_inset: DimensionValue,
    /// Font size of query input text.
    pub query_font_size: DimensionValue,
    /// Font size of result item title.
    pub item_font_size: DimensionValue,
    /// Font size of category badge.
    pub category_font_size: DimensionValue,
    /// Font size of keyboard shortcut badge.
    pub shortcut_font_size: DimensionValue,
}

impl Default for PaletteMetrics {
    fn default() -> Self {
        Self {
            width: DimensionValue::from_pt(COMMAND_PALETTE_WIDTH),
            max_height: DimensionValue::from_pt(COMMAND_PALETTE_HEIGHT),
            corner_radius: DimensionValue::from_pt(10.0),
            search_bar_height: DimensionValue::from_pt(36.0),
            search_padding_x: DimensionValue::from_pt(12.0),
            search_icon_size: DimensionValue::from_pt(14.0),
            row_height: DimensionValue::from_pt(28.0),
            row_padding_x: DimensionValue::from_pt(12.0),
            row_corner_radius: DimensionValue::from_pt(5.0),
            row_inset: DimensionValue::from_pt(4.0),
            query_font_size: DimensionValue::from_pt(14.0),
            item_font_size: DimensionValue::from_pt(12.5),
            category_font_size: DimensionValue::from_pt(10.5),
            shortcut_font_size: DimensionValue::from_pt(10.5),
        }
    }
}

/// SPEC-008 Context Menu metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ContextMenuMetrics {
    /// Context menu item row height.
    pub item_height: DimensionValue,
    /// Minimum context menu width.
    pub min_width: DimensionValue,
    /// Inner padding of context menu container.
    pub padding: DimensionValue,
    /// Horizontal text inset inside context menu row.
    pub item_padding_x: DimensionValue,
    /// Context menu outer corner radius.
    pub corner_radius: DimensionValue,
    /// Highlighted context menu item corner radius.
    pub row_corner_radius: DimensionValue,
    /// Minimum clearance between context menu label and shortcut.
    pub shortcut_clearance: DimensionValue,
    /// Context menu item font size.
    pub font_size: DimensionValue,
}

impl Default for ContextMenuMetrics {
    fn default() -> Self {
        Self {
            item_height: DimensionValue::from_pt(22.0),
            min_width: DimensionValue::from_pt(140.0),
            padding: DimensionValue::from_pt(4.0),
            item_padding_x: DimensionValue::from_pt(8.0),
            corner_radius: DimensionValue::from_pt(6.0),
            row_corner_radius: DimensionValue::from_pt(4.0),
            shortcut_clearance: DimensionValue::from_pt(24.0),
            font_size: DimensionValue::from_pt(12.0),
        }
    }
}

/// SPEC-008 Toast Notification metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ToastMetrics {
    /// Floating toast pill height.
    pub height: DimensionValue,
    /// Minimum floating toast pill width.
    pub min_width: DimensionValue,
    /// Top viewport margin for floating toast stack.
    pub top_margin: DimensionValue,
    /// Vertical spacing between stacked toast pills.
    pub spacing: DimensionValue,
    /// Total horizontal text padding inside toast pill.
    pub padding_x: DimensionValue,
    /// Default toast display duration in seconds.
    pub duration_secs: DimensionValue,
    /// Toast message typography font size.
    pub font_size: DimensionValue,
}

impl Default for ToastMetrics {
    fn default() -> Self {
        Self {
            height: DimensionValue::from_pt(32.0),
            min_width: DimensionValue::from_pt(120.0),
            top_margin: DimensionValue::from_pt(20.0),
            spacing: DimensionValue::from_pt(8.0),
            padding_x: DimensionValue::from_pt(32.0),
            duration_secs: DimensionValue::from_pt(DEFAULT_TOAST_DURATION),
            font_size: DimensionValue::from_pt(12.0),
        }
    }
}

/// Combined navigation and feedback metrics container.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct NavigationMetrics {
    /// Menu bar and dropdown metrics.
    pub menu_bar: MenuBarMetrics,
    /// Canvas footer metrics.
    pub footer: FooterMetrics,
    /// Spotlight command palette metrics.
    pub palette: PaletteMetrics,
    /// Right-click context menu metrics.
    pub context_menu: ContextMenuMetrics,
    /// Floating toast notification metrics.
    pub toast: ToastMetrics,
}
