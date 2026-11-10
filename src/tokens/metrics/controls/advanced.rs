//! Advanced AppKit & AVKit control geometry metrics tokens (SPEC-012).
//!
//! Grounded in Apple AppKit Sonoma 14.8.9 hardware inspection (Darwin 23.6.0).

use crate::tokens::dtcg::DimensionValue;
use crate::tokens::metrics::controls::basic::default_border_width;
use serde::{Deserialize, Serialize};

/// `MacDisclosureGroup` and `MacDisclosureButton` geometry metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisclosureGroupMetrics {
    /// Invariant chevron bounding box size (13.0 pt).
    pub chevron_size: DimensionValue,
    /// Horizontal gap between chevron and trailing label (6.0 pt).
    pub label_gap: DimensionValue,
    /// Regular rounded pill disclosure button width (28.0 pt).
    pub button_regular_width: DimensionValue,
    /// Regular rounded pill disclosure button height (26.0 pt).
    pub button_regular_height: DimensionValue,
    /// Regular rounded pill disclosure button intrinsic size (20.0 pt).
    pub button_regular_intrinsic: DimensionValue,
    /// Small rounded pill disclosure button width (24.0 pt).
    pub button_small_width: DimensionValue,
    /// Small rounded pill disclosure button height (21.0 pt).
    pub button_small_height: DimensionValue,
    /// Small rounded pill disclosure button intrinsic size (16.0 pt).
    pub button_small_intrinsic: DimensionValue,
    /// Mini rounded pill disclosure button width (19.0 pt).
    pub button_mini_width: DimensionValue,
    /// Mini rounded pill disclosure button height (16.0 pt).
    pub button_mini_height: DimensionValue,
    /// Mini rounded pill disclosure button intrinsic size (13.0 pt).
    pub button_mini_intrinsic: DimensionValue,
    /// CoreAnimation spring animation mass (1.0).
    pub spring_mass: DimensionValue,
    /// CoreAnimation spring animation stiffness (300.0).
    pub spring_stiffness: DimensionValue,
    /// CoreAnimation spring animation damping (25.0).
    pub spring_damping: DimensionValue,
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
        }
    }
}

/// `MacComboBox` geometry and popover metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComboBoxMetrics {
    /// Regular frame height (23.0 pt).
    pub regular_height: DimensionValue,
    /// Regular trigger button width (27.0 pt).
    pub regular_button_width: DimensionValue,
    /// Regular text inset (9.0 pt).
    pub regular_inset: DimensionValue,
    /// Regular corner radius (5.0 pt).
    pub regular_corner_radius: DimensionValue,
    /// Small frame height (19.0 pt).
    pub small_height: DimensionValue,
    /// Small trigger button width (20.0 pt).
    pub small_button_width: DimensionValue,
    /// Small text inset (7.0 pt).
    pub small_inset: DimensionValue,
    /// Small corner radius (4.0 pt).
    pub small_corner_radius: DimensionValue,
    /// Mini frame height (15.0 pt).
    pub mini_height: DimensionValue,
    /// Mini trigger button width (18.0 pt).
    pub mini_button_width: DimensionValue,
    /// Mini text inset (6.0 pt).
    pub mini_inset: DimensionValue,
    /// Mini corner radius (3.0 pt).
    pub mini_corner_radius: DimensionValue,
    /// Popover single item row height (18.0 pt).
    pub item_height: DimensionValue,
    /// Default number of visible items before scrolling (5).
    pub visible_items: DimensionValue,
    /// Horizontal intercell spacing in popover (3.0 pt).
    pub intercell_spacing_x: DimensionValue,
    /// Vertical intercell spacing in popover (2.0 pt).
    pub intercell_spacing_y: DimensionValue,
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
        }
    }
}

