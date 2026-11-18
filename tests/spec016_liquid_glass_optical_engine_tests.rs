//! Normative COTS Tests for `SPEC-016`: Apple Liquid Glass Optical Engine,
//! Interactive Pointer `CASpring` Physics (`.lift` & `.highlight`),
//! Snellius 3D Lens Refraction ($\eta = 1.48$), Blurred Vibrancy Backdrop,
//! and Smooth-Min Metaball SDF Union (`.glassEffectUnion`).

mod common;

use egui::{pos2, vec2, Color32, ColorImage, Event, Pos2, RawInput, Rect};
use egui_macview::{
    build_liquid_glass_backdrop_image, contrast_ratio, dome_surface_normal_3d, refract_point_through_glass, sdf_rounded_capsule,
    sdf_smooth_union, MacAppearance, MacContextThumbDial, MacThumbContextPreset, MacThumbIntentMask, MacThumbSector,
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

    // Frame 0 at t=0.0: pointer at rest far outside the dial
    let mut north_center_rest = Pos2::ZERO;
    let _ = common::run_frame(
        &ctx,
        RawInput {
            time: Some(0.0),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 320.0))),
            ..Default::default()
        },
        vec2(320.0, 320.0),
        |ui| {
            let r = MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary)
                .preset(MacThumbContextPreset::Primitives)
                .theme(&theme)
                .show(ui);
            north_center_rest = r.sector_center(MacThumbSector::North);
        },
    );

    // Frame 1 at t=0.02s: pointer arrives on North cushion -> initiates CASpring target transition (0.0 -> 1.0)
    let hover_pt = pos2(north_center_rest.x, north_center_rest.y - 8.0);
    let _ = common::run_frame(
        &ctx,
        RawInput {
            time: Some(0.02),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 320.0))),
            events: vec![Event::PointerMoved(hover_pt)],
            ..Default::default()
        },
        vec2(320.0, 320.0),
        |ui| {
            MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary)
                .preset(MacThumbContextPreset::Primitives)
                .theme(&theme)
                .show(ui);
        },
    );
    assert!(
        ctx.has_requested_repaint(),
        "REQ-016-03: Active CASpring hover transition at t=0.02s must request next animation frame"
    );

    // Frame 2 at t=0.04s: spring advances in-flight -> North cushion centroid magnetically lifts upward toward pointer!
    let mut north_center_lifted = Pos2::ZERO;
    let shapes_north = common::run_frame(
        &ctx,
        RawInput {
            time: Some(0.04),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 320.0))),
            events: vec![Event::PointerMoved(hover_pt)],
            ..Default::default()
        },
        vec2(320.0, 320.0),
        |ui| {
            let r = MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary)
                .preset(MacThumbContextPreset::Primitives)
                .theme(&theme)
                .show(ui);
            north_center_lifted = r.sector_center(MacThumbSector::North);
        },
    );

    let lift_shift_y = north_center_lifted.y - north_center_rest.y;
    assert!(
        lift_shift_y < -0.25,
        "REQ-016-03 (.lift): Hovered North cushion centroid must magnetically lift upward toward pointer (shift_y={lift_shift_y}, rest={north_center_rest:?}, lifted={north_center_lifted:?})"
    );
    assert!(
        ctx.has_requested_repaint(),
        "REQ-016-03: In-flight CASpring hover transition at t=0.08s must continue requesting animation frames"
    );

    // Verify 3D mesh vertices on the North cushion are brighter when pointer illuminates North than when pointer moves to South
    let north_luma_hovered = average_mesh_luma_near(&shapes_north, north_center_lifted, 18.0);

    // Move pointer to South at t=0.20s, then evaluate settled South illumination at t=1.50s
    let south_pt = pos2(north_center_rest.x, north_center_rest.y + 90.0);
    let _ = common::run_frame(
        &ctx,
        RawInput {
            time: Some(0.20),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 320.0))),
            events: vec![Event::PointerMoved(south_pt)],
            ..Default::default()
        },
        vec2(320.0, 320.0),
        |ui| {
            MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary)
                .preset(MacThumbContextPreset::Primitives)
                .theme(&theme)
                .show(ui);
        },
    );
    let shapes_south = common::run_frame(
        &ctx,
        RawInput {
            time: Some(1.50),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 320.0))),
            events: vec![Event::PointerMoved(south_pt)],
            ..Default::default()
        },
        vec2(320.0, 320.0),
        |ui| {
            MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary)
                .preset(MacThumbContextPreset::Primitives)
                .theme(&theme)
                .show(ui);
        },
    );
    let north_luma_when_south = average_mesh_luma_near(&shapes_south, north_center_rest, 18.0);
    assert!(
        north_luma_hovered > north_luma_when_south + 2.0,
        "REQ-016-03 (.highlight): Pointer-reactive 3D Blinn-Phong specular & Fresnel must brighten North cushion under cursor ({north_luma_hovered:.2} vs {north_luma_when_south:.2})"
    );

    // Frame at t=1.60s settles any remaining hover release, then t=3.00s verifies zero idle wakeups!
    let _ = common::run_frame(
        &ctx,
        RawInput {
            time: Some(1.60),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 320.0))),
            ..Default::default()
        },
        vec2(320.0, 320.0),
        |ui| {
            MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary)
                .preset(MacThumbContextPreset::Primitives)
                .theme(&theme)
                .show(ui);
        },
    );
    let mut out_idle = ctx.run_ui(
        RawInput {
            time: Some(3.00),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 320.0))),
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
                MacContextThumbDial::new("lg_dial", &mut latched, &mut rotary)
                    .preset(MacThumbContextPreset::Primitives)
                    .theme(&theme)
                    .show(ui);
            });
        },
    );
    out_idle.textures_delta.clear();
    let idle_delay = out_idle.viewport_output.get(&egui::ViewportId::ROOT).map(|v| v.repaint_delay).unwrap_or(std::time::Duration::MAX);
    assert_eq!(
        idle_delay,
        std::time::Duration::MAX,
        "REQ-016-03 (SPEC-004 invariant): Settled LiquidPointerSpring must schedule ZERO idle repaints (got {idle_delay:?})"
    );
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

fn average_mesh_luma_near(shapes: &[egui::epaint::ClippedShape], target: Pos2, radius: f32) -> f32 {
    let mut sum = 0.0f32;
    let mut count = 0usize;
    for cs in shapes {
        if let egui::Shape::Mesh(mesh) = &cs.shape {
            for v in &mesh.vertices {
                if (v.pos - target).length() <= radius {
                    let [r, g, b, _] = v.color.to_array();
                    sum += 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
                    count += 1;
                }
            }
        }
    }
    if count > 0 { sum / count as f32 } else { 0.0 }
}
