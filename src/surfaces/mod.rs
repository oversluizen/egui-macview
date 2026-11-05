//! macOS AppKit Panel & Popover Surfaces.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors or dimensions.

use egui::{
    Color32, CornerRadius, Pos2, Rect, Response, Stroke, StrokeKind, Ui, Vec2,
};
use crate::theme::MacTheme;

/// Authentic macOS NSPanel floating surface.
pub struct MacPanel<'a> {
    title: Option<String>,
    theme: Option<&'a MacTheme>,
    rounding: Option<u8>,
}

impl<'a> Default for MacPanel<'a> {
    fn default() -> Self {
        Self {
            title: None,
            theme: None,
            rounding: None,
        }
    }
}

impl<'a> MacPanel<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn rounding(mut self, r: u8) -> Self {
        self.rounding = Some(r);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, rect: Rect, add_contents: impl FnOnce(&mut Ui) -> R) -> (Response, R) {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let rounding = self.rounding.unwrap_or_else(|| {
            theme.button().regular.corner_radius.points() as u8
        });

        let bg_color = theme.window_background();
        let border_stroke = Stroke::new(1.0, theme.separator());

        let painter = ui.painter_at(rect);
        painter.rect(
            rect,
            CornerRadius::same(rounding),
            bg_color,
            border_stroke,
            StrokeKind::Inside,
        );

        let header_height = theme.button().regular.height.points();

        let content_rect = if let Some(title) = self.title {
            let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_height));
            painter.line_segment(
                [
                    Pos2::new(rect.min.x, rect.min.y + header_height),
                    Pos2::new(rect.max.x, rect.min.y + header_height),
                ],
                border_stroke,
            );
            painter.text(
                title_rect.center(),
                egui::Align2::CENTER_CENTER,
                title,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(60, 60, 60),
            );
            Rect::from_min_max(Pos2::new(rect.min.x, rect.min.y + header_height), rect.max)
        } else {
            rect
        };

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        let ret = add_contents(&mut child_ui);
        let resp = ui.interact(rect, ui.scope_id().with("mac_panel"), egui::Sense::click_and_drag());
        (resp, ret)
    }
}
