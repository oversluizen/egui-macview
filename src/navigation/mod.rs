//! Desktop Navigation, Canvas Footer, and Spotlight Command Palette Suite.

pub mod footer;
pub mod item;
pub mod menu;
pub mod model;
pub mod palette;

pub use footer::{MacCanvasFooter, MacCanvasFooterAction, CANVAS_FOOTER_HEIGHT};
pub use item::{MacKeyboardShortcut, MacMenuItemKind, NSMenuItem};
pub use menu::MacMenuBar;
pub use model::NSMenu;
pub use palette::{
    CommandPaletteItem, MacCommandPalette, COMMAND_PALETTE_HEIGHT, COMMAND_PALETTE_WIDTH,
};
