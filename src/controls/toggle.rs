//! Authentic macOS Toggle / Switch.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].

use crate::theme::MacTheme;
use egui::{Color32, CornerRadius, Pos2, Response, Stroke, StrokeKind, Ui, Vec2, Widget};

/// Authentic macOS Toggle / Switch.
///
/// Features animated sliding knob, smooth green activation color transition,
/// ambient drop shadow, and theme-driven metrics.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacTheme, MacToggle};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut enabled = true;
///
/// ui.add(MacToggle::new(&mut enabled).theme(&theme));
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacToggle<'a> {
    value: &'a mut bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacToggle<'a> {
    /// Creates a new toggle switch bound to a mutable `bool` reference.
    pub fn new(value: &'a mut bool) -> Self {
        Self { value, theme: None }
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl<'a> Widget for MacToggle<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let metrics = theme.toggle();
        let height = metrics.height.points();
        let width = metrics.width.points();
        let radius = metrics.corner_radius.points() as u8;

        let desired_size = Vec2::new(width, height);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        if response.clicked() {
            *self.value = !*self.value;
            response.mark_changed();
        }

        let anim_progress = ui.ctx().animate_bool_with_time(
            response.id.with("toggle_anim"),
            *self.value,
            metrics.animation_duration.points(),
        );

        let painter = ui.painter_at(rect);
        let bg_color = theme
            .control()
            .lerp_to_gamma(theme.system_green(), anim_progress);

        painter.rect(
            rect,
            CornerRadius::same(radius),
            bg_color,
            Stroke::new(metrics.border_width.points(), theme.separator()),
            StrokeKind::Inside,
        );

        let knob_radius = metrics.knob_size.points() * 0.5;
        let x_off = rect.min.x + metrics.knob_offset_off.points() + knob_radius;
        let x_on = rect.min.x + metrics.knob_offset_on.points() + knob_radius;
        let knob_x = x_off + (x_on - x_off) * anim_progress;
        let knob_center = Pos2::new(knob_x, rect.center().y);

        // Subtle ambient shadow under knob
        painter.circle_filled(
            knob_center + Vec2::new(0.0, 1.0),
            knob_radius,
            Color32::from_black_alpha(30),
        );
        painter.circle_filled(knob_center, knob_radius, Color32::WHITE);

        response
    }
}
