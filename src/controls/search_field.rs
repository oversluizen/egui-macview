//! Authentic macOS AppKit Search Field (NSSearchField equivalent).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors or dimensions.

use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    Align2, CornerRadius, FontId, Key, Modifiers, Pos2, Rect, Response, Sense, Stroke, StrokeKind,
    TextEdit, Ui, Vec2, Widget,
};

/// Authentic macOS AppKit Search Field (`NSSearchField` equivalent).
///
/// Features authentic pill rounding, embedded magnifying glass icon,
/// clickable clear button (with 'x' icon), Escape key cancellation, and theme styling.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacSearchField, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut query = String::new();
///
/// MacSearchField::new(&mut query)
///     .theme(&theme)
///     .placeholder("Search files...")
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacSearchField<'a> {
    id: Option<egui::Id>,
    text: &'a mut String,
    placeholder: &'a str,
    theme: Option<&'a MacTheme>,
    width: Option<f32>,
    height: Option<f32>,
    corner_radius: Option<f32>,
    enabled: bool,
}

impl<'a> MacSearchField<'a> {
    /// Creates a new search field bound to a mutable string buffer.
    pub fn new(text: &'a mut String) -> Self {
        Self {
            id: None,
            text,
            placeholder: "Search",
            theme: None,
            width: None,
            height: None,
            corner_radius: None,
            enabled: true,
        }
    }

