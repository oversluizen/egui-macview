//! Compact Floating HUD Palette (`MacFloatingInfoBox`, `SPEC-006 §2.2`).
//!
//! Renders an authentic `178x63pt` (`compact_panel_width` x `compact_panel_height`)
//! floating dark translucent HUD window (`NSPanelStyleMaskHUDWindow`) with a `22pt`
//! draggable header bar and two `18pt` compact row slots.

use crate::controls::{MacControlSize, MacInputCell};
use crate::theme::MacTheme;
use egui::{
    Align, Align2, Context, CornerRadius, FontId, Id, Layout, Order, Pos2, Rect, Response,
    RichText, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2,
};
use std::hash::Hash;

/// Floating `178x63pt` Compact HUD Palette (`NSPanelStyleMaskHUDWindow`) from `SPEC-006 §2.2`.
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacFloatingInfoBox, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |_ui| {
/// let theme = MacTheme::new(MacAppearance::DarkAqua);
/// let mut hud_pos = egui::pos2(24.0, 24.0);
///
/// MacFloatingInfoBox::new("transform_hud", &mut hud_pos)
///     .title("Transform")
///     .theme(&theme)
///     .show(
///         &ctx,
///         |row1| { row1.label("X: 120 pt"); },
///         |row2| { row2.label("Y: 80 pt"); },
///     );
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacFloatingInfoBox<'a> {
    id: Id,
    title: String,
    pos: &'a mut Pos2,
    bounds: Option<Rect>,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacFloatingInfoBox<'a> {
    /// Creates a new `MacFloatingInfoBox` with `id_source` anchored at mutable `pos`.
    pub fn new(id_source: impl Hash + std::fmt::Debug, pos: &'a mut Pos2) -> Self {
        Self {
            id: Id::NULL.with(id_source),
            title: "Compact HUD".to_string(),
            pos,
            bounds: None,
            theme: None,
        }
    }

    /// Sets an explicit widget `Id` for the HUD palette.
    pub fn id(mut self, id_source: impl Hash + std::fmt::Debug) -> Self {
        self.id = Id::NULL.with(id_source);
        self
    }

    /// Sets the header title of the HUD palette.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Constrains dragging of `pos` within `bounds`.
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the `MacTheme`.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the two-row compact HUD palette inside an `Order::Foreground` area.
    pub fn show(
        self,
        ctx: &Context,
        row1: impl FnOnce(&mut Ui),
        row2: impl FnOnce(&mut Ui),
    ) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ctx);
        let theme: &MacTheme = &theme_cow;

        let pop_metrics = theme.popover_metrics();
        let compact_tf = &theme.text_field().compact_infobox;
        let hud_w = pop_metrics.compact_panel_width.points();
        let hud_h = pop_metrics.compact_panel_height.points();
        let header_h = pop_metrics.header_height.points();
        let corner_r = pop_metrics.corner_radius.points() as u8;
        let border_w = pop_metrics.border_width.points();
        let header_font_pt = pop_metrics.header_font_size.points();
        let row_h = compact_tf.height.points();
        let row_gap = compact_tf
            .padding_vertical
            .as_ref()
            .map(|d| d.points())
            .unwrap_or(border_w * 2.0);

        egui::Area::new(self.id)
            .order(Order::Foreground)
            .sense(Sense::click_and_drag())
            .fixed_pos(*self.pos)
            .show(ctx, |ui| {
                let (hud_rect, hud_resp) =
                    ui.allocate_exact_size(Vec2::new(hud_w, hud_h), Sense::click_and_drag());

                let pointer_inside = ui.rect_contains_pointer(hud_rect)
                    || ui.input(|i| i.pointer.hover_pos().is_some_and(|p| hud_rect.contains(p)));
                if pointer_inside {
                    ui.input_mut(|i| {
                        i.smooth_scroll_delta = Vec2::ZERO;
                    });
                }

                let header_rect = Rect::from_min_size(hud_rect.min, Vec2::new(hud_w, header_h));
                let header_resp = ui.interact(
                    header_rect,
                    self.id.with("hud_header_drag"),
                    Sense::click_and_drag(),
                );
                let drag_delta = if header_resp.dragged() {
                    header_resp.drag_delta()
                } else if hud_resp.dragged() {
                    hud_resp.drag_delta()
                } else {
                    Vec2::ZERO
                };

                if drag_delta != Vec2::ZERO {
                    let mut next_pos = *self.pos + drag_delta;
                    if let Some(b) = self.bounds {
                        let max_x = (b.max.x - hud_w).max(b.min.x);
                        let max_y = (b.max.y - hud_h).max(b.min.y);
                        next_pos.x = next_pos.x.clamp(b.min.x, max_x);
                        next_pos.y = next_pos.y.clamp(b.min.y, max_y);
                    }
                    *self.pos = next_pos;
                }

                let painter = ui.painter_at(hud_rect);
                let border_stroke = Stroke::new(border_w, theme.hud_border());
                painter.rect(
                    hud_rect,
                    CornerRadius::same(corner_r),
                    theme.hud_background(),
                    border_stroke,
                    StrokeKind::Inside,
                );

                // Header bar separator & centered title
                let sep_y = hud_rect.min.y + header_h;
                painter.line_segment(
                    [
                        Pos2::new(hud_rect.min.x, sep_y),
                        Pos2::new(hud_rect.max.x, sep_y),
                    ],
                    border_stroke,
                );
                painter.text(
                    header_rect.center(),
                    Align2::CENTER_CENTER,
                    &self.title,
                    FontId::proportional(header_font_pt),
                    theme.hud_secondary_text(),
                );

                // Row 1 and Row 2 layout inside body
                let side_inset = row_gap * 2.0;
                let row1_top = sep_y + row_gap;
                let row1_rect = Rect::from_min_size(
                    Pos2::new(hud_rect.min.x + side_inset, row1_top),
                    Vec2::new(hud_w - side_inset * 2.0, row_h),
                );
                let row2_top = row1_top + row_h + row_gap;
                let row2_rect = Rect::from_min_size(
                    Pos2::new(hud_rect.min.x + side_inset, row2_top),
                    Vec2::new(hud_w - side_inset * 2.0, row_h),
                );

                let mut row1_ui = ui.new_child(
                    UiBuilder::new()
                        .id_salt((self.id, "hud_row1"))
                        .max_rect(row1_rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );
                row1(&mut row1_ui);

                let mut row2_ui = ui.new_child(
                    UiBuilder::new()
                        .id_salt((self.id, "hud_row2"))
                        .max_rect(row2_rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );
                row2(&mut row2_ui);

                hud_resp
            })
            .inner
    }

    /// Renders the standard X/Y and W/H 2x2 coordinate HUD palette (`SPEC-006 §2.2`).
    pub fn show_xywh(
        self,
        ctx: &Context,
        x_val: &mut String,
        y_val: &mut String,
        w_val: &mut String,
        h_val: &mut String,
    ) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ctx);
        let theme_ref: &MacTheme = &theme_cow;
        let hud_id = self.id;
        let cell_w = theme_ref
            .text_field()
            .compact_infobox
            .default_width
            .points();
        let font_pt = theme_ref.text_field().compact_infobox.font_size.points();
        let label_col = theme_ref.hud_secondary_text();

        let id_x = hud_id.with("hud_x");
        let id_y = hud_id.with("hud_y");
        let id_w = hud_id.with("hud_w");
        let id_h = hud_id.with("hud_h");

        let draw_cell = |ui: &mut Ui, lbl: &str, id: Id, val: &mut String, prev: Id, next: Id| {
            ui.label(
                RichText::new(lbl)
                    .font(FontId::proportional(font_pt))
                    .color(label_col),
            );
            MacInputCell::new(id, val)
                .theme(theme_ref)
                .control_size(MacControlSize::CompactInfobox)
                .width(cell_w)
                .prev_focus(prev)
                .next_focus(next)
                .show(ui);
        };

        self.show(
            ctx,
            |r1| {
                draw_cell(r1, "X", id_x, x_val, id_h, id_y);
                draw_cell(r1, "Y", id_y, y_val, id_x, id_w);
            },
            |r2| {
                draw_cell(r2, "W", id_w, w_val, id_y, id_h);
                draw_cell(r2, "H", id_h, h_val, id_w, id_x);
            },
        )
    }
}
