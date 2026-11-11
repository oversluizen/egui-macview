//! macOS Semantic System Color Tokens (NSColor Aqua and DarkAqua).

use super::dtcg::ColorValue;
use serde::{Deserialize, Serialize};

/// Appearance semantic colors for light or dark mode.
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
    #[serde(default = "default_label_color")]
    pub label: ColorValue,
    /// Secondary caption/subtitle label text color (`secondaryLabelColor`).
    #[serde(default = "default_secondary_label_color")]
    pub secondary_label: ColorValue,
    /// System green highlight color (`systemGreenColor`).
    #[serde(default = "default_system_green")]
    pub system_green: ColorValue,
    /// System yellow/amber highlight color (`systemYellowColor`).
    #[serde(default = "default_system_yellow")]
    pub system_yellow: ColorValue,
    /// System orange highlight color (`systemOrangeColor`).
    #[serde(default = "default_system_orange")]
    pub system_orange: ColorValue,
    /// System red alert/critical color (`systemRedColor`).
    #[serde(default = "default_system_red")]
    pub system_red: ColorValue,
    /// Even table row background color (`alternatingContentBackgroundColors[0]`).
    #[serde(default = "default_alternating_row_even")]
    pub alternating_row_even: ColorValue,
    /// Odd table row background color (`alternatingContentBackgroundColors[1]`).
    #[serde(default = "default_alternating_row_odd")]
    pub alternating_row_odd: ColorValue,
    /// QuickTime / AVKit trim selection border and handle color.
    #[serde(default = "default_scrubber_selection_border")]
    pub scrubber_selection_border: ColorValue,
    /// Dimming overlay color outside active trim range.
    #[serde(default = "default_scrubber_outside_dim")]
    pub scrubber_outside_dim: ColorValue,
    /// Waveform bar fill color inside trim scrubber track.
    #[serde(default = "default_scrubber_waveform")]
    pub scrubber_waveform: ColorValue,
    /// Playhead line and timecode badge color.
    #[serde(default = "default_scrubber_playhead")]
    pub scrubber_playhead: ColorValue,
    /// Window close button fill color.
    #[serde(default = "default_traffic_light_close")]
    pub traffic_light_close: ColorValue,
    /// Window close button border stroke color.
    #[serde(default = "default_traffic_light_close_border")]
    pub traffic_light_close_border: ColorValue,
    /// Window minimize button fill color.
    #[serde(default = "default_traffic_light_minimize")]
    pub traffic_light_minimize: ColorValue,
    /// Window minimize button border stroke color.
    #[serde(default = "default_traffic_light_minimize_border")]
    pub traffic_light_minimize_border: ColorValue,
    /// Window zoom/fullscreen button fill color.
    #[serde(default = "default_traffic_light_zoom")]
    pub traffic_light_zoom: ColorValue,
    /// Window zoom/fullscreen button border stroke color.
    #[serde(default = "default_traffic_light_zoom_border")]
    pub traffic_light_zoom_border: ColorValue,
    /// High-contrast white label color on system accent and active controls.
    #[serde(default = "default_on_accent_text")]
    pub on_accent_text: ColorValue,
    /// Pure white surface fill for switch and slider thumb knobs.
    #[serde(default = "default_knob_fill")]
    pub knob_fill: ColorValue,
    /// Subtle ambient drop shadow color for controls and floating surfaces.
    #[serde(default = "default_drop_shadow")]
    pub drop_shadow: ColorValue,
    /// Modal backdrop dimming overlay color.
    #[serde(default = "default_backdrop_dim")]
    pub backdrop_dim: ColorValue,
    /// Translucent dark NSPanelStyleMaskHUDWindow background fill.
    #[serde(default = "default_hud_background")]
    pub hud_background: ColorValue,
    /// Hairline border stroke for dark floating HUD panels.
    #[serde(default = "default_hud_border")]
    pub hud_border: ColorValue,
    /// Primary label color inside dark floating HUD panels.
    #[serde(default = "default_hud_text")]
    pub hud_text: ColorValue,
    /// Secondary coordinate label color inside dark floating HUD panels.
    #[serde(default = "default_hud_secondary_text")]
    pub hud_secondary_text: ColorValue,
    /// Windows 11 caption close button hover fill.
    #[serde(default = "default_caption_close_hover")]
    pub caption_close_hover: ColorValue,
    /// Windows 11 caption close button pressed fill.
    #[serde(default = "default_caption_close_pressed")]
    pub caption_close_pressed: ColorValue,
    /// Windows 11 caption button hover fill.
    #[serde(default = "default_caption_hover")]
    pub caption_hover: ColorValue,
    /// Windows 11 caption button pressed fill.
    #[serde(default = "default_caption_pressed")]
    pub caption_pressed: ColorValue,
    /// Interior glyph stroke color for hovered macOS traffic light buttons.
    #[serde(default = "default_traffic_light_glyph")]
    pub traffic_light_glyph: ColorValue,
    /// Overlay scrollbar thumb base color.
    #[serde(default = "default_scrollbar_thumb")]
    pub scrollbar_thumb: ColorValue,
    /// Overlay scrollbar thumb hovered/dragged color.
    #[serde(default = "default_scrollbar_thumb_active")]
    pub scrollbar_thumb_active: ColorValue,
    /// Tertiary label text color (`NSColor.tertiaryLabelColor`).
    #[serde(default = "default_tertiary_label")]
    pub tertiary_label: ColorValue,
    /// Subtle recessed container surface color for `NSBoxPrimary` (`#E5E5E5` in Aqua, `#232528` in DarkAqua).
    #[serde(default = "default_box_background")]
    pub box_background: ColorValue,
    /// Perimeter hairline border color for `NSBoxPrimary` (`#D8D8D8` in Aqua, `#3A3D42` in DarkAqua).
    #[serde(default = "default_box_border")]
    pub box_border: ColorValue,
}

