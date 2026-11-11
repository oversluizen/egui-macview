//! Authentic macOS AppKit Hierarchical Outline View (`NSOutlineView` equivalent).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors or dimensions.

pub mod row;

pub use row::MacOutlineRow;

use crate::theme::MacTheme;
use egui::{CornerRadius, Id, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use std::fmt::Debug;
use std::hash::Hash;

/// Interaction response from an outline row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacOutlineRowResponse {
    /// True if the row body was clicked (excluding disclosure chevron and trailing accessory clicks).
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
        Self {
            id: Id::NULL.with(id_source),
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
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;

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
        let row_id = self.id.with(id_source);
        let avail_w = self.ui.available_width();
        let (row_rect, row_resp) = self
            .ui
            .allocate_exact_size(Vec2::new(avail_w, self.row_height), Sense::click());

        let metrics = self.theme.outline_view();
        let corner_radius = metrics.corner_radius.points();
        let painter = self.ui.painter_at(row_rect);

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

            let spring = self.theme.motion().swiftui_snappy.to_spring();
            let state_key = row_id.with("disc_spring_state");
            let target_frac = if *expanded { 1.0f32 } else { 0.0f32 };
            let current_frac = spring.animate_f32(self.ui.ctx(), state_key, target_frac);

            let angle = current_frac * std::f32::consts::FRAC_PI_2;
            let (sin_a, cos_a) = angle.sin_cos();
            let rotate = |dx: f32, dy: f32, center: Pos2| -> Pos2 {
                Pos2::new(
                    center.x + dx * cos_a - dy * sin_a,
                    center.y + dx * sin_a + dy * cos_a,
                )
            };

            let center = disc_rect.center();
            let half = metrics.chevron_size.points() * 0.5;
            let chev_depth = metrics.chevron_depth.points();
            let chevron_stroke = Stroke::new(
                metrics.chevron_stroke_width.points(),
                if selected {
                    let bg_style = self.theme.selected_control_background_style();
                    bg_style.primary_text_color(self.theme)
                } else {
                    self.theme.placeholder_text()
                },
            );

            let p_top = rotate(-chev_depth, -half, center);
            let p_tip = rotate(chev_depth, 0.0, center);
            let p_bot = rotate(-chev_depth, half, center);
            painter.line_segment([p_top, p_tip], chevron_stroke);
            painter.line_segment([p_tip, p_bot], chevron_stroke);
        }

        let content_spacing = metrics.content_spacing.points();
        let accessory_spacing = metrics.accessory_spacing.points();
        let content_left = disc_x + disc_size + content_spacing;
        let mut row_builder = MacOutlineRow::new(
            self.ui,
            self.theme,
            row_id,
            row_rect,
            content_left,
            row_rect.right() - accessory_spacing,
            selected,
        );

        content(&mut row_builder);

        MacOutlineRowResponse {
            clicked: row_resp.clicked() && !disclosure_clicked && !row_builder.accessory_clicked,
            double_clicked: row_resp.double_clicked(),
            disclosure_clicked,
            is_expanded: *expanded,
            is_selected: selected,
        }
    }
}
