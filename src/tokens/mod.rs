//! Typed W3C DTCG-compliant Design Token models for `egui-macview`.
//!
//! Grounded in Apple AppKit, macOS Human Interface Guidelines,
//! Apple Typography specifications, and CoreAnimation spring physics.

pub mod color;
pub mod dtcg;
pub mod metrics;
pub mod motion;
pub mod typography;

pub use color::{MacAppearanceColors, MacColorTokens};
pub use dtcg::{ColorValue, DimensionValue};
pub use metrics::{
    BoxViewMetrics, ButtonMetrics, ButtonSizeMetrics, CollectionViewMetrics, ColorWellMetrics,
    ComboBoxMetrics, ContextMenuMetrics, DatePickerMetrics, DisclosureGroupMetrics,
    FocusRingMetrics, FooterMetrics, LayoutMetrics, LevelIndicatorMetrics, MacControlMetrics,
    MacMetricsTokens, MenuBarMetrics, NavigationMetrics, OutlineViewMetrics, PaletteMetrics,
    PathControlMetrics, PlayerViewMetrics, PopUpButtonMetrics, PopoverMetrics,
    ProgressIndicatorMetrics, ProgressSizeMetrics, ScrollViewMetrics, SearchFieldMetrics,
    SegmentedMetrics, SegmentedSizeMetrics, SliderMetrics, SplitViewMetrics, StackViewMetrics,
    StepperMetrics, TableViewMetrics, TabViewMetrics, TextFieldMetrics, TextFieldSizeMetrics,
    ThumbnailCellMetrics, ToastMetrics, ToggleMetrics, TokenFieldMetrics, ToolbarMetrics,
    TrimScrubberMetrics, CANVAS_FOOTER_HEIGHT, COMMAND_PALETTE_HEIGHT, COMMAND_PALETTE_WIDTH,
    DEFAULT_TOAST_DURATION,
};
pub use motion::{MacMotionTokens, MotionPresetToken};
pub use typography::{
    sf_pro_tracking, CubicCoefficients, CubicFit, MacTypographyTokens, PolynomialSpec001,
    QuadraticCoefficients, QuadraticFit, TrackingPoint, TrackingTableEntry,
};

use serde::{Deserialize, Serialize};

/// Inner container grouping the token subcategories.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacViewTokens {
    /// Geometry and dimension tokens.
    pub metrics: MacMetricsTokens,
    /// Semantic color palettes for Aqua and DarkAqua.
    pub color: MacColorTokens,
    /// SF Pro typography and tracking tokens.
    pub typography: MacTypographyTokens,
    /// Spring physics and animation preset tokens.
    pub motion: MacMotionTokens,
}

/// Root token catalog deserialized from `tokens/design_tokens.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacDesignTokens {
    /// Token catalog schema name.
    #[serde(rename = "$name", default)]
    pub name: String,
    /// Token catalog schema version string.
    #[serde(rename = "$version", default)]
    pub version: String,
    /// Catalog description.
    #[serde(rename = "$description", default)]
    pub description: String,
    /// Root namespace holding all token subcategories.
    pub macview: MacViewTokens,
}

impl MacDesignTokens {
    /// Loads default design tokens from embedded `tokens/design_tokens.json`.
    ///
    /// Executes parsing within a dedicated stack thread (4MB) to guarantee immunity
    /// against stack overflows on platforms with restricted default main stacks.
    pub fn load_default() -> Self {
        std::thread::Builder::new()
            .name("mac_token_loader".into())
            .stack_size(4 * 1024 * 1024)
            .spawn(|| {
                Self::from_json(include_str!("../../tokens/design_tokens.json"))
                    .expect("Failed to parse embedded tokens/design_tokens.json")
            })
            .expect("Failed to spawn token loader thread")
            .join()
            .expect("Token loader panicked")
    }

    /// Deserializes `MacDesignTokens` from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Accesses control metrics tokens.
    #[inline]
    pub fn control_metrics(&self) -> &MacControlMetrics {
        &self.macview.metrics.control
    }

    /// Accesses toolbar metrics tokens.
    #[inline]
    pub fn toolbar_metrics(&self) -> &ToolbarMetrics {
        &self.macview.metrics.toolbar
    }

    /// Accesses popover and panel metrics tokens.
    #[inline]
    pub fn popover_metrics(&self) -> &PopoverMetrics {
        &self.macview.metrics.popover
    }

    /// Accesses split view metrics tokens.
    #[inline]
    pub fn split_view_metrics(&self) -> &SplitViewMetrics {
        &self.macview.metrics.split_view
    }

    /// Accesses layout container metrics tokens.
    #[inline]
    pub fn layout_metrics(&self) -> &LayoutMetrics {
        &self.macview.metrics.layout
    }

    /// Accesses scroll view metrics tokens.
    #[inline]
    pub fn scroll_view_metrics(&self) -> &ScrollViewMetrics {
        &self.macview.metrics.layout.scroll_view
    }

    /// Accesses stack view metrics tokens.
    #[inline]
    pub fn stack_view_metrics(&self) -> &StackViewMetrics {
        &self.macview.metrics.layout.stack_view
    }

    /// Accesses outline view metrics tokens.
    #[inline]
    pub fn outline_view_metrics(&self) -> &OutlineViewMetrics {
        &self.macview.metrics.control.outline_view
    }

    /// Accesses slider metrics tokens.
    #[inline]
    pub fn slider_metrics(&self) -> &SliderMetrics {
        &self.macview.metrics.control.slider
    }

