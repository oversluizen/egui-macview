//! `egui-macview` — Deterministic macOS AppKit Layout Engine & CoreAnimation Physics for egui.

pub mod layout;
pub mod controls;
pub mod motion;
pub mod surfaces;
pub mod tokens;

pub use layout::{subpixel_snap, subpixel_snap_pos, subpixel_snap_rect, MacFixedView, MacFlexView, MacFrame};
pub use controls::{MacButtonCell, MacInputCell, MacSegmentedControl, MacToggle};
pub use motion::CASpring;
pub use tokens::{
    sf_pro_tracking, MacColorTokens, MacControlMetrics, MacDesignTokens, MacMotionTokens,
    MacTypographyTokens,
};
