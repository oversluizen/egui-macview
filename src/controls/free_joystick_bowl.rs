//! Standalone AppKit "Liquid Glass" $360^\circ$ Free Joystick Bowl (`MacFreeJoystickBowl`, `SPEC-014 §2.6–§2.7`).
//!
//! Renders an unobstructed $360^\circ$ recessed 3D *Liquid Glass* bowl at the exact same compact
//! $1:1$ circular footprint as [`crate::controls::MacContextThumbDial`], featuring dotted
//! `Fine` and `Fast` speed rings, a **5-layer 3D translucent crystal glass marble**, curved
//! `↶ Undo` / `↷ Redo` crescent buttons, custom `.glass_accent(Color32)` adapted by the
//! Apple calculated vibrancy & contrast system, and [`MacTapticEvent`] micro-click telemetry.

use crate::controls::context_thumb_dial::icons::MacTapticEvent;
use crate::controls::context_thumb_dial::paint::{paint_undo_redo_crescent_arcs, UndoRedoPaintStyle};
use crate::controls::context_thumb_dial::render::{
    allocate_docked_thumb_rect, compute_undo_redo_hit_rects, hit_test_undo_redo_arcs, step_joystick_bowl_physics,
};
use crate::controls::{MacThumbArcPlacement, MacThumbDockZone};
use crate::theme::{MacAppearance, MacTheme};
use crate::tokens::ThumbDialMetrics;
use egui::{Align2, Color32, ColorImage, FontId, Id, Painter, Pos2, Rect, Response, Shape, Ui, Vec2};
use std::hash::{Hash, Hasher};

pub(crate) struct MarblePaintSpec {
    pub(crate) id_source: Id,
    pub(crate) hub_center: Pos2,
    pub(crate) marble_center: Pos2,
    pub(crate) r_outer: f32,
    pub(crate) r_bowl: f32,
    pub(crate) r_skirt_in: f32,
    pub(crate) r_skirt_out: f32,
    pub(crate) undo_arc_deg: (f32, f32),
    pub(crate) redo_arc_deg: (f32, f32),
    pub(crate) undo_active_weight: f32,
    pub(crate) redo_active_weight: f32,
    pub(crate) puck_r: f32,
    pub(crate) scale: f32,
    pub(crate) accent_col: Color32,
}

type MarbleTexGeom = (vitrea::GlamVec2, vitrea::GlamVec2, f32, f32, (f32, f32), ((f32, f32), (f32, f32), f32, f32), (f32, f32));

fn color_to_rgb_norm(c: Color32) -> [f32; 3] {
    let [r, g, b, _] = c.to_array();
    let max_u8 = f32::from(u8::MAX);
    [f32::from(r) / max_u8, f32::from(g) / max_u8, f32::from(b) / max_u8]
}

