//! macOS AppKit `NSPopover` Floating Surface (`MacPopover`, `MacPopoverArrowEdge`).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].
//! Fully data-driven: zero hardcoded colors or dimensions.

use crate::theme::MacTheme;
use egui::{CornerRadius, Id, Pos2, Rect, Response, Stroke, StrokeKind, Ui, Vec2};
use std::hash::Hash;

/// Direction that the popover arrow points towards the anchor element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacPopoverArrowEdge {
    /// Arrow emerges from the top edge pointing upwards.
    #[default]
    Top,
    /// Arrow emerges from the bottom edge pointing downwards.
    Bottom,
    /// Arrow emerges from the left edge pointing leftwards.
    Left,
    /// Arrow emerges from the right edge pointing rightwards.
    Right,
    /// No arrow indicator (bubble only).
    None,
}

/// Authentic macOS AppKit NSPopover floating surface.
///
/// Features authentic balloon bubble geometry, arrow pointer directed at anchor points,
/// dynamic scaling and opacity transitions, and theme-driven background/border colors.
///
/// # Example
/// ```rust
/// use egui_macview::{MacPopover, MacPopoverArrowEdge, MacTheme};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let rect = egui::Rect::from_min_size(egui::Pos2::new(50.0, 50.0), egui::vec2(160.0, 100.0));
/// MacPopover::new().theme(&theme).arrow_edge(MacPopoverArrowEdge::Top).show(ui, rect, |_| {});
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacPopover<'a> {
    id: Option<Id>,
    theme: Option<&'a MacTheme>,
    arrow_edge: MacPopoverArrowEdge,
    arrow_offset: Option<f32>,
    scale: f32,
    alpha: f32,
}

impl<'a> Default for MacPopover<'a> {
    fn default() -> Self {
        Self {
            id: None,
            theme: None,
            arrow_edge: MacPopoverArrowEdge::Top,
            arrow_offset: None,
            scale: 1.0,
            alpha: 1.0,
        }
    }
}

impl<'a> MacPopover<'a> {
    /// Creates a new popover surface.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets an explicit widget `Id` for this popover.
    pub fn id(mut self, id_source: impl Hash + std::fmt::Debug) -> Self {
        self.id = Some(Id::NULL.with(id_source));
        self
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Specifies the edge where the directional arrow emerges.
    pub fn arrow_edge(mut self, edge: MacPopoverArrowEdge) -> Self {
        self.arrow_edge = edge;
        self
    }

    /// Sets custom offset along the arrow edge in points (defaults to center).
    pub fn arrow_offset(mut self, offset: f32) -> Self {
        self.arrow_offset = Some(offset);
        self
    }

    /// Scales the popover geometry around its center (used during zoom transitions).
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Sets opacity alpha multiplier in `[0.0, 1.0]` for fade transitions.
    pub fn alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    /// Renders the popover balloon and invokes `add_contents` for inner child widgets.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        rect: Rect,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> (Response, R) {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;

        let metrics = theme.popover_metrics();
        let corner_radius = metrics.corner_radius.points();
        let border_width = metrics.border_width.points();
        let arrow_w = metrics.arrow_base_width.points();
        let arrow_h = metrics.arrow_height.points();

        // Apply scale transformation around rect center
        let center = rect.center();
        let scaled_size = rect.size() * self.scale.max(f32::EPSILON);
        let active_rect = Rect::from_center_size(center, scaled_size);

        let popover_id = self.id.unwrap_or_else(|| {
            ui.scope_id().with((
                "mac_popover",
                rect.min.x.to_bits(),
                rect.min.y.to_bits(),
                self.arrow_edge as u8,
            ))
        });
        let resp = ui.interact(active_rect, popover_id, egui::Sense::click_and_drag());
        if ui.rect_contains_pointer(active_rect) {
            ui.input_mut(|i| {
                i.smooth_scroll_delta = Vec2::ZERO;
            });
        }

        let bg_color = theme.window_background().gamma_multiply(self.alpha);
        let border_color = theme.separator().gamma_multiply(self.alpha);
        let stroke = Stroke::new(border_width, border_color);

        let painter = ui.painter_at(active_rect);

        // Body bubble
        painter.rect(
            active_rect,
            CornerRadius::same(corner_radius as u8),
            bg_color,
            stroke,
            StrokeKind::Inside,
        );

        if self.arrow_edge != MacPopoverArrowEdge::None {
            let arrow_points = match self.arrow_edge {
                MacPopoverArrowEdge::Top => {
                    let ax = self.arrow_offset.unwrap_or(active_rect.center().x);
                    [Pos2::new(ax - arrow_w * 0.5, active_rect.min.y), Pos2::new(ax, active_rect.min.y - arrow_h), Pos2::new(ax + arrow_w * 0.5, active_rect.min.y)]
                }
                MacPopoverArrowEdge::Bottom => {
                    let ax = self.arrow_offset.unwrap_or(active_rect.center().x);
                    [Pos2::new(ax - arrow_w * 0.5, active_rect.max.y), Pos2::new(ax, active_rect.max.y + arrow_h), Pos2::new(ax + arrow_w * 0.5, active_rect.max.y)]
                }
                MacPopoverArrowEdge::Left => {
                    let ay = self.arrow_offset.unwrap_or(active_rect.center().y);
                    [Pos2::new(active_rect.min.x, ay - arrow_w * 0.5), Pos2::new(active_rect.min.x - arrow_h, ay), Pos2::new(active_rect.min.x, ay + arrow_w * 0.5)]
                }
                MacPopoverArrowEdge::Right | MacPopoverArrowEdge::None => {
                    let ay = self.arrow_offset.unwrap_or(active_rect.center().y);
                    [Pos2::new(active_rect.max.x, ay - arrow_w * 0.5), Pos2::new(active_rect.max.x + arrow_h, ay), Pos2::new(active_rect.max.x, ay + arrow_w * 0.5)]
                }
            };
            painter.add(egui::Shape::convex_polygon(arrow_points.to_vec(), bg_color, Stroke::NONE));
            painter.line_segment([arrow_points[0], arrow_points[1]], stroke);
            painter.line_segment([arrow_points[1], arrow_points[2]], stroke);
            painter.line_segment([arrow_points[0], arrow_points[2]], Stroke::new(border_width * 2.0, bg_color));
        }

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((popover_id, "mac_popover_content"))
                .max_rect(active_rect.shrink(corner_radius.min(metrics.inner_padding.points())))
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        let ret = add_contents(&mut child_ui);
        (resp, ret)
    }
}

