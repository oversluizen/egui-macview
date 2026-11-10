//! AppKit Control geometry metrics tokens.

pub mod advanced;
pub mod basic;
pub mod extended;
pub mod selection_and_value;
pub mod studio;

pub use advanced::{
    BoxViewMetrics, CollectionViewMetrics, ComboBoxMetrics, DatePickerMetrics,
    DisclosureGroupMetrics, PlayerViewMetrics, TabViewMetrics,
};
pub use basic::{
    default_border_width, ButtonMetrics, ButtonSizeMetrics, FocusRingMetrics, SegmentedMetrics,
    SegmentedSizeMetrics, TextFieldMetrics, TextFieldSizeMetrics,
};
pub use extended::{
    ColorWellMetrics, OutlineViewMetrics, PopUpButtonMetrics, SearchFieldMetrics, SliderMetrics,
    StepperMetrics, ThumbnailCellMetrics, ToggleMetrics,
};
pub use studio::{
    LevelIndicatorMetrics, PathControlMetrics, ProgressIndicatorMetrics, ProgressSizeMetrics,
    TableViewMetrics, TokenFieldMetrics, TrimScrubberMetrics,
};

use crate::tokens::dtcg::DimensionValue;
use serde::{Deserialize, Serialize};

fn default_disabled_opacity() -> DimensionValue {
    DimensionValue::from_pt(0.45)
}

/// Control metrics collection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacControlMetrics {
    /// Keyboard focus ring metrics.
    pub focus_ring: FocusRingMetrics,
    /// Text input field metrics.
    pub text_field: TextFieldMetrics,
    /// Push button metrics.
    pub button: ButtonMetrics,
    /// Segmented control metrics.
    pub segmented: SegmentedMetrics,
    /// Slider control metrics.
    #[serde(default)]
    pub slider: SliderMetrics,
    /// Thumbnail cell metrics.
    #[serde(default)]
    pub thumbnail: ThumbnailCellMetrics,
    /// Outline tree view metrics.
    #[serde(default)]
    pub outline_view: OutlineViewMetrics,
    /// Pop-up menu button metrics.
    #[serde(default)]
    pub popup_button: PopUpButtonMetrics,
    /// Search field metrics.
    #[serde(default)]
    pub search_field: SearchFieldMetrics,
    /// Numeric stepper metrics.
    #[serde(default)]
    pub stepper: StepperMetrics,
    /// Color well swatch metrics.
    #[serde(default)]
    pub color_well: ColorWellMetrics,
    /// Toggle switch metrics.
    #[serde(default)]
    pub toggle: ToggleMetrics,
    /// Opacity multiplier for disabled controls.
    #[serde(default = "default_disabled_opacity")]
    pub disabled_opacity: DimensionValue,
    /// Progress indicator metrics.
    #[serde(default)]
    pub progress_indicator: ProgressIndicatorMetrics,
    /// Table view and header metrics.
    #[serde(default)]
    pub table_view: TableViewMetrics,
    /// Token field pill input metrics.
    #[serde(default)]
    pub token_field: TokenFieldMetrics,
    /// Level indicator meter metrics.
    #[serde(default)]
    pub level_indicator: LevelIndicatorMetrics,
    /// Trim scrubber timeline metrics.
    #[serde(default)]
    pub trim_scrubber: TrimScrubberMetrics,
    /// Path control breadcrumb metrics.
    #[serde(default)]
    pub path_control: PathControlMetrics,
    /// Disclosure group & button metrics.
    #[serde(default)]
    pub disclosure_group: DisclosureGroupMetrics,
    /// Combo box metrics.
    #[serde(default)]
    pub combo_box: ComboBoxMetrics,
    /// Tab view metrics.
    #[serde(default)]
    pub tab_view: TabViewMetrics,
    /// Date picker metrics.
    #[serde(default)]
    pub date_picker: DatePickerMetrics,
    /// Collection view metrics.
    #[serde(default)]
    pub collection_view: CollectionViewMetrics,
    /// Player view metrics.
    #[serde(default)]
    pub player_view: PlayerViewMetrics,
    /// Box view metrics.
    #[serde(default)]
    pub box_view: BoxViewMetrics,
}