fn eval_bowl_backdrop_at(
    p: vitrea::GlamVec2,
    (hub_c, marble_c, r_bowl, puck_r): (vitrea::GlamVec2, vitrea::GlamVec2, f32, f32),
    (bowl_core, bowl_rim, accent_rgb, dot_rgb): ([f32; 3], [f32; 3], [f32; 3], [f32; 3]),
    blur_scale: f32,
    m: &ThumbDialMetrics,
) -> [f32; 3] {
    let d_vec = p - hub_c;
    let r = d_vec.length();
    let r_norm = (r / r_bowl.max(1.0)).clamp(0.0, 1.0);
    let rim_mix = m.hover_fill_opacity.points() * r_norm;
    let mut rgb = [0, 1, 2].map(|ch| bowl_core[ch] + (bowl_rim[ch] - bowl_core[ch]) * rim_mix);
    let fine_r = r_bowl * m.joystick_fine_ratio.points();
    let fast_r = r_bowl * m.joystick_fast_ratio.points();
    let slow_r = (fine_r + fast_r) * 0.5;
    let ring_d = (r - fine_r).abs().min((r - slow_r).abs()).min((r - fast_r).abs());
    let b_s = blur_scale.max(1.0);
    let bw = (m.active_stroke_width.points() * 2.0 * b_s).max(1.0);
    let grid_step = puck_r.max(2.0);
    let grid_d = (d_vec.x - (d_vec.x / grid_step).round() * grid_step).abs().min((d_vec.y - (d_vec.y / grid_step).round() * grid_step).abs()).min((r - r_bowl).abs());
    let grid_t = (1.0 - (grid_d / bw).clamp(0.0, 1.0)).powi(2) * (m.hover_fill_opacity.points() / (b_s * b_s));
    rgb = [0, 1, 2].map(|ch| rgb[ch] * (1.0 - grid_t) + dot_rgb[ch] * grid_t);
    if ring_d <= bw * 2.0 {
        let ang_deg = d_vec.y.atan2(d_vec.x) * m.half_circle_deg.points() / std::f32::consts::PI;
        let step_d = (m.detent_step_deg.points() * 0.5).max(1.0);
        let rem = (ang_deg.rem_euclid(step_d) - step_d * 0.5).abs();
        let arc_d = rem * (std::f32::consts::PI / m.half_circle_deg.points()) * r;
        let dot_w = (1.0 - (ring_d * ring_d + arc_d * arc_d).sqrt() / (bw * 2.0)).clamp(0.0, 1.0) * (m.active_fill_opacity.points() * 2.0 / b_s);
        rgb = [0, 1, 2].map(|ch| rgb[ch] * (1.0 - dot_w) + dot_rgb[ch] * dot_w);
    }
    let seg = marble_c - hub_c;
    let seg_len_sq = seg.length_squared();
    let t_seg = if seg_len_sq > 0.0 { ((p - hub_c).dot(seg) / seg_len_sq).clamp(0.0, 1.0) } else { 0.0 };
    let d_tether = (p - (hub_c + seg * t_seg)).length();
    let tether_w = (1.0 - d_tether / bw).clamp(0.0, 1.0) * (m.glass_fill_alpha.points() / b_s);
    let caus_off = vitrea::GlamVec2::splat(m.latch_dot_radius.points() * 2.0);
    let caus_t = (1.0 - (p - (marble_c + caus_off)).length() / (puck_r * 2.0).max(1.0)).clamp(0.0, 1.0);
    let glow_w = (tether_w + caus_t * caus_t * m.active_fill_opacity.points()).clamp(0.0, 1.0);
    [0, 1, 2].map(|ch| (rgb[ch] * (1.0 - glow_w) + accent_rgb[ch] * glow_w).clamp(0.0, 1.0))
}

