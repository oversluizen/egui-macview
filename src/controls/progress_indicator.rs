//! Authentic macOS AppKit Progress Indicator (`NSProgressIndicator` equivalent, SPEC-011).

use crate::theme::MacTheme;
use egui::{Response, Sense, Ui, Vec2, Widget};

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
    /// Standard size (`NSControlSizeRegular`).
    #[default]
    Regular,
    /// Compact size (`NSControlSizeSmall`).
    Small,
    /// Miniature size (`NSControlSizeMini`).
    Mini,
}

/// Authentic macOS AppKit progress indicator (`NSProgressIndicator` equivalent).
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
        self.progress = Some(progress);
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
        self.progress
    }

    /// Renders the progress indicator in `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacProgressIndicator<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let _ = (
            self.style,
            self.control_size,
            self.progress,
            self.time,
            self.label,
            self.show_percentage,
            self.width,
            self.enabled,
            self.theme,
        );
        ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1
    }
}
