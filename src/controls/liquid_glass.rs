//! View-layer `egui` adapter over the UI-agnostic `vitrea` Model & Controller crate (`SPEC-016`, `v0.9.0`).
//!
//! All 2D/3D Signed Distance Field (SDF) geometry, chain-rule gradients, exact 3D Snellius vector
//! refraction (`glam::Vec3::refract`), chromatic dispersion, physical Schlick-Fresnel reflection,
//! star-convex meniscus triangulation, and area-preserving droplet kinematics ($\det J = 1$)
//! execute inside `vitrea`, configured by [`ThumbDialMetrics`] optical tokens.

use crate::theme::MacTheme;
use crate::tokens::ThumbDialMetrics;
use egui::{Color32, ColorImage, Painter, Pos2, Rect, Shape, Stroke, Vec2};

#[inline]
fn glam_pt(p: Pos2) -> vitrea::GlamVec2 {
    vitrea::GlamVec2::new(p.x, p.y)
}

#[inline]
fn glam_vec(v: Vec2) -> vitrea::GlamVec2 {
    vitrea::GlamVec2::new(v.x, v.y)
}

#[inline]
fn egui_pt(v: vitrea::GlamVec2) -> Pos2 {
    Pos2::new(v.x, v.y)
}

/// Computes the signed distance $d(\vec{x})$ to a rounded capsule/rectangle `rect` with corner radius `radius`.
///
/// Negative inside the capsule, zero on the boundary, positive outside (`REQ-016-01`).
pub fn sdf_rounded_capsule(pt: Pos2, rect: Rect, radius: f32) -> f32 {
    vitrea::sdf_rounded_capsule_sample(glam_pt(pt), glam_pt(rect.center()), glam_vec(rect.size() * 0.5), radius).distance
}

/// $C^1$-continuous polynomial smooth-minimum SDF union ($d_{\cup}(\vec{x})$) for `.glassEffectUnion`
/// metaball coalescing between two Liquid Glass elements (`REQ-016-05`).
pub fn sdf_smooth_union(d_a: f32, d_b: f32, merge_radius: f32) -> f32 {
    let sa = vitrea::SdfSample2d { distance: d_a, gradient: vitrea::GlamVec2::X };
    let sb = vitrea::SdfSample2d { distance: d_b, gradient: vitrea::GlamVec2::X };
    vitrea::sdf_smooth_union_with_grad(sa, sb, merge_radius).distance
}

/// Evaluates the exact signed distance $d_{\text{cushion}}(p)$ to a constant-radius rounded
/// annular pebble sector (`SPEC-016 §4.1`, `REQ-016-01`, `REQ-014-08.1`).
pub fn sdf_annular_rounded_sector(
    pt: Pos2,
    center: Pos2,
    (r_in, r_out): (f32, f32),
    (center_deg, half_sweep_deg): (f32, f32),
    corner_radius: f32,
    _m: &ThumbDialMetrics,
) -> f32 {
    vitrea::sdf_annular_rounded_sector_sample(glam_pt(pt), glam_pt(center), (r_in, r_out), (center_deg, half_sweep_deg), corner_radius).distance
}

/// Evaluates the exact signed distance $d_{\text{stadium}}(p)$ to a curved semicircular stadium
/// arc capsule (`"↶ Undo"` & `"↷ Redo"`, `SPEC-016 §4.1`, `REQ-014-08.3`).
pub fn sdf_curved_stadium_arc(
    pt: Pos2,
    center: Pos2,
    (r_in, r_out): (f32, f32),
    (start_deg, end_deg): (f32, f32),
    _m: &ThumbDialMetrics,
) -> f32 {
    vitrea::sdf_curved_stadium_arc_sample(glam_pt(pt), glam_pt(center), (r_in, r_out), (start_deg, end_deg)).distance
}

