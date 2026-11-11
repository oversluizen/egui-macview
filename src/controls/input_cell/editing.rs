//! Transactional editing, keyboard stepping, and baseline rendering for [`super::MacInputCell`].

use super::{MacControlSize, MacInputCell};
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{Align, CornerRadius, FontId, Key, Modifiers, Pos2, Response, Stroke, StrokeKind, Ui, Vec2};

#[derive(Clone, Debug, Default)]
struct InputCellTransactionState {
    active: bool,
    snapshot: String,
    buffer: String,
}

fn decimal_places_if_valid(s: &str) -> usize {
    let t = s.trim();
    if t.parse::<f64>().is_ok() {
        t.split_once('.').map_or(0, |(_, f)| f.split(['e', 'E']).next().unwrap_or("").len())
    } else {
        0
    }
}

fn format_stepped_number(reference: &str, fallback_ref: &str, step: f64, val: f64) -> String {
    let ref_decimals = decimal_places_if_valid(reference).max(decimal_places_if_valid(fallback_ref));
    let step_decimals = if (step - step.round()).abs() > f64::EPSILON {
        format!("{step}").split_once('.').map_or(1, |(_, f)| f.len())
    } else {
        0
    };
    let prec = ref_decimals.max(step_decimals);
    format!("{val:.prec$}")
}

