//! Authentic macOS AppKit Continuous Slider (NSSlider equivalent).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors or dimensions.

use crate::theme::MacTheme;
use egui::{
    Color32, CornerRadius, Key, Modifiers, Pos2, Rect, Response, Stroke, StrokeKind, Ui, Vec2,
    Widget,
};
use std::ops::RangeInclusive;

/// Authentic macOS AppKit continuous slider (`NSSlider` equivalent).
///
/// Features authentic track geometry, thumb knob with subtle drop shadow,
/// optional tick marks with magnetic snapping, and keyboard arrow step support.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacSlider, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut progress = 50.0f32;
///
/// MacSlider::new(&mut progress, 0.0..=100.0)
///     .theme(&theme)
///     .ticks(5, true)
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacSlider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    theme: Option<&'a MacTheme>,
    width: Option<f32>,
    num_ticks: Option<usize>,
    snap_to_ticks: bool,
    enabled: bool,
}

impl<'a> MacSlider<'a> {
    /// Creates a new continuous slider bound to `value` over the given `range`.
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            theme: None,
            width: None,
            num_ticks: None,
            snap_to_ticks: false,
            enabled: true,
        }
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets explicit control width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Configures tick marks along the slider track with optional snapping.
    pub fn ticks(mut self, count: usize, snap_to_ticks: bool) -> Self {
        self.num_ticks = Some(count);
        self.snap_to_ticks = snap_to_ticks;
        self
    }

    /// Sets interactive enabled state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Shows the slider in the UI and returns its interaction response.
    pub fn show(self, ui: &mut Ui) -> Response {
        self.ui(ui)
    }
}

