//! # `egui-macview`
//!
//! Native-style macOS AppKit controls, layouts, and smooth spring animations for [`egui`].
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

/// Crate release version string extracted from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod controls;
pub mod feedback;
pub mod layout;
pub mod motion;
pub mod navigation;
pub mod state;
pub mod surfaces;
pub mod symbols;
pub mod theme;
pub mod tokens;

pub use symbols::{
    setup_macview_fonts, setup_symbols_font, MacSymbol, CUPERTINO_ICONS_FONT_BYTES,
    CUPERTINO_ICONS_FONT_NAME, INTER_REGULAR_FONT_BYTES, INTER_REGULAR_FONT_NAME,
    INTER_SEMIBOLD_FONT_BYTES, INTER_SEMIBOLD_FONT_NAME,
};

pub use theme::{
    alpha_composite, contrast_ratio, ensure_wcag_contrast, plus_darker, plus_lighter,
    relative_luminance, MacAppearance, MacBackgroundStyle, MacTheme, WindowControlsStyle,
};

pub use controls::{
    calculate_pan_weights, format_remaining_timecode, format_timecode, MacBox, MacButton,
    MacButtonCell, MacCheckbox, MacCircularSlider, MacCollectionView, MacCollectionViewFlowLayout,
    MacColorWell, MacComboBox, MacControlSize, MacDate, MacDatePicker, MacDatePickerElementFlags,
    MacDatePickerStyle, MacDisclosureButton, MacDisclosureGroup, MacDisclosureResponse,
    MacFontWeight, MacHelpButton, MacInlineActionButton, MacInputCell, MacInsets, MacLabel,
    MacLabelColor, MacLevelIndicator, MacLevelIndicatorStyle, MacLevelOrientation, MacOriginAnchor,
    MacOriginMatrix, MacOutlineRow, MacOutlineRowResponse, MacOutlineTree, MacOutlineView,
    MacPathComponent, MacPathControl, MacPathControlResponse, MacPlayerControlsStyle,
    MacPlayerView, MacPopUpButton, MacProgressControlSize, MacProgressIndicator, MacProgressStyle,
    MacRadioButton, MacRatingIndicator, MacRuleEditor, MacRuleRow, MacSearchField,
    MacSegmentEvent, MacSegmentItem, MacSegmentedControl, MacSlider, MacSortDirection, MacStepper,
    MacTabItem, MacTabStyle, MacTabView, MacTableActionBar, MacTableActionResponse, MacTableColumn,
    MacTableView, MacTableViewOutput, MacTableViewResponse, MacTagColorPicker, MacThumbnailCell,
    MacTimelineZone, MacToggle, MacTokenField, MacTrimScrubber, MacTrimScrubberResponse,
    SegmentContent, SegmentEvent, SegmentIconFn, SegmentIconPainter, SegmentItem,
    SegmentMenuTrigger, UNIT_STAR,
};
pub use feedback::{clamp_menu_rect, MacContextMenu, MacToast, MacToastManager, ToastLevel};
pub use layout::{
    is_window_active, subpixel_snap, subpixel_snap_pos, subpixel_snap_rect, MacFixedView, MacForm,
    MacFormUi, MacFrame, MacRow, MacScrollView, MacScrollViewOutput, MacSplitOrientation,
    MacSplitView, MacStackAlignment, MacStackOrientation, MacStackUi, MacStackView,
    MacStackViewOutput,
};
pub use motion::CASpring;
pub use navigation::{
    CommandPaletteItem, MacCanvasFooter, MacCanvasFooterAction, MacCommandPalette,
    MacKeyboardShortcut, MacMenuBar, MacMenuItemKind, MacToolbar, MacToolbarResponse, NSMenu,
    NSMenuItem, CANVAS_FOOTER_HEIGHT, COMMAND_PALETTE_HEIGHT, COMMAND_PALETTE_WIDTH,
};
pub use state::{NSUndoManager, DEFAULT_UNDO_CAPACITY};
pub use surfaces::{
    MacCard, MacFloatingInfoBox, MacPanel, MacPopover, MacPopoverArrowEdge, MacPopoverSurface,
};
pub use tokens::{
    sf_pro_tracking, CheckboxMetrics, CheckboxSizeMetrics, ColorWellMetrics, HelpButtonMetrics,
    InlineActionButtonMetrics, LabelMetrics, LayoutMetrics, LevelIndicatorMetrics, MacColorTokens,
    MacControlMetrics, MacDesignTokens, MacMotionTokens, MacTagLabelColors, MacTypographyTokens,
    OriginMatrixMetrics, OutlineViewMetrics, PathControlMetrics, PopUpButtonMetrics,
    ProgressIndicatorMetrics, ProgressSizeMetrics, ScrollViewMetrics, SearchFieldMetrics,
    SliderMetrics, SplitViewMetrics, StackViewMetrics, StepperMetrics, TableActionBarMetrics,
    TableViewMetrics, TagColorPickerMetrics, ThumbnailCellMetrics, ToggleMetrics, TokenFieldMetrics,
    TrimScrubberMetrics,
};