fn color_val(val: &str) -> ColorValue { ColorValue::new(val) }
fn default_label_color() -> ColorValue { color_val("rgba(0, 0, 0, 0.85)") }
fn default_secondary_label_color() -> ColorValue { color_val("rgba(0, 0, 0, 0.50)") }
fn default_tertiary_label() -> ColorValue { color_val("rgba(0, 0, 0, 0.30)") }
fn default_box_background() -> ColorValue { color_val("#E5E5E5") }
fn default_box_border() -> ColorValue { color_val("#D8D8D8") }
fn default_system_green() -> ColorValue { color_val("#34C759") }
fn default_system_yellow() -> ColorValue { color_val("#FFCC00") }
fn default_system_orange() -> ColorValue { color_val("#FF9500") }
fn default_system_red() -> ColorValue { color_val("#FF3B30") }
fn default_alternating_row_even() -> ColorValue { color_val("#FFFFFF") }
fn default_alternating_row_odd() -> ColorValue { color_val("#F4F5F5") }
fn default_scrubber_selection_border() -> ColorValue { color_val("#FFCC00") }
fn default_scrubber_outside_dim() -> ColorValue { color_val("rgba(0, 0, 0, 0.58)") }
fn default_scrubber_waveform() -> ColorValue { color_val("#007AFF") }
fn default_scrubber_playhead() -> ColorValue { color_val("#FFFFFF") }
fn default_traffic_light_close() -> ColorValue { color_val("#FF5F56") }
fn default_traffic_light_close_border() -> ColorValue { color_val("#E0443E") }
fn default_traffic_light_minimize() -> ColorValue { color_val("#FFBD2E") }
fn default_traffic_light_minimize_border() -> ColorValue { color_val("#D69A1E") }
fn default_traffic_light_zoom() -> ColorValue { color_val("#27C93F") }
fn default_traffic_light_zoom_border() -> ColorValue { color_val("#1BA931") }
fn default_on_accent_text() -> ColorValue { color_val("#FFFFFF") }
fn default_knob_fill() -> ColorValue { color_val("#FFFFFF") }
fn default_drop_shadow() -> ColorValue { color_val("rgba(0, 0, 0, 0.12)") }
fn default_backdrop_dim() -> ColorValue { color_val("rgba(0, 0, 0, 0.40)") }
fn default_hud_background() -> ColorValue { color_val("rgba(30, 30, 32, 0.90)") }
fn default_hud_border() -> ColorValue { color_val("#3C3C3E") }
fn default_hud_text() -> ColorValue { color_val("#F2F2F7") }
fn default_hud_secondary_text() -> ColorValue { color_val("#B4B4B8") }
fn default_caption_close_hover() -> ColorValue { color_val("#E81123") }
fn default_caption_close_pressed() -> ColorValue { color_val("#C42B1C") }
fn default_caption_hover() -> ColorValue { color_val("rgba(0, 0, 0, 0.06)") }
fn default_caption_pressed() -> ColorValue { color_val("rgba(0, 0, 0, 0.11)") }
fn default_traffic_light_glyph() -> ColorValue { color_val("rgba(0, 0, 0, 0.63)") }
fn default_scrollbar_thumb() -> ColorValue { color_val("rgba(0, 0, 0, 0.45)") }
fn default_scrollbar_thumb_active() -> ColorValue { color_val("rgba(0, 0, 0, 0.70)") }