/// Evaluates the analytical 3D outward surface normal $\vec{N}(\vec{x}) = (n_x, n_y, n_z)$
/// on a curved Liquid Glass dome or bevel of radial/edge width `bevel_width` via `vitrea::dome_normal_3d_from_sdf` (`REQ-016-01.3`).
pub fn dome_surface_normal_3d(pt: Pos2, center: Pos2, radius: f32, bevel_width: f32) -> [f32; 3] {
    let delta = glam_pt(pt) - glam_pt(center);
    let dist = delta.length();
    let sample = vitrea::SdfSample2d {
        distance: dist - radius.max(1.0),
        gradient: if dist > f32::EPSILON { delta / dist } else { vitrea::GlamVec2::ZERO },
    };
    vitrea::dome_normal_3d_from_sdf(sample, bevel_width).to_array()
}

/// Evaluates [`vitrea::SdfSample2d`] (signed distance + outward unit gradient) for an arbitrary
/// closed 2D polyline contour (`&[Pos2]`, flattened glyph contour or SVG icon path, `REQ-016-01.1`).
pub fn sdf_closed_polyline(pt: Pos2, contour: &[Pos2]) -> vitrea::SdfSample2d {
    let glam_contour: Vec<vitrea::GlamVec2> = contour.iter().copied().map(glam_pt).collect();
    vitrea::sdf_closed_polyline_sample(glam_pt(pt), &glam_contour)
}

/// Evaluates the universal 3D outward surface normal $\vec{N}(\vec{x}) = (n_x, n_y, n_z)$
/// directly from an arbitrary 2D [`vitrea::SdfSample2d`] (capsules, smooth-union metaballs, or closed-polyline glyphs).
pub fn dome_surface_normal_3d_from_sample(sample: vitrea::SdfSample2d, bevel_width: f32) -> [f32; 3] {
    vitrea::dome_normal_3d_from_sdf(sample, bevel_width).to_array()
}

/// Refracts a background screen coordinate `pt` seen through a curved Liquid Glass element
/// with surface normal `normal_3d` and optical thickness `thickness_pt`
/// using exact 3D Snellius vector refraction (`vitrea::refract_ray_snell_3d`, $\eta_{\text{glass}} = 1.48$, `REQ-016-02.1`).
pub fn refract_point_through_glass(pt: Pos2, normal_3d: [f32; 3], thickness_pt: f32, theme: &MacTheme) -> Pos2 {
    let ior = theme.thumb_dial().ior_glass.points();
    let refr = vitrea::refract_ray_snell_3d(glam_pt(pt), vitrea::GlamVec3::from_array(normal_3d), thickness_pt, ior);
    egui_pt(refr)
}

/// Synthesizes a quarter-resolution Dual-Kawase Gaussian blurred and vibrancy-boosted
/// `egui::ColorImage` backdrop for real-time Liquid Glass UV lens refraction (`REQ-016-02`).
pub fn build_liquid_glass_backdrop_image(source: &ColorImage, theme: &MacTheme) -> ColorImage {
    let [sw, sh] = source.size;
    let dw = (sw / 2).max(1);
    let dh = (sh / 2).max(1);
    let mut half = vec![[0.0f32; 4]; dw * dh];

    for y in 0..dh {
        for x in 0..dw {
            let sx = (x * 2).min(sw.saturating_sub(1));
            let sy = (y * 2).min(sh.saturating_sub(1));
            let mut acc = [0.0f32; 4];
            let mut w_sum = 0.0f32;
            for oy in 0..=2usize {
                for ox in 0..=2usize {
                    let px = (sx + ox).saturating_sub(1).min(sw.saturating_sub(1));
                    let py = (sy + oy).saturating_sub(1).min(sh.saturating_sub(1));
                    let c = source.pixels[py * sw + px].to_array();
                    for ch in 0..4 {
                        acc[ch] += f32::from(c[ch]);
                    }
                    w_sum += 1.0;
                }
            }
            for ch in 0..4 {
                half[y * dw + x][ch] = acc[ch] / w_sum;
            }
        }
    }

    let sat_boost = theme.thumb_dial().vibrancy_chroma_boost.points();
    let mut out_pixels = Vec::with_capacity(dw * dh);
    for y in 0..dh {
        for x in 0..dw {
            let mut acc = [0.0f32; 4];
            let mut w_sum = 0.0f32;
            for dy in 0..=2usize {
                for dx in 0..=2usize {
                    let nx = (x + dx).saturating_sub(1).min(dw.saturating_sub(1));
                    let ny = (y + dy).saturating_sub(1).min(dh.saturating_sub(1));
                    let p = half[ny * dw + nx];
                    for ch in 0..4 {
                        acc[ch] += p[ch];
                    }
                    w_sum += 1.0;
                }
            }
            let r = (acc[0] / w_sum).clamp(0.0, f32::from(u8::MAX)) as u8;
            let g = (acc[1] / w_sum).clamp(0.0, f32::from(u8::MAX)) as u8;
            let b = (acc[2] / w_sum).clamp(0.0, f32::from(u8::MAX)) as u8;
            let mut hsva = egui::epaint::Hsva::from_srgb([r, g, b]);
            hsva.s = (hsva.s * sat_boost).clamp(0.0, 1.0);
            out_pixels.push(Color32::from(hsva));
        }
    }
    ColorImage { size: [dw, dh], source_size: Vec2::new(dw as f32, dh as f32), pixels: out_pixels }
}

