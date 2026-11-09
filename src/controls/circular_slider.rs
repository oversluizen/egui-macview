//! Authentic macOS AppKit rotary circular slider (`NSSlider` circular style).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].
//! Features an authentic rotary dial body, active track arc progression,
//! directional needle indicator, vertical drag & keyboard stepping, and focus ring.

use crate::theme::MacTheme;
use egui::{Color32, Key, Modifiers, Response, Sense, Stroke, Ui, Vec2, Widget};
use std::ops::RangeInclusive;

/// Authentic macOS AppKit rotary circular slider (`NSSlider` circular style).
pub struct MacCircularSlider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    theme: Option<&'a MacTheme>,
    diameter: Option<f32>,
    step: Option<f32>,
    enabled: bool,
}

impl<'a> MacCircularSlider<'a> {
    /// Creates a new rotary circular slider bound to `value` over `range`.
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            theme: None,
            diameter: None,
            step: None,
            enabled: true,
        }
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets explicit circular diameter in points.
    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = Some(diameter);
        self
    }

    /// Sets discrete step interval for keyboard and drag increments.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Sets interactive enabled state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Shows the rotary slider in the UI and returns its interaction response.
    pub fn show(self, ui: &mut Ui) -> Response {
        self.ui(ui)
    }
}

impl<'a> Widget for MacCircularSlider<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.slider();

        let diameter = self
            .diameter
            .unwrap_or_else(|| metrics.thumb_size.points() * 2.0);
        let radius = diameter * 0.5;
        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::splat(diameter),
            if self.enabled {
                Sense::click_and_drag()
            } else {
                Sense::hover()
            },
        );

        let (min_val, max_val) = if *self.range.start() <= *self.range.end() {
            (*self.range.start(), *self.range.end())
        } else {
            (*self.range.end(), *self.range.start())
        };
        let span = (max_val - min_val).max(1e-5);
        let step = self
            .step
            .unwrap_or_else(|| span * metrics.key_step_fraction.points())
            .max(1e-5);

        if *self.value < min_val || self.value.is_nan() {
            *self.value = min_val;
        } else if *self.value > max_val {
            *self.value = max_val;
        }

        if self.enabled {
            if response.clicked() {
                ui.memory_mut(|m| m.request_focus(response.id));
            }

            if response.dragged() {
                let dy = -ui.input(|i| i.pointer.delta().y);
                let delta_val = (dy / diameter.max(1.0)) * span;
                let new_raw = *self.value + delta_val;
                let stepped = (new_raw / step).round() * step;
                let clamped = stepped.clamp(min_val, max_val);
                if (clamped - *self.value).abs() > 1e-5 {
                    *self.value = clamped;
                    response.mark_changed();
                }
            }

            if response.has_focus() {
                let mut key_delta = 0.0f32;
                if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp))
                    || ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight))
                {
                    key_delta += step;
                }
                if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown))
                    || ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft))
                {
                    key_delta -= step;
                }
                if key_delta != 0.0 {
                    *self.value = (*self.value + key_delta).clamp(min_val, max_val);
                    response.mark_changed();
                }
            }
        }

        let alpha = if self.enabled {
            1.0
        } else {
            theme.disabled_opacity()
        };
        let painter = ui.painter_at(rect);
        let center = rect.center();

        // 1. Ambient drop shadow
        let shadow_offset = Vec2::new(0.0, metrics.shadow_offset_y.points());
        painter.circle_filled(
            center + shadow_offset,
            radius,
            theme.drop_shadow().gamma_multiply(alpha),
        );

        // 2. Rotary dial body fill & border
        let dial_bg = if !self.enabled || response.is_pointer_button_down_on() || response.dragged()
        {
            theme.control().gamma_multiply(alpha)
        } else {
            theme.control_background().gamma_multiply(alpha)
        };
        let dial_stroke = Stroke::new(
            metrics.thumb_stroke_width.points(),
            theme.separator().gamma_multiply(alpha),
        );
        painter.circle(center, radius, dial_bg, dial_stroke);

        // 3. 270-degree active arc progression (from 135 deg to 405 deg)
        let frac = ((*self.value - min_val) / span).clamp(0.0, 1.0);
        let start_angle = std::f32::consts::PI * (f32::from(3u8) / f32::from(4u8));
        let total_sweep = std::f32::consts::PI * (f32::from(3u8) / f32::from(2u8));
        let current_angle = start_angle + frac * total_sweep;

        let arc_r = radius - metrics.track_height.points();
        let arc_steps = 28usize;
        let active_steps = (frac * (arc_steps as f32)).round() as usize;
        let arc_stroke = Stroke::new(
            metrics.track_height.points(),
            if self.enabled {
                theme.control_accent()
            } else {
                theme.selected_control_inactive().gamma_multiply(alpha)
            },
        );
        for i in 0..active_steps {
            let a0 = start_angle + (i as f32 / arc_steps as f32) * total_sweep;
            let a1 = start_angle + ((i + 1) as f32 / arc_steps as f32) * total_sweep;
            let p0 = center + Vec2::new(a0.cos() * arc_r, a0.sin() * arc_r);
            let p1 = center + Vec2::new(a1.cos() * arc_r, a1.sin() * arc_r);
            painter.line_segment([p0, p1], arc_stroke);
        }

        // 4. Pointer needle indicator from dial center to outer rim
        let needle_inner_r = radius * 0.5 * 0.5;
        let needle_outer_r = radius - metrics.thumb_stroke_width.points() * 2.0;
        let needle_dir = Vec2::new(current_angle.cos(), current_angle.sin());
        let needle_p0 = center + needle_dir * needle_inner_r;
        let needle_p1 = center + needle_dir * needle_outer_r;
        let needle_stroke = Stroke::new(
            metrics.thumb_stroke_width.points() * 2.0,
            if self.enabled {
                theme.label()
            } else {
                theme.secondary_label().gamma_multiply(alpha)
            },
        );
        painter.line_segment([needle_p0, needle_p1], needle_stroke);

        // 5. Focus ring
        if self.enabled && response.has_focus() {
            let ring = theme.focus_ring();
            let halo_stroke =
                Stroke::new(ring.stroke_width.points(), theme.keyboard_focus_indicator());
            painter.circle(
                center,
                radius + ring.offset.points(),
                Color32::TRANSPARENT,
                halo_stroke,
            );
        }

        response
    }
}
