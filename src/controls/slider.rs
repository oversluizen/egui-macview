//! Authentic macOS AppKit Continuous Slider (NSSlider equivalent).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors or dimensions.

use crate::controls::MacControlSize;
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
/// ```rust
/// use egui_macview::{MacSlider, MacTheme};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut progress = 50.0f32;
/// MacSlider::new(&mut progress, 0.0..=100.0).theme(&theme).ticks(5, true).show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacSlider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    theme: Option<&'a MacTheme>,
    width: Option<f32>,
    fixed_rect: Option<Rect>,
    control_size: MacControlSize,
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
            fixed_rect: None,
            control_size: MacControlSize::Regular,
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

    /// Pins the slider to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Sets the AppKit control size (`Regular`, `Small`, or `Mini`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Configures `NSSlider.numberOfTickMarks` along the slider track.
    pub fn tick_marks(mut self, count: usize) -> Self {
        self.num_ticks = (count > 0).then_some(count);
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
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.slider();
        let track_height = metrics.track_height.points();
        let thumb_size = metrics.thumb_size.points();
        let thumb_radius = thumb_size * 0.5;
        let tick_height = metrics.tick_height.points();
        let tick_offset = metrics.tick_offset.points();

        let has_ticks = self.num_ticks.map(|n| n >= 2).unwrap_or(false);
        let eff_track_height = if has_ticks { metrics.ticked_track_height.points() } else { track_height };
        let height = if has_ticks { thumb_size + tick_offset + tick_height } else { thumb_size };
        let width = self.width.unwrap_or_else(|| metrics.default_width.points());
        let sense = if self.enabled { egui::Sense::click_and_drag() } else { egui::Sense::hover() };
        let (rect, mut response) = if let Some(fixed) = self.fixed_rect {
            let id = ui.next_auto_id().with("mac_slider_fixed");
            ui.skip_ahead_auto_ids(1);
            (fixed, ui.interact(fixed, id, sense))
        } else {
            ui.allocate_exact_size(Vec2::new(width, height), sense)
        };

        let (min_val, max_val) = (*self.range.start(), *self.range.end());
        let span = (max_val - min_val).max(f32::EPSILON);
        if *self.value < min_val || self.value.is_nan() {
            *self.value = min_val;
        } else if *self.value > max_val {
            *self.value = max_val;
        }

        let (track_left, track_right) = (rect.left() + thumb_radius, rect.right() - thumb_radius);
        let min_track_w = metrics.thumb_stroke_width.points().max(f32::EPSILON);
        let track_width = (track_right - track_left).max(min_track_w);
        let center_y = rect.top() + thumb_radius;

        if self.enabled && response.clicked() {
            ui.memory_mut(|m| m.request_focus(response.id));
        }

        if self.enabled && (response.clicked() || response.dragged()) {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let mut fraction = ((mouse_pos.x - track_left) / track_width).clamp(0.0, 1.0);
                if self.snap_to_ticks {
                    if let Some(ticks) = self.num_ticks.filter(|&t| t >= 2) {
                        let step = 1.0 / (ticks - 1) as f32;
                        fraction = ((fraction / step).round() * step).clamp(0.0, 1.0);
                    }
                }
                let new_value = (min_val + fraction * span).clamp(min_val, max_val);
                if (*self.value - new_value).abs() > f32::EPSILON {
                    *self.value = new_value;
                    response.mark_changed();
                }
            }
        }

        if self.enabled && response.has_focus() {
            let key_step_frac = metrics.key_step_fraction.points();
            let step = self.num_ticks.filter(|&t| t >= 2).map_or(span * key_step_frac, |t| span / (t - 1) as f32);
            let mut delta = 0.0f32;
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft) || i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                delta -= step;
            }
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight) || i.consume_key(Modifiers::NONE, Key::ArrowUp)) {
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
        let alpha_factor = if self.enabled { 1.0 } else { theme.disabled_opacity() };
        let painter = ui.painter_at(rect);

        // 1. Unfilled groove track background
        let track_rect = Rect::from_min_max(
            Pos2::new(track_left, center_y - eff_track_height * 0.5),
            Pos2::new(track_right, center_y + eff_track_height * 0.5),
        );
        let track_radius = eff_track_height * 0.5;
        let track_fill = if has_ticks {
            theme.slider_track_bg().gamma_multiply(alpha_factor)
        } else {
            theme.control().gamma_multiply(alpha_factor)
        };
        let track_stroke = if has_ticks {
            Stroke::new(metrics.ticked_track_border_width.points(), theme.slider_track_border().gamma_multiply(alpha_factor))
        } else {
            Stroke::NONE
        };
        painter.rect(track_rect, CornerRadius::same(track_radius as u8), track_fill, track_stroke, StrokeKind::Inside);

        // 2. Active filled progress track (unticked continuous NSSlider only)
        if !has_ticks && current_fraction > 0.0 {
            let active_rect = Rect::from_min_max(
                Pos2::new(track_left, center_y - eff_track_height * 0.5),
                Pos2::new(thumb_center_x, center_y + eff_track_height * 0.5),
            );
            let fill_c = if self.enabled { theme.control_accent() } else { theme.selected_control_inactive().gamma_multiply(alpha_factor) };
            painter.rect(active_rect, CornerRadius::same(track_radius as u8), fill_c, Stroke::NONE, StrokeKind::Inside);
        }

        // 3. Cross-track tick marks (AppKit NSSliderCell with numberOfTickMarks > 0)
        if let Some(ticks) = self.num_ticks.filter(|&t| t >= 2) {
            let tick_y_start = center_y + tick_offset;
            let tick_y_end = tick_y_start + tick_height;
            let tick_stroke = Stroke::new(metrics.tick_width.points(), theme.slider_tick_mark_color().gamma_multiply(alpha_factor));
            for i in 0..ticks {
                let tick_x = track_left + (i as f32 / (ticks - 1) as f32) * track_width;
                painter.line_segment([Pos2::new(tick_x, tick_y_start), Pos2::new(tick_x, tick_y_end)], tick_stroke);
            }
        }

        // 4. Thumb knob (circular knob or vertical pill thumb when has_ticks is true)
        let is_pressed_or_dragged = self.enabled && (response.is_pointer_button_down_on() || response.dragged());
        let press_scale_target = if is_pressed_or_dragged { metrics.thumb_press_scale.points() } else { 1.0 };
        let spring = theme.motion().swiftui_snappy.to_spring();
        let anim_scale = spring.animate_f32(ui.ctx(), response.id.with("press_scale"), press_scale_target);
        let scaled_thumb_radius = thumb_radius * anim_scale;
        let thumb_pos = Pos2::new(thumb_center_x, center_y);

        if self.enabled && response.has_focus() {
            let ring = theme.focus_ring();
            let halo_r = scaled_thumb_radius + ring.offset.points();
            let halo_stroke = Stroke::new(ring.stroke_width.points(), theme.keyboard_focus_indicator());
            painter.circle(thumb_pos, halo_r, Color32::TRANSPARENT, halo_stroke);
        }

        if has_ticks {
            let pw = metrics.ticked_thumb_width.points() * anim_scale;
            let ph = metrics.ticked_thumb_height.points() * anim_scale;
            let pill_rect = Rect::from_center_size(thumb_pos, Vec2::new(pw, ph));
            let pill_r = CornerRadius::same(metrics.ticked_thumb_corner_radius.points() as u8);
            let pill_bg = theme.slider_thumb_bg().gamma_multiply(alpha_factor);
            let pill_stroke = Stroke::new(metrics.thumb_stroke_width.points(), theme.slider_thumb_border_color().gamma_multiply(alpha_factor));
            painter.rect(pill_rect, pill_r, pill_bg, pill_stroke, StrokeKind::Inside);
        } else {
            let thumb_bg = if !self.enabled || is_pressed_or_dragged {
                theme.control().gamma_multiply(alpha_factor)
            } else {
                theme.control_background().gamma_multiply(alpha_factor)
            };
            let thumb_stroke = Stroke::new(metrics.thumb_stroke_width.points(), theme.separator().gamma_multiply(alpha_factor));
            painter.circle_filled(
                thumb_pos + Vec2::new(0.0, metrics.shadow_offset_y.points()),
                scaled_thumb_radius,
                theme.drop_shadow().gamma_multiply(alpha_factor),
            );
            painter.circle(thumb_pos, scaled_thumb_radius, thumb_bg, thumb_stroke);
        }

        response
    }
}