pub(crate) fn step_dial_pointer_springs(
    ctx: &egui::Context,
    theme: &MacTheme,
    m: &ThumbDialMetrics,
    id_source: egui::Id,
    (hub_center, r_clover_mid, r_outer, r_skirt_out, gap): (Pos2, f32, f32, f32, f32),
    (pointer_pos, latched_sector, held_sector, hovered_sector, now): (
        Option<Pos2>,
        Option<super::MacThumbSector>,
        Option<super::MacThumbSector>,
        Option<super::MacThumbSector>,
        f64,
    ),
    sector_centers: &mut [Pos2; 6],
) -> (Vec2, [f32; 6], Option<(f32, f32)>) {
    let base_spring = theme.tokens().macview.motion.swiftui_snappy.to_spring();
    let fast_ratio = m.viscous_stretch_gain.points();
    let spring = crate::motion::CASpring::new(base_spring.response * fast_ratio, 1.0);
    let raw_p = pointer_pos
        .filter(|p| (*p - hub_center).length() <= r_skirt_out)
        .map(|p| {
            let d = (p - hub_center) / r_outer.max(1.0);
            Vec2::new(d.x.clamp(-1.0, 1.0), d.y.clamp(-1.0, 1.0))
        })
        .unwrap_or_else(|| ctx.data(|d| d.get_temp::<Vec2>(id_source.with("lg_last_raw_p"))).unwrap_or(Vec2::ZERO));
    if pointer_pos.is_some() {
        ctx.data_mut(|d| d.insert_temp(id_source.with("lg_last_raw_p"), raw_p));
    }
    let eff_hover = hovered_sector.or_else(|| {
        if pointer_pos.is_none() {
            ctx.data(|d| d.get_temp::<Option<super::MacThumbSector>>(id_source.with("lg_last_hov_sec"))).flatten()
        } else {
            None
        }
    });
    if pointer_pos.is_some() {
        ctx.data_mut(|d| d.insert_temp(id_source.with("lg_last_hov_sec"), hovered_sector));
    }
    let pointer_norm = Vec2::new(
        spring.animate_f32_at(ctx, id_source.with("lg_ptr_x"), raw_p.x, now, false),
        spring.animate_f32_at(ctx, id_source.with("lg_ptr_y"), raw_p.y, now, false),
    );
    let lift_gain = gap * m.vibrancy_chroma_boost.points();
    let sector_lifts = super::MacThumbSector::ALL.map(|sec| {
        let target_lift = if held_sector == Some(sec) || eff_hover == Some(sec) { 1.0 } else { 0.0 };
        let l = spring.animate_f32_at(ctx, id_source.with(("lg_sec_lift", sec as u8)), target_lift, now, false);
        let c_deg = sec.center_angle_deg(m);
        let c_rad = super::context_thumb_dial::render::deg_to_rad(c_deg, m);
        let sec_unit = Vec2::new(c_rad.cos(), c_rad.sin());
        let lift_off = (sec_unit * 0.5 + pointer_norm * 0.5) * (l * lift_gain);
        sector_centers[sec as usize] =
            super::context_thumb_dial::render::polar_pos(hub_center + lift_off, r_clover_mid + l * gap, c_deg, m);
        l
    });

    // Iter P3 (`REQ-016-03.5`): Coupled Polar DualSpringMorphBlob (`lg_polar_lead_deg`, `lg_polar_trail_deg`)
    let target_sec = held_sector.or(eff_hover).or(latched_sector);
    let polar_morph = target_sec.map(|sec| {
        let raw_deg = sec.center_angle_deg(m);
        let cont_id = id_source.with("lg_polar_cont_deg");
        let prev_cont = ctx.data(|d| d.get_temp::<f32>(cont_id)).unwrap_or(raw_deg);
        let diff = super::context_thumb_dial::render::shortest_angle_diff_deg(raw_deg, prev_cont, m);
        let cont_target = prev_cont + diff;
        ctx.data_mut(|d| d.insert_temp(cont_id, cont_target));
        let lead_s = crate::motion::CASpring::new(base_spring.response * 0.5, base_spring.damping_ratio);
        let trail_s = base_spring;
        let lead_deg = lead_s.animate_f32_at(ctx, id_source.with("lg_polar_lead"), cont_target, now, false);
        let trail_deg = trail_s.animate_f32_at(ctx, id_source.with("lg_polar_trail"), cont_target, now, false);
        (lead_deg, trail_deg)
    });
    (pointer_norm, sector_lifts, polar_morph)
}

