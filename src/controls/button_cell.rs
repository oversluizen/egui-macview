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
            theme: None,
            width: None,
            height: None,
            corner_radius: None,
        }
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
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

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
                .layout_no_wrap(self.text.to_owned(), font_id.clone(), Color32::WHITE);
        let desired_width = self
            .width
            .unwrap_or_else(|| text_layout.size().x + padding_h * 2.0);

        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(desired_width, height), egui::Sense::click());

        let painter = ui.painter_at(rect);

        // Dynamic AppKit styling
        let bg_color = if self.selected {
            theme.selected_control()
        } else if response.is_pointer_button_down_on() {
            theme.selected_control_inactive()
        } else if response.hovered() {
            theme.control()
        } else {
            theme.control_background()
        };

        let border_stroke = Stroke::new(metrics.border_width.points(), theme.separator());

        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            bg_color,
            border_stroke,
            StrokeKind::Inside,
        );

        let bg_style = theme.background_style(bg_color);
        let text_color = bg_style.primary_text_color(theme);

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            self.text,
            font_id,
            text_color,
        );

        response
    }
}
