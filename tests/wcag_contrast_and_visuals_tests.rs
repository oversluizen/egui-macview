//! Automated WCAG 2.1 AA Contrast and egui Visuals Synchronization Tests.
//!
//! Normative References:
//! - WCAG 2.1 §1.4.3: Contrast (Minimum) Level AA (>= 4.5:1 for normal text, >= 3.0:1 for large text).
//! - Apple HIG: Dark Mode typography and semantic colors (NSColor.labelColor, NSColor.secondaryLabelColor).
//! - GL-1015: Regel 0 (De Verankeringswet) & Regel 7 (ISO/IEC/IEEE 29119 Spec-First).

use egui::{Color32, Context};
use egui_macview::theme::{MacAppearance, MacTheme};

/// Calculates linear luminance component for an sRGB channel in 0.0..=1.0.
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Calculates relative luminance according to WCAG 2.1 definition.
fn relative_luminance(color: Color32) -> f32 {
    let r = srgb_to_linear(color.r() as f32 / 255.0);
    let g = srgb_to_linear(color.g() as f32 / 255.0);
    let b = srgb_to_linear(color.b() as f32 / 255.0);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Computes WCAG 2.1 contrast ratio between foreground and background colors.
/// If foreground has alpha < 255, it is alpha-blended over the background first.
fn contrast_ratio(fg: Color32, bg: Color32) -> f32 {
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

#[test]
fn test_wcag_contrast_primary_labels() {
    let aqua = MacTheme::new(MacAppearance::Aqua);
    let dark_aqua = MacTheme::new(MacAppearance::DarkAqua);

    let aqua_contrast = contrast_ratio(aqua.label(), aqua.window_background());
    assert!(
        aqua_contrast >= 4.5,
        "WCAG 2.1 AA Violation in Aqua: label contrast {aqua_contrast:.2}:1 is below 4.5:1"
    );

    let dark_contrast = contrast_ratio(dark_aqua.label(), dark_aqua.window_background());
    assert!(
        dark_contrast >= 4.5,
        "WCAG 2.1 AA Violation in DarkAqua: label contrast {dark_contrast:.2}:1 is below 4.5:1"
    );
}

#[test]
fn test_wcag_contrast_secondary_and_stack_header_labels() {
    let dark_aqua = MacTheme::new(MacAppearance::DarkAqua);

    // Stack headers and secondary labels MUST meet WCAG 2.1 AA (>= 4.5:1 for regular text, >= 3.0:1 for bold/headers)
    // Currently, placeholder_text (25% opacity) yields ~2.2:1 contrast which fails catastrophically!
    let secondary = dark_aqua.secondary_label();
    let contrast = contrast_ratio(secondary, dark_aqua.window_background());

    assert!(
        contrast >= 4.5,
        "WCAG 2.1 AA Violation in DarkAqua: secondary label contrast {contrast:.2}:1 is below 4.5:1"
    );
}

#[test]
fn test_egui_visuals_synchronization_dark_mode() {
    let dark_aqua = MacTheme::new(MacAppearance::DarkAqua);
    let ctx = Context::default();

    // Must synchronize egui::Visuals with macOS appearance
    dark_aqua.apply_to_egui(&ctx);

    let visuals = ctx.style_of(ctx.theme()).visuals.clone();
    assert!(
        visuals.dark_mode,
        "egui Visuals was not set to dark mode for DarkAqua appearance!"
    );

    // Verify non-interactive label foreground color meets WCAG contrast against window background
    let egui_text_color = visuals.widgets.noninteractive.fg_stroke.color;
    let contrast = contrast_ratio(egui_text_color, dark_aqua.window_background());
    assert!(
        contrast >= 4.5,
        "egui noninteractive text color in DarkAqua has insufficient contrast: {contrast:.2}:1"
    );
}
