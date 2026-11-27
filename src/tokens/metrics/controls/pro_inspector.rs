//! AppKit Pro-Inspector control metrics (`MacOriginMatrix`, `MacTagColorPicker`, `MacTableActionBar`).

use crate::tokens::dtcg::DimensionValue;
use crate::tokens::MacDesignTokens;
use serde::{Deserialize, Serialize};

/// Geometry tokens for `MacOriginMatrix` (`3x3` 9-point transform origin / anchor pin grid).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OriginMatrixMetrics {
    /// Regular square bounding box size (`34.0pt`, matching `TransformView.nib` origin grid).
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
        MacDesignTokens::default_shared().origin_matrix_metrics().clone()
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
        MacDesignTokens::default_shared().tag_color_picker_metrics().clone()
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
        MacDesignTokens::default_shared().table_action_bar_metrics().clone()
    }
}

/// Geometry tokens for `MacImageView` (`NSImageView` / `NSImageCell`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageViewMetrics {
    /// Default bounding box size (`32.0pt`).
    pub default_size: DimensionValue,
    /// Inner padding for `NSImageFramePhoto` (`2.0pt`).
    pub photo_frame_padding: DimensionValue,
    /// Corner radius for `NSImageFrameGrayBezel` (`4.0pt`).
    pub bezel_corner_radius: DimensionValue,
    /// Frame stroke border width (`1.0pt`).
    pub border_width: DimensionValue,
}

impl Default for ImageViewMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().image_view_metrics().clone()
    }
}

/// Geometry tokens for `MacTextView` (`NSTextView` multi-line editor).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextViewMetrics {
    /// Default proportional font size (`13.0pt`).
    pub font_size: DimensionValue,
    /// Default monospace font size (`12.0pt`).
    pub mono_font_size: DimensionValue,
    /// Line height pitch (`18.0pt`).
    pub line_height: DimensionValue,
    /// Horizontal content inset (`6.0pt`).
    pub padding_horizontal: DimensionValue,
    /// Vertical content inset (`6.0pt`).
    pub padding_vertical: DimensionValue,
    /// Outer scroll-bezel corner radius (`2.0pt`).
    pub corner_radius: DimensionValue,
    /// Outer border stroke width (`1.0pt`).
    pub border_width: DimensionValue,
    /// Line-number gutter width (`32.0pt`).
    pub gutter_width: DimensionValue,
}

impl Default for TextViewMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().text_view_metrics().clone()
    }
}

