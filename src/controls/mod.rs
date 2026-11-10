//! macOS AppKit Authentic Controls Suite for egui.
//!
//! Grounded in Apple macOS Human Interface Guidelines (HIG)
//! and W3C DTCG Token Catalogues (`tokens/design_tokens.json`).

pub mod button_cell;
pub mod circular_slider;
pub mod color_well;
pub mod input_cell;
pub mod level_indicator;
pub mod outline_view;
pub mod path_control;
pub mod popup_button;
pub mod progress_indicator;
pub mod rating_indicator;
pub mod search_field;
pub mod segmented;
pub mod slider;
pub mod stepper;
pub mod table_view;
pub mod thumbnail_cell;
pub mod toggle;
pub mod token_field;
pub mod trim_scrubber;

pub use button_cell::MacButtonCell;
pub use circular_slider::MacCircularSlider;
pub use color_well::MacColorWell;
pub use input_cell::{MacControlSize, MacInputCell};
pub use level_indicator::{
    calculate_pan_weights, MacLevelIndicator, MacLevelIndicatorStyle, MacLevelOrientation,
};
pub use outline_view::{MacOutlineRow, MacOutlineRowResponse, MacOutlineTree, MacOutlineView};
pub use path_control::{MacPathComponent, MacPathControl, MacPathControlResponse};
pub use popup_button::MacPopUpButton;
pub use progress_indicator::{MacProgressControlSize, MacProgressIndicator, MacProgressStyle};
pub use rating_indicator::{MacRatingIndicator, UNIT_STAR};
pub use search_field::MacSearchField;
pub use segmented::MacSegmentedControl;
pub use slider::MacSlider;
pub use stepper::MacStepper;
pub use table_view::{
    MacSortDirection, MacTableColumn, MacTableView, MacTableViewOutput, MacTableViewResponse,
};
pub use thumbnail_cell::MacThumbnailCell;
pub use toggle::MacToggle;
pub use token_field::MacTokenField;
pub use trim_scrubber::{MacTimelineZone, MacTrimScrubber, MacTrimScrubberResponse};
