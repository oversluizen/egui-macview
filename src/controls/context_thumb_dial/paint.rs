//! 3D Cushion, Circular Tactile Ring, and Transient HUD Tooltip Painter for [`super::MacContextThumbDial`].

use super::icons::{paint_annular_wedge, paint_sector_icon, WedgeGeometry};
use super::render::{polar_pos, resolve_undo_redo_arcs};
use super::{MacContextThumbDial, MacThumbDialLayer, MacThumbSector};
use crate::controls::free_joystick_bowl::{paint_3d_recessed_bowl, MarblePaintSpec};
use crate::theme::{MacBackgroundStyle, MacTheme};
use crate::tokens::ThumbDialMetrics;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};

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
    let r_clover_in = r_hub + gap;
    let r_clover_out = m.clover_outer_radius.points() * s;
    let r_outer = m.outer_radius.points() * s;
    let r_skirt_in = m.skirt_inner_radius.points() * s;
    let r_skirt_out = m.skirt_outer_radius.points() * s;
    let r_wheel_in = r_clover_out + gap;
    let bw = m.border_stroke_width.points();

    let border_stroke = Stroke::new(bw, theme.separator());
    let active_stroke = Stroke::new(m.active_stroke_width.points(), st.accent_col);
    let specular_col = theme.specular_rim_top();
    let glass_bg = theme.popover_background().gamma_multiply(m.glass_fill_alpha.points());

    painter.circle_filled(hub_center + Vec2::new(0.0, gap * 2.0), r_outer + bw * 2.0, theme.drop_shadow());
    painter.circle_filled(hub_center + Vec2::new(0.0, gap), r_outer + bw, theme.drop_shadow());
    painter.circle(hub_center, r_outer, glass_bg, border_stroke);
    painter.circle_stroke(hub_center, r_outer - bw, Stroke::new(m.active_stroke_width.points(), specular_col));

    let wheel_bg = if st.hovered_rotary {
        theme.control()
    } else {
        theme.box_background().gamma_multiply(m.glass_fill_alpha.points())
    };
    painter.circle_stroke(
        hub_center,
        (r_wheel_in + r_outer) * 0.5,
        Stroke::new((r_outer - r_wheel_in).max(1.0), wheel_bg),
    );
    painter.circle_stroke(hub_center, r_outer, border_stroke);
    painter.circle_stroke(hub_center, r_wheel_in, Stroke::new(bw, specular_col));
    let tick_count = 24usize;
    let full_deg = m.full_circle_deg.points();
    for i in 0..tick_count {
        let a_deg = full_deg * (i as f32 / tick_count as f32);
        painter.line_segment(
            [
                polar_pos(hub_center, r_wheel_in + bw, a_deg, m),
                polar_pos(hub_center, r_outer - bw, a_deg, m),
            ],
            Stroke::new(bw, theme.separator()),
        );
    }

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
        painter.circle_filled(hub_center, r_clover_out + bw, theme.drop_shadow());
        for sector in MacThumbSector::ALL {
            let is_latched = st.latched_sector == Some(sector);
            let is_held = st.held_sector == Some(sector);
            let is_hovered = st.hovered_sector == Some(sector);
            let c_deg = sector.center_angle_deg(m);
            let half_sweep = sector.sweep_deg(m) * 0.5 - half_gap_deg;
            let fill = if is_held {
                st.accent_col
            } else if is_latched {
                st.accent_col.gamma_multiply(m.active_fill_opacity.points())
            } else if is_hovered {
                st.accent_col.gamma_multiply(m.hover_fill_opacity.points())
            } else if sector.is_diagonal_combo() {
                theme.box_background().gamma_multiply(m.glass_fill_alpha.points())
            } else {
                theme.control_background().gamma_multiply(m.glass_fill_alpha.points())
            };
            let stroke = if is_latched || is_held || is_hovered {
                active_stroke
            } else {
                Stroke::new(bw, specular_col)
            };
            paint_annular_wedge(
                painter,
                WedgeGeometry {
                    center: hub_center,
                    r_in: r_clover_in,
                    r_out: r_clover_out,
                    start_deg: c_deg - half_sweep,
                    end_deg: c_deg + half_sweep,
                },
                fill,
                stroke,
                m,
            );
            let spec = dial.sectors.sector(sector);
            let center_pt = polar_pos(hub_center, r_clover_mid, c_deg, m);
            let fg_col = if is_held {
                MacBackgroundStyle::from_color(st.accent_col).primary_text_color(theme)
            } else {
                theme.label()
            };
            let icon_sz = Vec2::splat(m.label_font_size.points() * s);
            paint_sector_icon(
                painter,
                spec.icon,
                Rect::from_center_size(center_pt + Vec2::new(0.0, bw), icon_sz),
                theme.drop_shadow(),
                m,
            );
            paint_sector_icon(painter, spec.icon, Rect::from_center_size(center_pt, icon_sz), fg_col, m);
            if is_latched {
                let dot_pos = polar_pos(hub_center, r_hub + m.latch_dot_inset.points() * s, c_deg, m);
                let dot_r = m.latch_dot_radius.points() * s;
                painter.circle_filled(dot_pos, dot_r * 2.0, theme.system_orange().gamma_multiply(m.active_fill_opacity.points()));
                painter.circle_filled(dot_pos, dot_r, theme.system_orange());
            }
        }
        let has_active = st.latched_sector.is_some() || st.held_sector.is_some();
        let hub_fill = if has_active {
            theme.control()
        } else {
            theme.control_background().gamma_multiply(m.glass_fill_alpha.points())
        };
        painter.circle_filled(hub_center + Vec2::new(0.0, bw * 2.0), r_hub, theme.drop_shadow());
        painter.circle(
            hub_center,
            r_hub,
            hub_fill,
            if has_active { active_stroke } else { Stroke::new(bw, specular_col) },
        );
        painter.circle_filled(hub_center - Vec2::new(0.0, r_hub * 0.5), r_hub * 0.5, specular_col.gamma_multiply(0.5));
        let ch_arm = m.latch_dot_inset.points() * s;
        painter.circle_stroke(hub_center, ch_arm * 0.5, border_stroke);
        painter.line_segment([hub_center - Vec2::new(ch_arm, 0.0), hub_center + Vec2::new(ch_arm, 0.0)], border_stroke);
        painter.line_segment([hub_center - Vec2::new(0.0, ch_arm), hub_center + Vec2::new(0.0, ch_arm)], border_stroke);
    }

    let ((undo_start, undo_end), (redo_start, redo_end)) = resolve_undo_redo_arcs(dial.arc_placement, m);
    let r_skirt_mid = (r_skirt_in + r_skirt_out) * 0.5;
    for (start_d, end_d, sym, enabled, hovered) in [
        (undo_start, undo_end, crate::symbols::MacSymbol::ArrowCounterClockwise, dial.can_undo, st.hovered_undo),
        (redo_start, redo_end, crate::symbols::MacSymbol::ArrowClockwise, dial.can_redo, st.hovered_redo),
    ] {
        let fill = if hovered && enabled {
            st.accent_col.gamma_multiply(m.active_fill_opacity.points())
        } else {
            theme.control_background().gamma_multiply(m.glass_fill_alpha.points())
        };
        let stroke = if hovered && enabled { active_stroke } else { border_stroke };
        paint_annular_wedge(
            painter,
            WedgeGeometry {
                center: hub_center,
                r_in: r_skirt_in,
                r_out: r_skirt_out,
                start_deg: start_d,
                end_deg: end_d,
            },
            fill,
            stroke,
            m,
        );
        let mid_pt = polar_pos(hub_center, r_skirt_mid, (start_d + end_d) * 0.5, m);
        painter.text(
            mid_pt,
            Align2::CENTER_CENTER,
            sym.as_str(),
            FontId::proportional(m.label_font_size.points() * s),
            if enabled { theme.label() } else { theme.tertiary_label() },
        );
    }

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
