//! macOS AppKit Panel, Popover & Card Surfaces.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].
//! Fully data-driven: zero hardcoded colors or dimensions.

pub mod card;
pub mod hud;
pub mod popover;
pub use card::MacCard;
pub use hud::MacFloatingInfoBox;
pub use popover::{MacPopover, MacPopoverArrowEdge, MacPopoverSurface};

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
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;

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
