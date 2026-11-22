//! 3D Cushion, Circular Tactile Ring, and Transient HUD Tooltip Painter for [`super::MacContextThumbDial`].

use super::icons::{paint_sector_icon, RosetteContainerSpec};
use super::render::{polar_pos, resolve_undo_redo_arcs, UndoRedoArcs};
use super::{MacContextThumbDial, MacThumbDialLayer, MacThumbSector};
use crate::controls::free_joystick_bowl::{paint_3d_recessed_bowl, MarblePaintSpec};
use crate::theme::{MacBackgroundStyle, MacTheme};
use crate::tokens::ThumbDialMetrics;
use egui::{Align2, Color32, ColorImage, FontId, Painter, Pos2, Rect, Shape, Stroke, Vec2};

pub(crate) type DishBackdropPalette = ([f32; 3], [f32; 3], [f32; 3], [f32; 3], [f32; 3]);

pub(crate) fn eval_shared_dish_matrix_at(
    p: vitrea::GlamVec2,
    sc: &RosetteContainerSpec,
    (dish_core, dish_rim, accent_rgb, grid_rgb, warm_rgb): DishBackdropPalette,
    blur_scale: f32,
    m: &ThumbDialMetrics,
) -> [f32; 3] {
    let d_vec = p - sc.hub_center;
    let r = d_vec.length();
    let r_norm = (r / sc.r_dish.max(1.0)).clamp(0.0, 1.0);
    let rim_mix = r_norm * m.hover_fill_opacity.points();
    let mut rgb = [0, 1, 2].map(|ch| dish_core[ch] + (dish_rim[ch] - dish_core[ch]) * rim_mix);
    // Chromatic ambient backdrop band passing through the mid-radius of the left-docked glass bowl (REQ-016-A0.3):
    let west_t = ((-d_vec.x) / sc.r_dish.max(1.0)).clamp(0.0, 1.0);
    let band_r = (1.0 - ((r_norm - 0.5).abs() * 2.0).clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let warm_w = (west_t * band_r * (m.active_fill_opacity.points() + m.hover_fill_opacity.points())).clamp(0.0, 1.0);
    rgb = [0, 1, 2].map(|ch| rgb[ch] * (1.0 - warm_w) + warm_rgb[ch] * warm_w);

    // Energy-preserving Gaussian spread for B_sharp (blur_scale = 1.0) vs B_blur (blur_scale > 1.0) (REQ-016-B1.10b):
    let b_s = blur_scale.max(1.0);
    let grid_step = sc.r_hub.max(2.0);
    let grid_d = (d_vec.x - (d_vec.x / grid_step).round() * grid_step)
        .abs()
        .min((d_vec.y - (d_vec.y / grid_step).round() * grid_step).abs());
    let (r_fine, r_fast) = (sc.r_dish * m.joystick_fine_ratio.points(), sc.r_dish * m.joystick_fast_ratio.points());
    let ring_d = (r - r_fine).abs().min((r - r_fast).abs()).min((r - sc.r_dish).abs()).min((r - sc.r_outer).abs());
    let line_w = (m.active_stroke_width.points() * sc.scale * b_s).max(1.0);
    let line_t = (1.0 - (grid_d.min(ring_d) / line_w).clamp(0.0, 1.0)).powi(2);
    let line_w_mix = (line_t * (m.hover_fill_opacity.points() / (b_s * b_s))).clamp(0.0, 1.0);
    rgb = [0, 1, 2].map(|ch| rgb[ch] * (1.0 - line_w_mix) + grid_rgb[ch] * line_w_mix);

    let (boost, bleed_gain) = (m.vibrancy_chroma_boost.points(), m.glass_fill_alpha.points());
    for mem in &sc.members {
        let w = (mem.active_weight + mem.lift_weight * 0.5).clamp(0.0, 1.0);
        if w > 0.0 {
            let r_mid = (mem.radii.0 + mem.radii.1) * 0.5;
            let rad = super::render::deg_to_rad(mem.angles_deg.0, m);
            let c_pos = mem.center + vitrea::GlamVec2::new(rad.cos() * r_mid, rad.sin() * r_mid);
            let t = (1.0 - (p - c_pos).length() / ((mem.radii.1 - mem.radii.0) * boost).max(1.0)).clamp(0.0, 1.0);
            let glow = t * t * w * bleed_gain;
            rgb = [0, 1, 2].map(|ch| (rgb[ch] * (1.0 - glow) + accent_rgb[ch] * glow).clamp(0.0, 1.0));
        }
    }
    rgb
}

pub(crate) fn paint_raytraced_rosette_quad(
    painter: &Painter,
    id_source: egui::Id,
    dial_rect: Rect,
    sc: &RosetteContainerSpec,
    tex_dim: usize,
    theme: &MacTheme,
    m: &ThumbDialMetrics,
) {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let max_u8 = f32::from(u8::MAX);
    tex_dim.hash(&mut hasher);
    (theme.appearance == crate::theme::MacAppearance::Aqua).hash(&mut hasher);
    sc.accent_col.to_array().hash(&mut hasher);
    ((sc.undo_active_weight * max_u8).round() as i32).hash(&mut hasher);
    ((sc.redo_active_weight * max_u8).round() as i32).hash(&mut hasher);
    sc.undo_arc_deg.0.to_bits().hash(&mut hasher);
    if let Some(cp) = sc.cursor_pos {
        let q = (m.bevel_width_pt.points() * sc.scale).max(1.0);
        ((cp.x / q).round() as i32).hash(&mut hasher);
        ((cp.y / q).round() as i32).hash(&mut hasher);
    }
    for c in &sc.members {
        let q = (m.active_stroke_width.points() * sc.scale).max(1.0);
        ((c.center.x / q).round() as i32).hash(&mut hasher);
        ((c.center.y / q).round() as i32).hash(&mut hasher);
        ((c.radii.1 / q).round() as i32).hash(&mut hasher);
        ((c.active_weight * m.bevel_width_pt.points()).round() as i32).hash(&mut hasher);
        ((c.lift_weight * m.bevel_width_pt.points()).round() as i32).hash(&mut hasher);
    }
    let state_hash = hasher.finish();
    let cache_id = id_source.with("lg_rosette_ray_tex");
    let ctx = painter.ctx();
    let cached = ctx.data(|d| d.get_temp::<(u64, egui::TextureHandle)>(cache_id));
    let tex_handle = match cached {
        Some((prev_h, handle)) if prev_h == state_hash => handle,
        prev_opt => {
            let flat = super::icons::synthesize_rosette_matrix_rgba(tex_dim, sc, theme, m);
            let img = ColorImage::from_rgba_unmultiplied([tex_dim, tex_dim], &flat);
            let handle = if let Some((_, mut existing)) = prev_opt {
                existing.set(img, egui::TextureOptions::LINEAR);
                existing
            } else {
                ctx.load_texture("vitrea_rosette_lg", img, egui::TextureOptions::LINEAR)
            };
            ctx.data_mut(|d| d.insert_temp(cache_id, (state_hash, handle.clone())));
            handle
        }
    };
    let mut mesh = egui::epaint::Mesh::with_texture(tex_handle.id());
    let tint = Color32::from(egui::epaint::Hsva::new(0.0, 0.0, 1.0, 1.0));
    mesh.add_rect_with_uv(dial_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), tint);
    painter.add(Shape::mesh(mesh));
}

