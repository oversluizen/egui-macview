//! Authentic macOS Canvas Footer & Zoom HUD Layout Engine.
//!
//! Normative Reference: SPEC-007 §3.
//! Enforces a deterministic 24.0pt vertical layout budget via `MacRow`.

use crate::layout::MacRow;
use crate::theme::MacTheme;
use egui::{
    Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
};

/// Standard canvas footer height in points.
pub const CANVAS_FOOTER_HEIGHT: f32 = 24.0;

/// Action triggered by footer interactions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MacCanvasFooterAction {
    /// User clicked a predefined zoom factor preset (e.g. 0.5, 1.0, 2.0).
    SelectZoom(f32),
    /// User clicked a breadcrumb path segment at the specified index.
    BreadcrumbClicked(usize),
}

/// Canvas Footer container with deterministic 24.0pt budget.
pub struct MacCanvasFooter<'a> {
    breadcrumbs: &'a [&'a str],
    cursor_pos: Option<Pos2>,
    zoom_factor: f32,
    theme: Option<&'a MacTheme>,
    height: f32,
}

impl<'a> MacCanvasFooter<'a> {
    /// Creates a new canvas footer with breadcrumbs, cursor readout, and current zoom.
    pub fn new(breadcrumbs: &'a [&'a str], cursor_pos: Option<Pos2>, zoom_factor: f32) -> Self {
        Self {
            breadcrumbs,
            cursor_pos,
            zoom_factor,
            theme: None,
            height: CANVAS_FOOTER_HEIGHT,
        }
    }

    /// Sets the dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Overrides default height (default is 24.0pt).
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// Height of the footer container.
    pub fn get_height(&self) -> f32 {
        self.height
    }

    /// Renders the canvas footer within the allocated container rectangle.
    pub fn show(self, ui: &mut Ui, container_width: f32) -> Option<MacCanvasFooterAction> {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let footer_height = self.height;
        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(container_width, footer_height), Sense::hover());
        let painter = ui.painter();

        // Footer background and 1px top border hairline
        painter.rect_filled(rect, CornerRadius::ZERO, theme.window_background());
        painter.line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(1.0, theme.separator()),
        );

        let item_spacing = theme.stack_view().default_spacing.points();
        let mut row = MacRow::new(rect.top(), footer_height, container_width, item_spacing);
        let font_id = FontId::proportional(theme.stack_view().section_header_font_size.points());
        let text_color = theme.secondary_label();

        let mut action = None;

        // 1. Leading Zone: Breadcrumbs
        let mut breadcrumb_str = String::new();
        for (idx, crumb) in self.breadcrumbs.iter().enumerate() {
            if idx > 0 {
                breadcrumb_str.push_str(" > ");
            }
            breadcrumb_str.push_str(crumb);
        }

        if !breadcrumb_str.is_empty() {
            let galley = painter.layout_no_wrap(breadcrumb_str, font_id.clone(), text_color);
            let crumb_width = galley.size().x.min(container_width * 0.4);
            let (rx, ry, rw, rh) = row.allocate_cell(crumb_width + item_spacing);
            let cell_rect = Rect::from_min_size(
                Pos2::new(rect.left() + rx + item_spacing, ry),
                Vec2::new(rw, rh),
            );

            painter.text(
                cell_rect.left_center(),
                Align2::LEFT_CENTER,
                galley.text(),
                font_id.clone(),
                text_color,
            );
        }

        // 2. Trailing Zone: Zoom Presets (50% | 100% | 200%)
        let zoom_presets = [0.5f32, 1.0f32, 2.0f32];
        let preset_btn_width = 36.0f32;
        let total_zoom_w = (preset_btn_width * (zoom_presets.len() as f32))
            + (zoom_presets.len() as f32 - 1.0) * 2.0;

        let zoom_start_x = rect.right() - total_zoom_w - item_spacing;
        for (z_idx, &preset) in zoom_presets.iter().enumerate() {
            let bx = zoom_start_x + (z_idx as f32) * (preset_btn_width + 2.0);
            let btn_rect = Rect::from_min_size(
                Pos2::new(bx, rect.top() + (footer_height - 18.0) * 0.5),
                Vec2::new(preset_btn_width, 18.0),
            );

            let is_selected = (self.zoom_factor - preset).abs() < 0.05;
            let resp = ui.interact(
                btn_rect,
                ui.scope_id().with("footer_zoom").with(z_idx),
                Sense::click(),
            );

            let bg = if is_selected {
                theme.control_accent()
            } else if resp.hovered() {
                theme.control()
            } else {
                Color32::TRANSPARENT
            };

            let fg = if is_selected {
                Color32::WHITE
            } else {
                theme.label()
            };

            painter.rect(
                btn_rect,
                CornerRadius::same(3),
                bg,
                Stroke::NONE,
                StrokeKind::Inside,
            );

            let label = format!("{:.0}%", preset * 100.0);
            painter.text(
                btn_rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(10.0),
                fg,
            );

            if resp.clicked() {
                action = Some(MacCanvasFooterAction::SelectZoom(preset));
            }
        }

        // 3. Center Zone: Coordinate readout
        if let Some(pos) = self.cursor_pos {
            let coord_text = format!("X: {:.1} pt   Y: {:.1} pt", pos.x, pos.y);
            let coord_rect = Rect::from_center_size(rect.center(), Vec2::new(160.0, footer_height));
            painter.text(
                coord_rect.center(),
                Align2::CENTER_CENTER,
                coord_text,
                FontId::monospace(10.5),
                theme.secondary_label(),
            );
        }

        action
    }
}
