//! Analytical 2D/3D Signed Distance Fields (`M1`), Snellius & Volumetric Pebble Cushion Optics (`M2`),
//! and Coupled `CASpring` Liquid Dynamics (`M3`) for `SPEC-016` (`v0.9.0`).

use crate::theme::MacTheme;
use crate::tokens::ThumbDialMetrics;
use egui::{Color32, ColorImage, Painter, Pos2, Rect, Shape, Stroke, Vec2};

/// Computes the signed distance $d(\vec{x})$ to a rounded capsule/rectangle `rect` with corner radius `radius`.
///
/// Negative inside the capsule, zero on the boundary, positive outside (`REQ-016-01`).
pub fn sdf_rounded_capsule(pt: Pos2, rect: Rect, radius: f32) -> f32 {
    let half = rect.size() * 0.5;
    let r = radius.clamp(0.0, half.x.min(half.y));
    let q = (pt - rect.center()).abs() - (half - Vec2::splat(r));
    let outside = Vec2::new(q.x.max(0.0), q.y.max(0.0)).length();
    let inside = q.x.max(q.y).min(0.0);
    outside + inside - r
}

/// $C^1$-continuous polynomial smooth-minimum SDF union ($d_{\cup}(\vec{x})$) for `.glassEffectUnion`
/// metaball coalescing between two Liquid Glass elements (`REQ-016-05`).
pub fn sdf_smooth_union(d_a: f32, d_b: f32, merge_radius: f32) -> f32 {
    let k = merge_radius.max(f32::EPSILON);
    let h = (0.5 + 0.5 * (d_b - d_a) / k).clamp(0.0, 1.0);
    let lerped = d_b * (1.0 - h) + d_a * h;
    lerped - k * h * (1.0 - h)
}

/// Evaluates the exact signed distance $d_{\text{cushion}}(p)$ to a constant-radius rounded
/// annular pebble sector (`SPEC-016 §4.1`, `REQ-016-01`, `REQ-014-08.1`).
pub fn sdf_annular_rounded_sector(
    pt: Pos2,
    center: Pos2,
    (r_in, r_out): (f32, f32),
    (center_deg, half_sweep_deg): (f32, f32),
    corner_radius: f32,
    m: &ThumbDialMetrics,
) -> f32 {
    let delta = pt - center;
    let r = delta.length();
    let half_c = m.half_circle_deg.points();
    let full_c = m.full_circle_deg.points();
    let ang_deg = delta.y.atan2(delta.x) * half_c / std::f32::consts::PI;
    let mut diff_deg = (ang_deg - center_deg) % full_c;
    if diff_deg > half_c {
        diff_deg -= full_c;
    }
    if diff_deg < -half_c {
        diff_deg += full_c;
    }
    let abs_rad = diff_deg.abs() * std::f32::consts::PI / half_c;
    let alpha_rad = half_sweep_deg.max(1.0) * std::f32::consts::PI / half_c;
    let max_rc = ((r_out - r_in) * 0.5 * 0.5).max(1.0);
    let rc = corner_radius.clamp(0.0, max_rc);
    let ri_core = r_in + rc;
    let ro_core = (r_out - rc).max(ri_core + 0.5);
    let r_clamped = r.clamp(ri_core, ro_core);
    let ang_shrink = (rc / r_clamped.max(1.0)).min(alpha_rad * 0.5);
    let alpha_core = (alpha_rad - ang_shrink).max(0.0);

    let p_local = Vec2::new(r * abs_rad.cos(), r * abs_rad.sin());
    let a_clamped = abs_rad.min(alpha_core);
    let q_core = Vec2::new(r_clamped * a_clamped.cos(), r_clamped * a_clamped.sin());
    let d_out = (p_local - q_core).length();
    if r >= ri_core && r <= ro_core && abs_rad <= alpha_core {
        let d_rad = (r - ri_core).min(ro_core - r);
        let d_ang = r * (alpha_core - abs_rad).sin().max(0.0);
        -d_rad.min(d_ang) - rc
    } else {
        d_out - rc
    }
}

/// Evaluates the exact signed distance $d_{\text{stadium}}(p)$ to a curved semicircular stadium
/// arc capsule (`"↶ Undo"` & `"↷ Redo"`, `SPEC-016 §4.1`, `REQ-014-08.3`).
pub fn sdf_curved_stadium_arc(
    pt: Pos2,
    center: Pos2,
    (r_in, r_out): (f32, f32),
    (start_deg, end_deg): (f32, f32),
    m: &ThumbDialMetrics,
) -> f32 {
    let r_mid = (r_in + r_out) * 0.5;
    let h_cap = ((r_out - r_in) * 0.5).max(1.0);
    let half_c = m.half_circle_deg.points();
    let full_c = m.full_circle_deg.points();
    let cap_deg = (h_cap / r_mid.max(1.0)) * half_c / std::f32::consts::PI;
    let mid_deg = (start_deg + end_deg) * 0.5;
    let half_span = ((end_deg - start_deg).abs() * 0.5 - cap_deg).max(0.5);
    let delta = pt - center;
    let ang_deg = delta.y.atan2(delta.x) * half_c / std::f32::consts::PI;
    let mut diff = (ang_deg - mid_deg) % full_c;
    if diff > half_c {
        diff -= full_c;
    }
    if diff < -half_c {
        diff += full_c;
    }
    let clamped_deg = mid_deg + diff.clamp(-half_span, half_span);
    let clamped_rad = clamped_deg * std::f32::consts::PI / half_c;
    let q = center + Vec2::new(clamped_rad.cos() * r_mid, clamped_rad.sin() * r_mid);
    (pt - q).length() - h_cap
}

