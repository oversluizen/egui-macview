//! Authentic macOS NSTextFieldCell primitive with Tab-chain and Transactional Buffer.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors, radiuses, or padding metrics.

use crate::theme::MacTheme;
use egui::{CornerRadius, Key, Modifiers, Response, Stroke, StrokeKind, Ui, Vec2};
use std::ops::RangeInclusive;

#[derive(Clone, Debug, Default)]
struct InputCellTransactionState {
    active: bool,
    snapshot: String,
    buffer: String,
}

fn decimal_places_if_valid(s: &str) -> usize {
    let t = s.trim();
    if t.parse::<f64>().is_ok() {
        t.split_once('.')
            .map_or(0, |(_, f)| f.split(['e', 'E']).next().unwrap_or("").len())
    } else {
        0
    }
}

fn format_stepped_number(reference: &str, fallback_ref: &str, step: f64, val: f64) -> String {
    let ref_decimals =
        decimal_places_if_valid(reference).max(decimal_places_if_valid(fallback_ref));
    let step_decimals = if (step - step.round()).abs() > f64::EPSILON {
        format!("{step}")
            .split_once('.')
            .map_or(1, |(_, f)| f.len())
    } else {
        0
    };
    let prec = ref_decimals.max(step_decimals);
    format!("{val:.prec$}")
}

/// Standard AppKit `NSControlSize` variants for `MacInputCell`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacControlSize {
    /// `NSControlSizeRegular` (`22pt` height, `13pt` font).
    Regular,
    /// `NSControlSizeSmall` (`19pt` height, `11pt` font).
    Small,
    /// `NSControlSizeMini` (`16pt` height, `9pt` font).
    Mini,
    /// `NSControlSizeLarge` (`28pt` height, `16pt` font).
    Large,
    /// Dense inspector / HUD infobox cell (`18pt` height, `11pt` font).
    #[default]
    CompactInfobox,
}

/// Authentic macOS compact text entry cell (AppKit `NSTextFieldCell` style).
///
/// Features transactional input buffering, Escape rollback, Return confirmation,
/// deterministic Tab-chain cyclic focus navigation, and theme-driven focus ring styling.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacInputCell, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut text = String::from("Hello macOS");
///
/// MacInputCell::new("input_demo", &mut text)
///     .theme(&theme)
///     .placeholder("Type here...")
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacInputCell<'a> {
    id: egui::Id,
    value: &'a mut String,
    theme: Option<&'a MacTheme>,
    width: Option<f32>,
    height: Option<f32>,
    font_size: Option<f32>,
    corner_radius: Option<f32>,
    control_size: MacControlSize,
    enabled: bool,
    next_focus_id: Option<egui::Id>,
    prev_focus_id: Option<egui::Id>,
    placeholder: Option<&'a str>,
    numeric_config: Option<(RangeInclusive<f64>, f64)>,
}

impl<'a> MacInputCell<'a> {
    /// Creates a new `MacInputCell` bound to `value` and identified by `id_source`.
    pub fn new(id_source: impl std::hash::Hash + std::fmt::Debug, value: &'a mut String) -> Self {
        let id = egui::Id::NULL.with(id_source);
        Self::with_id(id, value)
    }

    /// Creates a cell directly with an existing `egui::Id` without double-hashing.
    pub fn with_id(id: egui::Id, value: &'a mut String) -> Self {
        Self {
            id,
            value,
            theme: None,
            width: None,
            height: None,
            font_size: None,
            corner_radius: None,
            control_size: MacControlSize::CompactInfobox,
            enabled: true,
            next_focus_id: None,
            prev_focus_id: None,
            placeholder: None,
            numeric_config: None,
        }
    }

    /// Configures numeric arrow-key stepping (`ArrowUp`/`ArrowDown` by `±step`,
    /// `Shift` by `±10 * step`, `Cmd`/`Ctrl` by `±100 * step`, clamped to `range`).
    pub fn numeric(mut self, range: RangeInclusive<f64>, step: f64) -> Self {
        self.numeric_config = Some((range, step));
        self
    }

    /// Sets the AppKit control size preset (`Regular`, `Small`, `Mini`, `Large`, `CompactInfobox`).
    pub fn control_size(mut self, control_size: MacControlSize) -> Self {
        self.control_size = control_size;
        self
    }