    /// Accesses thumbnail metrics tokens.
    #[inline]
    pub fn thumbnail_metrics(&self) -> &ThumbnailCellMetrics {
        &self.macview.metrics.control.thumbnail
    }

    /// Accesses popup button metrics tokens.
    #[inline]
    pub fn popup_button_metrics(&self) -> &PopUpButtonMetrics {
        &self.macview.metrics.control.popup_button
    }

    /// Accesses search field metrics tokens.
    #[inline]
    pub fn search_field_metrics(&self) -> &SearchFieldMetrics {
        &self.macview.metrics.control.search_field
    }

    /// Accesses stepper metrics tokens.
    #[inline]
    pub fn stepper_metrics(&self) -> &StepperMetrics {
        &self.macview.metrics.control.stepper
    }

    /// Accesses color well metrics tokens.
    #[inline]
    pub fn color_well_metrics(&self) -> &ColorWellMetrics {
        &self.macview.metrics.control.color_well
    }

    /// Accesses toggle switch metrics tokens.
    #[inline]
    pub fn toggle_metrics(&self) -> &ToggleMetrics {
        &self.macview.metrics.control.toggle
    }

    /// Accesses navigation and feedback metrics tokens.
    #[inline]
    pub fn navigation_metrics(&self) -> &NavigationMetrics {
        &self.macview.metrics.navigation
    }

    /// Accesses menu bar metrics tokens.
    #[inline]
    pub fn menu_bar_metrics(&self) -> &MenuBarMetrics {
        &self.macview.metrics.navigation.menu_bar
    }

    /// Accesses canvas footer metrics tokens.
    #[inline]
    pub fn footer_metrics(&self) -> &FooterMetrics {
        &self.macview.metrics.navigation.footer
    }

    /// Accesses command palette metrics tokens.
    #[inline]
    pub fn palette_metrics(&self) -> &PaletteMetrics {
        &self.macview.metrics.navigation.palette
    }

    /// Accesses context menu metrics tokens.
    #[inline]
    pub fn context_menu_metrics(&self) -> &ContextMenuMetrics {
        &self.macview.metrics.navigation.context_menu
    }

    /// Accesses toast notification metrics tokens.
    #[inline]
    pub fn toast_metrics(&self) -> &ToastMetrics {
        &self.macview.metrics.navigation.toast
    }

    /// Accesses progress indicator metrics tokens.
    #[inline]
    pub fn progress_indicator_metrics(&self) -> &ProgressIndicatorMetrics {
        &self.macview.metrics.control.progress_indicator
    }

    /// Accesses table view metrics tokens.
    #[inline]
    pub fn table_view_metrics(&self) -> &TableViewMetrics {
        &self.macview.metrics.control.table_view
    }

    /// Accesses token field metrics tokens.
    #[inline]
    pub fn token_field_metrics(&self) -> &TokenFieldMetrics {
        &self.macview.metrics.control.token_field
    }

    /// Accesses level indicator metrics tokens.
    #[inline]
    pub fn level_indicator_metrics(&self) -> &LevelIndicatorMetrics {
        &self.macview.metrics.control.level_indicator
    }

    /// Accesses trim scrubber metrics tokens.
    #[inline]
    pub fn trim_scrubber_metrics(&self) -> &TrimScrubberMetrics {
        &self.macview.metrics.control.trim_scrubber
    }

    /// Accesses path control metrics tokens.
    #[inline]
    pub fn path_control_metrics(&self) -> &PathControlMetrics {
        &self.macview.metrics.control.path_control
    }

    /// Accesses disclosure group metrics tokens.
    #[inline]
    pub fn disclosure_group_metrics(&self) -> &DisclosureGroupMetrics {
        &self.macview.metrics.control.disclosure_group
    }

    /// Accesses combo box metrics tokens.
    #[inline]
    pub fn combo_box_metrics(&self) -> &ComboBoxMetrics {
        &self.macview.metrics.control.combo_box
    }

    /// Accesses tab view metrics tokens.
    #[inline]
    pub fn tab_view_metrics(&self) -> &TabViewMetrics {
        &self.macview.metrics.control.tab_view
    }

    /// Accesses date picker metrics tokens.
    #[inline]
    pub fn date_picker_metrics(&self) -> &DatePickerMetrics {
        &self.macview.metrics.control.date_picker
    }

    /// Accesses collection view metrics tokens.
    #[inline]
    pub fn collection_view_metrics(&self) -> &CollectionViewMetrics {
        &self.macview.metrics.control.collection_view
    }

    /// Accesses player view metrics tokens.
    #[inline]
    pub fn player_view_metrics(&self) -> &PlayerViewMetrics {
        &self.macview.metrics.control.player_view
    }

    /// Accesses box view metrics tokens.
    #[inline]
    pub fn box_view_metrics(&self) -> &BoxViewMetrics {
        &self.macview.metrics.control.box_view
    }

    /// Accesses semantic color tokens.
    #[inline]
    pub fn colors(&self) -> &MacColorTokens {
        &self.macview.color
    }

    /// Accesses typography and optical tracking tokens.
    #[inline]
    pub fn typography(&self) -> &MacTypographyTokens {
        &self.macview.typography
    }

    /// Accesses CoreAnimation motion preset tokens.
    #[inline]
    pub fn motion(&self) -> &MacMotionTokens {
        &self.macview.motion
    }
}
