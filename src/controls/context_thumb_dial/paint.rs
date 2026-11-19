//! 3D Cushion, Circular Tactile Ring, and Transient HUD Tooltip Painter for [`super::MacContextThumbDial`].

use super::icons::paint_sector_icon;
use super::render::{polar_pos, resolve_undo_redo_arcs, UndoRedoArcs};
use super::{MacContextThumbDial, MacThumbDialLayer, MacThumbSector};
use crate::controls::free_joystick_bowl::{paint_3d_recessed_bowl, MarblePaintSpec};
use crate::theme::{MacBackgroundStyle, MacTheme};
use crate::tokens::ThumbDialMetrics;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};

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
    let is_light = theme.appearance == crate::theme::MacAppearance::Aqua;

    if st.active_layer == MacThumbDialLayer::FreeJoystickBowl {
        let border_stroke = Stroke::new(bw, theme.separator());
        let glass_bg = if is_light {
            theme.box_background()
        } else {
            theme.box_background().gamma_multiply(m.glass_fill_alpha.points())
        };
        let outer_rim_col = if st.hovered_rotary { st.accent_col } else { specular_col };
        painter.circle_filled(hub_center + Vec2::new(0.0, gap * 2.0), r_outer + bw * 2.0, theme.drop_shadow());
        painter.circle_filled(hub_center + Vec2::new(0.0, gap), r_outer + bw, theme.drop_shadow());
        painter.circle(hub_center, r_outer, glass_bg, border_stroke);
        painter.circle_stroke(hub_center, r_outer - bw, Stroke::new(m.active_stroke_width.points(), outer_rim_col));
        paint_3d_recessed_bowl(
            painter,
            MarblePaintSpec {
                hub_center,
                marble_center: hub_center + st.joystick_offset,
                puck_r: m.joystick_puck_radius.points() * s,
                scale: s,
                accent_col: st.accent_col,
            },
            r_clover_out,
            theme,
            m,
        );
    } else {
        let border_stroke = Stroke::new(bw, theme.separator());
        let glass_bg = if is_light {
            theme.box_background()
        } else {
            theme.box_background().gamma_multiply(m.glass_fill_alpha.points())
        };
        let outer_rim_col = if st.hovered_rotary { st.accent_col } else { specular_col };
        painter.circle_filled(hub_center + Vec2::new(0.0, gap * 2.0), r_outer + bw * 2.0, theme.drop_shadow());
        painter.circle_filled(hub_center + Vec2::new(0.0, gap), r_outer + bw, theme.drop_shadow());
        painter.circle(hub_center, r_outer, glass_bg, border_stroke);
        painter.circle_stroke(hub_center, r_outer - bw, Stroke::new(m.active_stroke_width.points(), outer_rim_col));

        let bowl_core = if is_light {
            theme.window_background()
        } else {
            theme.window_background().gamma_multiply(m.glass_fill_alpha.points())
        };
        let bowl_rim = if is_light {
            theme.box_background()
        } else {
            theme.control_background().gamma_multiply(m.glass_fill_alpha.points())
        };
        painter.circle_filled(hub_center, r_clover_out + bw, theme.drop_shadow().gamma_multiply(0.5));
        crate::controls::free_joystick_bowl::paint_radial_disc_mesh(painter, hub_center, r_clover_out, bowl_core, bowl_rim, m);
        painter.circle_stroke(hub_center, r_clover_out, border_stroke);
        painter.circle_stroke(
            hub_center - Vec2::new(0.0, bw),
            r_clover_out - bw,
            Stroke::new(m.active_stroke_width.points(), theme.drop_shadow().gamma_multiply(0.5)),
        );
        painter.circle_stroke(hub_center + Vec2::new(0.0, bw), r_clover_out - bw, Stroke::new(bw, specular_col));

        let half_gap_deg = m.sector_gap_deg.points() * 0.5;
        let r_clover_mid = (r_clover_in + r_clover_out) * 0.5;
        let pad = m.bevel_width_pt.points() * s;
        let dial_rect = Rect::from_center_size(hub_center, Vec2::splat((r_outer + pad) * 2.0));
        let px_scale = 2.0f32;
        let tex_dim = ((dial_rect.width() * px_scale).round() as usize).max(2);
        let to_tex = |pt: Pos2| -> vitrea::GlamVec2 {
            vitrea::GlamVec2::new((pt.x - dial_rect.min.x) * px_scale, (pt.y - dial_rect.min.y) * px_scale)
        };

        for i in 0..MacThumbSector::ALL.len() {
            let s_a = MacThumbSector::ALL[i];
            let s_b = MacThumbSector::ALL[(i + 1) % MacThumbSector::ALL.len()];
            let l_a = st.sector_lifts[s_a as usize];
            let l_b = st.sector_lifts[s_b as usize];
            super::super::liquid_glass::paint_meniscus_smooth_union_bridge(
                painter,
                hub_center,
                (r_clover_in, r_clover_out, gap * 2.0, gap * 2.0),
                (s_a.center_angle_deg(m), s_a.sweep_deg(m) * 0.5 - half_gap_deg, l_a),
                (s_b.center_angle_deg(m), s_b.sweep_deg(m) * 0.5 - half_gap_deg, l_b),
                st.accent_col,
                m,
            );
        }

        let passive_half_sweep = m.diagonal_sweep_deg.points() * 0.5 - half_gap_deg;
        let fillet_px = gap * 2.0 * px_scale;
        let mut cushions = Vec::with_capacity(8);
        for passive_deg in [m.angle_north_west_deg.points(), m.angle_south_west_deg.points()] {
            cushions.push(vitrea::RosetteCushionSpec {
                center: to_tex(hub_center),
                radii: (r_clover_in * px_scale, r_clover_out * px_scale),
                angles_deg: (passive_deg, passive_half_sweep),
                corner_radius: fillet_px,
                active_weight: 0.0,
                lift_weight: 0.0,
                deform_vel: vitrea::GlamVec2::ZERO,
            });
        }
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
            cushions.push(vitrea::RosetteCushionSpec {
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
                cushions.push(vitrea::RosetteCushionSpec {
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
        let acc = st.accent_col.to_array();
        let max_u8 = f32::from(u8::MAX);
        let cursor_tex = (st.pointer_norm.length_sq() > 0.0).then(|| to_tex(hub_center + st.pointer_norm * r_outer));
        let scene = vitrea::RosetteOpticalScene {
            hub_center: to_tex(hub_center),
            r_outer: r_outer * px_scale,
            r_dish: r_clover_out * px_scale,
            r_hub: r_hub * px_scale,
            bevel_width: m.bevel_width_pt.points() * s * px_scale,
            dome_thickness: m.dome_thickness_pt.points() * s * px_scale,
            meniscus_k: m.meniscus_merge_k_pt.points() * s * px_scale,
            cursor_pos: cursor_tex,
            is_light_mode: is_light,
            accent_rgb: [f32::from(acc[0]) / max_u8, f32::from(acc[1]) / max_u8, f32::from(acc[2]) / max_u8],
            cushions,
        };
        super::super::liquid_glass::paint_raytraced_rosette_quad(painter, dial.id_source, dial_rect, &scene, tex_dim);

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
        resolve_undo_redo_arcs(dial.arc_placement, m),
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
    let is_light = theme.appearance == crate::theme::MacAppearance::Aqua;
    let specular_col = theme.specular_rim_top();
    let r_skirt_mid = (r_skirt_in + r_skirt_out) * 0.5;
    for (start_d, end_d, sym, word, enabled, hovered) in [
        (undo_start, undo_end, crate::symbols::MacSymbol::ArrowCounterClockwise, "Undo", style.can_undo, style.hovered_undo),
        (redo_start, redo_end, crate::symbols::MacSymbol::ArrowClockwise, "Redo", style.can_redo, style.hovered_redo),
    ] {
        let fill = if hovered && enabled {
            style.accent_col.gamma_multiply(m.active_fill_opacity.points())
        } else if is_light {
            theme.window_background()
        } else {
            theme.control_background().gamma_multiply(m.glass_fill_alpha.points())
        };
        let stroke_col = if hovered && enabled { style.accent_col } else { specular_col };
        super::super::liquid_glass::paint_curved_stadium_capsule(
            painter,
            hub_center,
            (r_skirt_in, r_skirt_out),
            (start_d, end_d),
            (fill, stroke_col, theme.drop_shadow()),
            m,
        );
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
