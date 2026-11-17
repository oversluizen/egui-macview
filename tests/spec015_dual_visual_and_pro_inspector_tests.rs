//! Normative verification suite for `SPEC-015`:
//! - External Host Font Injection API (`setup_custom_macview_fonts`)
//! - Pro-Inspector Micro-Styling (`+` green hover, `-` red hover, `focused_row_fill`)
//! - `MacContextThumbDial` transient HUD tooltip (`1.0s` hold + `0.35s` fade-out)
//! - True Monospace (`Hack`) & `Ubuntu-Light` font preservation (`setup_symbols_font`)
//! - `MacColorWell` achromatic saturation unlock

use egui::epaint::Hsva;
use egui::{Event, PointerButton, Pos2, RawInput, Vec2};
use egui_macview::{
    setup_custom_macview_fonts, setup_symbols_font, InlineActionKind, MacAppearance, MacColorWell, MacContextThumbDial,
    MacContextThumbDialState, MacInlineActionButton, MacTheme, MacThumbContextPreset, MacThumbDialLayer, MacThumbIntentMask,
    MacThumbSector, INTER_REGULAR_FONT_BYTES, INTER_REGULAR_FONT_NAME, INTER_SEMIBOLD_FONT_BYTES, INTER_SEMIBOLD_FONT_NAME, VERSION,
};

#[test]
fn test_req_015_09_crate_version_v081() {
    assert_eq!(VERSION, "0.8.1", "Crate version must be 0.8.1 for Milestone 0.8.1 release");
}

#[test]
fn test_req_015_03_all_presets_have_six_distinct_sector_icons_and_labels() {
    for preset in MacThumbContextPreset::ALL {
        let set = preset.sectors();
        for i in 0..set.items.len() {
            for j in (i + 1)..set.items.len() {
                assert_ne!(
                    set.items[i].icon, set.items[j].icon,
                    "Preset {:?} must have 6 distinct icons, but sector {} and {} share {:?}",
                    preset, i, j, set.items[i].icon
                );
                assert_ne!(
                    set.items[i].label, set.items[j].label,
                    "Preset {:?} must have 6 distinct labels, but sector {} and {} share {:?}",
                    preset, i, j, set.items[i].label
                );
            }
        }
    }
}

#[test]
fn test_req_015_03_spatial_intent_normal_300ms_click_latches_and_single_hub_click_toggles_joystick() {
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    setup_symbols_font(&mut fonts);
    ctx.set_fonts(fonts);

    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut latched = MacThumbIntentMask::NONE;
    let mut rotary = 0.5f32;
    let mut layer = MacThumbDialLayer::CompassModifiers;

    let run_dial = |ctx: &egui::Context, pos: Pos2, pressed: Option<bool>, time: f64,
                    latched: &mut MacThumbIntentMask, rotary: &mut f32, layer: &mut MacThumbDialLayer| {
        let mut events = vec![Event::PointerMoved(pos)];
        if let Some(down) = pressed {
            events.push(Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: down,
                modifiers: egui::Modifiers::NONE,
            });
        }
        let raw = RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
            time: Some(time),
            events,
            ..Default::default()
        };
        let mut resp_out = None;
        let mut full_out = ctx.run_ui(raw, |ui| {
            resp_out = Some(
                MacContextThumbDial::new("spec015_spatial_dial", latched, rotary)
                    .layer(layer)
                    .preset(MacThumbContextPreset::Primitives)
                    .theme(&theme)
                    .show(ui),
            );
        });
        full_out.textures_delta.clear();
        resp_out.expect("dial response")
    };

    let init_resp = run_dial(&ctx, Pos2::ZERO, None, 0.0, &mut latched, &mut rotary, &mut layer);
    let north_pos = init_resp.sector_center(MacThumbSector::North);
    let hub_pos = init_resp.hub_center;

    // 1. A normal human click lasting 300 ms (t=1.00 down, t=1.30 up) directly on North MUST latch North in 1 click!
    run_dial(&ctx, north_pos, Some(true), 1.00, &mut latched, &mut rotary, &mut layer);
    run_dial(&ctx, north_pos, Some(false), 1.30, &mut latched, &mut rotary, &mut layer);
    assert_eq!(
        latched,
        MacThumbIntentMask::CONSTRAIN,
        "A normal 300ms click directly on North cushion must latch CONSTRAIN in 1 click"
    );

    // 2. While North is latched, a SINGLE click on the center hub (t=2.00 down, t=2.25 up) MUST immediately switch to FreeJoystickBowl!
    run_dial(&ctx, hub_pos, Some(true), 2.00, &mut latched, &mut rotary, &mut layer);
    run_dial(&ctx, hub_pos, Some(false), 2.25, &mut latched, &mut rotary, &mut layer);
    assert_eq!(
        layer,
        MacThumbDialLayer::FreeJoystickBowl,
        "A single click on the center hub must immediately toggle to FreeJoystickBowl without requiring a double-click"
    );
}

