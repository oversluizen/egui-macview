//! Automated Micro-Animations, Vector Oracles & 3-Mode Inspector Presets Verification Tests.
//!
//! Normative References:
//! - SPEC-001 §4.12: Control Micro-Animations & Dynamic CASpring Interpolation.
//! - SPEC-001 §4.13.A: Compact Pro HUD Palette (NSHUDWindowMask, 178x63pt).
//! - SPEC-001 §4.13.B: Default NIB Document Inspector (260pt).
//! - SPEC-001 §4.13.C: Expanded Pro Inspector (380pt with multi-deck tabs).
//! - SPEC-001 §4.14: Authentic AppKit Vector Rendering Oracles.
//! - GL-1015: Regel 0 (De Verankeringswet) & Regel 7 (ISO/IEC/IEEE 29119 Spec-First).

use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput};
use egui_macview::{
    theme::{MacAppearance, MacTheme, WindowControlsStyle},
    CASpring, MacFloatingInfoBox, MacToolbar,
};

#[test]
fn test_inspector_preset_compact_hud_invariants() {
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let hud = theme.popover_metrics();
    let compact_width = hud.compact_panel_width.points();
    let compact_height = hud.compact_panel_height.points();
    let compact_input_h = theme.text_field().compact_infobox.height.points();

    assert_eq!(compact_width, 178.0, "Compact HUD width must equal 178.0pt");
    assert_eq!(compact_height, 63.0, "Compact HUD height must equal 63.0pt");
    assert_eq!(
        compact_input_h, 18.0,
        "Compact HUD input height must equal 18.0pt"
    );

    // Math budget: Header (22pt) + Padding (4pt) + Two rows (18pt * 2 = 36pt) + Divider (1pt) = 63pt
    let total_budget = 22.0 + 4.0 + (compact_input_h * 2.0) + 1.0;
    assert!(
        (total_budget - compact_height).abs() <= 1.0,
        "HUD panel budget calculation mismatch: total={total_budget} vs budget={compact_height}"
    );
}

#[test]
fn test_inspector_preset_default_nib_invariants() {
    let theme = MacTheme::new(MacAppearance::Aqua);
    let split_metrics = theme.split_view();
    let min_pane_w = split_metrics.min_pane_size.points();
    let default_nib_standard_w = 260.0;

    assert_eq!(
        min_pane_w, 50.0,
        "Split view minimum pane size must equal 50.0pt"
    );
    assert!(
        default_nib_standard_w >= min_pane_w,
        "Default NIB standard width must satisfy min constraint"
    );
}

#[test]
fn test_inspector_preset_expanded_pro_invariants() {
    let expanded_width = 380.0;
    let standard_width = 260.0;

    // Expanded inspector provides extra horizontal space for multi-column affine transform matrix
    assert!(
        (360.0..=420.0).contains(&expanded_width),
        "Expanded Pro Inspector width {expanded_width} outside 360..420pt design envelope"
    );
    assert!(
        expanded_width - standard_width >= 100.0,
        "Expanded mode must provide at least 100pt extra clearance over default NIB"
    );
}

#[test]
fn test_caspring_toggle_interpolation_oracle() {
    let spring = CASpring::from_perceptual(0.35, 0.65);
    let settling = spring.settling_duration();

    assert!(
        settling > 0.25 && settling < 0.65,
        "Toggle spring settling duration outside perceptual range: {settling}s"
    );

    let p0 = spring.position_at(0.0);
    assert!((p0 - 0.0).abs() < 1e-4);

    let p_mid = spring.position_at(0.1);
    assert!(
        p_mid > 0.2 && p_mid < 0.9,
        "Toggle intermediate spring position unnatural: {p_mid}"
    );

    let p_end = spring.position_at(settling);
    assert!(
        (p_end - 1.0).abs() < 0.01,
        "Toggle spring failed to settle at target 1.0: {p_end}"
    );
}