fn paint_marble_bowl_region_rgba(
    flat: &mut [u8],
    tex_dim: usize,
    include_marble: bool,
    (hub_c, marble_c, r_outer, r_bowl, skirt_radii, (undo_deg, redo_deg, undo_act, redo_act), (puck_r, scale)): MarbleTexGeom,
    accent_col: Color32,
    theme: &MacTheme,
    m: &ThumbDialMetrics,
) {
    let is_light = theme.appearance == MacAppearance::Aqua;
    let bowl_core_col = theme.window_background();
    let bowl_core = color_to_rgb_norm(bowl_core_col);
    let bowl_rim = color_to_rgb_norm(if is_light { theme.box_background() } else { theme.control_background() });
    let passive_tint = color_to_rgb_norm(theme.specular_rim_top());
    let accent_rgb = color_to_rgb_norm(accent_col);
    let dot_rgb = color_to_rgb_norm(theme.secondary_label_on(bowl_core_col));
    let shadow_rgb = color_to_rgb_norm(theme.drop_shadow());
    let params = vitrea::LiquidGlassOpticalParams {
        ior_glass: m.ior_glass.points(),
        ior_dispersion_rgb: [m.ior_red.points(), m.ior_green.points(), m.ior_blue.points()],
        fresnel_f0: m.fresnel_f0.points(),
        bevel_width_pt: puck_r,
        dome_thickness_pt: puck_r * 0.5,
        vibrancy_chroma_boost: m.vibrancy_chroma_boost.points(),
        meniscus_merge_k_pt: m.meniscus_merge_k_pt.points() * scale,
        shadow_decay_length_pt: (puck_r * 0.5).max(2.0),
        glare_angle_rad: std::f32::consts::FRAC_PI_4,
        ..Default::default()
    };
    let stretch_gains = (m.viscous_stretch_gain.points(), m.viscous_press_gain.points());
    let vel_off = marble_c - hub_c;
    let press_amt = (vel_off.length() / (puck_r * 2.0).max(1.0)).clamp(0.0, 1.0);
    let dot_px = m.latch_dot_radius.points() * scale;
    let sh_off = vitrea::GlamVec2::new(-dot_px, dot_px * 2.0);
    let cursor_pt = Some(marble_c - vitrea::GlamVec2::splat(dot_px * 2.0));
    let (fill_a, act_a, hov_a, spec_a) = (m.glass_fill_alpha.points(), m.active_fill_opacity.points(), m.hover_fill_opacity.points(), m.glass_specular_alpha.points());
    let (max_u8, b_geom, b_cols) = (f32::from(u8::MAX), (hub_c, marble_c, r_bowl, puck_r), (bowl_core, bowl_rim, accent_rgb, dot_rgb));
    let patch_r = puck_r * 2.0 + dot_px * 2.0;
    let (x_range, y_range) = if include_marble {
        let (min_pt, max_pt) = (hub_c.min(marble_c) - vitrea::GlamVec2::splat(patch_r), hub_c.max(marble_c) + vitrea::GlamVec2::splat(patch_r));
        ((min_pt.x.floor().max(0.0) as usize)..((max_pt.x.ceil() as usize).min(tex_dim)), (min_pt.y.floor().max(0.0) as usize)..((max_pt.y.ceil() as usize).min(tex_dim)))
    } else {
        (0..tex_dim, 0..tex_dim)
    };
    for y in y_range {
        for x in x_range.clone() {
            let p = vitrea::GlamVec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let r_hub_d = (p - hub_c).length();
            if r_hub_d > skirt_radii.1 + puck_r * 2.0 {
                continue;
            }
            if include_marble && (p - marble_c).length() > patch_r {
                let seg_len_sq = vel_off.length_squared();
                let t_seg = if seg_len_sq > 0.0 { ((p - hub_c).dot(vel_off) / seg_len_sq).clamp(0.0, 1.0) } else { 0.0 };
                if (p - (hub_c + vel_off * t_seg)).length() > dot_px * 2.0 {
                    continue;
                }
            }
            let (mut glass_s, mut is_marble, mut arc_act_w, mut d_sh) = if include_marble {
                let p_def = vitrea::deform_viscous_droplet_point(p, marble_c, -vel_off, puck_r * 2.0, press_amt * 0.5, stretch_gains);
                let d_vec = p_def - marble_c;
                let dist = d_vec.length();
                let grad = if dist > 0.0 { d_vec / dist } else { vitrea::GlamVec2::X };
                (vitrea::SdfSample2d { distance: dist - puck_r, gradient: grad }, true, 0.0f32, ((p_def - sh_off - marble_c).length() - puck_r).max(0.0))
            } else {
                (vitrea::SdfSample2d { distance: skirt_radii.1, gradient: vitrea::GlamVec2::X }, false, 0.0f32, skirt_radii.1)
            };
            if !include_marble || r_hub_d >= r_outer - puck_r * 0.5 {
                for (arc_d, a_w) in [(undo_deg, undo_act), (redo_deg, redo_act)] {
                    let s_arc = vitrea::sdf_curved_stadium_arc_sample(p, hub_c, skirt_radii, arc_d);
                    d_sh = d_sh.min(vitrea::sdf_curved_stadium_arc_sample(p - sh_off, hub_c, skirt_radii, arc_d).distance.max(0.0));
                    if s_arc.distance < glass_s.distance {
                        (glass_s, is_marble, arc_act_w) = (s_arc, false, a_w);
                    }
                }
            }
            let ao_crease = (1.0 - (glass_s.distance.max(0.0) / (m.active_stroke_width.points() * scale).max(1.0)).clamp(0.0, 1.0)).powi(2) * act_a;
            let sh_win = (1.0 - (d_sh / puck_r.max(1.0)).clamp(0.0, 1.0)).powi(2);
            let exp_sh = (vitrea::exponential_sdf_attenuation(d_sh, &params, act_a) * sh_win + ao_crease).clamp(0.0, fill_a);
            let caus_t = if include_marble {
                (1.0 - (((p - vitrea::GlamVec2::splat(dot_px * 2.0) - marble_c).length() - puck_r).max(0.0) / (dot_px * 2.0).max(1.0)).clamp(0.0, 1.0)).powi(2) * act_a
            } else {
                0.0
            };
            let dish_here = eval_bowl_backdrop_at(p, b_geom, b_cols, 1.0, m);
            let mut floor_rgb = [0, 1, 2].map(|ch| dish_here[ch] * (1.0 - exp_sh) + shadow_rgb[ch] * exp_sh);
            if caus_t > 0.0 {
                floor_rgb = [0, 1, 2].map(|ch| (floor_rgb[ch] * (1.0 - caus_t) + accent_rgb[ch] * caus_t).clamp(0.0, 1.0));
            }
            let outer_cov = ((1.0 - (r_hub_d - r_outer)) * 0.5).clamp(0.0, 1.0);
            let floor_a = if r_hub_d <= r_outer + 1.0 {
                let outer_grad = if r_hub_d > 0.0 { (p - hub_c) / r_hub_d } else { vitrea::GlamVec2::X };
                let (f_rim, g_rim) = vitrea::anisotropic_bipolar_rim_glare(vitrea::SdfSample2d { distance: r_hub_d - r_outer, gradient: outer_grad }, &params, None);
                let rim_glint = ((f_rim * hov_a + g_rim * spec_a) * outer_cov).clamp(0.0, 1.0);
                floor_rgb = [0, 1, 2].map(|ch| (floor_rgb[ch] + (passive_tint[ch] - floor_rgb[ch]) * rim_glint).clamp(0.0, 1.0));
                let rim_t = ((r_hub_d - r_bowl).max(0.0) / (r_outer - r_bowl).max(1.0)).clamp(0.0, 1.0);
                ((act_a + hov_a + rim_t * act_a + exp_sh + caus_t + rim_glint) * outer_cov).clamp(0.0, fill_a)
            } else {
                let d_ext_bowl = ((p - sh_off - hub_c).length() - r_outer).max(0.0);
                let ext_win = (1.0 - (d_ext_bowl / puck_r.max(1.0)).clamp(0.0, 1.0)).powi(2);
                let bowl_sh = (vitrea::exponential_sdf_attenuation(d_ext_bowl, &params, hov_a) * ext_win).clamp(0.0, hov_a);
                floor_rgb = shadow_rgb;
                bowl_sh.max(exp_sh.clamp(0.0, act_a))
            };
            let surf = vitrea::evaluate_glass_surface_sample(p, glass_s, &params, if is_marble { cursor_pt } else { None }, puck_r * 2.0);
            let idx = (y * tex_dim + x) * 4;
            if surf.coverage <= 0.0 {
                if floor_a > 0.0 {
                    flat[idx..idx + 4].copy_from_slice(&[(floor_rgb[0] * max_u8).round() as u8, (floor_rgb[1] * max_u8).round() as u8, (floor_rgb[2] * max_u8).round() as u8, (floor_a * max_u8).round() as u8]);
                }
                continue;
            }
            let ([p_r, p_g, p_b], _, _) = vitrea::refract_bevel_chromatic_rgb(p, glass_s, &params);
            let sharp_rgb = [eval_bowl_backdrop_at(p_r, b_geom, b_cols, 1.0, m)[0], eval_bowl_backdrop_at(p_g, b_geom, b_cols, 1.0, m)[1], eval_bowl_backdrop_at(p_b, b_geom, b_cols, 1.0, m)[2]];
            let blur_rgb = [eval_bowl_backdrop_at(p_r, b_geom, b_cols, 2.0, m)[0], eval_bowl_backdrop_at(p_g, b_geom, b_cols, 2.0, m)[1], eval_bowl_backdrop_at(p_b, b_geom, b_cols, 2.0, m)[2]];
            let (tint_rgb, tint_a, glare_g) = if is_marble {
                (accent_rgb, act_a, fill_a)
            } else {
                ([0, 1, 2].map(|ch| passive_tint[ch] * (1.0 - arc_act_w) + accent_rgb[ch] * arc_act_w), (hov_a * 0.5) * (1.0 - arc_act_w) + fill_a * arc_act_w, spec_a + act_a + act_a * arc_act_w)
            };
            let lch_rgba = vitrea::compose_dielectric_lch_pixel(sharp_rgb, blur_rgb, [tint_rgb[0], tint_rgb[1], tint_rgb[2], tint_a], &surf, &params, 1.0, glare_g);
            let s_bev = ((-glass_s.distance) / params.bevel_width_pt.max(1.0)).clamp(0.0, 1.0);
            let rim_crest = (1.0 - (s_bev / (act_a + hov_a)).clamp(0.0, 1.0)).powi(2);
            let spec_pin = if is_marble {
                (surf.key_illumination.powi(2) * surf.dome_elevation * hov_a + surf.fresnel_specular * spec_a * rim_crest).clamp(0.0, 1.0)
            } else {
                ((surf.fresnel_specular * spec_a + surf.bipolar_glare_factor * hov_a) * rim_crest).clamp(0.0, 1.0)
            };
            let g_rgb = [0, 1, 2].map(|ch| ((lch_rgba[ch] + (1.0 - lch_rgba[ch]) * spec_pin).clamp(0.0, 1.0) * surf.coverage + floor_rgb[ch] * (1.0 - surf.coverage)).clamp(0.0, 1.0));
            let out_a = (surf.coverage + (1.0 - surf.coverage) * floor_a).clamp(0.0, 1.0);
            flat[idx..idx + 4].copy_from_slice(&[(g_rgb[0] * max_u8).round() as u8, (g_rgb[1] * max_u8).round() as u8, (g_rgb[2] * max_u8).round() as u8, (out_a * max_u8).round() as u8]);
        }
    }
}

