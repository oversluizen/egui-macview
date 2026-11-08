//! Authentic macOS AppKit Floating Overlay Scroll View (NSScrollView equivalent).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded dimensions or colors.
//! Features zero-gutter content reservation (full viewport width/height),
//! smooth hardware scroll clipping, virtual viewport computation,
//! and interactive floating overlay scrollbars that smoothly fade out when inactive.

use crate::layout::scrollbar::render_overlay_thumb;
use crate::theme::MacTheme;
use egui::scroll_area::ScrollBarVisibility;
use egui::{Color32, Id, Pos2, Rect, ScrollArea, Ui, Vec2};
use std::hash::Hash;

/// Output returned by [`MacScrollView::show`].
#[derive(Debug, Clone)]
pub struct MacScrollViewOutput<R> {
    /// Return value of the inner content closure.
    pub inner: R,
    /// Bounding rectangle of the visible viewport in logical points.
    pub inner_rect: Rect,
    /// Total dimensions of the scrollable content in points.
    pub content_size: Vec2,
    /// Current logical scroll offset in points.
    pub offset: Vec2,
}

/// Authentic macOS AppKit Scroll View (`NSScrollView` equivalent).
///
/// Features zero-gutter content reservation (full viewport width/height),
/// smooth hardware scroll clipping, virtual viewport computation,
/// and interactive floating overlay scrollbars that smoothly fade out when inactive.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacScrollView, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
///
/// MacScrollView::new("log_scroll")
///     .theme(&theme)
///     .max_height(200.0)
///     .show(ui, |content_ui| {
///         for i in 0..50 {
///             content_ui.label(format!("Log line {}", i));
///         }
///     });
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacScrollView<'a> {
    id: Id,
    theme: Option<&'a MacTheme>,
    horizontal: bool,
    vertical: bool,
    auto_hide: bool,
    max_height: Option<f32>,
    max_width: Option<f32>,
    min_scrolled_height: Option<f32>,
    min_scrolled_width: Option<f32>,
    offset_override: Option<&'a mut Vec2>,
}

impl<'a> MacScrollView<'a> {
    /// Creates a scroll view with state identified by `id_source`.
    pub fn new(id_source: impl Hash + std::fmt::Debug) -> Self {
        let id = Id::NULL.with(id_source);
        Self {
            id,
            theme: None,
            horizontal: false,
            vertical: true,
            auto_hide: true,
            max_height: None,
            max_width: None,
            min_scrolled_height: None,
            min_scrolled_width: None,
            offset_override: None,
        }
    }

