//! Authentic macOS AppKit Hairline Split View (NSSplitView equivalent).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors or dimensions.

use crate::theme::MacTheme;
use egui::{
    Align, CornerRadius, CursorIcon, Id, Layout, Pos2, Rect, Stroke, StrokeKind, Ui, UiBuilder,
    Vec2,
};

/// Orientation of the split view divider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacSplitOrientation {
    /// Left and right panes divided by a vertical hairline (default).
    #[default]
    Horizontal,
    /// Top and bottom panes divided by a horizontal hairline.
    Vertical,
}

/// Authentic macOS AppKit split container (`NSSplitView` equivalent).
///
/// Features draggable hairline dividers, min/max pane constraints, collapse states,
/// optional drag handle indicators, and horizontal or vertical divider orientations.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacSplitOrientation, MacSplitView, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
///
/// MacSplitView::new("main_split", MacSplitOrientation::Horizontal)
///     .theme(&theme)
///     .min_first(100.0)
///     .show(ui,
///         |left_ui| { left_ui.label("Sidebar"); },
///         |right_ui| { right_ui.label("Detail Content"); },
///     );
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacSplitView<'a> {
    id: Id,
    orientation: MacSplitOrientation,
    pos_override: Option<&'a mut f32>,
    theme: Option<&'a MacTheme>,
    min_first: Option<f32>,
    max_first: Option<f32>,
    min_second: Option<f32>,
    collapsed_first: bool,
    collapsed_second: bool,
    show_handle: bool,
    default_ratio: Option<f32>,
}

impl<'a> MacSplitView<'a> {
    /// Creates a split view with state tracked automatically via `egui::Id`.
    pub fn new(
        id_source: impl std::hash::Hash + std::fmt::Debug,
        orientation: MacSplitOrientation,
    ) -> Self {
        let id = Id::NULL.with(id_source);
        Self {
            id,
            orientation,
            pos_override: None,
            theme: None,
            min_first: None,
            max_first: None,
            min_second: None,
            collapsed_first: false,
            collapsed_second: false,
            show_handle: true,
            default_ratio: None,
        }
    }

    /// Creates a split view binding directly to an external `&mut f32` position variable.
    pub fn with_pos(
        id_source: impl std::hash::Hash + std::fmt::Debug,
        orientation: MacSplitOrientation,
        pos: &'a mut f32,
    ) -> Self {
        let mut view = Self::new(id_source, orientation);
        view.pos_override = Some(pos);
        view
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Minimum size constraint for the first pane in points.
    pub fn min_first(mut self, min: f32) -> Self {
        self.min_first = Some(min);
        self
    }

    /// Maximum size constraint for the first pane in points.
    pub fn max_first(mut self, max: f32) -> Self {
        self.max_first = Some(max);
        self
    }

    /// Minimum size constraint for the second pane in points.
    pub fn min_second(mut self, min: f32) -> Self {
        self.min_second = Some(min);
        self
    }

    /// Collapses the first pane completely (width/height = 0).
    pub fn collapsed_first(mut self, collapsed: bool) -> Self {
        self.collapsed_first = collapsed;
        self
    }

    /// Collapses the second pane completely (width/height = 0).
    pub fn collapsed_second(mut self, collapsed: bool) -> Self {
        self.collapsed_second = collapsed;
        self
    }

    /// Controls visibility of the subtle center drag handle pill on the divider.
    pub fn show_handle(mut self, show: bool) -> Self {
        self.show_handle = show;
        self
    }

    /// Default split fraction if not yet persisted.
    pub fn default_ratio(mut self, ratio: f32) -> Self {
        self.default_ratio = Some(ratio.clamp(0.0, 1.0));
        self
    }

    /// Computes the layout geometry, handles divider drag interaction and painting,
    /// and returns `(first_rect, second_rect)` for caller-managed child rendering.
    pub fn split_panes(&mut self, ui: &mut Ui) -> (Rect, Rect) {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::from_ctx(ui.ctx());
                &fallback_theme
            }
        };

        let metrics = theme.split_view();
        let divider_thickness = metrics.divider_thickness.points();
        let hit_thickness = metrics.hit_thickness.points();
        let min_pane_size = metrics.min_pane_size.points();
        let handle_len = metrics.handle_length.points();
        let handle_th = metrics.handle_thickness.points();

        let container_rect = ui.available_rect_before_wrap();
        let total_span = match self.orientation {
            MacSplitOrientation::Horizontal => container_rect.width(),
            MacSplitOrientation::Vertical => container_rect.height(),
        };

        let default_ratio = self
            .default_ratio
            .unwrap_or_else(|| metrics.default_ratio.points());
        // Determine split position
        let mut pos = if let Some(ref p) = self.pos_override {
            **p
        } else {
            ui.data_mut(|d| *d.get_temp_mut_or_insert_with(self.id, || total_span * default_ratio))
        };

        let min_1 = self.min_first.unwrap_or(min_pane_size);
        let min_2 = self.min_second.unwrap_or(min_pane_size);
        let max_1 = self
            .max_first
            .unwrap_or(total_span - min_2)
            .min(total_span - min_2);
        pos = pos.clamp(min_1, max_1.max(min_1));