pub(crate) fn paint_3d_recessed_bowl(painter: &Painter, spec: MarblePaintSpec, theme: &MacTheme, m: &ThumbDialMetrics) {
    let (s, r_bowl, r_outer, bowl_core) = (spec.scale, spec.r_bowl, spec.r_outer, theme.window_background());

    let vel_off = spec.marble_center - spec.hub_center;
    let press_amt = (vel_off.length() / (spec.puck_r * 2.0).max(1.0)).clamp(0.0, 1.0);
    let probe_pt =
        super::liquid_glass::deform_viscous_droplet_point(spec.marble_center, spec.marble_center, vel_off, spec.puck_r * 2.0, press_amt);
    let probe_n = super::liquid_glass::dome_surface_normal_3d(probe_pt, spec.marble_center, spec.puck_r, spec.puck_r);
    let _ = super::liquid_glass::evaluate_vertex_cursor_fresnel_lighting(probe_pt, probe_n, Some(spec.marble_center), spec.puck_r * 2.0);

    let (px_scale, max_u8) = (2.0f32, f32::from(u8::MAX));
    let pad = m.bevel_width_pt.points() * s;
    let bowl_rect = Rect::from_center_size(spec.hub_center, Vec2::splat((spec.r_skirt_out + pad) * 2.0));
    let tex_dim = ((bowl_rect.width() * px_scale).round() as usize).max(2);
    let to_tex = |pt: Pos2| vitrea::GlamVec2::new((pt.x - bowl_rect.min.x) * px_scale, (pt.y - bowl_rect.min.y) * px_scale);
    let mut static_hasher = std::collections::hash_map::DefaultHasher::new();
    (tex_dim, theme.appearance == MacAppearance::Aqua, spec.accent_col.to_array()).hash(&mut static_hasher);
    (
        ((spec.undo_active_weight * max_u8).round() as i32),
        ((spec.redo_active_weight * max_u8).round() as i32),
        spec.undo_arc_deg.0.to_bits(),
    )
        .hash(&mut static_hasher);
    let static_hash = static_hasher.finish();
    let mut frame_hasher = static_hasher;
    (((vel_off.x * 2.0).round() as i32), ((vel_off.y * 2.0).round() as i32)).hash(&mut frame_hasher);
    let state_hash = frame_hasher.finish();

    let (base_cache_id, cache_id, ctx) =
        (spec.id_source.with("lg_marble_bowl_static_base"), spec.id_source.with("lg_marble_bowl_ray_tex"), painter.ctx());
    let tex_handle = match ctx.data(|d| d.get_temp::<(u64, egui::TextureHandle)>(cache_id)) {
        Some((prev_h, handle)) if prev_h == state_hash => handle,
        prev_opt => {
            let hub_tex = to_tex(spec.hub_center);
            let geom_base = (
                hub_tex,
                hub_tex,
                r_outer * px_scale,
                r_bowl * px_scale,
                (spec.r_skirt_in * px_scale, spec.r_skirt_out * px_scale),
                (spec.undo_arc_deg, spec.redo_arc_deg, spec.undo_active_weight, spec.redo_active_weight),
                (spec.puck_r * px_scale, s * px_scale),
            );
            let base_arc = match ctx.data(|d| d.get_temp::<(u64, std::sync::Arc<Vec<u8>>)>(base_cache_id)) {
                Some((bh, arc)) if bh == static_hash && arc.len() == tex_dim * tex_dim * 4 => arc,
                _ => {
                    let mut base = vec![0u8; tex_dim * tex_dim * 4];
                    paint_marble_bowl_region_rgba(&mut base, tex_dim, false, geom_base, spec.accent_col, theme, m);
                    let arc = std::sync::Arc::new(base);
                    ctx.data_mut(|d| d.insert_temp(base_cache_id, (static_hash, arc.clone())));
                    arc
                }
            };
            let mut flat = (*base_arc).clone();
            let mut geom_dyn = geom_base;
            geom_dyn.1 = to_tex(spec.marble_center);
            paint_marble_bowl_region_rgba(&mut flat, tex_dim, true, geom_dyn, spec.accent_col, theme, m);
            let img = ColorImage::from_rgba_unmultiplied([tex_dim, tex_dim], &flat);
            let handle = if let Some((_, mut existing)) = prev_opt {
                existing.set(img, egui::TextureOptions::LINEAR);
                existing
            } else {
                ctx.load_texture("vitrea_marble_lg", img, egui::TextureOptions::LINEAR)
            };
            ctx.data_mut(|d| d.insert_temp(cache_id, (state_hash, handle.clone())));
            handle
        }
    };
    let mut mesh = egui::epaint::Mesh::with_texture(tex_handle.id());
    mesh.add_rect_with_uv(
        bowl_rect,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::from(egui::epaint::Hsva::new(0.0, 0.0, 1.0, 1.0)),
    );
    painter.add(Shape::mesh(mesh));

    let fine_r = r_bowl * m.joystick_fine_ratio.points();
    let fast_r = r_bowl * m.joystick_fast_ratio.points();
    for label_spec in [(fine_r, "Fine"), (fast_r, "Fast")] {
        painter.text(
            spec.hub_center + Vec2::new(0.0, label_spec.0 - m.hint_vertical_offset.points() * s),
            Align2::CENTER_CENTER,
            label_spec.1,
            FontId::proportional(m.hint_font_size.points() * s),
            theme.secondary_label_on(bowl_core),
        );
    }
}

