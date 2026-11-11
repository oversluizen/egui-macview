//! Advanced AppKit Container & Control geometry metrics tokens (`SPEC-012` & `v0.6.5`).

use super::basic::default_border_width;
use crate::tokens::dtcg::DimensionValue;
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
        Self {
            chevron_size: DimensionValue::from_pt(13.0),
            label_gap: DimensionValue::from_pt(6.0),
            button_regular_width: DimensionValue::from_pt(28.0),
            button_regular_height: DimensionValue::from_pt(26.0),
            button_regular_intrinsic: DimensionValue::from_pt(20.0),
            button_small_width: DimensionValue::from_pt(24.0),
            button_small_height: DimensionValue::from_pt(21.0),
            button_small_intrinsic: DimensionValue::from_pt(16.0),
            button_mini_width: DimensionValue::from_pt(19.0),
            button_mini_height: DimensionValue::from_pt(16.0),
            button_mini_intrinsic: DimensionValue::from_pt(13.0),
            spring_mass: DimensionValue::from_pt(1.0),
            spring_stiffness: DimensionValue::from_pt(300.0),
            spring_damping: DimensionValue::from_pt(25.0),
            header_font_size: DimensionValue::from_pt(12.0),
            trailing_button_size: DimensionValue::from_pt(16.0),
            trailing_plus_half_span: DimensionValue::from_pt(4.0),
            trailing_plus_stroke_width: DimensionValue::from_pt(1.4),
        }
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
        Self {
            regular_height: DimensionValue::from_pt(23.0),
            regular_button_width: DimensionValue::from_pt(27.0),
            regular_inset: DimensionValue::from_pt(9.0),
            regular_corner_radius: DimensionValue::from_pt(5.0),
            small_height: DimensionValue::from_pt(19.0),
            small_button_width: DimensionValue::from_pt(20.0),
            small_inset: DimensionValue::from_pt(7.0),
            small_corner_radius: DimensionValue::from_pt(4.0),
            mini_height: DimensionValue::from_pt(15.0),
            mini_button_width: DimensionValue::from_pt(18.0),
            mini_inset: DimensionValue::from_pt(6.0),
            mini_corner_radius: DimensionValue::from_pt(3.0),
            item_height: DimensionValue::from_pt(18.0),
            visible_items: DimensionValue::from_pt(5.0),
            intercell_spacing_x: DimensionValue::from_pt(3.0),
            intercell_spacing_y: DimensionValue::from_pt(2.0),
            badge_button_width: DimensionValue::from_pt(17.0),
            button_inset: DimensionValue::from_pt(2.0),
            button_corner_radius: DimensionValue::from_pt(4.0),
            chevron_half_width: DimensionValue::from_pt(2.5),
            chevron_half_height: DimensionValue::from_pt(1.5),
            chevron_stroke_width: DimensionValue::from_pt(1.3),
            regular_first_baseline_offset: DimensionValue::from_pt(15.0),
            small_first_baseline_offset: DimensionValue::from_pt(12.0),
            mini_first_baseline_offset: DimensionValue::from_pt(11.0),
            regular_visual_height: DimensionValue::from_pt(20.0),
            small_visual_height: DimensionValue::from_pt(16.0),
            mini_visual_height: DimensionValue::from_pt(13.0),
        }
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
        Self {
            regular_bar_height: DimensionValue::from_pt(24.0),
            regular_inset_top: DimensionValue::from_pt(33.0),
            regular_inset_bottom: DimensionValue::from_pt(13.0),
            regular_inset_horizontal: DimensionValue::from_pt(10.0),
            small_bar_height: DimensionValue::from_pt(20.0),
            small_inset_top: DimensionValue::from_pt(25.0),
            small_inset_bottom: DimensionValue::from_pt(13.0),
            small_inset_horizontal: DimensionValue::from_pt(10.0),
            mini_bar_height: DimensionValue::from_pt(15.0),
            mini_inset_top: DimensionValue::from_pt(19.0),
            mini_inset_bottom: DimensionValue::from_pt(13.0),
            mini_inset_horizontal: DimensionValue::from_pt(10.0),
            corner_radius: DimensionValue::from_pt(5.0),
            regular_min_width: DimensionValue::from_pt(150.0),
            regular_min_height: DimensionValue::from_pt(36.0),
            small_min_width: DimensionValue::from_pt(122.0),
            small_min_height: DimensionValue::from_pt(28.0),
            mini_min_width: DimensionValue::from_pt(107.0),
            mini_min_height: DimensionValue::from_pt(22.0),
            tab_padding_horizontal: DimensionValue::from_pt(24.0),
        }
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
        Self {
            row_height: DimensionValue::from_pt(32.0),
            add_remove_button_size: DimensionValue::from_pt(20.0),
            row_horizontal_spacing: DimensionValue::from_pt(8.0),
        }
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
    #[serde(default = "default_border_width")]
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
        Self {
            inset_top: DimensionValue::from_pt(20.0),
            inset_bottom: DimensionValue::from_pt(10.0),
            inset_left: DimensionValue::from_pt(9.0),
            inset_right: DimensionValue::from_pt(9.0),
            corner_radius: DimensionValue::from_pt(5.0),
            border_width: default_border_width(),
            separator_frame_height: DimensionValue::from_pt(5.0),
            title_height: DimensionValue::from_pt(14.0),
            title_font_size: DimensionValue::from_pt(11.0),
        }
    }
}
