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

/// Semantic system color tokens for Aqua and DarkAqua.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacColorTokens {
    /// Light mode appearance colors (`NSAppearanceNameAqua`).
    pub aqua: MacAppearanceColors,
    /// Dark mode appearance colors (`NSAppearanceNameDarkAqua`).
    pub dark_aqua: MacAppearanceColors,
}
