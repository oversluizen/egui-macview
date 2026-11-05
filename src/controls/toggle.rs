//! Authentic macOS Toggle / Switch.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.

use egui::{
    Color32, CornerRadius, Pos2, Response, Stroke, StrokeKind, Ui, Vec2, Widget,
};
use crate::theme::MacTheme;

/// Authentic macOS Toggle / Switch.
pub struct MacToggle<'a> {
    value: &'a mut bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacToggle<'a> {
    pub fn new(value: &'a mut bool) -> Self {
        Self { value, theme: None }
    }

    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl<'a> Widget for MacToggle<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let height = 18.0;
        let width = 32.0;
        let radius = (height / 2.0) as u8;

        let desired_size = Vec2::new(width, height);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        if response.clicked() {
            *self.value = !*self.value;
            response.mark_changed();
        }

        let painter = ui.painter_at(rect);
        let bg_color = if *self.value {
            Color32::from_rgb(52, 199, 89) // macOS System Green
        } else {
            theme.control()
        };

        painter.rect(rect, CornerRadius::same(radius), bg_color, Stroke::new(1.0, theme.separator()), StrokeKind::Inside);

        let knob_radius = height / 2.0 - 1.5;
        let knob_x = if *self.value {
            rect.max.x - (height / 2.0)
        } else {
            rect.min.x + (height / 2.0)
        };
        let knob_center = Pos2::new(knob_x, rect.center().y);
        painter.circle_filled(knob_center, knob_radius, Color32::WHITE);

        response
    }
}
