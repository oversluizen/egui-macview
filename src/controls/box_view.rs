//! Authentic macOS AppKit Grouping Box and Separator (`NSBox` equivalent, REQ-012-07).
//!
//! Grounded in Apple AppKit `NSBox` with primary grouped border insets
//! (`{top: 20.0, bottom: 10.0, left: 9.0, right: 9.0} pt`), $5.0\,\text{pt}$ corner radius,
//! and $1.0\,\text{pt}$ hairline separator primitive.

use crate::controls::MacInsets;
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    CornerRadius, Id, InnerResponse, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    UiBuilder, Vec2, WidgetText,
};

/// Authentic macOS AppKit Grouping Box container (`NSBox` equivalent).
pub struct MacBox<'a> {
    title: Option<WidgetText>,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacBox<'a> {
    /// Creates a new grouping box with an optional title.
    pub fn new(title: impl Into<WidgetText>) -> Self {
        Self {
            title: Some(title.into()),
            theme: None,
        }
    }

    /// Creates an untitled primary grouping box.
    pub fn untitled() -> Self {
        Self {
            title: None,
            theme: None,
        }
    }

    /// Injects dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Content padding insets from outer box border.
    pub fn content_insets(&self) -> MacInsets {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        let m = theme_cow.box_view();
        MacInsets {
            top: m.inset_top.points(),
            bottom: m.inset_bottom.points(),
            left: m.inset_left.points(),
            right: m.inset_right.points(),
        }
    }

    /// Grouping box corner radius in points ($5.0\,\text{pt}$).
    pub fn corner_radius(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        theme_cow.box_view().corner_radius.points()
    }

    /// Grouping box outer border stroke width ($1.0\,\text{pt}$).
    pub fn border_width(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        theme_cow.box_view().border_width.points()
    }

    /// Shows the grouping box container enclosing inner content.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let insets = self.content_insets();
        let corner_r = self.corner_radius();
        let border_w = self.border_width();

        let available_w = ui.available_width();
        let start_pos = ui.cursor().min;

        let content_rect = Rect::from_min_max(
            Pos2::new(start_pos.x + insets.left, start_pos.y + insets.top),
            Pos2::new(start_pos.x + available_w - insets.right, f32::INFINITY),
        );

        let mut child_ui = ui.new_child(
            UiBuilder::new()
                .id_salt(Id::NULL.with("mac_box_content"))
                .max_rect(content_rect),
        );
        let inner = add_contents(&mut child_ui);
        let content_min_rect = child_ui.min_rect();

        let box_h = insets.top + content_min_rect.height() + insets.bottom;
        let box_rect = Rect::from_min_size(start_pos, Vec2::new(available_w, box_h));
        let box_rect = subpixel_snap_rect(box_rect, ui.ctx().pixels_per_point());

        let (rect, response) = ui.allocate_exact_size(box_rect.size(), Sense::hover());
        let painter = ui.painter_at(rect);

        // Draw outer bezel frame
        painter.rect(
            rect,
            CornerRadius::same(corner_r as u8),
            theme.control_background(),
            Stroke::new(border_w, theme.separator()),
            StrokeKind::Inside,
        );

        // Draw title label in top-left inset if provided
        if let Some(title) = self.title {
            let galley = title.into_galley(
                ui,
                Some(egui::TextWrapMode::Extend),
                available_w,
                egui::TextStyle::Body,
            );
            let title_pos = Pos2::new(rect.min.x + insets.left, rect.min.y + insets.top * 0.5 - galley.size().y * 0.5);
            painter.galley(title_pos, galley, theme.label());
        }

        InnerResponse::new(inner, response)
    }

    /// Draws an authentic macOS horizontal hairline separator primitive inside a $5.0\,\text{pt}$ frame.
    pub fn separator(ui: &mut Ui) -> Response {
        let theme = MacTheme::resolve(None, ui.ctx());
        let frame_h = theme.box_view().separator_frame_height.points();
        let border_w = theme.box_view().border_width.points();

        let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), frame_h), Sense::hover());
        let painter = ui.painter_at(rect);
        let cy = rect.center().y;

        painter.line_segment(
            [Pos2::new(rect.min.x, cy), Pos2::new(rect.max.x, cy)],
            Stroke::new(border_w, theme.separator()),
        );

        response
    }
}