/// `MacTabView` layout and content inset metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabViewMetrics {
    /// Regular tab bar height (24.0 pt).
    pub regular_bar_height: DimensionValue,
    /// Regular content inset top (33.0 pt).
    pub regular_inset_top: DimensionValue,
    /// Regular content inset bottom (13.0 pt).
    pub regular_inset_bottom: DimensionValue,
    /// Regular content inset left and right (10.0 pt).
    pub regular_inset_horizontal: DimensionValue,
    /// Small tab bar height (20.0 pt).
    pub small_bar_height: DimensionValue,
    /// Small content inset top (25.0 pt).
    pub small_inset_top: DimensionValue,
    /// Small content inset bottom (13.0 pt).
    pub small_inset_bottom: DimensionValue,
    /// Small content inset left and right (10.0 pt).
    pub small_inset_horizontal: DimensionValue,
    /// Mini tab bar height (15.0 pt).
    pub mini_bar_height: DimensionValue,
    /// Mini content inset top (19.0 pt).
    pub mini_inset_top: DimensionValue,
    /// Mini content inset bottom (13.0 pt).
    pub mini_inset_bottom: DimensionValue,
    /// Mini content inset left and right (10.0 pt).
    pub mini_inset_horizontal: DimensionValue,
    /// Tab bezel corner radius (5.0 pt).
    pub corner_radius: DimensionValue,
    /// Regular minimum width (150.0 pt).
    pub regular_min_width: DimensionValue,
    /// Regular minimum height (36.0 pt).
    pub regular_min_height: DimensionValue,
    /// Small minimum width (122.0 pt).
    pub small_min_width: DimensionValue,
    /// Small minimum height (28.0 pt).
    pub small_min_height: DimensionValue,
    /// Mini minimum width (107.0 pt).
    pub mini_min_width: DimensionValue,
    /// Mini minimum height (22.0 pt).
    pub mini_min_height: DimensionValue,
    /// Tab padding horizontal (24.0 pt).
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

/// `MacDatePicker` geometry and element metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatePickerMetrics {
    /// Stepper button width (19.0 pt).
    pub stepper_width: DimensionValue,
    /// Stepper chevron pitch (9.0 pt).
    pub stepper_chevron_pitch: DimensionValue,
    /// Segment selection corner radius (2.5 pt).
    pub selection_corner_radius: DimensionValue,
    /// Regular corner radius (5.0 pt).
    pub corner_radius_regular: DimensionValue,
    /// Small corner radius (4.0 pt).
    pub corner_radius_small: DimensionValue,
    /// Mini corner radius (3.0 pt).
    pub corner_radius_mini: DimensionValue,
    /// DateAndTime Regular width (161.0 pt).
    pub datetime_regular_width: DimensionValue,
    /// DateAndTime Regular height (28.0 pt).
    pub datetime_regular_height: DimensionValue,
    /// DateAndTime Small width (157.0 pt).
    pub datetime_small_width: DimensionValue,
    /// DateAndTime Small height (22.0 pt).
    pub datetime_small_height: DimensionValue,
    /// DateAndTime Mini width (155.0 pt).
    pub datetime_mini_width: DimensionValue,
    /// DateAndTime Mini height (21.0 pt).
    pub datetime_mini_height: DimensionValue,
    /// DateOnly Regular width (97.0 pt).
    pub date_only_regular_width: DimensionValue,
    /// DateOnly Regular height (28.0 pt).
    pub date_only_regular_height: DimensionValue,
    /// DateOnly Small width (93.0 pt).
    pub date_only_small_width: DimensionValue,
    /// DateOnly Small height (22.0 pt).
    pub date_only_small_height: DimensionValue,
    /// DateOnly Mini width (91.0 pt).
    pub date_only_mini_width: DimensionValue,
    /// DateOnly Mini height (21.0 pt).
    pub date_only_mini_height: DimensionValue,
    /// TimeOnly Regular width (81.0 pt).
    pub time_only_regular_width: DimensionValue,
    /// TimeOnly Regular height (28.0 pt).
    pub time_only_regular_height: DimensionValue,
    /// TimeOnly Small width (77.0 pt).
    pub time_only_small_width: DimensionValue,
    /// TimeOnly Small height (22.0 pt).
    pub time_only_small_height: DimensionValue,
    /// TimeOnly Mini width (75.0 pt).
    pub time_only_mini_width: DimensionValue,
    /// TimeOnly Mini height (21.0 pt).
    pub time_only_mini_height: DimensionValue,
    /// Month Calendar matrix width (139.0 pt).
    pub calendar_width: DimensionValue,
    /// Month Calendar matrix height (148.0 pt).
    pub calendar_height: DimensionValue,
    /// Analog Clock face width (122.0 pt).
    pub clock_face_width: DimensionValue,
    /// Analog Clock face height (123.0 pt).
    pub clock_face_height: DimensionValue,
    /// Analog Clock intrinsic circle diameter (119.0 pt).
    pub clock_size: DimensionValue,
    /// Calendar navigation header height (24.0 pt).
    pub calendar_header_height: DimensionValue,
    /// Calendar weekday header height (16.0 pt).
    pub calendar_weekday_height: DimensionValue,
    /// Calendar days matrix height (108.0 pt).
    pub calendar_matrix_height: DimensionValue,
    /// Calendar single day cell height (18.0 pt).
    pub calendar_cell_height: DimensionValue,
}

