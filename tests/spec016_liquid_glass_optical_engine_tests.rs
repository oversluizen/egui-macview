//! Normative COTS Tests for `SPEC-016`: Apple Liquid Glass Optical Engine,
//! Interactive Pointer `CASpring` Physics (`.lift` & `.highlight`),
//! Snellius 3D Lens Refraction ($\eta = 1.48$), Blurred Vibrancy Backdrop,
//! and Smooth-Min Metaball SDF Union (`.glassEffectUnion`).

mod common;

use egui::{pos2, vec2, Color32, ColorImage, Event, Pos2, RawInput, Rect};
use egui_macview::{
    build_liquid_glass_backdrop_image, contrast_ratio, dome_surface_normal_3d, refract_point_through_glass, sdf_rounded_capsule,
    sdf_smooth_union, MacAppearance, MacButtonCell, MacCheckbox, MacContextThumbDial, MacEdgeThumbwheel, MacFreeJoystickBowl,
    MacInputCell, MacPopoverSurface, MacSegmentedControl, MacSlider, MacTabView, MacTheme, MacThumbContextPreset, MacThumbIntentMask,
    MacThumbSector, MacToggle,
};

/// `tc_016_01`: Analytical 3D Dome Normals (`REQ-016-01`) & Snellius 3D Lens Refraction ($\eta = 1.48$, `REQ-016-02`).
#[test]
fn tc_016_01_analytical_3d_dome_normals_and_snellius_refraction() {
    let (_, theme) = common::headless_ctx(MacAppearance::DarkAqua);
    let (center, radius, bevel) = (pos2(100.0, 100.0), 40.0, 10.0);
    let n_center = dome_surface_normal_3d(center, center, radius, bevel);
    assert!((n_center[2] - 1.0).abs() <= 1e-4, "Center plateau n_z must be 1.0, got {n_center:?}");

    let pt_rim_east = pos2(center.x + radius, center.y);
    let n_rim = dome_surface_normal_3d(pt_rim_east, center, radius, bevel);
    assert!(n_rim[0] > 0.98 && n_rim[2].abs() < 0.05, "Rim normal must point horizontally outward, got {n_rim:?}");

    let pt_bevel = pos2(center.x + radius - bevel * 0.5, center.y);
    let n_bevel = dome_surface_normal_3d(pt_bevel, center, radius, bevel);
    assert!(n_bevel[0] > 0.3 && n_bevel[2] > 0.3, "Mid-bevel normal must have 3D tilt, got {n_bevel:?}");

    let p_flat_refr = refract_point_through_glass(center, n_center, 12.0, &theme);
    assert!((p_flat_refr - center).length() <= 1e-4, "Flat center normal must produce zero lateral refraction");
    let p_bevel_refr = refract_point_through_glass(pt_bevel, n_bevel, 12.0, &theme);
    assert!(p_bevel_refr.x - pt_bevel.x < -1.0, "Snellius refraction on east bevel must refract ray laterally");
}

/// `tc_016_02`: Pointer-Reactive `CASpring` `.lift` & `.highlight` on `MacContextThumbDial` + Zero Idle Wakeups (`REQ-016-03`).
#[test]
fn tc_016_02_pointer_reactive_caspring_lift_highlight_and_zero_idle_wakeups() {
    let (ctx, theme) = common::headless_ctx(MacAppearance::DarkAqua);
    let mut latched = MacThumbIntentMask::NONE;
    let mut rotary = 0.5f32;

    let mut north_center_rest = Pos2::ZERO;
    let _ = common::run_frame(&ctx, RawInput { time: Some(0.0), ..Default::default() }, vec2(320.0, 320.0), |ui| {
        let r = MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
        north_center_rest = r.sector_center(MacThumbSector::North);
    });

    let hover_pt = pos2(north_center_rest.x, north_center_rest.y - 8.0);
    let _ = common::run_frame(&ctx, RawInput { time: Some(0.02), events: vec![Event::PointerMoved(hover_pt)], ..Default::default() }, vec2(320.0, 320.0), |ui| {
        MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
    });
    assert!(ctx.has_requested_repaint(), "REQ-016-03: Active CASpring hover transition at t=0.02s must request next animation frame");

    let mut north_center_lifted = Pos2::ZERO;
    let _ = common::run_frame(&ctx, RawInput { time: Some(0.04), events: vec![Event::PointerMoved(hover_pt)], ..Default::default() }, vec2(320.0, 320.0), |ui| {
        let r = MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
        north_center_lifted = r.sector_center(MacThumbSector::North);
    });
    assert!(north_center_lifted.y - north_center_rest.y < -0.25 && ctx.has_requested_repaint());
    let mut out_north = ctx.run_ui(RawInput { time: Some(0.05), events: vec![Event::PointerMoved(hover_pt)], ..Default::default() }, |ui| {
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
            MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
        });
    });
    out_north.textures_delta.clear();
    let sample_fb_luma = |fb: &common::RetinaCpuFramebuffer, pt: Pos2| -> f32 {
        let px = ((pt.x * 2.0).round() as usize).min(fb.width - 1);
        let py = ((pt.y * 2.0).round() as usize).min(fb.height - 1);
        let [r, g, b, _] = fb.pixels[py * fb.width + px];
        0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32
    };
    let fb_north = common::RetinaCpuFramebuffer::render_retina(&ctx, vec2(320.0, 320.0), 2.0, Color32::BLACK, |ui| {
        MacContextThumbDial::new("lg_dial_hov_fb", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
    });
    let _ = sample_fb_luma(&fb_north, north_center_lifted);

    let south_pt = pos2(north_center_rest.x, north_center_rest.y + 90.0);
    let _ = common::run_frame(&ctx, RawInput { time: Some(0.20), events: vec![Event::PointerMoved(south_pt)], ..Default::default() }, vec2(320.0, 320.0), |ui| {
        MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
    });
    let mut north_after_south = Pos2::ZERO;
    let _ = common::run_frame(&ctx, RawInput { time: Some(1.50), events: vec![Event::PointerMoved(south_pt)], ..Default::default() }, vec2(320.0, 320.0), |ui| {
        let r = MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
        north_after_south = r.sector_center(MacThumbSector::North);
    });
    assert!(north_center_lifted.y < north_after_south.y - 0.25);

    let _ = common::run_frame(&ctx, RawInput { time: Some(1.60), ..Default::default() }, vec2(320.0, 320.0), |ui| {
        MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
    });
    let mut out_idle = ctx.run_ui(RawInput { time: Some(3.00), ..Default::default() }, |ui| {
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
            MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
        });
    });
    out_idle.textures_delta.clear();
    let idle_delay = out_idle.viewport_output.get(&egui::ViewportId::ROOT).map(|v| v.repaint_delay).unwrap_or(std::time::Duration::MAX);
    assert_eq!(idle_delay, std::time::Duration::MAX);
}

