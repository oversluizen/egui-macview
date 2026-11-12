//! Advanced AppKit Container & Control geometry metrics tokens (`SPEC-012` & `v0.6.5`).

use crate::tokens::dtcg::DimensionValue;
use crate::tokens::MacDesignTokens;
use serde::{Deserialize, Serialize};

/// `MacDisclosureGroup` & `MacDisclosureButton` geometry and spring physics metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisclosureGroupMetrics {
    /// Disclosure triangle bounding size (`13.0 pt`).
    pub chevron_size: DimensionValue,
    /// Gap between chevron and label (`6.0 pt`).
    pub label_gap: DimensionValue,
    /// Regular disclosure button width (`28.0 pt`).
    pub button_regular_width: DimensionValue,
    /// Regular disclosure button height (`26.0 pt`).
    pub button_regular_height: DimensionValue,
    /// Regular disclosure button intrinsic visual size (`20.0 pt`).
    pub button_regular_intrinsic: DimensionValue,
    /// Small disclosure button width (`24.0 pt`).
    pub button_small_width: DimensionValue,
    /// Small disclosure button height (`21.0 pt`).
    pub button_small_height: DimensionValue,
    /// Small disclosure button intrinsic visual size (`16.0 pt`).
    pub button_small_intrinsic: DimensionValue,
    /// Mini disclosure button width (`19.0 pt`).
    pub button_mini_width: DimensionValue,
    /// Mini disclosure button height (`16.0 pt`).
    pub button_mini_height: DimensionValue,
    /// Mini disclosure button intrinsic visual size (`13.0 pt`).
    pub button_mini_intrinsic: DimensionValue,
    /// CASpring mass (`1.0`).
    pub spring_mass: DimensionValue,
    /// CASpring stiffness (`300.0`).
    pub spring_stiffness: DimensionValue,
    /// CASpring damping (`25.0`).
    pub spring_damping: DimensionValue,
    /// Section header label font size (`12.0 pt`).
    pub header_font_size: DimensionValue,
    /// Trailing `+` action button hit size (`16.0 pt`).
    pub trailing_button_size: DimensionValue,
    /// Trailing `+` cross arm half-span (`4.0 pt`).
    pub trailing_plus_half_span: DimensionValue,
    /// Trailing `+` stroke thickness (`1.4 pt`).
    pub trailing_plus_stroke_width: DimensionValue,
}

impl Default for DisclosureGroupMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared()
            .disclosure_group_metrics()
            .clone()
    }
}

/// `MacComboBox` (`NSComboBoxCell`) geometry and dropdown metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComboBoxMetrics {
    /// Regular total height (`23.0 pt`).
    pub regular_height: DimensionValue,
    /// Regular trigger button zone width (`27.0 pt`).
    pub regular_button_width: DimensionValue,
    /// Regular horizontal text inset (`9.0 pt`).
    pub regular_inset: DimensionValue,
    /// Regular corner radius (`5.0 pt`).
    pub regular_corner_radius: DimensionValue,
    /// Small total height (`19.0 pt`).
    pub small_height: DimensionValue,
    /// Small trigger button zone width (`20.0 pt`).
    pub small_button_width: DimensionValue,
    /// Small horizontal text inset (`7.0 pt`).
    pub small_inset: DimensionValue,
    /// Small corner radius (`4.0 pt`).
    pub small_corner_radius: DimensionValue,
    /// Mini total height (`15.0 pt`).
    pub mini_height: DimensionValue,
    /// Mini trigger button zone width (`18.0 pt`).
    pub mini_button_width: DimensionValue,
    /// Mini horizontal text inset (`6.0 pt`).
    pub mini_inset: DimensionValue,
    /// Mini corner radius (`3.0 pt`).
    pub mini_corner_radius: DimensionValue,
    /// Dropdown row height (`18.0 pt`).
    pub item_height: DimensionValue,
    /// Default number of visible items (`5.0`).
    pub visible_items: DimensionValue,
    /// Intercell horizontal spacing (`3.0 pt`).
    pub intercell_spacing_x: DimensionValue,
    /// Intercell vertical spacing (`2.0 pt`).
    pub intercell_spacing_y: DimensionValue,
    /// Right accent button badge width (`17.0 pt`).
    pub badge_button_width: DimensionValue,
    /// Right accent button badge inset from field bezel (`2.0 pt`).
    pub button_inset: DimensionValue,
    /// Right accent button badge corner radius (`4.0 pt`).
    pub button_corner_radius: DimensionValue,
    /// Badge chevron horizontal half-width (`2.5 pt`).
    pub chevron_half_width: DimensionValue,
    /// Badge chevron vertical half-height (`1.5 pt`).
    pub chevron_half_height: DimensionValue,
    /// Badge chevron stroke width (`1.3 pt`).
    pub chevron_stroke_width: DimensionValue,
    /// AppKit `firstBaselineOffsetFromTop` for Regular combo box (`15.0 pt`).
    pub regular_first_baseline_offset: DimensionValue,
    /// AppKit `firstBaselineOffsetFromTop` for Small combo box (`12.0 pt`).
    pub small_first_baseline_offset: DimensionValue,
    /// AppKit `firstBaselineOffsetFromTop` for Mini combo box (`11.0 pt`).
    pub mini_first_baseline_offset: DimensionValue,
    /// AppKit visual bezel height for Regular combo box inside a 23pt NIB frame (`20.0 pt`).
    pub regular_visual_height: DimensionValue,
    /// AppKit visual bezel height for Small combo box inside a 19pt NIB frame (`16.0 pt`).
    pub small_visual_height: DimensionValue,
    /// AppKit visual bezel height for Mini combo box inside a 15pt NIB frame (`13.0 pt`).
    pub mini_visual_height: DimensionValue,
}

