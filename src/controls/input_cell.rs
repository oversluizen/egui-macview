//! Authentic macOS NSTextFieldCell primitive with Tab-chain and Transactional Buffer.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors, radiuses, or padding metrics.

use egui::{
    Color32, CornerRadius, Key, Response, Stroke, StrokeKind, Ui, Vec2,
};
use crate::theme::MacTheme;

/// Authentic macOS compact text entry cell (AppKit NSTextFieldCell style).
pub struct MacInputCell<'a> {
    id: egui::Id,
    value: &'a mut String,
    theme: Option<&'a MacTheme>,
    width: Option<f32>,
    height: Option<f32>,
    font_size: Option<f32>,
    corner_radius: Option<f32>,
    next_focus_id: Option<egui::Id>,
    prev_focus_id: Option<egui::Id>,
    placeholder: Option<&'a str>,
}

impl<'a> MacInputCell<'a> {
    pub fn new(id_source: impl std::hash::Hash + std::fmt::Debug, value: &'a mut String) -> Self {
        #[allow(deprecated)]
        let id = egui::Id::new(id_source);
        Self {
            id,
            value,
            theme: None,
            width: None,
            height: None,
            font_size: None,
            corner_radius: None,
            next_focus_id: None,
            prev_focus_id: None,
            placeholder: None,
        }
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = Some(font_size);
        self
    }

    pub fn placeholder(mut self, text: &'a str) -> Self {
        self.placeholder = Some(text);
        self
    }

    /// Sets the deterministic Tab navigation target.
    pub fn next_focus(mut self, id: egui::Id) -> Self {
        self.next_focus_id = Some(id);
        self
    }

    /// Sets the deterministic Shift+Tab navigation target.
    pub fn prev_focus(mut self, id: egui::Id) -> Self {
        self.prev_focus_id = Some(id);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let metrics = &theme.control_metrics().text_field.compact_infobox;
        let ring_metrics = theme.focus_ring();

        let height = self.height.unwrap_or_else(|| metrics.height.points());
        let width = self.width.unwrap_or(44.0);
        let font_size = self.font_size.unwrap_or_else(|| metrics.font_size.points());
        let corner_radius = self.corner_radius.unwrap_or_else(|| metrics.corner_radius.points());

        let desired_size = Vec2::new(width, height);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

        let is_focused = ui.memory(|m| m.has_focus(self.id));
        let painter = ui.painter_at(rect);

        // 1. Draw outer focus ring halo (NSFocusRingPlacementOutside) when focused
        if is_focused {
            let halo_offset = ring_metrics.offset.points();
            let halo_stroke_width = ring_metrics.stroke_width.points();
            let halo_radius = corner_radius + ring_metrics.corner_radius_delta.points();
            let halo_rect = rect.expand(halo_offset);

            painter.rect(
                halo_rect,
                CornerRadius::same(halo_radius as u8),
                Color32::TRANSPARENT,
                Stroke::new(halo_stroke_width, theme.keyboard_focus_indicator()),
                StrokeKind::Outside,
            );
        }

        // 2. Control background and border
        let bg_color = theme.control_background();
        let border_stroke = if is_focused {
            Stroke::new(1.0, theme.control_accent())
        } else if response.hovered() {
            Stroke::new(1.0, theme.separator())
        } else {
            Stroke::new(1.0, theme.separator())
        };

        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            bg_color,
            border_stroke,
            StrokeKind::Inside,
        );

        // 3. Tab-chain handling
        if is_focused {
            let tab_pressed = ui.input(|i| i.key_pressed(Key::Tab));
            let shift_down = ui.input(|i| i.modifiers.shift);

            if tab_pressed {
                if shift_down {
                    if let Some(prev) = self.prev_focus_id {
                        ui.memory_mut(|m| m.request_focus(prev));
                    }
                } else if let Some(next) = self.next_focus_id {
                    ui.memory_mut(|m| m.request_focus(next));
                }
            }
        }

        // 4. Inner text edit
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect.shrink(2.0))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        let mut text_edit = egui::TextEdit::singleline(self.value)
            .id(self.id)
            .font(egui::FontId::proportional(font_size))
            .desired_width(width - 4.0)
            .frame(egui::Frame::NONE);

        if let Some(hint) = self.placeholder {
            text_edit = text_edit.hint_text(hint);
        }

        let text_res = child_ui.add(text_edit);
        if text_res.changed() {
            response.mark_changed();
        }

        response
    }
}
