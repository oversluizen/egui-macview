//! macOS AppKit Panel & Popover Surfaces.

use egui::{Color32, CornerRadius, Pos2, Rect, Response, Stroke, StrokeKind, Ui, Vec2};

/// Authentic macOS NSPanel floating surface.
pub struct MacPanel {
    title: Option<String>,
    rounding: u8,
    bg_color: Color32,
    border_stroke: Stroke,
}

impl Default for MacPanel {
    fn default() -> Self {
        Self {
            title: None,
            rounding: 6,
            bg_color: Color32::from_rgb(246, 246, 246),
            border_stroke: Stroke::new(1.0, Color32::from_rgb(215, 215, 215)),
        }
    }
}

impl MacPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn show<R>(self, ui: &mut Ui, rect: Rect, add_contents: impl FnOnce(&mut Ui) -> R) -> (Response, R) {
        let painter = ui.painter_at(rect);
        // Frosted border & fill
        painter.rect(
            rect,
            CornerRadius::same(self.rounding),
            self.bg_color,
            self.border_stroke,
            StrokeKind::Inside,
        );

        let content_rect = if let Some(title) = self.title {
            let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 22.0));
            painter.line_segment(
                [Pos2::new(rect.min.x, rect.min.y + 22.0), Pos2::new(rect.max.x, rect.min.y + 22.0)],
                self.border_stroke,
            );
            painter.text(
                title_rect.center(),
                egui::Align2::CENTER_CENTER,
                title,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(60, 60, 60),
            );
            Rect::from_min_max(Pos2::new(rect.min.x, rect.min.y + 22.0), rect.max)
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
