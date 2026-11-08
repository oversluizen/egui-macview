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
}

fn default_label_color() -> ColorValue {
    ColorValue {
        value: "rgba(0, 0, 0, 0.85)".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_secondary_label_color() -> ColorValue {
    ColorValue {
        value: "rgba(0, 0, 0, 0.50)".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_system_green() -> ColorValue {
    ColorValue {
        value: "#34C759".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_system_yellow() -> ColorValue {
    ColorValue {
        value: "#FFCC00".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_system_orange() -> ColorValue {
    ColorValue {
        value: "#FF9500".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_system_red() -> ColorValue {
    ColorValue {
        value: "#FF3B30".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_alternating_row_even() -> ColorValue {
    ColorValue {
        value: "#FFFFFF".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_alternating_row_odd() -> ColorValue {
    ColorValue {
        value: "#F4F5F5".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_scrubber_selection_border() -> ColorValue {
    ColorValue {
        value: "#FFCC00".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_scrubber_outside_dim() -> ColorValue {
    ColorValue {
        value: "rgba(0, 0, 0, 0.58)".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_scrubber_waveform() -> ColorValue {
    ColorValue {
        value: "#007AFF".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_scrubber_playhead() -> ColorValue {
    ColorValue {
        value: "#FFFFFF".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_traffic_light_close() -> ColorValue {
    ColorValue {
        value: "#FF5F56".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_traffic_light_close_border() -> ColorValue {
    ColorValue {
        value: "#E0443E".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_traffic_light_minimize() -> ColorValue {
    ColorValue {
        value: "#FFBD2E".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_traffic_light_minimize_border() -> ColorValue {
    ColorValue {
        value: "#D69A1E".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_traffic_light_zoom() -> ColorValue {
    ColorValue {
        value: "#27C93F".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_traffic_light_zoom_border() -> ColorValue {
    ColorValue {
        value: "#1BA931".to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn color_val(val: &str) -> ColorValue {
    ColorValue {
        value: val.to_string(),
        token_type: Some("color".to_string()),
        description: None,
    }
}

fn default_on_accent_text() -> ColorValue {
    color_val("#FFFFFF")
}
fn default_knob_fill() -> ColorValue {
    color_val("#FFFFFF")
}
fn default_drop_shadow() -> ColorValue {
    color_val("rgba(0, 0, 0, 0.12)")
}
fn default_backdrop_dim() -> ColorValue {
    color_val("rgba(0, 0, 0, 0.40)")
}
fn default_hud_background() -> ColorValue {
    color_val("rgba(30, 30, 32, 0.90)")
}
fn default_hud_border() -> ColorValue {
    color_val("#3C3C3E")
}
fn default_hud_text() -> ColorValue {
    color_val("#F2F2F7")
}
fn default_hud_secondary_text() -> ColorValue {
    color_val("#B4B4B8")
}
fn default_caption_close_hover() -> ColorValue {
    color_val("#E81123")
}
fn default_caption_close_pressed() -> ColorValue {
    color_val("#C42B1C")
}
fn default_caption_hover() -> ColorValue {
    color_val("rgba(0, 0, 0, 0.06)")
}
fn default_caption_pressed() -> ColorValue {
    color_val("rgba(0, 0, 0, 0.11)")
}
fn default_traffic_light_glyph() -> ColorValue {
    color_val("rgba(0, 0, 0, 0.63)")
}
fn default_scrollbar_thumb() -> ColorValue {
    color_val("rgba(0, 0, 0, 0.45)")
}
fn default_scrollbar_thumb_active() -> ColorValue {
    color_val("rgba(0, 0, 0, 0.70)")
}

/// Semantic system color tokens for Aqua and DarkAqua.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacColorTokens {
    /// Light mode appearance colors (`NSAppearanceNameAqua`).
    pub aqua: MacAppearanceColors,
    /// Dark mode appearance colors (`NSAppearanceNameDarkAqua`).
    pub dark_aqua: MacAppearanceColors,
}
