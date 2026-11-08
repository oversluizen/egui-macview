//! Unified AppKit Window Toolbar (`MacToolbar`, `SPEC-005`).
//!
//! Renders an authentic macOS unified titlebar/toolbar with configurable
//! [`WindowControlsStyle`] (`AppleTrafficLights`, `WindowsCaptionButtons`, or `NativeHost`),
//! double-click titlebar zoom toggle, background window drag initiation, and
//! center/trailing content region.

use crate::theme::{MacTheme, WindowControlsStyle};
use egui::{
    Align, Align2, CornerRadius, FontId, Id, Layout, Pos2, Rect, Response, Sense, Stroke, Ui,
    UiBuilder, Vec2,
};
use std::hash::Hash;

/// Interaction result returned by [`MacToolbar::show`].
#[derive(Debug, Clone)]
pub struct MacToolbarResponse {
    /// Underlying toolbar `egui::Response`.
    pub response: Response,
    /// Whether the close button was clicked.
    pub close_clicked: bool,
    /// Whether the minimize button was clicked.
    pub minimize_clicked: bool,
    /// Whether the zoom/maximize button was clicked or the titlebar was double-clicked.
    pub zoom_toggled: bool,
    /// Whether a background window drag gesture was initiated.
    pub drag_started: bool,
    /// Bounding rectangle of the rendered toolbar.
    pub rect: Rect,
}