impl<'a> Widget for MacSlider<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::from_ctx(ui.ctx());
                &fallback_theme
            }
        };

        let metrics = theme.slider();
        let track_height = metrics.track_height.points();
        let thumb_size = metrics.thumb_size.points();
        let thumb_radius = thumb_size * 0.5;
        let tick_height = metrics.tick_height.points();
        let tick_offset = metrics.tick_offset.points();

        let has_ticks = self.num_ticks.map(|n| n >= 2).unwrap_or(false);
        let height = if has_ticks {
            thumb_size + tick_offset + tick_height
        } else {
            thumb_size
        };

        let width = self.width.unwrap_or_else(|| metrics.default_width.points());
        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(width, height),
            if self.enabled {
                egui::Sense::click_and_drag()
            } else {
                egui::Sense::hover()
            },
        );

        let min_val = *self.range.start();
        let max_val = *self.range.end();
        let span = (max_val - min_val).max(f32::EPSILON);

        // Enforce range invariant unconditionally
        if *self.value < min_val {
            *self.value = min_val;
        } else if *self.value > max_val {
            *self.value = max_val;
        } else if self.value.is_nan() {
            *self.value = min_val;
        }

        let track_left = rect.left() + thumb_radius;
        let track_right = rect.right() - thumb_radius;
        let min_track_w = metrics.thumb_stroke_width.points().max(f32::EPSILON);
        let track_width = (track_right - track_left).max(min_track_w);
        let center_y = rect.top() + thumb_radius;

        if self.enabled && response.clicked() {
            ui.memory_mut(|m| m.request_focus(response.id));
        }

        // Interaction processing
        if self.enabled && (response.clicked() || response.dragged()) {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let mut fraction = ((mouse_pos.x - track_left) / track_width).clamp(0.0, 1.0);

                if self.snap_to_ticks {
                    if let Some(ticks) = self.num_ticks {
                        if ticks >= 2 {
                            let step = 1.0 / (ticks - 1) as f32;
                            let tick_idx = (fraction / step).round();
                            fraction = (tick_idx * step).clamp(0.0, 1.0);
                        }
                    }
                }

                let new_value = (min_val + fraction * span).clamp(min_val, max_val);
                if (*self.value - new_value).abs() > f32::EPSILON {
                    *self.value = new_value;
                    response.mark_changed();
                }
            }
        }

        // Accessible keyboard navigation (consuming arrow keys)
        if self.enabled && response.has_focus() {
            let key_step_frac = metrics.key_step_fraction.points();
            let step = if let Some(ticks) = self.num_ticks {
                if ticks >= 2 {
                    span / (ticks - 1) as f32
                } else {
                    span * key_step_frac
                }
            } else {
                span * key_step_frac
            };

            let mut delta = 0.0f32;
            if ui.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::ArrowLeft)
                    || i.consume_key(Modifiers::NONE, Key::ArrowDown)
            }) {
                delta -= step;
            }
            if ui.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::ArrowRight)
                    || i.consume_key(Modifiers::NONE, Key::ArrowUp)
            }) {
                delta += step;
            }

            if delta.abs() > f32::EPSILON {
                let new_value = (*self.value + delta).clamp(min_val, max_val);
                if (*self.value - new_value).abs() > f32::EPSILON {
                    *self.value = new_value;
                    response.mark_changed();
                }
            }
        }

        let current_fraction = ((*self.value - min_val) / span).clamp(0.0, 1.0);
        let thumb_center_x = track_left + current_fraction * track_width;
        let alpha_factor = if self.enabled {
            1.0
        } else {
            theme.disabled_opacity()
        };

        let painter = ui.painter_at(rect);

        // 1. Unfilled groove track background
        let track_rect = Rect::from_min_max(
            Pos2::new(track_left, center_y - track_height * 0.5),
            Pos2::new(track_right, center_y + track_height * 0.5),
        );
        let track_radius = track_height * 0.5;
        painter.rect(
            track_rect,
            CornerRadius::same(track_radius as u8),
            theme.control().gamma_multiply(alpha_factor),
            Stroke::NONE,
            StrokeKind::Inside,
        );

        // 2. Active filled progress track
        if current_fraction > 0.0 {
            let active_rect = Rect::from_min_max(
                Pos2::new(track_left, center_y - track_height * 0.5),
                Pos2::new(thumb_center_x, center_y + track_height * 0.5),
            );
            painter.rect(
                active_rect,
                CornerRadius::same(track_radius as u8),
                if self.enabled {
                    theme.control_accent()
                } else {
                    theme
                        .selected_control_inactive()
                        .gamma_multiply(alpha_factor)
                },
                Stroke::NONE,
                StrokeKind::Inside,
            );
        }

        // 3. Tick marks
        if let Some(ticks) = self.num_ticks {
            if ticks >= 2 {
                let tick_y_start = center_y + thumb_radius + tick_offset;
                let tick_y_end = tick_y_start + tick_height;
                let tick_stroke = Stroke::new(
                    metrics.tick_width.points(),
                    theme.separator().gamma_multiply(alpha_factor),
                );

                for i in 0..ticks {
                    let tick_frac = i as f32 / (ticks - 1) as f32;
                    let tick_x = track_left + tick_frac * track_width;
                    painter.line_segment(
                        [
                            Pos2::new(tick_x, tick_y_start),
                            Pos2::new(tick_x, tick_y_end),
                        ],
                        tick_stroke,
                    );
                }
            }
        }

        // 4. Thumb knob (with 1.15x CASpring press-scale interpolation)
        let is_pressed_or_dragged =
            self.enabled && (response.is_pointer_button_down_on() || response.dragged());
        let press_scale_target = if is_pressed_or_dragged {
            metrics.thumb_press_scale.points()
        } else {
            1.0
        };
        let spring = theme.motion().swiftui_snappy.to_spring();
        let now = ui.input(|i| i.time);
        let spring_key = response.id.with("press_scale");
        let (prev_target, start_val, start_time) = ui.data(|d| {
            d.get_temp::<(f32, f32, f64)>(spring_key).unwrap_or((
                press_scale_target,
                press_scale_target,
                now,
            ))
        });
        let anim_scale = if (prev_target - press_scale_target).abs() > f32::EPSILON {
            let elapsed = (now - start_time).max(0.0) as f32;
            let cur = start_val + (prev_target - start_val) * spring.position_at(elapsed);
            ui.data_mut(|d| d.insert_temp(spring_key, (press_scale_target, cur, now)));
            ui.ctx().request_repaint();
            cur
        } else {
            let elapsed = (now - start_time).max(0.0) as f32;
            if elapsed < spring.settling_duration() {
                ui.ctx().request_repaint();
                start_val + (press_scale_target - start_val) * spring.position_at(elapsed)
            } else {
                press_scale_target
            }
        };
        let scaled_thumb_radius = thumb_radius * anim_scale;

        let thumb_pos = Pos2::new(thumb_center_x, center_y);
        let thumb_bg = if !self.enabled || is_pressed_or_dragged {
            theme.control().gamma_multiply(alpha_factor)
        } else {
            theme.control_background().gamma_multiply(alpha_factor)
        };

        let thumb_stroke = Stroke::new(
            metrics.thumb_stroke_width.points(),
            theme.separator().gamma_multiply(alpha_factor),
        );

        if self.enabled && response.has_focus() {
            let ring = theme.focus_ring();
            let halo_r = scaled_thumb_radius + ring.offset.points();
            let halo_stroke =
                Stroke::new(ring.stroke_width.points(), theme.keyboard_focus_indicator());
            painter.circle(thumb_pos, halo_r, Color32::TRANSPARENT, halo_stroke);
        }

        // Ambient drop shadow sourced from design tokens
        painter.circle_filled(
            thumb_pos + Vec2::new(0.0, metrics.shadow_offset_y.points()),
            scaled_thumb_radius,
            theme.drop_shadow().gamma_multiply(alpha_factor),
        );

        painter.circle(thumb_pos, scaled_thumb_radius, thumb_bg, thumb_stroke);

        response
    }
}
