//! macOS AppKit Dynamic Theming System.
//!
//! Grounded in Apple AppKit `NSAppearance` semantics and W3C DTCG design tokens.
//! Guarantees zero hardcoded constants by deriving all geometry metrics,
//! typography bounds, and semantic colors dynamically from [`MacDesignTokens`].

use crate::tokens::{MacAppearanceColors, MacDesignTokens};
use egui::{Color32, Id};
use std::sync::{Arc, OnceLock};

pub mod accessors;
pub mod contrast;
pub use contrast::{
    contrast_ratio, relative_luminance, srgb_channel_to_linear, MacBackgroundStyle,
};

static DEFAULT_TOKENS: OnceLock<Arc<MacDesignTokens>> = OnceLock::new();

fn default_cached_tokens() -> Arc<MacDesignTokens> {
    DEFAULT_TOKENS
        .get_or_init(|| Arc::new(MacDesignTokens::load_default()))
        .clone()
}

/// macOS system appearance mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacAppearance {
    /// Authentic Apple macOS light appearance (`NSAppearanceNameAqua`).
    #[default]
    Aqua,
    /// Authentic Apple macOS dark appearance (`NSAppearanceNameDarkAqua`).
    DarkAqua,
}

/// Window caption controls presentation style (Strategy 2: Token-Driven / Configurable Presentation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowControlsStyle {
    /// Authentic Apple macOS traffic lights on the leading (left) edge.
    #[default]
    AppleTrafficLights,
    /// Authentic Microsoft Windows 11 caption buttons on the trailing (right) edge.
    WindowsCaptionButtons,
    /// Native host OS window manager decorations (decorations provided by OS).
    NativeHost,
}

/// Dynamic theme resolving AppKit semantic styling from [`MacDesignTokens`].
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacAppearance, MacTheme, WindowControlsStyle};
///
/// let theme = MacTheme::new(MacAppearance::DarkAqua)
///     .with_window_controls_style(WindowControlsStyle::AppleTrafficLights);
///
/// let bg = theme.window_background();
/// let text_color = theme.label();
/// let corner_radius = theme.button().regular.corner_radius.points();
/// assert_eq!(corner_radius, 6.0);
/// ```
#[derive(Debug, Clone)]
pub struct MacTheme {
    /// Active macOS appearance mode ([`MacAppearance::Aqua`] or [`MacAppearance::DarkAqua`]).
    pub appearance: MacAppearance,
    /// Window caption controls presentation style.
    pub window_controls_style: WindowControlsStyle,
    tokens: Arc<MacDesignTokens>,
    custom_accent: Option<Color32>,
}

impl Default for MacTheme {
    fn default() -> Self {
        Self::new(MacAppearance::Aqua)
    }
}

impl MacTheme {
    /// Creates a new `MacTheme` for the given appearance using the embedded default token catalog.
    pub fn new(appearance: MacAppearance) -> Self {
        Self {
            appearance,
            window_controls_style: WindowControlsStyle::AppleTrafficLights,
            tokens: default_cached_tokens(),
            custom_accent: None,
        }
    }

    /// Retrieves the active `MacTheme` stored in `egui::Context` by [`MacTheme::apply_to_egui`],
    /// falling back to [`MacTheme::default`] (`Aqua`) if none was registered.
    pub fn from_ctx(ctx: &egui::Context) -> Self {
        ctx.data(|d| d.get_temp::<MacTheme>(Id::NULL))
            .unwrap_or_default()
    }

    /// Sets the window controls presentation style (Strategy 2).
    pub fn with_window_controls_style(mut self, style: WindowControlsStyle) -> Self {
        self.window_controls_style = style;
        self
    }

    /// Returns the active window controls presentation style.
    pub fn window_controls_style(&self) -> WindowControlsStyle {
        self.window_controls_style
    }

    /// Sets a dynamic system accent color override (matching macOS System Settings Appearance Accent).
    pub fn with_accent(mut self, accent: Color32) -> Self {
        self.custom_accent = Some(accent);
        self
    }

    /// Creates a `MacTheme` with custom loaded tokens.
    pub fn with_tokens(appearance: MacAppearance, tokens: MacDesignTokens) -> Self {
        Self {
            appearance,
            window_controls_style: WindowControlsStyle::AppleTrafficLights,
            tokens: Arc::new(tokens),
            custom_accent: None,
        }
    }

    /// Creates a `MacTheme` with custom loaded tokens wrapped in an `Arc`.
    pub fn with_arc_tokens(appearance: MacAppearance, tokens: Arc<MacDesignTokens>) -> Self {
        Self {
            appearance,
            window_controls_style: WindowControlsStyle::AppleTrafficLights,
            tokens,
            custom_accent: None,
        }
    }

    /// Returns the underlying design token catalog.
    pub fn tokens(&self) -> &MacDesignTokens {
        &self.tokens
    }

    /// Active appearance color palette.
    pub fn colors(&self) -> &MacAppearanceColors {
        match self.appearance {
            MacAppearance::Aqua => &self.tokens.colors().aqua,
            MacAppearance::DarkAqua => &self.tokens.colors().dark_aqua,
        }
    }

