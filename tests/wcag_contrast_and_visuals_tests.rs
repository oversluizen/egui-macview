//! Automated WCAG 2.1 AA Contrast and egui Visuals Synchronization Tests.
//!
//! Normative References:
//! - WCAG 2.1 §1.4.3: Contrast (Minimum) Level AA (>= 4.5:1 for normal text, >= 3.0:1 for large text).
//! - Apple HIG: Dark Mode typography and semantic colors (NSColor.labelColor, NSColor.secondaryLabelColor).
//! - Test-Driven Development (TDD / Specification-Driven Verification) & ISO/IEC/IEEE 29119 Spec-First.

use egui::{Color32, Context};
use egui_macview::surfaces::MacCard;
use egui_macview::theme::{
    contrast_ratio as crate_contrast_ratio, relative_luminance as crate_relative_luminance,
    srgb_channel_to_linear, MacAppearance, MacBackgroundStyle, MacTheme,
};

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
    let [fg_r, fg_g, fg_b, fg_a] = fg.to_srgba_unmultiplied();
    let alpha = fg_a as f32 / 255.0;
    let blended_r = ((fg_r as f32 * alpha) + (bg.r() as f32 * (1.0 - alpha))).round() as u8;
    let blended_g = ((fg_g as f32 * alpha) + (bg.g() as f32 * (1.0 - alpha))).round() as u8;
    let blended_b = ((fg_b as f32 * alpha) + (bg.b() as f32 * (1.0 - alpha))).round() as u8;
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

#[test]
fn test_wcag_contrast_anthracite_mode() {
    let anthracite = MacTheme::new(MacAppearance::Anthracite);

    let label_contrast = contrast_ratio(anthracite.label(), anthracite.window_background());
    assert!(
        label_contrast >= 4.5,
        "WCAG 2.1 AA Violation in Anthracite: label contrast {label_contrast:.2}:1 is below 4.5:1"
    );

    let sec_contrast = contrast_ratio(anthracite.secondary_label(), anthracite.window_background());
    assert!(
        sec_contrast >= 4.5,
        "WCAG 2.1 AA Violation in Anthracite: secondary label contrast {sec_contrast:.2}:1 is below 4.5:1"
    );

    // Verify calm intermediate studio anthracite surface colors
    assert_eq!(
        anthracite.window_background(),
        Color32::from_rgb(50, 54, 62)
    );
    assert_eq!(
        anthracite.control_background(),
        Color32::from_rgb(60, 64, 73)
    );
    assert_eq!(anthracite.separator(), Color32::from_rgb(72, 77, 87));
}

#[test]
fn test_egui_visuals_synchronization_anthracite_mode() {
    let anthracite = MacTheme::new(MacAppearance::Anthracite);
    let ctx = Context::default();

    anthracite.apply_to_egui(&ctx);

    let visuals = ctx.style_of(ctx.theme()).visuals.clone();
    assert!(
        visuals.dark_mode,
        "egui Visuals was not set to dark mode for Anthracite appearance!"
    );
    assert_eq!(visuals.panel_fill, anthracite.window_background());
    assert_eq!(visuals.window_fill, anthracite.window_background());
}

#[test]
fn test_mac_background_style_relative_luminance_and_contrast_formula() {
    // Pure black has 0.0 relative luminance
    let black = Color32::from_rgb(0, 0, 0);
    assert!((crate_relative_luminance(black) - 0.0).abs() < 1e-5);

    // Pure white has 1.0 relative luminance
    let white = Color32::from_rgb(255, 255, 255);
    assert!((crate_relative_luminance(white) - 1.0).abs() < 1e-4);

    // Contrast between black and white is exactly 21.0:1
    let bw_contrast = crate_contrast_ratio(white, black);
    assert!((bw_contrast - 21.0).abs() < 0.1);

    // Linear channel conversions
    assert_eq!(srgb_channel_to_linear(0), 0.0);
    assert!((srgb_channel_to_linear(255) - 1.0).abs() < 1e-4);
}

#[test]
fn test_mac_background_style_threshold_wcag_boundary() {
    // Threshold L <= 0.179 => Dark, L > 0.179 => Light
    // Pure black (L = 0.0) -> Dark
    assert_eq!(
        MacBackgroundStyle::from_color(Color32::from_rgb(0, 0, 0)),
        MacBackgroundStyle::Dark
    );
    // Pure white (L = 1.0) -> Light
    assert_eq!(
        MacBackgroundStyle::from_color(Color32::from_rgb(255, 255, 255)),
        MacBackgroundStyle::Light
    );

    // Boundary check near L = 0.179
    // Mid gray around 118 sRGB gives luminance ~0.180 -> Light
    let mid_light = Color32::from_rgb(119, 119, 119);
    assert_eq!(
        MacBackgroundStyle::from_color(mid_light),
        MacBackgroundStyle::Light
    );

    // Mid gray around 117 sRGB gives luminance ~0.176 -> Dark
    let mid_dark = Color32::from_rgb(117, 117, 117);
    assert_eq!(
        MacBackgroundStyle::from_color(mid_dark),
        MacBackgroundStyle::Dark
    );
}

