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

    let run_dial = |ctx: &egui::Context,
                    pos: Pos2,
                    pressed: Option<bool>,
                    time: f64,
                    latched: &mut MacThumbIntentMask,
                    rotary: &mut f32,
                    layer: &mut MacThumbDialLayer| {
        let mut events = vec![Event::PointerMoved(pos)];
        if let Some(down) = pressed {
            events.push(Event::PointerButton { pos, button: PointerButton::Primary, pressed: down, modifiers: egui::Modifiers::NONE });
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
    assert_eq!(latched, MacThumbIntentMask::CONSTRAIN, "A normal 300ms click directly on North cushion must latch CONSTRAIN in 1 click");

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

#[test]
fn test_req_015_10_rotary_ring_only_emits_taptic_tick_on_15_deg_detent_crossing() {
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    setup_symbols_font(&mut fonts);
    ctx.set_fonts(fonts);

    let theme = MacTheme::new(MacAppearance::Aqua);
    let m = theme.thumb_dial();
    let r_rotary = (m.clover_outer_radius.points() + m.outer_radius.points()) * 0.5;
    let mut latched = MacThumbIntentMask::NONE;
    let mut rotary = 0.5f32;

    let run_at_angle = |ctx: &egui::Context,
                        center: Pos2,
                        deg: f32,
                        pressed: Option<bool>,
                        time: f64,
                        latched: &mut MacThumbIntentMask,
                        rotary: &mut f32| {
        let rad = deg.to_radians();
        let pos = center + Vec2::new(rad.cos() * r_rotary, rad.sin() * r_rotary);
        let mut events = vec![Event::PointerMoved(pos)];
        if let Some(down) = pressed {
            events.push(Event::PointerButton { pos, button: PointerButton::Primary, pressed: down, modifiers: egui::Modifiers::NONE });
        }
        let raw = RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
            time: Some(time),
            events,
            ..Default::default()
        };
        let mut resp_out = None;
        let mut full_out = ctx.run_ui(raw, |ui| {
            resp_out = Some(MacContextThumbDial::new("spec015_rotary_detent", latched, rotary).theme(&theme).show(ui));
        });
        full_out.textures_delta.clear();
        resp_out.expect("dial response")
    };

    let init_resp = run_at_angle(&ctx, Pos2::ZERO, 0.0, None, 0.0, &mut latched, &mut rotary);
    let center = init_resp.hub_center;

    // Press down on rotary ring at 1.0 deg
    let _ = run_at_angle(&ctx, center, 1.0, Some(true), 1.0, &mut latched, &mut rotary);

    // Move by tiny 1.0-degree increments from 2.0 to 10.0 deg (all within the first 0..15 deg detent bucket):
    // MUST NOT emit RotaryDetentTick on sub-detent micro-movements!
    for step in 2..=10 {
        let resp = run_at_angle(&ctx, center, step as f32, None, 1.0 + step as f64 * 0.016, &mut latched, &mut rotary);
        assert_eq!(resp.taptic_event, None, "Sub-detent movement at {} deg (within 0..15 deg bucket) must not spam RotaryDetentTick", step);
    }

    // Crossing 15.0 deg (at 17.0 deg) MUST emit exactly one RotaryDetentTick!
    let cross_resp = run_at_angle(&ctx, center, 17.0, None, 1.25, &mut latched, &mut rotary);
    assert_eq!(
        cross_resp.taptic_event,
        Some(egui_macview::MacTapticEvent::RotaryDetentTick),
        "Crossing the 15-degree detent boundary must emit RotaryDetentTick"
    );
}

#[test]
fn test_req_015_10_origin_anchor_detent_stepping_and_stationary_card_center() {
    use egui_macview::MacOriginAnchor;

    let mut accum_deg = 0.0f32;
    let mut anchor = MacOriginAnchor::Center;

    // 10 frames of 0.8 deg flywheel/rotary micro-delta (8.0 deg total < 15.0 deg detent):
    // Origin anchor MUST NOT advance!
    for _ in 0..10 {
        anchor = anchor.step_by_detent(&mut accum_deg, 0.8, 15.0);
        assert_eq!(anchor, MacOriginAnchor::Center, "Sub-detent rotation must not advance origin_anchor");
    }

    // Adding another 8.0 deg crosses the 15.0 deg threshold (16.0 deg total) -> advances by exactly 1 step!
    anchor = anchor.step_by_detent(&mut accum_deg, 8.0, 15.0);
    assert_eq!(anchor, MacOriginAnchor::MiddleRight, "Crossing 15.0 deg must advance origin_anchor by 1 step");
    assert!((accum_deg - 1.0).abs() < 1e-4, "Residual angle (1.0 deg) must be preserved in accumulator");

    // Verify state1_canvas.rs has zero card-center offset jump tied to self.origin_anchor
    let canvas_src = std::fs::read_to_string("examples/gallery/state1_canvas.rs").expect("read state1_canvas.rs");
    assert!(!canvas_src.contains("drag_off.x + ox"), "state1_canvas.rs must not add an (ox, oy) origin offset to card_center");
}

#[test]
fn test_req_015_11_tooltip_hold_window_uses_timed_repaint_not_zero_delay_spin() {
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    setup_symbols_font(&mut fonts);
    ctx.set_fonts(fonts);

    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut latched = MacThumbIntentMask::NONE;
    let mut rotary = 0.5f32;

    // Frame 0: layout dial and find North cushion center
    let mut north_pos = Pos2::ZERO;
    let mut out0 = ctx.run_ui(
        RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
            time: Some(1.0),
            ..Default::default()
        },
        |ui| {
            let r = MacContextThumbDial::new("spec015_battery_dial", &mut latched, &mut rotary).theme(&theme).show(ui);
            north_pos = r.sector_center(MacThumbSector::North);
        },
    );
    out0.textures_delta.clear();

    // Frame 1 (t = 10.0s): hover North cushion -> triggers tooltip
    let mut out1 = ctx.run_ui(
        RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
            time: Some(10.0),
            events: vec![Event::PointerMoved(north_pos)],
            ..Default::default()
        },
        |ui| {
            let _ = MacContextThumbDial::new("spec015_battery_dial", &mut latched, &mut rotary).theme(&theme).show(ui);
        },
    );
    out1.textures_delta.clear();

    // Frame 2 (t = 10.05s): egui settles pointer delta after PointerMoved
    let mut out1b = ctx.run_ui(
        RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
            time: Some(10.05),
            events: vec![],
            ..Default::default()
        },
        |ui| {
            let _ = MacContextThumbDial::new("spec015_battery_dial", &mut latched, &mut rotary).theme(&theme).show(ui);
        },
    );
    out1b.textures_delta.clear();

    // Frame 3 (t = 10.2s): pointer stationary inside North cushion during static 1.0s hold window
    let mut out = ctx.run_ui(
        RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0))),
            time: Some(10.2),
            events: vec![],
            ..Default::default()
        },
        |ui| {
            let _ = MacContextThumbDial::new("spec015_battery_dial", &mut latched, &mut rotary).theme(&theme).show(ui);
        },
    );
    out.textures_delta.clear();
    let repaint_delay = out.viewport_output.get(&egui::ViewportId::ROOT).map(|v| v.repaint_delay).unwrap_or(std::time::Duration::ZERO);
    assert!(
        repaint_delay >= std::time::Duration::from_millis(500),
        "During the static 1.0s tooltip hold window (at t=10.2s), dial must schedule a timed wake (>= 500ms), not an immediate 0ms repaint spin; got {:?}",
        repaint_delay
    );
}