        let (first_rect, second_rect, divider_line, handle_rect) = if self.collapsed_first {
            let r2 = container_rect;
            let r1 = Rect::from_min_size(container_rect.min, Vec2::ZERO);
            (r1, r2, None, None)
        } else if self.collapsed_second {
            let r1 = container_rect;
            let r2 = Rect::from_min_size(container_rect.max, Vec2::ZERO);
            (r1, r2, None, None)
        } else {
            match self.orientation {
                MacSplitOrientation::Horizontal => {
                    let hit_rect = Rect::from_min_max(
                        Pos2::new(
                            container_rect.left() + pos - hit_thickness * 0.5,
                            container_rect.top(),
                        ),
                        Pos2::new(
                            container_rect.left() + pos + hit_thickness * 0.5,
                            container_rect.bottom(),
                        ),
                    );
                    let hit_resp = ui.interact(
                        hit_rect,
                        self.id.with("split_drag"),
                        egui::Sense::click_and_drag(),
                    );
                    if hit_resp.hovered() || hit_resp.dragged() {
                        ui.output_mut(|o| o.cursor_icon = CursorIcon::ResizeHorizontal);
                    }
                    if hit_resp.dragged() {
                        pos += hit_resp.drag_delta().x;
                        pos = pos.clamp(min_1, max_1.max(min_1));
                    }

                    let split_x = container_rect.left() + pos;
                    let divider_center_x = split_x + divider_thickness * 0.5;
                    let r1 = Rect::from_min_max(
                        container_rect.min,
                        Pos2::new(split_x.min(container_rect.right()), container_rect.bottom()),
                    );
                    let r2_left = (split_x + divider_thickness).min(container_rect.right());
                    let r2 = Rect::from_min_max(
                        Pos2::new(r2_left, container_rect.top()),
                        container_rect.max,
                    );

                    let line = (
                        Pos2::new(divider_center_x, container_rect.top()),
                        Pos2::new(divider_center_x, container_rect.bottom()),
                    );
                    let h_rect = if self.show_handle {
                        let center_y = container_rect.center().y;
                        Some(Rect::from_center_size(
                            Pos2::new(divider_center_x, center_y),
                            Vec2::new(handle_th, handle_len),
                        ))
                    } else {
                        None
                    };

                    (r1, r2, Some(line), h_rect)
                }
                MacSplitOrientation::Vertical => {
                    let hit_rect = Rect::from_min_max(
                        Pos2::new(
                            container_rect.left(),
                            container_rect.top() + pos - hit_thickness * 0.5,
                        ),
                        Pos2::new(
                            container_rect.right(),
                            container_rect.top() + pos + hit_thickness * 0.5,
                        ),
                    );
                    let hit_resp = ui.interact(
                        hit_rect,
                        self.id.with("split_drag"),
                        egui::Sense::click_and_drag(),
                    );
                    if hit_resp.hovered() || hit_resp.dragged() {
                        ui.output_mut(|o| o.cursor_icon = CursorIcon::ResizeVertical);
                    }
                    if hit_resp.dragged() {
                        pos += hit_resp.drag_delta().y;
                        pos = pos.clamp(min_1, max_1.max(min_1));
                    }

                    let split_y = container_rect.top() + pos;
                    let divider_center_y = split_y + divider_thickness * 0.5;
                    let r1 = Rect::from_min_max(
                        container_rect.min,
                        Pos2::new(container_rect.right(), split_y.min(container_rect.bottom())),
                    );
                    let r2_top = (split_y + divider_thickness).min(container_rect.bottom());
                    let r2 = Rect::from_min_max(
                        Pos2::new(container_rect.left(), r2_top),
                        container_rect.max,
                    );

                    let line = (
                        Pos2::new(container_rect.left(), divider_center_y),
                        Pos2::new(container_rect.right(), divider_center_y),
                    );
                    let h_rect = if self.show_handle {
                        let center_x = container_rect.center().x;
                        Some(Rect::from_center_size(
                            Pos2::new(center_x, divider_center_y),
                            Vec2::new(handle_len, handle_th),
                        ))
                    } else {
                        None
                    };

                    (r1, r2, Some(line), h_rect)
                }
            }
        };

        // Persist updated position
        if let Some(ref mut p) = self.pos_override {
            **p = pos;
        } else {
            ui.data_mut(|d| d.insert_temp(self.id, pos));
        }

        // Draw hairline and handle
        let painter = ui.painter_at(container_rect);
        if let Some((p1, p2)) = divider_line {
            painter.line_segment([p1, p2], Stroke::new(divider_thickness, theme.separator()));
        }
        if let Some(hr) = handle_rect {
            painter.rect(
                hr,
                CornerRadius::same((handle_th * 0.5) as u8),
                theme.control(),
                Stroke::NONE,
                StrokeKind::Inside,
            );
        }

        (first_rect, second_rect)
    }

    /// Renders the split container and both pane closures, returning their outputs.
    pub fn show<R1, R2>(
        mut self,
        ui: &mut Ui,
        first: impl FnOnce(&mut Ui) -> R1,
        second: impl FnOnce(&mut Ui) -> R2,
    ) -> (R1, R2) {
        let container_rect = ui.available_rect_before_wrap();
        let (first_rect, second_rect) = self.split_panes(ui);

        // Render panes as top-down vertical columns
        let mut ui1 = ui.new_child(
            UiBuilder::new()
                .max_rect(first_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        let r1 = first(&mut ui1);

        let mut ui2 = ui.new_child(
            UiBuilder::new()
                .max_rect(second_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        let r2 = second(&mut ui2);

        ui.advance_cursor_after_rect(container_rect);

        (r1, r2)
    }
}
