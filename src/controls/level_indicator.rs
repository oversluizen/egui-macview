//! Authentic macOS AppKit Level Indicator (`NSLevelIndicator` equivalent, SPEC-011).
//!
//! Grounded in Apple AppKit `NSLevelIndicator` and W3C DTCG tokens via [`MacTheme`].
//! Supports `Continuous` and `Discrete` styles in both `Horizontal` and `Vertical` orientations,
//! with normal (`system_green`), warning (`system_yellow`), and critical (`system_red`) thresholds
//! plus an optional peak-hold marker line.

use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    Color32, CornerRadius, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget,
};
use std::ops::RangeInclusive;

/// Visual meter style for [`MacLevelIndicator`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacLevelIndicatorStyle {
    /// Smooth continuous level bar (`NSLevelIndicatorStyleContinuousCapacity`).
    #[default]
    Continuous,
    /// Segmented block meter (`NSLevelIndicatorStyleDiscreteCapacity`).
    Discrete,
}

/// Primary axis orientation for [`MacLevelIndicator`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacLevelOrientation {
    /// Left-to-right horizontal meter.
    #[default]
    Horizontal,
    /// Bottom-to-top vertical meter.
    Vertical,
}

/// Authentic macOS AppKit signal and audio level meter (`NSLevelIndicator` equivalent).
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacLevelIndicator, MacLevelIndicatorStyle, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// MacLevelIndicator::new(-12.0, -60.0..=0.0)
///     .theme(&theme)
///     .warning_value(-18.0)
///     .critical_value(-6.0)
///     .peak_hold(-3.0)
///     .style(MacLevelIndicatorStyle::Discrete)
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacLevelIndicator<'a> {
    value: f32,
    range: RangeInclusive<f32>,
    warning_value: f32,
    critical_value: f32,
    peak_hold_value: Option<f32>,
    style: MacLevelIndicatorStyle,
    orientation: MacLevelOrientation,
    segment_count: Option<usize>,
    segmented_threshold_colors: bool,
    tick_marks: usize,
    length: Option<f32>,
    thickness: Option<f32>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacLevelIndicator<'a> {
    /// Creates a new level indicator for `value` within `range`.
    pub fn new(value: f32, range: RangeInclusive<f32>) -> Self {
        let min_v = *range.start();
        let max_v = *range.end();
        let span = max_v - min_v;
        Self {
            value,
            range,
            warning_value: min_v + span * 0.5,
            critical_value: min_v + span,
            peak_hold_value: None,
            style: MacLevelIndicatorStyle::Continuous,
            orientation: MacLevelOrientation::Horizontal,
            segment_count: None,
            segmented_threshold_colors: false,
            tick_marks: 0,
            length: None,
            thickness: None,
            enabled: true,
            theme: None,
        }
    }

    /// Sets warning threshold where meter switches from green to yellow/amber.
    pub fn warning_value(mut self, warning: f32) -> Self {
        self.warning_value = warning;
        self
    }

    /// Sets critical threshold where meter switches from yellow/amber to red.
    pub fn critical_value(mut self, critical: f32) -> Self {
        self.critical_value = critical;
        self
    }

    /// Sets optional peak-hold marker value.
    pub fn peak_hold(mut self, peak: f32) -> Self {
        self.peak_hold_value = Some(peak);
        self
    }

    /// Sets meter style (`Continuous` or `Discrete`).
    pub fn style(mut self, style: MacLevelIndicatorStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets meter orientation (`Horizontal` or `Vertical`).
    pub fn orientation(mut self, orientation: MacLevelOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets discrete segment count.
    pub fn segments(mut self, count: usize) -> Self {
        self.segment_count = Some(count.max(1));
        self
    }

    /// Enables per-segment VU meter threshold coloring instead of uniform AppKit discrete capacity color.
    pub fn segmented_threshold_colors(mut self, enabled: bool) -> Self {
        self.segmented_threshold_colors = enabled;
        self
    }

    /// Sets the number of graduation tick marks rendered along the meter axis.
    pub fn tick_marks(mut self, count: usize) -> Self {
        self.tick_marks = count;
        self
    }

    /// Overrides primary-axis length in points.
    pub fn length(mut self, length: f32) -> Self {
        self.length = Some(length);
        self
    }

    /// Overrides cross-axis thickness in points.
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = Some(thickness);
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

    /// Evaluates the semantic AppKit meter color for a given sample level `sample_value`.
    pub fn color_for_value(&self, theme: &MacTheme, sample_value: f32) -> Color32 {
        if !self.enabled {
            return theme.selected_control_inactive();
        }
        if sample_value >= self.critical_value {
            theme.system_red()
        } else if sample_value >= self.warning_value {
            theme.system_yellow()
        } else {
            theme.system_green()
        }
    }

    fn normalize(&self, v: f32) -> f32 {
        let min_v = *self.range.start();
        let max_v = *self.range.end();
        let span = (max_v - min_v).max(f32::EPSILON);
        ((v - min_v) / span).clamp(0.0, 1.0)
    }

    /// Renders the level indicator in `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacLevelIndicator<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let lm = theme.level_indicator();
        let len = self.length.unwrap_or_else(|| lm.default_length.points());
        let thick = self.thickness.unwrap_or_else(|| lm.thickness.points());
        let size = match self.orientation {
            MacLevelOrientation::Horizontal => Vec2::new(len, thick),
            MacLevelOrientation::Vertical => Vec2::new(thick, len),
        };

        let (raw_rect, response) = ui.allocate_exact_size(size, Sense::hover());
        let rect = subpixel_snap_rect(raw_rect, ui.ctx().pixels_per_point());
        let painter = ui.painter_at(rect);

        let outer_r = CornerRadius::same(lm.corner_radius.points() as u8);
        painter.rect(
            rect,
            outer_r,
            theme.control_background(),
            Stroke::new(lm.border_width.points(), theme.separator()),
            StrokeKind::Inside,
        );

        let inset = lm.track_inset.points();
        let inner = rect.shrink(inset);
        let frac = self.normalize(self.value);
        let min_v = *self.range.start();
        let max_v = *self.range.end();
        let span = (max_v - min_v).max(f32::EPSILON);

        match self.style {
            MacLevelIndicatorStyle::Continuous => {
                let seg_r = CornerRadius::same(lm.segment_corner_radius.points() as u8);
                let fill_color = self.color_for_value(theme, self.value);
                if frac > 0.0 {
                    let fill_rect = match self.orientation {
                        MacLevelOrientation::Horizontal => Rect::from_min_size(
                            inner.min,
                            Vec2::new(inner.width() * frac, inner.height()),
                        ),
                        MacLevelOrientation::Vertical => {
                            let h = inner.height() * frac;
                            Rect::from_min_max(
                                Pos2::new(inner.left(), inner.bottom() - h),
                                inner.max,
                            )
                        }
                    };
                    painter.rect_filled(fill_rect, seg_r, fill_color);
                }
            }
            MacLevelIndicatorStyle::Discrete => {
                let seg_count = self
                    .segment_count
                    .unwrap_or_else(|| (lm.default_segment_count.points().round() as usize).max(1));
                let gap = lm.segment_spacing.points();
                let seg_r = CornerRadius::same(lm.segment_corner_radius.points() as u8);
                let total_axis = match self.orientation {
                    MacLevelOrientation::Horizontal => inner.width(),
                    MacLevelOrientation::Vertical => inner.height(),
                };
                let total_gaps = gap * ((seg_count.saturating_sub(1)) as f32);
                let seg_len = ((total_axis - total_gaps) / (seg_count as f32)).max(1.0);

                for idx in 0..seg_count {
                    let seg_frac_end = (idx + 1) as f32 / (seg_count as f32);
                    let seg_mid_frac = (idx as f32 + 0.5) / (seg_count as f32);
                    let seg_val = min_v + seg_mid_frac * span;
                    let is_lit = frac >= (idx as f32 / (seg_count as f32))
                        && frac > 0.0
                        && (frac >= seg_frac_end
                            || idx == ((frac * seg_count as f32).floor() as usize));

                    let seg_rect = match self.orientation {
                        MacLevelOrientation::Horizontal => {
                            let x = inner.left() + (idx as f32) * (seg_len + gap);
                            Rect::from_min_size(
                                Pos2::new(x, inner.top()),
                                Vec2::new(seg_len, inner.height()),
                            )
                        }
                        MacLevelOrientation::Vertical => {
                            let y_bottom = inner.bottom() - (idx as f32) * (seg_len + gap);
                            Rect::from_min_max(
                                Pos2::new(inner.left(), y_bottom - seg_len),
                                Pos2::new(inner.right(), y_bottom),
                            )
                        }
                    };

                    let color = if is_lit {
                        if self.segmented_threshold_colors {
                            self.color_for_value(theme, seg_val)
                        } else {
                            self.color_for_value(theme, self.value)
                        }
                    } else {
                        theme.control()
                    };
                    painter.rect_filled(seg_rect, seg_r, color);
                }
            }
        }

        if self.tick_marks > 0 {
            let tick_stroke = Stroke::new(lm.border_width.points(), theme.secondary_label());
            let tick_len = lm.track_inset.points() + lm.border_width.points();
            let denom = (self.tick_marks.saturating_sub(1)).max(1) as f32;
            for idx in 0..self.tick_marks {
                let t_frac = if self.tick_marks == 1 {
                    0.5
                } else {
                    (idx as f32) / denom
                };
                match self.orientation {
                    MacLevelOrientation::Horizontal => {
                        let tx = inner.left() + inner.width() * t_frac;
                        painter.line_segment(
                            [
                                Pos2::new(tx, rect.bottom() - tick_len),
                                Pos2::new(tx, rect.bottom()),
                            ],
                            tick_stroke,
                        );
                    }
                    MacLevelOrientation::Vertical => {
                        let ty = inner.bottom() - inner.height() * t_frac;
                        painter.line_segment(
                            [
                                Pos2::new(rect.right() - tick_len, ty),
                                Pos2::new(rect.right(), ty),
                            ],
                            tick_stroke,
                        );
                    }
                }
            }
        }

        if let Some(peak) = self.peak_hold_value {
            let peak_frac = self.normalize(peak);
            let peak_color = self.color_for_value(theme, peak);
            let peak_stroke = Stroke::new(lm.peak_line_width.points(), peak_color);
            match self.orientation {
                MacLevelOrientation::Horizontal => {
                    let px = inner.left() + inner.width() * peak_frac;
                    painter.line_segment(
                        [Pos2::new(px, inner.top()), Pos2::new(px, inner.bottom())],
                        peak_stroke,
                    );
                }
                MacLevelOrientation::Vertical => {
                    let py = inner.bottom() - inner.height() * peak_frac;
                    painter.line_segment(
                        [Pos2::new(inner.left(), py), Pos2::new(inner.right(), py)],
                        peak_stroke,
                    );
                }
            }
        }

        response
    }
}
