//! Authentic macOS AppKit Multi-Line Text & Code View (`NSTextView` inside `NSScrollView`, REQ-013-02A).
//!
//! Provides a scrollable multi-line text and code editor with AppKit `1.0pt` scroll-border framing,
//! configurable proportional or monospace typography, optional line-number gutter, and read-only mode.

use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{CornerRadius, FontFamily, FontId, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, TextEdit, Ui, UiBuilder, Vec2};

/// Authentic macOS AppKit `NSTextView` multi-line scrollable text and code editor.
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacTextView, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let mut source = "feature liga {\n  sub f i by f_i;\n} liga;".to_string();
/// MacTextView::new("fea_editor", &mut source)
///     .theme(&theme)
///     .monospace(true)
///     .line_numbers(true)
///     .show(ui);
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacTextView<'a> {
    id: Id,
    text: &'a mut String,
    monospace: bool,
    line_numbers: bool,
    editable: bool,
    fixed_rect: Option<Rect>,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacTextView<'a> {
    /// Creates a new multi-line text view bound to `text`.
    pub fn new(id_source: impl std::hash::Hash + std::fmt::Debug, text: &'a mut String) -> Self {
        Self { id: Id::NULL.with(id_source), text, monospace: false, line_numbers: false, editable: true, fixed_rect: None, theme: None }
    }

    /// Enables or disables monospace code typography (`FontFamily::Monospace`).
    pub fn monospace(mut self, monospace: bool) -> Self {
        self.monospace = monospace;
        self
    }

    /// Shows or hides the leading line-number gutter.
    pub fn line_numbers(mut self, show: bool) -> Self {
        self.line_numbers = show;
        self
    }

    /// Sets whether the text view is editable or read-only.
    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// Pins the text view to an exact NIB or scene layout rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Injects dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the multi-line text view inside `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let m = theme.text_view();

        let border_w = m.border_width.points();
        let corner_r = m.corner_radius.points() as u8;
        let pad_x = m.padding_horizontal.points();
        let pad_y = m.padding_vertical.points();
        let gutter_w = m.gutter_width.points();
        let line_h = m.line_height.points();
        let font_sz = if self.monospace { m.mono_font_size.points() } else { m.font_size.points() };

        let outer_rect = if let Some(fixed) = self.fixed_rect {
            subpixel_snap_rect(fixed, ui.ctx().pixels_per_point())
        } else {
            let w = ui.available_width();
            let h = gutter_w * 2.0 + pad_y * 2.0;
            let (r, _) = ui.allocate_exact_size(Vec2::new(w, h), Sense::hover());
            subpixel_snap_rect(r, ui.ctx().pixels_per_point())
        };

        let bg = theme.control_background();
        theme.register_surface_rect(ui.ctx(), outer_rect, bg);
        ui.painter().rect(outer_rect, CornerRadius::same(corner_r), bg, Stroke::new(border_w, theme.box_border()), StrokeKind::Inside);

        let text_left = if self.line_numbers {
            let gutter_rect =
                Rect::from_min_max(outer_rect.min, Pos2::new((outer_rect.min.x + gutter_w).min(outer_rect.max.x), outer_rect.max.y));
            ui.painter().rect_filled(gutter_rect, CornerRadius::same(corner_r), theme.box_background());
            ui.painter().line_segment(
                [Pos2::new(gutter_rect.max.x, outer_rect.min.y), Pos2::new(gutter_rect.max.x, outer_rect.max.y)],
                Stroke::new(border_w, theme.separator()),
            );

            let line_count = self.text.lines().count().max(1);
            let num_fg = theme.secondary_label_on(theme.box_background());
            let mut y = outer_rect.min.y + pad_y;
            for idx in 1..=line_count {
                if y + line_h > outer_rect.max.y {
                    break;
                }
                let num_str = idx.to_string();
                let galley = theme.layout_tracked_galley(ui, &num_str, FontId::monospace(font_sz), num_fg, gutter_w);
                let x = (gutter_rect.max.x - pad_x - galley.size().x).max(gutter_rect.min.x);
                ui.painter().galley(Pos2::new(x, y), galley, num_fg);
                y += line_h;
            }
            gutter_rect.max.x + pad_x
        } else {
            outer_rect.min.x + pad_x
        };

        let inner_rect = Rect::from_min_max(
            Pos2::new(text_left, outer_rect.min.y + pad_y),
            Pos2::new((outer_rect.max.x - pad_x).max(text_left), (outer_rect.max.y - pad_y).max(outer_rect.min.y + pad_y)),
        );

        let font_id = if self.monospace { FontId::new(font_sz, FontFamily::Monospace) } else { FontId::proportional(font_sz) };
        let text_fg = theme.label_on(bg);

        let mut child_ui = ui.new_child(UiBuilder::new().id_salt(self.id.with("text_view_inner")).max_rect(inner_rect));
        child_ui.set_clip_rect(inner_rect.intersect(ui.clip_rect()));
        let edit = TextEdit::multiline(self.text)
            .id(self.id)
            .font(font_id)
            .text_color(text_fg)
            .frame(egui::Frame::NONE)
            .interactive(self.editable && ui.is_enabled())
            .desired_width(inner_rect.width());
        child_ui.add(edit)
    }
}
