//! Authentic macOS Toggle / Switch.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].

use crate::theme::MacTheme;
use egui::{CornerRadius, Pos2, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget};

/// Authentic macOS Toggle / Switch.
///
/// Features animated sliding knob via `CASpring`, smooth green activation color transition,
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
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacToggle<'a> {
    /// Creates a new toggle switch bound to a mutable `bool` reference.
    pub fn new(value: &'a mut bool) -> Self {
        Self {
            value,
            enabled: true,
            theme: None,
        }
    }

    /// Enables or disables the toggle switch.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl<'a> Widget for MacToggle<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;

        let metrics = theme.toggle();
        let height = metrics.height.points();
        let width = metrics.width.points();
        let radius = metrics.corner_radius.points() as u8;

        let desired_size = Vec2::new(width, height);
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, mut response) = ui.allocate_exact_size(desired_size, sense);
        if self.enabled && response.clicked() {
            *self.value = !*self.value;
            response.mark_changed();
        }

        // CASpring thumb interpolation stored in egui temp memory
        let spring = theme.motion().swiftui_bouncy.to_spring();
        let spring_key = response.id.with("toggle_caspring");
        let target_val = if *self.value { 1.0f32 } else { 0.0f32 };
        let anim_progress = spring
            .animate_f32(ui.ctx(), spring_key, target_val)
            .clamp(0.0, 1.0);

        let alpha_factor = if self.enabled {
            1.0
        } else {
            theme.disabled_opacity()
        };
        let painter = ui.painter_at(rect);
        let bg_color = theme
            .control()
            .lerp_to_gamma(theme.system_green(), anim_progress)
            .gamma_multiply(alpha_factor);

        painter.rect(
            rect,
            CornerRadius::same(radius),
            bg_color,
            Stroke::new(
                metrics.border_width.points(),
                theme.separator().gamma_multiply(alpha_factor),
            ),
            StrokeKind::Inside,
        );

        let knob_radius = metrics.knob_size.points() * 0.5;
        let x_off = rect.min.x + metrics.knob_offset_off.points() + knob_radius;
        let x_on = rect.min.x + metrics.knob_offset_on.points() + knob_radius;
        let knob_x = x_off + (x_on - x_off) * anim_progress;
        let knob_center = Pos2::new(knob_x, rect.center().y);

        painter.circle_filled(
            knob_center + Vec2::new(0.0, metrics.border_width.points()),
            knob_radius,
            theme.drop_shadow().gamma_multiply(alpha_factor),
        );
        painter.circle_filled(
            knob_center,
            knob_radius,
            theme.knob_fill().gamma_multiply(alpha_factor),
        );

        response
    }
}