#[test]
fn test_req_002_11_color_well_popover_renders_continuous_gpu_mesh_not_blocky_rects() {
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    setup_symbols_font(&mut fonts);
    ctx.set_fonts(fonts);

    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let mut color = egui::Color32::from_rgb(40, 120, 220);
    let well_pos = Pos2::new(20.0, 20.0);

    // Frame 0: render color well
    let mut out0 = ctx.run_ui(
        RawInput { screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))), ..Default::default() },
        |ui| {
            let _ = MacColorWell::new(&mut color).theme(&theme).show(ui);
        },
    );
    out0.textures_delta.clear();

    // Frame 1: click color well to open popover
    let mut out1 = ctx.run_ui(
        RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
            events: vec![
                Event::PointerMoved(well_pos),
                Event::PointerButton { pos: well_pos, button: PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE },
                Event::PointerButton { pos: well_pos, button: PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE },
            ],
            ..Default::default()
        },
        |ui| {
            let _ = MacColorWell::new(&mut color).theme(&theme).show(ui);
        },
    );
    out1.textures_delta.clear();

    // Frame 2: inspect shapes in open popover
    let mut out = ctx.run_ui(
        RawInput { screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))), ..Default::default() },
        |ui| {
            let _ = MacColorWell::new(&mut color).theme(&theme).show(ui);
        },
    );
    out.textures_delta.clear();

    fn count_shapes(shapes: &[egui::epaint::Shape], meshes: &mut usize, rects: &mut usize) {
        for s in shapes {
            match s {
                egui::epaint::Shape::Mesh(m) if m.vertices.len() >= 16 => *meshes += 1,
                egui::epaint::Shape::Rect(_) => *rects += 1,
                egui::epaint::Shape::Vec(v) => count_shapes(v, meshes, rects),
                _ => {}
            }
        }
    }
    let raw_shapes: Vec<egui::epaint::Shape> = out.shapes.into_iter().map(|c| c.shape).collect();
    let mut mesh_count = 0usize;
    let mut rect_count = 0usize;
    count_shapes(&raw_shapes, &mut mesh_count, &mut rect_count);

    assert!(
        mesh_count >= 2,
        "Open MacColorWell popover must render continuous GPU-interpolated Mesh gradients for SV square and Hue bar (found {} meshes)",
        mesh_count
    );
    assert!(rect_count < 80, "Open MacColorWell popover must not render a 256-rect blocky grid (found {} rects)", rect_count);
}

