//! Authentic macOS AppKit Card Surface Container.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].
//! Features dynamic background fill, optional ambient drop shadow, border stroke,
//! and automatic [`MacBackgroundStyle`] contrast resolution for child content.

use crate::theme::{contrast::MacBackgroundStyle, MacTheme};
use egui::{CornerRadius, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

/// Authentic macOS AppKit Card surface container (`NSBox` / Card style).
///
/// Automatically evaluates [`MacBackgroundStyle`] from its resolved background fill,
/// ensuring text and icons placed inside the card always satisfy WCAG 2.1 AA (>= 4.5:1).
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacCard, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let card_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 120.0));
///
/// MacCard::new()
///     .theme(&theme)
///     .show(ui, card_rect, |card_ui, bg_style| {
///         let text_color = bg_style.primary_text_color(&theme);
///         card_ui.label(egui::RichText::new("Card Content").color(text_color));
///     });
/// # });
/// # output.textures_delta.clear();
/// ```
#[derive(Default)]
pub struct MacCard<'a> {
    theme: Option<&'a MacTheme>,
    fill: Option<egui::Color32>,
    stroke: Option<Stroke>,
    corner_radius: Option<f32>,
    shadow: bool,
    width: Option<f32>,
    height: Option<f32>,
}

impl<'a> MacCard<'a> {
    /// Creates a new card surface container.
    pub fn new() -> Self {
        Self::default()
    }

    /// Injects dynamic theme to resolve default metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets custom background fill color.
    pub fn fill(mut self, fill: egui::Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Overrides border stroke.
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Overrides card corner radius in points.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Enables or disables ambient drop shadow.
    pub fn shadow(mut self, shadow: bool) -> Self {
        self.shadow = shadow;
        self
    }

    /// Sets desired width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets desired height in points.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Resolves the [`MacBackgroundStyle`] for the card's active background fill.
    pub fn background_style(&self, theme: &MacTheme) -> MacBackgroundStyle {
        let fill = self.fill.unwrap_or_else(|| theme.control_background());
        MacBackgroundStyle::from_color(fill)
    }

    /// Renders the card within the specified `rect` and invokes `add_contents`
    /// passing child UI and resolved [`MacBackgroundStyle`].
    pub fn show<R>(
        self,
        ui: &mut Ui,
        rect: Rect,
        add_contents: impl FnOnce(&mut Ui, MacBackgroundStyle) -> R,
    ) -> (Response, R) {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;

        let metrics = theme.popover_metrics();
        let rounding = self
            .corner_radius
            .unwrap_or_else(|| metrics.corner_radius.points());
        let bg_fill = self.fill.unwrap_or_else(|| theme.control_background());
        let border_stroke = self
            .stroke
            .unwrap_or_else(|| Stroke::new(metrics.border_width.points(), theme.separator()));

        let resp = ui.interact(
            rect,
            ui.scope_id().with("mac_card"),
            Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);

        // Ambient drop shadow if enabled
        if self.shadow {
            let shadow_offset = metrics.border_width.points() * 2.0;
            let shadow_rect = rect.translate(Vec2::new(0.0, shadow_offset));
            painter.rect_filled(
                shadow_rect,
                CornerRadius::same((rounding + metrics.border_width.points()) as u8),
                theme.drop_shadow(),
            );
        }

        // Card body surface
        painter.rect(
            rect,
            CornerRadius::same(rounding as u8),
            bg_fill,
            border_stroke,
            StrokeKind::Inside,
        );

        let bg_style = MacBackgroundStyle::from_color(bg_fill);
        let inner_padding = metrics.inner_padding.points();
        let content_rect = rect.shrink(inner_padding);

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((resp.id, "mac_card_content"))
                .max_rect(content_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        child_ui.visuals_mut().panel_fill = bg_fill;
        let surface_slot = theme.register_surface_rect_slot(ui.ctx(), Rect::NOTHING, bg_fill);
        let ret = add_contents(&mut child_ui, bg_style);
        theme.update_surface_rect_slot(ui.ctx(), surface_slot, rect);

        (resp, ret)
    }

    /// Automatically allocates card size and renders contents.
    pub fn show_content<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui, MacBackgroundStyle) -> R,
    ) -> (Response, R) {
        let desired_w = self.width.unwrap_or_else(|| ui.available_width());
        let desired_h = self.height.unwrap_or_else(|| {
            if let Some(t) = self.theme {
                t.popover_metrics().compact_panel_height.points()
            } else {
                ui.available_height()
            }
        });

        let (rect, _) = ui.allocate_exact_size(Vec2::new(desired_w, desired_h), Sense::hover());
        self.show(ui, rect, add_contents)
    }
}
