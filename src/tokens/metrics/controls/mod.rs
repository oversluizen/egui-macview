//! AppKit Control geometry metrics tokens.

pub mod basic;
pub mod extended;

pub use basic::{
    default_border_width, ButtonMetrics, ButtonSizeMetrics, FocusRingMetrics, SegmentedMetrics,
    SegmentedSizeMetrics, TextFieldMetrics, TextFieldSizeMetrics,
};
pub use extended::{
    ColorWellMetrics, OutlineViewMetrics, PopUpButtonMetrics, SearchFieldMetrics, SliderMetrics,
    StepperMetrics, ThumbnailCellMetrics, ToggleMetrics,
};

use serde::{Deserialize, Serialize};

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
}
