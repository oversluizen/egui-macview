//! macOS AppKit Authentic Controls Suite for egui.
//!
//! Grounded in Apple macOS Human Interface Guidelines (HIG)
//! and W3C DTCG Token Catalogues (`tokens/design_tokens.json`).

pub mod button_cell;
pub mod input_cell;
pub mod segmented;
pub mod toggle;

pub use button_cell::MacButtonCell;
pub use input_cell::MacInputCell;
pub use segmented::MacSegmentedControl;
pub use toggle::MacToggle;
