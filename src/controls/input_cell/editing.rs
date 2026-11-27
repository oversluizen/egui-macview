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
    focus_ring_target: f32,
    focus_ring_prev: f32,
    focus_ring_start: f64,
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
    let step_decimals =
        if (step - step.round()).abs() > f64::EPSILON { format!("{step}").split_once('.').map_or(1, |(_, f)| f.len()) } else { 0 };
    let prec = ref_decimals.max(step_decimals);
    format!("{val:.prec$}")
}

pub(super) fn show_input_cell(cell: MacInputCell<'_>, ui: &mut Ui) -> Response {
    let put_rect = (ui.layout().main_justify && ui.layout().cross_justify).then(|| ui.max_rect());
    let effective_fixed = cell.fixed_rect.or(put_rect);
    let theme = MacTheme::resolve(cell.theme, ui.ctx());
    let tf_tokens = &theme.control_metrics().text_field;
    let max_r = ui.max_rect();
    let candidate_h = effective_fixed.map(|r| r.height()).or(cell.height).or_else(|| {
        (max_r.height().is_finite() && max_r.height() < tf_tokens.auto_size_regular_min_height.points()).then(|| max_r.height())
    });
    let mut resolved_size = cell.resolved_control_size();
    if let Some(h) = candidate_h {
        if !cell.explicit_control_size && h < tf_tokens.auto_size_regular_min_height.points() && resolved_size == MacControlSize::Regular {
            resolved_size = if h < tf_tokens.auto_size_small_min_height.points() {
                if h >= tf_tokens.auto_size_compact_min_height.points() {
                    MacControlSize::CompactInfobox
                } else {
                    MacControlSize::Mini
                }
            } else {
                MacControlSize::Small
            };
        }
    }
    let metrics = match resolved_size {
        MacControlSize::Regular => &tf_tokens.regular,
        MacControlSize::Small => &tf_tokens.small,
        MacControlSize::Mini => &tf_tokens.mini,
        MacControlSize::Large => &tf_tokens.large,
        MacControlSize::CompactInfobox => &tf_tokens.compact_infobox,
    };

    let height = candidate_h.unwrap_or_else(|| metrics.height.points());
    let compact_w_limit = tf_tokens.compact_infobox.default_width.points();
    let width = effective_fixed
        .map(|r| r.width())
        .or(cell.width)
        .or_else(|| (max_r.width().is_finite() && max_r.width() <= compact_w_limit).then(|| max_r.width()))
        .unwrap_or_else(|| metrics.default_width.points());
    let font_size = cell.font_size.unwrap_or_else(|| {
        if height <= tf_tokens.mini.height.points() + 0.5 && resolved_size != MacControlSize::Mini {
            metrics.font_size.points() - 0.5
        } else {
            metrics.font_size.points()
        }
    });
    let corner_radius = cell.corner_radius.unwrap_or_else(|| {
        if cell.recessed_pill {
            metrics.baseline_offset_from_bottom.points()
        } else {
            metrics.corner_radius.points()
        }
    });
    let enabled = cell.enabled && ui.is_enabled();
    let sense = if enabled { egui::Sense::click_and_drag() } else { egui::Sense::hover() };
    let ppp = ui.ctx().pixels_per_point();
    let (rect, mut response) = if let Some(fixed) = effective_fixed {
        let snapped = subpixel_snap_rect(fixed, ppp);
        (snapped, ui.interact(snapped, cell.id.with("fixed_hit"), sense))
    } else {
        let (r, resp) = ui.allocate_exact_size(Vec2::new(width, height), sense);
        (subpixel_snap_rect(r, ppp), resp)
    };

    let pointer_in_rect = enabled
        && ui.is_rect_visible(rect)
        && ui.input(|i| {
            (i.pointer.primary_pressed() || i.pointer.primary_clicked())
                && i.pointer.interact_pos().or_else(|| i.pointer.latest_pos()).is_some_and(|p| rect.contains(p))
        });
    if enabled && (response.clicked() || response.has_focus() || pointer_in_rect) {
        ui.memory_mut(|m| m.request_focus(cell.id));
    }

    let is_focused = enabled && ui.memory(|m| m.has_focus(cell.id));
    let alpha_factor = if enabled { 1.0 } else { theme.disabled_opacity() };
    let tx_key = cell.id.with("mac_input_tx_state");
    let (mut tx, injected_buf) =
        ui.data_mut(|d| (d.get_temp::<InputCellTransactionState>(tx_key), d.remove_temp::<String>(cell.id.with("mac_input_buffer"))));
    let mut tx_dirty = false;
    let was_active = tx.as_ref().is_some_and(|s| s.active);

    let esc_pressed = (is_focused || was_active)
        && (ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape))
            || ui.input(|i| i.raw.events.iter().any(|e| matches!(e, egui::Event::Key { key: Key::Escape, pressed: true, .. }))));

    if is_focused && !was_active && !esc_pressed {
        let initial = cell.value.clone();
        tx = Some(InputCellTransactionState {
            active: true,
            snapshot: initial.clone(),
            buffer: initial,
            focus_ring_target: 0.0,
            focus_ring_prev: 0.0,
            focus_ring_start: ui.input(|i| i.time),
        });
        tx_dirty = true;
    } else if is_focused && was_active && !esc_pressed {
        if let Some(ref mut st) = tx {
            if st.buffer == st.snapshot && *cell.value != st.snapshot {
                st.snapshot = cell.value.clone();
                st.buffer = cell.value.clone();
                tx_dirty = true;
            }
        }
    } else if !is_focused && was_active && !esc_pressed {
        if let Some(ref mut st) = tx {
            if st.buffer != st.snapshot {
                *cell.value = st.buffer.clone();
            }
            st.active = false;
            tx_dirty = true;
        }
    }
    if let Some(override_buf) = injected_buf {
        let st = tx.get_or_insert_with(|| InputCellTransactionState {
            active: is_focused,
            snapshot: cell.value.clone(),
            buffer: String::new(),
            ..Default::default()
        });
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
    let show_focus = is_focused && !surrender_now;
    let target_f = if show_focus { 1.0f32 } else { 0.0f32 };
    if let Some(ref mut st) = tx {
        let now = ui.input(|i| i.time);
        let base_s = theme.motion().swiftui_snappy.to_spring();
        let focus_s = crate::motion::CASpring::new(base_s.response * 0.5, base_s.damping_ratio);
        if (st.focus_ring_target - target_f).abs() > f32::EPSILON {
            let dt0 = (now - st.focus_ring_start).max(0.0) as f32;
            st.focus_ring_prev = st.focus_ring_prev + (st.focus_ring_target - st.focus_ring_prev) * focus_s.position_at(dt0);
            st.focus_ring_target = target_f;
            st.focus_ring_start = now;
            tx_dirty = true;
        }
        let dt = (now - st.focus_ring_start).max(0.0) as f32;
        let prog = (st.focus_ring_prev + (st.focus_ring_target - st.focus_ring_prev) * focus_s.position_at(dt)).clamp(0.0, 1.0);
        if (st.focus_ring_target - st.focus_ring_prev).abs() > f32::EPSILON && dt < focus_s.settling_duration() {
            ui.ctx().request_repaint();
        }
        if prog > 0.0 {
            let ring = theme.focus_ring();
            painter.rect_stroke(
                rect.expand(ring.offset.points() * prog),
                CornerRadius::same((corner_radius + ring.corner_radius_delta.points() * prog) as u8),
                Stroke::new(ring.stroke_width.points() * prog, theme.keyboard_focus_indicator().gamma_multiply(prog)),
                StrokeKind::Outside,
            );
        }
    } else if show_focus {
        theme.paint_focus_ring(&painter, rect, corner_radius);
    }

    let surface_bg = theme.active_surface_at(ui, rect.center());
    let bg_color = if cell.liquid_glass_pill {
        theme.effective_control_accent(ui.ctx()).gamma_multiply(alpha_factor)
    } else if cell.recessed_pill && !show_focus {
        theme.recessed_pill_fill_on(surface_bg).gamma_multiply(alpha_factor)
    } else {
        theme.control_background().gamma_multiply(alpha_factor)
    };
    let border_stroke = if show_focus || cell.liquid_glass_pill {
        Stroke::new(metrics.border_width.points(), theme.effective_control_accent(ui.ctx()))
    } else if cell.recessed_pill {
        Stroke::new(metrics.border_width.points(), theme.recessed_pill_border_on(bg_color))
    } else {
        Stroke::new(metrics.border_width.points(), theme.separator().gamma_multiply(alpha_factor))
    };
    if cell.liquid_glass_pill {
        let pill_r = (height * 0.5).max(corner_radius);
        super::super::liquid_glass::paint_liquid_glass_capsule(
            &painter,
            rect,
            pill_r,
            bg_color,
            border_stroke,
            theme.specular_rim_top().gamma_multiply(alpha_factor),
        );
    } else {
        painter.rect(rect, CornerRadius::same(corner_radius as u8), bg_color, border_stroke, StrokeKind::Inside);
        if !cell.recessed_pill {
            painter.line_segment(
                [Pos2::new(rect.min.x + 1.0, rect.min.y + 1.0), Pos2::new(rect.max.x - 1.0, rect.min.y + 1.0)],
                Stroke::new(1.0, theme.input_top_inset_shadow().gamma_multiply(alpha_factor)),
            );
        }
    }

    let shrink_pad = if effective_fixed.is_some()
        || height < tf_tokens.auto_size_regular_min_height.points()
        || width <= compact_w_limit
        || matches!(resolved_size, MacControlSize::Small | MacControlSize::Mini | MacControlSize::CompactInfobox)
    {
        tf_tokens.regular.corner_radius.points()
    } else {
        metrics.padding_horizontal.points()
    };
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

    let is_heavy = matches!(
        cell.font_weight,
        crate::controls::MacFontWeight::Medium | crate::controls::MacFontWeight::Semibold | crate::controls::MacFontWeight::Bold
    );
    let (font_id, has_semibold_family) = if cell.monospace {
        (FontId::monospace(font_size), false)
    } else if cell.stylistic_alt {
        (theme.stylistic_alt_font_id(ui, font_size), false)
    } else if is_heavy {
        theme.semibold_font_id(ui, font_size)
    } else if cell.font_weight == crate::controls::MacFontWeight::Light {
        (theme.light_font_id(ui, font_size), false)
    } else {
        (FontId::proportional(font_size), false)
    };
    let text_color = if cell.liquid_glass_pill {
        theme.background_style(bg_color).primary_text_color(&theme).gamma_multiply(alpha_factor)
    } else {
        theme.label().gamma_multiply(alpha_factor)
    };
    let hint_color = theme.placeholder_text().gamma_multiply(alpha_factor);

    if !(is_focused && !surrender_now) {
        let display_str = if cell.value.is_empty() { cell.placeholder.unwrap_or("") } else { cell.value.as_str() };
        if !display_str.is_empty() {
            let draw_color = if cell.value.is_empty() { hint_color } else { text_color };
            let lm = theme.label_metrics();
            let galley = if let Some(split_idx) = (!cell.value.is_empty()
                && (display_str.starts_with('=') || cell.numeric_config.is_some()))
            .then(|| display_str.rfind(" ("))
            .flatten()
            .filter(|&i| i > 0 && display_str.ends_with(')') && display_str[i + 2..display_str.len() - 1].trim().parse::<f64>().is_ok())
            {
                let (prefix, suffix) = display_str.split_at(split_idx);
                let sec_color = theme.secondary_label().gamma_multiply(alpha_factor);
                let sec_font = FontId::proportional(lm.small_font_size.points());
                let mut job = egui::text::LayoutJob::default();
                job.wrap.max_width = f32::INFINITY;
                job.append(prefix, 0.0, theme.tracked_text_format(font_id.clone(), draw_color));
                job.append(suffix, 0.0, theme.tracked_text_format(sec_font, sec_color));
                ui.painter().layout_job(job)
            } else {
                theme.layout_tracked_galley(ui, display_str, font_id.clone(), draw_color, f32::INFINITY)
            };
            let ascender = match resolved_size {
                MacControlSize::Regular | MacControlSize::Large => lm.regular_ascender.points(),
                MacControlSize::Small | MacControlSize::CompactInfobox => lm.small_ascender.points(),
                MacControlSize::Mini => lm.mini_ascender.points(),
            };
            let eff_baseline_from_top =
                metrics.first_baseline_offset_from_top.points() - (metrics.height.points() - rect.height()).max(0.0) * 0.5;
            let text_y = theme.baseline_text_y(rect, eff_baseline_from_top, ascender, ppp);
            let text_x = match cell.text_align {
                Align::Max => (inner_rect.right() - galley.size().x).max(inner_rect.left()),
                Align::Center => inner_rect.center().x - galley.size().x * 0.5,
                Align::Min => inner_rect.left(),
            };
            let clip_p = painter.with_clip_rect(inner_rect);
            clip_p.galley(Pos2::new(text_x, text_y), galley.clone(), draw_color);
            if is_heavy && !has_semibold_family {
                let boost = match cell.font_weight {
                    crate::controls::MacFontWeight::Bold => lm.bold_stroke_boost.points(),
                    crate::controls::MacFontWeight::Medium | crate::controls::MacFontWeight::Semibold => lm.semibold_stroke_boost.points(),
                    crate::controls::MacFontWeight::Light | crate::controls::MacFontWeight::Regular => 0.0,
                };
                if boost > 0.0 {
                    clip_p.galley(Pos2::new(text_x + boost, text_y), galley, draw_color);
                }
            }
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
        egui::text_edit::TextEditState::load(ui.ctx(), cell.id).unwrap_or_default().store(ui.ctx(), cell.id);
        let r = child_ui.interact(inner_rect, cell.id, sense);
        if enabled && (r.clicked() || r.is_pointer_button_down_on() || r.has_focus()) {
            ui.memory_mut(|m| m.request_focus(cell.id));
        }
        r
    };

    if is_focused && !surrender_now && pointer_in_rect {
        ui.memory_mut(|m| m.request_focus(cell.id));
    }

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
            if st.buffer != st.snapshot {
                *cell.value = st.buffer.clone();
            }
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