#[test]
fn test_req_015_01_external_host_font_injection_api() {
    let mut fonts = egui::FontDefinitions::default();
    setup_custom_macview_fonts(&mut fonts, INTER_REGULAR_FONT_BYTES.to_vec(), Some(INTER_SEMIBOLD_FONT_BYTES.to_vec()));
    assert!(fonts.font_data.contains_key(INTER_REGULAR_FONT_NAME));
    assert!(fonts.font_data.contains_key(INTER_SEMIBOLD_FONT_NAME));
    assert!(fonts.font_data.contains_key("cupertino_icons"));
}

#[test]
fn test_req_015_02_pro_inspector_semantic_hover_and_focused_row_fill() {
    for appearance in [MacAppearance::Aqua, MacAppearance::DarkAqua] {
        let theme = MacTheme::new(appearance);

        let (add_fg, add_bg) = MacInlineActionButton::hover_colors(&theme, InlineActionKind::Add);
        assert!(
            add_fg.g() > add_fg.r() && add_fg.g() > add_fg.b(),
            "Add (+) hover icon color must be green-dominant in {:?}: {:?}",
            appearance,
            add_fg
        );
        assert!(add_bg.a() > 0, "Add (+) hover pill background must be visible");

        let (rem_fg, rem_bg) = MacInlineActionButton::hover_colors(&theme, InlineActionKind::Remove);
        assert!(
            rem_fg.r() > rem_fg.g() && rem_fg.r() > rem_fg.b(),
            "Remove (-) hover icon color must be red-dominant in {:?}: {:?}",
            appearance,
            rem_fg
        );
        assert!(rem_bg.a() > 0, "Remove (-) hover pill background must be visible");

        let other_hover = theme.inspector_hover_fill();
        assert!(other_hover.a() > 0, "Borderless inspector rollover fill must be visible");

        let row_fill = theme.focused_row_fill();
        assert!(row_fill.a() > 0, "Focused inspector row highlight must be visible");
    }
}

#[test]
fn test_req_015_03_context_thumb_dial_transient_hud_tooltip_fade_out() {
    let mut state = MacContextThumbDialState::default();
    state.trigger_tooltip("1:1 Aspect", 10.0);

    // Full opacity during the 1.0s hold window
    assert!((state.tooltip_opacity_at(10.0) - 1.0).abs() < 1e-4);
    assert!((state.tooltip_opacity_at(10.8) - 1.0).abs() < 1e-4);
    assert_eq!(state.active_tooltip_text(10.8), Some("1:1 Aspect"));

    // Mid-fade during the 0.35s decay window (11.0s .. 11.35s)
    let mid_alpha = state.tooltip_opacity_at(11.175);
    assert!(mid_alpha > 0.2 && mid_alpha < 0.8, "Expected mid-fade opacity around 0.5, got {}", mid_alpha);

    // Fully faded out after 1.35s
    assert_eq!(state.tooltip_opacity_at(11.40), 0.0);
    assert_eq!(state.active_tooltip_text(11.40), None);
}

#[test]
fn test_req_015_04_monospace_font_is_not_overridden_by_proportional_inter() {
    let mut fonts = egui::FontDefinitions::default();
    setup_symbols_font(&mut fonts);

    let mono_list = fonts.families.get(&egui::FontFamily::Monospace).expect("Monospace family must exist");
    assert!(!mono_list.is_empty(), "Monospace font family list must not be empty");
    assert_ne!(mono_list[0], INTER_REGULAR_FONT_NAME, "Primary Monospace font must not be proportional Inter-Regular");
    let ubuntu_list = fonts.families.get(&egui::FontFamily::Name("Ubuntu-Light".into())).expect("Ubuntu-Light family must exist");
    assert_eq!(ubuntu_list[0], "Ubuntu-Light", "Primary Ubuntu-Light font must be Ubuntu-Light");
}

#[test]
fn test_req_015_05_color_well_unlocks_achromatic_grey_and_black_hsva() {
    // Neutral light grey (#ECECEC -> S = 0.0, V = 0.925)
    let grey = Hsva::new(0.60, 0.0, 0.925, 1.0);
    let unlocked_grey = MacColorWell::unlock_achromatic_hsva(grey);
    assert!(
        unlocked_grey.s >= 0.75,
        "Achromatic grey must unlock saturation so Hue changes produce visible color, got s={}",
        unlocked_grey.s
    );

    // Near-black (#141416 -> S ~ 0.08, V ~ 0.086)
    let near_black = Hsva::new(0.66, 0.0, 0.08, 1.0);
    let unlocked_black = MacColorWell::unlock_achromatic_hsva(near_black);
    assert!(
        unlocked_black.s >= 0.75 && unlocked_black.v >= 0.75,
        "Achromatic near-black must unlock both saturation and value when picking a hue, got {:?}",
        unlocked_black
    );
}
