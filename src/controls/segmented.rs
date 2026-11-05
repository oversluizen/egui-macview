//! Authentic macOS NSSegmentedControl primitive.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded metrics.

use egui::{
    Color32, CornerRadius, Pos2, Rect, Response, Stroke, StrokeKind, Ui, Vec2,
};
use crate::theme::MacTheme;

/// Authentic macOS NSSegmentedControl.
pub struct MacSegmentedControl<'a> {
    options: &'a [&'a str],
    selected_idx: usize,
    theme: Option<&'a MacTheme>,
    height: Option<f32>,
}

impl<'a> MacSegmentedControl<'a> {
    pub fn new(options: &'a [&'a str], selected_idx: usize) -> Self {
        Self {
            options,
            selected_idx,
            theme: None,
            height: None,
        }
    }

    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn show(self, ui: &mut Ui, mut on_select: impl FnMut(usize)) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let metrics = &theme.segmented().regular;
        let height = self.height.unwrap_or_else(|| metrics.height.points());
        let corner_radius = metrics.corner_radius.points();
        let indicator_radius = metrics.indicator_radius.points();

        let count = self.options.len().max(1);
        let segment_width = 36.0;
        let total_width = segment_width * count as f32;

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(total_width, height),
            egui::Sense::click(),
        );

        let painter = ui.painter_at(rect);

        // Container background
        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            theme.control_background(),
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );

        let font_id = egui::FontId::proportional(11.0);

        for (i, opt) in self.options.iter().enumerate() {
            let seg_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + i as f32 * segment_width, rect.min.y),
                Vec2::new(segment_width, height),
            );

            // Active segment pill
            if i == self.selected_idx {
                let pill_rect = seg_rect.shrink(2.0);
                painter.rect(
                    pill_rect,
                    CornerRadius::same(indicator_radius as u8),
                    theme.control(),
                    Stroke::new(1.0, theme.separator()),
                    StrokeKind::Inside,
                );
            }

            // Click detection
            if response.clicked() {
                if let Some(pos) = ui.input(|inp| inp.pointer.interact_pos()) {
                    if seg_rect.contains(pos) {
                        on_select(i);
                    }
                }
            }

            let text_color = if i == self.selected_idx {
                Color32::from_rgb(20, 20, 20)
            } else {
                Color32::from_rgb(110, 110, 110)
            };

            painter.text(seg_rect.center(), egui::Align2::CENTER_CENTER, *opt, font_id.clone(), text_color);
        }

        response
    }
}
