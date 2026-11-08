//! Authentic macOS AppKit Stepper Control (NSStepper equivalent).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors or dimensions.

use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{CornerRadius, Key, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget};
use std::ops::RangeInclusive;

/// Authentic macOS AppKit Stepper Control (`NSStepper` equivalent).
///
/// Features authentic dual stacked increment/decrement arrow chevrons,
/// continuous auto-repeat when held down, keyboard arrow support, and optional value wrap-around.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacStepper, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut count = 5.0f64;
///
/// MacStepper::new(&mut count, 0.0..=100.0, 1.0)
///     .theme(&theme)
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacStepper<'a> {
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    step: f64,
    theme: Option<&'a MacTheme>,
    width: Option<f32>,
    height: Option<f32>,
    corner_radius: Option<f32>,
    wraps: bool,
    enabled: bool,
}

impl<'a> MacStepper<'a> {
    /// Creates a new stepper bound to a value, inclusive range, and step delta.
    pub fn new(value: &'a mut f64, range: RangeInclusive<f64>, step: f64) -> Self {
        Self {
            value,
            range,
            step,
            theme: None,
            width: None,
            height: None,
            corner_radius: None,
            wraps: false,
            enabled: true,
        }
    }

    /// Sets the dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Overrides default width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides default height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Overrides default corner radius.
    pub fn corner_radius(mut self, r: f32) -> Self {
        self.corner_radius = Some(r);
        self
    }

    /// Configures wrap-around at range boundaries.
    pub fn wraps(mut self, wraps: bool) -> Self {
        self.wraps = wraps;
        self
    }

