//! Authentic macOS AppKit Hierarchical Outline View (NSOutlineView equivalent).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors or dimensions.

use crate::controls::thumbnail_cell::MacThumbnailCell;
use crate::theme::MacTheme;
use egui::{
    Align2, CornerRadius, FontId, Id, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    Vec2,
};
use std::fmt::Debug;
use std::hash::Hash;

/// Interaction response from an outline row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacOutlineRowResponse {
    /// True if the row body was clicked (excluding disclosure chevron click).
    pub clicked: bool,
    /// True if the row was double clicked.
    pub double_clicked: bool,
    /// True if the disclosure triangle was clicked.
    pub disclosure_clicked: bool,
    /// True if the row is currently in an expanded state.
    pub is_expanded: bool,
    /// True if the row is currently selected.
    pub is_selected: bool,
}

/// Authentic macOS AppKit hierarchical outline tree view (`NSOutlineView` equivalent).
///
/// Features hierarchical indentation steps, disclosure chevron toggling, selection styling,
/// thumbnail preview slots, and trailing accessory action buttons.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacOutlineView, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut expanded = true;
///
/// MacOutlineView::new("sidebar_outline")
///     .theme(&theme)
///     .show(ui, |tree| {
///         tree.row("row_1", 0, true, &mut expanded, false, |row| {
///             row.label("Documents");
///         });
///     });
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacOutlineView<'a> {
    id: Id,
    theme: Option<&'a MacTheme>,
    row_height: Option<f32>,
    indent_step: Option<f32>,
}

impl<'a> MacOutlineView<'a> {
    /// Creates a new `MacOutlineView` identified by `id_source`.
    pub fn new(id_source: impl Hash + Debug) -> Self {
        #[allow(deprecated)]
        let id = Id::new(id_source);
        Self {
            id,
            theme: None,
            row_height: None,
            indent_step: None,
        }
    }

    /// Unique identifier for this outline view.
    pub fn id(&self) -> Id {
        self.id
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets explicit row height in points.
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = Some(height);
        self
    }

    /// Sets explicit horizontal indentation step per depth level.
    pub fn indent_step(mut self, step: f32) -> Self {
        self.indent_step = Some(step);
        self
    }

    /// Renders the outline view container with a tree builder closure.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut MacOutlineTree<'_>) -> R) -> R {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |child_ui| {
            let mut tree = MacOutlineTree {
                id: self.id,
                ui: child_ui,
                theme,
                row_height: self
                    .row_height
                    .unwrap_or_else(|| theme.outline_view().row_height.points()),
                indent_step: self
                    .indent_step
                    .unwrap_or_else(|| theme.outline_view().indent_step.points()),
            };

            content(&mut tree)
        })
        .inner
    }
}

/// Outline tree builder passed to the show closure.
pub struct MacOutlineTree<'t> {
    /// Unique identifier of the outline tree.
    pub id: Id,
    /// Underlying egui `Ui` context.
    pub ui: &'t mut Ui,
    /// Injected macOS theme.
    pub theme: &'t MacTheme,
    row_height: f32,
    indent_step: f32,
}