/// Unified window header and toolbar (`SPEC-005`) with configurable `WindowControlsStyle`.
pub struct MacToolbar<'a> {
    id: Id,
    title: String,
    style: Option<WindowControlsStyle>,
    is_maximized: Option<&'a mut bool>,
    compact: bool,
    height: Option<f32>,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacToolbar<'a> {
    /// Creates a new `MacToolbar` with the given ID or window title.
    pub fn new(title: impl Into<String>) -> Self {
        let title_str = title.into();
        Self {
            id: Id::NULL.with(&title_str),
            title: title_str,
            style: None,
            is_maximized: None,
            compact: true,
            height: None,
            theme: None,
        }
    }

    /// Overrides the widget `Id`.
    pub fn id(mut self, id_source: impl Hash + std::fmt::Debug) -> Self {
        self.id = Id::NULL.with(id_source);
        self
    }

    /// Sets the window title text.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Overrides the `WindowControlsStyle`.
    pub fn window_controls_style(mut self, style: WindowControlsStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Binds the mutable maximized state for zoom/restore toggling.
    pub fn maximized(mut self, is_maximized: &'a mut bool) -> Self {
        self.is_maximized = Some(is_maximized);
        self
    }

    /// Uses compact toolbar height (`38.0pt` instead of unified `52.0pt`).
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Overrides the toolbar height in points.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the `MacTheme`.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the toolbar inside `rect` with center/trailing controls closure.
    pub fn show_in_rect(
        self,
        ui: &mut Ui,
        rect: Rect,
        contents: impl FnOnce(&mut Ui),
    ) -> MacToolbarResponse {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;

        let tb = theme.toolbar_metrics();
        let style = self.style.unwrap_or(theme.window_controls_style);
        let border_w = theme.popover_metrics().border_width.points();
        let title_font_pt = theme.popover_metrics().header_font_size.points();

        // Register background interaction first so child controls sit on top
        let bg_resp = ui.interact(rect, self.id.with("toolbar_bg"), Sense::click_and_drag());

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, CornerRadius::ZERO, theme.window_background());
        painter.line_segment(
            [
                Pos2::new(rect.left(), rect.bottom()),
                Pos2::new(rect.right(), rect.bottom()),
            ],
            Stroke::new(border_w, theme.separator()),
        );

        let mut close_clicked = false;
        let mut minimize_clicked = false;
        let mut zoom_toggled = bg_resp.double_clicked();

        let mut content_left = rect.left() + tb.item_spacing.points();
        let mut content_right = rect.right() - tb.item_spacing.points();

        match style {
            WindowControlsStyle::AppleTrafficLights => {
                let r = tb.traffic_light_radius.points();
                let diameter = r * 2.0;
                let spacing = tb.traffic_light_spacing.points();
                let cy = rect.center().y;
                let c1_x = rect.left() + diameter * 2.0;
                let c2_x = c1_x + diameter + spacing;
                let c3_x = c2_x + diameter + spacing;

                let cluster_rect =
                    Rect::from_min_max(Pos2::new(c1_x - r, cy - r), Pos2::new(c3_x + r, cy + r));
                let cluster_hovered = ui.rect_contains_pointer(cluster_rect);

                let close_rect = Rect::from_center_size(Pos2::new(c1_x, cy), Vec2::splat(diameter));
                let min_rect = Rect::from_center_size(Pos2::new(c2_x, cy), Vec2::splat(diameter));
                let zoom_rect = Rect::from_center_size(Pos2::new(c3_x, cy), Vec2::splat(diameter));

                let close_resp = ui.interact(close_rect, self.id.with("tl_close"), Sense::click());
                let min_resp = ui.interact(min_rect, self.id.with("tl_min"), Sense::click());
                let zoom_resp = ui.interact(zoom_rect, self.id.with("tl_zoom"), Sense::click());

                let (close_fill, close_border) = theme.traffic_light_close();
                let (min_fill, min_border) = theme.traffic_light_minimize();
                let (zoom_fill, zoom_border) = theme.traffic_light_zoom();

                painter.circle(
                    close_rect.center(),
                    r,
                    close_fill,
                    Stroke::new(border_w * 0.5, close_border),
                );
                painter.circle(
                    min_rect.center(),
                    r,
                    min_fill,
                    Stroke::new(border_w * 0.5, min_border),
                );
                painter.circle(
                    zoom_rect.center(),
                    r,
                    zoom_fill,
                    Stroke::new(border_w * 0.5, zoom_border),
                );

                if cluster_hovered {
                    let glyph_col = theme.traffic_light_glyph();
                    let glyph_stroke = Stroke::new(border_w, glyph_col);
                    let c_box = close_rect.shrink(r * 0.5);
                    painter.line_segment([c_box.left_top(), c_box.right_bottom()], glyph_stroke);
                    painter.line_segment([c_box.left_bottom(), c_box.right_top()], glyph_stroke);

                    let m_box = min_rect.shrink(r * 0.5);
                    painter.line_segment(
                        [
                            Pos2::new(m_box.left(), m_box.center().y),
                            Pos2::new(m_box.right(), m_box.center().y),
                        ],
                        glyph_stroke,
                    );

                    let z_box = zoom_rect.shrink(r * 0.5);
                    painter.line_segment(
                        [
                            Pos2::new(z_box.left(), z_box.center().y),
                            Pos2::new(z_box.right(), z_box.center().y),
                        ],
                        glyph_stroke,
                    );
                    painter.line_segment(
                        [
                            Pos2::new(z_box.center().x, z_box.top()),
                            Pos2::new(z_box.center().x, z_box.bottom()),
                        ],
                        glyph_stroke,
                    );
                }

                if close_resp.clicked() {
                    close_clicked = true;
                }
                if min_resp.clicked() {
                    minimize_clicked = true;
                }
                if zoom_resp.clicked() {
                    zoom_toggled = true;
                }

                content_left = rect.left() + tb.traffic_lights_inset.points();
            }
            WindowControlsStyle::WindowsCaptionButtons => {
                let btn_w = tb.caption_button_width.points();
                let btn_h = tb.caption_button_height.points().min(rect.height());
                let icon_sz = tb.caption_icon_size.points();

                let close_rect = Rect::from_min_size(
                    Pos2::new(rect.right() - btn_w, rect.top()),
                    Vec2::new(btn_w, btn_h),
                );
                let max_rect = Rect::from_min_size(
                    Pos2::new(close_rect.left() - btn_w, rect.top()),
                    Vec2::new(btn_w, btn_h),
                );
                let min_rect = Rect::from_min_size(
                    Pos2::new(max_rect.left() - btn_w, rect.top()),
                    Vec2::new(btn_w, btn_h),
                );

                let min_resp = ui.interact(min_rect, self.id.with("win_min"), Sense::click());
                let max_resp = ui.interact(max_rect, self.id.with("win_max"), Sense::click());
                let close_resp = ui.interact(close_rect, self.id.with("win_close"), Sense::click());

                if min_resp.hovered() {
                    let bg = if min_resp.is_pointer_button_down_on() {
                        theme.windows_caption_pressed_bg()
                    } else {
                        theme.windows_caption_hover_bg()
                    };
                    painter.rect_filled(min_rect, CornerRadius::ZERO, bg);
                }
                if max_resp.hovered() {
                    let bg = if max_resp.is_pointer_button_down_on() {
                        theme.windows_caption_pressed_bg()
                    } else {
                        theme.windows_caption_hover_bg()
                    };
                    painter.rect_filled(max_rect, CornerRadius::ZERO, bg);
                }
                if close_resp.hovered() {
                    let bg = if close_resp.is_pointer_button_down_on() {
                        theme.windows_caption_close_pressed_bg()
                    } else {
                        theme.windows_caption_close_hover_bg()
                    };
                    painter.rect_filled(close_rect, CornerRadius::ZERO, bg);
                }

                let min_icon = Rect::from_center_size(min_rect.center(), Vec2::splat(icon_sz));
                let max_icon = Rect::from_center_size(max_rect.center(), Vec2::splat(icon_sz));
                let close_icon = Rect::from_center_size(close_rect.center(), Vec2::splat(icon_sz));

                painter.line_segment(
                    [
                        Pos2::new(min_icon.left(), min_icon.center().y),
                        Pos2::new(min_icon.right(), min_icon.center().y),
                    ],
                    Stroke::new(border_w, theme.label()),
                );
                painter.rect_stroke(
                    max_icon,
                    CornerRadius::ZERO,
                    Stroke::new(border_w, theme.label()),
                    egui::StrokeKind::Inside,
                );
                let close_fg = if close_resp.hovered() {
                    theme.on_accent_text()
                } else {
                    theme.label()
                };
                let close_stroke = Stroke::new(border_w, close_fg);
                painter.line_segment(
                    [close_icon.left_top(), close_icon.right_bottom()],
                    close_stroke,
                );
                painter.line_segment(
                    [close_icon.left_bottom(), close_icon.right_top()],
                    close_stroke,
                );

                if close_resp.clicked() {
                    close_clicked = true;
                }
                if min_resp.clicked() {
                    minimize_clicked = true;
                }
                if max_resp.clicked() {
                    zoom_toggled = true;
                }

                content_right = min_rect.left() - tb.item_spacing.points();
            }
            WindowControlsStyle::NativeHost => {}
        }

        if zoom_toggled {
            if let Some(max_ref) = self.is_maximized {
                *max_ref = !*max_ref;
            }
        }

        if !self.title.is_empty() {
            let title_galley = painter.layout_no_wrap(
                self.title,
                FontId::proportional(title_font_pt),
                egui::Color32::PLACEHOLDER,
            );
            let title_pos = Align2::LEFT_CENTER
                .anchor_size(
                    Pos2::new(content_left, rect.center().y),
                    title_galley.size(),
                )
                .min;
            content_left += title_galley.size().x + tb.item_spacing.points() * 2.0;
            painter.galley(title_pos, title_galley, theme.label());
        }

        let inner_rect = Rect::from_min_max(
            Pos2::new(content_left.min(content_right), rect.top()),
            Pos2::new(content_right, rect.bottom()),
        );
        let mut child_ui = ui.new_child(
            UiBuilder::new()
                .id_salt((self.id, "toolbar_contents"))
                .max_rect(inner_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        contents(&mut child_ui);

        let drag_started =
            bg_resp.drag_started() && !close_clicked && !minimize_clicked && !zoom_toggled;

        MacToolbarResponse {
            response: bg_resp,
            close_clicked,
            minimize_clicked,
            zoom_toggled,
            drag_started,
            rect,
        }
    }

    /// Allocates full width and renders the toolbar with center/trailing controls closure.
    pub fn show(self, ui: &mut Ui, contents: impl FnOnce(&mut Ui)) -> MacToolbarResponse {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let tb = theme.toolbar_metrics();
        let target_h = self.height.unwrap_or_else(|| {
            if self.compact {
                tb.height_compact.points()
            } else {
                tb.height_unified.points()
            }
        });
        let avail_w = ui.available_width().max(tb.traffic_lights_inset.points());
        let (rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, target_h), Sense::hover());
        self.show_in_rect(ui, rect, contents)
    }
}