    /// Enables or disables the control.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Renders the stepper inside the specified UI container.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacStepper<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::from_ctx(ui.ctx());
                &fallback_theme
            }
        };

        let metrics = theme.stepper();
        let width = self.width.unwrap_or_else(|| metrics.width.points());
        let height = self.height.unwrap_or_else(|| metrics.height.points());
        let corner_radius = self
            .corner_radius
            .unwrap_or_else(|| metrics.corner_radius.points());
        let divider_thickness = metrics.divider_thickness.points();
        let arrow_size = metrics.arrow_size.points();
        let arrow_stroke = metrics.arrow_stroke_width.points();

        let (raw_rect, mut response) =
            ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
        let rect = subpixel_snap_rect(raw_rect, ui.ctx().pixels_per_point());

        let top_rect = Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.center().y));
        let bottom_rect = Rect::from_min_max(Pos2::new(rect.min.x, rect.center().y), rect.max);

        let top_id = response.id.with("mac_stepper_up");
        let bottom_id = response.id.with("mac_stepper_down");

        let top_resp = ui.interact(
            top_rect,
            top_id,
            if self.enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        let bottom_resp = ui.interact(
            bottom_rect,
            bottom_id,
            if self.enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );

        let min_val = *self.range.start();
        let max_val = *self.range.end();
        if *self.value < min_val {
            *self.value = min_val;
        } else if *self.value > max_val {
            *self.value = max_val;
        }
        let mut changed = false;

        let clicked_top = top_resp.clicked();
        let clicked_bottom = bottom_resp.clicked();
        if clicked_top
            || clicked_bottom
            || top_resp.is_pointer_button_down_on()
            || bottom_resp.is_pointer_button_down_on()
        {
            response.request_focus();
        }

        let now = ui.input(|i| i.time);
        let repeat_delay = metrics.auto_repeat_delay.points() as f64;
        let repeat_interval = metrics.auto_repeat_interval.points() as f64;

        let repeat_top = if self.enabled && top_resp.is_pointer_button_down_on() {
            ui.ctx().request_repaint();
            let state = ui.data_mut(|d| d.get_temp::<(f64, f64)>(top_id));
            match state {
                None => {
                    ui.data_mut(|d| d.insert_temp(top_id, (now, now)));
                    false
                }
                Some((press_start, last_repeat)) => {
                    if now - press_start >= repeat_delay && now - last_repeat >= repeat_interval {
                        ui.data_mut(|d| d.insert_temp(top_id, (press_start, now)));
                        true
                    } else {
                        false
                    }
                }
            }
        } else {
            ui.data_mut(|d| d.remove_temp::<(f64, f64)>(top_id));
            false
        };

        let repeat_bottom = if self.enabled && bottom_resp.is_pointer_button_down_on() {
            ui.ctx().request_repaint();
            let state = ui.data_mut(|d| d.get_temp::<(f64, f64)>(bottom_id));
            match state {
                None => {
                    ui.data_mut(|d| d.insert_temp(bottom_id, (now, now)));
                    false
                }
                Some((press_start, last_repeat)) => {
                    if now - press_start >= repeat_delay && now - last_repeat >= repeat_interval {
                        ui.data_mut(|d| d.insert_temp(bottom_id, (press_start, now)));
                        true
                    } else {
                        false
                    }
                }
            }
        } else {
            ui.data_mut(|d| d.remove_temp::<(f64, f64)>(bottom_id));
            false
        };

        let ptr_mods = ui.input(|i| {
            let mut m = i.modifiers;
            for ev in &i.raw.events {
                if let egui::Event::PointerButton { modifiers, .. } = ev {
                    m |= *modifiers;
                }
            }
            m
        });
        let shift_mult = metrics.shift_multiplier.points() as f64;
        let cmd_mult = metrics.cmd_multiplier.points() as f64;
        let ptr_mult = if ptr_mods.command || ptr_mods.ctrl || ptr_mods.mac_cmd {
            cmd_mult
        } else if ptr_mods.shift {
            shift_mult
        } else {
            1.0
        };

        let key_up_mult = if self.enabled && response.has_focus() {
            ui.input_mut(|i| {
                if i.consume_key(egui::Modifiers::COMMAND, Key::ArrowUp)
                    || i.consume_key(egui::Modifiers::CTRL, Key::ArrowUp)
                    || i.consume_key(egui::Modifiers::MAC_CMD, Key::ArrowUp)
                {
                    Some(cmd_mult)
                } else if i.consume_key(egui::Modifiers::SHIFT, Key::ArrowUp) {
                    Some(shift_mult)
                } else if i.consume_key(egui::Modifiers::NONE, Key::ArrowUp) {
                    Some(1.0)
                } else {
                    None
                }
            })
        } else {
            None
        };

        let key_down_mult = if self.enabled && response.has_focus() {
            ui.input_mut(|i| {
                if i.consume_key(egui::Modifiers::COMMAND, Key::ArrowDown)
                    || i.consume_key(egui::Modifiers::CTRL, Key::ArrowDown)
                    || i.consume_key(egui::Modifiers::MAC_CMD, Key::ArrowDown)
                {
                    Some(cmd_mult)
                } else if i.consume_key(egui::Modifiers::SHIFT, Key::ArrowDown) {
                    Some(shift_mult)
                } else if i.consume_key(egui::Modifiers::NONE, Key::ArrowDown) {
                    Some(1.0)
                } else {
                    None
                }
            })
        } else {
            None
        };

        let step_up_mult = if self.enabled && (clicked_top || repeat_top) {
            Some(ptr_mult)
        } else {
            key_up_mult
        };

        let step_down_mult = if self.enabled && (clicked_bottom || repeat_bottom) {
            Some(ptr_mult)
        } else {
            key_down_mult
        };

        let eps = 1e-9 * self.step.abs().max(1.0);

        // Up arrow click / hold-to-repeat / keyboard handling
        if let Some(mult) = step_up_mult {
            let next_val = *self.value + self.step * mult;
            if next_val <= max_val + eps {
                *self.value = next_val.min(max_val);
            } else if self.wraps {
                *self.value = min_val;
            } else {
                *self.value = max_val;
            }
            changed = true;
        }

        // Down arrow click / hold-to-repeat / keyboard handling
        if let Some(mult) = step_down_mult {
            let next_val = *self.value - self.step * mult;
            if next_val >= min_val - eps {
                *self.value = next_val.max(min_val);
            } else if self.wraps {
                *self.value = max_val;
            } else {
                *self.value = min_val;
            }
            changed = true;
        }

        if changed {
            response.mark_changed();
        }

        let painter = ui.painter_at(rect);

        // Outer focus ring
        if self.enabled && response.has_focus() {
            let ring = theme.focus_ring();
            let focus_rect = rect.expand(ring.offset.points());
            let focus_radius = corner_radius + ring.corner_radius_delta.points();
            painter.rect_stroke(
                focus_rect,
                CornerRadius::same(focus_radius as u8),
                Stroke::new(ring.stroke_width.points(), theme.keyboard_focus_indicator()),
                StrokeKind::Outside,
            );
        }

        // Outer bezel
        let bg_color = theme.control_background();
        let border_stroke = if response.has_focus() {
            Stroke::new(metrics.border_width.points(), theme.control_accent())
        } else {
            Stroke::new(metrics.border_width.points(), theme.separator())
        };

        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            bg_color,
            border_stroke,
            StrokeKind::Inside,
        );

        // Highlight compartments on hover / click
        if self.enabled {
            if top_resp.is_pointer_button_down_on() {
                painter.rect_filled(
                    top_rect,
                    CornerRadius {
                        nw: corner_radius as u8,
                        ne: corner_radius as u8,
                        sw: 0,
                        se: 0,
                    },
                    theme.selected_control_inactive(),
                );
            } else if top_resp.hovered() {
                painter.rect_filled(
                    top_rect,
                    CornerRadius {
                        nw: corner_radius as u8,
                        ne: corner_radius as u8,
                        sw: 0,
                        se: 0,
                    },
                    theme.control(),
                );
            }

            if bottom_resp.is_pointer_button_down_on() {
                painter.rect_filled(
                    bottom_rect,
                    CornerRadius {
                        nw: 0,
                        ne: 0,
                        sw: corner_radius as u8,
                        se: corner_radius as u8,
                    },
                    theme.selected_control_inactive(),
                );
            } else if bottom_resp.hovered() {
                painter.rect_filled(
                    bottom_rect,
                    CornerRadius {
                        nw: 0,
                        ne: 0,
                        sw: corner_radius as u8,
                        se: corner_radius as u8,
                    },
                    theme.control(),
                );
            }
        }

        // Divider line between segments
        let divider_y = rect.center().y;
        painter.line_segment(
            [
                Pos2::new(rect.left(), divider_y),
                Pos2::new(rect.right(), divider_y),
            ],
            Stroke::new(divider_thickness, theme.separator()),
        );

        // Chevrons
        let arrow_color = if !self.enabled {
            theme.placeholder_text()
        } else {
            theme.label()
        };
        let arrow_stroke_style = Stroke::new(arrow_stroke, arrow_color);
        let arrow_half_w = arrow_size * 0.5;
        let arrow_delta = metrics.arrow_delta.points();

        // Up arrow ^
        let up_cx = top_rect.center().x;
        let up_cy = top_rect.center().y;
        let up_tip = Pos2::new(up_cx, up_cy - arrow_delta);
        let up_left = Pos2::new(up_cx - arrow_half_w, up_cy + arrow_delta);
        let up_right = Pos2::new(up_cx + arrow_half_w, up_cy + arrow_delta);
        painter.line_segment([up_left, up_tip], arrow_stroke_style);
        painter.line_segment([up_tip, up_right], arrow_stroke_style);

        // Down arrow v
        let down_cx = bottom_rect.center().x;
        let down_cy = bottom_rect.center().y;
        let down_tip = Pos2::new(down_cx, down_cy + arrow_delta);
        let down_left = Pos2::new(down_cx - arrow_half_w, down_cy - arrow_delta);
        let down_right = Pos2::new(down_cx + arrow_half_w, down_cy - arrow_delta);
        painter.line_segment([down_left, down_tip], arrow_stroke_style);
        painter.line_segment([down_tip, down_right], arrow_stroke_style);

        response
    }
}