impl Default for DatePickerMetrics {
    fn default() -> Self {
        Self {
            stepper_width: DimensionValue::from_pt(19.0),
            stepper_chevron_pitch: DimensionValue::from_pt(9.0),
            selection_corner_radius: DimensionValue::from_pt(2.5),
            corner_radius_regular: DimensionValue::from_pt(5.0),
            corner_radius_small: DimensionValue::from_pt(4.0),
            corner_radius_mini: DimensionValue::from_pt(3.0),
            datetime_regular_width: DimensionValue::from_pt(161.0),
            datetime_regular_height: DimensionValue::from_pt(28.0),
            datetime_small_width: DimensionValue::from_pt(157.0),
            datetime_small_height: DimensionValue::from_pt(22.0),
            datetime_mini_width: DimensionValue::from_pt(155.0),
            datetime_mini_height: DimensionValue::from_pt(21.0),
            date_only_regular_width: DimensionValue::from_pt(97.0),
            date_only_regular_height: DimensionValue::from_pt(28.0),
            date_only_small_width: DimensionValue::from_pt(93.0),
            date_only_small_height: DimensionValue::from_pt(22.0),
            date_only_mini_width: DimensionValue::from_pt(91.0),
            date_only_mini_height: DimensionValue::from_pt(21.0),
            time_only_regular_width: DimensionValue::from_pt(81.0),
            time_only_regular_height: DimensionValue::from_pt(28.0),
            time_only_small_width: DimensionValue::from_pt(77.0),
            time_only_small_height: DimensionValue::from_pt(22.0),
            time_only_mini_width: DimensionValue::from_pt(75.0),
            time_only_mini_height: DimensionValue::from_pt(21.0),
            calendar_width: DimensionValue::from_pt(139.0),
            calendar_height: DimensionValue::from_pt(148.0),
            calendar_header_height: DimensionValue::from_pt(24.0),
            calendar_weekday_height: DimensionValue::from_pt(16.0),
            calendar_matrix_height: DimensionValue::from_pt(108.0),
            calendar_cell_height: DimensionValue::from_pt(18.0),
            clock_face_width: DimensionValue::from_pt(122.0),
            clock_face_height: DimensionValue::from_pt(123.0),
            clock_size: DimensionValue::from_pt(119.0),
        }
    }
}

/// `MacCollectionView` layout and selection metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionViewMetrics {
    /// Default item width and height (50.0 pt).
    pub item_size: DimensionValue,
    /// Default minimum line spacing (10.0 pt).
    pub minimum_line_spacing: DimensionValue,
    /// Default minimum interitem spacing (10.0 pt).
    pub minimum_interitem_spacing: DimensionValue,
    /// Default section inset (0.0 pt).
    pub section_inset: DimensionValue,
    /// Selection ring corner radius (6.0 pt).
    pub selection_corner_radius: DimensionValue,
    /// Selection ring stroke width (2.0 pt).
    pub selection_stroke_width: DimensionValue,
    /// Marquee fill alpha opacity (0.20).
    pub marquee_alpha_fill: DimensionValue,
    /// Marquee stroke alpha opacity (0.80).
    pub marquee_alpha_stroke: DimensionValue,
}

