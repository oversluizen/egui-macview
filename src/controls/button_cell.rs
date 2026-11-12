//! Authentic macOS AppKit Push Button / Tool Button.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].
//! Fully data-driven: zero hardcoded colors or dimensions.

use crate::controls::MacControlSize;
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{Color32, CornerRadius, Rect, Response, Stroke, StrokeKind, Ui, Vec2, Widget};

/// Authentic macOS AppKit Push Button / Tool Button (`NSButtonCell` equivalent).
///
/// Features authentic theme-driven background, border, selection states,
/// typography metrics with Apple HIG optical tracking, and hover feedback.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacButtonCell, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
///
/// if ui.add(MacButtonCell::new("Save Changes").theme(&theme)).clicked() {
///     // Handle save
/// }
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacButtonCell<'a> {
    text: &'a str,
    selected: bool,
    enabled: bool,
    theme: Option<&'a MacTheme>,
    control_size: MacControlSize,
    width: Option<f32>,
    height: Option<f32>,
    corner_radius: Option<f32>,
    fixed_rect: Option<Rect>,
}

/// Convenience alias for [`MacButtonCell`].
pub type MacButton<'a> = MacButtonCell<'a>;

impl<'a> MacButtonCell<'a> {
    /// Creates a new button with label `text`.
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            selected: false,
            enabled: true,
            theme: None,
            control_size: MacControlSize::Regular,
            width: None,
            height: None,
            corner_radius: None,
            fixed_rect: None,
        }
    }

    /// Sets the AppKit control size (`Regular` `22pt`, `Small` `18pt`, or `Mini` `15pt`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Pins the button cell to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Renders the button cell into `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }

    /// Enables or disables the button cell.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets whether the button is presented in an active/selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Overrides button width in points (defaults to content width + padding).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides button height in points (defaults to token regular button height).
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Overrides corner radius in points (defaults to token corner radius).
    pub fn corner_radius(mut self, r: f32) -> Self {
        self.corner_radius = Some(r);
        self
    }
}

impl<'a> Widget for MacButtonCell<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;

        let metrics = match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => &theme.button().regular,
            MacControlSize::Small | MacControlSize::CompactInfobox => &theme.button().small,
            MacControlSize::Mini => &theme.button().mini,
        };
        let height = self
            .fixed_rect
            .map(|r| r.height())
            .or(self.height)
            .unwrap_or_else(|| metrics.height.points());
        let font_sz = if height <= theme.button().mini.height.points() {
            theme.button().mini.font_size.points()
        } else if height <= theme.button().small.height.points() {
            theme.button().small.font_size.points()
        } else {
            metrics.font_size.points()
        };
        let corner_radius = self
            .corner_radius
            .unwrap_or_else(|| metrics.corner_radius.points());
        let padding_h = metrics
            .padding_horizontal
            .as_ref()
            .map(|d| d.points())
            .unwrap_or(font_sz);

        let font_id = egui::FontId::proportional(font_sz);
        let text_layout = theme.layout_tracked_galley(
            ui,
            self.text,
            font_id,
            Color32::PLACEHOLDER,
            f32::INFINITY,
        );
        let desired_width = self
            .width
            .unwrap_or_else(|| text_layout.size().x + padding_h * 2.0);

        let enabled = self.enabled && ui.is_enabled();
        let sense = if enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        };
        let ppp = ui.ctx().pixels_per_point();
        let (rect, response) = if let Some(fixed) = self.fixed_rect {
            let snapped = subpixel_snap_rect(fixed, ppp);
            let id = ui.next_auto_id().with("mac_button_cell_fixed");
            ui.skip_ahead_auto_ids(1);
            let resp = ui.interact(snapped, id, sense);
            (snapped, resp)
        } else {
            let (r, resp) = ui.allocate_exact_size(Vec2::new(desired_width, height), sense);
            (subpixel_snap_rect(r, ppp), resp)
        };
        let painter = ui.painter_at(rect);

        let alpha_factor = if enabled {
            1.0
        } else {
            theme.disabled_opacity()
        };

        // Dynamic AppKit styling (pressed feedback works even when selected == true)
        let base_bg = if !enabled {
            if self.selected {
                theme.selected_control_inactive()
            } else {
                theme.control_background()
            }
        } else if response.is_pointer_button_down_on() {
            theme.selected_control_inactive()
        } else if self.selected {
            theme.selected_control()
        } else if response.hovered() {
            theme.control()
        } else {
            theme.control_background()
        };
        let bg_color = base_bg.gamma_multiply(alpha_factor);

        let border_stroke = Stroke::new(
            metrics.border_width.points(),
            theme.separator().gamma_multiply(alpha_factor),
        );

        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            bg_color,
            border_stroke,
            StrokeKind::Inside,
        );

        let bg_style = theme.background_style(base_bg);
        let text_color = bg_style
            .primary_text_color(theme)
            .gamma_multiply(alpha_factor);

        let text_pos = egui::Align2::CENTER_CENTER
            .anchor_size(rect.center(), text_layout.size())
            .min;
        painter.galley(text_pos, text_layout, text_color);

        response
    }
}
