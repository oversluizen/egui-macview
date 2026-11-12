//! AppKit keyboard and pointer/menu gesture state machine for [`super::MacSegmentedControl`].

use super::item::{SegmentEvent, SegmentItem, SegmentMenuTrigger};
use super::layout::SegLayoutSlot;
use super::storage::InlineBuf;
use egui::{Id, Key, Modifiers, PointerButton, Pos2, Response, Ui};

#[derive(Clone, Copy)]
struct SegPressState {
    seg_idx: usize,
    origin: Pos2,
    press_time: f64,
    menu_fired: bool,
}

#[derive(Clone, Copy)]
pub(super) struct InteractionConfig {
    pub enabled: bool,
    pub momentary: bool,
    pub hold_delay: f64,
    pub drag_threshold: f32,
}

#[derive(Clone, Copy)]
struct PointerSnapshot {
    now: f64,
    primary_pressed: bool,
    primary_down: bool,
    primary_released: bool,
    ctrl_down: bool,
    latest_pos: Option<Pos2>,
    hit_pos: Option<Pos2>,
}

fn find_enabled_segment(
    segments: &InlineBuf<SegmentItem<'_>>,
    slots: &InlineBuf<SegLayoutSlot>,
    pos: Pos2,
) -> Option<usize> {
    let idx = slots.iter().position(|s| s.rect.contains(pos))?;
    segments.get(idx).is_some_and(|s| s.enabled).then_some(idx)
}

fn handle_keyboard_navigation(
    ui: &mut Ui,
    response: &mut Response,
    segments: &InlineBuf<SegmentItem<'_>>,
    slots: &InlineBuf<SegLayoutSlot>,
    active_idx: &mut usize,
    momentary: bool,
    on_event: &mut impl FnMut(SegmentEvent),
) {
    if !response.has_focus() || segments.is_empty() {
        return;
    }
    let count = segments.len();
    let cursor_id = response.id.with("seg_momentary_cursor");
    let current = if momentary {
        ui.ctx().data(|d| d.get_temp::<usize>(cursor_id)).unwrap_or(*active_idx).min(count.saturating_sub(1))
    } else {
        *active_idx
    };

    if ui.input_mut(|i| {
        i.consume_key(Modifiers::NONE, Key::ArrowDown)
            || i.consume_key(Modifiers::SHIFT, Key::F10)
    }) {
        if let (Some(seg), Some(slot)) = (segments.get(current), slots.get(current)) {
            if seg.enabled && seg.has_menu {
                on_event(SegmentEvent {
                    index: current,
                    menu_trigger: Some(SegmentMenuTrigger::Keyboard),
                    rect: slot.rect,
                });
                response.mark_changed();
            }
        }
        return;
    }

    if momentary
        && ui.input_mut(|i| {
            i.consume_key(Modifiers::NONE, Key::Space) || i.consume_key(Modifiers::NONE, Key::Enter)
        })
    {
        if let (Some(seg), Some(slot)) = (segments.get(current), slots.get(current)) {
            if seg.enabled {
                on_event(SegmentEvent { index: current, menu_trigger: None, rect: slot.rect });
                response.mark_changed();
            }
        }
        return;
    }

    let target = if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight)) {
        ((current + 1)..count).find(|&j| segments.get(j).is_some_and(|s| s.enabled))
    } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft)) {
        (0..current).rev().find(|&j| segments.get(j).is_some_and(|s| s.enabled))
    } else {
        None
    };
    if let Some(idx) = target {
        if momentary {
            ui.ctx().data_mut(|d| d.insert_temp(cursor_id, idx));
        } else if let Some(slot) = slots.get(idx) {
            *active_idx = idx;
            on_event(SegmentEvent { index: idx, menu_trigger: None, rect: slot.rect });
            response.mark_changed();
        }
    }
}

fn clear_press_state_if_present(ctx: &egui::Context, state_id: Id) -> Option<SegPressState> {
    let existing = ctx.data(|d| d.get_temp::<SegPressState>(state_id))?;
    ctx.data_mut(|d| d.remove::<SegPressState>(state_id));
    Some(existing)
}

