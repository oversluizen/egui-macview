#![doc = include_str!("../README.md")]
//!
//! Grounded in Apple AppKit, macOS Human Interface Guidelines (HIG), and W3C DTCG Design Tokens.
//!
//! # Architecture & Highlights
//!
//! - **Authentic Control Primitives**: Complete macOS AppKit controls including [`MacToggle`], [`MacSlider`],
//!   [`MacSegmentedControl`], [`MacPopUpButton`], [`MacSearchField`], [`MacStepper`], [`MacColorWell`],
//!   [`MacOutlineView`], [`MacButtonCell`], [`MacInputCell`], and [`MacThumbnailCell`].
//! - **Deterministic Layout Engines**: Subpixel-snapped linear [`MacStackView`] (`NSStackView`), draggable hairline [`MacSplitView`] (`NSSplitView`),
//!   zero-gutter floating overlay [`MacScrollView`] (`NSScrollView`), absolute [`MacFixedView`], and autolayout [`MacRow`].
//! - **CoreAnimation Physics**: Analytical spring physics runtime [`CASpring`] matching Apple's `CASpringAnimation`
//!   and SwiftUI `.spring(response:dampingRatio:)` with analytical displacement and velocity calculations.
//! - **W3C DTCG Token Catalog & Theming**: Dynamic theming via [`MacTheme`] and [`MacAppearance`] supporting Aqua and DarkAqua
//!   appearances, configurable window control decorations via [`WindowControlsStyle`], and dynamic system accent colors.
//! - **Subpixel Snapping**: Physical pixel boundary rounding via [`subpixel_snap`], [`subpixel_snap_pos`], and [`subpixel_snap_rect`].
//!
//! # Quick Start Example
//!
//! ```rust
//! use egui_macview::{
//!     CASpring, MacAppearance, MacSlider, MacStackView, MacTheme, MacToggle,
//! };
//!
//! # let ctx = egui::Context::default();
//! # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
//! // 1. Initialize macOS dynamic theme (Aqua or DarkAqua)
//! let theme = MacTheme::new(MacAppearance::DarkAqua);
//!
//! // 2. Assemble controls inside an NSStackView linear layout
//! let mut enabled = true;
//! let mut volume = 75.0f32;
//!
//! MacStackView::vertical()
//!     .theme(&theme)
//!     .show_separators(true)
//!     .show(ui, |stack| {
//!         stack.add(|ui| {
//!             ui.add(MacToggle::new(&mut enabled).theme(&theme));
//!         });
//!         stack.add(|ui| {
//!             MacSlider::new(&mut volume, 0.0..=100.0)
//!                 .theme(&theme)
//!                 .ticks(5, true)
//!                 .show(ui);
//!         });
//!     });
//!
//! // 3. Analytical spring physics evaluation
//! let spring = CASpring::new(0.35, 0.85);
//! let displacement = spring.evaluate(0.15); // t = 150ms
//! # assert!(displacement > 0.0);
//! # });
//! # output.textures_delta.clear();
//! ```

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
    setup_symbols_font, MacSymbol, CUPERTINO_ICONS_FONT_BYTES, CUPERTINO_ICONS_FONT_NAME,
};

pub use theme::{
    contrast_ratio, relative_luminance, MacAppearance, MacBackgroundStyle, MacTheme,
    WindowControlsStyle,
};

pub use controls::{
    MacButtonCell, MacColorWell, MacControlSize, MacInputCell, MacOutlineRow,
    MacOutlineRowResponse, MacOutlineTree, MacOutlineView, MacPopUpButton, MacSearchField,
    MacSegmentedControl, MacSlider, MacStepper, MacThumbnailCell, MacToggle,
};
pub use feedback::{clamp_menu_rect, MacContextMenu, MacToast, MacToastManager, ToastLevel};
pub use layout::{
    subpixel_snap, subpixel_snap_pos, subpixel_snap_rect, MacFixedView, MacForm, MacFormUi,
    MacFrame, MacRow, MacScrollView, MacScrollViewOutput, MacSplitOrientation, MacSplitView,
    MacStackAlignment, MacStackOrientation, MacStackUi, MacStackView, MacStackViewOutput,
};
pub use motion::CASpring;
pub use navigation::{
    CommandPaletteItem, MacCanvasFooter, MacCanvasFooterAction, MacCommandPalette,
    MacKeyboardShortcut, MacMenuBar, MacMenuItemKind, MacToolbar, MacToolbarResponse, NSMenu,
    NSMenuItem, CANVAS_FOOTER_HEIGHT, COMMAND_PALETTE_HEIGHT, COMMAND_PALETTE_WIDTH,
};
pub use state::{NSUndoManager, DEFAULT_UNDO_CAPACITY};
pub use surfaces::{MacCard, MacFloatingInfoBox, MacPanel, MacPopover, MacPopoverArrowEdge};
pub use tokens::{
    sf_pro_tracking, ColorWellMetrics, LayoutMetrics, MacColorTokens, MacControlMetrics,
    MacDesignTokens, MacMotionTokens, MacTypographyTokens, OutlineViewMetrics, PopUpButtonMetrics,
    ScrollViewMetrics, SearchFieldMetrics, SliderMetrics, SplitViewMetrics, StackViewMetrics,
    StepperMetrics, ThumbnailCellMetrics,
};