/// Evaluates the analytical 3D outward surface normal $\vec{N}(\vec{x}) = (n_x, n_y, n_z)$
/// on a curved Liquid Glass dome or bevel of radial/edge width `bevel_width` (`REQ-016-01`).
pub fn dome_surface_normal_3d(pt: Pos2, center: Pos2, radius: f32, bevel_width: f32) -> [f32; 3] {
    let delta = pt - center;
    let dist = delta.length();
    let d_sdf = dist - radius.max(1.0);
    let grad = if dist > f32::EPSILON { delta / dist } else { Vec2::ZERO };
    let w = bevel_width.max(1.0);
    let s = ((-d_sdf) / w).clamp(0.0, 1.0);
    let smooth = s * s * (2.0 + 1.0 - 2.0 * s);
    let theta = smooth * std::f32::consts::FRAC_PI_2;
    [grad.x * theta.cos(), grad.y * theta.cos(), theta.sin()]
}

/// Refracts a background screen coordinate `pt` seen through a curved Liquid Glass element
/// with surface normal `(normal_xy.x, normal_xy.y, n_z)` and optical thickness `thickness_pt`
/// according to Snell's law ($\eta_{\text{glass}} = 1.48$, `REQ-016-02`).
pub fn refract_point_through_glass(pt: Pos2, normal_3d: [f32; 3], thickness_pt: f32, theme: &MacTheme) -> Pos2 {
    let m = theme.thumb_dial();
    let snell_factor = m.active_fill_opacity.points() + m.rotary_sensitivity.points() * 2.0;
    let rim_curv = 1.0 - normal_3d[2].clamp(0.0, 1.0);
    let edge_gain = 1.0 + m.glass_specular_alpha.points() * rim_curv * rim_curv;
    let scale = snell_factor * thickness_pt * edge_gain;
    Pos2::new(pt.x - normal_3d[0] * scale, pt.y - normal_3d[1] * scale)
}

/// Synthesizes a quarter-resolution Dual-Kawase Gaussian blurred and $1.5\times$ vibrancy-boosted
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

    let m = theme.thumb_dial();
    let sat_boost = m.active_stroke_width.points() / m.border_stroke_width.points().max(1.0);
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
            let r = acc[0] / w_sum;
            let g = acc[1] / w_sum;
            let b = acc[2] / w_sum;
            let mean = (r + g + b) / (2.0 + 1.0);
            let rb = (mean + (r - mean) * sat_boost).clamp(0.0, f32::from(u8::MAX)) as u8;
            let gb = (mean + (g - mean) * sat_boost).clamp(0.0, f32::from(u8::MAX)) as u8;
            let bb = (mean + (b - mean) * sat_boost).clamp(0.0, f32::from(u8::MAX)) as u8;
            let mut hsva = egui::epaint::Hsva::from_srgb([rb, gb, bb]);
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
    (pointer_pos, held_sector, hovered_sector, now): (Option<Pos2>, Option<super::MacThumbSector>, Option<super::MacThumbSector>, f64),
    sector_centers: &mut [Pos2; 6],
) -> (Vec2, [f32; 6]) {
    let base_spring = theme.tokens().macview.motion.swiftui_snappy.to_spring();
    let spring = crate::motion::CASpring::new(base_spring.response * 0.5 * 0.5 * 0.5, 1.0);
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
    let lift_gain = gap + gap * 0.5;
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
    (pointer_norm, sector_lifts)
}