pub(crate) struct UndoRedoPaintStyle {
    pub(crate) can_undo: bool,
    pub(crate) can_redo: bool,
    pub(crate) hovered_undo: bool,
    pub(crate) hovered_redo: bool,
    pub(crate) accent_col: Color32,
}

pub(super) struct DialPaintState {
    pub(super) hub_center: Pos2,
    pub(super) active_layer: MacThumbDialLayer,
    pub(super) latched_sector: Option<MacThumbSector>,
    pub(super) held_sector: Option<MacThumbSector>,
    pub(super) hovered_sector: Option<MacThumbSector>,
    pub(super) hovered_rotary: bool,
    pub(super) hovered_undo: bool,
    pub(super) hovered_redo: bool,
    pub(super) joystick_offset: Vec2,
    pub(super) pointer_norm: Vec2,
    pub(super) sector_lifts: [f32; 6],
    pub(super) polar_morph: Option<(f32, f32)>,
    pub(super) accent_col: Color32,
    pub(super) tooltip_alpha: f32,
    pub(super) tooltip_str: Option<String>,
}

pub(super) fn paint_dial_layers(
    painter: &Painter,
    theme: &MacTheme,
    m: &ThumbDialMetrics,
    dial: &MacContextThumbDial<'_>,
    st: DialPaintState,
) {
    let s = dial.scale;
    let hub_center = st.hub_center;
    let r_hub = m.hub_radius.points() * s;
    let gap = m.ring_gap.points() * s;
    let r_clover_in = r_hub + gap * 2.0;
    let r_outer = m.outer_radius.points() * s;
    let r_clover_out = r_outer - gap * 2.0;
    let r_skirt_in = m.skirt_inner_radius.points() * s;
    let r_skirt_out = m.skirt_outer_radius.points() * s;
    let bw = m.border_stroke_width.points();
    let specular_col = theme.specular_rim_top();
    let arcs = resolve_undo_redo_arcs(dial.arc_placement, m);
    let undo_act = if st.hovered_undo && dial.can_undo { 1.0 } else { 0.0 };
    let redo_act = if st.hovered_redo && dial.can_redo { 1.0 } else { 0.0 };

    if st.active_layer == MacThumbDialLayer::FreeJoystickBowl {
        paint_3d_recessed_bowl(
            painter,
            MarblePaintSpec {
                id_source: dial.id_source,
                hub_center,
                marble_center: hub_center + st.joystick_offset,
                r_outer,
                r_bowl: r_clover_out,
                r_skirt_in,
                r_skirt_out,
                undo_arc_deg: arcs.0,
                redo_arc_deg: arcs.1,
                undo_active_weight: undo_act,
                redo_active_weight: redo_act,
                puck_r: m.joystick_puck_radius.points() * s,
                scale: s,
                accent_col: st.accent_col,
            },
            theme,
            m,
        );
    } else {
        let _ = st.hovered_rotary;
        let half_gap_deg = m.sector_gap_deg.points() * 0.5;
        let r_clover_mid = (r_clover_in + r_clover_out) * 0.5;
        let pad = m.bevel_width_pt.points() * s;
        let dial_rect = Rect::from_center_size(hub_center, Vec2::splat((r_skirt_out + pad) * 2.0));
        let px_scale = 2.0f32;
        let tex_dim = ((dial_rect.width() * px_scale).round() as usize).max(2);
        let to_tex = |pt: Pos2| -> vitrea::GlamVec2 {
            vitrea::GlamVec2::new((pt.x - dial_rect.min.x) * px_scale, (pt.y - dial_rect.min.y) * px_scale)
        };

        let passive_half_sweep = m.diagonal_sweep_deg.points() * 0.5 - half_gap_deg;
        let fillet_px = gap * 2.0 * px_scale;
        // REQ-016-B1.8: Only the 6 active sectors (N, NE, E, SE, S, W) receive 3D Liquid Glass cushions;
        // NW (-135 deg) and SW (+135 deg) remain intentionally open recessed glass bowl pockets!
        let mut members = Vec::with_capacity(7);
        let lift_gain = gap + gap * 0.5;
        for sector in MacThumbSector::ALL {
            let is_latched = st.latched_sector == Some(sector);
            let is_held = st.held_sector == Some(sector);
            let is_hovered = st.hovered_sector == Some(sector);
            let lift = st.sector_lifts[sector as usize].clamp(0.0, 1.0);
            let c_deg = sector.center_angle_deg(m);
            let c_rad = super::render::deg_to_rad(c_deg, m);
            let sec_unit = Vec2::new(c_rad.cos(), c_rad.sin());
            let lift_off = (sec_unit * 0.5 + st.pointer_norm * 0.5) * (lift * lift_gain);
            let r_out_lifted = r_clover_out + lift * gap;
            let half_sweep = sector.sweep_deg(m) * 0.5 - half_gap_deg;
            let vel_px = vitrea::GlamVec2::new(lift_off.x * px_scale, lift_off.y * px_scale) * 2.0;
            let hover_boost = if is_hovered { m.hover_fill_opacity.points() } else { 0.0 };
            members.push(super::icons::RosetteMemberSpec {
                center: to_tex(hub_center + lift_off),
                radii: (r_clover_in * px_scale, r_out_lifted * px_scale),
                angles_deg: (c_deg, half_sweep),
                corner_radius: fillet_px,
                active_weight: if is_held || is_latched { 1.0 } else { hover_boost },
                lift_weight: lift,
                deform_vel: vel_px,
            });
        }
        if let Some((lead_deg, trail_deg)) = st.polar_morph {
            let span_diff = (lead_deg - trail_deg).abs();
            if span_diff > half_gap_deg {
                let mid_d = (lead_deg + trail_deg) * 0.5;
                let half_s = span_diff * 0.5 + passive_half_sweep * 0.5;
                members.push(super::icons::RosetteMemberSpec {
                    center: to_tex(hub_center),
                    radii: (r_clover_in * px_scale, r_clover_out * px_scale),
                    angles_deg: (mid_d, half_s),
                    corner_radius: fillet_px,
                    active_weight: 0.5,
                    lift_weight: 0.5,
                    deform_vel: vitrea::GlamVec2::ZERO,
                });
            }
        }
        let cursor_tex = (st.pointer_norm.length_sq() > 0.0).then(|| to_tex(hub_center + st.pointer_norm * r_outer));
        let container = RosetteContainerSpec {
            hub_center: to_tex(hub_center),
            r_outer: r_outer * px_scale,
            r_dish: r_clover_out * px_scale,
            r_hub: r_hub * px_scale,
            r_skirt_in: r_skirt_in * px_scale,
            r_skirt_out: r_skirt_out * px_scale,
            undo_arc_deg: arcs.0,
            redo_arc_deg: arcs.1,
            undo_active_weight: undo_act,
            redo_active_weight: redo_act,
            scale: s * px_scale,
            cursor_pos: cursor_tex,
            accent_col: st.accent_col,
            members,
        };
        paint_raytraced_rosette_quad(painter, dial.id_source, dial_rect, &container, tex_dim, theme, m);

        for sector in MacThumbSector::ALL {
            let is_latched = st.latched_sector == Some(sector);
            let is_held = st.held_sector == Some(sector);
            let lift = st.sector_lifts[sector as usize].clamp(0.0, 1.0);
            let c_deg = sector.center_angle_deg(m);
            let c_rad = super::render::deg_to_rad(c_deg, m);
            let sec_unit = Vec2::new(c_rad.cos(), c_rad.sin());
            let lift_off = (sec_unit * 0.5 + st.pointer_norm * 0.5) * (lift * lift_gain);
            let r_out_lifted = r_clover_out + lift * gap;
            let half_sweep = sector.sweep_deg(m) * 0.5 - half_gap_deg;
            let spec = dial.sectors.sector(sector);
            let parallax = m.active_stroke_width.points() / bw.max(1.0);
            let center_pt = polar_pos(hub_center + lift_off * parallax, r_clover_mid + lift * gap * 0.5, c_deg, m);
            let core_d = super::super::liquid_glass::sdf_annular_rounded_sector(
                center_pt,
                hub_center + lift_off,
                (r_clover_in, r_out_lifted),
                (c_deg, half_sweep),
                gap * 2.0,
                m,
            );
            let _core_n = super::super::liquid_glass::dome_surface_normal_3d_from_sample(
                vitrea::SdfSample2d { distance: core_d, gradient: vitrea::SdfSample2d { distance: 0.0, gradient: Default::default() }.gradient },
                m.bevel_width_pt.points(),
            );
            let fg_col = if is_held || is_latched {
                MacBackgroundStyle::from_color(st.accent_col).primary_text_color(theme)
            } else {
                theme.label()
            };
            let hint_col = if is_held || is_latched {
                fg_col.gamma_multiply(m.glass_fill_alpha.points())
            } else {
                theme.secondary_label()
            };
            let icon_y_off = (m.hint_vertical_offset.points() + m.label_vertical_offset.points() * 0.5) * s;
            let lbl_y_off = m.border_stroke_width.points() * s;
            let hint_y_off = (m.hint_vertical_offset.points() + m.label_vertical_offset.points()) * s;
            let icon_pt = center_pt - Vec2::new(0.0, icon_y_off);
            let icon_sz = Vec2::splat(m.hint_font_size.points() * s);
            paint_sector_icon(painter, spec.icon, Rect::from_center_size(icon_pt, icon_sz), fg_col, m);
            painter.text(
                center_pt + Vec2::new(0.0, lbl_y_off),
                Align2::CENTER_CENTER,
                spec.label,
                FontId::proportional(m.hint_font_size.points() * s),
                fg_col,
            );
            painter.text(
                center_pt + Vec2::new(0.0, hint_y_off),
                Align2::CENTER_CENTER,
                spec.shortcut_hint,
                FontId::proportional(m.hub_title_font_size.points() * s),
                hint_col,
            );
            if is_latched {
                let led_deg = c_deg - half_sweep * 0.5;
                let dot_pos = polar_pos(hub_center + lift_off, r_clover_in + m.latch_dot_inset.points() * s, led_deg, m);
                let dot_r = m.latch_dot_radius.points() * s;
                painter.circle_filled(dot_pos, dot_r * 2.0, theme.system_orange().gamma_multiply(m.hover_fill_opacity.points()));
                painter.circle_filled(dot_pos, dot_r * 1.0 + bw, theme.system_orange().gamma_multiply(m.active_fill_opacity.points()));
                painter.circle_filled(dot_pos, dot_r, theme.system_orange());
                painter.circle_filled(dot_pos - Vec2::splat(dot_r * 0.5), dot_r * 0.5, specular_col);
            }
        }
        let ch_arm = m.latch_dot_inset.points() * s;
        let ch_stroke = Stroke::new(bw, theme.secondary_label());
        let hub_n = super::super::liquid_glass::dome_surface_normal_3d(hub_center + st.pointer_norm * bw, hub_center, r_hub, r_hub);
        let refr_hub = super::super::liquid_glass::refract_point_through_glass(hub_center, hub_n, bw, theme);
        painter.circle_stroke(refr_hub, ch_arm * 0.5, ch_stroke);
        painter.line_segment([refr_hub - Vec2::new(ch_arm, 0.0), refr_hub + Vec2::new(ch_arm, 0.0)], ch_stroke);
        painter.line_segment([refr_hub - Vec2::new(0.0, ch_arm), refr_hub + Vec2::new(0.0, ch_arm)], ch_stroke);
    }

    paint_undo_redo_crescent_arcs(
        painter,
        theme,
        m,
        (hub_center, r_skirt_in, r_skirt_out, s),
        arcs,
        UndoRedoPaintStyle {
            can_undo: dial.can_undo,
            can_redo: dial.can_redo,
            hovered_undo: st.hovered_undo,
            hovered_redo: st.hovered_redo,
            accent_col: st.accent_col,
        },
    );

    if let Some(tip) = st.tooltip_str.as_deref().filter(|_| st.tooltip_alpha > 0.0) {
        let a = st.tooltip_alpha;
        let text_col = theme.label().gamma_multiply(a);
        let galley = painter.layout_no_wrap(tip.to_owned(), FontId::proportional(m.label_font_size.points() * s), text_col);
        let pill_sz = galley.size() + Vec2::new(gap * 2.0, gap) * 2.0;
        let pill_rect = Rect::from_center_size(Pos2::new(hub_center.x, hub_center.y - r_skirt_out + pill_sz.y * 0.5 + gap), pill_sz);
        let cr = egui::CornerRadius::same((pill_sz.y * 0.5) as u8);
        painter.rect_filled(pill_rect.translate(Vec2::new(0.0, bw * 2.0)), cr, theme.drop_shadow().gamma_multiply(a));
        painter.rect(
            pill_rect,
            cr,
            theme.popover_background().gamma_multiply(a),
            Stroke::new(bw, specular_col.gamma_multiply(a)),
            egui::StrokeKind::Inside,
        );
        painter.galley(pill_rect.center() - galley.size() * 0.5, galley, text_col);
    }
}