    /// Sets explicit widget identifier for focus management.
    pub fn id(mut self, id: impl Into<egui::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets placeholder prompt text.
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Sets the dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Overrides default width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides default height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Overrides default corner radius.
    pub fn corner_radius(mut self, r: f32) -> Self {
        self.corner_radius = Some(r);
        self
    }

    /// Enables or disables the control.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Renders the search field inside the specified UI container.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacSearchField<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::from_ctx(ui.ctx());
                &fallback_theme
            }
        };

        let metrics = theme.search_field();
        let height = self.height.unwrap_or_else(|| metrics.height.points());
        let corner_radius = self
            .corner_radius
            .unwrap_or_else(|| metrics.corner_radius.points());
        let icon_size = metrics.icon_size.points();
        let icon_stroke = metrics.icon_stroke_width.points();
        let padding_h = metrics.padding_horizontal.points();
        let clear_size = metrics.clear_button_size.points();

        let desired_width = self.width.unwrap_or_else(|| metrics.default_width.points());
        let (raw_rect, response) =
            ui.allocate_exact_size(Vec2::new(desired_width, height), Sense::click());
        let rect = subpixel_snap_rect(raw_rect, ui.ctx().pixels_per_point());

        let has_text_initial = !self.text.is_empty();
        let clear_rect = if has_text_initial {
            let cx = rect.right() - padding_h - clear_size * 0.5;
            let cy = rect.center().y;
            Rect::from_center_size(Pos2::new(cx, cy), Vec2::splat(clear_size))
        } else {
            Rect::NOTHING
        };

        let text_id = self
            .id
            .unwrap_or_else(|| response.id.with("mac_search_input"));
        let clear_id = response.id.with("mac_search_clear");
        let clear_response = if has_text_initial && self.enabled {
            ui.interact(clear_rect, clear_id, Sense::click())
        } else {
            ui.interact(Rect::NOTHING, clear_id, Sense::hover())
        };

        let mut text_changed = false;
        if clear_response.clicked() {
            self.text.clear();
            text_changed = true;
            ui.ctx().memory_mut(|m| m.request_focus(text_id));
        } else if response.clicked() && self.enabled {
            ui.ctx().memory_mut(|m| m.request_focus(text_id));
        }

        // Handle Escape key and consume it BEFORE TextEdit processes it
        let was_focused_id = response.id.with("mac_search_was_focused");
        let was_focused = ui.data(|d| d.get_temp::<bool>(was_focused_id).unwrap_or(false));
        let currently_focused = ui.ctx().memory(|m| m.has_focus(text_id));
        let is_initially_focused = currently_focused || was_focused;
        let mut surrendered_by_escape = false;

        if is_initially_focused && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            if !self.text.is_empty() {
                self.text.clear();
                text_changed = true;
                ui.ctx().memory_mut(|m| m.request_focus(text_id));
            } else {
                ui.ctx().memory_mut(|m| m.surrender_focus(text_id));
                surrendered_by_escape = true;
            }
        }

        let has_text = !self.text.is_empty();
        let is_focused_pre = !surrendered_by_escape
            && (ui.ctx().memory(|m| m.has_focus(text_id)) || response.has_focus());

        let content_gap = metrics.content_spacing.points();
        let text_min_x = rect.left() + padding_h + icon_size + content_gap;
        let text_max_x = if has_text {
            rect.right() - padding_h - clear_size - content_gap
        } else {
            rect.right() - padding_h
        };
        let v_padding = metrics.padding_vertical.points();
        let text_rect = Rect::from_min_max(
            Pos2::new(text_min_x, rect.top() + v_padding),
            Pos2::new(text_max_x.max(text_min_x), rect.bottom() - v_padding),
        );
        let font_id = FontId::proportional(metrics.font_size.points());

        // 1. Paint pill background, border, focus ring, and leading icon BEFORE TextEdit so text is never obscured!
        let painter = ui.painter_at(rect);
        let bg_color = theme.control_background();
        let border_stroke = if is_focused_pre {
            Stroke::new(metrics.border_width.points(), theme.control_accent())
        } else {
            Stroke::new(metrics.border_width.points(), theme.separator())
        };

        if self.enabled && is_focused_pre {
            let ring = theme.focus_ring();
            let focus_rect = rect.expand(ring.offset.points());
            let focus_radius = corner_radius + ring.corner_radius_delta.points();
            painter.rect_stroke(
                focus_rect,
                CornerRadius::same(focus_radius as u8),
                Stroke::new(ring.stroke_width.points(), theme.keyboard_focus_indicator()),
                StrokeKind::Outside,
            );
        }

        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            bg_color,
            border_stroke,
            StrokeKind::Inside,
        );

        let icon_cx = rect.left() + padding_h + icon_size * 0.5;
        let icon_cy = rect.center().y;
        let glass_r = metrics.glass_radius.points();
        let stroke = Stroke::new(icon_stroke, theme.placeholder_text());
        let icon_offset = metrics.border_width.points();
        let ring_center = Pos2::new(icon_cx - icon_offset, icon_cy - icon_offset);
        painter.circle_stroke(ring_center, glass_r, stroke);

        let handle_start = Pos2::new(ring_center.x + glass_r * 0.5, ring_center.y + glass_r * 0.5);
        let handle_end = Pos2::new(icon_cx + icon_size * 0.5, icon_cy + icon_size * 0.5);
        painter.line_segment([handle_start, handle_end], stroke);

        if !has_text && !is_focused_pre {
            painter.text(
                Pos2::new(text_min_x, rect.center().y),
                Align2::LEFT_CENTER,
                self.placeholder,
                font_id.clone(),
                theme.placeholder_text(),
            );
        }

        // 2. Render TextEdit AFTER background bezel so text and cursor sit on top
        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));
        let edit_response = child_ui.add_sized(
            text_rect.size(),
            TextEdit::singleline(self.text)
                .id(text_id)
                .font(font_id)
                .text_color(if self.enabled {
                    theme.label()
                } else {
                    theme.placeholder_text()
                })
                .frame(egui::Frame::NONE)
                .interactive(self.enabled),
        );

        if edit_response.changed() {
            text_changed = true;
        }

        let is_text_focused = !surrendered_by_escape
            && (edit_response.has_focus() || ui.ctx().memory(|m| m.has_focus(text_id)));
        let is_focused = is_text_focused || (!surrendered_by_escape && response.has_focus());
        ui.data_mut(|d| d.insert_temp(was_focused_id, is_focused));

        // 3. Draw trailing clear button if text is present
        if !self.text.is_empty() && clear_rect != Rect::NOTHING {
            let clear_painter = ui.painter_at(clear_rect);
            let btn_bg = if clear_response.is_pointer_button_down_on() {
                theme.control_accent()
            } else if clear_response.hovered() {
                theme.separator()
            } else {
                theme.placeholder_text()
            };

            clear_painter.circle_filled(clear_rect.center(), clear_size * 0.5, btn_bg);

            let x_span = metrics.clear_button_icon_padding.points();
            let cc = clear_rect.center();
            let x_color = theme.background_style(btn_bg).primary_text_color(theme);
            let x_stroke = Stroke::new(icon_stroke, x_color);
            clear_painter.line_segment(
                [
                    Pos2::new(cc.x - x_span, cc.y - x_span),
                    Pos2::new(cc.x + x_span, cc.y + x_span),
                ],
                x_stroke,
            );
            clear_painter.line_segment(
                [
                    Pos2::new(cc.x - x_span, cc.y + x_span),
                    Pos2::new(cc.x + x_span, cc.y - x_span),
                ],
                x_stroke,
            );
        }

        let mut final_response = edit_response.union(response);
        if text_changed {
            final_response.mark_changed();
        }

        final_response
    }
}
