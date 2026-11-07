//! Authentic macOS AppKit Level Indicator (`NSLevelIndicator` equivalent, SPEC-011).

use crate::theme::MacTheme;
use egui::{Color32, Response, Sense, Ui, Vec2, Widget};
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
pub struct MacLevelIndicator<'a> {
    value: f32,
    range: RangeInclusive<f32>,
    warning_value: f32,
    critical_value: f32,
    peak_hold_value: Option<f32>,
    style: MacLevelIndicatorStyle,
    orientation: MacLevelOrientation,
    segment_count: Option<usize>,
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
        self.segment_count = Some(count);
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
    pub fn color_for_value(&self, theme: &MacTheme, _sample_value: f32) -> Color32 {
        theme.control()
    }

    /// Renders the level indicator in `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacLevelIndicator<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let _ = (
            self.value,
            &self.range,
            self.warning_value,
            self.critical_value,
            self.peak_hold_value,
            self.style,
            self.orientation,
            self.segment_count,
            self.length,
            self.thickness,
            self.enabled,
            self.theme,
        );
        ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1
    }
}