#[derive(Clone, Copy, Default)]
struct FreeJoystickMemory {
    was_pressed: bool,
    is_dragging_bowl: bool,
    joystick_offset: Vec2,
}

/// Interaction output returned by [`MacFreeJoystickBowl::show`].
#[derive(Debug, Clone)]
pub struct MacFreeJoystickResponse {
    /// Underlying `egui::Response` of the joystick bowl widget.
    pub response: Response,
    /// Continuous $360^\circ$ point-nudge vector (`(dx, dy)`) scaled quadratically from center (`Fine`) to rim (`Fast`).
    pub nudge_delta: Vec2,
    /// Current screen-space center of the 3D crystal glass marble.
    pub marble_center: Pos2,
    /// Screen-space center of the bowl hub.
    pub hub_center: Pos2,
    /// `true` if the curved `↶ Undo` crescent button was clicked.
    pub undo_clicked: bool,
    /// `true` if the curved `↷ Redo` crescent button was clicked.
    pub redo_clicked: bool,
    /// Screen-space hit rect of the curved `↶ Undo` button.
    pub undo_rect: Rect,
    /// Screen-space hit rect of the curved `↷ Redo` button.
    pub redo_rect: Rect,
    /// Effective Liquid Glass accent color resolved by the Apple calculated color system (`REQ-014-06`).
    pub effective_glass_accent: Color32,
    /// Semantic Haptic / Auditory Micro-Click event emitted on this frame (`REQ-014-07`).
    pub taptic_event: Option<MacTapticEvent>,
}

