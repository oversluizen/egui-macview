//! Authentic macOS AppKit Help Button (`NSBezelStyleHelpButton`) and Inline Action Badge (`v0.6.5`).

use crate::controls::MacControlSize;
use crate::layout::subpixel_snap_rect;
use crate::symbols::MacSymbol;
use crate::theme::MacTheme;
use egui::{Align2, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

/// Authentic macOS AppKit Circular Help Button (`NSBezelStyleHelpButton`).
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacHelpButton, MacTheme};
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// if MacHelpButton::new().theme(&theme).show(ui).clicked() {}
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacHelpButton<'a> {
    control_size: MacControlSize,
    fixed_rect: Option<Rect>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> Default for MacHelpButton<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MacHelpButton<'a> {
    /// Creates a new circular AppKit help button (`?`).
    pub fn new() -> Self {
        Self {
            control_size: MacControlSize::Regular,
            fixed_rect: None,
            enabled: true,
            theme: None,
        }
    }

    /// Sets the AppKit control size (`Regular` `20pt`, `Small` `16pt`, `Mini` `13pt`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Pins the help button to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Enables or disables interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Returns the visual circular diameter in points (`20.0pt`, `16.0pt`, or `13.0pt`).
    pub fn diameter(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let hm = theme_cow.help_button();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => hm.diameter.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => hm.small_diameter.points(),
            MacControlSize::Mini => hm.mini_diameter.points(),
        }
    }

    /// Returns the `?` glyph font size in points (`12.0pt` Regular, `10.0pt` Small, `9.0pt` Mini).
    pub fn font_size(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let hm = theme_cow.help_button();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => hm.font_size.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => hm.small_font_size.points(),
            MacControlSize::Mini => hm.mini_font_size.points(),
        }
    }

    /// Renders the help button into `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacHelpButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let hm = theme.help_button();

        let enabled = self.enabled && ui.is_enabled();
        let diam = self.diameter();
        let border_w = hm.border_width.points();
        let shadow_dy = hm.shadow_offset_y.points();
        let font_sz = self.font_size();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let ppp = ui.ctx().pixels_per_point();
        let (rect, response) = if let Some(fixed) = self.fixed_rect {
            let snapped = subpixel_snap_rect(fixed, ppp);
            let id = ui.next_auto_id().with("mac_help_btn_fixed");
            ui.skip_ahead_auto_ids(1);
            (snapped, ui.interact(snapped, id, sense))
        } else {
            let (r, resp) = ui.allocate_exact_size(Vec2::splat(diam), sense);
            (subpixel_snap_rect(r, ppp), resp)
        };

        let alpha = if enabled { 1.0 } else { theme.disabled_opacity() };
        let painter = ui.painter_at(rect);
        let visual_rect = theme.clamp_nib_visual_rect(rect, diam, ppp);
        let center = visual_rect.center();
        let visual_diam = diam.min(visual_rect.width()).min(visual_rect.height());
        let radius = visual_diam * 0.5;

        if enabled && response.has_focus() {
            theme.paint_focus_ring(&painter, Rect::from_center_size(center, Vec2::splat(visual_diam)), radius);
        }

        painter.circle_filled(
            Pos2::new(center.x, center.y + shadow_dy),
            radius,
            theme.drop_shadow().gamma_multiply(alpha),
        );

        let fill = if !enabled {
            theme.control_background().gamma_multiply(alpha)
        } else if response.is_pointer_button_down_on() {
            theme.selected_control_inactive()
        } else if response.hovered() {
            theme.control()
        } else {
            theme.control_background()
        };

        painter.circle(center, radius, fill, Stroke::new(border_w, theme.separator().gamma_multiply(alpha)));
        let text_color = theme.label().gamma_multiply(alpha);
        let (font_id, has_semibold) = theme.semibold_font_id(ui, font_sz);
        let galley = theme.layout_tracked_galley(ui, "?", font_id, text_color, f32::INFINITY);
        let text_pos = Align2::CENTER_CENTER.anchor_size(center, galley.size()).min;
        painter.galley(text_pos, galley.clone(), text_color);
        if !has_semibold {
            let boost = theme.label_metrics().semibold_stroke_boost.points();
            painter.galley(Pos2::new(text_pos.x + boost, text_pos.y), galley, text_color);
        }
        response
    }
}