    /// Convenience constructor for both horizontal and vertical scrolling.
    pub fn both(id_source: impl Hash + std::fmt::Debug) -> Self {
        Self::new(id_source).horizontal(true).vertical(true)
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Enables or disables horizontal scrolling (default: false).
    pub fn horizontal(mut self, enabled: bool) -> Self {
        self.horizontal = enabled;
        self
    }

    /// Enables or disables vertical scrolling (default: true).
    pub fn vertical(mut self, enabled: bool) -> Self {
        self.vertical = enabled;
        self
    }

    /// Controls whether overlay scrollbars fade out when inactive (default: true).
    pub fn auto_hide(mut self, auto_hide: bool) -> Self {
        self.auto_hide = auto_hide;
        self
    }

    /// Sets maximum height constraint in points.
    pub fn max_height(mut self, max_h: f32) -> Self {
        self.max_height = Some(max_h);
        self
    }

    /// Sets maximum width constraint in points.
    pub fn max_width(mut self, max_w: f32) -> Self {
        self.max_width = Some(max_w);
        self
    }

    /// Sets minimum scrolled height in points.
    pub fn min_scrolled_height(mut self, min_h: f32) -> Self {
        self.min_scrolled_height = Some(min_h);
        self
    }

    /// Sets minimum scrolled width in points.
    pub fn min_scrolled_width(mut self, min_w: f32) -> Self {
        self.min_scrolled_width = Some(min_w);
        self
    }

    /// Binds directly to an external `&mut Vec2` offset variable for programmatic control.
    pub fn with_offset(mut self, offset: &'a mut Vec2) -> Self {
        self.offset_override = Some(offset);
        self
    }

    /// Renders the scrollable container and child content, overlaying authentic floating scrollbars.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> MacScrollViewOutput<R> {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;

        let metrics = theme.scroll_view();
        let scrollbar_thickness = metrics.scrollbar_thickness.points();
        let scrollbar_hover_thickness = metrics.scrollbar_hover_thickness.points();
        let thumb_min_length = metrics.thumb_min_length.points();
        let corner_radius = metrics.corner_radius.points();
        let fade_timeout = metrics.fade_timeout.points();
        let edge_inset = metrics.edge_inset.points();

        let min_w = self.min_scrolled_width.unwrap_or(0.0);
        let min_h = self.min_scrolled_height.unwrap_or(0.0);

        // 1. Configure underlying ScrollArea with AlwaysHidden scrollbars to eliminate 16px gutter reservation
        let mut area = ScrollArea::new([self.horizontal, self.vertical])
            .id_salt(self.id.with("scroll_area"))
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .scroll_source(egui::containers::scroll_area::ScrollSource::MOUSE_WHEEL)
            .auto_shrink([false, false])
            .min_scrolled_width(min_w)
            .min_scrolled_height(min_h);

        if let Some(mh) = self.max_height {
            area = area.max_height(mh);
        }
        if let Some(mw) = self.max_width {
            area = area.max_width(mw);
        }

        if let Some(ref o) = self.offset_override {
            area = area.scroll_offset(**o);
        } else if let Some(drag_o) = ui.data_mut(|d| {
            d.get_temp::<Option<Vec2>>(self.id.with("drag_offset"))
                .flatten()
        }) {
            area = area.scroll_offset(drag_o);
            ui.data_mut(|d| d.insert_temp(self.id.with("drag_offset"), None::<Vec2>));
        }

        let output = area.show(ui, content);
        let viewport_rect = output.inner_rect;
        let content_size = output.content_size;
        let mut current_offset = output.state.clamped_offset();

        // 2. Compute virtual viewport bounds and overflow
        let max_scroll_x = (content_size.x - viewport_rect.width()).max(0.0);
        let max_scroll_y = (content_size.y - viewport_rect.height()).max(0.0);
        current_offset.x = current_offset.x.clamp(0.0, max_scroll_x);
        current_offset.y = current_offset.y.clamp(0.0, max_scroll_y);

        // 3. Inactivity & Fade computation
        let prev_offset_id = self.id.with("prev_offset");
        let prev_offset: Vec2 =
            ui.data_mut(|d| *d.get_temp_mut_or_insert_with(prev_offset_id, || current_offset));
        let scrolled_this_frame = (current_offset - prev_offset).length_sq() > f32::EPSILON;
        ui.data_mut(|d| d.insert_temp(prev_offset_id, current_offset));

        let last_active_id = self.id.with("last_active_time");
        let now = ui.input(|i| i.time);
        let mut last_active: f64 =
            ui.data_mut(|d| *d.get_temp_mut_or_insert_with(last_active_id, || now));

        let is_pointer_in_viewport = ui.rect_contains_pointer(viewport_rect);
        let has_scroll_delta = ui.input(|i| {
            i.smooth_scroll_delta != Vec2::ZERO
                || i.raw
                    .events
                    .iter()
                    .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        });
        let pointer_moving = ui.input(|i| i.pointer.delta() != Vec2::ZERO);

        let hit_zone_padding = metrics.hit_zone_padding.points();
        let min_visible_thumb_len = metrics.min_visible_thumb_length.points();
        let zone_span = scrollbar_thickness + hit_zone_padding;
        let v_zone = Rect::from_min_max(
            Pos2::new(viewport_rect.right() - zone_span, viewport_rect.top()),
            viewport_rect.max,
        );
        let h_zone = Rect::from_min_max(
            Pos2::new(viewport_rect.left(), viewport_rect.bottom() - zone_span),
            viewport_rect.max,
        );
        let in_scrollbar_zone = (self.vertical && ui.rect_contains_pointer(v_zone))
            || (self.horizontal && ui.rect_contains_pointer(h_zone));

        if scrolled_this_frame
            || (is_pointer_in_viewport && has_scroll_delta)
            || (in_scrollbar_zone && pointer_moving)
        {
            last_active = now;
            ui.data_mut(|d| d.insert_temp(last_active_id, last_active));
        }

        let init_id = self.id.with("init_done");
        let initialized: bool = ui.data_mut(|d| *d.get_temp_mut_or_insert_with(init_id, || false));
        if !initialized {
            ui.data_mut(|d| d.insert_temp(init_id, true));
        }

        let has_v = self.vertical && max_scroll_y > 0.0;
        let has_h = self.horizontal && max_scroll_x > 0.0;
        let elapsed = (now - last_active).max(0.0);
        let timeout_secs = fade_timeout as f64;
        let fade_duration = timeout_secs * 0.5 * 0.5;
        let alpha = if !self.auto_hide {
            1.0
        } else if !(has_v || has_h) {
            0.0
        } else if !initialized || elapsed < timeout_secs {
            let remaining = (timeout_secs - elapsed) as f32;
            if remaining > 0.0 {
                ui.ctx().request_repaint_after_secs(remaining);
            } else {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(16));
            }
            1.0
        } else if elapsed < timeout_secs + fade_duration {
            let t = ((elapsed - timeout_secs) / fade_duration).clamp(0.0, 1.0) as f32;
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
            1.0 - t
        } else {
            0.0
        };