impl Default for ComboBoxMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().combo_box_metrics().clone()
    }
}

/// `MacTabView` geometry and content inset metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabViewMetrics {
    /// Regular tab bar height (`24.0 pt`).
    pub regular_bar_height: DimensionValue,
    /// Regular top content inset (`33.0 pt`).
    pub regular_inset_top: DimensionValue,
    /// Regular bottom content inset (`13.0 pt`).
    pub regular_inset_bottom: DimensionValue,
    /// Regular left/right content inset (`10.0 pt`).
    pub regular_inset_horizontal: DimensionValue,
    /// Small tab bar height (`20.0 pt`).
    pub small_bar_height: DimensionValue,
    /// Small top content inset (`25.0 pt`).
    pub small_inset_top: DimensionValue,
    /// Small bottom content inset (`13.0 pt`).
    pub small_inset_bottom: DimensionValue,
    /// Small left/right content inset (`10.0 pt`).
    pub small_inset_horizontal: DimensionValue,
    /// Mini tab bar height (`15.0 pt`).
    pub mini_bar_height: DimensionValue,
    /// Mini top content inset (`19.0 pt`).
    pub mini_inset_top: DimensionValue,
    /// Mini bottom content inset (`13.0 pt`).
    pub mini_inset_bottom: DimensionValue,
    /// Mini left/right content inset (`10.0 pt`).
    pub mini_inset_horizontal: DimensionValue,
    /// Content box corner radius (`5.0 pt`).
    pub corner_radius: DimensionValue,
    /// Regular minimum width (`150.0 pt`).
    pub regular_min_width: DimensionValue,
    /// Regular minimum height (`36.0 pt`).
    pub regular_min_height: DimensionValue,
    /// Small minimum width (`122.0 pt`).
    pub small_min_width: DimensionValue,
    /// Small minimum height (`28.0 pt`).
    pub small_min_height: DimensionValue,
    /// Mini minimum width (`107.0 pt`).
    pub mini_min_width: DimensionValue,
    /// Mini minimum height (`22.0 pt`).
    pub mini_min_height: DimensionValue,
    /// Tab padding horizontal (`24.0 pt`).
    pub tab_padding_horizontal: DimensionValue,
}

impl Default for TabViewMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().tab_view_metrics().clone()
    }
}

/// `MacRuleEditor` layout and button metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleEditorMetrics {
    /// Predicate row height (`32.0 pt`).
    pub row_height: DimensionValue,
    /// Action [+] / [-] button size (`20.0 pt`).
    pub add_remove_button_size: DimensionValue,
    /// Spacing between rule clauses (`8.0 pt`).
    pub row_horizontal_spacing: DimensionValue,
}

impl Default for RuleEditorMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared()
            .rule_editor_metrics()
            .clone()
    }
}

/// `MacBox` container and separator metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoxViewMetrics {
    /// Inset from border top (`20.0 pt`).
    pub inset_top: DimensionValue,
    /// Inset from border bottom (`10.0 pt`).
    pub inset_bottom: DimensionValue,
    /// Inset from border left (`9.0 pt`).
    pub inset_left: DimensionValue,
    /// Inset from border right (`9.0 pt`).
    pub inset_right: DimensionValue,
    /// Box corner radius (`5.0 pt`).
    pub corner_radius: DimensionValue,
    /// Box border stroke width (`1.0 pt`).
    pub border_width: DimensionValue,
    /// Separator frame height (`5.0 pt`).
    pub separator_frame_height: DimensionValue,
    /// Title region height (`14.0 pt`).
    pub title_height: DimensionValue,
    /// Title typography font size (`11.0 pt`).
    pub title_font_size: DimensionValue,
}

impl Default for BoxViewMetrics {
    fn default() -> Self {
        MacDesignTokens::default_shared().box_view_metrics().clone()
    }
}