/// Paints a 3D semicircular curved stadium capsule (`"↶ Undo"` or `"↷ Redo"`) with true $180^\circ$ rounded end-caps
/// and 5-layer volumetric Liquid Glass dome shading via `vitrea::sdf_curved_stadium_arc_sample` & `vitrea::evaluate_glass_surface_sample` (`REQ-014-08.3`, `SPEC-016 §4.5.1`).
pub(crate) fn paint_curved_stadium_capsule(
    painter: &Painter,
    center: Pos2,
    (r_in, r_out): (f32, f32),
    (start_deg, end_deg): (f32, f32),
    (fill, stroke_col, shadow_col): (Color32, Color32, Color32),
    m: &ThumbDialMetrics,
) {
    let r_mid = (r_in + r_out) * 0.5;
    let h_cap = ((r_out - r_in) * 0.5).max(1.0);
    let half_c = m.half_circle_deg.points();
    let cap_deg = (h_cap / r_mid.max(1.0)) * half_c / std::f32::consts::PI;
    let s_deg = start_deg + cap_deg;
    let e_deg = (end_deg - cap_deg).max(s_deg + 1.0);
    let arc_steps = 20usize;
    let cap_steps = 10usize;
    let mut boundary = Vec::with_capacity((arc_steps + 1) * 2 + cap_steps * 2);

    for i in 0..=arc_steps {
        let a = s_deg + (e_deg - s_deg) * (i as f32 / arc_steps as f32);
        boundary.push(super::context_thumb_dial::render::polar_pos(center, r_out, a, m));
    }
    let end_center = super::context_thumb_dial::render::polar_pos(center, r_mid, e_deg, m);
    for i in 1..cap_steps {
        let frac = i as f32 / cap_steps as f32;
        let a_rad = (e_deg + frac * half_c) * std::f32::consts::PI / half_c;
        boundary.push(end_center + Vec2::new(a_rad.cos() * h_cap, a_rad.sin() * h_cap));
    }
    for i in (0..=arc_steps).rev() {
        let a = s_deg + (e_deg - s_deg) * (i as f32 / arc_steps as f32);
        boundary.push(super::context_thumb_dial::render::polar_pos(center, r_in, a, m));
    }
    let start_center = super::context_thumb_dial::render::polar_pos(center, r_mid, s_deg, m);
    for i in 1..cap_steps {
        let frac = i as f32 / cap_steps as f32;
        let a_rad = (s_deg - half_c + frac * half_c) * std::f32::consts::PI / half_c;
        boundary.push(start_center + Vec2::new(a_rad.cos() * h_cap, a_rad.sin() * h_cap));
    }

    let bw = m.border_stroke_width.points();
    let n = boundary.len();
    let mut shadow_mesh = egui::epaint::Mesh::default();
    let mut mesh = egui::epaint::Mesh::default();
    shadow_mesh.vertices.reserve(n * 2);
    mesh.vertices.reserve(n * 2);
    let params = vitrea::LiquidGlassOpticalParams {
        ior_glass: m.ior_glass.points(),
        ior_dispersion_rgb: [m.ior_red.points(), m.ior_green.points(), m.ior_blue.points()],
        fresnel_f0: m.fresnel_f0.points(),
        bevel_width_pt: h_cap,
        dome_thickness_pt: m.dome_thickness_pt.points(),
        vibrancy_chroma_boost: m.vibrancy_chroma_boost.points(),
        meniscus_merge_k_pt: m.meniscus_merge_k_pt.points(),
    };
    let mut hi_hsva = egui::epaint::Hsva::from(fill);
    hi_hsva.v = (hi_hsva.v + (1.0 - hi_hsva.v) * m.active_fill_opacity.points()).clamp(0.0, 1.0);
    let core_col = Color32::from(hi_hsva);
    let sh_off = Vec2::new(0.0, bw * 2.0);
    for &b_pt in &boundary {
        let dir = b_pt - center;
        let s_pt = center + (dir / dir.length().max(1.0)) * r_mid;
        let b_sdf = vitrea::sdf_curved_stadium_arc_sample(glam_pt(b_pt), glam_pt(center), (r_in, r_out), (start_deg, end_deg));
        let b_surf = vitrea::evaluate_glass_surface_sample(glam_pt(b_pt), b_sdf, &params, None, r_out);
        let rim_mix = (b_surf.fresnel_specular * m.glass_specular_alpha.points() + b_surf.key_illumination * 0.5).clamp(0.0, 1.0);
        let rim_c = Color32::from(egui::epaint::Hsva::from(fill)).lerp_to_gamma(stroke_col, rim_mix);
        shadow_mesh.colored_vertex(s_pt + sh_off, shadow_col);
        shadow_mesh.colored_vertex(b_pt + sh_off, Color32::TRANSPARENT);
        mesh.colored_vertex(s_pt, core_col);
        mesh.colored_vertex(b_pt, rim_c);
    }
    for i in 0..n {
        let (b0, b1) = ((i * 2) as u32, (((i + 1) % n) * 2) as u32);
        shadow_mesh.add_triangle(b0, b0 + 1, b1 + 1);
        shadow_mesh.add_triangle(b0, b1 + 1, b1);
        mesh.add_triangle(b0, b0 + 1, b1 + 1);
        mesh.add_triangle(b0, b1 + 1, b1);
    }
    painter.add(Shape::mesh(shadow_mesh));
    painter.add(Shape::mesh(mesh));
    painter.add(Shape::closed_line(boundary, Stroke::new(bw, stroke_col)));
}

