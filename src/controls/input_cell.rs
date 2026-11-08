//! Authentic macOS NSTextFieldCell primitive with Tab-chain and Transactional Buffer.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors, radiuses, or padding metrics.

use crate::theme::MacTheme;
use egui::{Color32, CornerRadius, Key, Modifiers, Response, Stroke, StrokeKind, Ui, Vec2};
use std::ops::RangeInclusive;

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
    let ref_decimals = decimal_places_if_valid(reference).max(decimal_places_if_valid(fallback_ref));
    let step_decimals = if (step - step.round()).abs() > f64::EPSILON {
        format!("{step}").split_once('.').map_or(1, |(_, f)| f.len())
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
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::from_ctx(ui.ctx());
                &fallback_theme
            }
        };

        let tf_tokens = &theme.control_metrics().text_field;
        let metrics = match self.control_size {
            MacControlSize::Regular => &tf_tokens.regular,
            MacControlSize::Small => &tf_tokens.small,
            MacControlSize::Mini => &tf_tokens.mini,
            MacControlSize::Large => &tf_tokens.large,
            MacControlSize::CompactInfobox => &tf_tokens.compact_infobox,
        };
        let ring_metrics = theme.focus_ring();

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

        // Transactional buffer state management in egui::Memory
        let active_key = self.id.with("mac_input_editing_active");
        let snapshot_key = self.id.with("mac_input_snapshot");
        let buffer_key = self.id.with("mac_input_buffer");
        let was_active = ui.data(|d| d.get_temp::<bool>(active_key).unwrap_or(false));

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
            ui.data_mut(|d| {
                d.insert_temp(active_key, true);
                d.insert_temp(snapshot_key, initial.clone());
                d.insert_temp(buffer_key, initial);
            });
        } else if !is_focused && was_active && !esc_pressed {
            if let Some(buf) = ui.data(|d| d.get_temp::<String>(buffer_key)) {
                *self.value = buf;
            }
            ui.data_mut(|d| {
                d.insert_temp(active_key, false);
            });
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
            if let Some(snapshot) = ui.data(|d| d.get_temp::<String>(snapshot_key)) {
                *self.value = snapshot.clone();
                ui.data_mut(|d| d.insert_temp(buffer_key, snapshot));
                response.mark_changed();
            }
            ui.data_mut(|d| d.insert_temp(active_key, false));
            ui.memory_mut(|m| m.surrender_focus(self.id));
            surrender_now = true;
        } else if is_focused {
            if let Some((ref range, step)) = self.numeric_config {
                let stepper_tokens = &theme.control_metrics().stepper;
                let shift_mult = stepper_tokens.shift_multiplier.points() as f64;
                let cmd_mult = stepper_tokens.cmd_multiplier.points() as f64;
                let num_delta = ui.input_mut(|i| {
                    let mut check_dir = |key: Key| -> Option<f64> {
                        if i.consume_key(Modifiers::COMMAND, key)
                            || i.consume_key(Modifiers::CTRL, key)
                            || i.consume_key(Modifiers::MAC_CMD, key)
                        {
                            Some(cmd_mult)
                        } else if i.consume_key(Modifiers::SHIFT, key) {
                            Some(shift_mult)
                        } else if i.consume_key(Modifiers::NONE, key) {
                            Some(1.0)
                        } else {
                            None
                        }
                    };
                    if let Some(m) = check_dir(Key::ArrowUp) {
                        Some(step * m)
                    } else {
                        check_dir(Key::ArrowDown).map(|m| -step * m)
                    }
                });
                if let Some(delta) = num_delta {
                    let cur_str = ui
                        .data(|d| d.get_temp::<String>(buffer_key))
                        .unwrap_or_else(|| self.value.clone());
                    let snap_str = ui
                        .data(|d| d.get_temp::<String>(snapshot_key))
                        .unwrap_or_else(|| self.value.clone());
                    let cur_val = cur_str
                        .trim()
                        .parse::<f64>()
                        .or_else(|_| snap_str.trim().parse::<f64>())
                        .unwrap_or(*range.start());
                    let next_val = (cur_val + delta).clamp(*range.start(), *range.end());
                    let formatted = format_stepped_number(&cur_str, &snap_str, step, next_val);
                    *self.value = formatted.clone();
                    ui.data_mut(|d| {
                        d.insert_temp(buffer_key, formatted.clone());
                        d.insert_temp(snapshot_key, formatted);
                    });
                    response.mark_changed();
                }
            }
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
                if let Some(buf) = ui.data(|d| d.get_temp::<String>(buffer_key)) {
                    *self.value = buf;
                }
                ui.data_mut(|d| {
                    d.insert_temp(active_key, false);
                    d.insert_temp(snapshot_key, self.value.clone());
                });
                ui.memory_mut(|m| m.surrender_focus(self.id));
                surrender_now = true;
            } else if let Some(prev) = self.prev_focus_id {
                if ui.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::Tab)) {
                    if let Some(buf) = ui.data(|d| d.get_temp::<String>(buffer_key)) {
                        *self.value = buf;
                    }
                    ui.data_mut(|d| d.insert_temp(active_key, false));
                    ui.memory_mut(|m| m.surrender_focus(self.id));
                    focus_transition_to = Some(prev);
                } else if let Some(next) = self.next_focus_id {
                    if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Tab)) {
                        if let Some(buf) = ui.data(|d| d.get_temp::<String>(buffer_key)) {
                            *self.value = buf;
                        }
                        ui.data_mut(|d| d.insert_temp(active_key, false));
                        ui.memory_mut(|m| m.surrender_focus(self.id));
                        focus_transition_to = Some(next);
                    }
                }
            } else if let Some(next) = self.next_focus_id {
                if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Tab)) {
                    if let Some(buf) = ui.data(|d| d.get_temp::<String>(buffer_key)) {
                        *self.value = buf;
                    }
                    ui.data_mut(|d| d.insert_temp(active_key, false));
                    ui.memory_mut(|m| m.surrender_focus(self.id));
                    focus_transition_to = Some(next);
                }
            }
        }

        let painter = ui.painter_at(rect);

        // 1. Draw outer focus ring halo (NSFocusRingPlacementOutside) when focused
        if is_focused && !surrender_now {
            let halo_offset = ring_metrics.offset.points();
            let halo_stroke_width = ring_metrics.stroke_width.points();
            let halo_radius = corner_radius + ring_metrics.corner_radius_delta.points();
            let halo_rect = rect.expand(halo_offset);

            painter.rect(
                halo_rect,
                CornerRadius::same(halo_radius as u8),
                Color32::TRANSPARENT,
                Stroke::new(halo_stroke_width, theme.keyboard_focus_indicator()),
                StrokeKind::Outside,
            );
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
                .max_rect(rect.shrink2(Vec2::new(shrink_pad, 0.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        let mut edit_buf = if is_focused && !surrender_now {
            ui.data(|d| d.get_temp::<String>(buffer_key))
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
            ui.data_mut(|d| d.insert_temp(buffer_key, edit_buf));
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
            if let Some(buf) = ui.data(|d| d.get_temp::<String>(buffer_key)) {
                *self.value = buf;
            }
            ui.data_mut(|d| d.insert_temp(active_key, false));
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