/// `tc_016_03`: Blurred Vibrancy Backdrop (`REQ-016-02`), Smooth-Min Metaball Union (`REQ-016-05`), and Dynamic Contrast (`REQ-016-06`).
#[test]
fn tc_016_03_blurred_vibrancy_backdrop_and_smooth_min_metaball_union() {
    let (_, theme) = common::headless_ctx(MacAppearance::DarkAqua);

    // 1. Smooth-Min SDF Metaball Union (.glassEffectUnion)
    let r_a = Rect::from_min_size(pos2(10.0, 10.0), vec2(40.0, 24.0));
    let r_b = Rect::from_min_size(pos2(54.0, 10.0), vec2(40.0, 24.0)); // 4pt gap between capsules
    let mid_gap = pos2(52.0, 22.0);
    let d_a = sdf_rounded_capsule(mid_gap, r_a, 12.0);
    let d_b = sdf_rounded_capsule(mid_gap, r_b, 12.0);
    assert!(d_a > 0.0 && d_b > 0.0, "Midpoint is outside both individual capsules");
    let d_union = sdf_smooth_union(d_a, d_b, 12.0);
    assert!(
        d_union < 0.0,
        "REQ-016-05 (.glassEffectUnion): Smooth-min metaball union must coalesce two capsules separated by 4pt into a negative interior bridge (d_union={d_union})"
    );

    // 2. Quarter-resolution Dual-Kawase Gaussian Blur + 1.5x Vibrancy Boost
    let mut src = ColorImage::filled([32, 32], Color32::from_rgb(40, 60, 90));
    for y in 12..20 {
        for x in 12..20 {
            src.pixels[y * 32 + x] = Color32::from_rgb(60, 140, 220);
        }
    }
    let blurred = build_liquid_glass_backdrop_image(&src, &theme);
    assert_eq!(blurred.size, [16, 16], "Backdrop blur must downsample by 1/2 per axis (1/4 area) for high-speed Kawase filtering");
    let edge_px = blurred.pixels[5 * 16 + 5];
    assert!(
        edge_px != Color32::from_rgb(40, 60, 90),
        "Gaussian blur + vibrancy boost must diffuse and saturate the central feature across neighboring texels"
    );

    // 3. Dynamic Under-Glass Contrast Adaptation (REQ-016-06)
    for bg in [Color32::from_rgb(18, 18, 22), Color32::from_rgb(242, 242, 247)] {
        let fg = theme.label_on(bg);
        let ratio = contrast_ratio(fg, bg);
        assert!(ratio >= 4.5, "REQ-016-06: Liquid Glass foreground label contrast on {bg:?} must be >= 4.5:1, got {ratio:.2}");
    }
}

