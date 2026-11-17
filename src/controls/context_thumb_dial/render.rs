//! AppKit "Liquid Glass" 2-Layer Renderer & State Machine for [`MacContextThumbDial`] (`SPEC-014`).

use super::icons::{paint_annular_wedge, paint_sector_icon, MacTapticEvent, WedgeGeometry};
use super::{
    MacContextThumbDial, MacContextThumbResponse, MacThumbArcPlacement, MacThumbDialLayer, MacThumbDockZone, MacThumbIntent,
    MacThumbIntentMask, MacThumbSector, ThumbDialPressMemory,
};
use crate::controls::free_joystick_bowl::{paint_3d_recessed_bowl, MarblePaintSpec};
use crate::theme::{MacBackgroundStyle, MacTheme};
use crate::tokens::ThumbDialMetrics;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Sense, Stroke, Ui, Vec2};

#[inline]
pub(crate) fn deg_to_rad(deg: f32, m: &ThumbDialMetrics) -> f32 {
    deg * std::f32::consts::PI / m.half_circle_deg.points()
}

#[inline]
pub(crate) fn rad_to_deg(rad: f32, m: &ThumbDialMetrics) -> f32 {
    rad * m.half_circle_deg.points() / std::f32::consts::PI
}

#[inline]
pub(crate) fn polar_pos(center: Pos2, radius: f32, angle_deg: f32, m: &ThumbDialMetrics) -> Pos2 {
    let rad = deg_to_rad(angle_deg, m);
    center + Vec2::new(rad.cos() * radius, rad.sin() * radius)
}

#[inline]
fn shortest_angle_diff_deg(a_deg: f32, b_deg: f32, m: &ThumbDialMetrics) -> f32 {
    let full = m.full_circle_deg.points();
    let half = m.half_circle_deg.points();
    let mut diff = (a_deg - b_deg) % full;
    if diff > half {
        diff -= full;
    }
    if diff < -half {
        diff += full;
    }
    diff
}

fn hit_test_sector(angle_deg: f32, m: &ThumbDialMetrics) -> MacThumbSector {
    let mut best_sector = MacThumbSector::West;
    let mut best_dist = m.full_circle_deg.points();
    for sector in MacThumbSector::ALL {
        let dist = shortest_angle_diff_deg(angle_deg, sector.center_angle_deg(m), m).abs();
        let half_sweep = sector.sweep_deg(m) * 0.5;
        if dist <= half_sweep && dist < best_dist {
            best_dist = dist;
            best_sector = sector;
        }
    }
    if best_dist == m.full_circle_deg.points() {
        for sector in MacThumbSector::ALL {
            let dist = shortest_angle_diff_deg(angle_deg, sector.center_angle_deg(m), m).abs();
            if dist < best_dist {
                best_dist = dist;
                best_sector = sector;
            }
        }
    }
    best_sector
}



pub(crate) fn resolve_undo_redo_arcs(placement: MacThumbArcPlacement, m: &ThumbDialMetrics) -> ((f32, f32), (f32, f32)) {
    match placement {
        MacThumbArcPlacement::TopCrown => {
            ((m.top_undo_arc_start_deg.points(), m.top_undo_arc_end_deg.points()), (m.top_redo_arc_start_deg.points(), m.top_redo_arc_end_deg.points()))
        }
        MacThumbArcPlacement::BottomSkirt => {
            ((m.undo_arc_start_deg.points(), m.undo_arc_end_deg.points()), (m.redo_arc_start_deg.points(), m.redo_arc_end_deg.points()))
        }
        MacThumbArcPlacement::TopAndBottomCaps => {
            ((m.cap_top_start_deg.points(), m.cap_top_end_deg.points()), (m.cap_bottom_start_deg.points(), m.cap_bottom_end_deg.points()))
        }
    }
}

#[inline]
pub(crate) fn angle_in_arc(angle_deg: f32, start_deg: f32, end_deg: f32, m: &ThumbDialMetrics) -> bool {
    let mid = (start_deg + end_deg) * 0.5;
    let half = (end_deg - start_deg).abs() * 0.5;
    shortest_angle_diff_deg(angle_deg, mid, m).abs() <= half
}

fn infer_latched_sector(mask: MacThumbIntentMask, current: Option<MacThumbSector>) -> Option<MacThumbSector> {
    if mask.is_empty() {
        return None;
    }
    if let Some(sec) = current {
        if sec.intent_mask() == mask {
            return Some(sec);
        }
    }
    MacThumbSector::ALL.into_iter().find(|s| s.intent_mask() == mask)
}