        let painter = ui.painter_at(viewport_rect);

        // Helper for thumb styling derived from MacTheme semantic tokens
        let get_thumb_color = |hovered: bool, dragged: bool| -> Color32 {
            theme.scrollbar_thumb_color(hovered || dragged, alpha)
        };
        let track_bg_color = theme.control().gamma_multiply(alpha * 0.5);

        // 4. Vertical Floating Overlay Scrollbar
        if has_v {
            let trail = if has_h {
                edge_inset * 2.0 + scrollbar_thickness
            } else {
                edge_inset
            };
            if render_overlay_thumb(
                ui,
                &painter,
                self.id.with("v_thumb"),
                viewport_rect,
                viewport_rect.height(),
                content_size.y,
                edge_inset,
                trail,
                edge_inset,
                scrollbar_thickness,
                scrollbar_hover_thickness,
                corner_radius,
                thumb_min_length,
                min_visible_thumb_len,
                &mut current_offset.y,
                max_scroll_y,
                true,
                alpha,
                track_bg_color,
                &get_thumb_color,
                last_active_id,
                now,
            ) && self.offset_override.is_none()
            {
                ui.data_mut(|d| d.insert_temp(self.id.with("drag_offset"), Some(current_offset)));
            }
        }

        // 5. Horizontal Floating Overlay Scrollbar
        if has_h {
            let trail = if has_v {
                edge_inset * 2.0 + scrollbar_thickness
            } else {
                edge_inset
            };
            if render_overlay_thumb(
                ui,
                &painter,
                self.id.with("h_thumb"),
                viewport_rect,
                viewport_rect.width(),
                content_size.x,
                edge_inset,
                trail,
                edge_inset,
                scrollbar_thickness,
                scrollbar_hover_thickness,
                corner_radius,
                thumb_min_length,
                min_visible_thumb_len,
                &mut current_offset.x,
                max_scroll_x,
                false,
                alpha,
                track_bg_color,
                &get_thumb_color,
                last_active_id,
                now,
            ) && self.offset_override.is_none()
            {
                ui.data_mut(|d| d.insert_temp(self.id.with("drag_offset"), Some(current_offset)));
            }
        }

        if self.auto_hide && (has_v || has_h) {
            let updated_last_active: f64 = ui
                .data(|d| d.get_temp(last_active_id))
                .unwrap_or(last_active);
            if updated_last_active > last_active {
                if alpha < 1.0 {
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(16));
                } else {
                    ui.ctx().request_repaint_after_secs(fade_timeout);
                }
            }
        }

        // 6. Persist updated offset
        if let Some(ref mut o) = self.offset_override {
            **o = current_offset;
        }

        MacScrollViewOutput {
            inner: output.inner,
            inner_rect: viewport_rect,
            content_size,
            offset: current_offset,
        }
    }
}
