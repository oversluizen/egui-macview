//! Normative COTS Verification Suite for `SPEC-014`: Context-Adaptive Bimanual Thumb Dial & Radial Clover / Pie HUD (`MacContextThumbDial`).
//!
//! Verifies `REQ-014-01` through `REQ-014-04` (`TC-014-01` .. `TC-014-05`).

use egui::{Context, Event, PointerButton, Pos2, RawInput, Rect, Vec2};
use egui_macview::{
    setup_symbols_font, MacAppearance, MacContextThumbDial, MacEdgeThumbwheel, MacTheme, MacThumbArcPlacement, MacThumbContextPreset,
    MacThumbDialLayer, MacThumbDockZone, MacThumbIntent, MacThumbIntentMask, MacThumbSector, VERSION,
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
        input.events.push(Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: is_down,
            modifiers: egui::Modifiers::NONE,
        });
    }
    input
}

/// TC-014-01 (`REQ-014-01`): 4 Invariant Cardinal Intentions (`N`, `E`, `S`, `W`) + Diagonal Combination Wedges (`NE`, `SE`).
#[test]
fn tc_014_01_cardinal_intentions_and_diagonal_combination_sectors() {
    let pen = MacThumbContextPreset::Pen.sectors();
    assert_eq!(pen.sector(MacThumbSector::North).label, "∠ 45°");
    assert_eq!(pen.sector(MacThumbSector::North).shortcut_hint, "⇧");
    assert_eq!(pen.sector(MacThumbSector::East).label, "∧ Cusp");
    assert_eq!(pen.sector(MacThumbSector::East).shortcut_hint, "⌥");
    assert_eq!(pen.sector(MacThumbSector::South).label, "★ Genius");
    assert_eq!(pen.sector(MacThumbSector::South).shortcut_hint, "⌘");
    assert_eq!(pen.sector(MacThumbSector::West).label, "✥ Move Pt");
    assert_eq!(pen.sector(MacThumbSector::West).shortcut_hint, "␣");
    assert_eq!(pen.sector(MacThumbSector::NorthEast).label, "∠45°+∧Cusp");
    assert_eq!(pen.sector(MacThumbSector::NorthEast).shortcut_hint, "⇧+⌥");

    let select = MacThumbContextPreset::Select.sectors();
    assert_eq!(select.sector(MacThumbSector::North).label, "∠ Axis");
    assert_eq!(select.sector(MacThumbSector::East).label, "⧉ Clone");
    assert_eq!(select.sector(MacThumbSector::South).label, "↝ Slide");
    assert_eq!(select.sector(MacThumbSector::West).label, "✋ Pan");
    assert_eq!(select.sector(MacThumbSector::SouthEast).label, "↝Slide+1-Side");

    let prim = MacThumbContextPreset::Primitives.sectors();
    assert_eq!(prim.sector(MacThumbSector::North).label, "1:1");
    assert_eq!(prim.sector(MacThumbSector::East).label, "⊙ Center");
    assert_eq!(prim.sector(MacThumbSector::NorthEast).label, "1:1+⊙Center");
    assert_eq!(prim.sector(MacThumbSector::SouthEast).label, "⇄ Shear");
    assert_eq!(prim.sector(MacThumbSector::West).label, "✥ Move");

    let knife = MacThumbContextPreset::Knife.sectors();
    assert_eq!(knife.sector(MacThumbSector::North).label, "∠ 15°");
    assert_eq!(knife.sector(MacThumbSector::East).label, "⊕ Nodes");
    assert_eq!(knife.sector(MacThumbSector::South).label, "⌖ Snap");
    assert_eq!(knife.sector(MacThumbSector::West).label, "✋ Pan");

    assert_eq!(
        MacThumbSector::NorthEast.intent_mask(),
        MacThumbIntentMask::CONSTRAIN.union(MacThumbIntentMask::ALTER)
    );
    assert_eq!(
        MacThumbSector::SouthEast.intent_mask(),
        MacThumbIntentMask::GLIDE.union(MacThumbIntentMask::ALTER)
    );
    assert!(MacThumbSector::NorthEast.intent_mask().contains(MacThumbIntent::Constrain));
    assert!(MacThumbSector::NorthEast.intent_mask().contains(MacThumbIntent::Alter));
}