/// Paints text character-by-character along a circular arc centerline of radius `r_mid`
/// centered at `mid_deg`, orienting each glyph to the exact local arc tangent (`REQ-014-08.3`).
pub(crate) fn paint_text_along_arc(
    painter: &Painter,
    center: Pos2,
    r_mid: f32,
    mid_deg: f32,
    text: &str,
    font_id: egui::FontId,
    color: Color32,
    m: &ThumbDialMetrics,
) {
    let (half_c, north_deg) = (m.half_circle_deg.points(), m.angle_north_deg.points());
    let glyphs: Vec<std::sync::Arc<egui::Galley>> =
        text.chars().map(|ch| painter.layout_no_wrap(ch.to_string(), font_id.clone(), color)).collect();
    let total_w: f32 = glyphs.iter().map(|g| g.size().x).sum();
    let mut cursor_x = -total_w * 0.5;
    for g in glyphs {
        let (gw, gh) = (g.size().x, g.size().y);
        let char_mid_x = cursor_x + gw * 0.5;
        cursor_x += gw;
        let char_deg = mid_deg + (char_mid_x / r_mid.max(1.0)) * half_c / std::f32::consts::PI;
        let char_pt = super::context_thumb_dial::render::polar_pos(center, r_mid, char_deg, m);
        let angle_rad = super::context_thumb_dial::render::deg_to_rad(char_deg - north_deg, m);
        let (c_a, s_a) = (angle_rad.cos(), angle_rad.sin());
        let rot_off = Vec2::new((gw * c_a - gh * s_a) * 0.5, (gw * s_a + gh * c_a) * 0.5);
        let mut ts = egui::epaint::TextShape::new(char_pt - rot_off, g, color);
        ts.angle = angle_rad;
        painter.add(Shape::Text(ts));
    }
}