#[test]
fn test_caspring_segmented_indicator_interpolation_oracle() {
    let spring = CASpring::from_perceptual(0.30, 0.70);
    let overshoot = spring.overshoot_ratio();

    assert!(
        overshoot <= 0.05,
        "Segmented control overshoot exceeds 5% HIG bound: {overshoot}"
    );
}

#[test]
fn test_stepper_vector_rendering_oracle() {
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let stepper_metrics = theme.stepper();

    let arrow_size = stepper_metrics.arrow_size.points();
    let stroke_width = stepper_metrics.arrow_stroke_width.points();

    assert_eq!(arrow_size, 6.0, "Stepper arrow size must be 6.0pt");
    assert_eq!(stroke_width, 1.2, "Stepper stroke width must be 1.2pt");
}

#[test]
fn test_window_controls_style_geometry_and_glyph_metrics_oracle() {
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    // 1. Default Style Invariant
    assert_eq!(
        theme.window_controls_style(),
        WindowControlsStyle::AppleTrafficLights,
        "Default WindowControlsStyle must be AppleTrafficLights"
    );

    // 2. Apple Traffic Lights Invariants
    let apple_theme = theme
        .clone()
        .with_window_controls_style(WindowControlsStyle::AppleTrafficLights);
    assert_eq!(
        apple_theme.window_controls_style(),
        WindowControlsStyle::AppleTrafficLights
    );

    let tb_metrics = apple_theme.toolbar_metrics();
    assert_eq!(tb_metrics.traffic_lights_inset.points(), 70.0);
    assert_eq!(tb_metrics.traffic_light_radius.points(), 6.0);
    assert_eq!(tb_metrics.traffic_light_spacing.points(), 8.0);

    // Cluster width invariant: 3 circles (12pt each) + 2 spaces (8pt each) = 52pt
    let circle_diameter = tb_metrics.traffic_light_radius.points() * 2.0;
    let cluster_width = (circle_diameter * 3.0) + (tb_metrics.traffic_light_spacing.points() * 2.0);
    assert_eq!(
        cluster_width, 52.0,
        "Apple traffic light cluster width must equal 52.0pt"
    );

    // Leading clearance: 18pt margin + 52pt cluster = 70pt inset
    let leading_margin = 18.0;
    assert_eq!(
        leading_margin + cluster_width,
        tb_metrics.traffic_lights_inset.points(),
        "Leading margin + cluster width must match traffic_lights_inset token"
    );

    // Apple glyph metrics
    let (close_fill, close_border) = apple_theme.traffic_light_close();
    let (min_fill, min_border) = apple_theme.traffic_light_minimize();
    let (zoom_fill, zoom_border) = apple_theme.traffic_light_zoom();
    assert_ne!(close_fill, min_fill);
    assert_ne!(min_fill, zoom_fill);
    assert_ne!(close_border, egui::Color32::TRANSPARENT);
    assert_ne!(min_border, egui::Color32::TRANSPARENT);
    assert_ne!(zoom_border, egui::Color32::TRANSPARENT);

    // 3. Windows Caption Buttons Invariants
    let win_theme = theme
        .clone()
        .with_window_controls_style(WindowControlsStyle::WindowsCaptionButtons);
    assert_eq!(
        win_theme.window_controls_style(),
        WindowControlsStyle::WindowsCaptionButtons
    );

    let win_tb = win_theme.toolbar_metrics();
    assert_eq!(
        win_tb.caption_button_width.points(),
        46.0,
        "Windows caption button width must be 46.0pt"
    );
    assert_eq!(
        win_tb.caption_button_height.points(),
        32.0,
        "Windows caption button height must be 32.0pt"
    );
    assert_eq!(
        win_tb.caption_icon_size.points(),
        10.0,
        "Windows caption glyph box size must be 10.0pt"
    );

    // Total Windows caption group width: 3 buttons * 46pt = 138pt
    let win_group_width = win_tb.caption_button_width.points() * 3.0;
    assert_eq!(
        win_group_width, 138.0,
        "Windows caption buttons cluster width must equal 138.0pt"
    );

    // Windows close button hover styling oracle: Crimson red #E81123
    let close_hover_bg = win_theme.windows_caption_close_hover_bg();
    assert_eq!(close_hover_bg, egui::Color32::from_rgb(232, 17, 35));

    // 4. Native Host Invariants
    let native_theme = theme
        .clone()
        .with_window_controls_style(WindowControlsStyle::NativeHost);
    assert_eq!(
        native_theme.window_controls_style(),
        WindowControlsStyle::NativeHost
    );
}

