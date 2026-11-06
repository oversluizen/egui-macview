//! Authentic macOS AppKit Image/Thumbnail Preview Cell (NSImageCell equivalent).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors or dimensions.

use crate::theme::MacTheme;
use egui::{CornerRadius, Painter, Rect, Response, Stroke, StrokeKind, Ui, Vec2, Widget};

/// Type alias for the custom closure drawing into the thumbnail cell.
pub type ThumbnailDrawer<'a> = Box<dyn FnOnce(&Painter, Rect) + 'a>;

/// Authentic macOS preview canvas cell (`NSImageCell` equivalent).
///
/// Provides a bounded canvas rectangle for rendering preview thumbnails, icons, or document previews
/// with authentic macOS corner rounding, borders, and selection highlight styling.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacTheme, MacThumbnailCell};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
///
/// MacThumbnailCell::new()
///     .theme(&theme)
///     .square(32.0)
///     .custom_draw(|painter, rect| {
///         painter.circle_filled(rect.center(), 8.0, egui::Color32::BLUE);
///     })
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
#[derive(Default)]
pub struct MacThumbnailCell<'a> {
    selected: bool,
    theme: Option<&'a MacTheme>,
    size: Option<Vec2>,
    corner_radius: Option<f32>,
    border_width: Option<f32>,
    drawer: Option<ThumbnailDrawer<'a>>,
}

impl<'a> MacThumbnailCell<'a> {
    /// Creates a new thumbnail preview cell.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether the thumbnail cell is in a selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets explicit width and height dimensions.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets explicit square dimensions (e.g. 24.0 for 24x24 pt).
    pub fn square(mut self, side: f32) -> Self {
        self.size = Some(Vec2::splat(side));
        self
    }

    /// Sets custom corner radius.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Sets custom border width.
    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = Some(width);
        self
    }

    /// Custom drawing callback inside the cell canvas rect.
    pub fn custom_draw(mut self, drawer: impl FnOnce(&Painter, Rect) + 'a) -> Self {
        self.drawer = Some(Box::new(drawer));
        self
    }

    /// Shows the thumbnail cell in the UI and returns its interaction response.
    pub fn show(self, ui: &mut Ui) -> Response {
        self.ui(ui)
    }

    /// Paints the thumbnail cell inside a specific rect using an existing painter.
    pub fn paint_in_rect(self, painter: &Painter, rect: Rect, hovered: bool) {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let metrics = theme.thumbnail();
        let corner_radius = self
            .corner_radius
            .unwrap_or_else(|| metrics.corner_radius.points());
        let border_width = self
            .border_width
            .unwrap_or_else(|| metrics.border_width.points());

        let bg_color = if self.selected {
            theme.selected_control()
        } else if hovered {
            theme.control()
        } else {
            theme.control_background()
        };

        let stroke_color = if self.selected {
            theme.control_accent()
        } else {
            theme.separator()
        };

        let border_stroke = Stroke::new(border_width, stroke_color);

        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            bg_color,
            border_stroke,
            StrokeKind::Inside,
        );

        if let Some(drawer) = self.drawer {
            let inner_rect = rect.shrink(border_width);
            drawer(painter, inner_rect);
        }
    }
}

impl<'a> Widget for MacThumbnailCell<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let metrics = theme.thumbnail();
        let default_side = metrics.size.points();
        let size = self.size.unwrap_or_else(|| Vec2::splat(default_side));

        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        let painter = ui.painter_at(rect);
        let hovered = response.hovered();

        self.paint_in_rect(&painter, rect, hovered);

        response
    }
}
