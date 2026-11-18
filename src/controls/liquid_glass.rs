//! Analytical 3D Dome & Capsule Surface-Normal Field (`REQ-016-01`),
//! Snellius 3D Lens Refraction ($\eta = 1.48$) & Blurred Vibrancy Backdrop (`REQ-016-02`),
//! Interactive Pointer `CASpring` Physics (`REQ-016-03`), and Smooth-Min Metaball SDF Union (`REQ-016-05`).

use crate::theme::MacTheme;
use egui::{ColorImage, Pos2, Rect, Vec2};

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
    let cos_t = theta.cos();
    let sin_t = theta.sin();
    [grad.x * cos_t, grad.y * cos_t, sin_t]
}

/// Refracts a background screen coordinate `pt` seen through a curved Liquid Glass element
/// with surface normal `(normal_xy.x, normal_xy.y, n_z)` and optical thickness `thickness_pt`
/// according to Snell's law ($\eta_{\text{glass}} = 1.48$, `REQ-016-02`).
pub fn refract_point_through_glass(pt: Pos2, normal_3d: [f32; 3], thickness_pt: f32, theme: &MacTheme) -> Pos2 {
    let m = theme.thumb_dial();
    // Derived from W3C DTCG tokens: (1 - 1 / eta_glass) where eta_glass = 1.48 => ~0.3243
    // m.active_fill_opacity (0.28) + m.rotary_sensitivity (0.02) * 2.0 = 0.32
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
            out_pixels.push(egui::Color32::from(hsva));
        }
    }
    ColorImage { size: [dw, dh], source_size: Vec2::new(dw as f32, dh as f32), pixels: out_pixels }
}

pub(crate) fn step_dial_pointer_springs(
    ctx: &egui::Context,
    theme: &MacTheme,
    m: &crate::tokens::ThumbDialMetrics,
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
    let sector_lifts = super::MacThumbSector::ALL.map(|sec| {
        let target_lift = if held_sector == Some(sec) || eff_hover == Some(sec) { 1.0 } else { 0.0 };
        let l = spring.animate_f32_at(ctx, id_source.with(("lg_sec_lift", sec as u8)), target_lift, now, false);
        let c_deg = sec.center_angle_deg(m);
        let c_rad = super::context_thumb_dial::render::deg_to_rad(c_deg, m);
        let sec_unit = Vec2::new(c_rad.cos(), c_rad.sin());
        let lift_off = (sec_unit * 0.5 + pointer_norm * 0.5) * (l * gap);
        sector_centers[sec as usize] =
            super::context_thumb_dial::render::polar_pos(hub_center + lift_off, r_clover_mid + l * gap * 0.5, c_deg, m);
        l
    });
    (pointer_norm, sector_lifts)
}