impl Default for CollectionViewMetrics {
    fn default() -> Self {
        Self {
            item_size: DimensionValue::from_pt(50.0),
            minimum_line_spacing: DimensionValue::from_pt(10.0),
            minimum_interitem_spacing: DimensionValue::from_pt(10.0),
            section_inset: DimensionValue::from_pt(0.0),
            selection_corner_radius: DimensionValue::from_pt(6.0),
            selection_stroke_width: DimensionValue::from_pt(2.0),
            marquee_alpha_fill: DimensionValue::from_pt(0.20),
            marquee_alpha_stroke: DimensionValue::from_pt(0.80),
        }
    }
}

/// `MacPlayerView` media playback metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerViewMetrics {
    /// Floating HUD pod width (481.0 pt).
    pub hud_width: DimensionValue,
    /// Floating HUD pod height (64.0 pt).
    pub hud_height: DimensionValue,
    /// Floating HUD pod corner radius (16.0 pt).
    pub hud_corner_radius: DimensionValue,
    /// Floating HUD bottom margin (16.0 pt).
    pub hud_bottom_margin: DimensionValue,
    /// Floating HUD auto-hide idle duration in seconds (3.0 s).
    pub auto_hide_seconds: DimensionValue,
    /// HUD top row height (34.0 pt).
    pub top_row_height: DimensionValue,
    /// Scrubber container height (22.5 pt).
    pub scrubber_height: DimensionValue,
    /// Scrubber track height (3.5 pt).
    pub scrubber_track_height: DimensionValue,
    /// Scrubber track corner radius (1.75 pt).
    pub scrubber_track_corner_radius: DimensionValue,
    /// Scrubber knob diameter (8.0 pt).
    pub scrubber_knob_size: DimensionValue,
    /// Inline control bar height (24.0 pt).
    pub inline_height: DimensionValue,
    /// Inline control bar corner radius (0.0 pt).
    pub inline_corner_radius: DimensionValue,
    /// Minimal circular transport width (29.0 pt).
    pub minimal_width: DimensionValue,
    /// Minimal circular transport height (18.0 pt).
    pub minimal_height: DimensionValue,
    /// Skip 15s button width (25.0 pt).
    pub skip_button_width: DimensionValue,
    /// Skip 15s button height (15.0 pt).
    pub skip_button_height: DimensionValue,
    /// Frame step button width (19.0 pt).
    pub step_button_width: DimensionValue,
    /// Frame step button height (21.0 pt).
    pub step_button_height: DimensionValue,
    /// Mute button width (24.0 pt).
    pub mute_button_width: DimensionValue,
    /// Mute button height (17.0 pt).
    pub mute_button_height: DimensionValue,
    /// Volume slider track width (125.0 pt).
    pub volume_slider_width: DimensionValue,
    /// Volume slider track height (18.0 pt).
    pub volume_slider_height: DimensionValue,
    /// Center transport play button width (29.0 pt).
    pub play_button_width: DimensionValue,
    /// Center transport play button height (34.0 pt).
    pub play_button_height: DimensionValue,
    /// Auxiliary AirPlay / PiP button size (20.0 pt).
    pub aux_button_size: DimensionValue,
    /// Inter-button horizontal spacing in HUD (6.0 pt).
    pub button_spacing: DimensionValue,
    /// Playback speed button width (36.0 pt).
    pub speed_button_width: DimensionValue,
    /// Playback speed button height (18.0 pt).
    pub speed_button_height: DimensionValue,
    /// HUD transport icon glyph size (8.0 pt).
    pub icon_size: DimensionValue,
    /// Skip 15s duration in seconds (15.0 s).
    pub skip_duration_seconds: DimensionValue,
    /// Frame stepping frame-rate in FPS (30.0 fps).
    pub frame_step_fps: DimensionValue,
    /// Default fallback duration in seconds (100.0 s).
    pub default_duration_seconds: DimensionValue,
    /// Minimum repaint delay in seconds (0.05 s).
    pub min_repaint_delay_seconds: DimensionValue,
    /// Timeline scrubber buffered range fill opacity alpha (0.28).
    pub buffered_fill_alpha: DimensionValue,
    /// Inline control bar play button width (20.0 pt).
    pub inline_play_button_width: DimensionValue,
    /// Inline control bar play button horizontal inset (8.0 pt).
    pub inline_play_button_inset: DimensionValue,
}

