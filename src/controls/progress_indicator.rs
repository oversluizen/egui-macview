//! Authentic macOS AppKit Progress Indicator (`NSProgressIndicator` equivalent, SPEC-011).
//!
//! Grounded in Apple AppKit `NSProgressIndicator` and W3C DTCG tokens via [`MacTheme`].
//! Supports `Bar` (determinate with `CASpring` smoothing & indeterminate pulse) and
//! `Spinning` (12-spoke radial fin spinner) across `Regular`, `Small`, and `Mini` control sizes.

use crate::layout::subpixel_snap_rect;
use crate::motion::CASpring;
use crate::theme::MacTheme;
use crate::tokens::ProgressSizeMetrics;
use egui::{
    Align2, CornerRadius, FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget,
};

/// Presentation style for [`MacProgressIndicator`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacProgressStyle {
    /// Horizontal determinate or indeterminate progress bar (`NSProgressIndicatorBarStyle`).
    #[default]
    Bar,
    /// Radial 12-spoke fin spinner (`NSProgressIndicatorSpinningStyle`).
    Spinning,
}

/// Control size tier for [`MacProgressIndicator`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacProgressControlSize {
    /// Standard size (`NSControlSizeRegular`: 6pt bar, 32pt spinner).
    #[default]
    Regular,
    /// Compact size (`NSControlSizeSmall`: 4pt bar, 16pt spinner).
    Small,
    /// Miniature size (`NSControlSizeMini`: 4pt bar, 12pt spinner).
    Mini,
}

