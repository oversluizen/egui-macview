//! `egui-macview` — Deterministic macOS AppKit Layout Engine & CoreAnimation Physics for egui.

pub mod layout;
pub mod controls;
pub mod motion;
pub mod surfaces;

pub use layout::{MacFixedView, MacFlexView, MacFrame};
pub use controls::{MacButtonCell, MacInputCell, MacSegmentedControl, MacToggle};
pub use motion::CASpring;
