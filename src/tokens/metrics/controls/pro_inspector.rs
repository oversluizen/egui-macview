//! AppKit Pro-Inspector control metrics (`MacOriginMatrix`, `MacTagColorPicker`, `MacTableActionBar`).

use crate::tokens::dtcg::DimensionValue;
use crate::tokens::MacDesignTokens;
use serde::{Deserialize, Serialize};

/// Geometry tokens for `MacOriginMatrix` (`3x3` 9-point transform origin / anchor pin grid).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OriginMatrixMetrics {
    /// Regular square bounding box size (`34.0pt`, matching `TransformView.nib` `GSTransformGrid`).
    pub grid_size: DimensionValue,
    /// Small / CompactInfobox square bounding box size (`28.0pt`).
    pub small_grid_size: DimensionValue,
    /// Mini square bounding box size (`22.0pt`).
    pub mini_grid_size: DimensionValue,
    /// Regular center-to-center step between adjacent grid pins (`12.0pt`).
    pub cell_step: DimensionValue,
    /// Small / CompactInfobox center-to-center step between adjacent grid pins (`10.0pt`).
    pub small_cell_step: DimensionValue,
    /// Mini center-to-center step between adjacent grid pins (`8.0pt`).
    pub mini_cell_step: DimensionValue,
    /// Stroke width of the outer bounding-box wireframe rectangle (`1.0pt`).
    pub line_stroke_width: DimensionValue,
    /// Inactive solid anchor pin radius (`2.5pt`, `5.0pt` diameter).
    pub node_radius: DimensionValue,
    /// Selected solid anchor pin radius (`3.5pt`, `7.0pt` diameter).
    pub active_node_radius: DimensionValue,
    /// Hovered solid anchor pin radius (`3.0pt`, `6.0pt` diameter).
    pub hover_node_radius: DimensionValue,
    /// Highlight ring stroke width on the selected anchor pin (`1.0pt`).
    pub active_ring_stroke_width: DimensionValue,
}

impl Default for OriginMatrixMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared()
            .origin_matrix_metrics()
            .clone()
    }
}

/// Geometry tokens for `MacTagColorPicker` (macOS Finder / AppKit color label swatch dots).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagColorPickerMetrics {
    /// Regular circular swatch dot diameter (`12.0pt`).
    pub dot_diameter: DimensionValue,
    /// Small / CompactInfobox circular swatch dot diameter (`10.0pt`).
    pub small_dot_diameter: DimensionValue,
    /// Mini circular swatch dot diameter (`8.0pt`).
    pub mini_dot_diameter: DimensionValue,
    /// Regular gap between adjacent swatch dots (`5.0pt`).
    pub dot_spacing: DimensionValue,
    /// Small / CompactInfobox gap between adjacent swatch dots (`4.0pt`).
    pub small_dot_spacing: DimensionValue,
    /// Mini gap between adjacent swatch dots (`3.0pt`).
    pub mini_dot_spacing: DimensionValue,
    /// Radial expansion offset for the active selection ring (`1.5pt`).
    pub selected_ring_offset: DimensionValue,
    /// Stroke width of the active selection ring (`1.5pt`).
    pub selected_ring_stroke: DimensionValue,
    /// Hairline border stroke width around each color dot (`1.0pt`).
    pub border_stroke_width: DimensionValue,
    /// Stroke width of the diagonal cross inside the clear (`None`) swatch (`1.2pt`).
    pub clear_cross_stroke_width: DimensionValue,
    /// Normalized radial inset ratio for the diagonal cross inside the clear swatch (`0.28`).
    pub clear_cross_inset_ratio: DimensionValue,
    /// Radial expansion applied when a swatch dot is hovered (`1.0pt`).
    pub hover_radius_delta: DimensionValue,
    /// Stroke width of the inner vector checkmark on the selected color swatch (`1.4pt`).
    pub checkmark_stroke_width: DimensionValue,
    /// Normalized start X of the checkmark inside the swatch diameter (`0.26`).
    pub check_p1_x: DimensionValue,
    /// Normalized start Y of the checkmark inside the swatch diameter (`0.52`).
    pub check_p1_y: DimensionValue,
    /// Normalized vertex X of the checkmark inside the swatch diameter (`0.44`).
    pub check_p2_x: DimensionValue,
    /// Normalized vertex Y of the checkmark inside the swatch diameter (`0.70`).
    pub check_p2_y: DimensionValue,
    /// Normalized end X of the checkmark inside the swatch diameter (`0.74`).
    pub check_p3_x: DimensionValue,
    /// Normalized end Y of the checkmark inside the swatch diameter (`0.32`).
    pub check_p3_y: DimensionValue,
    /// Default row count for `MacTagColorPicker` (`1.0`).
    pub default_rows: DimensionValue,
    /// Default horizontal column gap when `.rows(...)` layout is used (`6.0pt`).
    pub default_col_gap: DimensionValue,
    /// Default vertical row gap when `.rows(...)` layout is used (`4.0pt`).
    pub default_row_gap: DimensionValue,
}

