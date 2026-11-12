//! Authentic macOS AppKit Grouping Box and Separator (`NSBox` equivalent, REQ-012-07).
//!
//! Grounded in Apple AppKit `NSBox` with primary grouped border insets
//! (`{top: 20.0, bottom: 10.0, left: 9.0, right: 9.0} pt`), $5.0\,\text{pt}$ corner radius,
//! subtle grouped box background (`theme.box_background()`), and $1.0\,\text{pt}$ hairline separator.

use crate::controls::MacInsets;
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    CornerRadius, FontId, InnerResponse, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    UiBuilder, Vec2, WidgetText,
};

/// Authentic macOS AppKit Grouping Box container (`NSBox` equivalent).
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacBox, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
///
/// MacBox::new("Audio Output")
///     .theme(&theme)
///     .show(ui, |ui| {
///         ui.label("Sample Rate: 48.0 kHz");
///         MacBox::separator(ui);
///         ui.label("Bit Depth: 24-bit");
///     });
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacBox<'a> {
    title: Option<WidgetText>,
    theme: Option<&'a MacTheme>,
    fixed_rect: Option<Rect>,
}

impl<'a> MacBox<'a> {
    /// Creates a new grouping box with an optional title.
    pub fn new(title: impl Into<WidgetText>) -> Self {
        Self {
            title: Some(title.into()),
            theme: None,
            fixed_rect: None,
        }
    }

    /// Creates an untitled primary grouping box.
    pub fn untitled() -> Self {
        Self {
            title: None,
            theme: None,
            fixed_rect: None,
        }
    }

    /// Injects dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Pins the grouping box to an exact NIB coordinate rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Content padding insets from outer box border.
    pub fn content_insets(&self) -> MacInsets {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
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
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.box_view().corner_radius.points()
    }

    /// Grouping box outer border stroke width ($1.0\,\text{pt}$).
    pub fn border_width(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.box_view().border_width.points()
    }

    /// Paints the grouping box background, border, and optional title at `rect` without allocating flow space.
    pub fn paint_at(&self, ui: &mut Ui, rect: Rect) {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let insets = self.content_insets();
        let corner_r = self.corner_radius();
        let border_w = self.border_width();
        let m = theme.box_view();
        let snapped = subpixel_snap_rect(rect, ui.ctx().pixels_per_point());
        let box_bg = theme.box_background();
        theme.register_surface_rect(ui.ctx(), rect.union(snapped), box_bg);

        ui.painter().add(egui::epaint::RectShape::new(
            snapped,
            CornerRadius::same(corner_r as u8),
            box_bg,
            Stroke::new(border_w, theme.box_border()),
            StrokeKind::Inside,
        ));

        if let Some(ref title) = self.title {
            let font_id = FontId::proportional(m.title_font_size.points());
            let title_fg = theme.label_on(theme.box_background());
            let galley = theme.layout_tracked_galley(
                ui,
                title.text(),
                font_id,
                title_fg,
                snapped.width(),
            );
            let title_pos = Pos2::new(
                snapped.min.x + insets.left,
                snapped.min.y + insets.top * 0.5 - galley.size().y * 0.5,
            );
            ui.painter().galley(title_pos, galley, title_fg);
        }
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
        let title_font_sz = theme.box_view().title_font_size.points();

        let (start_pos, available_w, fixed_box_rect) = if let Some(fixed) = self.fixed_rect {
            (fixed.min, fixed.width(), Some(fixed))
        } else {
            (ui.cursor().min, ui.available_width(), None)
        };

        let content_max_y = fixed_box_rect.map_or(f32::INFINITY, |r| r.max.y - insets.bottom);
        let content_rect = Rect::from_min_max(
            Pos2::new(start_pos.x + insets.left, start_pos.y + insets.top),
            Pos2::new(start_pos.x + available_w - insets.right, content_max_y),
        );

        let bg_shape_idx = ui.painter().add(egui::Shape::Noop);
        let box_scope_id = ui.next_auto_id().with("mac_box_content");

        let mut child_ui = ui.new_child(
            UiBuilder::new()
                .id_salt(box_scope_id)
                .max_rect(content_rect),
        );
        let box_bg = theme.box_background();
        child_ui.visuals_mut().panel_fill = box_bg;
        let surface_slot = theme.register_surface_rect_slot(ui.ctx(), Rect::NOTHING, box_bg);
        let inner = add_contents(&mut child_ui);
        let content_min_rect = child_ui.min_rect();

        let (rect, response) = if let Some(fixed) = fixed_box_rect {
            let r = subpixel_snap_rect(fixed, ui.ctx().pixels_per_point());
            let resp = ui.interact(r, box_scope_id.with("fixed_box"), Sense::hover());
            (r, resp)
        } else {
            let box_h = insets.top + content_min_rect.height() + insets.bottom;
            let box_rect = Rect::from_min_size(start_pos, Vec2::new(available_w, box_h));
            let box_rect = subpixel_snap_rect(box_rect, ui.ctx().pixels_per_point());
            ui.allocate_exact_size(box_rect.size(), Sense::hover())
        };
        theme.update_surface_rect_slot(
            ui.ctx(),
            surface_slot,
            fixed_box_rect.map_or(rect, |f| f.union(rect)),
        );

        ui.painter().set(
            bg_shape_idx,
            egui::epaint::RectShape::new(
                rect,
                CornerRadius::same(corner_r as u8),
                box_bg,
                Stroke::new(border_w, theme.box_border()),
                StrokeKind::Inside,
            ),
        );

        if let Some(title) = self.title {
            let title_fg = theme.label_on(box_bg);
            let galley = theme.layout_tracked_galley(
                ui,
                title.text(),
                FontId::proportional(title_font_sz),
                title_fg,
                available_w,
            );
            let title_pos = Pos2::new(
                rect.min.x + insets.left,
                rect.min.y + insets.top * 0.5 - galley.size().y * 0.5,
            );
            ui.painter().galley(title_pos, galley, title_fg);
        }

        InnerResponse::new(inner, response)
    }

    /// Draws an authentic macOS horizontal hairline separator primitive inside a $5.0\,\text{pt}$ frame.
    pub fn separator(ui: &mut Ui) -> Response {
        let theme = MacTheme::resolve(None, ui.ctx());
        let frame_h = theme.box_view().separator_frame_height.points();
        let border_w = theme.box_view().border_width.points();

        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), frame_h), Sense::hover());
        let painter = ui.painter_at(rect);
        let cy = rect.center().y;

        painter.line_segment(
            [Pos2::new(rect.min.x, cy), Pos2::new(rect.max.x, cy)],
            Stroke::new(border_w, theme.separator()),
        );

        response
    }

    /// Paints a horizontal or vertical hairline separator at an exact NIB `rect`.
    pub fn separator_at(ui: &mut Ui, rect: Rect, theme: &MacTheme) {
        let border_w = theme.box_view().border_width.points();
        let stroke = Stroke::new(border_w, theme.separator());
        let painter = ui.painter();
        if rect.width() >= rect.height() {
            let cy = rect.center().y;
            painter.line_segment([Pos2::new(rect.min.x, cy), Pos2::new(rect.max.x, cy)], stroke);
        } else {
            let cx = rect.center().x;
            painter.line_segment([Pos2::new(cx, rect.min.y), Pos2::new(cx, rect.max.y)], stroke);
        }
    }
}
