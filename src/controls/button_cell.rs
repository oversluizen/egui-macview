//! Authentic macOS AppKit Push Button / Tool Button.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].
//! Fully data-driven: zero hardcoded colors or dimensions.

use crate::theme::MacTheme;
use egui::{Color32, CornerRadius, Response, Stroke, StrokeKind, Ui, Vec2, Widget};

/// Authentic macOS AppKit Push Button / Tool Button (`NSButtonCell` equivalent).
///
/// Features authentic theme-driven background, border, selection states,
/// typography metrics, and hover feedback.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacButtonCell, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
///
/// if ui.add(MacButtonCell::new("Save Changes").theme(&theme)).clicked() {
///     // Handle save
/// }
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacButtonCell<'a> {
    text: &'a str,
    selected: bool,
    enabled: bool,
    theme: Option<&'a MacTheme>,
    width: Option<f32>,
    height: Option<f32>,
    corner_radius: Option<f32>,
}

impl<'a> MacButtonCell<'a> {
    /// Creates a new button with label `text`.
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            selected: false,
            enabled: true,
            theme: None,
            width: None,
            height: None,
            corner_radius: None,
        }
    }

    /// Enables or disables the button cell.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets whether the button is presented in an active/selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Overrides button width in points (defaults to content width + padding).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides button height in points (defaults to token regular button height).
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Overrides corner radius in points (defaults to token corner radius).
    pub fn corner_radius(mut self, r: f32) -> Self {
        self.corner_radius = Some(r);
        self
    }
}

impl<'a> Widget for MacButtonCell<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;

        let metrics = &theme.button().regular;
        let height = self.height.unwrap_or_else(|| metrics.height.points());
        let corner_radius = self
            .corner_radius
            .unwrap_or_else(|| metrics.corner_radius.points());
        let padding_h = metrics
            .padding_horizontal
            .as_ref()
            .map(|d| d.points())
            .unwrap_or_else(|| metrics.font_size.points());

        let font_id = egui::FontId::proportional(metrics.font_size.points());
        let text_layout =
            ui.painter()
                .layout_no_wrap(self.text.to_owned(), font_id, Color32::PLACEHOLDER);
        let desired_width = self
            .width
            .unwrap_or_else(|| text_layout.size().x + padding_h * 2.0);

        let sense = if self.enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(desired_width, height), sense);
        let painter = ui.painter_at(rect);

        let alpha_factor = if self.enabled {
            1.0
        } else {
            theme.disabled_opacity()
        };

        // Dynamic AppKit styling (pressed feedback works even when selected == true)
        let base_bg = if !self.enabled {
            if self.selected {
                theme.selected_control_inactive()
            } else {
                theme.control_background()
            }
        } else if response.is_pointer_button_down_on() {
            theme.selected_control_inactive()
        } else if self.selected {
            theme.selected_control()
        } else if response.hovered() {
            theme.control()
        } else {
            theme.control_background()
        };
        let bg_color = base_bg.gamma_multiply(alpha_factor);

        let border_stroke = Stroke::new(
            metrics.border_width.points(),
            theme.separator().gamma_multiply(alpha_factor),
        );

        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            bg_color,
            border_stroke,
            StrokeKind::Inside,
        );

        let bg_style = theme.background_style(base_bg);
        let text_color = bg_style
            .primary_text_color(theme)
            .gamma_multiply(alpha_factor);

        let text_pos = egui::Align2::CENTER_CENTER
            .anchor_size(rect.center(), text_layout.size())
            .min;
        painter.galley(text_pos, text_layout, text_color);

        response
    }
}
