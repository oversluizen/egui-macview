//! Authentic macOS AppKit menu item representations and keyboard shortcuts.
//!
//! Normative Reference: SPEC-007 §2.

use egui::Key;

use super::model::NSMenu;

/// Keyboard shortcut representation with modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacKeyboardShortcut {
    /// Base key code.
    pub key: Key,
    /// Whether Cmd (macOS) or Ctrl (Windows/Linux) modifier must be pressed.
    pub command: bool,
    /// Whether Shift modifier must be pressed.
    pub shift: bool,
    /// Whether Alt/Option modifier must be pressed.
    pub alt: bool,
}

impl MacKeyboardShortcut {
    /// Constructs a basic shortcut without modifiers.
    pub const fn new(key: Key) -> Self {
        Self {
            key,
            command: false,
            shift: false,
            alt: false,
        }
    }

    /// Constructs a Cmd/Ctrl shortcut (`Ctrl+key` or `Cmd+key`).
    pub const fn cmd(key: Key) -> Self {
        Self {
            key,
            command: true,
            shift: false,
            alt: false,
        }
    }

    /// Constructs a Cmd+Shift shortcut (`Ctrl+Shift+key` or `Cmd+Shift+key`).
    pub const fn cmd_shift(key: Key) -> Self {
        Self {
            key,
            command: true,
            shift: true,
            alt: false,
        }
    }

    /// Checks if the input state matches this keyboard shortcut.
    pub fn matches(&self, input: &egui::InputState) -> bool {
        let input_cmd = input.modifiers.command || input.modifiers.ctrl || input.modifiers.mac_cmd;
        let input_shift = input.modifiers.shift;
        let input_alt = input.modifiers.alt;

        if input_cmd == self.command
            && input_shift == self.shift
            && input_alt == self.alt
            && input.key_pressed(self.key)
        {
            return true;
        }

        // Check matching key-press event in current frame events
        input.events.iter().any(|ev| {
            if let egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = ev
            {
                if *key == self.key {
                    let ev_cmd = modifiers.command || modifiers.ctrl || modifiers.mac_cmd;
                    return ev_cmd == self.command
                        && modifiers.shift == self.shift
                        && modifiers.alt == self.alt;
                }
            }
            false
        })
    }

    /// Formats shortcut as a human-readable display string (e.g. `"Ctrl+Z"`).
    pub fn format_text(&self) -> String {
        let mut s = String::new();
        if self.command {
            s.push_str("Ctrl+");
        }
        if self.alt {
            s.push_str("Alt+");
        }
        if self.shift {
            s.push_str("Shift+");
        }
        s.push_str(&format!("{:?}", self.key));
        s
    }
}

/// Variant of menu item in a menu hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub enum MacMenuItemKind {
    /// Leaf action triggering a designated action identifier.
    Action {
        /// Deterministic action identifier string.
        id: String,
        /// Optional keyboard shortcut trigger.
        shortcut: Option<MacKeyboardShortcut>,
    },
    /// Submenu containing nested child items.
    Submenu {
        /// Nested child menu hierarchy.
        menu: NSMenu,
    },
    /// Visual dividing line between logical menu item clusters.
    Separator,
}

/// Item within an `NSMenu` hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct NSMenuItem {
    /// Human-readable menu title label.
    pub title: String,
    /// Specific item kind (Action, Submenu, or Separator).
    pub kind: MacMenuItemKind,
    /// Whether the item is interactive or grayed out.
    pub enabled: bool,
    /// Whether the item displays a checkmark indicator.
    pub checked: bool,
}

impl NSMenuItem {
    /// Creates an action item with title and identifier.
    pub fn action(title: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            kind: MacMenuItemKind::Action {
                id: id.into(),
                shortcut: None,
            },
            enabled: true,
            checked: false,
        }
    }

    /// Creates an action item with title, identifier, and keyboard shortcut accelerator.
    pub fn action_with_shortcut(
        title: impl Into<String>,
        id: impl Into<String>,
        shortcut: MacKeyboardShortcut,
    ) -> Self {
        Self {
            title: title.into(),
            kind: MacMenuItemKind::Action {
                id: id.into(),
                shortcut: Some(shortcut),
            },
            enabled: true,
            checked: false,
        }
    }

    /// Creates a cascading submenu item.
    pub fn submenu(title: impl Into<String>, menu: NSMenu) -> Self {
        Self {
            title: title.into(),
            kind: MacMenuItemKind::Submenu { menu },
            enabled: true,
            checked: false,
        }
    }

    /// Creates a visual horizontal separator.
    pub fn separator() -> Self {
        Self {
            title: String::new(),
            kind: MacMenuItemKind::Separator,
            enabled: false,
            checked: false,
        }
    }

    /// Enables or disables the menu item.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the checked/selected state of the item.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
}