    /// Enables or disables user interaction on the input cell.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Overrides cell width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides cell height in points (defaults to token text field height).
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Overrides font size in points.
    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = Some(font_size);
        self
    }

    /// Sets placeholder text displayed when the text field is empty and unfocused.
    pub fn placeholder(mut self, text: &'a str) -> Self {
        self.placeholder = Some(text);
        self
    }

    /// Sets the deterministic Tab navigation target.
    pub fn next_focus(mut self, id: egui::Id) -> Self {
        self.next_focus_id = Some(id);
        self
    }

    /// Sets the deterministic Shift+Tab navigation target.
    pub fn prev_focus(mut self, id: egui::Id) -> Self {
        self.prev_focus_id = Some(id);
        self
    }

    /// Renders the input cell into `ui` and handles keyboard/focus interaction.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = MacTheme::resolve(self.theme, ui.ctx());

        let tf_tokens = &theme.control_metrics().text_field;
        let metrics = match self.control_size {
            MacControlSize::Regular => &tf_tokens.regular,
            MacControlSize::Small => &tf_tokens.small,
            MacControlSize::Mini => &tf_tokens.mini,
            MacControlSize::Large => &tf_tokens.large,
            MacControlSize::CompactInfobox => &tf_tokens.compact_infobox,
        };

        let height = self.height.unwrap_or_else(|| metrics.height.points());
        let width = self.width.unwrap_or_else(|| metrics.default_width.points());
        let font_size = self.font_size.unwrap_or_else(|| metrics.font_size.points());
        let corner_radius = self
            .corner_radius
            .unwrap_or_else(|| metrics.corner_radius.points());

        let desired_size = Vec2::new(width, height);
        let sense = if self.enabled {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::hover()
        };
        let (rect, mut response) = ui.allocate_exact_size(desired_size, sense);

        if self.enabled && (response.clicked() || response.has_focus()) {
            ui.memory_mut(|m| m.request_focus(self.id));
        }

        let is_focused = self.enabled && ui.memory(|m| m.has_focus(self.id));
        let alpha_factor = if self.enabled {
            1.0
        } else {
            theme.disabled_opacity()
        };

        // Consolidated transactional buffer state in egui::Memory (single IdTypeMap entry)
        let tx_key = self.id.with("mac_input_tx_state");
        let (mut tx, injected_buf) = ui.data_mut(|d| {
            (
                d.get_temp::<InputCellTransactionState>(tx_key),
                d.remove_temp::<String>(self.id.with("mac_input_buffer")),
            )
        });
        let mut tx_dirty = false;
        let was_active = tx.as_ref().is_some_and(|s| s.active);

        let esc_pressed = (is_focused || was_active)
            && (ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape))
                || ui.input(|i| {
                    i.raw.events.iter().any(|e| {
                        matches!(
                            e,
                            egui::Event::Key {
                                key: Key::Escape,
                                pressed: true,
                                ..
                            }
                        )
                    })
                }));

        if is_focused && !was_active && !esc_pressed {
            let initial = self.value.clone();
            tx = Some(InputCellTransactionState {
                active: true,
                snapshot: initial.clone(),
                buffer: initial,
            });
            tx_dirty = true;
        } else if !is_focused && was_active && !esc_pressed {
            if let Some(ref mut st) = tx {
                *self.value = st.buffer.clone();
                st.active = false;
                tx_dirty = true;
            }
        }
        if let Some(override_buf) = injected_buf {
            let st = tx.get_or_insert_with(|| InputCellTransactionState {
                active: is_focused,
                snapshot: self.value.clone(),
                buffer: String::new(),
            });
            st.buffer = override_buf;
            tx_dirty = true;
        }

        // Prevent egui's built-in FocusDirection::Next / Previous from hijacking Tab/Shift+Tab
        if is_focused || was_active || self.next_focus_id.is_some() || self.prev_focus_id.is_some()
        {
            ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
        }

        // Intercept Escape, Enter, and Tab/Shift+Tab before TextEdit surrenders focus or cascades
        let mut focus_transition_to: Option<egui::Id> = None;
        let mut surrender_now = false;

        if esc_pressed {
            if let Some(ref mut st) = tx {
                *self.value = st.snapshot.clone();
                st.buffer = st.snapshot.clone();
                st.active = false;
                tx_dirty = true;
                response.mark_changed();
            }
            ui.memory_mut(|m| m.surrender_focus(self.id));
            surrender_now = true;
        } else if is_focused {
            if let Some((ref range, step)) = self.numeric_config {
                let stepper_tokens = &theme.control_metrics().stepper;
                let shift_mult = stepper_tokens.shift_multiplier.points() as f64;
                let cmd_mult = stepper_tokens.cmd_multiplier.points() as f64;
                let num_delta =
                    super::stepper::consume_step_arrow_mult(ui, Key::ArrowUp, cmd_mult, shift_mult)
                        .map(|m| step * m)
                        .or_else(|| {
                            super::stepper::consume_step_arrow_mult(
                                ui,
                                Key::ArrowDown,
                                cmd_mult,
                                shift_mult,
                            )
                            .map(|m| -step * m)
                        });
                if let Some(delta) = num_delta {
                    let cur_str = tx
                        .as_ref()
                        .map(|s| s.buffer.as_str())
                        .unwrap_or(self.value.as_str());
                    let snap_str = tx
                        .as_ref()
                        .map(|s| s.snapshot.as_str())
                        .unwrap_or(self.value.as_str());
                    let cur_val = cur_str
                        .trim()
                        .parse::<f64>()
                        .or_else(|_| snap_str.trim().parse::<f64>())
                        .unwrap_or(*range.start());
                    let next_val = (cur_val + delta).clamp(*range.start(), *range.end());
                    let formatted = format_stepped_number(cur_str, snap_str, step, next_val);
                    *self.value = formatted.clone();
                    let st = tx.get_or_insert_with(InputCellTransactionState::default);
                    st.buffer = formatted.clone();
                    st.snapshot = formatted;
                    tx_dirty = true;
                    response.mark_changed();
                }
            }

            let mut commit_and_surrender = |ui: &mut Ui, update_snapshot: bool| {
                if let Some(ref mut st) = tx {
                    *self.value = st.buffer.clone();
                    st.active = false;
                    if update_snapshot {
                        st.snapshot = self.value.clone();
                    }
                    tx_dirty = true;
                }
                ui.memory_mut(|m| m.surrender_focus(self.id));
            };

            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
                commit_and_surrender(ui, true);
                surrender_now = true;
            } else if let Some(prev) = self
                .prev_focus_id
                .filter(|_| ui.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::Tab)))
            {
                commit_and_surrender(ui, false);
                focus_transition_to = Some(prev);
            } else if let Some(next) = self
                .next_focus_id
                .filter(|_| ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Tab)))
            {
                commit_and_surrender(ui, false);
                focus_transition_to = Some(next);
            }
        }

        let painter = ui.painter_at(rect);

        // 1. Draw outer focus ring halo (NSFocusRingPlacementOutside) when focused
        if is_focused && !surrender_now {
            theme.paint_focus_ring(&painter, rect, corner_radius);
        }

        // 2. Control background and border (painted BEFORE TextEdit so text is never obscured)
        let bg_color = theme.control_background().gamma_multiply(alpha_factor);
        let border_stroke = if is_focused && !surrender_now {
            Stroke::new(metrics.border_width.points(), theme.control_accent())
        } else {
            Stroke::new(
                metrics.border_width.points(),
                theme.separator().gamma_multiply(alpha_factor),
            )
        };

        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            bg_color,
            border_stroke,
            StrokeKind::Inside,
        );

        // 3. Inner text edit bound to transactional memory buffer while active
        let shrink_pad = metrics
            .padding_horizontal
            .as_ref()
            .map(|d| d.points())
            .unwrap_or_else(|| metrics.corner_radius.points());
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((self.id, "input_cell_inner"))
                .max_rect(rect.shrink2(Vec2::new(shrink_pad, 0.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        let mut edit_buf = if is_focused && !surrender_now {
            tx.as_ref()
                .map(|s| s.buffer.clone())
                .unwrap_or_else(|| self.value.clone())
        } else {
            self.value.clone()
        };

        let mut text_edit = egui::TextEdit::singleline(&mut edit_buf)
            .id(self.id)
            .font(egui::FontId::proportional(font_size))
            .text_color(theme.label().gamma_multiply(alpha_factor))
            .desired_width((width - shrink_pad * 2.0).max(0.0))
            .frame(egui::Frame::NONE)
            .lock_focus(true)
            .interactive(self.enabled);

        if let Some(hint) = self.placeholder {
            text_edit = text_edit.hint_text(hint);
        }

        let text_res = child_ui.add(text_edit);
        if text_res.changed() && !surrender_now {
            *self.value = edit_buf.clone();
            let st = tx.get_or_insert_with(InputCellTransactionState::default);
            st.buffer = edit_buf;
            tx_dirty = true;
            response.mark_changed();
        }

        if is_focused && !surrender_now {
            // Consume single-character canvas keys and editing keys so they don't leak after TextEdit processes them
            ui.input_mut(|i| {
                let _ = i.consume_key(Modifiers::NONE, Key::Backspace);
                let _ = i.consume_key(Modifiers::NONE, Key::Delete);
                let _ = i.consume_key(Modifiers::NONE, Key::Space);
                let _ = i.consume_key(Modifiers::NONE, Key::ArrowLeft);
                let _ = i.consume_key(Modifiers::NONE, Key::ArrowRight);
                if !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.mac_cmd {
                    i.keys_down.clear();
                }
                i.events.retain(|ev| match ev {
                    egui::Event::Key { modifiers, .. } => {
                        modifiers.command || modifiers.ctrl || modifiers.mac_cmd
                    }
                    _ => true,
                });
            });
        }

        let post_focused = ui.memory(|m| m.has_focus(self.id));
        if is_focused && !post_focused && !surrender_now && focus_transition_to.is_none() {
            // Focus lost via outside click (blur) -> commit transactional buffer
            if let Some(ref mut st) = tx {
                *self.value = st.buffer.clone();
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
            ui.memory_mut(|m| m.surrender_focus(self.id));
        } else if let Some(target_id) = focus_transition_to {
            ui.memory_mut(|m| {
                m.move_focus(egui::FocusDirection::None);
                m.request_focus(target_id);
            });
        } else if post_focused {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    self.id,
                    egui::EventFilter {
                        tab: true,
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                    },
                );
            });
        }

        text_res.union(response).with_new_rect(rect)
    }
}