/// Processes keyboard and pointer gestures, returning the currently depressed (`isHighlighted`) segment index if any.
pub(super) fn handle_segmented_input(
    ui: &mut Ui,
    response: &mut Response,
    segments: &InlineBuf<SegmentItem<'_>>,
    slots: &InlineBuf<SegLayoutSlot>,
    active_idx: &mut usize,
    cfg: InteractionConfig,
    on_event: &mut impl FnMut(SegmentEvent),
) -> Option<usize> {
    let state_id = response.id.with("seg_press_state");
    if !cfg.enabled || segments.is_empty() {
        let _ = clear_press_state_if_present(ui.ctx(), state_id);
        return None;
    }

    handle_keyboard_navigation(
        ui,
        response,
        segments,
        slots,
        active_idx,
        cfg.momentary,
        on_event,
    );

    let snap = ui.input(|i| {
        let latest_pos = i.pointer.latest_pos();
        let interact_pos = i.pointer.interact_pos();
        PointerSnapshot {
            now: i.time,
            primary_pressed: i.pointer.button_pressed(PointerButton::Primary),
            primary_down: i.pointer.primary_down(),
            primary_released: i.pointer.button_released(PointerButton::Primary),
            ctrl_down: i.modifiers.ctrl,
            latest_pos,
            hit_pos: response.interact_pointer_pos().or(interact_pos).or(latest_pos),
        }
    });

    if response.secondary_clicked() || (response.clicked() && snap.ctrl_down) {
        if let Some(idx) = snap.hit_pos.and_then(|p| find_enabled_segment(segments, slots, p)) {
            if let (Some(seg), Some(slot)) = (segments.get(idx), slots.get(idx)) {
                if seg.has_menu {
                    response.request_focus();
                    on_event(SegmentEvent {
                        index: idx,
                        menu_trigger: Some(SegmentMenuTrigger::SecondaryClick),
                        rect: slot.rect,
                    });
                    response.mark_changed();
                }
            }
        }
    }

    if snap.primary_pressed && !snap.ctrl_down {
        if let Some(pos) = snap.hit_pos {
            let unoccluded = response.is_pointer_button_down_on()
                || ui.ctx().layer_id_at(pos).is_none_or(|l| l == ui.layer_id());
            if unoccluded {
                if let Some(idx) = find_enabled_segment(segments, slots, pos) {
                    response.request_focus();
                    let st = SegPressState { seg_idx: idx, origin: pos, press_time: snap.now, menu_fired: false };
                    ui.ctx().data_mut(|d| d.insert_temp(state_id, st));
                }
            }
        }
    }

    if snap.primary_down {
        let st = ui.ctx().data(|d| d.get_temp::<SegPressState>(state_id))?;
        let (item, slot) = (segments.get(st.seg_idx)?, slots.get(st.seg_idx)?);
        if !item.enabled {
            return None;
        }
        if item.has_menu && !st.menu_fired {
            let pulled = snap.latest_pos.is_some_and(|p| (p.y - st.origin.y) >= cfg.drag_threshold);
            let elapsed = (snap.now - st.press_time).max(0.0);
            if pulled || elapsed >= cfg.hold_delay {
                let trigger = if pulled {
                    SegmentMenuTrigger::PullDown
                } else {
                    SegmentMenuTrigger::PressAndHold
                };
                let updated = SegPressState { menu_fired: true, ..st };
                ui.ctx().data_mut(|d| d.insert_temp(state_id, updated));
                on_event(SegmentEvent { index: st.seg_idx, menu_trigger: Some(trigger), rect: slot.rect });
                response.mark_changed();
                return None;
            }
            ui.ctx().request_repaint_after_secs((cfg.hold_delay - elapsed).max(0.0) as f32);
        }
        let pointer_still_inside = snap.latest_pos.or(snap.hit_pos).is_some_and(|p| slot.rect.contains(p));
        return (!st.menu_fired && pointer_still_inside).then_some(st.seg_idx);
    }

    if let Some(st) = clear_press_state_if_present(ui.ctx(), state_id) {
        let inside = slots
            .get(st.seg_idx)
            .is_some_and(|sl| snap.latest_pos.or(snap.hit_pos).is_some_and(|p| sl.rect.contains(p)));
        if snap.primary_released
            && !snap.ctrl_down
            && !st.menu_fired
            && segments.get(st.seg_idx).is_some_and(|s| s.enabled)
            && inside
        {
            if let Some(slot) = slots.get(st.seg_idx) {
                response.request_focus();
                if cfg.momentary {
                    ui.ctx().data_mut(|d| d.insert_temp(response.id.with("seg_momentary_cursor"), st.seg_idx));
                } else {
                    *active_idx = st.seg_idx;
                }
                on_event(SegmentEvent { index: st.seg_idx, menu_trigger: None, rect: slot.rect });
                response.mark_changed();
            }
        }
    } else if !snap.ctrl_down && response.clicked() {
        if let Some(idx) = snap.hit_pos.and_then(|p| find_enabled_segment(segments, slots, p)) {
            if let Some(slot) = slots.get(idx) {
                response.request_focus();
                if cfg.momentary {
                    ui.ctx().data_mut(|d| d.insert_temp(response.id.with("seg_momentary_cursor"), idx));
                } else {
                    *active_idx = idx;
                }
                on_event(SegmentEvent { index: idx, menu_trigger: None, rect: slot.rect });
                response.mark_changed();
            }
        }
    }
    None
}