pub(super) fn show_input_cell(cell: MacInputCell<'_>, ui: &mut Ui) -> Response {
    let resolved_size = cell.resolved_control_size();
    let theme = MacTheme::resolve(cell.theme, ui.ctx());
    let tf_tokens = &theme.control_metrics().text_field;
    let metrics = match resolved_size {
        MacControlSize::Regular => &tf_tokens.regular,
        MacControlSize::Small => &tf_tokens.small,
        MacControlSize::Mini => &tf_tokens.mini,
        MacControlSize::Large => &tf_tokens.large,
        MacControlSize::CompactInfobox => &tf_tokens.compact_infobox,
    };

    let height = cell.fixed_rect.map(|r| r.height()).or(cell.height).unwrap_or_else(|| metrics.height.points());
    let width = cell.fixed_rect.map(|r| r.width()).or(cell.width).unwrap_or_else(|| metrics.default_width.points());
    let font_size = cell.font_size.unwrap_or_else(|| metrics.font_size.points());
    let corner_radius = cell.corner_radius.unwrap_or_else(|| metrics.corner_radius.points());
    let enabled = cell.enabled && ui.is_enabled();
    let sense = if enabled { egui::Sense::click_and_drag() } else { egui::Sense::hover() };
    let ppp = ui.ctx().pixels_per_point();
    let (rect, mut response) = if let Some(fixed) = cell.fixed_rect {
        let snapped = subpixel_snap_rect(fixed, ppp);
        (snapped, ui.interact(snapped, cell.id.with("fixed_hit"), sense))
    } else {
        let (r, resp) = ui.allocate_exact_size(Vec2::new(width, height), sense);
        (subpixel_snap_rect(r, ppp), resp)
    };

    if enabled && (response.clicked() || response.has_focus()) {
        ui.memory_mut(|m| m.request_focus(cell.id));
    }

    let is_focused = enabled && ui.memory(|m| m.has_focus(cell.id));
    let alpha_factor = if enabled { 1.0 } else { theme.disabled_opacity() };
    let tx_key = cell.id.with("mac_input_tx_state");
    let (mut tx, injected_buf) = ui.data_mut(|d| {
        (d.get_temp::<InputCellTransactionState>(tx_key), d.remove_temp::<String>(cell.id.with("mac_input_buffer")))
    });
    let mut tx_dirty = false;
    let was_active = tx.as_ref().is_some_and(|s| s.active);

    let esc_pressed = (is_focused || was_active)
        && (ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape))
            || ui.input(|i| i.raw.events.iter().any(|e| matches!(e, egui::Event::Key { key: Key::Escape, pressed: true, .. }))));

    if is_focused && !was_active && !esc_pressed {
        let initial = cell.value.clone();
        tx = Some(InputCellTransactionState { active: true, snapshot: initial.clone(), buffer: initial });
        tx_dirty = true;
    } else if !is_focused && was_active && !esc_pressed {
        if let Some(ref mut st) = tx {
            *cell.value = st.buffer.clone();
            st.active = false;
            tx_dirty = true;
        }
    }
    if let Some(override_buf) = injected_buf {
        let st = tx.get_or_insert_with(|| InputCellTransactionState { active: is_focused, snapshot: cell.value.clone(), buffer: String::new() });
        st.buffer = override_buf;
        tx_dirty = true;
    }

    if is_focused || was_active || cell.next_focus_id.is_some() || cell.prev_focus_id.is_some() {
        ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
    }

    let mut focus_transition_to: Option<egui::Id> = None;
    let mut surrender_now = false;

    if esc_pressed {
        if let Some(ref mut st) = tx {
            *cell.value = st.snapshot.clone();
            st.buffer = st.snapshot.clone();
            st.active = false;
            tx_dirty = true;
            response.mark_changed();
        }
        ui.memory_mut(|m| m.surrender_focus(cell.id));
        surrender_now = true;
    } else if is_focused {
        if let Some((ref range, step)) = cell.numeric_config {
            let stepper_tokens = &theme.control_metrics().stepper;
            let shift_mult = stepper_tokens.shift_multiplier.points() as f64;
            let cmd_mult = stepper_tokens.cmd_multiplier.points() as f64;
            let num_delta = super::super::stepper::consume_step_arrow_mult(ui, Key::ArrowUp, cmd_mult, shift_mult)
                .map(|m| step * m)
                .or_else(|| super::super::stepper::consume_step_arrow_mult(ui, Key::ArrowDown, cmd_mult, shift_mult).map(|m| -step * m));
            if let Some(delta) = num_delta {
                let cur_str = tx.as_ref().map(|s| s.buffer.as_str()).unwrap_or(cell.value.as_str());
                let snap_str = tx.as_ref().map(|s| s.snapshot.as_str()).unwrap_or(cell.value.as_str());
                let cur_val = cur_str.trim().parse::<f64>().or_else(|_| snap_str.trim().parse::<f64>()).unwrap_or(*range.start());
                let next_val = (cur_val + delta).clamp(*range.start(), *range.end());
                let formatted = format_stepped_number(cur_str, snap_str, step, next_val);
                *cell.value = formatted.clone();
                let st = tx.get_or_insert_with(InputCellTransactionState::default);
                st.buffer = formatted.clone();
                st.snapshot = formatted;
                tx_dirty = true;
                response.mark_changed();
            }
        }

        let mut commit_and_surrender = |ui: &mut Ui, update_snapshot: bool| {
            if let Some(ref mut st) = tx {
                *cell.value = st.buffer.clone();
                st.active = false;
                if update_snapshot {
                    st.snapshot = cell.value.clone();
                }
                tx_dirty = true;
            }
            ui.memory_mut(|m| m.surrender_focus(cell.id));
        };

        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
            commit_and_surrender(ui, true);
            surrender_now = true;
        } else if let Some(prev) = cell.prev_focus_id.filter(|_| ui.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::Tab))) {
            commit_and_surrender(ui, false);
            focus_transition_to = Some(prev);
        } else if let Some(next) = cell.next_focus_id.filter(|_| ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Tab))) {
            commit_and_surrender(ui, false);
            focus_transition_to = Some(next);
        }
    }

    let painter = ui.painter_at(rect);
    if is_focused && !surrender_now {
        theme.paint_focus_ring(&painter, rect, corner_radius);
    }

    let bg_color = theme.control_background().gamma_multiply(alpha_factor);
    let border_stroke = if is_focused && !surrender_now {
        Stroke::new(metrics.border_width.points(), theme.control_accent())
    } else {
        Stroke::new(metrics.border_width.points(), theme.separator().gamma_multiply(alpha_factor))
    };
    painter.rect(rect, CornerRadius::same(corner_radius as u8), bg_color, border_stroke, StrokeKind::Inside);

    let shrink_pad = metrics.padding_horizontal.points();
    let inner_rect = rect.shrink2(Vec2::new(shrink_pad, 0.0));
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .id_salt((cell.id, "input_cell_inner"))
            .max_rect(inner_rect)
            .layout(egui::Layout::left_to_right(Align::Center)),
    );

    let mut edit_buf = if is_focused && !surrender_now {
        tx.as_ref().map(|s| s.buffer.clone()).unwrap_or_else(|| cell.value.clone())
    } else {
        cell.value.clone()
    };

    let font_id = FontId::proportional(font_size);
    let text_color = theme.label().gamma_multiply(alpha_factor);
    let hint_color = theme.placeholder_text().gamma_multiply(alpha_factor);

    if !(is_focused && !surrender_now) {
        let display_str = if cell.value.is_empty() { cell.placeholder.unwrap_or("") } else { cell.value.as_str() };
        if !display_str.is_empty() {
            let draw_color = if cell.value.is_empty() { hint_color } else { text_color };
            let galley = theme.layout_tracked_galley(ui, display_str, font_id.clone(), draw_color, inner_rect.width().max(0.0));
            let lm = theme.label_metrics();
            let ascender = match resolved_size {
                MacControlSize::Regular | MacControlSize::Large => lm.regular_ascender.points(),
                MacControlSize::Small | MacControlSize::CompactInfobox => lm.small_ascender.points(),
                MacControlSize::Mini => lm.mini_ascender.points(),
            };
            let text_y = theme.baseline_text_y(rect, metrics.first_baseline_offset_from_top.points(), ascender, ppp);
            let text_x = match cell.text_align {
                Align::Max => (inner_rect.right() - galley.size().x).max(inner_rect.left()),
                Align::Center => inner_rect.center().x - galley.size().x * 0.5,
                Align::Min => inner_rect.left(),
            };
            painter.with_clip_rect(inner_rect).galley(Pos2::new(text_x, text_y), galley, draw_color);
        }
    }

    let theme_ref = &theme;
    let layout_font = font_id.clone();
    let mut layouter = |ui: &Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
        theme_ref.layout_tracked_galley(ui, buf.as_str(), layout_font.clone(), text_color, wrap_width)
    };

    let text_res = if is_focused && !surrender_now {
        let mut text_edit = egui::TextEdit::singleline(&mut edit_buf)
            .id(cell.id)
            .font(font_id)
            .align(egui::Align2([cell.text_align, Align::Center]))
            .text_color(text_color)
            .desired_width((width - shrink_pad * 2.0).max(0.0))
            .frame(egui::Frame::NONE)
            .lock_focus(true)
            .interactive(enabled)
            .layouter(&mut layouter);
        if let Some(hint) = cell.placeholder {
            text_edit = text_edit.hint_text(hint);
        }
        let r = child_ui.add(text_edit);
        if r.changed() {
            *cell.value = edit_buf.clone();
            let st = tx.get_or_insert_with(InputCellTransactionState::default);
            st.buffer = edit_buf;
            tx_dirty = true;
            response.mark_changed();
        }
        r
    } else {
        egui::text_edit::TextEditState::load(ui.ctx(), cell.id)
            .unwrap_or_default()
            .store(ui.ctx(), cell.id);
        child_ui.interact(inner_rect, cell.id, sense)
    };

    if is_focused && !surrender_now {
        ui.input_mut(|i| {
            for k in [Key::Backspace, Key::Delete, Key::Space, Key::ArrowLeft, Key::ArrowRight] {
                let _ = i.consume_key(Modifiers::NONE, k);
            }
            if !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.mac_cmd {
                i.keys_down.clear();
            }
            i.events.retain(|ev| match ev {
                egui::Event::Key { modifiers, .. } => modifiers.command || modifiers.ctrl || modifiers.mac_cmd,
                _ => true,
            });
        });
    }

    let post_focused = ui.memory(|m| m.has_focus(cell.id));
    if is_focused && !post_focused && !surrender_now && focus_transition_to.is_none() {
        if let Some(ref mut st) = tx {
            *cell.value = st.buffer.clone();
            st.active = false;
            tx_dirty = true;
        }
    }
    if tx_dirty {
        if let Some(st) = tx {
            ui.data_mut(|d| d.insert_temp(tx_key, st));
        }
    }
    if surrender_now {
        ui.memory_mut(|m| m.surrender_focus(cell.id));
    } else if let Some(target_id) = focus_transition_to {
        ui.memory_mut(|m| {
            m.move_focus(egui::FocusDirection::None);
            m.request_focus(target_id);
        });
    } else if post_focused {
        ui.memory_mut(|m| {
            m.move_focus(egui::FocusDirection::None);
            m.set_focus_lock_filter(cell.id, egui::EventFilter { tab: true, horizontal_arrows: true, vertical_arrows: true, escape: true });
        });
    }
    response | text_res
}
