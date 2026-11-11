//! Outline row content context for custom thumbnail, label, and accessory slots.

use crate::controls::thumbnail_cell::MacThumbnailCell;
use crate::theme::MacTheme;
use egui::{FontId, Id, Painter, Pos2, Rect, Response, Sense, Ui, Vec2};
use std::fmt::Debug;
use std::hash::Hash;

/// Outline row content context for custom thumbnail, label, and generic accessory slots.
pub struct MacOutlineRow<'a> {
    /// Underlying egui `Ui` context.
    pub ui: &'a mut Ui,
    /// Injected macOS theme.
    pub theme: &'a MacTheme,
    pub(crate) row_id: Id,
    pub(crate) row_rect: Rect,
    pub(crate) cursor_x: f32,
    pub(crate) accessory_cursor_right: f32,
    pub(crate) selected: bool,
    pub(crate) accessory_clicked: bool,
}

impl<'a> MacOutlineRow<'a> {
    pub(crate) fn new(
        ui: &'a mut Ui,
        theme: &'a MacTheme,
        row_id: Id,
        row_rect: Rect,
        cursor_x: f32,
        accessory_cursor_right: f32,
        selected: bool,
    ) -> Self {
        Self {
            ui,
            theme,
            row_id,
            row_rect,
            cursor_x,
            accessory_cursor_right,
            selected,
            accessory_clicked: false,
        }
    }

    /// Renders a thumbnail preview cell in the row.
    pub fn thumbnail(&mut self, drawer: impl FnOnce(&Painter, Rect)) {
        let size = self.theme.thumbnail().size.points();
        let rect = Rect::from_center_size(
            Pos2::new(self.cursor_x + size * 0.5, self.row_rect.center().y),
            Vec2::splat(size),
        );
        let painter = self.ui.painter_at(rect);
        MacThumbnailCell::new()
            .theme(self.theme)
            .custom_draw(drawer)
            .paint_in_rect(&painter, rect, false);
        self.cursor_x += size + self.theme.outline_view().content_spacing.points();
    }

    /// Renders the primary text label of the row with Apple HIG optical tracking and elision.
    pub fn label(&mut self, text: &str) {
        let spacing = self.theme.outline_view().content_spacing.points();
        let avail_w = (self.accessory_cursor_right - self.cursor_x - spacing).max(0.0);
        let label_rect = Rect::from_min_size(
            Pos2::new(self.cursor_x, self.row_rect.top()),
            Vec2::new(avail_w, self.row_rect.height()),
        );
        let painter = self.ui.painter_at(label_rect);
        let font_size = self.theme.outline_view().font_size.points();
        let font_id = FontId::proportional(font_size);
        let text_color = if self.selected {
            let bg_style = self.theme.selected_control_background_style();
            bg_style.primary_text_color(self.theme)
        } else {
            self.theme.label()
        };

        let format = self.theme.tracked_text_format(font_id, text_color);
        let mut job = egui::text::LayoutJob::single_section(text.to_string(), format);
        job.wrap.max_width = avail_w;
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = false;
        job.wrap.overflow_character = Some('…');
        let galley = painter.layout_job(job);
        let pos = Pos2::new(
            label_rect.left(),
            label_rect.center().y - galley.size().y * 0.5,
        );
        painter.galley(pos, galley, text_color);
    }

    /// Renders an AppKit Small Caps section header (uppercase with optical tracking and secondary label tint).
    pub fn header(&mut self, text: &str) {
        let spacing = self.theme.outline_view().content_spacing.points();
        let avail_w = (self.accessory_cursor_right - self.cursor_x - spacing).max(0.0);
        let label_rect = Rect::from_min_size(
            Pos2::new(self.cursor_x, self.row_rect.top()),
            Vec2::new(avail_w, self.row_rect.height()),
        );
        let font_size = self.theme.outline_view().header_font_size.points();
        let font_id = FontId::proportional(font_size);
        let text_color = self.theme.secondary_label();
        let upper = text.to_uppercase();
        let galley = self
            .theme
            .layout_tracked_galley(self.ui, &upper, font_id, text_color, avail_w);
        let painter = self.ui.painter_at(label_rect);
        let pos = Pos2::new(
            label_rect.left(),
            label_rect.center().y - galley.size().y * 0.5,
        );
        painter.galley(pos, galley, text_color);
    }

    /// Renders a generic, program-agnostic trailing accessory slot aligned to the right.
    /// Consumers supply their own vector drawer callbacks.
    pub fn accessory_custom(
        &mut self,
        id_source: impl Hash + Debug,
        active: &mut bool,
        drawer: impl FnOnce(&Painter, Rect, bool),
    ) -> Response {
        let metrics = self.theme.outline_view();
        let size = metrics.accessory_size.points();
        let spacing = metrics.accessory_spacing.points();

        self.accessory_cursor_right -= size;
        let rect = Rect::from_center_size(
            Pos2::new(
                self.accessory_cursor_right + size * 0.5,
                self.row_rect.center().y,
            ),
            Vec2::splat(size),
        );
        self.accessory_cursor_right -= spacing;

        let resp = self
            .ui
            .interact(rect, self.row_id.with(id_source), Sense::click());
        if resp.clicked() {
            *active = !*active;
            self.accessory_clicked = true;
        } else if resp.is_pointer_button_down_on() {
            self.accessory_clicked = true;
        }

        let painter = self.ui.painter_at(rect);
        drawer(&painter, rect, *active);

        resp
    }
}