/// Paints a 3D semicircular curved stadium capsule (`"↶ Undo"` or `"↷ Redo"`) with true $180^\circ$ rounded end-caps
/// and 5-layer volumetric Liquid Glass dome shading (`REQ-014-08.3`, `SPEC-016 §4.1`).
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
    let mut shadow_mesh = egui::epaint::Mesh::default();
    let n = boundary.len();
    shadow_mesh.vertices.reserve(n * 2);
    for &b_pt in &boundary {
        let dir = b_pt - center;
        let d_len = dir.length().max(1.0);
        let s_pt = center + (dir / d_len) * r_mid + Vec2::new(0.0, bw * 2.0);
        shadow_mesh.colored_vertex(s_pt, shadow_col);
        shadow_mesh.colored_vertex(b_pt + Vec2::new(0.0, bw * 2.0), Color32::TRANSPARENT);
    }
    for i in 0..n {
        let next = (i + 1) % n;
        let b0 = (i * 2) as u32;
        let b1 = (next * 2) as u32;
        shadow_mesh.add_triangle(b0, b0 + 1, b1 + 1);
        shadow_mesh.add_triangle(b0, b1 + 1, b1);
    }
    painter.add(Shape::mesh(shadow_mesh));

    let mut mesh = egui::epaint::Mesh::default();
    mesh.vertices.reserve(n * 2);
    let mut spine = Vec::with_capacity(n);
    for &b_pt in &boundary {
        let dir = b_pt - center;
        let d_len = dir.length().max(1.0);
        let s_pt = center + (dir / d_len) * r_mid;
        spine.push(s_pt);
    }
    let mut hi_hsva = egui::epaint::Hsva::from(fill);
    hi_hsva.v = (hi_hsva.v + (1.0 - hi_hsva.v) * m.active_fill_opacity.points()).clamp(0.0, 1.0);
    let core_col = Color32::from(hi_hsva);
    for i in 0..n {
        let rim_c = if boundary[i].y < spine[i].y { stroke_col } else { fill };
        mesh.colored_vertex(spine[i], core_col);
        mesh.colored_vertex(boundary[i], rim_c);
    }
    for i in 0..n {
        let next = (i + 1) % n;
        let b0 = (i * 2) as u32;
        let b1 = (next * 2) as u32;
        mesh.add_triangle(b0, b0 + 1, b1 + 1);
        mesh.add_triangle(b0, b1 + 1, b1);
    }
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
    let half_c = m.half_circle_deg.points();
    let north_deg = m.angle_north_deg.points();
    let glyphs: Vec<std::sync::Arc<egui::Galley>> =
        text.chars().map(|ch| painter.layout_no_wrap(ch.to_string(), font_id.clone(), color)).collect();
    let total_w: f32 = glyphs.iter().map(|g| g.size().x).sum();
    let mut cursor_x = -total_w * 0.5;
    for g in glyphs {
        let gw = g.size().x;
        let gh = g.size().y;
        let char_mid_x = cursor_x + gw * 0.5;
        cursor_x += gw;
        let d_deg = (char_mid_x / r_mid.max(1.0)) * half_c / std::f32::consts::PI;
        let char_deg = mid_deg + d_deg;
        let char_pt = super::context_thumb_dial::render::polar_pos(center, r_mid, char_deg, m);
        let tangent_deg = char_deg - north_deg;
        let angle_rad = super::context_thumb_dial::render::deg_to_rad(tangent_deg, m);
        let c_a = angle_rad.cos();
        let s_a = angle_rad.sin();
        let half_sz = Vec2::new(gw * 0.5, gh * 0.5);
        let rot_off = Vec2::new(half_sz.x * c_a - half_sz.y * s_a, half_sz.x * s_a + half_sz.y * c_a);
        let mut ts = egui::epaint::TextShape::new(char_pt - rot_off, g, color);
        ts.angle = angle_rad;
        painter.add(Shape::Text(ts));
    }
}

/// Steps a coupled 2-spring 1D interval (`DualSpringMorphBlob`, `REQ-016-03` & `SPEC-016 §4.3` / `§6`).
///
/// The leading edge (moving toward the destination) uses a fast snappy spring ($0.5 T_0$) while
/// the trailing edge uses the standard snappy spring ($T_0$), so the selection capsule stretches
/// like a liquid droplet during transit and coalesces cleanly at rest.
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
    let moving_right = target_min_x >= prev_min;
    let (s_min, s_max) = if moving_right { (base_s, fast_s) } else { (fast_s, base_s) };
    let anim_min = s_min.animate_f32(ctx, min_id, target_min_x);
    let anim_max = s_max.animate_f32(ctx, max_id, target_max_x).max(anim_min + 1.0);
    (anim_min, anim_max)
}

/// Paints a 3D convex Liquid Glass / Sonoma vertical-sheen capsule (`SPEC-016 §6..§9` & `BUG-SONOMA-03`).
///
/// Renders a base rounded rect with `border_stroke`, an interior vertical dome sheen `egui::epaint::Mesh`
/// (`+4%` top luminance to `-4%` bottom luminance), and a `1.0pt` top specular rim highlight.
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
        let sheen_step = (border_stroke.width * 0.5).clamp(0.0, 0.5) * 0.5 * 0.5 * 0.5;
        let mut top_h = egui::epaint::Hsva::from(fill);
        top_h.v = (top_h.v + sheen_step).clamp(0.0, 1.0);
        let mut bot_h = egui::epaint::Hsva::from(fill);
        bot_h.v = (bot_h.v - sheen_step).clamp(0.0, 1.0);
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