    /// Resolves `windowBackgroundColor` (NSColor) for the active appearance.
    pub fn window_background(&self) -> Color32 {
        self.colors().window_background.to_color32()
    }

    /// Resolves `controlBackgroundColor` (NSColor) for the active appearance.
    pub fn control_background(&self) -> Color32 {
        self.colors().control_background.to_color32()
    }

    /// Resolves `controlColor` (NSColor) for the active appearance.
    pub fn control(&self) -> Color32 {
        self.colors().control.to_color32()
    }

    /// Resolves `controlAccentColor` (NSColor) or custom accent override.
    pub fn control_accent(&self) -> Color32 {
        self.custom_accent
            .unwrap_or_else(|| self.colors().control_accent.to_color32())
    }

    /// Resolves `selectedControlColor` (NSColor) or custom accent override.
    pub fn selected_control(&self) -> Color32 {
        self.custom_accent
            .unwrap_or_else(|| self.colors().selected_control.to_color32())
    }

    /// Resolves `selectedControlColor` (NSColor) when the window or control is inactive/unfocused.
    pub fn selected_control_inactive(&self) -> Color32 {
        self.colors().selected_control_inactive.to_color32()
    }

    /// Resolves `separatorColor` (NSColor) for hairlines and borders.
    pub fn separator(&self) -> Color32 {
        self.colors().separator.to_color32()
    }

    /// Resolves `placeholderTextColor` (NSColor) for empty text fields and search inputs.
    pub fn placeholder_text(&self) -> Color32 {
        self.colors().placeholder_text.to_color32()
    }

    /// Resolves `keyboardFocusIndicatorColor` (NSColor) for focus rings.
    pub fn keyboard_focus_indicator(&self) -> Color32 {
        self.colors().keyboard_focus_indicator.to_color32()
    }

    /// Resolves primary `labelColor` (NSColor).
    pub fn label(&self) -> Color32 {
        self.colors().label.to_color32()
    }

    /// Resolves `secondaryLabelColor` (NSColor) for captions, subtitles, and section headers.
    pub fn secondary_label(&self) -> Color32 {
        self.colors().secondary_label.to_color32()
    }

    /// Resolves Apple system green semantic color (`systemGreenColor`).
    pub fn system_green(&self) -> Color32 {
        self.colors().system_green.to_color32()
    }

    /// Resolves Apple system yellow/amber semantic color (`systemYellowColor`).
    pub fn system_yellow(&self) -> Color32 {
        self.colors().system_yellow.to_color32()
    }

    /// Resolves Apple system orange semantic color (`systemOrangeColor`).
    pub fn system_orange(&self) -> Color32 {
        self.colors().system_orange.to_color32()
    }

    /// Resolves Apple system red semantic color (`systemRedColor`).
    pub fn system_red(&self) -> Color32 {
        self.colors().system_red.to_color32()
    }

    /// Resolves even table row background color (`alternatingContentBackgroundColors[0]`).
    pub fn alternating_row_even(&self) -> Color32 {
        self.colors().alternating_row_even.to_color32()
    }

    /// Resolves odd table row background color (`alternatingContentBackgroundColors[1]`).
    pub fn alternating_row_odd(&self) -> Color32 {
        self.colors().alternating_row_odd.to_color32()
    }

    /// Resolves alternating content background color for the given row index (`row_index % 2`).
    pub fn alternating_content_background(&self, row_index: usize) -> Color32 {
        if row_index.is_multiple_of(2) {
            self.alternating_row_even()
        } else {
            self.alternating_row_odd()
        }
    }

    /// Resolves QuickTime / AVKit trim selection border and handle color.
    pub fn scrubber_selection_border(&self) -> Color32 {
        self.colors().scrubber_selection_border.to_color32()
    }

    /// Resolves dimming overlay color for timeline regions outside active trim bounds.
    pub fn scrubber_outside_dim(&self) -> Color32 {
        self.colors().scrubber_outside_dim.to_color32()
    }

    /// Resolves waveform amplitude bar color inside trim scrubber track.
    pub fn scrubber_waveform(&self) -> Color32 {
        self.colors().scrubber_waveform.to_color32()
    }

    /// Resolves playhead cursor line and timecode badge accent color.
    pub fn scrubber_playhead(&self) -> Color32 {
        self.colors().scrubber_playhead.to_color32()
    }

    /// Resolves high-contrast white text color used on accent fills.
    pub fn on_accent_text(&self) -> Color32 {
        self.colors().on_accent_text.to_color32()
    }

    /// Resolves white thumb knob fill color for switches and sliders.
    pub fn knob_fill(&self) -> Color32 {
        self.colors().knob_fill.to_color32()
    }

    /// Resolves subtle ambient drop shadow color for controls and floating surfaces.
    pub fn drop_shadow(&self) -> Color32 {
        self.colors().drop_shadow.to_color32()
    }

    /// Resolves modal backdrop dimming overlay color.
    pub fn backdrop_dim(&self) -> Color32 {
        self.colors().backdrop_dim.to_color32()
    }