impl Default for PlayerViewMetrics {
    fn default() -> Self {
        Self {
            hud_width: DimensionValue::from_pt(481.0),
            hud_height: DimensionValue::from_pt(64.0),
            hud_corner_radius: DimensionValue::from_pt(16.0),
            hud_bottom_margin: DimensionValue::from_pt(16.0),
            auto_hide_seconds: DimensionValue::from_pt(3.0),
            top_row_height: DimensionValue::from_pt(34.0),
            scrubber_height: DimensionValue::from_pt(22.5),
            scrubber_track_height: DimensionValue::from_pt(3.5),
            scrubber_track_corner_radius: DimensionValue::from_pt(1.8),
            scrubber_knob_size: DimensionValue::from_pt(8.0),
            inline_height: DimensionValue::from_pt(24.0),
            inline_corner_radius: DimensionValue::from_pt(0.0),
            minimal_width: DimensionValue::from_pt(29.0),
            minimal_height: DimensionValue::from_pt(18.0),
            skip_button_width: DimensionValue::from_pt(25.0),
            skip_button_height: DimensionValue::from_pt(15.0),
            step_button_width: DimensionValue::from_pt(19.0),
            step_button_height: DimensionValue::from_pt(21.0),
            mute_button_width: DimensionValue::from_pt(24.0),
            mute_button_height: DimensionValue::from_pt(17.0),
            volume_slider_width: DimensionValue::from_pt(125.0),
            volume_slider_height: DimensionValue::from_pt(18.0),
            play_button_width: DimensionValue::from_pt(29.0),
            play_button_height: DimensionValue::from_pt(34.0),
            aux_button_size: DimensionValue::from_pt(20.0),
            button_spacing: DimensionValue::from_pt(6.0),
            speed_button_width: DimensionValue::from_pt(36.0),
            speed_button_height: DimensionValue::from_pt(18.0),
            icon_size: DimensionValue::from_pt(8.0),
            skip_duration_seconds: DimensionValue::from_pt(15.0),
            frame_step_fps: DimensionValue::from_pt(30.0),
            default_duration_seconds: DimensionValue::from_pt(100.0),
            min_repaint_delay_seconds: DimensionValue::from_pt(0.05),
            buffered_fill_alpha: DimensionValue::from_pt(0.28),
            inline_play_button_width: DimensionValue::from_pt(20.0),
            inline_play_button_inset: DimensionValue::from_pt(8.0),
        }
    }
}

/// `MacRuleEditor` layout and button metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleEditorMetrics {
    /// Predicate row height (32.0 pt).
    pub row_height: DimensionValue,
    /// Action [+] / [-] button size (20.0 pt).
    pub add_remove_button_size: DimensionValue,
    /// Spacing between rule clauses (8.0 pt).
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
    /// Inset from border top (20.0 pt).
    pub inset_top: DimensionValue,
    /// Inset from border bottom (10.0 pt).
    pub inset_bottom: DimensionValue,
    /// Inset from border left (9.0 pt).
    pub inset_left: DimensionValue,
    /// Inset from border right (9.0 pt).
    pub inset_right: DimensionValue,
    /// Box corner radius (5.0 pt).
    pub corner_radius: DimensionValue,
    /// Box border stroke width (1.0 pt).
    #[serde(default = "default_border_width")]
    pub border_width: DimensionValue,
    /// Separator frame height (5.0 pt).
    pub separator_frame_height: DimensionValue,
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
        }
    }
}