/// TC-014-02 (`REQ-014-02`): Hybrid Tap-to-Latch (`< 180 ms`), Sequential Multi-Sector Chording, and Center Hub Clear.
#[test]
fn tc_014_02_tap_to_latch_multi_sector_chording_and_hub_clear() {
    let ctx = init_headless_ctx();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let mut latched = MacThumbIntentMask::NONE;
    let mut rotary_val = 0.5f32;
    let mut geom_center = Pos2::ZERO;
    let mut north_pos = Pos2::ZERO;
    let mut east_pos = Pos2::ZERO;
    let mut ne_pos = Pos2::ZERO;

    // Frame 0: Layout inspection
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

    // 1. Quick tap (< 180 ms: 50 ms) on North ("1:1") -> latches CONSTRAIN
    run_frame(&ctx, pointer_frame(north_pos, Some(true), 1.00), |ui| {
        MacContextThumbDial::new("dial_latch_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
    });
    run_frame(&ctx, pointer_frame(north_pos, Some(false), 1.05), |ui| {
        MacContextThumbDial::new("dial_latch_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
    });
    assert_eq!(latched, MacThumbIntentMask::CONSTRAIN);

    // 2. Quick tap (< 180 ms: 50 ms) on East ("⊙ Center") -> latches BOTH CONSTRAIN and ALTER simultaneously!
    run_frame(&ctx, pointer_frame(east_pos, Some(true), 2.00), |ui| {
        MacContextThumbDial::new("dial_latch_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
    });
    run_frame(&ctx, pointer_frame(east_pos, Some(false), 2.05), |ui| {
        MacContextThumbDial::new("dial_latch_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
    });
    assert_eq!(latched, MacThumbIntentMask::CONSTRAIN.union(MacThumbIntentMask::ALTER));

    // 3. Quick tap on Center Thumb Rest Hub -> clears all active latches
    run_frame(&ctx, pointer_frame(geom_center, Some(true), 3.00), |ui| {
        MacContextThumbDial::new("dial_latch_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
    });
    run_frame(&ctx, pointer_frame(geom_center, Some(false), 3.05), |ui| {
        MacContextThumbDial::new("dial_latch_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
    });
    assert_eq!(latched, MacThumbIntentMask::NONE);

    // 4. Single quick tap on diagonal NorthEast ("1:1+⊙Center") -> latches BOTH CONSTRAIN and ALTER at once!
    run_frame(&ctx, pointer_frame(ne_pos, Some(true), 4.00), |ui| {
        MacContextThumbDial::new("dial_latch_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
    });
    run_frame(&ctx, pointer_frame(ne_pos, Some(false), 4.05), |ui| {
        MacContextThumbDial::new("dial_latch_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
    });
    assert_eq!(latched, MacThumbIntentMask::CONSTRAIN.union(MacThumbIntentMask::ALTER));
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

    // 2. Direct press and hold (>= 180 ms: 300 ms) on SouthEast ("⇄ Shear") -> Momentary Hold active, clears on release
    let mut held_effective = MacThumbIntentMask::NONE;
    run_frame(&ctx, pointer_frame(se_pos, Some(true), 2.00), |ui| {
        let resp = MacContextThumbDial::new("dial_hold_test", &mut latched, &mut rotary_val)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
        held_effective = resp.effective_mask;
    });
    assert_eq!(held_effective, MacThumbIntentMask::GLIDE.union(MacThumbIntentMask::ALTER));

    run_frame(&ctx, pointer_frame(se_pos, Some(false), 2.30), |ui| {
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
    assert_eq!(VERSION, "0.8.0");
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