/// Steps a coupled 2-spring 1D interval (`DualSpringMorphBlob`, `REQ-016-03` & `SPEC-016 §4.3` / `§6`).
pub(crate) fn step_dual_spring_morph_interval(
    ctx: &egui::Context,
    id: egui::Id,
    target_min_x: f32,
    target_max_x: f32,
    theme: &MacTheme,
) -> (f32, f32) {
    let base_s = theme.motion().swiftui_snappy.to_spring();
    let fast_s = crate::motion::CASpring::new(base_s.response * 0.5, base_s.damping_ratio);
    let min_id = id.with("morph_min_x");
    let max_id = id.with("morph_max_x");
    let prev_min = ctx.data(|d| d.get_temp::<(f32, f32, f64)>(min_id)).map(|t| t.0).unwrap_or(target_min_x);
    let moving_right = vitrea::DualSpringMorphController::is_moving_right(prev_min, target_min_x);
    let (s_min, s_max) = if moving_right { (base_s, fast_s) } else { (fast_s, base_s) };
    let anim_min = s_min.animate_f32(ctx, min_id, target_min_x);
    let anim_max = s_max.animate_f32(ctx, max_id, target_max_x).max(anim_min + 1.0);
    (anim_min, anim_max)
}

/// Paints a 3D convex Liquid Glass / Sonoma vertical-sheen capsule (`SPEC-016 §6..§9` & `BUG-SONOMA-03`).
pub(crate) fn paint_liquid_glass_capsule(
    painter: &Painter,
    rect: Rect,
    corner_radius: f32,
    fill: Color32,
    border_stroke: Stroke,
    specular_top: Color32,
) {
    painter.rect(rect, egui::CornerRadius::same(corner_radius as u8), fill, border_stroke, egui::StrokeKind::Inside);
    let inset = (corner_radius * 0.5).max(border_stroke.width).min(rect.width() * 0.5).min(rect.height() * 0.5);
    let inner = rect.shrink2(Vec2::new(inset, border_stroke.width));
    if inner.width() > 1.0 && inner.height() > 1.0 {
        let f0_step = vitrea::LiquidGlassOpticalParams::default().fresnel_f0;
        let mut top_h = egui::epaint::Hsva::from(fill);
        top_h.v = (top_h.v + f0_step).clamp(0.0, 1.0);
        let mut bot_h = egui::epaint::Hsva::from(fill);
        bot_h.v = (bot_h.v - f0_step).clamp(0.0, 1.0);
        let top_c = Color32::from(top_h);
        let bot_c = Color32::from(bot_h);
        let mid_y = inner.center().y;
        let rim_h = border_stroke.width.max(1.0);
        let mut mesh = egui::epaint::Mesh::default();
        mesh.colored_vertex(Pos2::new(inner.min.x, inner.min.y), specular_top);
        mesh.colored_vertex(Pos2::new(inner.max.x, inner.min.y), specular_top);
        mesh.colored_vertex(Pos2::new(inner.min.x, inner.min.y + rim_h), top_c);
        mesh.colored_vertex(Pos2::new(inner.max.x, inner.min.y + rim_h), top_c);
        mesh.colored_vertex(Pos2::new(inner.min.x, mid_y), fill);
        mesh.colored_vertex(Pos2::new(inner.max.x, mid_y), fill);
        mesh.colored_vertex(Pos2::new(inner.min.x, inner.max.y), bot_c);
        mesh.colored_vertex(Pos2::new(inner.max.x, inner.max.y), bot_c);
        for band in 0..3u32 {
            let b = band * 2;
            mesh.add_triangle(b, b + 2, b + 1);
            mesh.add_triangle(b + 1, b + 2, b + 3);
        }
        painter.add(Shape::mesh(mesh));
    }
}

