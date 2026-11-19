//! 3D Cushion, Circular Tactile Ring, and Transient HUD Tooltip Painter for [`super::MacContextThumbDial`].

use super::icons::{paint_annular_wedge_lit, paint_sector_icon, WedgeGeometry};
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

    let border_stroke = Stroke::new(bw, theme.separator());
    let active_stroke = Stroke::new(m.active_stroke_width.points(), st.accent_col);
    let specular_col = theme.specular_rim_top();
    let is_light = theme.appearance == crate::theme::MacAppearance::Aqua;
    let glass_bg = if is_light {
        theme.box_background()
    } else {
        theme.box_background().gamma_multiply(m.glass_fill_alpha.points())
    };
    let dish_shadow = if is_light {
        theme.separator().gamma_multiply(m.active_fill_opacity.points())
    } else {
        theme.drop_shadow().gamma_multiply(0.5)
    };
    let outer_rim_col = if st.hovered_rotary { st.accent_col } else { specular_col };
    painter.circle_filled(hub_center + Vec2::new(0.0, gap * 2.0), r_outer + bw * 2.0, theme.drop_shadow());
    painter.circle_filled(hub_center + Vec2::new(0.0, gap), r_outer + bw, theme.drop_shadow());
    painter.circle(hub_center, r_outer, glass_bg, border_stroke);
    painter.circle_stroke(hub_center, r_outer - bw, Stroke::new(m.active_stroke_width.points(), outer_rim_col));

    if st.active_layer == MacThumbDialLayer::FreeJoystickBowl {
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
        let half_gap_deg = m.sector_gap_deg.points() * 0.5;
        let r_clover_mid = (r_clover_in + r_clover_out) * 0.5;
        painter.circle_filled(hub_center, r_clover_out + bw, dish_shadow);
        let passive_fill = if is_light {
            theme.window_background().gamma_multiply(m.glass_fill_alpha.points())
        } else {
            theme.box_background().gamma_multiply(m.active_fill_opacity.points())
        };
        let passive_stroke = Stroke::new(bw, if is_light { specular_col } else { theme.separator().gamma_multiply(0.5) });
        let passive_half_sweep = m.diagonal_sweep_deg.points() * 0.5 - half_gap_deg;
        for passive_deg in [m.angle_north_west_deg.points(), m.angle_south_west_deg.points()] {
            paint_annular_wedge_lit(
                painter,
                WedgeGeometry {
                    center: hub_center,
                    r_in: r_clover_in,
                    r_out: r_clover_out,
                    start_deg: passive_deg - passive_half_sweep,
                    end_deg: passive_deg + passive_half_sweep,
                },
                passive_fill,
                passive_stroke,
                st.pointer_norm * 0.5,
                m,
            );
        }
        for sector in MacThumbSector::ALL {
            let is_latched = st.latched_sector == Some(sector);
            let is_held = st.held_sector == Some(sector);
            let is_hovered = st.hovered_sector == Some(sector);
            let lift = st.sector_lifts[sector as usize].clamp(0.0, 1.0);
            let c_deg = sector.center_angle_deg(m);
            let c_rad = super::render::deg_to_rad(c_deg, m);
            let sec_unit = Vec2::new(c_rad.cos(), c_rad.sin());
            let lift_gain = gap + gap * 0.5;
            let lift_off = (sec_unit * 0.5 + st.pointer_norm * 0.5) * (lift * lift_gain);
            let r_out_lifted = r_clover_out + lift * gap;
            let half_sweep = sector.sweep_deg(m) * 0.5 - half_gap_deg;
            let base_fill = if is_light {
                theme.window_background()
            } else {
                theme.control_background().gamma_multiply(m.glass_fill_alpha.points())
            };
            let fill = if is_held {
                st.accent_col
            } else if is_latched {
                if is_light {
                    let mut h = egui::epaint::Hsva::from(st.accent_col);
                    h.s = (h.s * m.glass_fill_alpha.points()).clamp(0.0, 1.0);
                    h.v = (h.v + (1.0 - h.v) * m.active_fill_opacity.points()).clamp(0.0, 1.0);
                    Color32::from(h)
                } else {
                    st.accent_col.gamma_multiply(m.glass_fill_alpha.points())
                }
            } else if is_hovered || lift > 0.0 {
                let mut hsva = egui::epaint::Hsva::from(base_fill);
                let acc_h = egui::epaint::Hsva::from(st.accent_col);
                let l_clamped = lift.clamp(0.0, 1.0);
                hsva.h = acc_h.h;
                hsva.s = (hsva.s + (acc_h.s - hsva.s) * (l_clamped * 0.5)).clamp(0.0, 1.0);
                hsva.v = (hsva.v + (1.0 - hsva.v) * (l_clamped * m.hover_fill_opacity.points())).clamp(0.0, 1.0);
                Color32::from(hsva)
            } else {
                base_fill
            };
            let stroke = if is_latched || is_held || is_hovered { active_stroke } else { Stroke::new(bw, specular_col) };
            let light_vec = if is_hovered || is_held || lift > 0.0 { st.pointer_norm * (0.5 + 0.5 * lift) } else { st.pointer_norm * 0.5 };
            paint_annular_wedge_lit(
                painter,
                WedgeGeometry {
                    center: hub_center + lift_off,
                    r_in: r_clover_in,
                    r_out: r_out_lifted,
                    start_deg: c_deg - half_sweep,
                    end_deg: c_deg + half_sweep,
                },
                fill,
                stroke,
                light_vec,
                m,
            );
            let spec = dial.sectors.sector(sector);
            let parallax = m.active_stroke_width.points() / bw.max(1.0);
            let center_pt = polar_pos(hub_center + lift_off * parallax, r_clover_mid + lift * gap * 0.5, c_deg, m);
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
        let hub_fill = if is_light {
            theme.window_background()
        } else {
            theme.control_background().gamma_multiply(m.glass_fill_alpha.points())
        };
        painter.circle_filled(hub_center + Vec2::new(0.0, bw * 2.0), r_hub, theme.drop_shadow());
        painter.circle(hub_center, r_hub, hub_fill, Stroke::new(bw, specular_col));
        if is_light {
            painter.circle_stroke(hub_center, r_hub, Stroke::new(bw, theme.separator().gamma_multiply(0.5)));
        }
        let ch_arm = m.latch_dot_inset.points() * s;
        let ch_stroke = Stroke::new(bw, theme.secondary_label());
        painter.circle_stroke(hub_center, ch_arm * 0.5, ch_stroke);
        painter.line_segment([hub_center - Vec2::new(ch_arm, 0.0), hub_center + Vec2::new(ch_arm, 0.0)], ch_stroke);
        painter.line_segment([hub_center - Vec2::new(0.0, ch_arm), hub_center + Vec2::new(0.0, ch_arm)], ch_stroke);
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
