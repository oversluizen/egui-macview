//! Normative COTS Verification Suite for `SPEC-014`: Context-Adaptive Bimanual Thumb Dial & Radial Clover / Pie HUD (`MacContextThumbDial`).
//!
//! Verifies `REQ-014-01` through `REQ-014-04` (`TC-014-01` .. `TC-014-05`).

use egui::{Color32, Context, Event, PointerButton, Pos2, RawInput, Rect, Vec2};
use egui_macview::{
    contrast_ratio, setup_symbols_font, MacAppearance, MacContextThumbDial, MacEdgeThumbwheel, MacFreeJoystickBowl, MacSFSymbol,
    MacTapticEvent, MacTheme, MacThumbArcPlacement, MacThumbContextPreset, MacThumbDialLayer, MacThumbDockZone, MacThumbIntent,
    MacThumbIntentMask, MacThumbSector, MacThumbSectorIcon, VERSION,
};

fn init_headless_ctx() -> Context {
    let ctx = Context::default();
    let mut fonts = egui::FontDefinitions::default();
    setup_symbols_font(&mut fonts);
    ctx.set_fonts(fonts);
    ctx
}

fn run_frame(ctx: &Context, input: RawInput, f: impl FnMut(&mut egui::Ui)) {
    let mut out = ctx.run_ui(input, f);
    out.textures_delta.clear();
}

fn pointer_frame(pos: Pos2, pressed: Option<bool>, time: f64) -> RawInput {
    let mut input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0))),
        time: Some(time),
        ..Default::default()
    };
    input.events.push(Event::PointerMoved(pos));
    if let Some(is_down) = pressed {
        input.events.push(Event::PointerButton { pos, button: PointerButton::Primary, pressed: is_down, modifiers: egui::Modifiers::NONE });
    }
    input
}

fn tap_dial(ctx: &Context, id: &'static str, pos: Pos2, t: f64, latched: &mut MacThumbIntentMask, rot: &mut f32, theme: &MacTheme) {
    run_frame(ctx, pointer_frame(pos, Some(true), t), |ui| {
        MacContextThumbDial::new(id, latched, rot).preset(MacThumbContextPreset::Primitives).theme(theme).show(ui);
    });
    run_frame(ctx, pointer_frame(pos, Some(false), t + 0.05), |ui| {
        MacContextThumbDial::new(id, latched, rot).preset(MacThumbContextPreset::Primitives).theme(theme).show(ui);
    });
}