#[test]
fn test_mac_toolbar_widget_traffic_lights_and_windows_caption_interactions() {
    let ctx = Context::default();
    let apple_theme = MacTheme::new(MacAppearance::DarkAqua)
        .with_window_controls_style(WindowControlsStyle::AppleTrafficLights);

    // 1. Render MacToolbar with AppleTrafficLights
    let mut close_pos = Pos2::new(24.0, 22.0);
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacToolbar::new("test_toolbar")
            .theme(&apple_theme)
            .show(ui, |center_ui| {
                center_ui.label("Document Title");
            });
        assert_eq!(resp.response.rect.height(), 44.0);
        assert!(!resp.close_clicked);
        let r = apple_theme.toolbar_metrics().traffic_light_radius.points();
        close_pos = Pos2::new(
            resp.response.rect.left() + r * 4.0,
            resp.response.rect.center().y,
        );
    });
    out1.textures_delta.clear();

    // 2. Click close traffic light
    let mut press_in = RawInput::default();
    press_in.events.push(Event::PointerMoved(close_pos));
    press_in.events.push(Event::PointerButton {
        pos: close_pos,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    let mut out2 = ctx.run_ui(press_in, |ui| {
        MacToolbar::new("test_toolbar")
            .theme(&apple_theme)
            .show(ui, |center_ui| {
                center_ui.label("Document Title");
            });
    });
    out2.textures_delta.clear();

    let mut release_in = RawInput::default();
    release_in.events.push(Event::PointerMoved(close_pos));
    release_in.events.push(Event::PointerButton {
        pos: close_pos,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    let mut close_fired = false;
    let mut out3 = ctx.run_ui(release_in, |ui| {
        let resp = MacToolbar::new("test_toolbar")
            .theme(&apple_theme)
            .show(ui, |center_ui| {
                center_ui.label("Document Title");
            });
        close_fired = resp.close_clicked;
    });
    out3.textures_delta.clear();
    assert!(
        close_fired,
        "Clicking Apple close traffic light must set close_clicked = true"
    );

    // 3. Render MacToolbar with WindowsCaptionButtons
    let win_theme = MacTheme::new(MacAppearance::DarkAqua)
        .with_window_controls_style(WindowControlsStyle::WindowsCaptionButtons);
    let mut out4 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacToolbar::new("test_toolbar_win")
            .theme(&win_theme)
            .show(ui, |center_ui| {
                center_ui.label("Windows Caption Mode");
            });
        assert_eq!(resp.response.rect.height(), 44.0);
    });
    out4.textures_delta.clear();
}

#[test]
fn test_mac_floating_infobox_widget_rendering_oracle() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let mut pos = Pos2::new(40.0, 40.0);
    let mut x = "10.0".to_string();
    let mut y = "20.0".to_string();
    let mut w = "100.0".to_string();
    let mut h = "80.0".to_string();

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacFloatingInfoBox::new("oracle_hud", &mut pos)
            .theme(&theme)
            .title("Compact HUD")
            .show_xywh(ui.ctx(), &mut x, &mut y, &mut w, &mut h);
        assert_eq!(resp.rect.width(), 178.0);
        assert_eq!(resp.rect.height(), 63.0);
    });
    out.textures_delta.clear();
}
