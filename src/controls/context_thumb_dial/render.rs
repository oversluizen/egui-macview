//! AppKit "Liquid Glass" 2-Layer Renderer & State Machine for [`MacContextThumbDial`] (`SPEC-014`).

use super::icons::MacTapticEvent;
use super::paint::{paint_dial_layers, DialPaintState};
use super::{
    MacContextThumbDial, MacContextThumbResponse, MacThumbArcPlacement, MacThumbDialLayer, MacThumbDockZone, MacThumbIntent,
    MacThumbIntentMask, MacThumbSector, ThumbDialPressMemory,
};
use crate::theme::MacTheme;
use crate::tokens::ThumbDialMetrics;
use egui::{Pos2, Rect, Sense, Ui, Vec2};

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
pub(crate) fn shortest_angle_diff_deg(a_deg: f32, b_deg: f32, m: &ThumbDialMetrics) -> f32 {
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

pub(crate) type UndoRedoArcs = ((f32, f32), (f32, f32));

pub(crate) fn resolve_undo_redo_arcs(placement: MacThumbArcPlacement, m: &ThumbDialMetrics) -> UndoRedoArcs {
    match placement {
        MacThumbArcPlacement::TopCrown => (
            (m.top_undo_arc_start_deg.points(), m.top_undo_arc_end_deg.points()),
            (m.top_redo_arc_start_deg.points(), m.top_redo_arc_end_deg.points()),
        ),
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

pub(crate) fn allocate_docked_thumb_rect(
    ui: &mut Ui,
    id_source: egui::Id,
    dock_target: Option<(Rect, MacThumbDockZone)>,
    desired_size: Vec2,
    m: &ThumbDialMetrics,
    scale: f32,
) -> (Rect, egui::Response) {
    if let Some((canvas_rect, zone)) = dock_target {
        let margin = m.dock_margin.points() * scale;
        let min_pos = match zone {
            MacThumbDockZone::BottomLeft => Pos2::new(canvas_rect.min.x + margin, canvas_rect.max.y - desired_size.y - margin),
            MacThumbDockZone::BottomRight | MacThumbDockZone::Inline => {
                Pos2::new(canvas_rect.max.x - desired_size.x - margin, canvas_rect.max.y - desired_size.y - margin)
            }
        };
        let docked_rect = Rect::from_min_size(min_pos, desired_size);
        let resp = ui.interact(docked_rect, id_source, Sense::click_and_drag());
        (docked_rect, resp)
    } else {
        ui.allocate_exact_size(desired_size, Sense::click_and_drag())
    }
}

pub(crate) fn compute_undo_redo_hit_rects(
    hub_center: Pos2,
    placement: MacThumbArcPlacement,
    r_skirt_in: f32,
    r_skirt_out: f32,
    m: &ThumbDialMetrics,
) -> (UndoRedoArcs, Rect, Rect) {
    let arcs @ ((undo_start, undo_end), (redo_start, redo_end)) = resolve_undo_redo_arcs(placement, m);
    let r_skirt_mid = (r_skirt_in + r_skirt_out) * 0.5;
    let undo_center = polar_pos(hub_center, r_skirt_mid, (undo_start + undo_end) * 0.5, m);
    let redo_center = polar_pos(hub_center, r_skirt_mid, (redo_start + redo_end) * 0.5, m);
    let btn_hit_size = Vec2::new((r_skirt_out - r_skirt_in) * 2.0, r_skirt_out - r_skirt_in);
    (arcs, Rect::from_center_size(undo_center, btn_hit_size), Rect::from_center_size(redo_center, btn_hit_size))
}

pub(crate) fn hit_test_undo_redo_arcs(
    pos: Pos2,
    hub_center: Pos2,
    (r_outer, r_skirt_out): (f32, f32),
    ((undo_start, undo_end), (redo_start, redo_end)): UndoRedoArcs,
    (undo_rect, redo_rect): (Rect, Rect),
    m: &ThumbDialMetrics,
) -> (bool, bool) {
    let delta = pos - hub_center;
    let dist = delta.length();
    let angle_deg = rad_to_deg(delta.y.atan2(delta.x), m);
    let in_skirt = dist >= r_outer && dist <= r_skirt_out;
    let d_undo = super::super::liquid_glass::sdf_curved_stadium_arc(pos, hub_center, (r_outer, r_skirt_out), (undo_start, undo_end), m);
    let d_redo = super::super::liquid_glass::sdf_curved_stadium_arc(pos, hub_center, (r_outer, r_skirt_out), (redo_start, redo_end), m);
    let hov_undo = d_undo <= 0.0 || undo_rect.contains(pos) || (in_skirt && angle_in_arc(angle_deg, undo_start, undo_end, m));
    let hov_redo = !hov_undo && (d_redo <= 0.0 || redo_rect.contains(pos) || (in_skirt && angle_in_arc(angle_deg, redo_start, redo_end, m)));
    (hov_undo, hov_redo)
}

pub(crate) fn step_joystick_bowl_physics(
    joystick_offset: &mut Vec2,
    delta: Vec2,
    dist: f32,
    r_clover_out: f32,
    m: &ThumbDialMetrics,
) -> (Vec2, Option<MacTapticEvent>) {
    let prev_r = joystick_offset.length();
    let clamped_r = dist.min(r_clover_out);
    let dir = if dist > 0.0 { delta / dist } else { Vec2::ZERO };
    *joystick_offset = dir * clamped_r;
    let fine_r = r_clover_out * m.joystick_fine_ratio.points();
    let taptic = if (prev_r < fine_r && clamped_r >= fine_r) || (prev_r < r_clover_out && clamped_r >= r_clover_out) {
        Some(MacTapticEvent::JoystickRingCrossed)
    } else {
        None
    };
    let ratio = (clamped_r / r_clover_out.max(1.0)).clamp(0.0, 1.0);
    let nudge = dir * (ratio * ratio * m.joystick_max_speed.points());
    (nudge, taptic)
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

fn toggle_sector_latch(sec: MacThumbSector, mem: &mut ThumbDialPressMemory, latched_mask: &mut MacThumbIntentMask) -> MacTapticEvent {
    if mem.latched_sector == Some(sec) {
        mem.latched_sector = None;
        *latched_mask = MacThumbIntentMask::NONE;
        MacTapticEvent::SectorUnlatched
    } else {
        mem.latched_sector = Some(sec);
        *latched_mask = sec.intent_mask();
        MacTapticEvent::SectorLatched(sec)
    }
}

fn toggle_hub_layer(
    mem: &mut ThumbDialPressMemory,
    latched_mask: &mut MacThumbIntentMask,
    layer_binding: &mut Option<&mut MacThumbDialLayer>,
) -> MacTapticEvent {
    let cur_layer = mem.stored_layer;
    if let Some(bound_layer) = layer_binding.as_deref_mut() {
        mem.latched_sector = None;
        *latched_mask = MacThumbIntentMask::NONE;
        let next_layer = cur_layer.toggle();
        mem.stored_layer = next_layer;
        *bound_layer = next_layer;
        MacTapticEvent::LayerSwitched(next_layer)
    } else if !latched_mask.is_empty() {
        mem.latched_sector = None;
        *latched_mask = MacThumbIntentMask::NONE;
        MacTapticEvent::SectorUnlatched
    } else {
        let next_layer = cur_layer.toggle();
        mem.stored_layer = next_layer;
        MacTapticEvent::LayerSwitched(next_layer)
    }
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

    let desired_size = Vec2::new((r_outer + gap) * 2.0, (r_skirt_out + gap) * 2.0);
    let (rect, response) = allocate_docked_thumb_rect(ui, dial.id_source, dial.dock_target, desired_size, m, s);

    let hub_center = rect.center();
    let r_clover_mid = (r_clover_in + r_clover_out) * 0.5;
    let mut sector_centers = MacThumbSector::ALL.map(|sec| polar_pos(hub_center, r_clover_mid, sec.center_angle_deg(m), m));
    let (arcs, undo_rect, redo_rect) = compute_undo_redo_hit_rects(hub_center, dial.arc_placement, r_skirt_in, r_skirt_out, m);

    let mem_id = dial.id_source.with("thumb_dial_press_state");
    let mut mem = ui.ctx().data_mut(|d| d.get_temp::<ThumbDialPressMemory>(mem_id).unwrap_or_default());
    if let Some(bound_layer) = dial.layer_binding.as_deref() {
        mem.stored_layer = *bound_layer;
    }
    mem.latched_sector = infer_latched_sector(*dial.latched_mask, mem.latched_sector);

    let now = ui.input(|i| i.time);
    let pointer_pos = response
        .interact_pointer_pos()
        .or_else(|| ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.latest_pos())));

    let mut hovered_sector = None;
    let mut hovered_undo = false;
    let mut hovered_redo = false;
    let mut hovered_rotary = false;

    if let Some(pos) = pointer_pos {
        let (hu, hr) = hit_test_undo_redo_arcs(pos, hub_center, (r_outer, r_skirt_out), arcs, (undo_rect, redo_rect), m);
        hovered_undo = hu;
        hovered_redo = hr;
        if !hovered_undo && !hovered_redo {
            let delta = pos - hub_center;
            let dist = delta.length();
            let angle_deg = rad_to_deg(delta.y.atan2(delta.x), m);
            if dist > r_clover_out && dist <= r_outer {
                hovered_rotary = true;
            } else if mem.stored_layer == MacThumbDialLayer::CompassModifiers && dist > r_hub && dist <= r_clover_out {
                hovered_sector = Some(hit_test_sector(angle_deg, m));
            }
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
                mem.rotary_detent_accum_deg = 0.0;
            }

            if mem.started_in_rotary {
                if let Some(prev_deg) = mem.last_rotary_angle_deg {
                    let step_deg = shortest_angle_diff_deg(angle_deg, prev_deg, m);
                    if step_deg.abs() > 0.0 {
                        let detent_deg = m.detent_step_deg.points().max(1.0);
                        rotary_delta_ticks = step_deg / detent_deg;
                        *dial.rotary_value = (*dial.rotary_value + step_deg / m.full_circle_deg.points()).clamp(0.0, 1.0);
                        mem.last_rotary_angle_deg = Some(angle_deg);
                        mem.rotary_detent_accum_deg += step_deg;
                        if mem.rotary_detent_accum_deg.abs() >= detent_deg {
                            mem.rotary_detent_accum_deg %= detent_deg;
                            taptic_event = Some(MacTapticEvent::RotaryDetentTick);
                        }
                    }
                }
            } else if mem.stored_layer == MacThumbDialLayer::FreeJoystickBowl && (!mem.started_in_hub || dist > eff_hub_r) {
                let (nudge, tap) = step_joystick_bowl_physics(&mut mem.joystick_offset, delta, dist, r_clover_out, m);
                joystick_nudge = nudge;
                taptic_event = tap;
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
            taptic_event = Some(toggle_hub_layer(&mut mem, dial.latched_mask, &mut dial.layer_binding));
        } else if is_quick_tap && cur_layer == MacThumbDialLayer::CompassModifiers && !mem.started_in_hub && !mem.started_in_rotary {
            if let Some(sec) = mem.active_sector.or(mem.initial_sector) {
                taptic_event = Some(toggle_sector_latch(sec, &mut mem, dial.latched_mask));
            }
        }
        mem = ThumbDialPressMemory {
            stored_layer: mem.stored_layer,
            latched_sector: mem.latched_sector,
            tooltip_text: mem.tooltip_text,
            tooltip_triggered_at: mem.tooltip_triggered_at,
            last_tooltip_key: mem.last_tooltip_key,
            ..Default::default()
        };
    } else if response.clicked() {
        if hovered_undo && dial.can_undo {
            undo_clicked = true;
            taptic_event = Some(MacTapticEvent::HistoryStep { is_redo: false });
        } else if hovered_redo && dial.can_redo {
            redo_clicked = true;
            taptic_event = Some(MacTapticEvent::HistoryStep { is_redo: true });
        } else if let Some(sec) = hovered_sector {
            taptic_event = Some(toggle_sector_latch(sec, &mut mem, dial.latched_mask));
        } else if pointer_pos.is_some_and(|p| (p - hub_center).length() <= r_hub) {
            taptic_event = Some(toggle_hub_layer(&mut mem, dial.latched_mask, &mut dial.layer_binding));
        }
    }

    if response.has_focus() {
        let key_sec = ui.input(|i| {
            if i.key_pressed(egui::Key::ArrowUp) {
                Some(MacThumbSector::North)
            } else if i.key_pressed(egui::Key::ArrowRight) {
                Some(MacThumbSector::East)
            } else if i.key_pressed(egui::Key::ArrowDown) {
                Some(MacThumbSector::South)
            } else if i.key_pressed(egui::Key::ArrowLeft) {
                Some(MacThumbSector::West)
            } else {
                None
            }
        });
        if let Some(sec) = key_sec {
            taptic_event = Some(toggle_sector_latch(sec, &mut mem, dial.latched_mask));
        } else if ui.input(|i| i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::Enter)) {
            taptic_event = Some(toggle_hub_layer(&mut mem, dial.latched_mask, &mut dial.layer_binding));
        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            mem.latched_sector = None;
            *dial.latched_mask = MacThumbIntentMask::NONE;
            taptic_event = Some(MacTapticEvent::SectorUnlatched);
        }
    }
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Liquid Glass Context Thumb Dial"));

    let (active_layer, latched_sector) = (mem.stored_layer, mem.latched_sector);
    let held_sector = if is_down && active_layer == MacThumbDialLayer::CompassModifiers { mem.active_sector } else { None };
    let (cur_key, cur_tip): (u8, &str) = if let Some(sec) = held_sector.or(hovered_sector) {
        ((sec as u8) + 1, dial.sectors.sector(sec).label)
    } else if hovered_rotary {
        (12, "Rotary Ring")
    } else {
        (0, "")
    };
    if cur_key != 0 && cur_key != mem.last_tooltip_key {
        mem.last_tooltip_key = cur_key;
        mem.trigger_tooltip(cur_tip, now);
    } else if cur_key == 0 {
        mem.last_tooltip_key = 0;
    }
    let (tooltip_alpha, tooltip_str) = (mem.tooltip_opacity_at(now), mem.active_tooltip_text(now).map(str::to_owned));
    if let Some(delay_secs) = mem.tooltip_repaint_delay_at(now) {
        ui.ctx().request_repaint_after_secs(delay_secs);
    }
    ui.ctx().data_mut(|d| d.insert_temp(mem_id, mem.clone()));

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
    let (pointer_norm, sector_lifts, polar_morph) = super::super::liquid_glass::step_dial_pointer_springs(
        ui.ctx(),
        theme,
        m,
        dial.id_source,
        (hub_center, r_clover_mid, r_outer, r_skirt_out, gap),
        (pointer_pos, latched_sector, held_sector, hovered_sector, now),
        &mut sector_centers,
    );
    let painter = ui.painter_at(rect.intersect(ui.clip_rect()));
    paint_dial_layers(
        &painter,
        theme,
        m,
        &dial,
        DialPaintState {
            hub_center,
            active_layer,
            latched_sector,
            held_sector,
            hovered_sector,
            hovered_rotary,
            hovered_undo,
            hovered_redo,
            joystick_offset: mem.joystick_offset,
            pointer_norm,
            sector_lifts,
            polar_morph,
            accent_col,
            tooltip_alpha,
            tooltip_str,
        },
    );

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