/// Authentic macOS AppKit `NSPopover` container surface (`MacPopoverSurface`).
///
/// Renders a popover surface container with `surface_corner_radius` (`8.0pt`),
/// `surface_border_width` (`1.0pt`), `popover_background`, `popover_border`, and
/// automatic surface vibrancy registration for nested child controls.
#[derive(Default)]
pub struct MacPopoverSurface<'a> {
    id: Option<Id>,
    theme: Option<&'a MacTheme>,
    fixed_rect: Option<Rect>,
    corner_radius: Option<f32>,
}

impl<'a> MacPopoverSurface<'a> {
    /// Creates a new AppKit `NSPopover` container surface.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets an explicit widget `Id` for this popover surface.
    pub fn id(mut self, id_source: impl Hash + std::fmt::Debug) -> Self {
        self.id = Some(Id::NULL.with(id_source));
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Pins the popover surface to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Overrides the corner radius in points (defaults to `surface_corner_radius` = `8.0pt`).
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Paints the popover surface at `rect` and registers its surface vibrancy slot.
    pub fn paint_at(self, ui: &mut Ui, rect: Rect) -> Response {
        self.fixed_rect(rect).show(ui, |_| {}).response
    }

    /// Renders the popover surface and invokes `add_contents` inside the padded child `Ui`.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.popover_metrics();
        let rect = self.fixed_rect.unwrap_or_else(|| {
            let size = Vec2::new(metrics.full_popover_width.points(), metrics.full_popover_height.points());
            ui.allocate_exact_size(size, egui::Sense::hover()).0
        });
        let surface_id = self.id.unwrap_or_else(|| {
            ui.scope_id().with(("mac_popover_surface", rect.min.x.to_bits(), rect.min.y.to_bits()))
        });
        let response = ui.interact(rect, surface_id, egui::Sense::click_and_drag());
        let rounding = self.corner_radius.unwrap_or_else(|| metrics.surface_corner_radius.points());
        let border_w = metrics.surface_border_width.points();
        let bg_fill = theme.popover_background();
        let border_stroke = Stroke::new(border_w, theme.popover_border());
        ui.painter_at(rect).rect(
            rect,
            CornerRadius::same(rounding as u8),
            bg_fill,
            border_stroke,
            StrokeKind::Inside,
        );
        let pad = metrics.inner_padding.points();
        let content_rect = rect.shrink2(Vec2::new(pad.min(rect.width() * 0.5), pad.min(rect.height() * 0.5)));
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((surface_id, "mac_popover_surface_content"))
                .max_rect(content_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        child_ui.visuals_mut().panel_fill = bg_fill;
        let surface_slot = theme.register_surface_rect_slot(ui.ctx(), rect, bg_fill);
        let inner = add_contents(&mut child_ui);
        theme.update_surface_rect_slot(ui.ctx(), surface_slot, rect);
        egui::InnerResponse::new(inner, response)
    }
}
