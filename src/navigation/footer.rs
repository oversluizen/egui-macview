//! Authentic macOS Canvas Footer & Zoom HUD Layout Engine.
//!
//! Normative Reference: SPEC-007 §3.
//! Enforces a deterministic 24.0pt vertical layout budget via `MacRow`.

use crate::theme::MacTheme;
use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Label, Layout, Pos2, Rect, RichText, Sense,
    Stroke, StrokeKind, Ui, UiBuilder, Vec2,
};

pub use crate::tokens::CANVAS_FOOTER_HEIGHT;

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
                fallback_theme = MacTheme::from_ctx(ui.ctx());
                &fallback_theme
            }
        };

        let fm = theme.footer_metrics();
        let footer_height = self.height;
        let border_w = theme.popover_metrics().border_width.points();
        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(container_width, footer_height), Sense::hover());

        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, theme.window_background());
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(border_w, theme.separator()),
        );

        let item_spacing = theme.stack_view().default_spacing.points();
        let font_id = FontId::proportional(fm.font_size.points());
        let text_color = theme.secondary_label();
        let on_accent = theme.on_accent_text();
        let mut action = None;

        // 2. Trailing Zone: Zoom Presets (50% | 100% | 200%) via Layout::right_to_left
        let zoom_presets = [2.0f32, 1.0f32, 0.5f32];
        let preset_btn_width = fm.preset_width.points();
        let preset_btn_height = fm.preset_height.points();
        let preset_corner_r = fm.preset_corner_radius.points() as u8;
        let preset_font_pt = fm.preset_font_size.points();
        let zoom_gap = fm.preset_spacing.points();
        let zoom_cluster_width =
            (preset_btn_width * (zoom_presets.len() as f32)) + (zoom_gap * 2.0);
        let zoom_left =
            (rect.right() - item_spacing - zoom_cluster_width).max(rect.left() + item_spacing);

        let zoom_area = Rect::from_min_max(
            Pos2::new(zoom_left, rect.top()),
            Pos2::new(rect.right() - item_spacing, rect.bottom()),
        );
        if zoom_area.width() >= zoom_cluster_width - border_w {
            let mut zoom_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(zoom_area)
                    .layout(Layout::right_to_left(Align::Center)),
            );

            for (z_idx, &preset) in zoom_presets.iter().enumerate() {
                let (btn_rect, resp) = zoom_ui.allocate_exact_size(
                    Vec2::new(preset_btn_width, preset_btn_height),
                    Sense::click(),
                );

                let is_selected = (self.zoom_factor - preset).abs() <= f32::EPSILON;
                let bg = if is_selected {
                    theme.control_accent()
                } else if resp.hovered() {
                    theme.control()
                } else {
                    Color32::TRANSPARENT
                };

                let fg = if is_selected {
                    on_accent
                } else {
                    theme.label()
                };

                zoom_ui.painter().rect(
                    btn_rect,
                    CornerRadius::same(preset_corner_r),
                    bg,
                    Stroke::NONE,
                    StrokeKind::Inside,
                );

                let pct_scale = f32::from(100u8);
                let label = format!("{:.0}%", preset * pct_scale);
                zoom_ui.painter().text(
                    btn_rect.center(),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::proportional(preset_font_pt),
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

        // 1. Leading Zone: Interactive Breadcrumb segments
        let mut crumb_end_x = rect.left() + item_spacing;
        let coord_full_w = fm.divider_trailing_offset.points();
        let coord_compact_w = fm.coords_width.points();
        let crumb_ratio = theme.split_view().default_ratio.points();

        if !self.breadcrumbs.is_empty() {
            let max_avail_for_crumbs =
                (zoom_left - crumb_end_x - item_spacing * 2.0 - coord_full_w).max(0.0);
            let crumb_max_w = (container_width * crumb_ratio).min(max_avail_for_crumbs);
            if crumb_max_w > item_spacing {
                let crumb_rect = Rect::from_min_size(
                    Pos2::new(crumb_end_x, rect.top()),
                    Vec2::new(crumb_max_w, footer_height),
                );
                let mut crumb_ui = ui.new_child(
                    UiBuilder::new()
                        .max_rect(crumb_rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );
                crumb_ui.set_clip_rect(crumb_rect.intersect(ui.clip_rect()));
                crumb_ui.spacing_mut().item_spacing.x = zoom_gap * 2.0;

                for (idx, crumb) in self.breadcrumbs.iter().enumerate() {
                    if idx > 0 {
                        crumb_ui.add(
                            Label::new(
                                RichText::new("›")
                                    .font(font_id.clone())
                                    .color(theme.placeholder_text()),
                            )
                            .selectable(false),
                        );
                    }
                    let seg_resp = crumb_ui.add(
                        Label::new(
                            RichText::new(*crumb)
                                .font(font_id.clone())
                                .color(text_color),
                        )
                        .sense(Sense::click())
                        .truncate(),
                    );
                    if seg_resp.hovered() {
                        ui.painter().line_segment(
                            [
                                Pos2::new(seg_resp.rect.left(), seg_resp.rect.bottom()),
                                Pos2::new(seg_resp.rect.right(), seg_resp.rect.bottom()),
                            ],
                            Stroke::new(border_w, theme.label()),
                        );
                    }
                    if seg_resp.clicked() {
                        action = Some(MacCanvasFooterAction::BreadcrumbClicked(idx));
                    }
                }
                crumb_end_x += crumb_max_w;
            }
        }

        // 3. Center Zone: Coordinate readout strictly bounded and clipped
        if let Some(pos) = self.cursor_pos {
            let center_min_x = crumb_end_x + item_spacing;
            let center_max_x = zoom_left - item_spacing;
            let avail_w = center_max_x - center_min_x;
            let coord_font_pt = fm.font_size.points();

            if avail_w >= coord_compact_w {
                let (coord_text, needed_w) = if avail_w >= coord_full_w {
                    (
                        format!("X: {:.1} pt   Y: {:.1} pt", pos.x, pos.y),
                        coord_full_w,
                    )
                } else {
                    (format!("{:.0}, {:.0} pt", pos.x, pos.y), coord_compact_w)
                };

                let center_x = rect
                    .center()
                    .x
                    .clamp(center_min_x + needed_w * 0.5, center_max_x - needed_w * 0.5);
                let clip_rect = Rect::from_min_max(
                    Pos2::new(center_min_x, rect.top()),
                    Pos2::new(center_max_x, rect.bottom()),
                );

                ui.painter_at(clip_rect).text(
                    Pos2::new(center_x, rect.center().y),
                    Align2::CENTER_CENTER,
                    coord_text,
                    FontId::monospace(coord_font_pt),
                    theme.secondary_label(),
                );
            }
        }

        action
    }
}