#[test]
fn test_req_013_03d_adaptive_semantic_reflow_groups_peer_buttons_horizontally_and_collapses_canvas_void() {
    use egui_macview::scene::{LayoutPolicy, XibXmlFrontend};

    let xml = std::fs::read_to_string("assets/nib_showcase/1_Vector_and_Transform_Inspector.xib")
        .expect("read 1_Vector_and_Transform_Inspector.xib");
    let scene = XibXmlFrontend::from_xml_str(&xml).expect("compile 1_Vector_and_Transform_Inspector.xib");

    let solved = scene.solve_layout(Vec2::new(720.0, 620.0), LayoutPolicy::AdaptiveSemanticReflow, u64::MAX, "");

    let find_rect = |title: &str| -> egui::Rect {
        let node =
            scene.nodes().iter().find(|n| n.title == title || n.ib_object_id == title).unwrap_or_else(|| panic!("missing node {}", title));
        *solved.get(&node.id).unwrap_or_else(|| panic!("missing solved rect for {}", title))
    };

    let r_pulse = find_rect("Trigger CASpring Pulse");
    let r_reset = find_rect("Reset Identity");
    let r_undo = find_rect("Undo");
    let r_redo = find_rect("Redo");

    // In AdaptiveSemanticReflow, same-row buttons must stay on the same horizontal row, not a diagonal staircase!
    assert!(
        (r_pulse.min.y - r_reset.min.y).abs() <= 4.0
            && (r_reset.min.y - r_undo.min.y).abs() <= 4.0
            && (r_undo.min.y - r_redo.min.y).abs() <= 4.0,
        "AdaptiveSemanticReflow must place same-row peer buttons on a shared horizontal row, got Ys: pulse={}, reset={}, undo={}, redo={}",
        r_pulse.min.y,
        r_reset.min.y,
        r_undo.min.y,
        r_redo.min.y
    );

    // Furthermore, the large 310pt CustomCanvasSlot void must collapse so controls start near the top (< 180pt)
    assert!(
        r_pulse.min.y < 180.0,
        "AdaptiveSemanticReflow must collapse large CustomCanvasSlot voids so controls do not start 310pt down (got r_pulse.min.y = {})",
        r_pulse.min.y
    );
}

#[test]
fn test_req_015_12_zero_hardcoded_stream_titles_help_tables_or_os_profile_tables() {
    let video_src = std::fs::read_to_string("examples/gallery/video_scene.rs").expect("read video_scene.rs");
    for forbidden in
        ["Big Buck Bunny", "Sintel", "Tears of Steel", "For Bigger Blazes", "Brandenburg", "Clair de Lune", "Color32::from_rgb("]
    {
        assert!(
            !video_src.contains(forbidden),
            "video_scene.rs must be 100% data-driven from state.audio_track and MacTheme without hardcoded `{forbidden}`"
        );
    }

    let builders_src = std::fs::read_to_string("examples/gallery/nib_showcase_builders.rs").expect("read nib_showcase_builders.rs");
    for forbidden in ["\"showVectorHelp:\"", "\"showTypographyHelp:\"", "\"s6-en-txt-reg\"", "\"s6-en-pop-preset\""] {
        assert!(
            !builders_src.contains(forbidden),
            "nib_showcase_builders.rs must not contain hardcoded selector/id tables (`{forbidden}`); use .xib tooltips and bindings"
        );
    }

    let sync_src = std::fs::read_to_string("examples/gallery/nib_store_sync.rs").expect("read nib_store_sync.rs");
    for forbidden in ["Sonoma 14.8", "Sequoia 15.0", "Big Sur 11.0", "Color32::from_rgb("] {
        assert!(
            !sync_src.contains(forbidden),
            "nib_store_sync.rs must read PopUpButton options from .xib store and colors from MacTheme without hardcoded `{forbidden}`"
        );
    }
}
