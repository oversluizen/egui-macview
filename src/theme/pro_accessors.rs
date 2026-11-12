//! Pro-Inspector Metric and Semantic Tag Color Accessors on [`MacTheme`].

use super::MacTheme;
use crate::tokens::{
    MacTagLabelColors, OriginMatrixMetrics, TableActionBarMetrics, TagColorPickerMetrics,
};
use egui::Color32;

impl MacTheme {
    /// Returns `3x3` 9-point transform origin / anchor pin grid tokens (`metrics.control.origin_matrix`).
    pub fn origin_matrix(&self) -> &OriginMatrixMetrics {
        self.tokens.origin_matrix_metrics()
    }

    /// Returns macOS Finder / AppKit tag color swatch picker tokens (`metrics.control.tag_color_picker`).
    pub fn tag_color_picker(&self) -> &TagColorPickerMetrics {
        self.tokens.tag_color_picker_metrics()
    }

    /// Returns `NSBezelStyleSmallSquare` `[ + | - ]` table action bar tokens (`metrics.control.table_action_bar`).
    pub fn table_action_bar(&self) -> &TableActionBarMetrics {
        self.tokens.table_action_bar_metrics()
    }

    /// Returns the semantic tag label color token group (`color.tag_labels`).
    pub fn tag_labels(&self) -> &MacTagLabelColors {
        &self.tokens.colors().tag_labels
    }

    /// Returns the 11 standard semantic Finder / AppKit tag swatch colors (`Red` through `Gray`).
    pub fn tag_label_palette(&self) -> [Color32; 11] {
        self.tag_labels().palette_11()
    }

    /// Returns the subtle perimeter border color around each tag color swatch dot (`rgba(0, 0, 0, 0.18)`).
    pub fn tag_swatch_border(&self) -> Color32 {
        self.tag_labels().swatch_border.to_color32()
    }

    /// Returns the neutral `NSSlider` track background fill (`slider_track_bg`).
    pub fn slider_track_bg(&self) -> Color32 {
        self.colors().slider_track_bg.to_color32()
    }

    /// Returns the neutral `NSSlider` track border color (`slider_track_border`).
    pub fn slider_track_border(&self) -> Color32 {
        self.colors().slider_track_border.to_color32()
    }

    /// Returns the AppKit `NSSlider` tick mark stroke color (`#B8B8B8` in Light, `#666666` in Dark).
    pub fn slider_tick_mark_color(&self) -> Color32 {
        self.colors().slider_tick_mark.to_color32()
    }

    /// Returns the AppKit `NSSlider` thumb fill color (`slider_thumb_bg`).
    pub fn slider_thumb_bg(&self) -> Color32 {
        self.colors().slider_thumb_bg.to_color32()
    }

    /// Returns the AppKit `NSSlider` thumb border color (`slider_thumb_border`).
    pub fn slider_thumb_border_color(&self) -> Color32 {
        self.colors().slider_thumb_border.to_color32()
    }

    /// Returns the `NSBezelStyleSmallSquare` border and divider color (`table_action_bar_border`).
    pub fn table_action_bar_border(&self) -> Color32 {
        self.colors().box_border.to_color32()
    }

    /// Returns the active segment fill for `NSSegmentedControl` `.smallSquare` (`segmented_small_square_active_bg`).
    pub fn segmented_small_square_active_bg(&self) -> Color32 {
        self.colors().segmented_small_square_active_bg.to_color32()
    }

    /// Returns the inactive segment fill for `NSSegmentedControl` `.smallSquare` (`segmented_small_square_inactive_bg`).
    pub fn segmented_small_square_inactive_bg(&self) -> Color32 {
        self.colors().segmented_small_square_inactive_bg.to_color32()
    }

    /// Returns the AppKit `NSPopover` container surface background color (`popover_background`).
    pub fn popover_background(&self) -> Color32 {
        self.colors().popover_background.to_color32()
    }

    /// Returns the AppKit `NSPopover` container subtle perimeter border color (`popover_border`).
    pub fn popover_border(&self) -> Color32 {
        self.colors().popover_border.to_color32()
    }
}