#[test]
fn test_mac_background_style_contrast_colors() {
    let theme = MacTheme::new(MacAppearance::Aqua);

    // 1. Purple accent / badge (#7B4382 = rgb(123, 67, 130))
    // Previously in gallery this had purple-on-purple 1:1 contrast defect!
    let purple = Color32::from_rgb(123, 67, 130);
    let purple_style = MacBackgroundStyle::from_color(purple);
    assert_eq!(purple_style, MacBackgroundStyle::Dark);

    let purple_fg = purple_style.primary_text_color(&theme);
    let purple_contrast = crate_contrast_ratio(purple_fg, purple);
    assert!(
        purple_contrast >= 4.5,
        "Purple background must guarantee >= 4.5:1 text contrast, got {purple_contrast:.2}:1"
    );

    // 2. Dark Blue (#004080)
    let dark_blue = Color32::from_rgb(0, 64, 128);
    let blue_style = MacBackgroundStyle::from_color(dark_blue);
    assert_eq!(blue_style, MacBackgroundStyle::Dark);
    let blue_contrast = crate_contrast_ratio(blue_style.primary_text_color(&theme), dark_blue);
    assert!(
        blue_contrast >= 4.5,
        "Dark blue background must guarantee >= 4.5:1 text contrast, got {blue_contrast:.2}:1"
    );

    // 3. Forest Green (#1E6B27)
    let forest_green = Color32::from_rgb(30, 107, 39);
    let green_style = MacBackgroundStyle::from_color(forest_green);
    assert_eq!(green_style, MacBackgroundStyle::Dark);
    let green_contrast = crate_contrast_ratio(green_style.primary_text_color(&theme), forest_green);
    assert!(
        green_contrast >= 4.5,
        "Green background must guarantee >= 4.5:1 text contrast, got {green_contrast:.2}:1"
    );

    // 4. Light surface (#F2F2F7)
    let light_surface = Color32::from_rgb(242, 242, 247);
    let light_style = MacBackgroundStyle::from_color(light_surface);
    assert_eq!(light_style, MacBackgroundStyle::Light);
    let light_contrast =
        crate_contrast_ratio(light_style.primary_text_color(&theme), light_surface);
    assert!(
        light_contrast >= 4.5,
        "Light surface must guarantee >= 4.5:1 text contrast, got {light_contrast:.2}:1"
    );
}

#[test]
fn test_mac_background_style_text_levels() {
    let theme = MacTheme::new(MacAppearance::Aqua);

    // Dark style text colors: 100% white primary, 85% white secondary, 70% white tertiary
    let dark_style = MacBackgroundStyle::Dark;
    assert_eq!(dark_style.primary_text_color(&theme), Color32::WHITE);
    assert_eq!(
        dark_style.secondary_text_color(&theme),
        Color32::from_white_alpha(217)
    );
    assert_eq!(
        dark_style.tertiary_text_color(&theme),
        Color32::from_white_alpha(179)
    );

    // Light style text colors delegate to Aqua semantic palette (ensuring dark text regardless of theme mode)
    let light_style = MacBackgroundStyle::Light;
    assert_eq!(
        light_style.primary_text_color(&theme),
        theme.tokens().colors().aqua.label.to_color32()
    );
    assert_eq!(
        light_style.secondary_text_color(&theme),
        theme.tokens().colors().aqua.secondary_label.to_color32()
    );
    assert_eq!(
        light_style.tertiary_text_color(&theme),
        theme.tokens().colors().aqua.placeholder_text.to_color32()
    );

    // Critical invariant: In DarkAqua mode, Light background style MUST still produce dark text!
    let dark_theme = MacTheme::new(MacAppearance::DarkAqua);
    let light_fg_in_dark_theme = light_style.primary_text_color(&dark_theme);
    assert_ne!(
        light_fg_in_dark_theme,
        dark_theme.label(),
        "Light background style in DarkAqua must NOT use white label text!"
    );
    let white_bg = Color32::from_rgb(255, 255, 255);
    let contrast_in_dark_mode = crate_contrast_ratio(light_fg_in_dark_theme, white_bg);
    assert!(
        contrast_in_dark_mode >= 4.5,
        "Light background style primary text in DarkAqua must meet >= 4.5:1 against white, got {contrast_in_dark_mode:.2}:1"
    );
}

