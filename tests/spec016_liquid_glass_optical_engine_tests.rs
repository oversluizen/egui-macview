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
    let center = pos2(100.0, 100.0);
    let radius = 40.0;
    let bevel = 10.0;

    // 1. Center plateau: normal points straight toward viewer (0, 0, 1)
    let n_center = dome_surface_normal_3d(center, center, radius, bevel);
    let len_c = (n_center[0] * n_center[0] + n_center[1] * n_center[1] + n_center[2] * n_center[2]).sqrt();
    assert!((len_c - 1.0).abs() <= 1e-4, "Normal at center must be unit length, got {n_center:?}");
    assert!((n_center[2] - 1.0).abs() <= 1e-4, "Center plateau n_z must be 1.0, got {n_center:?}");

    // 2. Right rim edge: normal points horizontally outward (+1, 0, 0)
    let pt_rim_east = pos2(center.x + radius, center.y);
    let n_rim = dome_surface_normal_3d(pt_rim_east, center, radius, bevel);
    let len_r = (n_rim[0] * n_rim[0] + n_rim[1] * n_rim[1] + n_rim[2] * n_rim[2]).sqrt();
    assert!((len_r - 1.0).abs() <= 1e-4, "Normal at rim must be unit length, got {n_rim:?}");
    assert!(n_rim[0] > 0.98 && n_rim[2].abs() < 0.05, "Rim normal must point horizontally outward, got {n_rim:?}");

    // 3. Mid-bevel: sloped 3D normal with both horizontal and vertical components > 0.3
    let pt_bevel = pos2(center.x + radius - bevel * 0.5, center.y);
    let n_bevel = dome_surface_normal_3d(pt_bevel, center, radius, bevel);
    let len_b = (n_bevel[0] * n_bevel[0] + n_bevel[1] * n_bevel[1] + n_bevel[2] * n_bevel[2]).sqrt();
    assert!((len_b - 1.0).abs() <= 1e-4, "Mid-bevel normal must be unit length, got {n_bevel:?}");
    assert!(n_bevel[0] > 0.3 && n_bevel[2] > 0.3, "Mid-bevel normal must have 3D tilt, got {n_bevel:?}");

    // 4. Snellius Refraction: flat center is unshifted; curved bevel refracts inward/outward along normal_xy
    let p_flat_refr = refract_point_through_glass(center, n_center, 12.0, &theme);
    assert!((p_flat_refr - center).length() <= 1e-4, "Flat center normal must produce zero lateral refraction");

    let p_bevel_refr = refract_point_through_glass(pt_bevel, n_bevel, 12.0, &theme);
    let shift_x = p_bevel_refr.x - pt_bevel.x;
    assert!(
        shift_x < -1.0,
        "Snellius refraction (eta = 1.48) on east bevel must refract background ray laterally (< -1.0pt), got shift_x = {shift_x}"
    );
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
    let shapes_north = common::run_frame(&ctx, RawInput { time: Some(0.04), events: vec![Event::PointerMoved(hover_pt)], ..Default::default() }, vec2(320.0, 320.0), |ui| {
        let r = MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
        north_center_lifted = r.sector_center(MacThumbSector::North);
    });
    assert!(north_center_lifted.y - north_center_rest.y < -0.25 && ctx.has_requested_repaint());
    let north_luma_hovered = average_mesh_luma_near(&shapes_north, north_center_lifted, 18.0);

    let south_pt = pos2(north_center_rest.x, north_center_rest.y + 90.0);
    let _ = common::run_frame(&ctx, RawInput { time: Some(0.20), events: vec![Event::PointerMoved(south_pt)], ..Default::default() }, vec2(320.0, 320.0), |ui| {
        MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
    });
    let shapes_south = common::run_frame(&ctx, RawInput { time: Some(1.50), events: vec![Event::PointerMoved(south_pt)], ..Default::default() }, vec2(320.0, 320.0), |ui| {
        MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary).preset(MacThumbContextPreset::Primitives).theme(&theme).show(ui);
    });
    assert!(north_luma_hovered > average_mesh_luma_near(&shapes_south, north_center_rest, 18.0) + 2.0);

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
    let mut hub_c = Pos2::ZERO;
    let mut east_rest = Pos2::ZERO;

    let shapes = common::run_frame(
        &ctx,
        RawInput {
            time: Some(0.0),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 320.0))),
            ..Default::default()
        },
        vec2(320.0, 320.0),
        |ui| {
            let r = MacContextThumbDial::new("mockup_dial", &mut latched, &mut rotary)
                .preset(MacThumbContextPreset::Primitives)
                .theme(&theme)
                .show(ui);
            hub_c = r.hub_center;
            east_rest = r.sector_center(MacThumbSector::East);
        },
    );

    // 1. Verify all 6 active sector labels ("1:1", "1:1+⊙", "Center", "Shear", "Slide", "Move") AND "Undo"/"Redo" are rendered as TextShapes!
    let mut texts = Vec::new();
    for cs in &shapes {
        common::collect_text_shapes(&cs.shape, &mut texts);
    }
    let rendered_strings: Vec<String> = texts.iter().map(|t| t.galley.job.text.clone()).collect();
    let joined_text = rendered_strings.join("");
    for required_caption in ["1:1", "Center", "Shear", "Slide", "Move", "Undo", "Redo"] {
        assert!(
            joined_text.contains(required_caption),
            "REQ-014-08.2/3 violation: Mockup caption '{required_caption}' is completely missing from rendered MacContextThumbDial text shapes! Found: {rendered_strings:?}"
        );
    }

    // 2. Verify Per-Pixel Raytraced Rosette Shader (`REQ-016-01.6` & `REQ-016-02.7`):
    //    MacContextThumbDial must render its 8-cushion Liquid Glass rosette via a textured UV quad (`texture_id != TextureId::default()`)
    //    rather than flat `WHITE_UV` Gouraud triangles or 13-point line segments!
    let has_raytraced_rosette_texture = shapes.iter().any(|cs| {
        matches!(
            &cs.shape,
            egui::Shape::Mesh(m) if m.texture_id != egui::TextureId::default() && m.vertices.len() == 4
        )
    });
    assert!(
        has_raytraced_rosette_texture,
        "REQ-016-01.6/02.7 violation: MacContextThumbDial must render its 8-cushion Liquid Glass rosette via a per-pixel raytraced TextureHandle quad (`texture_id != TextureId::default()`) instead of flat WHITE_UV Gouraud triangles!"
    );

    // 2b. Verify on the rendered 2x Retina framebuffer that:
    //     - Passive NW (-135 deg) and SW (+135 deg) cushions have distinctly elevated dome luminance above the recessed inter-cushion channel,
    //     - Active/latched NE (-45 deg) cushion has intense sapphire/azure volumetric dome emission (`B - R >= 55`) and casts a blue glow into the surrounding dish!
    let fb_dark = common::RetinaCpuFramebuffer::render_retina(&ctx, vec2(320.0, 320.0), 2.0, Color32::from_rgb(20, 22, 26), |ui| {
        MacContextThumbDial::new("mockup_dial_fb", &mut latched, &mut rotary)
            .preset(MacThumbContextPreset::Primitives)
            .theme(&theme)
            .show(ui);
    });
    let sample_rgb = |pt: Pos2| -> [u8; 4] {
        let px = ((pt.x * 2.0).round() as usize).min(fb_dark.width - 1);
        let py = ((pt.y * 2.0).round() as usize).min(fb_dark.height - 1);
        fb_dark.pixels[py * fb_dark.width + px]
    };
    let r_mid = 52.0f32;
    let nw_rgb = sample_rgb(pos2(hub_c.x - r_mid * 0.707, hub_c.y - r_mid * 0.707));
    let channel_rgb = sample_rgb(pos2(hub_c.x - r_mid * 0.38, hub_c.y - r_mid * 0.92)); // gap between N (-90) and NW (-135) at -112.5 deg
    let ne_rgb = sample_rgb(pos2(hub_c.x + r_mid * 0.707, hub_c.y - r_mid * 0.707));
    assert!(
        nw_rgb[0] > channel_rgb[0] + 8,
        "REQ-014-08.1 violation: Passive NW cushion dome ({nw_rgb:?}) must be visibly elevated above the recessed inter-cushion channel ({channel_rgb:?})"
    );
    assert!(
        (ne_rgb[2] as i32 - ne_rgb[0] as i32) >= 55 && ne_rgb[2] >= 150,
        "REQ-016-01.6 violation: Latched NE cushion ({ne_rgb:?}) must emit a deep volumetric sapphire/azure core matching Golden Mockup Screen 1!"
    );

    // 2c. Independent Group Ensemble Oracle O_group_1 (SPEC-016 §4.5.2):
    //     Cross-Neighbor Optical Symmetry Breaking on the 2x Retina Framebuffer!
    //     Passive cushion North (-90 deg) sits between passive NW (-135 deg) on its left and active glowing blue NE (-45 deg) on its right.
    //     Because North's right bevel (-73 deg) refracts the shared Layer 1 Dish Matrix B_group(x,y) containing NE's subsurface blue glow,
    //     the rendered blue chroma excess (B - R) on North's right bevel MUST exceed North's left bevel (-107 deg) by >= 12!
    let rad_left = (-107.0f32).to_radians();
    let rad_right = (-73.0f32).to_radians();
    let north_left_bevel_rgb = sample_rgb(pos2(hub_c.x + rad_left.cos() * r_mid, hub_c.y + rad_left.sin() * r_mid));
    let north_right_bevel_rgb = sample_rgb(pos2(hub_c.x + rad_right.cos() * r_mid, hub_c.y + rad_right.sin() * r_mid));
    let blue_chroma_left = north_left_bevel_rgb[2] as i32 - north_left_bevel_rgb[0] as i32;
    let blue_chroma_right = north_right_bevel_rgb[2] as i32 - north_right_bevel_rgb[0] as i32;
    assert!(
        blue_chroma_right >= blue_chroma_left + 12,
        "Oracle O_group_1 (SPEC-016 §4.5.2) violation: Passive North cushion right bevel ({north_right_bevel_rgb:?}, B-R={blue_chroma_right}) facing active blue NE must refract measurably more subsurface blue chroma from B_group(x,y) than its left bevel ({north_left_bevel_rgb:?}, B-R={blue_chroma_left}) facing passive NW!"
    );

    // 2d. Independent Solo Oracles O_solo_1..3 & Group Oracle O_group_2 + vitrea MC-Crate Purity (SPEC-016 §4.4.3 & §4.5.2):
    let vitrea_optics_src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../vitrea/src/optics.rs"),
    )
    .expect("read vitrea/src/optics.rs");
    for forbidden_in_vitrea in ["RosetteOpticalScene", "render_rosette_liquid_glass_rgba", "is_light_mode", "0.895", "0.215", "0.955"] {
        assert!(
            !vitrea_optics_src.contains(forbidden_in_vitrea),
            "REQ-016-02.6 / SPEC-016 §4.4.3 MC-Crate Purity violation: vitrea/src/optics.rs must NOT contain widget-specific or hardcoded color construct `{forbidden_in_vitrea}`!"
        );
    }
    assert!(
        vitrea_optics_src.contains("pub struct GlassSurfaceSample") && vitrea_optics_src.contains("pub fn evaluate_glass_surface_sample("),
        "SPEC-016 §4.5.1 violation: vitrea/src/optics.rs must expose shape-agnostic `GlassSurfaceSample` and `evaluate_glass_surface_sample`"
    );

    // Oracle O_solo_3: Monte-Carlo / 2D Pixel Grid Area Conservation under Viscous Stretch (det J = 1, <= 1.5% discrete grid error)
    let (mut area_rest, mut area_stretched) = (0usize, 0usize);
    let c_center = vitrea::GlamVec2::new(50.0, 50.0);
    let half_ext = vitrea::GlamVec2::new(14.0, 9.0);
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
    let rel_area_err = ((area_stretched as f32 - area_rest as f32).abs()) / (area_rest.max(1) as f32);
    assert!(
        rel_area_err <= 0.015,
        "Oracle O_solo_3 (SPEC-016 §4.5.2) violation: 2D pixel-integrated droplet area under viscous stretch ({area_stretched}) must match resting area ({area_rest}) within <= 1.5%, got {:.2}%",
        rel_area_err * 100.0
    );

    // Oracle O_group_2: Topological Meniscus Bridge Closure at inter-cushion gap midpoint (-22.5 deg between NE -45 deg and E 0 deg)
    let gap_rad = (-22.5f32).to_radians();
    let p_gap = vitrea::GlamVec2::new(hub_c.x + gap_rad.cos() * r_mid, hub_c.y + gap_rad.sin() * r_mid);
    let c_hub = vitrea::GlamVec2::new(hub_c.x, hub_c.y);
    let s_ne_rest = vitrea::sdf_annular_rounded_sector_sample(p_gap, c_hub, (30.0, 78.0), (-45.0, 19.0), 6.0);
    let s_e_rest = vitrea::sdf_annular_rounded_sector_sample(p_gap, c_hub, (30.0, 78.0), (0.0, 19.0), 6.0);
    assert!(s_ne_rest.distance > 0.0 && s_e_rest.distance > 0.0, "Oracle O_group_2: Gap midpoint must be outside glass at rest");
    let s_union_active = vitrea::sdf_smooth_union_with_grad(s_ne_rest, s_e_rest, 16.0);
    let n_bridge = vitrea::dome_normal_3d_from_sdf(s_union_active, 8.0);
    assert!(
        s_union_active.distance < 0.0 && (n_bridge.length() - 1.0).abs() < 1e-4,
        "Oracle O_group_2 (SPEC-016 §4.5.2) violation: Smooth-union meniscus bridge must topologically close across inter-cushion gap (d={:.2})",
        s_union_active.distance
    );

    // 3. Verify clearly visible fluid cushion lift (>= 2.0pt radial/magnetic shift at t=0.14s on a genuine 0.28s underdamped spring)
    let hover_east = pos2(east_rest.x + 10.0, east_rest.y);
    let _ = common::run_frame(
        &ctx,
        RawInput {
            time: Some(0.02),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 320.0))),
            events: vec![Event::PointerMoved(hover_east)],
            ..Default::default()
        },
        vec2(320.0, 320.0),
        |ui| {
            MacContextThumbDial::new("mockup_dial", &mut latched, &mut rotary)
                .preset(MacThumbContextPreset::Primitives)
                .theme(&theme)
                .show(ui);
        },
    );
    let mut east_lifted = Pos2::ZERO;
    let _ = common::run_frame(
        &ctx,
        RawInput {
            time: Some(0.14),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 320.0))),
            events: vec![Event::PointerMoved(hover_east)],
            ..Default::default()
        },
        vec2(320.0, 320.0),
        |ui| {
            let r = MacContextThumbDial::new("mockup_dial", &mut latched, &mut rotary)
                .preset(MacThumbContextPreset::Primitives)
                .theme(&theme)
                .show(ui);
            east_lifted = r.sector_center(MacThumbSector::East);
        },
    );
    let shift_x = east_lifted.x - east_rest.x;
    assert!(
        shift_x >= 2.0,
        "REQ-016-03 violation: Fluid cushion lift must be clearly visible to the human eye (>= 2.0pt at t=0.14s), got shift_x={shift_x:.2}pt"
    );
}

