//! Authentic macOS Toggle / Switch.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].

use crate::controls::MacControlSize;
use crate::theme::MacTheme;
use egui::{Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

/// Standalone AppKit `NSSwitch` pill switch alias (`REQ-013-02A`).
pub type MacSwitch<'a> = MacToggle<'a>;

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
    control_size: MacControlSize,
    fixed_rect: Option<Rect>,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacToggle<'a> {
    /// Creates a new toggle switch bound to a mutable `bool` reference.
    pub fn new(value: &'a mut bool) -> Self {
        Self { value, enabled: true, control_size: MacControlSize::Regular, fixed_rect: None, theme: None }
    }

    /// Enables or disables the toggle switch.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Configures the AppKit control size (`Regular`, `Small`, `Mini`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Pins the switch to an exact NIB or scene layout rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the switch inside `ui` and returns the interaction [`Response`].
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
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
        let sense = if self.enabled && ui.is_enabled() { Sense::click() } else { Sense::hover() };
        let (rect, mut response) = if let Some(fixed) = self.fixed_rect {
            let id = ui.next_auto_id().with("mac_switch_fixed");
            ui.skip_ahead_auto_ids(1);
            let resp = ui.interact(fixed, id, sense);
            (fixed, resp)
        } else {
            ui.allocate_exact_size(desired_size, sense)
        };
        if self.enabled && ui.is_enabled() && response.clicked() {
            *self.value = !*self.value;
            response.mark_changed();
        }

        // CASpring thumb interpolation stored in egui temp memory
        let spring = theme.motion().swiftui_bouncy.to_spring();
        let spring_key = response.id.with("toggle_caspring");
        let target_val = if *self.value { 1.0f32 } else { 0.0f32 };
        let anim_progress = spring.animate_f32(ui.ctx(), spring_key, target_val).clamp(0.0, 1.0);

        let alpha_factor = if self.enabled { 1.0 } else { theme.disabled_opacity() };
        let painter = ui.painter_at(rect);
        let active_green = if crate::layout::is_window_active(ui.ctx()) {
            theme.system_green()
        } else {
            theme.selected_control_inactive()
        };
        let bg_color = theme.control().lerp_to_gamma(active_green, anim_progress).gamma_multiply(alpha_factor);

        super::liquid_glass::paint_liquid_glass_capsule(
            &painter,
            rect,
            radius as f32,
            bg_color,
            Stroke::new(metrics.border_width.points(), theme.separator().gamma_multiply(alpha_factor)),
            theme.specular_rim_top().gamma_multiply(alpha_factor),
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
        painter.circle_filled(knob_center, knob_radius, theme.knob_fill().gamma_multiply(alpha_factor));

        response
    }
}
