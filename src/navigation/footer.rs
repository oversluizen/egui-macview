//! Authentic macOS Canvas Footer & Zoom HUD Layout Engine.
//!
//! Normative Reference: SPEC-007 §3.
//! Enforces a deterministic 24.0pt vertical layout budget via `MacRow`.

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
        // Footer background and 1px top border hairline
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, theme.window_background());
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(1.0, theme.separator()),
        );

        let item_spacing = theme.stack_view().default_spacing.points();
        let font_id = FontId::proportional(theme.stack_view().section_header_font_size.points());
        let text_color = theme.secondary_label();
        let mut action = None;

        // 2. Trailing Zone: Zoom Presets (50% | 100% | 200%) via Layout::right_to_left
        let zoom_presets = [2.0f32, 1.0f32, 0.5f32];
        let preset_btn_width = 36.0f32;
        let preset_btn_height = 18.0f32;
        let zoom_gap = 2.0f32;
        let zoom_cluster_width = (preset_btn_width * 3.0) + (zoom_gap * 2.0); // 112.0pt
        let zoom_left = (rect.right() - item_spacing - zoom_cluster_width).max(rect.left() + item_spacing);

        let zoom_area = Rect::from_min_max(
            Pos2::new(zoom_left, rect.top()),
            Pos2::new(rect.right() - item_spacing, rect.bottom()),
        );
        if zoom_area.width() >= zoom_cluster_width - 1.0 {
            let mut zoom_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(zoom_area)
                    .layout(egui::Layout::right_to_left(egui::Align::Center)),
            );

            for (z_idx, &preset) in zoom_presets.iter().enumerate() {
                let (btn_rect, resp) = zoom_ui.allocate_exact_size(
                    Vec2::new(preset_btn_width, preset_btn_height),
                    Sense::click(),
                );

                let is_selected = (self.zoom_factor - preset).abs() < 0.05;
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

                zoom_ui.painter().rect(
                    btn_rect,
                    CornerRadius::same(3),
                    bg,
                    Stroke::NONE,
                    StrokeKind::Inside,
                );

                let label = format!("{:.0}%", preset * 100.0);
                zoom_ui.painter().text(
                    btn_rect.center(),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::proportional(10.0),
                    fg,
                );

                if resp.clicked() {
                    action = Some(MacCanvasFooterAction::SelectZoom(preset));
                }

                if z_idx + 1 < zoom_presets.len() {
                    zoom_ui.add_space(zoom_gap);
                }
            }
        }

        // 1. Leading Zone: Breadcrumbs (truncated so they never spill into center or trailing zones)
        let mut breadcrumb_str = String::new();
        for (idx, crumb) in self.breadcrumbs.iter().enumerate() {
            if idx > 0 {
                breadcrumb_str.push_str(" > ");
            }
            breadcrumb_str.push_str(crumb);
        }

        let mut crumb_end_x = rect.left() + item_spacing;
        if !breadcrumb_str.is_empty() {
            let max_avail_for_crumbs = (zoom_left - crumb_end_x - item_spacing * 2.0 - 150.0).max(0.0);
            let crumb_max_w = (container_width * 0.35).min(max_avail_for_crumbs);
            if crumb_max_w > 10.0 {
                let crumb_rect = Rect::from_min_size(
                    Pos2::new(crumb_end_x, rect.top()),
                    Vec2::new(crumb_max_w, footer_height),
                );
                let mut crumb_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(crumb_rect)
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                );
                crumb_ui.add(
                    egui::Label::new(
                        egui::RichText::new(&breadcrumb_str)
                            .font(font_id)
                            .color(text_color),
                    )
                    .truncate(),
                );
                crumb_end_x += crumb_max_w;
            }
        }

        // 3. Center Zone: Coordinate readout strictly bounded and clipped
        if let Some(pos) = self.cursor_pos {
            let center_min_x = crumb_end_x + item_spacing;
            let center_max_x = zoom_left - item_spacing;
            let avail_w = center_max_x - center_min_x;

            if avail_w >= 75.0 {
                let (coord_text, needed_w) = if avail_w >= 155.0 {
                    (format!("X: {:.1} pt   Y: {:.1} pt", pos.x, pos.y), 155.0f32)
                } else {
                    (format!("{:.0}, {:.0} pt", pos.x, pos.y), 75.0f32)
                };

                let center_x = rect.center().x.clamp(
                    center_min_x + needed_w * 0.5,
                    center_max_x - needed_w * 0.5,
                );
                let clip_rect = Rect::from_min_max(
                    Pos2::new(center_min_x, rect.top()),
                    Pos2::new(center_max_x, rect.bottom()),
                );

                ui.painter_at(clip_rect).text(
                    Pos2::new(center_x, rect.center().y),
                    Align2::CENTER_CENTER,
                    coord_text,
                    FontId::monospace(10.5),
                    theme.secondary_label(),
                );
            }
        }

        action
    }
}