pub(super) fn show_dial(mut dial: MacContextThumbDial<'_>, ui: &mut Ui) -> MacContextThumbResponse {
    let theme_cow = MacTheme::resolve(dial.theme, ui.ctx());
    let theme: &MacTheme = &theme_cow;
    let m = theme.thumb_dial();
    let s = dial.scale;

    let r_hub = m.hub_radius.points() * s;
    let gap = m.ring_gap.points() * s;
    let r_clover_in = r_hub + gap;
    let r_clover_out = m.clover_outer_radius.points() * s;
    let r_outer = m.outer_radius.points() * s;
    let r_skirt_in = m.skirt_inner_radius.points() * s;
    let r_skirt_out = m.skirt_outer_radius.points() * s;

    let widget_w = (r_outer + gap) * 2.0;
    let widget_h = (r_skirt_out + gap) * 2.0;
    let desired_size = Vec2::new(widget_w, widget_h);

    let (rect, response) = if let Some((canvas_rect, zone)) = dial.dock_target {
        let margin = m.dock_margin.points() * s;
        let min_pos = match zone {
            MacThumbDockZone::BottomLeft => Pos2::new(canvas_rect.min.x + margin, canvas_rect.max.y - widget_h - margin),
            MacThumbDockZone::BottomRight | MacThumbDockZone::Inline => {
                Pos2::new(canvas_rect.max.x - widget_w - margin, canvas_rect.max.y - widget_h - margin)
            }
        };
        let docked_rect = Rect::from_min_size(min_pos, desired_size);
        let resp = ui.interact(docked_rect, dial.id_source, Sense::click_and_drag());
        (docked_rect, resp)
    } else {
        ui.allocate_exact_size(desired_size, Sense::click_and_drag())
    };

    let hub_center = rect.center();
    let r_clover_mid = (r_clover_in + r_clover_out) * 0.5;
    let sector_centers = MacThumbSector::ALL.map(|sec| polar_pos(hub_center, r_clover_mid, sec.center_angle_deg(m), m));

    let ((undo_start, undo_end), (redo_start, redo_end)) = resolve_undo_redo_arcs(dial.arc_placement, m);
    let r_skirt_mid = (r_skirt_in + r_skirt_out) * 0.5;
    let undo_center = polar_pos(hub_center, r_skirt_mid, (undo_start + undo_end) * 0.5, m);
    let redo_center = polar_pos(hub_center, r_skirt_mid, (redo_start + redo_end) * 0.5, m);
    let btn_hit_size = Vec2::new((r_skirt_out - r_skirt_in) * 2.0, r_skirt_out - r_skirt_in);
    let undo_rect = Rect::from_center_size(undo_center, btn_hit_size);
    let redo_rect = Rect::from_center_size(redo_center, btn_hit_size);

    let mem_id = dial.id_source.with("thumb_dial_press_state");
    let mut mem = ui.ctx().data_mut(|d| d.get_temp::<ThumbDialPressMemory>(mem_id).unwrap_or_default());
    if let Some(bound_layer) = dial.layer_binding.as_deref() {
        mem.stored_layer = *bound_layer;
    }
    mem.latched_sector = infer_latched_sector(*dial.latched_mask, mem.latched_sector);

    let now = ui.input(|i| i.time);
    let pointer_pos = response.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.hover_pos()));

    let mut hovered_sector = None;
    let mut hovered_undo = false;
    let mut hovered_redo = false;
    let mut hovered_rotary = false;

    if let Some(pos) = pointer_pos {
        let delta = pos - hub_center;
        let dist = delta.length();
        let angle_deg = rad_to_deg(delta.y.atan2(delta.x), m);
        if undo_rect.contains(pos) || (dist >= r_outer && dist <= r_skirt_out && angle_in_arc(angle_deg, undo_start, undo_end, m)) {
            hovered_undo = true;
        } else if redo_rect.contains(pos) || (dist >= r_outer && dist <= r_skirt_out && angle_in_arc(angle_deg, redo_start, redo_end, m)) {
            hovered_redo = true;
        } else if dist > r_clover_out && dist <= r_outer {
            hovered_rotary = true;
        } else if mem.stored_layer == MacThumbDialLayer::CompassModifiers && dist > r_hub && dist <= r_clover_out {
            hovered_sector = Some(hit_test_sector(angle_deg, m));
        }
    }

    let is_down = response.is_pointer_button_down_on();
    let mut rotary_delta_ticks = 0.0f32;
    let mut joystick_nudge = Vec2::ZERO;
    let mut undo_clicked = false;
    let mut redo_clicked = false;
    let mut taptic_event = None;

    if is_down {
        if let Some(pos) = pointer_pos {
            let delta = pos - hub_center;
            let dist = delta.length();
            let angle_deg = rad_to_deg(delta.y.atan2(delta.x), m);

            let eff_hub_r = if mem.stored_layer == MacThumbDialLayer::FreeJoystickBowl { r_hub * 0.5 } else { r_hub };
            if mem.press_start_time.is_none() {
                mem.press_start_time = Some(now);
                mem.started_in_hub = dist <= eff_hub_r;
                mem.started_in_rotary = dist > r_clover_out && dist <= r_outer;
                mem.initial_sector = if mem.stored_layer == MacThumbDialLayer::CompassModifiers && dist > r_hub && dist <= r_clover_out {
                    Some(hit_test_sector(angle_deg, m))
                } else {
                    None
                };
                mem.active_sector = mem.initial_sector;
                mem.last_rotary_angle_deg = if mem.started_in_rotary { Some(angle_deg) } else { None };
            }

            if mem.started_in_rotary {
                if let Some(prev_deg) = mem.last_rotary_angle_deg {
                    let step_deg = shortest_angle_diff_deg(angle_deg, prev_deg, m);
                    if step_deg.abs() > 0.0 {
                        rotary_delta_ticks = step_deg / m.detent_step_deg.points();
                        *dial.rotary_value = (*dial.rotary_value + step_deg / m.full_circle_deg.points()).clamp(0.0, 1.0);
                        mem.last_rotary_angle_deg = Some(angle_deg);
                        taptic_event = Some(MacTapticEvent::RotaryDetentTick);
                    }
                }
            } else if mem.stored_layer == MacThumbDialLayer::FreeJoystickBowl && (!mem.started_in_hub || dist > eff_hub_r) {
                let prev_r = mem.joystick_offset.length();
                let clamped_r = dist.min(r_clover_out);
                let dir = if dist > 0.0 { delta / dist } else { Vec2::ZERO };
                mem.joystick_offset = dir * clamped_r;
                let fine_r = r_clover_out * m.joystick_fine_ratio.points();
                if (prev_r < fine_r && clamped_r >= fine_r) || (prev_r < r_clover_out && clamped_r >= r_clover_out) {
                    taptic_event = Some(MacTapticEvent::JoystickRingCrossed);
                }
                let ratio = (clamped_r / r_clover_out.max(1.0)).clamp(0.0, 1.0);
                joystick_nudge = dir * (ratio * ratio * m.joystick_max_speed.points());
            } else if mem.stored_layer == MacThumbDialLayer::CompassModifiers {
                if dist > r_hub && dist <= r_outer {
                    mem.active_sector = Some(hit_test_sector(angle_deg, m));
                } else if dist <= r_hub {
                    mem.active_sector = None;
                }
            }
        }
    } else if let Some(start_t) = mem.press_start_time.take() {
        let elapsed = (now - start_t).max(0.0) as f32;
        let is_quick_tap = elapsed < m.latch_tap_max_secs.points();
        let cur_layer = mem.stored_layer;
        if hovered_undo && dial.can_undo {
            undo_clicked = true;
            taptic_event = Some(MacTapticEvent::HistoryStep { is_redo: false });
        } else if hovered_redo && dial.can_redo {
            redo_clicked = true;
            taptic_event = Some(MacTapticEvent::HistoryStep { is_redo: true });
        } else if is_quick_tap && mem.started_in_hub && mem.active_sector.is_none() && mem.joystick_offset == Vec2::ZERO {
            if !dial.latched_mask.is_empty() {
                mem.latched_sector = None;
                *dial.latched_mask = MacThumbIntentMask::NONE;
                taptic_event = Some(MacTapticEvent::SectorUnlatched);
            } else {
                let next_layer = cur_layer.toggle();
                mem.stored_layer = next_layer;
                if let Some(bound_layer) = dial.layer_binding.as_deref_mut() {
                    *bound_layer = next_layer;
                }
                taptic_event = Some(MacTapticEvent::LayerSwitched(next_layer));
            }
        } else if is_quick_tap && cur_layer == MacThumbDialLayer::CompassModifiers && !mem.started_in_hub && !mem.started_in_rotary {
            if let Some(sec) = mem.active_sector.or(mem.initial_sector) {
                if mem.latched_sector == Some(sec) {
                    mem.latched_sector = None;
                    *dial.latched_mask = MacThumbIntentMask::NONE;
                    taptic_event = Some(MacTapticEvent::SectorUnlatched);
                } else {
                    mem.latched_sector = Some(sec);
                    *dial.latched_mask = sec.intent_mask();
                    taptic_event = Some(MacTapticEvent::SectorLatched(sec));
                }
            }
        }
        let (saved_layer, saved_latch) = (mem.stored_layer, mem.latched_sector);
        mem = ThumbDialPressMemory { stored_layer: saved_layer, latched_sector: saved_latch, ..Default::default() };
    }

    let active_layer = mem.stored_layer;
    let latched_sector = mem.latched_sector;
    ui.ctx().data_mut(|d| d.insert_temp(mem_id, mem));

    let held_sector = if is_down && active_layer == MacThumbDialLayer::CompassModifiers { mem.active_sector } else { None };
    let momentary_mask = held_sector.map(MacThumbSector::intent_mask).unwrap_or(MacThumbIntentMask::NONE);
    let effective_mask = dial.latched_mask.union(momentary_mask);
    let nudge_delta = if joystick_nudge != Vec2::ZERO {
        joystick_nudge
    } else if rotary_delta_ticks != 0.0 {
        if effective_mask.contains(MacThumbIntent::Constrain) || effective_mask.contains(MacThumbIntent::Glide) {
            Vec2::new(0.0, -rotary_delta_ticks)
        } else {
            Vec2::new(rotary_delta_ticks, 0.0)
        }
    } else {
        Vec2::ZERO
    };

    let accent_col = theme.resolve_glass_accent_on(dial.glass_accent, theme.popover_background());
    let painter = ui.painter_at(rect.intersect(ui.clip_rect()));
    let state = DialPaintState {
        hub_center,
        active_layer,
        latched_sector,
        held_sector,
        hovered_sector,
        hovered_rotary,
        hovered_undo,
        hovered_redo,
        joystick_offset: mem.joystick_offset,
        accent_col,
    };
    paint_dial_layers(&painter, theme, m, &dial, state);

    MacContextThumbResponse {
        response,
        active_layer,
        latched_sector,
        latched_mask: *dial.latched_mask,
        momentary_mask,
        effective_mask,
        active_sector: held_sector.or(hovered_sector).or(latched_sector),
        rotary_delta_ticks,
        nudge_delta,
        undo_clicked,
        redo_clicked,
        hub_center,
        undo_rect,
        redo_rect,
        effective_glass_accent: accent_col,
        taptic_event,
        sector_centers,
    }
}