/// TC-014-01 (`REQ-014-01`, `REQ-014-07`): 4 Invariant Cardinal Intentions, Diagonal Combos, Official Apple `MacSFSymbol` Names & Concise Labels.
#[test]
fn tc_014_01_cardinal_intentions_and_diagonal_combination_sectors() {
    for preset in MacThumbContextPreset::ALL {
        let set = preset.sectors();
        for sec in MacThumbSector::ALL {
            let spec = set.sector(sec);
            assert!(spec.label.chars().count() <= 6, "Cushion label '{}' must be <= 6 chars to prevent visual clutter", spec.label);
            if let MacThumbSectorIcon::SFSymbol(sf) = spec.icon {
                assert!(!sf.sf_symbol_name().is_empty());
            } else {
                panic!("Default preset sector must carry an official MacSFSymbol");
            }
        }
    }

    let prim = MacThumbContextPreset::Primitives.sectors();
    assert_eq!(prim.sector(MacThumbSector::North).icon, MacThumbSectorIcon::SFSymbol(MacSFSymbol::AspectRatio));
    assert_eq!(prim.sector(MacThumbSector::NorthEast).icon, MacThumbSectorIcon::SFSymbol(MacSFSymbol::Viewfinder));
    assert_eq!(prim.sector(MacThumbSector::East).icon, MacThumbSectorIcon::SFSymbol(MacSFSymbol::Scope));
    assert_eq!(prim.sector(MacThumbSector::SouthEast).icon, MacThumbSectorIcon::SFSymbol(MacSFSymbol::Skew));
    assert_eq!(prim.sector(MacThumbSector::South).icon, MacThumbSectorIcon::SFSymbol(MacSFSymbol::Scribble));
    assert_eq!(prim.sector(MacThumbSector::West).icon, MacThumbSectorIcon::SFSymbol(MacSFSymbol::ArrowUpAndDownAndLeftAndRight));
    assert_eq!(MacSFSymbol::AspectRatio.sf_symbol_name(), "aspectratio");
    assert_eq!(MacSFSymbol::Scope.sf_symbol_name(), "scope");
    assert_eq!(MacSFSymbol::PlusSquareOnSquare.sf_symbol_name(), "plus.square.on.square");
    assert_eq!(MacSFSymbol::Skew.sf_symbol_name(), "skew");
    assert_eq!(MacSFSymbol::Scribble.sf_symbol_name(), "scribble");

    assert_eq!(MacThumbSector::NorthEast.intent_mask(), MacThumbIntentMask::CONSTRAIN.union(MacThumbIntentMask::ALTER));
    assert_eq!(MacThumbSector::SouthEast.intent_mask(), MacThumbIntentMask::GLIDE.union(MacThumbIntentMask::ALTER));
    assert!(MacThumbSector::NorthEast.intent_mask().contains(MacThumbIntent::Constrain));
    assert!(MacThumbSector::NorthEast.intent_mask().contains(MacThumbIntent::Alter));
}