/// Circular `16x16pt` AppKit Inline Action Badge Button (used for inline `+`, `-`, `x`, or symbol actions).
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacInlineActionButton, MacSymbol, MacTheme};
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// if MacInlineActionButton::from_symbol(MacSymbol::Plus).theme(&theme).show(ui).clicked() {}
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacInlineActionButton<'a> {
    glyph: &'a str,
    fixed_rect: Option<Rect>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacInlineActionButton<'a> {
    /// Creates an inline action badge button displaying `glyph` (such as `"+"`, `"-"`, or `"×"`).
    pub fn new(glyph: &'a str) -> Self {
        Self {
            glyph,
            fixed_rect: None,
            enabled: true,
            theme: None,
        }
    }

    /// Creates an inline action badge button displaying a [`MacSymbol`].
    pub fn from_symbol(symbol: MacSymbol) -> Self {
        Self::new(symbol.as_str())
    }

    /// Pins the inline action button to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Enables or disables interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Returns the circular badge diameter (`16.0pt`).
    pub fn diameter(&self) -> f32 {
        MacTheme::resolve_or_default(self.theme).inline_action_button().diameter.points()
    }

    /// Renders the inline action button into `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacInlineActionButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let im = theme.inline_action_button();

        let enabled = self.enabled && ui.is_enabled();
        let diam = im.diameter.points();
        let border_w = im.border_width.points();
        let icon_sz = im.icon_size.points();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let ppp = ui.ctx().pixels_per_point();
        let (rect, response) = if let Some(fixed) = self.fixed_rect {
            let snapped = subpixel_snap_rect(fixed, ppp);
            let id = ui.next_auto_id().with("mac_inline_action_fixed");
            ui.skip_ahead_auto_ids(1);
            (snapped, ui.interact(snapped, id, sense))
        } else {
            let (r, resp) = ui.allocate_exact_size(Vec2::splat(diam), sense);
            (subpixel_snap_rect(r, ppp), resp)
        };

        let alpha = if enabled { 1.0 } else { theme.disabled_opacity() };
        let painter = ui.painter_at(rect);
        let visual_rect = theme.clamp_nib_visual_rect(rect, diam, ppp);
        let center = visual_rect.center();
        let visual_diam = diam.min(visual_rect.width()).min(visual_rect.height());
        let visual_radius = visual_diam * 0.5;

        if enabled && response.has_focus() {
            theme.paint_focus_ring(&painter, Rect::from_center_size(center, Vec2::splat(visual_diam)), visual_radius);
        }

        let fill = if !enabled {
            theme.control().gamma_multiply(alpha)
        } else if response.is_pointer_button_down_on() {
            theme.selected_control_inactive()
        } else if response.hovered() {
            theme.control_background()
        } else {
            theme.control()
        };

        painter.circle(center, visual_radius, fill, Stroke::new(border_w, theme.separator().gamma_multiply(alpha)));
        let surface_bg = theme.active_surface_at(ui, center);
        let fg = if !enabled {
            theme.tertiary_label_on(surface_bg)
        } else if response.hovered() {
            theme.label_on(surface_bg)
        } else {
            theme.secondary_label_on(surface_bg)
        };
        let galley = theme.layout_tracked_galley(ui, self.glyph, FontId::proportional(icon_sz), fg, f32::INFINITY);
        let pos = Align2::CENTER_CENTER.anchor_size(center, galley.size()).min;
        painter.galley(pos, galley, fg);
        response
    }
}
