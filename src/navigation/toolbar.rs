//! Unified AppKit Window Toolbar (`MacToolbar`, `SPEC-005`).
//!
//! Renders an authentic macOS unified titlebar/toolbar with configurable
//! [`WindowControlsStyle`] (`AppleTrafficLights`, `WindowsCaptionButtons`, or `NativeHost`),
//! double-click titlebar zoom toggle, background window drag initiation, and
//! center/trailing content region.

use crate::theme::{MacTheme, WindowControlsStyle};
use egui::{Align, Align2, CornerRadius, FontId, Id, Layout, Pos2, Rect, Response, Sense, Stroke, Ui, UiBuilder, Vec2};
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
    /// Bounding rectangle of the rendered primary title text (or `Rect::NOTHING` if empty).
    pub title_rect: Rect,
    /// Bounding rectangle of the rendered subtitle text (or `Rect::NOTHING` if empty).
    pub subtitle_rect: Rect,
}

/// Unified window header and toolbar (`SPEC-005`) with configurable `WindowControlsStyle`.
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacButton, MacTheme, MacToolbar, WindowControlsStyle};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::DarkAqua);
///
/// MacToolbar::new("My Studio App")
///     .window_controls_style(WindowControlsStyle::AppleTrafficLights)
///     .theme(&theme)
///     .show(ui, |toolbar_ui| {
///         toolbar_ui.add(MacButton::new("Export").theme(&theme));
///     });
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacToolbar<'a> {
    id: Id,
    title: String,
    subtitle: String,
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
            subtitle: String::new(),
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

    /// Sets the secondary subtitle text displayed below the primary title in unified compact mode (`SPEC-005 §4`).
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
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
    pub fn show_in_rect(self, ui: &mut Ui, rect: Rect, contents: impl FnOnce(&mut Ui)) -> MacToolbarResponse {
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
            [Pos2::new(rect.left(), rect.bottom()), Pos2::new(rect.right(), rect.bottom())],
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
                let diameter = r + r;
                let spacing = tb.traffic_light_spacing.points();
                let cy = rect.center().y;
                let c1_x = rect.left() + tb.traffic_light_leading_inset.points();
                let c2_x = c1_x + diameter + spacing;
                let c3_x = c2_x + diameter + spacing;

                let cluster_rect = Rect::from_min_max(Pos2::new(c1_x - r, cy - r), Pos2::new(c3_x + r, cy + r));
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

                painter.circle(close_rect.center(), r, close_fill, Stroke::new(border_w * 0.5, close_border));
                painter.circle(min_rect.center(), r, min_fill, Stroke::new(border_w * 0.5, min_border));
                painter.circle(zoom_rect.center(), r, zoom_fill, Stroke::new(border_w * 0.5, zoom_border));

                if cluster_hovered {
                    let glyph_col = theme.traffic_light_glyph();
                    let glyph_stroke = Stroke::new(border_w, glyph_col);
                    let c_box = close_rect.shrink(r * 0.5);
                    painter.line_segment([c_box.left_top(), c_box.right_bottom()], glyph_stroke);
                    painter.line_segment([c_box.left_bottom(), c_box.right_top()], glyph_stroke);

                    let m_box = min_rect.shrink(r * 0.5);
                    painter.line_segment(
                        [Pos2::new(m_box.left(), m_box.center().y), Pos2::new(m_box.right(), m_box.center().y)],
                        glyph_stroke,
                    );

                    let z_box = zoom_rect.shrink(r * 0.5);
                    painter.line_segment(
                        [Pos2::new(z_box.left(), z_box.center().y), Pos2::new(z_box.right(), z_box.center().y)],
                        glyph_stroke,
                    );
                    painter.line_segment(
                        [Pos2::new(z_box.center().x, z_box.top()), Pos2::new(z_box.center().x, z_box.bottom())],
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

                let close_rect = Rect::from_min_size(Pos2::new(rect.right() - btn_w, rect.top()), Vec2::new(btn_w, btn_h));
                let max_rect = Rect::from_min_size(Pos2::new(close_rect.left() - btn_w, rect.top()), Vec2::new(btn_w, btn_h));
                let min_rect = Rect::from_min_size(Pos2::new(max_rect.left() - btn_w, rect.top()), Vec2::new(btn_w, btn_h));

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
                    [Pos2::new(min_icon.left(), min_icon.center().y), Pos2::new(min_icon.right(), min_icon.center().y)],
                    Stroke::new(border_w, theme.label()),
                );
                painter.rect_stroke(max_icon, CornerRadius::ZERO, Stroke::new(border_w, theme.label()), egui::StrokeKind::Inside);
                let close_fg = if close_resp.hovered() { theme.on_accent_text() } else { theme.label() };
                let close_stroke = Stroke::new(border_w, close_fg);
                painter.line_segment([close_icon.left_top(), close_icon.right_bottom()], close_stroke);
                painter.line_segment([close_icon.left_bottom(), close_icon.right_top()], close_stroke);

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

        let mut title_rect = Rect::NOTHING;
        let mut subtitle_rect = Rect::NOTHING;
        if !self.title.is_empty() {
            let (title_font, _) = theme.semibold_font_id(ui, title_font_pt);
            let title_galley = painter.layout_no_wrap(self.title, title_font, egui::Color32::PLACEHOLDER);
            if self.subtitle.is_empty() {
                let title_pos = Align2::LEFT_CENTER.anchor_size(Pos2::new(content_left, rect.center().y), title_galley.size()).min;
                title_rect = Rect::from_min_size(title_pos, title_galley.size());
                content_left += title_galley.size().x + tb.item_spacing.points() + tb.item_spacing.points();
                painter.galley(title_pos, title_galley, theme.label());
            } else {
                let title_pos = Pos2::new(content_left, rect.top() + tb.unified_title_top.points());
                let sub_galley = painter.layout_no_wrap(
                    self.subtitle,
                    FontId::proportional(tb.unified_subtitle_font_size.points()),
                    egui::Color32::PLACEHOLDER,
                );
                let sub_pos = Pos2::new(content_left, rect.top() + tb.unified_subtitle_top.points());
                title_rect = Rect::from_min_size(title_pos, title_galley.size());
                subtitle_rect = Rect::from_min_size(sub_pos, sub_galley.size());
                let stack_w = title_galley.size().x.max(sub_galley.size().x);
                content_left += stack_w + tb.item_spacing.points() + tb.item_spacing.points();
                painter.galley(title_pos, title_galley, theme.label());
                painter.galley(sub_pos, sub_galley, theme.secondary_label());
            }
        }

        let inner_rect =
            Rect::from_min_max(Pos2::new(content_left.min(content_right), rect.top()), Pos2::new(content_right, rect.bottom()));
        let mut child_ui = ui.new_child(
            UiBuilder::new().id_salt((self.id, "toolbar_contents")).max_rect(inner_rect).layout(Layout::left_to_right(Align::Center)),
        );
        contents(&mut child_ui);

        let drag_started = bg_resp.drag_started() && !close_clicked && !minimize_clicked && !zoom_toggled;

        MacToolbarResponse {
            response: bg_resp,
            close_clicked,
            minimize_clicked,
            zoom_toggled,
            drag_started,
            rect,
            title_rect,
            subtitle_rect,
        }
    }

    /// Renders a three-zone macOS Sonoma `NSWindowToolbarStyleUnifiedCompact` (`SPEC-005 §4`) toolbar inside `rect`:
    /// 1. Leading navigational items (`leading_nav`) starting at `unified_nav_leading_inset` (`79.0pt`),
    /// 2. Two-line `NSToolbarTitleStackView` (`title` + `subtitle`) immediately after `leading_nav`,
    /// 3. Horizontally centered principal items (`center_tools`) and right-aligned trailing items (`trailing_items`).
    pub fn show_unified_zones_in_rect(
        mut self,
        ui: &mut Ui,
        rect: Rect,
        leading_nav: impl FnOnce(&mut Ui),
        center_tools: impl FnOnce(&mut Ui),
        trailing_items: impl FnOnce(&mut Ui),
    ) -> MacToolbarResponse {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let tb = theme.toolbar_metrics().clone();
        let title_font_pt = theme.popover_metrics().header_font_size.points();
        let style = self.style.unwrap_or(theme.window_controls_style);
        let saved_title = std::mem::take(&mut self.title);
        let saved_subtitle = std::mem::take(&mut self.subtitle);
        let tb_id = self.id;

        let mut title_rect = Rect::NOTHING;
        let mut subtitle_rect = Rect::NOTHING;
        let nav_x = match style {
            WindowControlsStyle::AppleTrafficLights => rect.left() + tb.unified_nav_leading_inset.points(),
            _ => rect.left() + tb.edge_padding.points(),
        };
        let trail_max_x = match style {
            WindowControlsStyle::WindowsCaptionButtons => {
                let bw = tb.caption_button_width.points();
                rect.right() - (bw + bw + bw) - tb.unified_trailing_inset.points()
            }
            _ => rect.right() - tb.unified_trailing_inset.points(),
        };

        let mut resp = self.show_in_rect(ui, rect, |inner_ui| {
            let nav_rect = Rect::from_min_max(Pos2::new(nav_x, rect.top()), Pos2::new(trail_max_x, rect.bottom()));
            let mut nav_ui = inner_ui.new_child(
                UiBuilder::new().id_salt((tb_id, "unified_nav")).max_rect(nav_rect).layout(Layout::left_to_right(Align::Center)),
            );
            nav_ui.spacing_mut().item_spacing.x = 0.0;
            leading_nav(&mut nav_ui);
            let nav_end_x = nav_ui.min_rect().max.x.max(nav_x);

            let mut title_end_x = nav_end_x;
            if !saved_title.is_empty() {
                let tx = nav_end_x + tb.unified_title_gap.points();
                let painter = inner_ui.painter_at(rect);
                let (title_font, _) = theme.semibold_font_id(inner_ui, title_font_pt);
                let title_galley = painter.layout_no_wrap(saved_title, title_font, egui::Color32::PLACEHOLDER);
                if saved_subtitle.is_empty() {
                    let t_pos = Align2::LEFT_CENTER.anchor_size(Pos2::new(tx, rect.center().y), title_galley.size()).min;
                    title_rect = Rect::from_min_size(t_pos, title_galley.size());
                    title_end_x = title_rect.max.x;
                    painter.galley(t_pos, title_galley, theme.label());
                } else {
                    let t_pos = Pos2::new(tx, rect.top() + tb.unified_title_top.points());
                    let sub_galley = painter.layout_no_wrap(
                        saved_subtitle,
                        FontId::proportional(tb.unified_subtitle_font_size.points()),
                        egui::Color32::PLACEHOLDER,
                    );
                    let s_pos = Pos2::new(tx, rect.top() + tb.unified_subtitle_top.points());
                    title_rect = Rect::from_min_size(t_pos, title_galley.size());
                    subtitle_rect = Rect::from_min_size(s_pos, sub_galley.size());
                    title_end_x = title_rect.max.x.max(subtitle_rect.max.x);
                    painter.galley(t_pos, title_galley, theme.label());
                    painter.galley(s_pos, sub_galley, theme.secondary_label());
                }
            }

            let trail_rect = Rect::from_min_max(Pos2::new(title_end_x, rect.top()), Pos2::new(trail_max_x, rect.bottom()));
            let mut trail_ui = inner_ui.new_child(
                UiBuilder::new().id_salt((tb_id, "unified_trailing")).max_rect(trail_rect).layout(Layout::right_to_left(Align::Center)),
            );
            trail_ui.spacing_mut().item_spacing.x = 0.0;
            trailing_items(&mut trail_ui);
            let trail_start_x = trail_ui.min_rect().min.x.min(trail_max_x);

            let avail_center_half = (rect.center().x - title_end_x).min(trail_start_x - rect.center().x).max(0.0);
            let center_zone = Rect::from_min_max(
                Pos2::new(rect.center().x - avail_center_half, rect.top()),
                Pos2::new(rect.center().x + avail_center_half, rect.bottom()),
            );
            let mut center_ui = inner_ui.new_child(
                UiBuilder::new()
                    .id_salt((tb_id, "unified_center"))
                    .max_rect(center_zone)
                    .layout(Layout::centered_and_justified(egui::Direction::LeftToRight)),
            );
            center_ui.spacing_mut().item_spacing.x = 0.0;
            center_tools(&mut center_ui);
        });
        resp.title_rect = title_rect;
        resp.subtitle_rect = subtitle_rect;
        resp
    }

    /// Allocates full width and renders a three-zone macOS Sonoma `NSWindowToolbarStyleUnifiedCompact` (`SPEC-005 §4`) toolbar.
    pub fn show_unified_zones(
        self,
        ui: &mut Ui,
        leading_nav: impl FnOnce(&mut Ui),
        center_tools: impl FnOnce(&mut Ui),
        trailing_items: impl FnOnce(&mut Ui),
    ) -> MacToolbarResponse {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let tb = theme.toolbar_metrics();
        let target_h = self.height.unwrap_or_else(|| if self.compact { tb.height_compact.points() } else { tb.height_unified.points() });
        let avail_w = ui.available_width().max(tb.traffic_lights_inset.points());
        let (rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, target_h), Sense::hover());
        self.show_unified_zones_in_rect(ui, rect, leading_nav, center_tools, trailing_items)
    }

    /// Allocates full width and renders the toolbar with center/trailing controls closure.
    pub fn show(self, ui: &mut Ui, contents: impl FnOnce(&mut Ui)) -> MacToolbarResponse {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let tb = theme.toolbar_metrics();
        let target_h = self.height.unwrap_or_else(|| if self.compact { tb.height_compact.points() } else { tb.height_unified.points() });
        let avail_w = ui.available_width().max(tb.traffic_lights_inset.points());
        let (rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, target_h), Sense::hover());
        self.show_in_rect(ui, rect, contents)
    }
}