/// `tc_016_w2_edge_thumbwheel_drum_and_free_joystick_crystal_marble_physics` (`Iter 6: W2-V` & `Iter 7: W2-C`).
#[test]
fn tc_016_w2_edge_thumbwheel_drum_and_free_joystick_crystal_marble_physics() {
    let (ctx, theme) = common::headless_ctx(MacAppearance::Aqua);
    let mut wheel_val = 50.0f32;
    let mut hub_c = Pos2::ZERO;

    // 1. Render MacEdgeThumbwheel (wide drum) and MacFreeJoystickBowl at t=0.0
    let shapes_t0 = common::run_frame(&ctx, RawInput { time: Some(0.0), ..Default::default() }, vec2(320.0, 360.0), |ui| {
        MacEdgeThumbwheel::new("w2_drum", &mut wheel_val, 0.0..=100.0).wide_drum_width(140.0).theme(&theme).show(ui);
        hub_c = MacFreeJoystickBowl::new("w2_bowl").theme(&theme).show(ui).hub_center;
    });
    let total_mesh_verts: usize = shapes_t0.iter().filter_map(|cs| if let egui::Shape::Mesh(m) = &cs.shape { Some(m.vertices.len()) } else { None }).sum();
    assert!(total_mesh_verts >= 350, "Iter 6 (W2-V) violation: expected >= 350 mesh vertices, got {total_mesh_verts}");

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
        let mut latched = MacThumbSector::NorthEast.intent_mask();
        let mut rotary = 0.0f32;
        let mut wheel_v = 50.0f32;
        let (clean_ctx, _) = common::headless_ctx(MacAppearance::DarkAqua);
        let fb = common::RetinaCpuFramebuffer::render_retina(&clean_ctx, vec2(520.0, 560.0), 2.0, Color32::from_rgb(28, 30, 34), |ui| {
            ui.horizontal(|ui| {
                for (app, id_s) in [(MacAppearance::DarkAqua, "d"), (MacAppearance::Aqua, "l")] {
                    let b_id = egui::Id::NULL.with(id_s);
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(b_id.with("jb_ret_x"), (26.0f32, 26.0f32, 0.0f64));
                        d.insert_temp(b_id.with("jb_ret_y"), (-24.0f32, -24.0f32, 0.0f64));
                    });
                    let th = MacTheme::new(app);
                    ui.vertical(|ui| {
                        ui.painter().rect_filled(Rect::from_min_size(ui.cursor().min, vec2(250.0, 560.0)), 12.0, th.window_background());
                        MacEdgeThumbwheel::new(id_s, &mut wheel_v, 0.0..=100.0).wide_drum_width(140.0).theme(&th).show(ui);
                        let dial_r = MacContextThumbDial::new(id_s, &mut latched, &mut rotary)
                            .preset(MacThumbContextPreset::Primitives)
                            .undo_arc_placement(egui_macview::MacThumbArcPlacement::TopCrown)
                            .can_undo(true)
                            .can_redo(true)
                            .theme(&th)
                            .show(ui);
                        let _ = dial_r.sector_center(MacThumbSector::NorthEast);
                        ui.add_space(10.0);
                        MacFreeJoystickBowl::new(id_s)
                            .undo_arc_placement(egui_macview::MacThumbArcPlacement::TopCrown)
                            .can_undo(true)
                            .can_redo(true)
                            .theme(&th)
                            .show(ui);
                    });
                }
            });
        });
        fb.save_png(std::path::Path::new(&png_path));
    }
}

