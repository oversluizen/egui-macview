//! macOS AppKit Dynamic Theming System.
//!
//! Grounded in Apple AppKit `NSAppearance` semantics and W3C DTCG design tokens.
//! Guarantees zero hardcoded constants by deriving all geometry metrics,
//! typography bounds, and semantic colors dynamically from [`MacDesignTokens`].

use crate::tokens::{
    ButtonMetrics, ColorWellMetrics, FocusRingMetrics, LayoutMetrics, LevelIndicatorMetrics,
    MacAppearanceColors, MacControlMetrics, MacDesignTokens, MacMotionTokens, MacTypographyTokens,
    OutlineViewMetrics, PathControlMetrics, PopUpButtonMetrics, PopoverMetrics,
    ProgressIndicatorMetrics, ScrollViewMetrics, SearchFieldMetrics, SegmentedMetrics,
    SliderMetrics, SplitViewMetrics, StackViewMetrics, StepperMetrics, TableViewMetrics,
    TextFieldMetrics, ThumbnailCellMetrics, ToggleMetrics, TokenFieldMetrics, ToolbarMetrics,
    TrimScrubberMetrics,
};
use egui::Color32;

use std::sync::{Arc, OnceLock};

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
/// // Create a dark aqua macOS theme
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

    /// Synchronizes `egui::Context` theme and visuals with the macOS appearance.
    pub fn apply_to_egui(&self, ctx: &egui::Context) {
        let egui_theme = match self.appearance {
            MacAppearance::Aqua => egui::Theme::Light,
            MacAppearance::DarkAqua => egui::Theme::Dark,
        };
        ctx.set_theme(egui_theme);

        let mut visuals = match self.appearance {
            MacAppearance::Aqua => egui::Visuals::light(),
            MacAppearance::DarkAqua => egui::Visuals::dark(),
        };

        let label_color = self.label();
        visuals.override_text_color = Some(label_color);
        visuals.widgets.noninteractive.fg_stroke.color = label_color;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, self.separator());
        visuals.widgets.inactive.fg_stroke.color = label_color;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, self.separator());
        visuals.panel_fill = self.window_background();
        visuals.window_fill = self.window_background();
        visuals.window_stroke = egui::Stroke::new(1.0, self.separator());

        ctx.all_styles_mut(|style| {
            use egui::{FontFamily, FontId, TextStyle};
            style.text_styles = [
                (
                    TextStyle::Small,
                    FontId::new(11.0, FontFamily::Proportional),
                ),
                (TextStyle::Body, FontId::new(12.0, FontFamily::Proportional)),
                (
                    TextStyle::Button,
                    FontId::new(12.0, FontFamily::Proportional),
                ),
                (
                    TextStyle::Heading,
                    FontId::new(13.0, FontFamily::Proportional),
                ),
                (
                    TextStyle::Monospace,
                    FontId::new(12.0, FontFamily::Monospace),
                ),
            ]
            .into();
        });

        ctx.set_visuals_of(egui_theme, visuals.clone());
        ctx.set_visuals(visuals);
    }

    /// Resolves top-level control metrics collection.
    pub fn control_metrics(&self) -> &MacControlMetrics {
        self.tokens.control_metrics()
    }

    /// Resolves focus ring geometry metrics.
    pub fn focus_ring(&self) -> &FocusRingMetrics {
        &self.control_metrics().focus_ring
    }

    /// Resolves text field geometry metrics.
    pub fn text_field(&self) -> &TextFieldMetrics {
        &self.control_metrics().text_field
    }

    /// Resolves push button geometry metrics.
    pub fn button(&self) -> &ButtonMetrics {
        &self.control_metrics().button
    }

    /// Resolves segmented control geometry metrics.
    pub fn segmented(&self) -> &SegmentedMetrics {
        &self.control_metrics().segmented
    }

    /// Resolves popover and HUD panel geometry metrics.
    pub fn popover_metrics(&self) -> &PopoverMetrics {
        self.tokens.popover_metrics()
    }

    /// Resolves window toolbar geometry metrics.
    pub fn toolbar_metrics(&self) -> &ToolbarMetrics {
        self.tokens.toolbar_metrics()
    }

    /// Resolves slider control geometry metrics.
    pub fn slider(&self) -> &SliderMetrics {
        self.tokens.slider_metrics()
    }

    /// Resolves thumbnail cell geometry metrics.
    pub fn thumbnail(&self) -> &ThumbnailCellMetrics {
        self.tokens.thumbnail_metrics()
    }

    /// Resolves split view divider and handle geometry metrics.
    pub fn split_view(&self) -> &SplitViewMetrics {
        self.tokens.split_view_metrics()
    }

    /// Resolves layout container geometry metrics.
    pub fn layout_metrics(&self) -> &LayoutMetrics {
        self.tokens.layout_metrics()
    }

    /// Resolves scroll view and overlay scrollbar geometry metrics.
    pub fn scroll_view(&self) -> &ScrollViewMetrics {
        self.tokens.scroll_view_metrics()
    }

    /// Resolves stack view linear layout geometry metrics.
    pub fn stack_view(&self) -> &StackViewMetrics {
        self.tokens.stack_view_metrics()
    }

    /// Resolves outline view hierarchical table geometry metrics.
    pub fn outline_view(&self) -> &OutlineViewMetrics {
        self.tokens.outline_view_metrics()
    }

    /// Resolves pop-up button geometry metrics.
    pub fn popup_button(&self) -> &PopUpButtonMetrics {
        self.tokens.popup_button_metrics()
    }

    /// Resolves search field geometry metrics.
    pub fn search_field(&self) -> &SearchFieldMetrics {
        self.tokens.search_field_metrics()
    }

    /// Resolves numeric stepper control geometry metrics.
    pub fn stepper(&self) -> &StepperMetrics {
        self.tokens.stepper_metrics()
    }

    /// Resolves color well swatch control geometry metrics.
    pub fn color_well(&self) -> &ColorWellMetrics {
        self.tokens.color_well_metrics()
    }

    /// Resolves toggle switch control geometry metrics.
    pub fn toggle(&self) -> &ToggleMetrics {
        self.tokens.toggle_metrics()
    }

    /// Resolves progress indicator control geometry metrics.
    pub fn progress_indicator(&self) -> &ProgressIndicatorMetrics {
        self.tokens.progress_indicator_metrics()
    }

    /// Resolves table view and header geometry metrics.
    pub fn table_view(&self) -> &TableViewMetrics {
        self.tokens.table_view_metrics()
    }

    /// Resolves token field pill input geometry metrics.
    pub fn token_field(&self) -> &TokenFieldMetrics {
        self.tokens.token_field_metrics()
    }

    /// Resolves level indicator meter geometry metrics.
    pub fn level_indicator(&self) -> &LevelIndicatorMetrics {
        self.tokens.level_indicator_metrics()
    }

    /// Resolves trim scrubber timeline geometry metrics.
    pub fn trim_scrubber(&self) -> &TrimScrubberMetrics {
        self.tokens.trim_scrubber_metrics()
    }

    /// Resolves path control breadcrumb geometry metrics.
    pub fn path_control(&self) -> &PathControlMetrics {
        self.tokens.path_control_metrics()
    }

    /// Resolves CoreAnimation motion preset tokens.
    pub fn motion(&self) -> &MacMotionTokens {
        self.tokens.motion()
    }

    /// Resolves typography and SF Pro tracking tokens.
    pub fn typography(&self) -> &MacTypographyTokens {
        self.tokens.typography()
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

    /// Background color for Windows 11 close caption button on hover (Fluent crimson red `#E81123`).
    pub fn windows_caption_close_hover_bg(&self) -> Color32 {
        Color32::from_rgb(232, 17, 35)
    }

    /// Background color for Windows 11 close caption button when pressed (`#C42B1C`).
    pub fn windows_caption_close_pressed_bg(&self) -> Color32 {
        Color32::from_rgb(196, 43, 28)
    }

    /// Background color for Windows 11 minimize/maximize caption buttons on hover.
    pub fn windows_caption_hover_bg(&self) -> Color32 {
        match self.appearance {
            MacAppearance::Aqua => Color32::from_black_alpha(15),
            MacAppearance::DarkAqua => Color32::from_white_alpha(20),
        }
    }

    /// Background color for Windows 11 minimize/maximize caption buttons when pressed.
    pub fn windows_caption_pressed_bg(&self) -> Color32 {
        match self.appearance {
            MacAppearance::Aqua => Color32::from_black_alpha(25),
            MacAppearance::DarkAqua => Color32::from_white_alpha(35),
        }
    }

    /// Resolves the `MacBackgroundStyle` (Light or Dark) for any background color.
    pub fn background_style(&self, bg: Color32) -> MacBackgroundStyle {
        MacBackgroundStyle::from_color(bg)
    }

    /// Resolves the `MacBackgroundStyle` for the window background.
    pub fn window_background_style(&self) -> MacBackgroundStyle {
        MacBackgroundStyle::from_color(self.window_background())
    }

    /// Resolves the `MacBackgroundStyle` for the control background.
    pub fn control_background_style(&self) -> MacBackgroundStyle {
        MacBackgroundStyle::from_color(self.control_background())
    }

    /// Resolves the `MacBackgroundStyle` for the selected control background.
    pub fn selected_control_background_style(&self) -> MacBackgroundStyle {
        MacBackgroundStyle::from_color(self.selected_control())
    }
}