/// Applies the area-preserving viscous Squash & Stretch Jacobian ($\det J = 1$, `REQ-016-01.4`)
/// via `vitrea::deform_viscous_droplet_point`.
pub fn deform_viscous_droplet_point(
    pt: Pos2,
    center: Pos2,
    vel_offset: Vec2,
    r_ref: f32,
    press_amount: f32,
) -> Pos2 {
    let params = vitrea::LiquidGlassOpticalParams::default();
    let stretch_g = params.fresnel_f0 * 2.0;
    let press_g = params.fresnel_f0;
    egui_pt(vitrea::deform_viscous_droplet_point(
        glam_pt(pt),
        glam_pt(center),
        glam_vec(vel_offset),
        r_ref,
        press_amount,
        (stretch_g, press_g),
    ))
}

/// Evaluates the per-vertex 3D point-light physical Schlick-Fresnel & Blinn-Phong proximity glint (`REQ-016-02.2`)
/// via `vitrea::evaluate_schlick_fresnel_and_blinn_phong`.
pub fn evaluate_vertex_cursor_fresnel_lighting(
    vertex_pos: Pos2,
    normal_3d: [f32; 3],
    cursor_pos: Option<Pos2>,
    proximity_radius: f32,
) -> f32 {
    let ior = vitrea::LiquidGlassOpticalParams::default().ior_glass;
    vitrea::evaluate_schlick_fresnel_and_blinn_phong(
        glam_pt(vertex_pos),
        vitrea::GlamVec3::from_array(normal_3d),
        cursor_pos.map(glam_pt),
        proximity_radius,
        ior,
    )
}

/// Triangulates and paints the concave smooth-minimum metaball meniscus bridge ($d_{\text{union}}(p) \le 0$)
/// between two adjacent elevated/active cushions using `vitrea::tessellate_meniscus_bridge` (`REQ-016-01.5`).
pub fn paint_meniscus_smooth_union_bridge(
    painter: &Painter,
    center: Pos2,
    (r_in, r_out, corner_r, smooth_k): (f32, f32, f32, f32),
    (deg_a, half_sweep_a, lift_a): (f32, f32, f32),
    (deg_b, half_sweep_b, lift_b): (f32, f32, f32),
    bridge_col: Color32,
    m: &ThumbDialMetrics,
) {
    let bridge_mesh = vitrea::tessellate_meniscus_bridge(
        glam_pt(center),
        (r_in, r_out, corner_r, smooth_k),
        (deg_a, half_sweep_a, lift_a),
        (deg_b, half_sweep_b, lift_b),
    );
    if bridge_mesh.positions.is_empty() || bridge_mesh.indices.is_empty() {
        return;
    }
    let bridge_strength = ((lift_a + lift_b) * 0.5).clamp(0.0, 1.0);
    let max_alpha = (bridge_strength * m.active_fill_opacity.points() * 2.0).clamp(0.0, 1.0);
    let mut mesh = egui::epaint::Mesh::default();
    mesh.vertices.reserve(bridge_mesh.positions.len());
    mesh.indices = bridge_mesh.indices;
    for (p, w) in bridge_mesh.positions.into_iter().zip(bridge_mesh.core_weights) {
        mesh.colored_vertex(egui_pt(p), bridge_col.gamma_multiply((max_alpha * w).clamp(0.0, 1.0)));
    }
    painter.add(Shape::mesh(mesh));
}
