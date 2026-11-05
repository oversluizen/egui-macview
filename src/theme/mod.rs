//! macOS AppKit Dynamic Theming System.
//!
//! Grounded in Apple AppKit NSAppearance semantics and W3C DTCG design tokens.
//! Guarantees zero hardcoded constants by deriving all geometry metrics,
//! typography bounds, and semantic colors dynamically from `MacDesignTokens`.

use egui::Color32;
use crate::tokens::{
    ButtonMetrics, FocusRingMetrics, MacAppearanceColors, MacControlMetrics, MacDesignTokens,
    PopoverMetrics, SegmentedMetrics, TextFieldMetrics, ToolbarMetrics,
};

/// macOS system appearance mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacAppearance {
    #[default]
    Aqua,
    DarkAqua,
}

/// Dynamic theme resolving AppKit semantic styling from `MacDesignTokens`.
#[derive(Debug, Clone)]
pub struct MacTheme {
    pub appearance: MacAppearance,
    tokens: MacDesignTokens,
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
            tokens: MacDesignTokens::load_default(),
        }
    }

    /// Creates a `MacTheme` with custom loaded tokens.
    pub fn with_tokens(appearance: MacAppearance, tokens: MacDesignTokens) -> Self {
        Self { appearance, tokens }
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

    // Semantic NSColor dynamic resolvers
    pub fn window_background(&self) -> Color32 {
        self.colors().window_background.to_color32()
    }

    pub fn control_background(&self) -> Color32 {
        self.colors().control_background.to_color32()
    }

    pub fn control(&self) -> Color32 {
        self.colors().control.to_color32()
    }

    pub fn control_accent(&self) -> Color32 {
        self.colors().control_accent.to_color32()
    }

    pub fn selected_control(&self) -> Color32 {
        self.colors().selected_control.to_color32()
    }

    pub fn selected_control_inactive(&self) -> Color32 {
        self.colors().selected_control_inactive.to_color32()
    }

    pub fn separator(&self) -> Color32 {
        self.colors().separator.to_color32()
    }

    pub fn placeholder_text(&self) -> Color32 {
        self.colors().placeholder_text.to_color32()
    }

    pub fn keyboard_focus_indicator(&self) -> Color32 {
        self.colors().keyboard_focus_indicator.to_color32()
    }

    // Dynamic metrics resolvers
    pub fn control_metrics(&self) -> &MacControlMetrics {
        self.tokens.control_metrics()
    }

    pub fn focus_ring(&self) -> &FocusRingMetrics {
        &self.control_metrics().focus_ring
    }

    pub fn text_field(&self) -> &TextFieldMetrics {
        &self.control_metrics().text_field
    }

    pub fn button(&self) -> &ButtonMetrics {
        &self.control_metrics().button
    }

    pub fn segmented(&self) -> &SegmentedMetrics {
        &self.control_metrics().segmented
    }

    pub fn popover_metrics(&self) -> &PopoverMetrics {
        self.tokens.popover_metrics()
    }

    pub fn toolbar_metrics(&self) -> &ToolbarMetrics {
        self.tokens.toolbar_metrics()
    }
}