struct DialPaintState {
    hub_center: Pos2,
    active_layer: MacThumbDialLayer,
    latched_sector: Option<MacThumbSector>,
    held_sector: Option<MacThumbSector>,
    hovered_sector: Option<MacThumbSector>,
    hovered_rotary: bool,
    hovered_undo: bool,
    hovered_redo: bool,
    joystick_offset: Vec2,
    accent_col: Color32,
}

fn paint_dial_layers(painter: &Painter, theme: &MacTheme, m: &ThumbDialMetrics, dial: &MacContextThumbDial<'_>, st: DialPaintState) {
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
    let specular_col = theme.knob_fill().gamma_multiply(m.glass_specular_alpha.points());
    let glass_bg = theme.popover_background().gamma_multiply(m.glass_fill_alpha.points());

    // 5-Layer 3D Liquid Glass Base Puck (deep drop shadow + recessed rim + frosted disc + top Fresnel ring)
    painter.circle_filled(hub_center + Vec2::new(0.0, gap * 2.0), r_outer + bw, theme.drop_shadow());
    painter.circle_filled(hub_center + Vec2::new(0.0, gap), r_outer, theme.drop_shadow());
    painter.circle(hub_center, r_outer, glass_bg, border_stroke);
    painter.circle_stroke(hub_center, r_outer - bw, Stroke::new(m.active_stroke_width.points(), specular_col));

    // Outer tactile rim ring
    let wheel_bg = if st.hovered_rotary { theme.control() } else { theme.box_background().gamma_multiply(m.glass_fill_alpha.points()) };
    paint_annular_wedge(
        painter,
        WedgeGeometry { center: hub_center, r_in: r_wheel_in, r_out: r_outer, start_deg: 0.0, end_deg: m.full_circle_deg.points() },
        wheel_bg,
        border_stroke,
        m,
    );

    if st.active_layer == MacThumbDialLayer::FreeJoystickBowl {
        let marble_center = hub_center + st.joystick_offset;
        let puck_r = m.joystick_puck_radius.points() * s;
        paint_3d_recessed_bowl(
            painter,
            MarblePaintSpec { hub_center, marble_center, puck_r, scale: s, accent_col: st.accent_col },
            r_clover_out,
            theme,
            m,
        );
    } else {
        // Layer 1: 6 Exclusive 3D Liquid Glass Cushions with Apple SF Symbol Vector Icons
        let half_gap_deg = m.sector_gap_deg.points() * 0.5;
        let r_clover_mid = (r_clover_in + r_clover_out) * 0.5;
        painter.circle_filled(hub_center, r_clover_out + bw, theme.drop_shadow());
        for sector in MacThumbSector::ALL {
            let is_latched = st.latched_sector == Some(sector);
            let is_held = st.held_sector == Some(sector);
            let is_active = is_latched || is_held;
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
            let stroke = if is_active || is_hovered { active_stroke } else { Stroke::new(bw, specular_col) };
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
            let held_style = MacBackgroundStyle::from_color(st.accent_col);
            let fg_col = if is_held { held_style.primary_text_color(theme) } else { theme.label() };
            let icon_sz = m.label_font_size.points() * s;
            let icon_rect = Rect::from_center_size(center_pt - Vec2::new(0.0, m.label_vertical_offset.points() * s), Vec2::splat(icon_sz));
            paint_sector_icon(painter, spec.icon, icon_rect, fg_col, m);
            painter.text(
                center_pt + Vec2::new(0.0, m.hint_vertical_offset.points() * s),
                Align2::CENTER_CENTER,
                spec.label,
                FontId::proportional(m.hint_font_size.points() * s),
                fg_col,
            );

            if is_latched {
                let dot_pos = polar_pos(hub_center, r_hub + m.latch_dot_inset.points() * s, c_deg, m);
                let dot_r = m.latch_dot_radius.points() * s;
                painter.circle_filled(dot_pos, dot_r * 2.0, theme.system_orange().gamma_multiply(m.active_fill_opacity.points()));
                painter.circle_filled(dot_pos, dot_r, theme.system_orange());
            }
        }

        // 3D Center Thumb Rest & Layer-Toggle Hub
        let has_active = st.latched_sector.is_some() || st.held_sector.is_some();
        let hub_fill = if has_active { theme.control() } else { theme.control_background().gamma_multiply(m.glass_fill_alpha.points()) };
        painter.circle_filled(hub_center + Vec2::new(0.0, bw), r_hub, theme.drop_shadow());
        painter.circle(hub_center, r_hub, hub_fill, if has_active { active_stroke } else { Stroke::new(bw, specular_col) });
        let ch_arm = m.latch_dot_inset.points() * s;
        painter.circle_stroke(hub_center, ch_arm * 0.5, border_stroke);
        painter.line_segment([hub_center - Vec2::new(ch_arm, 0.0), hub_center + Vec2::new(ch_arm, 0.0)], border_stroke);
        painter.line_segment([hub_center - Vec2::new(0.0, ch_arm), hub_center + Vec2::new(0.0, ch_arm)], border_stroke);
    }

    // Curved 3D Liquid Glass Undo / Redo Crescents
    let ((undo_start, undo_end), (redo_start, redo_end)) = resolve_undo_redo_arcs(dial.arc_placement, m);
    let r_skirt_mid = (r_skirt_in + r_skirt_out) * 0.5;
    for (start_d, end_d, lbl, enabled, hovered) in [
        (undo_start, undo_end, "↶ Undo", dial.can_undo, st.hovered_undo),
        (redo_start, redo_end, "↷ Redo", dial.can_redo, st.hovered_redo),
    ] {
        let fill = if hovered && enabled {
            st.accent_col.gamma_multiply(m.active_fill_opacity.points())
        } else {
            theme.control_background().gamma_multiply(m.glass_fill_alpha.points())
        };
        let stroke = if hovered && enabled { active_stroke } else { Stroke::new(bw, specular_col) };
        paint_annular_wedge(
            painter,
            WedgeGeometry { center: hub_center, r_in: r_skirt_in, r_out: r_skirt_out, start_deg: start_d, end_deg: end_d },
            fill,
            stroke,
            m,
        );
        let mid_pt = polar_pos(hub_center, r_skirt_mid, (start_d + end_d) * 0.5, m);
        let fg = if enabled { theme.label() } else { theme.tertiary_label() };
        painter.text(mid_pt, Align2::CENTER_CENTER, lbl, FontId::proportional(m.label_font_size.points() * s), fg);
    }
}
