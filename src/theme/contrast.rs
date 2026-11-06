//! macOS AppKit Dynamic Contrast & Background Style Engine.
//!
//! Grounded in W3C WCAG 2.1 §1.4.3 (Level AA) and Apple AppKit `NSBackgroundStyle`.
//! Automatically resolves foreground typography tints against dynamic or custom
//! background fills (such as custom accent colors) to guarantee >= 4.5:1 contrast.

use crate::theme::MacTheme;
use egui::Color32;

/// Converts an 8-bit sRGB channel component in `[0..255]` to linear luminance in `[0.0..1.0]`.
pub fn srgb_channel_to_linear(c: u8) -> f32 {
    let s = c as f32 / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// Computes the relative luminance of an sRGB color according to WCAG 2.1:
/// `L = 0.2126 * R_lin + 0.7152 * G_lin + 0.0722 * B_lin`.
pub fn relative_luminance(color: Color32) -> f32 {
    let r = srgb_channel_to_linear(color.r());
    let g = srgb_channel_to_linear(color.g());
    let b = srgb_channel_to_linear(color.b());
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Computes the WCAG 2.1 contrast ratio between foreground and background colors.
/// If foreground has alpha < 255, it is alpha-blended over the background first.
pub fn contrast_ratio(fg: Color32, bg: Color32) -> f32 {
    let alpha = fg.a() as f32 / 255.0;
    let blended_r = ((fg.r() as f32 * alpha) + (bg.r() as f32 * (1.0 - alpha))).round() as u8;
    let blended_g = ((fg.g() as f32 * alpha) + (bg.g() as f32 * (1.0 - alpha))).round() as u8;
    let blended_b = ((fg.b() as f32 * alpha) + (bg.b() as f32 * (1.0 - alpha))).round() as u8;
    let effective_fg = Color32::from_rgb(blended_r, blended_g, blended_b);

    let l1 = relative_luminance(effective_fg);
    let l2 = relative_luminance(bg);

    let lighter = l1.max(l2);
    let darker = l1.min(l2);

    (lighter + 0.05) / (darker + 0.05)
}

/// macOS AppKit background style (`NSBackgroundStyle`).
///
/// Indicates whether an element's background is light or dark, allowing child
/// text, icons, and accessories to select high-contrast tints that satisfy WCAG 2.1 AA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacBackgroundStyle {
    /// Background is light (relative luminance `L > 0.179`). Use dark foreground text.
    #[default]
    Light,
    /// Background is dark (relative luminance `L <= 0.179`). Use light/white foreground text.
    Dark,
}

impl MacBackgroundStyle {
    /// Resolves `MacBackgroundStyle` from a background color's WCAG 2.1 relative luminance.
    ///
    /// Applies the normative WCAG 2.1 AA threshold `L = 0.179`:
    /// - `L <= 0.179` => `Dark` style (white text guarantees >= 4.5:1 contrast)
    /// - `L > 0.179` => `Light` style (dark text guarantees >= 4.5:1 contrast)
    pub fn from_color(bg: Color32) -> Self {
        let lum = relative_luminance(bg);
        if lum <= 0.179 {
            Self::Dark
        } else {
            Self::Light
        }
    }

    /// Primary text color guaranteeing >= 4.5:1 WCAG contrast against this background.
    ///
    /// - `Dark` => 100% white (`#FFFFFF`)
    /// - `Light` => dynamic theme label color (`theme.label()`)
    pub fn primary_text_color(&self, theme: &MacTheme) -> Color32 {
        match self {
            Self::Dark => Color32::from_rgb(255, 255, 255),
            Self::Light => theme.label(),
        }
    }

    /// Secondary text color (subtitles, captions, metadata).
    ///
    /// - `Dark` => 85% white (`rgba(255, 255, 255, 0.85)`)
    /// - `Light` => dynamic theme secondary label color (`theme.secondary_label()`)
    pub fn secondary_text_color(&self, theme: &MacTheme) -> Color32 {
        match self {
            Self::Dark => Color32::from_white_alpha(217),
            Self::Light => theme.secondary_label(),
        }
    }

    /// Tertiary text color (placeholders, disabled hints).
    ///
    /// - `Dark` => 70% white (`rgba(255, 255, 255, 0.70)`)
    /// - `Light` => theme placeholder text (`theme.placeholder_text()`)
    pub fn tertiary_text_color(&self, theme: &MacTheme) -> Color32 {
        match self {
            Self::Dark => Color32::from_white_alpha(179),
            Self::Light => theme.placeholder_text(),
        }
    }

    /// Standalone primary text color without requiring a `MacTheme` reference.
    pub fn primary_text(&self) -> Color32 {
        match self {
            Self::Dark => Color32::from_rgb(255, 255, 255),
            Self::Light => Color32::from_rgb(0, 0, 0),
        }
    }

    /// Standalone secondary text color without requiring a `MacTheme` reference.
    pub fn secondary_text(&self) -> Color32 {
        match self {
            Self::Dark => Color32::from_white_alpha(217),
            Self::Light => Color32::from_black_alpha(166),
        }
    }

    /// Standalone tertiary text color without requiring a `MacTheme` reference.
    pub fn tertiary_text(&self) -> Color32 {
        match self {
            Self::Dark => Color32::from_white_alpha(179),
            Self::Light => Color32::from_black_alpha(115),
        }
    }
}