#[test]
fn test_mac_card_headless_contrast_evaluation() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    // 1. Dark custom card passes MacBackgroundStyle::Dark to closure
    let dark_bg = Color32::from_rgb(30, 30, 35);
    let card_dark = MacCard::new().theme(&theme).fill(dark_bg);
    assert_eq!(card_dark.background_style(&theme), MacBackgroundStyle::Dark);

    let mut observed_style_dark = None;
    let mut out_dark = ctx.run_ui(egui::RawInput::default(), |ui| {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0));
        MacCard::new()
            .theme(&theme)
            .fill(dark_bg)
            .show(ui, rect, |_ui, style| {
                observed_style_dark = Some(style);
            });
    });
    out_dark.textures_delta.clear();
    assert_eq!(observed_style_dark, Some(MacBackgroundStyle::Dark));

    // 2. Light card passes MacBackgroundStyle::Light to closure
    let light_bg = Color32::from_rgb(250, 250, 250);
    let card_light = MacCard::new().theme(&theme).fill(light_bg);
    assert_eq!(
        card_light.background_style(&theme),
        MacBackgroundStyle::Light
    );

    let mut observed_style_light = None;
    let mut out_light = ctx.run_ui(egui::RawInput::default(), |ui| {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0));
        MacCard::new()
            .theme(&theme)
            .fill(light_bg)
            .show(ui, rect, |_ui, style| {
                observed_style_light = Some(style);
            });
    });
    out_light.textures_delta.clear();
    assert_eq!(observed_style_light, Some(MacBackgroundStyle::Light));
}

#[test]
fn test_v05_semantic_studio_colors_and_wcag_contrast() {
    let aqua = MacTheme::new(MacAppearance::Aqua);
    let dark_aqua = MacTheme::new(MacAppearance::DarkAqua);

    // System semantic colors must differ appropriately between Aqua and DarkAqua
    assert_eq!(aqua.system_yellow(), Color32::from_rgb(255, 204, 0));
    assert_eq!(dark_aqua.system_yellow(), Color32::from_rgb(255, 214, 10));
    assert_eq!(aqua.system_orange(), Color32::from_rgb(255, 149, 0));
    assert_eq!(dark_aqua.system_orange(), Color32::from_rgb(255, 159, 10));
    assert_eq!(aqua.system_red(), Color32::from_rgb(255, 59, 48));
    assert_eq!(dark_aqua.system_red(), Color32::from_rgb(255, 69, 58));

    // Alternating row backgrounds must maintain >= 4.5:1 label contrast
    for row_idx in [0usize, 1usize] {
        let aqua_row_bg = aqua.alternating_content_background(row_idx);
        let dark_row_bg = dark_aqua.alternating_content_background(row_idx);
        // Blend over control_background to verify effective cell contrast
        let blend = |fg: Color32, bg: Color32| -> Color32 {
            let [fr, fg_c, fb, fa] = fg.to_srgba_unmultiplied();
            let a = fa as f32 / 255.0;
            Color32::from_rgb(
                (fr as f32 * a + bg.r() as f32 * (1.0 - a)).round() as u8,
                (fg_c as f32 * a + bg.g() as f32 * (1.0 - a)).round() as u8,
                (fb as f32 * a + bg.b() as f32 * (1.0 - a)).round() as u8,
            )
        };
        let eff_aqua_bg = blend(aqua_row_bg, aqua.control_background());
        let eff_dark_bg = blend(dark_row_bg, dark_aqua.control_background());
        assert!(contrast_ratio(aqua.label(), eff_aqua_bg) >= 4.5);
        assert!(contrast_ratio(dark_aqua.label(), eff_dark_bg) >= 4.5);
    }
}

#[test]
fn test_wcag_contrast_on_mac_box_surfaces_all_appearances() {
    for appearance in [
        MacAppearance::Aqua,
        MacAppearance::Anthracite,
        MacAppearance::DarkAqua,
    ] {
        let theme = MacTheme::new(appearance);
        let box_bg = theme.box_background();
        let win_bg = theme.window_background();

        let label_on_box = contrast_ratio(theme.label(), box_bg);
        let sec_on_box = contrast_ratio(theme.secondary_label(), box_bg);
        let label_on_win = contrast_ratio(theme.label(), win_bg);
        let sec_on_win = contrast_ratio(theme.secondary_label(), win_bg);

        assert!(
            label_on_box >= 4.5,
            "WCAG 2.1 AA violation in {appearance:?}: label on box_background = {label_on_box:.2}:1 (< 4.5:1)"
        );
        assert!(
            sec_on_box >= 4.5,
            "WCAG 2.1 AA violation in {appearance:?}: secondary_label on box_background = {sec_on_box:.2}:1 (< 4.5:1)"
        );
        assert!(
            label_on_win >= 4.5,
            "WCAG 2.1 AA violation in {appearance:?}: label on window_background = {label_on_win:.2}:1 (< 4.5:1)"
        );
        assert!(
            sec_on_win >= 4.5,
            "WCAG 2.1 AA violation in {appearance:?}: secondary_label on window_background = {sec_on_win:.2}:1 (< 4.5:1)"
        );
    }
}
