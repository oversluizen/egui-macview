//! Authentic macOS AppKit `NSMenu` data structure and standard presets.
//!
//! Normative Reference: SPEC-007 §2.

use egui::Key;

use super::item::{MacKeyboardShortcut, NSMenuItem};

/// Data-driven application menu (`NSMenu` equivalent).
#[derive(Debug, Clone, PartialEq)]
pub struct NSMenu {
    /// Title of the top-level menu (e.g. "File", "Edit").
    pub title: String,
    /// List of child menu items in this menu.
    pub items: Vec<NSMenuItem>,
}

impl NSMenu {
    /// Creates an empty menu with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
        }
    }

    /// Creates a menu with a title and a list of items.
    pub fn with_items(title: impl Into<String>, items: Vec<NSMenuItem>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }

    /// Appends an item to the menu.
    pub fn item(mut self, item: NSMenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Default macOS "File" menu.
    pub fn default_file_menu() -> Self {
        Self::new("File")
            .item(NSMenuItem::action_with_shortcut(
                "New",
                "file.new",
                MacKeyboardShortcut::cmd(Key::N),
            ))
            .item(NSMenuItem::action_with_shortcut(
                "Open...",
                "file.open",
                MacKeyboardShortcut::cmd(Key::O),
            ))
            .item(NSMenuItem::separator())
            .item(NSMenuItem::action_with_shortcut(
                "Save",
                "file.save",
                MacKeyboardShortcut::cmd(Key::S),
            ))
            .item(NSMenuItem::action_with_shortcut(
                "Close Window",
                "file.close",
                MacKeyboardShortcut::cmd(Key::W),
            ))
    }

    /// Default macOS "Edit" menu.
    pub fn default_edit_menu() -> Self {
        Self::new("Edit")
            .item(NSMenuItem::action_with_shortcut(
                "Undo",
                "edit.undo",
                MacKeyboardShortcut::cmd(Key::Z),
            ))
            .item(NSMenuItem::action_with_shortcut(
                "Redo",
                "edit.redo",
                MacKeyboardShortcut::cmd_shift(Key::Z),
            ))
            .item(NSMenuItem::separator())
            .item(NSMenuItem::action_with_shortcut(
                "Cut",
                "edit.cut",
                MacKeyboardShortcut::cmd(Key::X),
            ))
            .item(NSMenuItem::action_with_shortcut(
                "Copy",
                "edit.copy",
                MacKeyboardShortcut::cmd(Key::C),
            ))
            .item(NSMenuItem::action_with_shortcut(
                "Paste",
                "edit.paste",
                MacKeyboardShortcut::cmd(Key::V),
            ))
            .item(NSMenuItem::action_with_shortcut(
                "Select All",
                "edit.select_all",
                MacKeyboardShortcut::cmd(Key::A),
            ))
    }

    /// Default macOS "View" menu.
    pub fn default_view_menu() -> Self {
        Self::new("View")
            .item(NSMenuItem::action_with_shortcut(
                "Zoom In",
                "view.zoom_in",
                MacKeyboardShortcut::cmd(Key::Plus),
            ))
            .item(NSMenuItem::action_with_shortcut(
                "Zoom Out",
                "view.zoom_out",
                MacKeyboardShortcut::cmd(Key::Minus),
            ))
            .item(NSMenuItem::action_with_shortcut(
                "Actual Size",
                "view.zoom_reset",
                MacKeyboardShortcut::cmd(Key::Num0),
            ))
    }

    /// Default macOS "Help" menu.
    pub fn default_help_menu() -> Self {
        Self::new("Help")
            .item(NSMenuItem::action_with_shortcut(
                "Command Palette...",
                "help.command_palette",
                MacKeyboardShortcut::cmd(Key::K),
            ))
            .item(NSMenuItem::action("About", "help.about"))
    }
}