fn average_mesh_luma_near(shapes: &[egui::epaint::ClippedShape], target: Pos2, radius: f32) -> f32 {
    let (mut sum, mut count) = (0.0f32, 0usize);
    for cs in shapes {
        if let egui::Shape::Mesh(mesh) = &cs.shape {
            for v in mesh.vertices.iter().filter(|v| (v.pos - target).length() <= radius) {
                let [r, g, b, _] = v.color.to_array();
                sum += 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
                count += 1;
            }
        }
    }
    if count > 0 { sum / count as f32 } else { 0.0 }
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
        MacInputCell::with_id(cell_id, &mut txt).theme(&theme).show(ui);
    });
    let shapes_w5_mid = common::run_frame(&ctx, RawInput { time: Some(0.33), ..Default::default() }, vec2(320.0, 180.0), |ui| {
        MacPopoverSurface::new().theme(&theme).show(ui, |p_ui| {
            MacInputCell::with_id(cell_id, &mut txt).theme(&theme).show(p_ui);
        });
    });
    let w6_verts: usize = shapes_w5_mid.iter().filter_map(|cs| if let egui::Shape::Mesh(m) = &cs.shape { Some(m.vertices.len()) } else { None }).sum();
    assert!(w6_verts >= 6, "Iter 14 (W6-V) violation: MacPopoverSurface must emit 3D dome sheen mesh (got {w6_verts})");

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