/// Authentic macOS AppKit progress indicator (`NSProgressIndicator` equivalent).
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacProgressControlSize, MacProgressIndicator, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// MacProgressIndicator::bar(0.75)
///     .theme(&theme)
///     .control_size(MacProgressControlSize::Regular)
///     .show_percentage(true)
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacProgressIndicator<'a> {
    style: MacProgressStyle,
    control_size: MacProgressControlSize,
    progress: Option<f32>,
    time: f64,
    label: Option<&'a str>,
    show_percentage: bool,
    width: Option<f32>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> Default for MacProgressIndicator<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MacProgressIndicator<'a> {
    /// Creates a new determinate bar progress indicator at `0.0`.
    pub fn new() -> Self {
        Self {
            style: MacProgressStyle::Bar,
            control_size: MacProgressControlSize::Regular,
            progress: Some(0.0),
            time: 0.0,
            label: None,
            show_percentage: false,
            width: None,
            enabled: true,
            theme: None,
        }
    }

    /// Creates a determinate horizontal progress bar with progress in `0.0..=1.0`.
    pub fn bar(progress: f32) -> Self {
        Self::new().style(MacProgressStyle::Bar).progress(progress)
    }

    /// Creates a radial spinning fin indicator animated by `time` in seconds.
    pub fn spinning(time: f64) -> Self {
        Self::new()
            .style(MacProgressStyle::Spinning)
            .indeterminate(time)
    }

    /// Sets the presentation style (`Bar` or `Spinning`).
    pub fn style(mut self, style: MacProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the AppKit control size (`Regular`, `Small`, or `Mini`).
    pub fn control_size(mut self, size: MacProgressControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Sets determinate progress fraction (`0.0..=1.0`).
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress.clamp(0.0, 1.0));
        self
    }

    /// Configures indeterminate mode driven by animation `time` in seconds.
    pub fn indeterminate(mut self, time: f64) -> Self {
        self.progress = None;
        self.time = time;
        self
    }

    /// Sets animation timestamp `time` in seconds.
    pub fn time(mut self, time: f64) -> Self {
        self.time = time;
        self
    }

    /// Attaches an optional status label next to or below the indicator.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Enables or disables automatic percentage readout (`0%`..=`100%`) for determinate bars.
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Overrides horizontal bar width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Enables or disables the control.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Returns the clamped determinate progress in `0.0..=1.0`, if determinate.
    pub fn clamped_progress(&self) -> Option<f32> {
        self.progress.map(|p| p.clamp(0.0, 1.0))
    }

    fn size_metrics<'t>(&self, theme: &'t MacTheme) -> &'t ProgressSizeMetrics {
        let pm = theme.progress_indicator();
        match self.control_size {
            MacProgressControlSize::Regular => &pm.regular,
            MacProgressControlSize::Small => &pm.small,
            MacProgressControlSize::Mini => &pm.mini,
        }
    }

    /// Renders the progress indicator in `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacProgressIndicator<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let pm = theme.progress_indicator();
        let sm = self.size_metrics(theme);
        let font_sz = sm.font_size.points();
        let label_spacing = pm.label_spacing.points();
        let pct_scale = pm.percentage_scale.points();

        let status_text: Option<String> = match (self.label, self.show_percentage, self.progress) {
            (Some(lbl), true, Some(p)) => {
                let pct = (p.clamp(0.0, 1.0) * pct_scale).round() as u32;
                Some(format!("{lbl} ({pct}%)"))
            }
            (Some(lbl), _, _) => Some(lbl.to_string()),
            (None, true, Some(p)) => {
                let pct = (p.clamp(0.0, 1.0) * pct_scale).round() as u32;
                Some(format!("{pct}%"))
            }
            (None, _, _) => None,
        };

        match self.style {
            MacProgressStyle::Bar => {
                let bar_w = self.width.unwrap_or_else(|| pm.default_bar_width.points());
                let bar_h = sm.bar_height.points();
                let total_h = if status_text.is_some() {
                    bar_h + label_spacing + font_sz
                } else {
                    bar_h
                };

                let (raw_rect, response) =
                    ui.allocate_exact_size(Vec2::new(bar_w, total_h), Sense::hover());
                let rect = subpixel_snap_rect(raw_rect, ui.ctx().pixels_per_point());
                let track_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), bar_h));
                let radius = CornerRadius::same(sm.bar_corner_radius.points() as u8);
                let painter = ui.painter_at(rect);

                painter.rect(
                    track_rect,
                    radius,
                    theme.control(),
                    Stroke::new(pm.border_width.points(), theme.separator()),
                    StrokeKind::Inside,
                );

                let fill_color = if self.enabled {
                    theme.control_accent()
                } else {
                    theme.selected_control_inactive()
                };

                if let Some(target_p) = self.clamped_progress() {
                    let spring_preset = &theme.motion().swiftui_smooth;
                    let spring = CASpring::new(spring_preset.response, spring_preset.damping_ratio);
                    let anim_id = response.id.with("spring_progress");
                    let now = if self.time > 0.0 {
                        self.time
                    } else {
                        ui.input(|i| i.time)
                    };
                    let (mut from_p, mut to_p, mut start_t) = ui
                        .data(|d| d.get_temp::<(f32, f32, f64)>(anim_id))
                        .unwrap_or((target_p, target_p, now));

                    let elapsed = ((now - start_t).max(0.0)) as f32;
                    let cur_eval = spring.evaluate(elapsed).clamp(0.0, 1.0);
                    let cur_p = (from_p + (to_p - from_p) * cur_eval).clamp(0.0, 1.0);

                    let smooth_p = if (target_p - to_p).abs() > f32::EPSILON {
                        from_p = cur_p;
                        to_p = target_p;
                        start_t = now;
                        ui.ctx().request_repaint();
                        from_p
                    } else {
                        if elapsed < spring.response * 2.0 {
                            ui.ctx().request_repaint();
                        }
                        cur_p
                    };
                    ui.data_mut(|d| d.insert_temp(anim_id, (from_p, to_p, start_t)));

                    if smooth_p > 0.0 {
                        let fill_w = track_rect.width() * smooth_p;
                        let fill_rect =
                            Rect::from_min_size(track_rect.min, Vec2::new(fill_w, bar_h));
                        painter.rect_filled(fill_rect, radius, fill_color);
                    }
                } else {
                    let pill_ratio = pm.indeterminate_pill_ratio.points().clamp(0.0, 1.0);
                    let pill_w = track_rect.width() * pill_ratio;
                    let travel = (track_rect.width() - pill_w).max(0.0);
                    let freq = pm.bar_indeterminate_speed.points() as f64;
                    let tau = pm.tau_radians.points() as f64;
                    let phase = (((self.time * freq * tau).sin() + 1.0) * 0.5) as f32;
                    let pill_x = track_rect.left() + travel * phase;
                    let pill_rect = Rect::from_min_size(
                        Pos2::new(pill_x, track_rect.top()),
                        Vec2::new(pill_w, bar_h),
                    );
                    painter.rect_filled(pill_rect, radius, fill_color);
                    ui.ctx().request_repaint();
                }

                if let Some(text) = status_text {
                    let label_y = track_rect.bottom() + label_spacing;
                    painter.text(
                        Pos2::new(track_rect.left(), label_y),
                        Align2::LEFT_TOP,
                        text,
                        FontId::proportional(font_sz),
                        theme.secondary_label(),
                    );
                }

                response
            }
            MacProgressStyle::Spinning => {
                let spin_sz = sm.spinner_size.points();
                let (raw_rect, response) =
                    ui.allocate_exact_size(Vec2::splat(spin_sz), Sense::hover());
                let rect = subpixel_snap_rect(raw_rect, ui.ctx().pixels_per_point());
                let painter = ui.painter_at(rect);

                let center = rect.center();
                let spoke_w = sm.spoke_width.points();
                let tip_r = spoke_w * 0.5;
                let spin_r = spin_sz * 0.5;
                let outer_r = (spin_r - tip_r).max(0.0);
                let inner_ratio = pm.spoke_inner_ratio.points();
                let inner_r = outer_r * inner_ratio;
                let spoke_count = (pm.spoke_count.points().round() as usize).max(1);
                let tau = pm.tau_radians.points();
                let min_alpha = pm.min_spoke_alpha.points();
                let speed_hz = pm.spin_speed_hz.points() as f64;
                let active_step =
                    ((self.time * speed_hz * spoke_count as f64).floor() as usize) % spoke_count;

                let base_color = theme.secondary_label();

                for idx in 0..spoke_count {
                    let angle = (idx as f32 / spoke_count as f32) * tau;
                    let dir = Vec2::new(angle.sin(), -angle.cos());
                    let p0 = center + dir * inner_r;
                    let p1 = center + dir * outer_r;

                    let dist = (active_step + spoke_count - idx) % spoke_count;
                    let decay = 1.0 - (dist as f32 / spoke_count as f32);
                    let factor = min_alpha + (1.0 - min_alpha) * decay;
                    let spoke_color = base_color.gamma_multiply(factor.clamp(0.0, 1.0));

                    painter.line_segment([p0, p1], Stroke::new(spoke_w, spoke_color));
                    painter.circle_filled(p0, tip_r, spoke_color);
                    painter.circle_filled(p1, tip_r, spoke_color);
                }

                ui.ctx().request_repaint();
                response
            }
        }
    }
}