pub(crate) fn paint_undo_redo_crescent_arcs(
    painter: &Painter,
    theme: &MacTheme,
    m: &ThumbDialMetrics,
    (hub_center, r_skirt_in, r_skirt_out, s): (Pos2, f32, f32, f32),
    ((undo_start, undo_end), (redo_start, redo_end)): UndoRedoArcs,
    style: UndoRedoPaintStyle,
) {
    let _ = (style.hovered_undo, style.hovered_redo, style.accent_col);
    let r_skirt_mid = (r_skirt_in + r_skirt_out) * 0.5;
    for (start_d, end_d, sym, word, enabled) in [
        (undo_start, undo_end, crate::symbols::MacSymbol::ArrowCounterClockwise, "Undo", style.can_undo),
        (redo_start, redo_end, crate::symbols::MacSymbol::ArrowClockwise, "Redo", style.can_redo),
    ] {
        let mid_deg = (start_d + end_d) * 0.5;
        let fg = if enabled { theme.label() } else { theme.tertiary_label() };
        let caption = format!("{} {word}", sym.as_str());
        super::super::liquid_glass::paint_text_along_arc(
            painter,
            hub_center,
            r_skirt_mid,
            mid_deg,
            &caption,
            FontId::proportional(m.hint_font_size.points() * s),
            fg,
            m,
        );
    }
}
