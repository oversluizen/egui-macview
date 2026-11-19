//! macOS AppKit Authentic Controls Suite for egui.
//!
//! Grounded in Apple macOS Human Interface Guidelines (HIG)
//! and W3C DTCG Token Catalogues (`tokens/design_tokens.json`).

pub mod box_view;
pub mod button_cell;
pub mod checkbox;
pub mod circular_slider;
pub mod collection_view;
pub mod color_well;
pub mod combo_box;
pub mod context_thumb_dial;
pub mod date_picker;
pub mod disclosure_group;
pub mod edge_thumbwheel;
pub mod free_joystick_bowl;
pub mod help_button;
pub mod image_view;
pub mod input_cell;
pub mod label;
pub mod level_indicator;
pub mod liquid_glass;
pub mod origin_matrix;
pub mod outline_view;
pub mod path_control;
pub mod player_view;
pub mod popup_button;
pub mod progress_indicator;
pub mod rating_indicator;
pub mod rule_editor;
pub mod search_field;
pub mod segmented;
pub mod slider;
pub mod stepper;
pub mod tab_view;
pub mod table_action_bar;
pub mod table_view;
pub mod tag_color_picker;
pub mod text_view;
pub mod thumbnail_cell;
pub mod toggle;
pub mod token_field;
pub mod trim_scrubber;

pub use box_view::MacBox;
pub use button_cell::{MacButton, MacButtonCell};
pub use checkbox::{MacCheckbox, MacRadioButton};
pub use circular_slider::MacCircularSlider;
pub use collection_view::{MacCollectionView, MacCollectionViewFlowLayout};
pub use color_well::MacColorWell;
pub use combo_box::MacComboBox;
pub use context_thumb_dial::{
    MacContextThumbDial, MacContextThumbDialState, MacContextThumbResponse, MacSFSymbol, MacTapticEvent, MacThumbArcPlacement,
    MacThumbContextPreset, MacThumbDialLayer, MacThumbDockZone, MacThumbIntent, MacThumbIntentMask, MacThumbSector, MacThumbSectorIcon,
    MacThumbSectorSet, MacThumbSectorSpec,
};
pub use date_picker::{MacDate, MacDatePicker, MacDatePickerElementFlags, MacDatePickerStyle};
pub use disclosure_group::{MacDisclosureButton, MacDisclosureGroup, MacDisclosureResponse};
pub use edge_thumbwheel::{MacEdgeThumbwheel, MacEdgeThumbwheelResponse};
pub use free_joystick_bowl::{MacFreeJoystickBowl, MacFreeJoystickResponse};
pub use help_button::{InlineActionKind, MacHelpButton, MacInlineActionButton};
pub use image_view::{MacImageFrameStyle, MacImageScaling, MacImageView};
pub use input_cell::{MacControlSize, MacInputCell};
pub use label::{MacFontWeight, MacLabel, MacLabelColor};
pub use level_indicator::{calculate_pan_weights, MacLevelIndicator, MacLevelIndicatorStyle, MacLevelOrientation};
pub use liquid_glass::{
    build_liquid_glass_backdrop_image, dome_surface_normal_3d, refract_point_through_glass, sdf_annular_rounded_sector,
    sdf_curved_stadium_arc, sdf_rounded_capsule, sdf_smooth_union,
};
pub use origin_matrix::{MacOriginAnchor, MacOriginMatrix};
pub use outline_view::{MacOutlineRow, MacOutlineRowResponse, MacOutlineTree, MacOutlineView};
pub use path_control::{MacPathComponent, MacPathControl, MacPathControlResponse};
pub use player_view::{
    format_remaining_timecode, format_timecode, MacPlayerControlsStyle, MacPlayerView, MacPlayerViewResponse, PlayerViewGeometry,
    PlayerViewState,
};
pub use popup_button::MacPopUpButton;
pub use progress_indicator::{MacProgressControlSize, MacProgressIndicator, MacProgressStyle};
pub use rating_indicator::{MacRatingIndicator, UNIT_STAR};
pub use rule_editor::{MacRuleEditor, MacRuleRow};
pub use search_field::MacSearchField;
pub use segmented::{
    MacSegmentEvent, MacSegmentItem, MacSegmentedControl, SegmentContent, SegmentEvent, SegmentIconFn, SegmentIconPainter, SegmentItem,
    SegmentMenuTrigger,
};
pub use slider::MacSlider;
pub use stepper::MacStepper;
pub use tab_view::{MacInsets, MacTabItem, MacTabStyle, MacTabView};
pub use table_action_bar::{MacTableActionBar, MacTableActionResponse};
pub use table_view::{MacSortDirection, MacTableColumn, MacTableView, MacTableViewOutput, MacTableViewResponse};
pub use tag_color_picker::MacTagColorPicker;
pub use text_view::MacTextView;
pub use thumbnail_cell::MacThumbnailCell;
pub use toggle::{MacSwitch, MacToggle};
pub use token_field::MacTokenField;
pub use trim_scrubber::{MacTimelineZone, MacTrimScrubber, MacTrimScrubberResponse};