impl Default for TagColorPickerMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared()
            .tag_color_picker_metrics()
            .clone()
    }
}

/// Geometry tokens for `MacTableActionBar` (`NSBezelStyleSmallSquare` `[ + | - ]` action strip).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableActionBarMetrics {
    /// Regular action bar height (`22.0pt`).
    pub bar_height: DimensionValue,
    /// Small / CompactInfobox action bar height (`18.0pt`).
    pub small_bar_height: DimensionValue,
    /// Mini action bar height (`15.0pt`).
    pub mini_bar_height: DimensionValue,
    /// Regular individual `+` / `-` button segment width (`24.0pt`).
    pub button_width: DimensionValue,
    /// Small / CompactInfobox individual `+` / `-` button segment width (`20.0pt`).
    pub small_button_width: DimensionValue,
    /// Mini individual `+` / `-` button segment width (`17.0pt`).
    pub mini_button_width: DimensionValue,
    /// Regular wider `[ ⚙ ⌄ ]` action menu segment width (`32.0pt`).
    pub action_button_width: DimensionValue,
    /// Small / CompactInfobox wider `[ ⚙ ⌄ ]` action menu segment width (`27.0pt`).
    pub small_action_button_width: DimensionValue,
    /// Mini wider `[ ⚙ ⌄ ]` action menu segment width (`23.0pt`).
    pub mini_action_button_width: DimensionValue,
    /// Hairline border and segment divider stroke width (`1.0pt`).
    pub border_width: DimensionValue,
    /// Outer corner radius of the `NSBezelStyleSmallSquare` bezel (`1.0pt`).
    pub corner_radius: DimensionValue,
    /// Regular symbol glyph font size (`13.0pt`).
    pub glyph_size: DimensionValue,
    /// Small / Mini symbol glyph font size (`11.0pt`).
    pub small_glyph_size: DimensionValue,
    /// Vector `+` / `-` glyph stroke width (`1.35pt`).
    pub glyph_stroke_width: DimensionValue,
    /// Regular half-length of vector `+` / `-` arms (`4.0pt`).
    pub glyph_arm_half: DimensionValue,
    /// Small / Mini half-length of vector `+` / `-` arms (`3.25pt`).
    pub small_glyph_arm_half: DimensionValue,
    /// Horizontal gap between the gear icon and downward chevron in the action segment (`3.0pt`).
    pub action_chevron_gap: DimensionValue,
    /// Half-width of the downward vector chevron in the action segment (`2.25pt`).
    pub action_chevron_half_width: DimensionValue,
    /// Vertical height of the downward vector chevron in the action segment (`2.0pt`).
    pub action_chevron_height: DimensionValue,
}

impl Default for TableActionBarMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared()
            .table_action_bar_metrics()
            .clone()
    }
}
