//! Metric and semantic token accessors for `MacTheme`.

use crate::theme::{MacBackgroundStyle, MacTheme};
use crate::tokens::{
    ButtonMetrics, ColorWellMetrics, ContextMenuMetrics, FocusRingMetrics, FooterMetrics,
    LayoutMetrics, MacControlMetrics, MacMotionTokens, MacTypographyTokens, MenuBarMetrics,
    NavigationMetrics, OutlineViewMetrics, PaletteMetrics, PopUpButtonMetrics, PopoverMetrics,
    ScrollViewMetrics, SearchFieldMetrics, SegmentedMetrics, SliderMetrics, SplitViewMetrics,
    StackViewMetrics, StepperMetrics, TextFieldMetrics, ThumbnailCellMetrics, ToastMetrics,
    ToggleMetrics, ToolbarMetrics,
};
use egui::Color32;

impl MacTheme {
    /// Resolves top-level control metrics collection.
    pub fn control_metrics(&self) -> &MacControlMetrics {
        self.tokens().control_metrics()
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
        self.tokens().popover_metrics()
    }

    /// Resolves window toolbar geometry metrics.
    pub fn toolbar_metrics(&self) -> &ToolbarMetrics {
        self.tokens().toolbar_metrics()
    }

    /// Resolves slider control geometry metrics.
    pub fn slider(&self) -> &SliderMetrics {
        self.tokens().slider_metrics()
    }

    /// Resolves thumbnail cell geometry metrics.
    pub fn thumbnail(&self) -> &ThumbnailCellMetrics {
        self.tokens().thumbnail_metrics()
    }

    /// Resolves split view divider and handle geometry metrics.
    pub fn split_view(&self) -> &SplitViewMetrics {
        self.tokens().split_view_metrics()
    }

    /// Resolves layout container geometry metrics.
    pub fn layout_metrics(&self) -> &LayoutMetrics {
        self.tokens().layout_metrics()
    }

    /// Resolves scroll view and overlay scrollbar geometry metrics.
    pub fn scroll_view(&self) -> &ScrollViewMetrics {
        self.tokens().scroll_view_metrics()
    }

    /// Resolves stack view linear layout geometry metrics.
    pub fn stack_view(&self) -> &StackViewMetrics {
        self.tokens().stack_view_metrics()
    }

    /// Resolves outline view hierarchical table geometry metrics.
    pub fn outline_view(&self) -> &OutlineViewMetrics {
        self.tokens().outline_view_metrics()
    }

    /// Resolves pop-up button geometry metrics.
    pub fn popup_button(&self) -> &PopUpButtonMetrics {
        self.tokens().popup_button_metrics()
    }

    /// Resolves search field geometry metrics.
    pub fn search_field(&self) -> &SearchFieldMetrics {
        self.tokens().search_field_metrics()
    }

    /// Resolves numeric stepper control geometry metrics.
    pub fn stepper(&self) -> &StepperMetrics {
        self.tokens().stepper_metrics()
    }

    /// Resolves color well swatch control geometry metrics.
    pub fn color_well(&self) -> &ColorWellMetrics {
        self.tokens().color_well_metrics()
    }

    /// Resolves toggle switch control geometry metrics.
    pub fn toggle(&self) -> &ToggleMetrics {
        self.tokens().toggle_metrics()
    }

    /// Resolves navigation and feedback metrics collection.
    pub fn navigation_metrics(&self) -> &NavigationMetrics {
        self.tokens().navigation_metrics()
    }

    /// Resolves menu bar and dropdown geometry metrics.
    pub fn menu_bar_metrics(&self) -> &MenuBarMetrics {
        self.tokens().menu_bar_metrics()
    }

    /// Resolves canvas footer geometry metrics.
    pub fn footer_metrics(&self) -> &FooterMetrics {
        self.tokens().footer_metrics()
    }

    /// Resolves command palette modal geometry metrics.
    pub fn palette_metrics(&self) -> &PaletteMetrics {
        self.tokens().palette_metrics()
    }

    /// Resolves context menu geometry metrics.
    pub fn context_menu_metrics(&self) -> &ContextMenuMetrics {
        self.tokens().context_menu_metrics()
    }

    /// Resolves toast notification geometry metrics.
    pub fn toast_metrics(&self) -> &ToastMetrics {
        self.tokens().toast_metrics()
    }

    /// Resolves CoreAnimation motion preset tokens.
    pub fn motion(&self) -> &MacMotionTokens {
        self.tokens().motion()
    }

    /// Resolves typography and SF Pro tracking tokens.
    pub fn typography(&self) -> &MacTypographyTokens {
        self.tokens().typography()
    }

    /// Resolves opacity multiplier for disabled controls (`0.45`).
    pub fn disabled_opacity(&self) -> f32 {
        self.control_metrics().disabled_opacity.points()
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