    /// Resolves translucent dark HUD palette background color (`SPEC-006 §2.2`).
    pub fn hud_background(&self) -> Color32 {
        self.colors().hud_background.to_color32()
    }

    /// Resolves dark HUD palette hairline border color.
    pub fn hud_border(&self) -> Color32 {
        self.colors().hud_border.to_color32()
    }

    /// Resolves dark HUD palette primary text color.
    pub fn hud_text(&self) -> Color32 {
        self.colors().hud_text.to_color32()
    }

    /// Resolves dark HUD palette secondary label color.
    pub fn hud_secondary_text(&self) -> Color32 {
        self.colors().hud_secondary_text.to_color32()
    }

    /// Resolves macOS window close traffic light fill and border colors `(fill, border)`.
    pub fn traffic_light_close(&self) -> (Color32, Color32) {
        (
            self.colors().traffic_light_close.to_color32(),
            self.colors().traffic_light_close_border.to_color32(),
        )
    }

    /// Resolves macOS window minimize traffic light fill and border colors `(fill, border)`.
    pub fn traffic_light_minimize(&self) -> (Color32, Color32) {
        (
            self.colors().traffic_light_minimize.to_color32(),
            self.colors().traffic_light_minimize_border.to_color32(),
        )
    }

    /// Resolves macOS window zoom/fullscreen traffic light fill and border colors `(fill, border)`.
    pub fn traffic_light_zoom(&self) -> (Color32, Color32) {
        (
            self.colors().traffic_light_zoom.to_color32(),
            self.colors().traffic_light_zoom_border.to_color32(),
        )
    }

    /// Interior glyph color for hovered macOS traffic light buttons.
    pub fn traffic_light_glyph(&self) -> Color32 {
        self.colors().traffic_light_glyph.to_color32()
    }

    /// Background color for Windows 11 close caption button on hover (`#E81123`).
    pub fn windows_caption_close_hover_bg(&self) -> Color32 {
        self.colors().caption_close_hover.to_color32()
    }

    /// Background color for Windows 11 close caption button when pressed (`#C42B1C`).
    pub fn windows_caption_close_pressed_bg(&self) -> Color32 {
        self.colors().caption_close_pressed.to_color32()
    }

    /// Background color for Windows 11 minimize/maximize caption buttons on hover.
    pub fn windows_caption_hover_bg(&self) -> Color32 {
        self.colors().caption_hover.to_color32()
    }

    /// Background color for Windows 11 minimize/maximize caption buttons when pressed.
    pub fn windows_caption_pressed_bg(&self) -> Color32 {
        self.colors().caption_pressed.to_color32()
    }

    /// Overlay scrollbar thumb color scaled by fade `alpha_factor` in `[0.0, 1.0]`.
    pub fn scrollbar_thumb_color(&self, active: bool, alpha_factor: f32) -> Color32 {
        let base = if active {
            self.colors().scrollbar_thumb_active.to_color32()
        } else {
            self.colors().scrollbar_thumb.to_color32()
        };
        base.gamma_multiply(alpha_factor.clamp(0.0, 1.0))
    }

    /// Synchronizes `egui::Context` theme, visuals, and stored `MacTheme` with the macOS appearance.
    pub fn apply_to_egui(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| d.insert_temp(Id::NULL, self.clone()));

        let egui_theme = match self.appearance {
            MacAppearance::Aqua => egui::Theme::Light,
            MacAppearance::DarkAqua => egui::Theme::Dark,
        };
        ctx.set_theme(egui_theme);

        let mut visuals = match self.appearance {
            MacAppearance::Aqua => egui::Visuals::light(),
            MacAppearance::DarkAqua => egui::Visuals::dark(),
        };

        let border_w = self.button().regular.border_width.points();
        let label_color = self.label();
        visuals.override_text_color = Some(label_color);
        visuals.widgets.noninteractive.fg_stroke.color = label_color;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(border_w, self.separator());
        visuals.widgets.inactive.fg_stroke.color = label_color;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(border_w, self.separator());
        visuals.panel_fill = self.window_background();
        visuals.window_fill = self.window_background();
        visuals.window_stroke = egui::Stroke::new(border_w, self.separator());

        let small_pt = self.text_field().small.font_size.points();
        let body_pt = self.outline_view().font_size.points();
        let heading_pt = self.text_field().regular.font_size.points();

        ctx.all_styles_mut(|style| {
            use egui::{FontFamily, FontId, TextStyle};
            style.text_styles = [
                (
                    TextStyle::Small,
                    FontId::new(small_pt, FontFamily::Proportional),
                ),
                (
                    TextStyle::Body,
                    FontId::new(body_pt, FontFamily::Proportional),
                ),
                (
                    TextStyle::Button,
                    FontId::new(body_pt, FontFamily::Proportional),
                ),
                (
                    TextStyle::Heading,
                    FontId::new(heading_pt, FontFamily::Proportional),
                ),
                (
                    TextStyle::Monospace,
                    FontId::new(body_pt, FontFamily::Monospace),
                ),
            ]
            .into();
        });

        ctx.set_visuals_of(egui_theme, visuals.clone());
        ctx.set_visuals(visuals);
    }
}
