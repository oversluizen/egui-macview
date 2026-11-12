//! AppKit Control geometry metrics tokens.

pub mod advanced;
pub mod basic;
pub mod extended;
pub mod media_and_date;
pub mod nib_primitives;
pub mod pro_inspector;
pub mod selection_and_value;
pub mod studio;

pub use advanced::{
    BoxViewMetrics, ComboBoxMetrics, DisclosureGroupMetrics, RuleEditorMetrics, TabViewMetrics,
};
pub use basic::{
    ButtonMetrics, ButtonSizeMetrics, FocusRingMetrics, SegmentedMetrics, SegmentedSizeMetrics,
    TextFieldMetrics, TextFieldSizeMetrics,
};
pub use extended::{
    ColorWellMetrics, OutlineViewMetrics, PopUpButtonMetrics, SearchFieldMetrics, SliderMetrics,
    StepperMetrics, ThumbnailCellMetrics, ToggleMetrics,
};
pub use media_and_date::{CollectionViewMetrics, DatePickerMetrics, PlayerViewMetrics};
pub use nib_primitives::{
    CheckboxMetrics, CheckboxSizeMetrics, HelpButtonMetrics, InlineActionButtonMetrics,
    LabelMetrics,
};
pub use pro_inspector::{OriginMatrixMetrics, TableActionBarMetrics, TagColorPickerMetrics};
pub use studio::{
    LevelIndicatorMetrics, PathControlMetrics, ProgressIndicatorMetrics, ProgressSizeMetrics,
    TableViewMetrics, TokenFieldMetrics, TrimScrubberMetrics,
};

use crate::tokens::dtcg::DimensionValue;
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
    pub slider: SliderMetrics,
    /// Thumbnail cell metrics.
    pub thumbnail: ThumbnailCellMetrics,
    /// Outline tree view metrics.
    pub outline_view: OutlineViewMetrics,
    /// Pop-up menu button metrics.
    pub popup_button: PopUpButtonMetrics,
    /// Search field metrics.
    pub search_field: SearchFieldMetrics,
    /// Numeric stepper metrics.
    pub stepper: StepperMetrics,
    /// Color well swatch metrics.
    pub color_well: ColorWellMetrics,
    /// Toggle switch metrics.
    pub toggle: ToggleMetrics,
    /// Opacity multiplier for disabled controls.
    pub disabled_opacity: DimensionValue,
    /// Progress indicator metrics.
    pub progress_indicator: ProgressIndicatorMetrics,
    /// Table view and header metrics.
    pub table_view: TableViewMetrics,
    /// Token field pill input metrics.
    pub token_field: TokenFieldMetrics,
    /// Level indicator meter metrics.
    pub level_indicator: LevelIndicatorMetrics,
    /// Trim scrubber timeline metrics.
    pub trim_scrubber: TrimScrubberMetrics,
    /// Path control breadcrumb metrics.
    pub path_control: PathControlMetrics,
    /// Disclosure group & button metrics.
    pub disclosure_group: DisclosureGroupMetrics,
    /// Combo box metrics.
    pub combo_box: ComboBoxMetrics,
    /// Tab view metrics.
    pub tab_view: TabViewMetrics,
    /// Date picker metrics.
    pub date_picker: DatePickerMetrics,
    /// Collection view metrics.
    pub collection_view: CollectionViewMetrics,
    /// Player view metrics.
    pub player_view: PlayerViewMetrics,
    /// Box view metrics.
    pub box_view: BoxViewMetrics,
    /// Rule editor metrics.
    pub rule_editor: RuleEditorMetrics,
    /// Square AppKit checkbox (`NSButtonTypeSwitch`) metrics.
    pub checkbox: CheckboxMetrics,
    /// Non-editable AppKit label (`NSTextField.labelWithString:`) metrics.
    pub label: LabelMetrics,
    /// Circular AppKit help button (`NSBezelStyleHelpButton`) metrics.
    pub help_button: HelpButtonMetrics,
    /// Circular inline action badge button (`16x16pt`) metrics.
    pub inline_action_button: InlineActionButtonMetrics,
    /// `3x3` 9-point transform origin / anchor pin grid metrics.
    pub origin_matrix: OriginMatrixMetrics,
    /// macOS Finder / AppKit color label swatch picker metrics.
    pub tag_color_picker: TagColorPickerMetrics,
    /// `NSBezelStyleSmallSquare` `[ + | - ]` table action bar metrics.
    pub table_action_bar: TableActionBarMetrics,
}