/// Geometry and timing tokens for `MacContextThumbDial` (`SPEC-014`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThumbDialMetrics {
    /// Outer radius of the tactile rotary jog wheel (`84.0pt`).
    pub outer_radius: DimensionValue,
    /// Outer radius of the inner compass / clover sectors (`68.0pt`).
    pub clover_outer_radius: DimensionValue,
    /// Radius of the central Thumb Rest Hub (`24.0pt`).
    pub hub_radius: DimensionValue,
    /// Angular separation gap in degrees between adjacent sectors (`4.0pt`).
    pub sector_gap_deg: DimensionValue,
    /// Angular sweep in degrees of each cardinal sector (`48.0pt`).
    pub cardinal_sweep_deg: DimensionValue,
    /// Angular sweep in degrees of each diagonal combination sector (`38.0pt`).
    pub diagonal_sweep_deg: DimensionValue,
    /// Maximum press duration in seconds (`0.18pt`) classified as a Tap-to-Latch.
    pub latch_tap_max_secs: DimensionValue,
    /// Radius of the sticky latch indicator dot inside a latched sector (`3.0pt`).
    pub latch_dot_radius: DimensionValue,
    /// Radial inset from `hub_radius` to the sticky latch dot (`7.0pt`).
    pub latch_dot_inset: DimensionValue,
    /// Radial gap between concentric layers (`3.0pt`).
    pub ring_gap: DimensionValue,
    /// Inner radius of the curved bottom Undo/Redo skirt buttons (`88.0pt`).
    pub skirt_inner_radius: DimensionValue,
    /// Outer radius of the curved bottom Undo/Redo skirt buttons (`108.0pt`).
    pub skirt_outer_radius: DimensionValue,
    /// Start angle in degrees of the left bottom curved Undo button (`92.0pt`).
    pub undo_arc_start_deg: DimensionValue,
    /// End angle in degrees of the left bottom curved Undo button (`148.0pt`).
    pub undo_arc_end_deg: DimensionValue,
    /// Start angle in degrees of the right bottom curved Redo button (`32.0pt`).
    pub redo_arc_start_deg: DimensionValue,
    /// End angle in degrees of the right bottom curved Redo button (`88.0pt`).
    pub redo_arc_end_deg: DimensionValue,
    /// Start angle in degrees of the top-left crown Undo button (`-148.0pt`).
    pub top_undo_arc_start_deg: DimensionValue,
    /// End angle in degrees of the top-left crown Undo button (`-92.0pt`).
    pub top_undo_arc_end_deg: DimensionValue,
    /// Start angle in degrees of the top-right crown Redo button (`-88.0pt`).
    pub top_redo_arc_start_deg: DimensionValue,
    /// End angle in degrees of the top-right crown Redo button (`-32.0pt`).
    pub top_redo_arc_end_deg: DimensionValue,
    /// Start angle in degrees of the top cap Undo button (`-118.0pt`).
    pub cap_top_start_deg: DimensionValue,
    /// End angle in degrees of the top cap Undo button (`-62.0pt`).
    pub cap_top_end_deg: DimensionValue,
    /// Start angle in degrees of the bottom cap Redo button (`62.0pt`).
    pub cap_bottom_start_deg: DimensionValue,
    /// End angle in degrees of the bottom cap Redo button (`118.0pt`).
    pub cap_bottom_end_deg: DimensionValue,
    /// Normalized radius ratio of the Fine speed ring inside the Layer 2 Joystick Bowl (`0.45pt`).
    pub joystick_fine_ratio: DimensionValue,
    /// Normalized radius ratio of the Fast speed ring inside the Layer 2 Joystick Bowl (`0.82pt`).
    pub joystick_fast_ratio: DimensionValue,
    /// Radius of the movable glass thumb-stick puck in Layer 2 (`14.0pt`).
    pub joystick_puck_radius: DimensionValue,
    /// Maximum point-nudge speed per frame at full rim deflection in Layer 2 (`12.0pt`).
    pub joystick_max_speed: DimensionValue,
    /// Opacity of the Liquid Glass top specular rim highlight (`0.38pt`).
    pub glass_specular_alpha: DimensionValue,
    /// Opacity of the Liquid Glass cushion fill (`0.82pt`).
    pub glass_fill_alpha: DimensionValue,
    /// Outer width of `MacEdgeThumbwheel` (`34.0pt`).
    pub edge_wheel_width: DimensionValue,
    /// Outer height of `MacEdgeThumbwheel` (`136.0pt`).
    pub edge_wheel_height: DimensionValue,
    /// Inner slot padding around the cylindrical roller in `MacEdgeThumbwheel` (`5.0pt`).
    pub edge_wheel_slot_padding: DimensionValue,
    /// Capsule corner radius of `MacEdgeThumbwheel` (`17.0pt`).
    pub edge_wheel_corner_radius: DimensionValue,
    /// Maximum visible cylindrical angle in degrees (`75.0pt`).
    pub edge_wheel_max_angle_deg: DimensionValue,
    /// Minimum release speed in pt/s that triggers Free-Spin Flywheel inertia (`120.0pt`).
    pub edge_wheel_flick_threshold: DimensionValue,
    /// Exponential friction decay constant for Free-Spin Flywheel (`3.5pt`).
    pub edge_wheel_friction_decay: DimensionValue,
    /// Speed threshold in pt/s below which Free-Spin Flywheel snaps to rest (`4.0pt`).
    pub edge_wheel_stop_threshold: DimensionValue,
    /// Non-linear velocity boost gain for fast flywheel flicks (`0.01pt`).
    pub edge_wheel_boost_gain: DimensionValue,
    /// Height of the bottom Undo/Redo thumb arc (`24.0pt`).
    pub undo_bar_height: DimensionValue,
    /// Gap between outer rotary ring and bottom Undo/Redo arc (`6.0pt`).
    pub undo_bar_gap: DimensionValue,
    /// Corner radius of Undo/Redo buttons (`12.0pt`).
    pub undo_bar_corner_radius: DimensionValue,
    /// Hairline border stroke width (`1.0pt`).
    pub border_stroke_width: DimensionValue,
    /// Active/latched sector border stroke width (`1.5pt`).
    pub active_stroke_width: DimensionValue,
    /// Angular step in degrees per rotary detent tick (`15.0pt`).
    pub detent_step_deg: DimensionValue,
    /// Radial length of each detent tick mark (`4.0pt`).
    pub detent_tick_length: DimensionValue,
    /// Font size for live semantic action labels (`10.0pt`).
    pub label_font_size: DimensionValue,
    /// Font size for desktop shortcut hints (`8.5pt`).
    pub hint_font_size: DimensionValue,
    /// Font size for central hub parameter title (`8.5pt`).
    pub hub_title_font_size: DimensionValue,
    /// Font size for central hub value readout (`10.5pt`).
    pub hub_value_font_size: DimensionValue,
    /// Margin inset when docked in a canvas corner (`12.0pt`).
    pub dock_margin: DimensionValue,
    /// Center angle in degrees for North (`-90.0pt`).
    pub angle_north_deg: DimensionValue,
    /// Center angle in degrees for NorthEast (`-45.0pt`).
    pub angle_north_east_deg: DimensionValue,
    /// Center angle in degrees for East (`0.0pt`).
    pub angle_east_deg: DimensionValue,
    /// Center angle in degrees for SouthEast (`45.0pt`).
    pub angle_south_east_deg: DimensionValue,
    /// Center angle in degrees for South (`90.0pt`).
    pub angle_south_deg: DimensionValue,
    /// Center angle in degrees for SouthWest (`135.0pt`).
    pub angle_south_west_deg: DimensionValue,
    /// Center angle in degrees for West (`180.0pt`).
    pub angle_west_deg: DimensionValue,
    /// Center angle in degrees for NorthWest (`-135.0pt`).
    pub angle_north_west_deg: DimensionValue,
    /// Full circle angle in degrees (`360.0pt`).
    pub full_circle_deg: DimensionValue,
    /// Half circle angle in degrees (`180.0pt`).
    pub half_circle_deg: DimensionValue,
    /// Vertical offset for primary sector label (`4.0pt`).
    pub label_vertical_offset: DimensionValue,
    /// Vertical offset for secondary shortcut hint (`6.0pt`).
    pub hint_vertical_offset: DimensionValue,
    /// Active sector fill opacity (`0.28pt`).
    pub active_fill_opacity: DimensionValue,
    /// Hovered sector fill opacity (`0.14pt`).
    pub hover_fill_opacity: DimensionValue,
    /// Rotary sensitivity factor per degree (`0.02pt`).
    pub rotary_sensitivity: DimensionValue,
    /// Refractive index of Apple Liquid Glass ($\eta_{\text{glass}} = 1.48$).
    pub ior_glass: DimensionValue,
    /// Red wavelength refractive index ($\eta_R = 1.474$).
    pub ior_red: DimensionValue,
    /// Green wavelength refractive index ($\eta_G = 1.480$).
    pub ior_green: DimensionValue,
    /// Blue wavelength refractive index ($\eta_B = 1.488$).
    pub ior_blue: DimensionValue,
    /// Normal-incidence dielectric reflectance ($F_0 = ((\eta - 1)/(\eta + 1))^2 = 0.0375$).
    pub fresnel_f0: DimensionValue,
    /// Bevel curvature width in points (`8.0pt`).
    pub bevel_width_pt: DimensionValue,
    /// Optical dome thickness in points (`6.0pt`).
    pub dome_thickness_pt: DimensionValue,
    /// Dual-Kawase backdrop vibrancy saturation multiplier (`1.50pt`).
    pub vibrancy_chroma_boost: DimensionValue,
    /// Smooth-minimum meniscus merge radius in points (`6.0pt`).
    pub meniscus_merge_k_pt: DimensionValue,
    /// Viscous droplet velocity stretch gain (`0.125pt`).
    pub viscous_stretch_gain: DimensionValue,
    /// Viscous droplet finger-press expansion gain (`0.0625pt`).
    pub viscous_press_gain: DimensionValue,
}

impl Default for ThumbDialMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().thumb_dial_metrics().clone()
    }
}