/// Semantic system color tokens for Aqua, Anthracite, and DarkAqua.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacColorTokens {
    /// Light mode appearance colors (`NSAppearanceNameAqua`).
    pub aqua: MacAppearanceColors,
    /// Anthracite studio appearance colors.
    pub anthracite: MacAppearanceColors,
    /// Dark mode appearance colors (`NSAppearanceNameDarkAqua`).
    pub dark_aqua: MacAppearanceColors,
}

/// Generates fallback anthracite studio color values.
pub fn default_anthracite_colors() -> MacAppearanceColors {
    MacAppearanceColors {
        window_background: color_val("#32363E"),
        control_background: color_val("#3C4049"),
        control: color_val("#3C4049"),
        control_accent: color_val("#0A84FF"),
        selected_control: color_val("#415A77"),
        selected_control_inactive: color_val("#444955"),
        separator: color_val("#484D57"),
        placeholder_text: color_val("rgba(255, 255, 255, 0.35)"),
        keyboard_focus_indicator: color_val("rgba(10, 132, 255, 0.50)"),
        label: color_val("#ECEEF2"),
        secondary_label: color_val("#B4BAC4"),
        system_green: color_val("#30D158"),
        system_yellow: color_val("#FFD60A"),
        system_orange: color_val("#FF9F0A"),
        system_red: color_val("#FF453A"),
        alternating_row_even: color_val("#32363E"),
        alternating_row_odd: color_val("rgba(255, 255, 255, 0.04)"),
        scrubber_selection_border: color_val("#FFD60A"),
        scrubber_outside_dim: color_val("rgba(0, 0, 0, 0.58)"),
        scrubber_waveform: color_val("#0A84FF"),
        scrubber_playhead: color_val("#FFFFFF"),
        traffic_light_close: color_val("#FF5F56"),
        traffic_light_close_border: color_val("#E0443E"),
        traffic_light_minimize: color_val("#FFBD2E"),
        traffic_light_minimize_border: color_val("#D69A1E"),
        traffic_light_zoom: color_val("#27C93F"),
        traffic_light_zoom_border: color_val("#1BA931"),
        on_accent_text: color_val("#FFFFFF"),
        knob_fill: color_val("#FFFFFF"),
        drop_shadow: color_val("rgba(0, 0, 0, 0.30)"),
        backdrop_dim: color_val("rgba(0, 0, 0, 0.50)"),
        hud_background: color_val("rgba(30, 30, 32, 0.90)"),
        hud_border: color_val("#3C3C3E"),
        hud_text: color_val("#F2F2F7"),
        hud_secondary_text: color_val("#B4B4B8"),
        caption_close_hover: color_val("#E81123"),
        caption_close_pressed: color_val("#C42B1C"),
        caption_hover: color_val("rgba(255, 255, 255, 0.08)"),
        caption_pressed: color_val("rgba(255, 255, 255, 0.12)"),
        traffic_light_glyph: color_val("rgba(0, 0, 0, 0.63)"),
        scrollbar_thumb: color_val("rgba(255, 255, 255, 0.45)"),
        scrollbar_thumb_active: color_val("rgba(255, 255, 255, 0.70)"),
        tertiary_label: color_val("rgba(255, 255, 255, 0.35)"),
        box_background: color_val("#2B2E35"),
        box_border: color_val("#424650"),
    }
}