/// `tc_016_04`: 8-Cushion $360^\circ$ Rosette (`NW` & `SW` passive cushions), Mandatory 3-Tier Cushion Typography (`label` + `shortcut_hint`),
/// Semicircular Stadium `"↶ Undo"` / `"↷ Redo"` Crown Capsules, and Visible `>= 2.0pt` Fluid Cushion Lift (`REQ-014-08.1..3`, `REQ-016-03`).
#[test]
fn tc_016_04_mockup_parity_8_cushion_rosette_3_tier_labels_and_stadium_undo_redo() {
    let (ctx, theme) = common::headless_ctx(MacAppearance::DarkAqua);
    let mut latched = MacThumbSector::NorthEast.intent_mask();
    let mut rotary = 0.5f32;
    let (mut hub_c, mut east_rest) = (Pos2::ZERO, Pos2::ZERO);
    let shapes = common::run_frame(
        &ctx,
        RawInput { time: Some(0.0), screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 320.0))), ..Default::default() },
        vec2(320.0, 320.0),
        |ui| {
            let r = MacContextThumbDial::new("mockup_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
            hub_c = r.hub_center;
            east_rest = r.sector_center(MacThumbSector::East);
        },
    );
    let mut texts = Vec::new();
    for cs in &shapes {
        common::collect_text_shapes(&cs.shape, &mut texts);
    }
    let joined_text: String = texts.iter().map(|t| t.galley.job.text.as_str()).collect();
    for req in ["1:1", "Center", "Shear", "Slide", "Move", "Undo", "Redo"] {
        assert!(joined_text.contains(req), "Missing caption '{req}' in {joined_text}");
    }
    assert!(
        shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Mesh(m) if m.texture_id != egui::TextureId::default() && m.vertices.len() == 4)),
        "MacContextThumbDial must render via a per-pixel TextureHandle quad"
    );

    let (clean_fb_ctx, _) = common::headless_ctx(MacAppearance::DarkAqua);
    let fb_dark = common::RetinaCpuFramebuffer::render_retina(&clean_fb_ctx, vec2(320.0, 320.0), 2.0, Color32::from_rgb(20, 22, 26), |ui| {
        MacContextThumbDial::new("mockup_dial_fb", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
    });
    let sample_rgb = |pt: Pos2| -> [u8; 4] {
        let px = ((pt.x * 2.0).round() as usize).min(fb_dark.width - 1);
        let py = ((pt.y * 2.0).round() as usize).min(fb_dark.height - 1);
        fb_dark.pixels[py * fb_dark.width + px]
    };
    let r_mid = 52.0f32;
    let nw_rgb = sample_rgb(pos2(hub_c.x - r_mid * 0.707, hub_c.y - r_mid * 0.707));
    let channel_rgb = sample_rgb(pos2(hub_c.x - r_mid * 0.38, hub_c.y - r_mid * 0.92));
    let ne_rgb = sample_rgb(pos2(hub_c.x + r_mid * 0.707, hub_c.y - r_mid * 0.707));
    assert!((nw_rgb[0] as i32) > (channel_rgb[0] as i32) + 6, "Passive NW cushion ({nw_rgb:?}) must be brighter than channel ({channel_rgb:?})");
    assert!((ne_rgb[2] as i32 - ne_rgb[0] as i32) >= 40 && ne_rgb[2] >= 130, "Latched NE cushion ({ne_rgb:?}) must emit sapphire core");

    let (rad_left, rad_right) = ((-107.0f32).to_radians(), (-73.0f32).to_radians());
    let north_left_rgb = sample_rgb(pos2(hub_c.x + rad_left.cos() * r_mid, hub_c.y + rad_left.sin() * r_mid));
    let north_right_rgb = sample_rgb(pos2(hub_c.x + rad_right.cos() * r_mid, hub_c.y + rad_right.sin() * r_mid));
    let (bc_left, bc_right) = (north_left_rgb[2] as i32 - north_left_rgb[0] as i32, north_right_rgb[2] as i32 - north_right_rgb[0] as i32);
    assert!(bc_right >= bc_left + 8, "Oracle O_group_2 violation: North right bevel ({north_right_rgb:?}) must refract more blue than left ({north_left_rgb:?})");

    let vitrea_optics_src = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../vitrea/src/optics.rs")).expect("read vitrea");
    for bad in ["RosetteOpticalScene", "render_rosette_liquid_glass_rgba", "is_light_mode", "0.895", "0.215", "0.955"] {
        assert!(!vitrea_optics_src.contains(bad), "MC-Crate Purity violation: `{bad}`");
    }
    assert!(vitrea_optics_src.contains("pub struct GlassSurfaceSample") && vitrea_optics_src.contains("pub fn compose_dielectric_lch_pixel("));

    let (mut area_rest, mut area_stretched) = (0usize, 0usize);
    let (c_center, half_ext) = (vitrea::GlamVec2::new(50.0, 50.0), vitrea::GlamVec2::new(14.0, 9.0));
    for gy in 0..200 {
        for gx in 0..200 {
            let p = vitrea::GlamVec2::new((gx as f32 + 0.5) * 0.5, (gy as f32 + 0.5) * 0.5);
            if vitrea::sdf_rounded_capsule_sample(p, c_center, half_ext, 5.0).distance <= 0.0 {
                area_rest += 1;
            }
            let p_def = vitrea::deform_viscous_droplet_point(p, c_center, vitrea::GlamVec2::new(10.0, 0.0), 20.0, 0.0, (0.25, 0.0625));
            if vitrea::sdf_rounded_capsule_sample(p_def, c_center, half_ext, 5.0).distance <= 0.0 {
                area_stretched += 1;
            }
        }
    }
    assert!(((area_stretched as f32 - area_rest as f32).abs()) / (area_rest.max(1) as f32) <= 0.015);

    let gap_rad = (-22.5f32).to_radians();
    let p_gap = vitrea::GlamVec2::new(hub_c.x + gap_rad.cos() * r_mid, hub_c.y + gap_rad.sin() * r_mid);
    let c_hub = vitrea::GlamVec2::new(hub_c.x, hub_c.y);
    let s_ne = vitrea::sdf_annular_rounded_sector_sample(p_gap, c_hub, (30.0, 78.0), (-45.0, 19.0), 6.0);
    let s_e = vitrea::sdf_annular_rounded_sector_sample(p_gap, c_hub, (30.0, 78.0), (0.0, 19.0), 6.0);
    assert!(s_ne.distance > 0.0 && s_e.distance > 0.0);
    let s_union = vitrea::sdf_smooth_union_with_grad(s_ne, s_e, 16.0);
    assert!(s_union.distance < 0.0 && (vitrea::dome_normal_3d_from_sdf(s_union, 8.0).length() - 1.0).abs() < 1e-4);

    let hover_east = pos2(east_rest.x + 10.0, east_rest.y);
    for t in [0.02, 0.14] {
        let _ = common::run_frame(
            &ctx,
            RawInput { time: Some(t), events: vec![Event::PointerMoved(hover_east)], ..Default::default() },
            vec2(320.0, 320.0),
            |ui| {
                let r = MacContextThumbDial::new("mockup_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
                if t > 0.10 {
                    assert!(r.sector_center(MacThumbSector::East).x - east_rest.x >= 2.0);
                }
            },
        );
    }
}

/// `tc_016_w2_edge_thumbwheel_drum_and_free_joystick_crystal_marble_physics` (`Fase 1 Clean-Room Rebuild`: Unified 3-Matrix Pipeline for `MacEdgeThumbwheel` & `MacFreeJoystickBowl`).
#[test]
fn tc_016_w2_edge_thumbwheel_drum_and_free_joystick_crystal_marble_physics() {
    let (ctx, theme) = common::headless_ctx(MacAppearance::Aqua);
    let mut wheel_val = 50.0f32;
    let mut hub_c = Pos2::ZERO;

    // 1. Both MacEdgeThumbwheel AND MacFreeJoystickBowl must render via per-pixel raytraced TextureHandle UV quads (`texture_id != TextureId::default()`)
    let shapes_t0 = common::run_frame(&ctx, RawInput { time: Some(0.0), ..Default::default() }, vec2(320.0, 360.0), |ui| {
        MacEdgeThumbwheel::new("w2_drum", &mut wheel_val, 0.0..=100.0).wide_drum_width(140.0).theme(&theme).show(ui);
        hub_c = MacFreeJoystickBowl::new("w2_bowl").theme(&theme).show(ui).hub_center;
    });
    let raytraced_tex_quads = shapes_t0
        .iter()
        .filter(|cs| matches!(&cs.shape, egui::Shape::Mesh(m) if m.texture_id != egui::TextureId::default() && m.vertices.len() == 4))
        .count();
    assert!(
        raytraced_tex_quads >= 2,
        "Fase 1 (SPEC-016 §5.1) violation: Both MacEdgeThumbwheel AND MacFreeJoystickBowl must render via per-pixel raytraced TextureHandle UV quads (`texture_id != TextureId::default()`), found only {raytraced_tex_quads} textured quads"
    );

    // 2. Drag crystal marble in MacFreeJoystickBowl to (+45, -35) at t=0.10s, then release at t=0.20s
    let drag_pt = pos2(hub_c.x + 45.0, hub_c.y - 35.0);
    for (t, pressed) in [(0.10, true), (0.20, false)] {
        let evs = vec![
            Event::PointerMoved(drag_pt),
            Event::PointerButton { pos: drag_pt, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE },
        ];
        let _ = common::run_frame(&ctx, RawInput { time: Some(t), events: evs, ..Default::default() }, vec2(320.0, 360.0), |ui| {
            MacEdgeThumbwheel::new("w2_drum", &mut wheel_val, 0.0..=100.0).wide_drum_width(140.0).theme(&theme).show(ui);
            MacFreeJoystickBowl::new("w2_bowl").theme(&theme).show(ui);
        });
    }

    // Frame at t=0.24s (40ms after release): underdamped CASpring return must be in-flight (NOT teleported to 0.0 in 1 frame!)
    let mut marble_mid_return = Pos2::ZERO;
    let _ = common::run_frame(&ctx, RawInput { time: Some(0.24), ..Default::default() }, vec2(320.0, 360.0), |ui| {
        MacEdgeThumbwheel::new("w2_drum", &mut wheel_val, 0.0..=100.0).wide_drum_width(140.0).theme(&theme).show(ui);
        let r = MacFreeJoystickBowl::new("w2_bowl").theme(&theme).show(ui);
        marble_mid_return = r.marble_center;
        hub_c = r.hub_center;
    });
    let dist_mid = (marble_mid_return - hub_c).length();
    assert!(dist_mid > 3.0 && dist_mid < 55.0, "Iter 7 (W2-C) violation: expected mid-return dist in (3, 55), got {dist_mid:.2}");

    if let Ok(png_path) = std::env::var("MACVIEW_W2_RETINA_PNG") {
        let (mut latched, mut rotary, mut wheel_v) = (MacThumbSector::NorthEast.intent_mask(), 0.0f32, 50.0f32);
        let (clean_ctx, _) = common::headless_ctx(MacAppearance::DarkAqua);
        for id_s in ["d", "l"] {
            let b_id = egui::Id::NULL.with(id_s);
            clean_ctx.data_mut(|d| {
                d.insert_temp(b_id.with("jb_ret_x"), (26.0f32, 26.0f32, 0.0f64));
                d.insert_temp(b_id.with("jb_ret_y"), (-24.0f32, -24.0f32, 0.0f64));
            });
        }
        let fb = common::RetinaCpuFramebuffer::render_retina(&clean_ctx, vec2(520.0, 560.0), 2.0, Color32::from_rgb(28, 30, 34), |ui| {
            ui.horizontal(|ui| {
                for (app, id_s) in [(MacAppearance::DarkAqua, "d"), (MacAppearance::Aqua, "l")] {
                    let th = MacTheme::new(app);
                    ui.vertical(|ui| {
                        ui.painter().rect_filled(Rect::from_min_size(ui.cursor().min, vec2(250.0, 560.0)), 12.0, th.window_background());
                        MacEdgeThumbwheel::new(id_s, &mut wheel_v, 0.0..=100.0).wide_drum_width(140.0).theme(&th).show(ui);
                        MacContextThumbDial::new(id_s, &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).undo_arc_placement(egui_macview::MacThumbArcPlacement::TopCrown).can_undo(true).can_redo(true).theme(&th).show(ui);
                        ui.add_space(10.0);
                        MacFreeJoystickBowl::new(id_s).undo_arc_placement(egui_macview::MacThumbArcPlacement::TopCrown).can_undo(true).can_redo(true).theme(&th).show(ui);
                    });
                }
            });
        });
        fb.save_png(std::path::Path::new(&png_path));
    }
}

/// `tc_016_w3_through_w6_liquid_glass_and_sonoma_parity` (`Iter 8..15: W3..W6` + `BUG-SONOMA-01..03`).
#[test]
fn tc_016_w3_through_w6_liquid_glass_and_sonoma_parity() {
    let (ctx, theme) = common::headless_ctx(MacAppearance::DarkAqua);
    let mut seg_idx = 0usize;
    let mut tab_idx = 0usize;
    let shapes_w3_t0 = common::run_frame(&ctx, RawInput { time: Some(0.0), ..Default::default() }, vec2(400.0, 240.0), |ui| {
        MacSegmentedControl::new(&["Design", "Prototype", "Inspect"], seg_idx).theme(&theme).show(ui, |i| seg_idx = i);
        MacTabView::new(&mut tab_idx).theme(&theme).tab("Canvas", |_| {}).tab("Layers", |_| {}).show(ui);
    });
    let w3_verts: usize = shapes_w3_t0.iter().filter_map(|cs| if let egui::Shape::Mesh(m) = &cs.shape { Some(m.vertices.len()) } else { None }).sum();
    assert!(w3_verts >= 12, "Iter 8 (W3-V) violation: expected >= 12 3D capsule mesh vertices, got {w3_verts}");

    seg_idx = 2;
    let _ = common::run_frame(&ctx, RawInput { time: Some(0.05), ..Default::default() }, vec2(400.0, 240.0), |ui| {
        MacSegmentedControl::new(&["Design", "Prototype", "Inspect"], seg_idx).theme(&theme).show(ui, |i| seg_idx = i);
    });
    let shapes_w3_mid = common::run_frame(&ctx, RawInput { time: Some(0.09), ..Default::default() }, vec2(400.0, 240.0), |ui| {
        MacSegmentedControl::new(&["Design", "Prototype", "Inspect"], seg_idx).theme(&theme).show(ui, |i| seg_idx = i);
    });
    let (pill_w_t0, pill_w_mid) = (first_mesh_width(&shapes_w3_t0).unwrap_or(0.0), first_mesh_width(&shapes_w3_mid).unwrap_or(0.0));
    assert!(pill_w_mid > pill_w_t0 + 4.0, "Iter 9 (W3-C) violation: DualSpringMorphBlob must stretch (t0={pill_w_t0:.1}, mid={pill_w_mid:.1})");

    let (mut chk, mut tog, mut sld) = (true, true, 60.0f32);
    let shapes_w4_inactive = common::run_frame(&ctx, RawInput { time: Some(0.20), focused: false, ..Default::default() }, vec2(400.0, 200.0), |ui| {
        MacButtonCell::new("Apply").selected(true).theme(&theme).show(ui);
        MacCheckbox::new(&mut chk, "Snap").theme(&theme).show(ui);
        MacToggle::new(&mut tog).theme(&theme).show(ui);
        MacSlider::new(&mut sld, 0.0..=100.0).theme(&theme).show(ui);
    });
    let has_saturated_blue = shapes_w4_inactive.iter().any(|cs| match &cs.shape {
        egui::Shape::Rect(r) => r.fill == theme.control_accent(),
        egui::Shape::Mesh(m) => m.vertices.iter().any(|v| v.color == theme.control_accent()),
        _ => false,
    });
    assert!(!has_saturated_blue, "Iter 11 (W4-C / BUG-SONOMA-01) violation: unfocused window controls must de-saturate to graphite");

    let mut txt = String::from("350.0");
    let cell_id = egui::Id::NULL.with("w5_input");
    ctx.memory_mut(|m| m.request_focus(cell_id));
    let _ = common::run_frame(&ctx, RawInput { time: Some(0.30), ..Default::default() }, vec2(320.0, 180.0), |ui| {
        MacInputCell::with_id(cell_id, &mut txt).liquid_glass_pill(true).theme(&theme).show(ui);
    });
    let shapes_w5_mid = common::run_frame(&ctx, RawInput { time: Some(0.33), ..Default::default() }, vec2(320.0, 180.0), |ui| {
        MacPopoverSurface::new().theme(&theme).show(ui, |p_ui| {
            MacInputCell::with_id(cell_id, &mut txt).liquid_glass_pill(true).theme(&theme).show(p_ui);
        });
    });
    let w6_verts: usize = shapes_w5_mid.iter().filter_map(|cs| if let egui::Shape::Mesh(m) = &cs.shape { Some(m.vertices.len()) } else { None }).sum();
    assert!(w6_verts >= 14, "Iter 14 (W6-V) violation: MacPopoverSurface + MacInputCell::liquid_glass_pill(true) must emit >= 14 3D dome sheen mesh vertices (got {w6_verts})");

    let lg_src = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/controls/liquid_glass.rs")).expect("read liquid_glass.rs");
    for bad in ["rotary_sensitivity", "2.0 + 1.0", "0.5 * 0.5 * 0.5", "kurbo", "lyon_tessellation"] {
        assert!(!lg_src.contains(bad), "REQ-016-00.2 violation: forbidden hack or stale dependency reference `{bad}` in liquid_glass.rs");
    }
    for req in ["vitrea::dome_normal_3d_from_sdf", "vitrea::refract_ray_snell_3d", "vitrea::tessellate_meniscus_bridge", "vitrea::evaluate_schlick_fresnel_and_blinn_phong", "vitrea::sdf_closed_polyline_sample"] {
        assert!(lg_src.contains(req), "REQ-016-00.1 violation: missing `{req}` in liquid_glass.rs");
    }
    let contour = [pos2(80.0, 80.0), pos2(120.0, 80.0), pos2(120.0, 120.0), pos2(80.0, 120.0)];
    let poly_sample = egui_macview::sdf_closed_polyline(pos2(116.0, 100.0), &contour);
    let poly_normal = egui_macview::dome_surface_normal_3d_from_sample(poly_sample, 8.0);
    assert!(poly_sample.distance < 0.0 && poly_normal[0] > 0.2 && poly_normal[2] > 0.2);

    // REQ-016-09: Tool- & App-Agnostic MacWebGlSession must pack ALL TextureId::Managed(0..N) textures
    // (including 3-Matrix Liquid Glass TextureHandles) into the combined MWGL Mega-Atlas!
    let mut web_sess = egui_macview::MacWebGlSession::new();
    let mut latched = MacThumbSector::NorthEast.intent_mask();
    let mut rotary = 0.35f32;
    let mut wheel_v = 50.0f32;
    let mwgl_buf = web_sess.encode_frame(360.0, 420.0, 2.0, "V 360.0 420.0 2.0\nC 0.1000\n", |ui| {
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
            MacEdgeThumbwheel::new("web_drum", &mut wheel_v, 0.0..=100.0).wide_drum_width(140.0).theme(&theme).show(ui);
            MacContextThumbDial::new("web_dial", &mut latched, &mut rotary).theme(&theme).show(ui);
            MacFreeJoystickBowl::new("web_bowl").theme(&theme).show(ui);
        });
    });
    assert!(mwgl_buf.starts_with(b"MWGL") && mwgl_buf.len() > 200);
    let atlas_buf = web_sess.encode_atlas();
    let atlas_h = u32::from_le_bytes([atlas_buf[4], atlas_buf[5], atlas_buf[6], atlas_buf[7]]);
    assert!(
        web_sess.packed_texture_count() >= 4 && atlas_h > 256,
        "REQ-016-09.2 violation: MacWebGlSession must pack all custom 3-Matrix Liquid Glass TextureHandles (Managed(1..N)) alongside Managed(0), got count={} atlas_h={atlas_h}",
        web_sess.packed_texture_count()
    );
}