impl<'t> MacOutlineTree<'t> {
    /// Renders a single outline row with hierarchical indentation and accessory buttons.
    pub fn row(
        &mut self,
        id_source: impl Hash + Debug,
        depth: usize,
        expandable: bool,
        expanded: &mut bool,
        selected: bool,
        content: impl FnOnce(&mut MacOutlineRow<'_>),
    ) -> MacOutlineRowResponse {
        #[allow(deprecated)]
        let row_id = Id::new(id_source);
        let avail_w = self.ui.available_width();
        let (row_rect, row_resp) = self
            .ui
            .allocate_exact_size(Vec2::new(avail_w, self.row_height), Sense::click());

        let metrics = self.theme.outline_view();
        let corner_radius = metrics.corner_radius.points();
        let painter = self.ui.painter_at(row_rect);

        // 1. Row selection or hover background
        if selected {
            painter.rect(
                row_rect,
                CornerRadius::same(corner_radius as u8),
                self.theme.selected_control(),
                Stroke::NONE,
                StrokeKind::Inside,
            );
        } else if row_resp.hovered() {
            painter.rect(
                row_rect,
                CornerRadius::same(corner_radius as u8),
                self.theme.control(),
                Stroke::NONE,
                StrokeKind::Inside,
            );
        }

        // 2. Hierarchical indentation and disclosure chevron
        let indent_offset = depth as f32 * self.indent_step;
        let disc_size = metrics.disclosure_size.points();
        let disc_margin = metrics.disclosure_margin.points();
        let disc_x = row_rect.left() + indent_offset + disc_margin;
        let disc_rect = Rect::from_center_size(
            Pos2::new(disc_x + disc_size * 0.5, row_rect.center().y),
            Vec2::splat(disc_size + disc_margin),
        );

        let mut disclosure_clicked = false;
        if expandable {
            let disc_resp = self
                .ui
                .interact(disc_rect, row_id.with("disclosure"), Sense::click());
            if disc_resp.clicked() {
                *expanded = !*expanded;
                disclosure_clicked = true;
            }

            // Draw disclosure chevron
            let center = disc_rect.center();
            let half = metrics.chevron_size.points() * 0.5;
            let depth = metrics.chevron_depth.points();
            let chevron_stroke = Stroke::new(
                metrics.chevron_stroke_width.points(),
                if selected {
                    self.theme.control_accent()
                } else {
                    self.theme.placeholder_text()
                },
            );
            if *expanded {
                // Chevron pointing down
                painter.line_segment(
                    [
                        Pos2::new(center.x - half, center.y - depth),
                        Pos2::new(center.x, center.y + depth),
                    ],
                    chevron_stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x, center.y + depth),
                        Pos2::new(center.x + half, center.y - depth),
                    ],
                    chevron_stroke,
                );
            } else {
                // Chevron pointing right
                painter.line_segment(
                    [
                        Pos2::new(center.x - depth, center.y - half),
                        Pos2::new(center.x + depth, center.y),
                    ],
                    chevron_stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x + depth, center.y),
                        Pos2::new(center.x - depth, center.y + half),
                    ],
                    chevron_stroke,
                );
            }
        }

        // 3. Row content & accessories
        let content_spacing = metrics.content_spacing.points();
        let accessory_spacing = metrics.accessory_spacing.points();
        let content_left = disc_x + disc_size + content_spacing;
        let mut row_builder = MacOutlineRow {
            ui: self.ui,
            theme: self.theme,
            row_id,
            row_rect,
            cursor_x: content_left,
            accessory_cursor_right: row_rect.right() - accessory_spacing,
            selected,
        };

        content(&mut row_builder);

        MacOutlineRowResponse {
            clicked: row_resp.clicked() && !disclosure_clicked,
            double_clicked: row_resp.double_clicked(),
            disclosure_clicked,
            is_expanded: *expanded,
            is_selected: selected,
        }
    }
}

/// Outline row content context for custom thumbnail, label, and generic accessory slots.
pub struct MacOutlineRow<'a> {
    /// Underlying egui `Ui` context.
    pub ui: &'a mut Ui,
    /// Injected macOS theme.
    pub theme: &'a MacTheme,
    row_id: Id,
    row_rect: Rect,
    cursor_x: f32,
    accessory_cursor_right: f32,
    selected: bool,
}

impl<'a> MacOutlineRow<'a> {
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

    /// Renders the primary text label of the row.
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
            self.theme.control_accent()
        } else {
            self.theme.label()
        };
        painter.text(
            Pos2::new(label_rect.left(), label_rect.center().y),
            Align2::LEFT_CENTER,
            text,
            font_id,
            text_color,
        );
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
        }

        let painter = self.ui.painter_at(rect);
        drawer(&painter, rect, *active);

        resp
    }
}
