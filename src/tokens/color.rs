//! macOS Semantic System Color Tokens (NSColor Aqua, Anthracite, and DarkAqua).

use super::dtcg::ColorValue;
use super::tag_colors::MacTagLabelColors;
use serde::{Deserialize, Serialize};

/// Appearance semantic colors for light, anthracite, or dark mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacAppearanceColors {
    /// Window background color (`windowBackgroundColor`).
    pub window_background: ColorValue,
    /// Control background color (`controlBackgroundColor`).
    pub control_background: ColorValue,
    /// Standard control fill color (`controlColor`).
    pub control: ColorValue,
    /// System accent fill color (`controlAccentColor`).
    pub control_accent: ColorValue,
    /// Selected control fill color (`selectedControlColor`).
    pub selected_control: ColorValue,
    /// Selected control fill color when window is inactive.
    pub selected_control_inactive: ColorValue,
    /// Hairline separator border color (`separatorColor`).
    pub separator: ColorValue,
    /// Placeholder text color (`placeholderTextColor`).
    pub placeholder_text: ColorValue,
    /// Keyboard focus indicator ring color (`keyboardFocusIndicatorColor`).
    pub keyboard_focus_indicator: ColorValue,
    /// Primary label text color (`labelColor`).
    pub label: ColorValue,
    /// Secondary caption/subtitle label text color (`secondaryLabelColor`).
    pub secondary_label: ColorValue,
    /// System green highlight color (`systemGreenColor`).
    pub system_green: ColorValue,
    /// System yellow/amber highlight color (`systemYellowColor`).
    pub system_yellow: ColorValue,
    /// System orange highlight color (`systemOrangeColor`).
    pub system_orange: ColorValue,
    /// System red alert/critical color (`systemRedColor`).
    pub system_red: ColorValue,
    /// Even table row background color (`alternatingContentBackgroundColors[0]`).
    pub alternating_row_even: ColorValue,
    /// Odd table row background color (`alternatingContentBackgroundColors[1]`).
    pub alternating_row_odd: ColorValue,
    /// QuickTime / AVKit trim selection border and handle color.
    pub scrubber_selection_border: ColorValue,
    /// Dimming overlay color outside active trim range.
    pub scrubber_outside_dim: ColorValue,
    /// Waveform bar fill color inside trim scrubber track.
    pub scrubber_waveform: ColorValue,
    /// Playhead line and timecode badge color.
    pub scrubber_playhead: ColorValue,
    /// Window close button fill color.
    pub traffic_light_close: ColorValue,
    /// Window close button border stroke color.
    pub traffic_light_close_border: ColorValue,
    /// Window minimize button fill color.
    pub traffic_light_minimize: ColorValue,
    /// Window minimize button border stroke color.
    pub traffic_light_minimize_border: ColorValue,
    /// Window zoom/fullscreen button fill color.
    pub traffic_light_zoom: ColorValue,
    /// Window zoom/fullscreen button border stroke color.
    pub traffic_light_zoom_border: ColorValue,
    /// High-contrast white label color on system accent and active controls.
    pub on_accent_text: ColorValue,
    /// Pure white surface fill for switch and slider thumb knobs.
    pub knob_fill: ColorValue,
    /// Subtle ambient drop shadow color for controls and floating surfaces.
    pub drop_shadow: ColorValue,
    /// Modal backdrop dimming overlay color.
    pub backdrop_dim: ColorValue,
    /// Translucent dark NSPanelStyleMaskHUDWindow background fill.
    pub hud_background: ColorValue,
    /// Hairline border stroke for dark floating HUD panels.
    pub hud_border: ColorValue,
    /// Primary label color inside dark floating HUD panels.
    pub hud_text: ColorValue,
    /// Secondary coordinate label color inside dark floating HUD panels.
    pub hud_secondary_text: ColorValue,
    /// Windows 11 caption close button hover fill.
    pub caption_close_hover: ColorValue,
    /// Windows 11 caption close button pressed fill.
    pub caption_close_pressed: ColorValue,
    /// Windows 11 caption button hover fill.
    pub caption_hover: ColorValue,
    /// Windows 11 caption button pressed fill.
    pub caption_pressed: ColorValue,
    /// Interior glyph stroke color for hovered macOS traffic light buttons.
    pub traffic_light_glyph: ColorValue,
    /// Overlay scrollbar thumb base color.
    pub scrollbar_thumb: ColorValue,
    /// Overlay scrollbar thumb hovered/dragged color.
    pub scrollbar_thumb_active: ColorValue,
    /// Tertiary label text color (`NSColor.tertiaryLabelColor`).
    pub tertiary_label: ColorValue,
    /// Subtle recessed container surface color for `NSBoxPrimary` (`#E5E5E5` in Aqua, `#232528` in DarkAqua).
    pub box_background: ColorValue,
    /// Perimeter hairline border color for `NSBoxPrimary` (`#D8D8D8` in Aqua, `#3A3D42` in DarkAqua).
    pub box_border: ColorValue,
    /// Neutral `NSSlider` track background fill (`#DCDCDC` in Aqua, `#3E3E42` in DarkAqua).
    pub slider_track_bg: ColorValue,
    /// Neutral `NSSlider` track border stroke (`#C4C4C4` in Aqua, `#545458` in DarkAqua).
    pub slider_track_border: ColorValue,
    /// `NSSlider` tick mark stroke color (`#B8B8B8` in Aqua, `#666666` in DarkAqua).
    pub slider_tick_mark: ColorValue,
    /// `NSSlider` thumb fill color (`#FFFFFF` in Aqua, `#D8D8DC` in DarkAqua).
    pub slider_thumb_bg: ColorValue,
    /// `NSSlider` thumb border stroke color (`#ADADAD` in Aqua, `#202022` in DarkAqua).
    pub slider_thumb_border: ColorValue,
    /// Active segment fill for `NSSegmentedControl` `.smallSquare` (`#D4D4D4` in Aqua, `#525257` in DarkAqua).
    pub segmented_small_square_active_bg: ColorValue,
    /// Inactive segment fill for `NSSegmentedControl` `.smallSquare` (`#F4F4F4` in Aqua, `#36363A` in DarkAqua).
    pub segmented_small_square_inactive_bg: ColorValue,
    /// `NSPopover` container surface background color (`#F4F4F6` in Aqua, `#2B2B2E` in DarkAqua).
    pub popover_background: ColorValue,
    /// `NSPopover` container subtle perimeter border color (`#D2D2D7` in Aqua, `#48484C` in DarkAqua).
    pub popover_border: ColorValue,
}

/// Semantic system color tokens for Aqua, Anthracite, and DarkAqua.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacColorTokens {
    /// Light mode appearance colors (`NSAppearanceNameAqua`).
    pub aqua: MacAppearanceColors,
    /// Anthracite studio appearance colors.
    pub anthracite: MacAppearanceColors,
    /// Dark mode appearance colors (`NSAppearanceNameDarkAqua`).
    pub dark_aqua: MacAppearanceColors,
    /// Semantic Finder / AppKit tag swatch label colors.
    pub tag_labels: MacTagLabelColors,
}