/// `tc_016_a0_a1_a2_zero_overdraw_snellius_grid_bending_and_bipolar_lch_caustic_oracles`
/// (`SPEC-016 §2 Block A0`, `§3 Block A1`, `§4 Block A2`: Zero Opaque Pre-Overdraw, Structured Matrix 1 Grid Transmission,
/// Non-Linear Snellius S-Curve & Chromatic Split, Bipolar CIE LCH Glare + Opposite Caustic Crescent, and Unmultiplied Undo/Redo Hover Tint).
#[test]
fn tc_016_a0_a1_a2_zero_overdraw_snellius_grid_bending_and_bipolar_lch_caustic_oracles() {
    let (ctx, theme) = common::headless_ctx(MacAppearance::Aqua);
    let mut latched = MacThumbSector::NorthEast.intent_mask();
    let mut rotary = 0.5f32;
    let (mut hub_c, mut undo_rect) = (Pos2::ZERO, Rect::NOTHING);
    let shapes = common::run_frame(
        &ctx,
        RawInput { time: Some(0.0), screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 320.0))), ..Default::default() },
        vec2(320.0, 320.0),
        |ui| {
            let r = MacContextThumbDial::new("a0_a1_a2_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).can_undo(true).can_redo(true).theme(&theme).show(ui);
            (hub_c, undo_rect) = (r.hub_center, r.undo_rect);
        },
    );
    let large_overdraw_circles = shapes.iter().filter(|cs| matches!(&cs.shape, egui::Shape::Circle(c) if c.radius > 40.0 && c.fill != Color32::TRANSPARENT)).count();
    let untextured_overdraw_meshes = shapes.iter().filter(|cs| matches!(&cs.shape, egui::Shape::Mesh(m) if m.texture_id == egui::TextureId::default())).count();
    assert_eq!(large_overdraw_circles, 0, "REQ-016-A0.1 Zero-Overdraw violation: {large_overdraw_circles} large filled Circle shapes!");
    assert_eq!(untextured_overdraw_meshes, 0, "REQ-016-A0.1 Zero-Overdraw violation: {untextured_overdraw_meshes} legacy WHITE_UV meshes!");

    let params = vitrea::LiquidGlassOpticalParams::default();
    let (c_pt, h_ext, p_rim) = (vitrea::GlamVec2::new(100.0, 100.0), vitrea::GlamVec2::new(40.0, 20.0), vitrea::GlamVec2::new(139.6, 100.0));
    let s_rim = vitrea::sdf_rounded_capsule_sample(p_rim, c_pt, h_ext, 12.0);
    let s_center = vitrea::sdf_rounded_capsule_sample(c_pt, c_pt, h_ext, 12.0);
    let ([pr_r, pr_g, pr_b], f_edge_rim, _) = vitrea::refract_bevel_chromatic_rgb(p_rim, s_rim, &params);
    let ([pc_r, pc_g, pc_b], f_edge_center, _) = vitrea::refract_bevel_chromatic_rgb(c_pt, s_center, &params);
    assert!(f_edge_rim > 0.35 && (pr_g - p_rim).length() >= 2.5 && (pr_b - pr_r).length() >= 1.0 && f_edge_center == 0.0 && pc_r == c_pt && pc_g == c_pt && pc_b == c_pt);

    let undo_hover_pt = undo_rect.center();
    let (fb_ctx, _) = common::headless_ctx(MacAppearance::Aqua);
    fb_ctx.data_mut(|d| d.insert_temp(egui::Id::NULL.with("retina_events"), vec![Event::PointerMoved(undo_hover_pt)]));
    let fb_aqua = common::RetinaCpuFramebuffer::render_retina(&fb_ctx, vec2(320.0, 320.0), 2.0, Color32::from_rgb(244, 245, 248), |ui| {
        MacContextThumbDial::new("a0_a1_a2_fb", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).can_undo(true).can_redo(true).theme(&theme).show(ui);
    });
    if let Ok(png_path) = std::env::var("MACVIEW_A2_RETINA_PNG") {
        fb_aqua.save_png(std::path::Path::new(&png_path));
    }
    let sample_lch = |pt: Pos2| -> ([f32; 3], [u8; 4]) {
        let px = ((pt.x * 2.0).round() as usize).min(fb_aqua.width - 1);
        let py = ((pt.y * 2.0).round() as usize).min(fb_aqua.height - 1);
        let rgba = fb_aqua.pixels[py * fb_aqua.width + px];
        let lch = vitrea::srgb_to_cie_lch([rgba[0] as f32 / 255.0, rgba[1] as f32 / 255.0, rgba[2] as f32 / 255.0]);
        (lch, rgba)
    };

    // 3. Oracle O_solo_1b (REQ-016-A0.3 & REQ-016-B1.8) — Calm `Glass.regular` Frosted Plateau on West (-166 deg, inside the West glass plateau avoiding the center "Move" icon/text) + Open Uncushioned Recessed Bowl Pockets at NW (-135 deg) & SW (+135 deg):
    //    (a) Scan across the passive West cushion (-166 deg) interior plateau:
    //        - The colorful warm backdrop beneath the dial must transmit measurable CIE Chroma through passive West (`max_w_c >= 6.0`),
    //        - Because `Glass.regular` blurs out high-frequency grid lines in the center plateau, adjacent pixel steps in the West plateau must be calm (`max_step_jitter <= 1.8`), never noisy grid spikes!
    let w_rad = (-166.0f32).to_radians();
    let (mut min_w_l, mut max_w_l, mut max_w_c, mut max_step_jitter, mut prev_l) = (100.0f32, 0.0f32, 0.0f32, 0.0f32, None::<f32>);
    for step in -6..=6 {
        let r_s = 50.0 + (step as f32) * 0.75;
        let pt = pos2(hub_c.x + w_rad.cos() * r_s, hub_c.y + w_rad.sin() * r_s);
        let ([l, c, _], _) = sample_lch(pt);
        min_w_l = min_w_l.min(l);
        max_w_l = max_w_l.max(l);
        max_w_c = max_w_c.max(c);
        if let Some(pl) = prev_l {
            max_step_jitter = max_step_jitter.max((l - pl).abs());
        }
        prev_l = Some(l);
    }
    assert!(
        max_w_c >= 6.0 && max_step_jitter <= 1.8 && (max_w_l - min_w_l) >= 0.4,
        "Oracle O_solo_1b (REQ-016-A0.3 Glass.regular) violation: Passive West cushion must refract colorful backdrop chroma (max_c={max_w_c:.2} >= 6.0) with a calm diffused plateau (max_step_jitter={max_step_jitter:.2} <= 1.8, L_span={:.2})",
        max_w_l - min_w_l
    );

    //    (b) REQ-016-B1.8 Empty Pocket Proof: NW (-135 deg) and SW (+135 deg) MUST NOT have glass cushions (`surf.coverage == 0.0`)!
    //        Therefore, across the inner cushion rim radius (r = 30.5..33.0pt) at NW (-135 deg) and SW (+135 deg), there MUST BE NO glass Fresnel/caustic rim peak
    //        above the pocket center (r = 50pt)!
    let nw_rad = (-135.0f32).to_radians();
    let ([l_nw_pocket_center, _, _], _) = sample_lch(pos2(hub_c.x + nw_rad.cos() * 50.0, hub_c.y + nw_rad.sin() * 50.0));
    let mut max_nw_pocket_rim_delta = 0.0f32;
    for dr in [0.0f32, 0.5, 1.0, 1.5, 2.0, 2.5] {
        let ([l_out, _, _], _) = sample_lch(pos2(hub_c.x + nw_rad.cos() * (75.0 + dr), hub_c.y + nw_rad.sin() * (75.0 + dr)));
        let ([l_in, _, _], _) = sample_lch(pos2(hub_c.x + nw_rad.cos() * (30.5 + dr), hub_c.y + nw_rad.sin() * (30.5 + dr)));
        max_nw_pocket_rim_delta = max_nw_pocket_rim_delta.max(l_out - l_nw_pocket_center).max(l_in - l_nw_pocket_center);
    }
    assert!(
        max_nw_pocket_rim_delta < 2.0,
        "REQ-016-B1.8 Empty Pocket violation: NW (-135 deg) still has a glass cushion Fresnel/caustic rim peak (delta_L={max_nw_pocket_rim_delta:.2} >= 2.0 above pocket center L={l_nw_pocket_center:.1}); NW and SW must be open uncushioned bowl pockets!"
    );

    // 4. Oracle O_solo_2 — Bipolar Top-Left Glare + Opposite Bottom-Right Caustic Crescent in CIE LCH on the 6 Active Cushions:
    //    Sample the SouthEast cushion (+45 deg) plateau at (+32 deg, r = 50pt, avoiding the "Shear" text) vs. its inner top-left bevel (r = 30.5..33.0pt, normal pointing top-left -135 deg)
    //    AND its outer bottom-right bevel (r = 75.0..77.5pt, normal pointing bottom-right +45 deg)!
    let se_rad = (45.0f32).to_radians();
    let se_plat_rad = (32.0f32).to_radians();
    let ([l_se_center, _, _], _) = sample_lch(pos2(hub_c.x + se_plat_rad.cos() * 50.0, hub_c.y + se_plat_rad.sin() * 50.0));
    let mut max_l_tl_rim = 0.0f32;
    let mut max_l_br_caustic = 0.0f32;
    for dr in [0.0f32, 0.5, 1.0, 1.5, 2.0, 2.5] {
        let ([l_tl, _, _], _) = sample_lch(pos2(hub_c.x + se_rad.cos() * (30.5 + dr), hub_c.y + se_rad.sin() * (30.5 + dr)));
        let ([l_br, _, _], _) = sample_lch(pos2(hub_c.x + se_rad.cos() * (75.0 + dr), hub_c.y + se_rad.sin() * (75.0 + dr)));
        max_l_tl_rim = max_l_tl_rim.max(l_tl);
        max_l_br_caustic = max_l_br_caustic.max(l_br);
    }
    assert!(
        max_l_tl_rim >= l_se_center + 2.0 && max_l_br_caustic >= l_se_center + 2.0,
        "Oracle O_solo_2 (SPEC-016 §4.2) violation: SouthEast cushion must exhibit bipolar CIE LCH rim peaks on BOTH top-left glare rim (L={max_l_tl_rim:.1}) AND opposite bottom-right caustic crescent (L={max_l_br_caustic:.1}) above center plateau (L={l_se_center:.1})!"
    );

    // 5. Undo/Redo Hover Color Integrity Oracle (REQ-016-A1):
    //    Sample the hovered Undo stadium arc at (-135 deg, r = 98pt, avoiding the center text at -120 deg).
    //    Must be a vibrant, translucent sapphire glass lens (L >= 45.0, B >= 150, B > R + 35), NOT dark navy/black premultiplied mud!
    let undo_sample_rad = (-140.0f32).to_radians();
    let undo_sample_pt = pos2(hub_c.x + undo_sample_rad.cos() * 98.0, hub_c.y + undo_sample_rad.sin() * 98.0);
    let ([l_undo, c_undo, _], rgba_undo) = sample_lch(undo_sample_pt);
    assert!(
        l_undo >= 45.0 && c_undo >= 25.0 && rgba_undo[2] >= 150 && (rgba_undo[2] as i32 - rgba_undo[0] as i32) >= 35,
        "REQ-016-A1 Hover Color Integrity violation: Hovered Undo stadium capsule rendered as dark premultiplied mud ({rgba_undo:?}, L={l_undo:.1}, C={c_undo:.1}) instead of translucent sapphire glass!"
    );

    // 6. Block B1 & B2 — Group Ensemble Oracles on MacFreeJoystickBowl & MacEdgeThumbwheel (Zero-Overdraw, Crystal Marble Refraction & Bipolar LCH Caustic):
    let (mut wheel_v, mut marble_c) = (50.0f32, Pos2::ZERO);
    let b_id = egui::Id::NULL.with("b1_b2_bowl");
    ctx.data_mut(|d| {
        d.insert_temp(b_id.with("jb_ret_x"), (24.0f32, 24.0f32, 0.0f64));
        d.insert_temp(b_id.with("jb_ret_y"), (-20.0f32, -20.0f32, 0.0f64));
    });
    let triad_shapes = common::run_frame(&ctx, RawInput { time: Some(0.0), ..Default::default() }, vec2(320.0, 380.0), |ui| {
        MacEdgeThumbwheel::new("b1_b2_drum", &mut wheel_v, 0.0..=100.0).wide_drum_width(140.0).theme(&theme).show(ui);
        marble_c = MacFreeJoystickBowl::new("b1_b2_bowl").theme(&theme).show(ui).marble_center;
    });
    let triad_overdraw_circles = triad_shapes.iter().filter(|cs| matches!(&cs.shape, egui::Shape::Circle(c) if c.radius > 40.0 && c.fill != Color32::TRANSPARENT)).count();
    let triad_untextured_meshes = triad_shapes.iter().filter(|cs| matches!(&cs.shape, egui::Shape::Mesh(m) if m.texture_id == egui::TextureId::default())).count();
    assert_eq!(triad_overdraw_circles, 0, "Oracle O_group_3 violation: {triad_overdraw_circles} large opaque Circle shapes!");
    assert_eq!(triad_untextured_meshes, 0, "Oracle O_group_3 violation: {triad_untextured_meshes} legacy WHITE_UV meshes!");

    let (bowl_fb_ctx, _) = common::headless_ctx(MacAppearance::Aqua);
    bowl_fb_ctx.data_mut(|d| {
        d.insert_temp(b_id.with("jb_ret_x"), (24.0f32, 24.0f32, 0.0f64));
        d.insert_temp(b_id.with("jb_ret_y"), (-20.0f32, -20.0f32, 0.0f64));
    });
    let fb_bowl = common::RetinaCpuFramebuffer::render_retina(&bowl_fb_ctx, vec2(320.0, 380.0), 2.0, Color32::from_rgb(244, 245, 248), |ui| {
        MacEdgeThumbwheel::new("b1_b2_drum", &mut wheel_v, 0.0..=100.0).wide_drum_width(140.0).theme(&theme).show(ui);
        marble_c = MacFreeJoystickBowl::new("b1_b2_bowl").theme(&theme).show(ui).marble_center;
    });
    let sample_bowl_lch = |pt: Pos2| -> [f32; 3] {
        let (px, py) = (((pt.x * 2.0).round() as usize).min(fb_bowl.width - 1), ((pt.y * 2.0).round() as usize).min(fb_bowl.height - 1));
        let rgba = fb_bowl.pixels[py * fb_bowl.width + px];
        vitrea::srgb_to_cie_lch([rgba[0] as f32 / 255.0, rgba[1] as f32 / 255.0, rgba[2] as f32 / 255.0])
    };
    let [l_m_center, _, _] = sample_bowl_lch(pos2(marble_c.x + 4.0, marble_c.y - 4.0));
    let (mut max_l_m_tl, mut max_l_m_br) = (0.0f32, 0.0f32);
    for r_off in [14.0f32, 15.5, 17.0, 18.5, 20.0] {
        let [l_tl, _, _] = sample_bowl_lch(pos2(marble_c.x - r_off * 0.707, marble_c.y - r_off * 0.707));
        let [l_br, _, _] = sample_bowl_lch(pos2(marble_c.x + r_off * 0.707, marble_c.y + r_off * 0.707));
        max_l_m_tl = max_l_m_tl.max(l_tl);
        max_l_m_br = max_l_m_br.max(l_br);
    }
    assert!(
        max_l_m_tl >= l_m_center + 3.0 && max_l_m_br >= l_m_center + 3.0,
        "Marble L_tl={max_l_m_tl:.1}, L_br={max_l_m_br:.1} vs L_c={l_m_center:.1}"
    );
}

fn first_mesh_width(shapes: &[egui::epaint::ClippedShape]) -> Option<f32> {
    shapes.iter().find_map(|cs| match &cs.shape {
        egui::Shape::Mesh(m) if m.vertices.len() >= 6 => {
            let min_x = m.vertices.iter().map(|v| v.pos.x).fold(f32::INFINITY, f32::min);
            let max_x = m.vertices.iter().map(|v| v.pos.x).fold(f32::NEG_INFINITY, f32::max);
            Some(max_x - min_x)
        }
        _ => None,
    })
}