/// Standalone $360^\circ$ Liquid Glass Free Joystick Bowl with a 3D Crystal Glass Marble (`SPEC-014 §2.6–§2.7`).
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacFreeJoystickBowl, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::DarkAqua);
/// let resp = MacFreeJoystickBowl::new("standalone_joystick")
///     .glass_accent(theme.system_green())
///     .theme(&theme)
///     .show(ui);
/// # let _ = resp.nudge_delta;
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacFreeJoystickBowl<'a> {
    id_source: Id,
    arc_placement: MacThumbArcPlacement,
    can_undo: bool,
    can_redo: bool,
    dock_target: Option<(Rect, MacThumbDockZone)>,
    scale: f32,
    glass_accent: Option<Color32>,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacFreeJoystickBowl<'a> {
    /// Creates a new standalone $360^\circ$ Liquid Glass Free Joystick Bowl.
    pub fn new(id_source: impl Hash + std::fmt::Debug) -> Self {
        Self {
            id_source: Id::NULL.with(id_source),
            arc_placement: MacThumbArcPlacement::TopCrown,
            can_undo: true,
            can_redo: true,
            dock_target: None,
            scale: 1.0,
            glass_accent: None,
            theme: None,
        }
    }

    /// Overrides the Liquid Glass accent color (adapted through `MacTheme::resolve_glass_accent_on` in the Apple calculated color system, `REQ-014-06`).
    pub fn glass_accent(mut self, color: Color32) -> Self {
        self.glass_accent = Some(color);
        self
    }

    /// Configures the placement of the curved `↶ Undo` and `↷ Redo` crescent buttons (defaults to [`MacThumbArcPlacement::TopCrown`]).
    pub fn undo_arc_placement(mut self, placement: MacThumbArcPlacement) -> Self {
        self.arc_placement = placement;
        self
    }

    /// Enables or disables the curved `↶ Undo` button.
    pub fn can_undo(mut self, can_undo: bool) -> Self {
        self.can_undo = can_undo;
        self
    }

    /// Enables or disables the curved `↷ Redo` button.
    pub fn can_redo(mut self, can_redo: bool) -> Self {
        self.can_redo = can_redo;
        self
    }

    /// Docks the joystick bowl in the `BottomLeft` or `BottomRight` thumb zone of `canvas_rect`.
    pub fn dock_in_rect(mut self, canvas_rect: Rect, zone: MacThumbDockZone) -> Self {
        self.dock_target = Some((canvas_rect, zone));
        self
    }

    /// Scales the overall radial dimensions (`1.0` default).
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.5);
        self
    }

    /// Injects a [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the standalone $360^\circ$ Liquid Glass Free Joystick Bowl and processes pointer/touch gestures.
    pub fn show(self, ui: &mut Ui) -> MacFreeJoystickResponse {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let m = theme.thumb_dial();
        let s = self.scale;
        let accent_col = theme.resolve_glass_accent_on(self.glass_accent, theme.popover_background());

        let gap = m.ring_gap.points() * s;
        let r_outer = m.outer_radius.points() * s;
        let r_clover_out = r_outer - gap * 2.0;
        let r_skirt_in = m.skirt_inner_radius.points() * s;
        let r_skirt_out = m.skirt_outer_radius.points() * s;
        let pad = m.bevel_width_pt.points() * s;

        let desired_size = Vec2::splat((r_skirt_out + pad) * 2.0);
        let (rect, response) = allocate_docked_thumb_rect(ui, self.id_source, self.dock_target, desired_size, m, s);

        let hub_center = rect.center();
        let (arcs, undo_rect, redo_rect) = compute_undo_redo_hit_rects(hub_center, self.arc_placement, r_skirt_in, r_skirt_out, m);

        let mem_id = self.id_source.with("free_joystick_bowl_state");
        let mut mem = ui.ctx().data_mut(|d| d.get_temp::<FreeJoystickMemory>(mem_id).unwrap_or_default());

        let now = ui.input(|i| i.time);
        let pointer_pos =
            response.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.latest_pos())));
        let (hovered_undo, hovered_redo) = pointer_pos
            .map(|pos| hit_test_undo_redo_arcs(pos, hub_center, (r_outer, r_skirt_out), arcs, (undo_rect, redo_rect), m))
            .unwrap_or((false, false));

        let is_down = response.is_pointer_button_down_on();
        let mut nudge_delta = Vec2::ZERO;
        let mut undo_clicked = false;
        let mut redo_clicked = false;
        let mut taptic_event = None;

        if is_down {
            mem.was_pressed = true;
            if let Some(pos) = pointer_pos {
                if !hovered_undo && !hovered_redo {
                    mem.is_dragging_bowl = true;
                }
                if mem.is_dragging_bowl {
                    let delta = pos - hub_center;
                    let (nudge, tap) = step_joystick_bowl_physics(&mut mem.joystick_offset, delta, delta.length(), r_clover_out, m);
                    nudge_delta = nudge;
                    taptic_event = tap;
                }
            }
        } else if mem.was_pressed {
            if !mem.is_dragging_bowl {
                if hovered_undo && self.can_undo {
                    undo_clicked = true;
                    taptic_event = Some(MacTapticEvent::HistoryStep { is_redo: false });
                } else if hovered_redo && self.can_redo {
                    redo_clicked = true;
                    taptic_event = Some(MacTapticEvent::HistoryStep { is_redo: true });
                }
            }
            mem = FreeJoystickMemory::default();
        }
        ui.ctx().data_mut(|d| d.insert_temp(mem_id, mem));

        let base_spring = theme.tokens().macview.motion.swiftui_snappy.to_spring();
        let return_spring = crate::motion::CASpring::new(base_spring.response * 0.5, base_spring.damping_ratio);
        let sx_id = self.id_source.with("jb_ret_x");
        let sy_id = self.id_source.with("jb_ret_y");
        let animated_offset = if is_down && mem.is_dragging_bowl {
            ui.ctx().data_mut(|d| {
                d.insert_temp(sx_id, (mem.joystick_offset.x, mem.joystick_offset.x, now));
                d.insert_temp(sy_id, (mem.joystick_offset.y, mem.joystick_offset.y, now));
            });
            mem.joystick_offset
        } else {
            Vec2::new(
                return_spring.animate_f32_at(ui.ctx(), sx_id, 0.0, now, false),
                return_spring.animate_f32_at(ui.ctx(), sy_id, 0.0, now, false),
            )
        };

        let painter = ui.painter_at(rect.intersect(ui.clip_rect()));
        let marble_center = hub_center + animated_offset;
        let puck_r = m.joystick_puck_radius.points() * (s + s * 0.5);
        let (undo_act, redo_act) = (if hovered_undo && self.can_undo { 1.0 } else { 0.0 }, if hovered_redo && self.can_redo { 1.0 } else { 0.0 });
        paint_3d_recessed_bowl(
            &painter,
            MarblePaintSpec { id_source: self.id_source, hub_center, marble_center, r_outer, r_bowl: r_clover_out, r_skirt_in, r_skirt_out, undo_arc_deg: arcs.0, redo_arc_deg: arcs.1, undo_active_weight: undo_act, redo_active_weight: redo_act, puck_r, scale: s, accent_col },
            theme,
            m,
        );
        paint_undo_redo_crescent_arcs(&painter, theme, m, (hub_center, r_skirt_in, r_skirt_out, s), arcs, UndoRedoPaintStyle { can_undo: self.can_undo, can_redo: self.can_redo, hovered_undo, hovered_redo, accent_col });
        MacFreeJoystickResponse { response, nudge_delta, marble_center, hub_center, undo_clicked, redo_clicked, undo_rect, redo_rect, effective_glass_accent: accent_col, taptic_event }
    }
}