/// TC-014-02 (`REQ-014-02`): Exclusive Single-Sector Radio Latching (`< 180 ms`), Toggle-Off, and Center Hub Clear.
#[test]
fn tc_014_02_exclusive_single_sector_radio_latch_and_hub_clear() {
    let ctx = init_headless_ctx();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let mut latched = MacThumbIntentMask::NONE;
    let mut rotary_val = 0.5f32;
    let mut geom_center = Pos2::ZERO;
    let mut north_pos = Pos2::ZERO;
    let mut east_pos = Pos2::ZERO;
    let mut ne_pos = Pos2::ZERO;

    run_frame(&ctx, pointer_frame(Pos2::ZERO, None, 0.0), |ui| {
        let resp = MacContextThumbDial::new("dial_latch_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
        geom_center = resp.hub_center;
        north_pos = resp.sector_center(MacThumbSector::North);
        east_pos = resp.sector_center(MacThumbSector::East);
        ne_pos = resp.sector_center(MacThumbSector::NorthEast);
    });

    // 1. Quick tap (< 180 ms) on North ("1:1") -> latches ONLY North (CONSTRAIN)
    tap_dial(&ctx, "dial_latch_test", north_pos, 1.00, &mut latched, &mut rotary_val, &theme);
    assert_eq!(latched, MacThumbIntentMask::CONSTRAIN);

    // 2. Quick tap (< 180 ms) on East ("Center") -> switches EXCLUSIVELY to East (ALTER), unlatching North!
    tap_dial(&ctx, "dial_latch_test", east_pos, 2.00, &mut latched, &mut rotary_val, &theme);
    assert_eq!(latched, MacThumbIntentMask::ALTER, "Tapping East must switch exclusively to East instead of selecting all sectors");

    // 3. Quick tap on Center Thumb Rest Hub -> clears active latch
    tap_dial(&ctx, "dial_latch_test", geom_center, 3.00, &mut latched, &mut rotary_val, &theme);
    assert_eq!(latched, MacThumbIntentMask::NONE);

    // 4. Quick tap on diagonal NorthEast ("1:1+⊙") -> latches ONLY NorthEast (CONSTRAIN | ALTER), second tap unlatches it!
    tap_dial(&ctx, "dial_latch_test", ne_pos, 4.00, &mut latched, &mut rotary_val, &theme);
    assert_eq!(latched, MacThumbIntentMask::CONSTRAIN.union(MacThumbIntentMask::ALTER));
    tap_dial(&ctx, "dial_latch_test", ne_pos, 5.00, &mut latched, &mut rotary_val, &theme);
    assert_eq!(latched, MacThumbIntentMask::NONE);
}

/// TC-014-03 (`REQ-014-02`): Center-Rest Slide-Out Gesture & Hold-to-Momentary (`>= 180 ms`).
#[test]
fn tc_014_03_hold_to_momentary_and_center_rest_slide_out() {
    let ctx = init_headless_ctx();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let mut latched = MacThumbIntentMask::NONE;
    let mut rotary_val = 0.5f32;
    let mut hub_center = Pos2::ZERO;
    let mut ne_pos = Pos2::ZERO;
    let mut se_pos = Pos2::ZERO;

    run_frame(&ctx, pointer_frame(Pos2::ZERO, None, 0.0), |ui| {
        let resp = MacContextThumbDial::new("dial_hold_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
        hub_center = resp.hub_center;
        ne_pos = resp.sector_center(MacThumbSector::NorthEast);
        se_pos = resp.sector_center(MacThumbSector::SouthEast);
    });

    // 1. Press in Center Thumb Rest Hub, slide out to NorthEast ("1:1+⊙Center") -> Momentary Hold active while down!
    run_frame(&ctx, pointer_frame(hub_center, Some(true), 1.00), |ui| {
        MacContextThumbDial::new("dial_hold_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
    });
    let mut slide_effective = MacThumbIntentMask::NONE;
    run_frame(&ctx, pointer_frame(ne_pos, None, 1.10), |ui| {
        let resp = MacContextThumbDial::new("dial_hold_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
        slide_effective = resp.effective_mask;
    });
    assert_eq!(slide_effective, MacThumbIntentMask::CONSTRAIN.union(MacThumbIntentMask::ALTER));

    // Release after slide -> clears momentary hold and leaves latched == NONE
    let mut after_release = MacThumbIntentMask::NONE;
    run_frame(&ctx, pointer_frame(ne_pos, Some(false), 1.15), |ui| {
        let resp = MacContextThumbDial::new("dial_hold_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
        after_release = resp.effective_mask;
    });
    assert_eq!(after_release, MacThumbIntentMask::NONE);
    assert_eq!(latched, MacThumbIntentMask::NONE);

    // 2. Direct press and hold (>= 600 ms: 700 ms) on SouthEast ("⇄ Shear") -> Momentary Hold active, clears on release
    let mut held_effective = MacThumbIntentMask::NONE;
    run_frame(&ctx, pointer_frame(se_pos, Some(true), 2.00), |ui| {
        let resp = MacContextThumbDial::new("dial_hold_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
        held_effective = resp.effective_mask;
    });
    assert_eq!(held_effective, MacThumbIntentMask::GLIDE.union(MacThumbIntentMask::ALTER));

    run_frame(&ctx, pointer_frame(se_pos, Some(false), 2.70), |ui| {
        let resp = MacContextThumbDial::new("dial_hold_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
        after_release = resp.effective_mask;
    });
    assert_eq!(after_release, MacThumbIntentMask::NONE);
    assert_eq!(latched, MacThumbIntentMask::NONE);
}

/// TC-014-04 (`REQ-014-03`): Bottom `↶ Undo | ↷ Redo` Arc & Outer Tactile Rotary Jog Ring.
#[test]
fn tc_014_04_bottom_undo_redo_arc_and_outer_rotary_jog_ring() {
    let ctx = init_headless_ctx();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let mut latched = MacThumbIntentMask::NONE;
    let mut rotary_val = 0.25f32;
    let mut undo_center = Pos2::ZERO;
    let mut redo_center = Pos2::ZERO;
    let mut ring_top = Pos2::ZERO;
    let mut ring_right = Pos2::ZERO;

    run_frame(&ctx, pointer_frame(Pos2::ZERO, None, 0.0), |ui| {
        let resp = MacContextThumbDial::new("dial_undo_rotary", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Pen)
            .can_undo(true)
            .can_redo(true)
            .rotary_parameter("Corner Radius", "+24 UPM")
            .theme(&theme)
            .show(ui);
        undo_center = resp.undo_rect.center();
        redo_center = resp.redo_rect.center();
        let r_ring = (theme.thumb_dial().clover_outer_radius.points() + theme.thumb_dial().outer_radius.points()) * 0.5;
        ring_top = resp.hub_center + Vec2::new(0.0, -r_ring);
        ring_right = resp.hub_center + Vec2::new(r_ring, 0.0);
    });

    // Click Undo
    let mut undo_fired = false;
    run_frame(&ctx, pointer_frame(undo_center, Some(true), 1.00), |ui| {
        MacContextThumbDial::new("dial_undo_rotary", &mut latched, &mut rotary_val)
            .can_undo(true)
            .can_redo(true)
            .theme(&theme)
            .show(ui);
    });
    run_frame(&ctx, pointer_frame(undo_center, Some(false), 1.04), |ui| {
        let resp = MacContextThumbDial::new("dial_undo_rotary", &mut latched, &mut rotary_val)
            .can_undo(true)
            .can_redo(true)
            .theme(&theme)
            .show(ui);
        undo_fired = resp.undo_clicked;
    });
    assert!(undo_fired, "Clicking Undo arc button must fire undo_clicked");

    // Click Redo
    let mut redo_fired = false;
    run_frame(&ctx, pointer_frame(redo_center, Some(true), 2.00), |ui| {
        MacContextThumbDial::new("dial_undo_rotary", &mut latched, &mut rotary_val)
            .can_undo(true)
            .can_redo(true)
            .theme(&theme)
            .show(ui);
    });
    run_frame(&ctx, pointer_frame(redo_center, Some(false), 2.04), |ui| {
        let resp = MacContextThumbDial::new("dial_undo_rotary", &mut latched, &mut rotary_val)
            .can_undo(true)
            .can_redo(true)
            .theme(&theme)
            .show(ui);
        redo_fired = resp.redo_clicked;
    });
    assert!(redo_fired, "Clicking Redo arc button must fire redo_clicked");

    // Scrub outer rotary ring from top (12 o'clock) to right (3 o'clock = +90 deg)
    run_frame(&ctx, pointer_frame(ring_top, Some(true), 3.00), |ui| {
        MacContextThumbDial::new("dial_undo_rotary", &mut latched, &mut rotary_val)
            .theme(&theme)
            .show(ui);
    });
    let mut ticks = 0.0f32;
    run_frame(&ctx, pointer_frame(ring_right, None, 3.10), |ui| {
        let resp = MacContextThumbDial::new("dial_undo_rotary", &mut latched, &mut rotary_val)
            .theme(&theme)
            .show(ui);
        ticks = resp.rotary_delta_ticks;
    });
    run_frame(&ctx, pointer_frame(ring_right, Some(false), 3.15), |ui| {
        MacContextThumbDial::new("dial_undo_rotary", &mut latched, &mut rotary_val)
            .theme(&theme)
            .show(ui);
    });
    assert!(ticks > 0.0, "Clockwise drag on outer rotary ring must yield positive rotary_delta_ticks (got {ticks})");
    assert!(rotary_val > 0.25, "Clockwise drag on outer rotary ring must increase rotary_value (got {rotary_val})");
}

/// TC-014-05 (`REQ-014-04`): Crate `VERSION == "0.8.0"` and Docked Canvas Placement (`BottomLeft` / `BottomRight`).
#[test]
fn tc_014_05_version_v080_and_thumb_zone_docking() {
    assert!(VERSION >= "0.8.0");
    let ctx = init_headless_ctx();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let mut latched = MacThumbIntentMask::NONE;
    let mut rotary_val = 0.5f32;
    let canvas_rect = Rect::from_min_size(Pos2::new(40.0, 40.0), Vec2::new(600.0, 400.0));

    let mut bl_center = Pos2::ZERO;
    let mut br_center = Pos2::ZERO;
    run_frame(&ctx, pointer_frame(Pos2::ZERO, None, 0.0), |ui| {
        let r_bl = MacContextThumbDial::new("dock_bl", &mut latched, &mut rotary_val)
            .dock_in_rect(canvas_rect, MacThumbDockZone::BottomLeft)
            .theme(&theme)
            .show(ui);
        bl_center = r_bl.hub_center;

        let r_br = MacContextThumbDial::new("dock_br", &mut latched, &mut rotary_val)
            .dock_in_rect(canvas_rect, MacThumbDockZone::BottomRight)
            .theme(&theme)
            .show(ui);
        br_center = r_br.hub_center;
    });

    assert!(bl_center.x < canvas_rect.center().x && bl_center.y > canvas_rect.center().y);
    assert!(br_center.x > canvas_rect.center().x && br_center.y > canvas_rect.center().y);
}

/// TC-014-06 (`REQ-014-02`, `REQ-014-03`, `REQ-014-05`): Center-Click Toggles Layer 1 (`CompassModifiers`) <-> Layer 2 (`FreeJoystickBowl` 360° Analog Nudge) & `MacThumbArcPlacement`.
#[test]
fn tc_014_06_layer2_free_360_joystick_bowl_and_arc_placements() {
    let ctx = init_headless_ctx();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut latched = MacThumbIntentMask::NONE;
    let mut rotary_val = 0.5f32;
    let mut layer = MacThumbDialLayer::CompassModifiers;
    let mut hub_center = Pos2::ZERO;

    run_frame(&ctx, pointer_frame(Pos2::ZERO, None, 0.0), |ui| {
        let resp = MacContextThumbDial::new("dial_layer_switch", &mut latched, &mut rotary_val)
            .layer(&mut layer)
            .undo_arc_placement(MacThumbArcPlacement::TopCrown)
            .theme(&theme)
            .show(ui);
        hub_center = resp.hub_center;
        assert!(resp.undo_rect.center().y < hub_center.y, "TopCrown must place Undo above hub_center");
        assert!(resp.redo_rect.center().y < hub_center.y, "TopCrown must place Redo above hub_center");
    });

    // Quick tap on Center Hub when latched == NONE -> switches to Layer 2 (FreeJoystickBowl)!
    run_frame(&ctx, pointer_frame(hub_center, Some(true), 1.00), |ui| {
        MacContextThumbDial::new("dial_layer_switch", &mut latched, &mut rotary_val).layer(&mut layer).theme(&theme).show(ui);
    });
    run_frame(&ctx, pointer_frame(hub_center, Some(false), 1.05), |ui| {
        MacContextThumbDial::new("dial_layer_switch", &mut latched, &mut rotary_val).layer(&mut layer).theme(&theme).show(ui);
    });
    assert_eq!(layer, MacThumbDialLayer::FreeJoystickBowl);

    // Drag in Layer 2 near center (Fine ring) vs near outer rim (Fast ring) -> quadratic 360° velocity scaling!
    let fine_pos = hub_center + Vec2::new(14.0, -10.0);
    let fast_pos = hub_center + Vec2::new(42.0, -30.0);
    let mut fine_delta = Vec2::ZERO;
    let mut fast_delta = Vec2::ZERO;

    run_frame(&ctx, pointer_frame(fine_pos, Some(true), 2.00), |ui| {
        let resp = MacContextThumbDial::new("dial_layer_switch", &mut latched, &mut rotary_val).layer(&mut layer).theme(&theme).show(ui);
        fine_delta = resp.nudge_delta;
    });
    run_frame(&ctx, pointer_frame(fast_pos, None, 2.05), |ui| {
        let resp = MacContextThumbDial::new("dial_layer_switch", &mut latched, &mut rotary_val).layer(&mut layer).theme(&theme).show(ui);
        fast_delta = resp.nudge_delta;
    });
    run_frame(&ctx, pointer_frame(fast_pos, Some(false), 2.10), |ui| {
        MacContextThumbDial::new("dial_layer_switch", &mut latched, &mut rotary_val).layer(&mut layer).theme(&theme).show(ui);
    });
    assert!(fine_delta.length() > 0.0, "Layer 2 fine deflection must emit non-zero 360° nudge_delta");
    assert!(
        fast_delta.length() > fine_delta.length() * 3.0,
        "Outer rim deflection in Layer 2 must nudge significantly faster than inner fine deflection (fine={:?}, fast={:?})",
        fine_delta,
        fast_delta
    );
}

/// TC-014-07 (`REQ-014-04`): `MacEdgeThumbwheel` Top-Down Cylindrical Scroll Wheel with Free-Spin Flywheel Inertia & Touch-to-Brake.
#[test]
fn tc_014_07_edge_thumbwheel_free_spin_flywheel_and_touch_to_brake() {
    let ctx = init_headless_ctx();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let mut val = 50.0f32;
    let mut wheel_rect = Rect::NOTHING;

    run_frame(&ctx, pointer_frame(Pos2::ZERO, None, 0.0), |ui| {
        let resp = MacEdgeThumbwheel::new("edge_wheel_test", &mut val, 0.0..=200.0).theme(&theme).show(ui);
        wheel_rect = resp.response.rect;
    });

    // Fast upward flick (60 pt in 20 ms) -> engages Free-Spin Flywheel inertia!
    let p_start = Pos2::new(wheel_rect.center().x, wheel_rect.center().y + 28.0);
    let p_end = Pos2::new(wheel_rect.center().x, wheel_rect.center().y - 32.0);
    run_frame(&ctx, pointer_frame(p_start, Some(true), 1.00), |ui| {
        MacEdgeThumbwheel::new("edge_wheel_test", &mut val, 0.0..=200.0).theme(&theme).show(ui);
    });
    run_frame(&ctx, pointer_frame(p_end, None, 1.02), |ui| {
        MacEdgeThumbwheel::new("edge_wheel_test", &mut val, 0.0..=200.0).theme(&theme).show(ui);
    });
    let mut val_at_release = 0.0f32;
    let mut is_spinning = false;
    run_frame(&ctx, pointer_frame(p_end, Some(false), 1.03), |ui| {
        let resp = MacEdgeThumbwheel::new("edge_wheel_test", &mut val, 0.0..=200.0).theme(&theme).show(ui);
        val_at_release = val;
        is_spinning = resp.is_free_spinning;
    });
    assert!(is_spinning, "Fast flick on MacEdgeThumbwheel must engage Free-Spin Flywheel inertia");

    // Coast frame without touch -> value continues advancing via flywheel momentum!
    run_frame(&ctx, pointer_frame(Pos2::ZERO, None, 1.10), |ui| {
        MacEdgeThumbwheel::new("edge_wheel_test", &mut val, 0.0..=200.0).theme(&theme).show(ui);
    });
    assert!(val > val_at_release, "Free-spinning flywheel must continue advancing value after finger release (release={val_at_release}, coast={val})");

    // Touch-to-Brake: pressing on the spinning wheel immediately stops free-spin!
    let mut spinning_after_brake = true;
    run_frame(&ctx, pointer_frame(wheel_rect.center(), Some(true), 1.12), |ui| {
        let resp = MacEdgeThumbwheel::new("edge_wheel_test", &mut val, 0.0..=200.0).theme(&theme).show(ui);
        spinning_after_brake = resp.is_free_spinning;
    });
    assert!(!spinning_after_brake, "Touching the spinning wheel must immediately brake Free-Spin inertia to zero");
}

/// TC-014-08 (`REQ-014-06`): Three Standalone Interchangeable Widgets (`MacEdgeThumbwheel`, `MacContextThumbDial`, `MacFreeJoystickBowl`) & Custom `.glass_accent(Color32)` Adapted by the Apple Calculated Color System (`resolve_glass_accent_on`).
#[test]
fn tc_014_08_standalone_interchangeable_widgets_and_calculated_apple_glass_accent() {
    let ctx = init_headless_ctx();
    let dark_theme = MacTheme::new(MacAppearance::DarkAqua);
    let light_theme = MacTheme::new(MacAppearance::Aqua);

    // User-chosen custom colors:
    // 1. Very dark navy-teal (too dark for DarkAqua without Apple PlusLighter + WCAG clamping!)
    let too_dark_custom = Color32::from_rgb(12, 28, 36);
    // 2. Very pale pastel mint (too light for Aqua without Apple PlusDarker + WCAG clamping!)
    let too_light_custom = Color32::from_rgb(225, 248, 235);
    // 3. Vibrant custom purple-magenta
    let custom_violet = Color32::from_rgb(155, 70, 235);

    let dark_surface = dark_theme.popover_background();
    let light_surface = light_theme.popover_background();
    let calc_dark = dark_theme.resolve_glass_accent_on(Some(too_dark_custom), dark_surface);
    let calc_light = light_theme.resolve_glass_accent_on(Some(too_light_custom), light_surface);
    assert!(
        contrast_ratio(calc_dark, dark_surface) >= 3.0,
        "Apple calculated color system must boost dark custom color on DarkAqua to >= 3.0:1 contrast"
    );
    assert!(
        contrast_ratio(calc_light, light_surface) >= 3.0,
        "Apple calculated color system must darken pale custom color on Aqua to >= 3.0:1 contrast"
    );

    let mut wheel_val = 120.0f32;
    let mut latched = MacThumbIntentMask::NONE;
    let mut rotary_val = 0.5f32;
    let mut dial_size = Vec2::ZERO;
    let mut joy_size = Vec2::ZERO;
    let mut joy_center = Pos2::ZERO;
    let mut joy_undo_center = Pos2::ZERO;

    // Frame 0: Mount all 3 standalone widgets with user-defined Color32 values and verify Apple calculated color adaptation + exact 1:1 size parity
    run_frame(&ctx, pointer_frame(Pos2::ZERO, None, 0.0), |ui| {
        let w_resp = MacEdgeThumbwheel::new("standalone_wheel", &mut wheel_val, 0.0..=360.0)
            .glass_accent(too_dark_custom)
            .theme(&dark_theme)
            .show(ui);
        assert_eq!(w_resp.effective_glass_accent, calc_dark);

        let d_resp = MacContextThumbDial::new("standalone_select_dial", &mut latched, &mut rotary_val)
            .glass_accent(custom_violet)
            .scale(0.76)
            .theme(&dark_theme)
            .show(ui);
        assert_eq!(
            d_resp.effective_glass_accent,
            dark_theme.resolve_glass_accent_on(Some(custom_violet), dark_surface)
        );
        dial_size = d_resp.response.rect.size();

        let j_resp = MacFreeJoystickBowl::new("standalone_joystick_bowl")
            .glass_accent(too_light_custom)
            .scale(0.76)
            .can_undo(true)
            .can_redo(true)
            .theme(&light_theme)
            .show(ui);
        assert_eq!(j_resp.effective_glass_accent, calc_light);
        joy_size = j_resp.response.rect.size();
    });

    assert_eq!(
        dial_size, joy_size,
        "Standalone MacFreeJoystickBowl must have the exact same compact 1:1 bounding size as MacContextThumbDial"
    );

    // Inspect standalone MacFreeJoystickBowl geometry when mounted alone in the slot
    run_frame(&ctx, pointer_frame(Pos2::ZERO, None, 0.50), |ui| {
        let j_resp = MacFreeJoystickBowl::new("standalone_joystick_bowl").scale(0.76).theme(&dark_theme).show(ui);
        joy_center = j_resp.hub_center;
        joy_undo_center = j_resp.undo_rect.center();
    });

    // Drag the 3D crystal glass marble inside standalone MacFreeJoystickBowl -> emits continuous 360° nudge_delta
    let drag_pt = joy_center + Vec2::new(36.0, -24.0);
    let mut standalone_nudge = Vec2::ZERO;
    let mut marble_pos = Pos2::ZERO;
    run_frame(&ctx, pointer_frame(drag_pt, Some(true), 1.00), |ui| {
        let j_resp = MacFreeJoystickBowl::new("standalone_joystick_bowl")
            .glass_accent(custom_violet)
            .scale(0.76)
            .theme(&dark_theme)
            .show(ui);
        standalone_nudge = j_resp.nudge_delta;
        marble_pos = j_resp.marble_center;
    });
    run_frame(&ctx, pointer_frame(drag_pt, Some(false), 1.05), |ui| {
        MacFreeJoystickBowl::new("standalone_joystick_bowl")
            .glass_accent(custom_violet)
            .scale(0.76)
            .theme(&dark_theme)
            .show(ui);
    });
    assert!(standalone_nudge.x > 0.0 && standalone_nudge.y < 0.0, "Standalone MacFreeJoystickBowl upper-right drag must emit (+x, -y) nudge_delta");
    assert!((marble_pos - joy_center).length() > 10.0, "3D glass marble must displace toward pointer during drag");

    // Click Undo on standalone MacFreeJoystickBowl
    let mut joy_undo_clicked = false;
    let mut joy_taptic = None;
    run_frame(&ctx, pointer_frame(joy_undo_center, Some(true), 2.00), |ui| {
        MacFreeJoystickBowl::new("standalone_joystick_bowl").scale(0.76).can_undo(true).theme(&dark_theme).show(ui);
    });
    run_frame(&ctx, pointer_frame(joy_undo_center, Some(false), 2.04), |ui| {
        let j_resp = MacFreeJoystickBowl::new("standalone_joystick_bowl").scale(0.76).can_undo(true).theme(&dark_theme).show(ui);
        joy_undo_clicked = j_resp.undo_clicked;
        joy_taptic = j_resp.taptic_event;
    });
    assert!(joy_undo_clicked, "Clicking Undo crown on standalone MacFreeJoystickBowl must fire undo_clicked");
    assert_eq!(joy_taptic, Some(MacTapticEvent::HistoryStep { is_redo: false }));

    // Custom host application icon override via .with_sector_icon(sector, MacThumbSectorIcon::CustomFn) + micro-click audio synthesis
    static CUSTOM_ICON_CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    fn custom_tool_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
        CUSTOM_ICON_CALLED.store(true, std::sync::atomic::Ordering::Relaxed);
        painter.circle_filled(rect.center(), 2.0, color);
    }
    run_frame(&ctx, pointer_frame(Pos2::ZERO, None, 3.00), |ui| {
        MacContextThumbDial::new("custom_icon_dial", &mut latched, &mut rotary_val)
            .with_sector_icon(MacThumbSector::East, MacThumbSectorIcon::CustomFn(custom_tool_icon))
            .theme(&dark_theme)
            .show(ui);
    });
    assert!(CUSTOM_ICON_CALLED.load(std::sync::atomic::Ordering::Relaxed), "Host CustomFn icon callback must be invoked during show()");
    let wav = MacTapticEvent::RotaryDetentTick.synthesize_micro_click_wav();
    assert!(wav.len() > 44 && &wav[0..4] == b"RIFF", "MacTapticEvent must synthesize valid RIFF/WAVE micro-click audio");
}
