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

    /// Recursively searches for a matching keyboard shortcut in this menu.
    pub fn find_shortcut(&self, input: &egui::InputState) -> Option<String> {
        Self::find_shortcut_in_items(&self.items, input)
    }

    /// Recursively searches for and consumes a matching keyboard shortcut in this menu.
    pub fn consume_shortcut(&self, input: &mut egui::InputState) -> Option<String> {
        Self::consume_shortcut_in_items(&self.items, input)
    }

    /// Recursively searches items for a matching keyboard shortcut and consumes it.
    pub fn consume_shortcut_in_items(
        items: &[NSMenuItem],
        input: &mut egui::InputState,
    ) -> Option<String> {
        for item in items {
            if !item.enabled {
                continue;
            }
            match &item.kind {
                super::item::MacMenuItemKind::Action {
                    id,
                    shortcut: Some(sc),
                } => {
                    if sc.consume(input) {
                        return Some(id.clone());
                    }
                }
                super::item::MacMenuItemKind::Submenu { menu } => {
                    if let Some(act) = Self::consume_shortcut_in_items(&menu.items, input) {
                        return Some(act);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Recursively searches items for a matching keyboard shortcut.
    pub fn find_shortcut_in_items(
        items: &[NSMenuItem],
        input: &egui::InputState,
    ) -> Option<String> {
        for item in items {
            if !item.enabled {
                continue;
            }
            match &item.kind {
                super::item::MacMenuItemKind::Action {
                    id,
                    shortcut: Some(sc),
                } => {
                    if sc.matches(input) {
                        return Some(id.clone());
                    }
                }
                super::item::MacMenuItemKind::Submenu { menu } => {
                    if let Some(act) = Self::find_shortcut_in_items(&menu.items, input) {
                        return Some(act);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Default macOS "File" menu (SPEC-007 §2.2).
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
            .item(NSMenuItem::action("Export...", "file.export"))
            .item(NSMenuItem::separator())
            .item(NSMenuItem::action_with_shortcut(
                "Close Window",
                "file.close",
                MacKeyboardShortcut::cmd(Key::W),
            ))
    }

    /// Default macOS "Edit" menu (SPEC-007 §2.2).
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

    /// Default macOS "View" menu (SPEC-007 §2.2).
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
            .item(NSMenuItem::separator())
            .item(NSMenuItem::action(
                "Toggle Inspector",
                "view.toggle_inspector",
            ))
            .item(NSMenuItem::action("Fullscreen", "view.fullscreen"))
    }

    /// Default macOS "Window" menu (SPEC-007 §2.2).
    pub fn default_window_menu() -> Self {
        Self::new("Window")
            .item(NSMenuItem::action_with_shortcut(
                "Minimize",
                "window.minimize",
                MacKeyboardShortcut::cmd(Key::M),
            ))
            .item(NSMenuItem::action("Zoom", "window.zoom"))
            .item(NSMenuItem::separator())
            .item(NSMenuItem::action(
                "Bring All to Front",
                "window.bring_all_to_front",
            ))
    }

    /// Default macOS "Help" menu (SPEC-007 §2.2).
    pub fn default_help_menu() -> Self {
        Self::new("Help")
            .item(NSMenuItem::action("Documentation", "help.documentation"))
            .item(NSMenuItem::action_with_shortcut(
                "Keyboard Shortcuts...",
                "help.shortcuts",
                MacKeyboardShortcut::cmd(Key::K),
            ))
            .item(NSMenuItem::separator())
            .item(NSMenuItem::action("About", "help.about"))
    }

    /// Standard root menu bar containing all 5 standard top-level menus (SPEC-007 §2.2).
    pub fn default_menu_bar() -> Vec<Self> {
        vec![
            Self::default_file_menu(),
            Self::default_edit_menu(),
            Self::default_view_menu(),
            Self::default_window_menu(),
            Self::default_help_menu(),
        ]
    }
}
