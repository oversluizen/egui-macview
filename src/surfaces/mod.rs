//! macOS AppKit Panel, Popover & Card Surfaces.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].
//! Fully data-driven: zero hardcoded colors or dimensions.

pub mod card;
pub mod hud;
pub use card::MacCard;
pub use hud::MacFloatingInfoBox;

use crate::theme::MacTheme;
use egui::{CornerRadius, Id, Pos2, Rect, Response, Stroke, StrokeKind, Ui, Vec2};
use std::hash::Hash;

/// Authentic macOS NSPanel floating surface container.
///
/// Features authentic window background fill, border hairline strokes, optional
/// title bar with centered title text, and customizable corner rounding.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacPanel, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let panel_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 150.0));
///
/// MacPanel::new()
///     .theme(&theme)
///     .title("Inspector")
///     .show(ui, panel_rect, |panel_ui| {
///         panel_ui.label("Panel Content");
///     });
/// # });
/// # output.textures_delta.clear();
/// ```
#[derive(Default)]
pub struct MacPanel<'a> {
    id: Option<Id>,
    title: Option<String>,
    theme: Option<&'a MacTheme>,
    rounding: Option<u8>,
}

impl<'a> MacPanel<'a> {
    /// Creates a new floating panel surface.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets an explicit widget `Id` for this panel.
    pub fn id(mut self, id_source: impl Hash + std::fmt::Debug) -> Self {
        self.id = Some(Id::NULL.with(id_source));
        self
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets optional panel title displayed in the header bar.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Overrides panel corner rounding in points.
    pub fn rounding(mut self, r: u8) -> Self {
        self.rounding = Some(r);
        self
    }

    /// Renders the panel within `rect` and invokes `add_contents` for child widgets.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        rect: Rect,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> (Response, R) {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::from_ctx(ui.ctx());
                &fallback_theme
            }
        };

        let panel_id = self.id.unwrap_or_else(|| {
            ui.scope_id().with((
                "mac_panel",
                self.title.as_deref().unwrap_or(""),
                rect.min.x.to_bits(),
                rect.min.y.to_bits(),
            ))
        });
        let resp = ui.interact(rect, panel_id, egui::Sense::click_and_drag());
        if ui.rect_contains_pointer(rect) {
            ui.input_mut(|i| {
                i.smooth_scroll_delta = Vec2::ZERO;
            });
        }

        let popover_metrics = theme.popover_metrics();
        let rounding = self
            .rounding
            .unwrap_or_else(|| popover_metrics.corner_radius.points() as u8);

        let bg_color = theme.window_background();
        let border_stroke = Stroke::new(popover_metrics.border_width.points(), theme.separator());

        let painter = ui.painter_at(rect);
        painter.rect(
            rect,
            CornerRadius::same(rounding),
            bg_color,
            border_stroke,
            StrokeKind::Inside,
        );

        let header_height = popover_metrics.header_height.points();

        let content_rect = if let Some(title) = self.title {
            let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_height));
            painter.line_segment(
                [
                    Pos2::new(rect.min.x, rect.min.y + header_height),
                    Pos2::new(rect.max.x, rect.min.y + header_height),
                ],
                border_stroke,
            );
            painter.text(
                title_rect.center(),
                egui::Align2::CENTER_CENTER,
                title,
                egui::FontId::proportional(popover_metrics.header_font_size.points()),
                theme.label(),
            );
            Rect::from_min_max(Pos2::new(rect.min.x, rect.min.y + header_height), rect.max)
        } else {
            rect
        };

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((panel_id, "mac_panel_content"))
                .max_rect(content_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        let ret = add_contents(&mut child_ui);
        (resp, ret)
    }
}

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
///
/// ```rust
/// use egui_macview::{MacPopover, MacPopoverArrowEdge, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let popover_rect = egui::Rect::from_min_size(egui::Pos2::new(50.0, 50.0), egui::vec2(160.0, 100.0));
///
/// MacPopover::new()
///     .theme(&theme)
///     .arrow_edge(MacPopoverArrowEdge::Top)
///     .show(ui, popover_rect, |popover_ui| {
///         popover_ui.label("Popover Content");
///     });
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
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::from_ctx(ui.ctx());
                &fallback_theme
            }
        };

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

        // Draw arrow if applicable
        if self.arrow_edge != MacPopoverArrowEdge::None {
            let mut arrow_points = [Pos2::ZERO; 3];
            match self.arrow_edge {
                MacPopoverArrowEdge::Top => {
                    let ax = self.arrow_offset.unwrap_or(active_rect.center().x);
                    arrow_points = [
                        Pos2::new(ax - arrow_w * 0.5, active_rect.min.y),
                        Pos2::new(ax, active_rect.min.y - arrow_h),
                        Pos2::new(ax + arrow_w * 0.5, active_rect.min.y),
                    ];
                }
                MacPopoverArrowEdge::Bottom => {
                    let ax = self.arrow_offset.unwrap_or(active_rect.center().x);
                    arrow_points = [
                        Pos2::new(ax - arrow_w * 0.5, active_rect.max.y),
                        Pos2::new(ax, active_rect.max.y + arrow_h),
                        Pos2::new(ax + arrow_w * 0.5, active_rect.max.y),
                    ];
                }
                MacPopoverArrowEdge::Left => {
                    let ay = self.arrow_offset.unwrap_or(active_rect.center().y);
                    arrow_points = [
                        Pos2::new(active_rect.min.x, ay - arrow_w * 0.5),
                        Pos2::new(active_rect.min.x - arrow_h, ay),
                        Pos2::new(active_rect.min.x, ay + arrow_w * 0.5),
                    ];
                }
                MacPopoverArrowEdge::Right => {
                    let ay = self.arrow_offset.unwrap_or(active_rect.center().y);
                    arrow_points = [
                        Pos2::new(active_rect.max.x, ay - arrow_w * 0.5),
                        Pos2::new(active_rect.max.x + arrow_h, ay),
                        Pos2::new(active_rect.max.x, ay + arrow_w * 0.5),
                    ];
                }
                MacPopoverArrowEdge::None => {}
            }

            if self.arrow_edge != MacPopoverArrowEdge::None {
                painter.add(egui::Shape::convex_polygon(
                    arrow_points.to_vec(),
                    bg_color,
                    Stroke::NONE,
                ));
                // Draw the two exposed outer legs of the arrow
                painter.line_segment([arrow_points[0], arrow_points[1]], stroke);
                painter.line_segment([arrow_points[1], arrow_points[2]], stroke);
                // Clear the body rectangle border along the arrow base for seamless interior continuity
                painter.line_segment(
                    [arrow_points[0], arrow_points[2]],
                    Stroke::new(border_width * 2.0, bg_color),
                );
            }
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
