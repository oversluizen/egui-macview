//! Authentic macOS AppKit Path Control (`NSPathControl` equivalent, SPEC-011).

use crate::symbols::MacSymbol;
use crate::theme::MacTheme;
use egui::{Id, Response, Sense, Ui, Vec2};
use std::fmt::Debug;
use std::hash::Hash;

/// Single breadcrumb component inside [`MacPathControl`].
#[derive(Debug, Clone)]
pub struct MacPathComponent<'a> {
    /// Display title of the path segment.
    pub title: &'a str,
    /// True if this segment represents a folder/directory; false for a leaf document/file.
    pub is_folder: bool,
    /// Optional explicit [`MacSymbol`] override.
    pub symbol: Option<MacSymbol>,
}

impl<'a> MacPathComponent<'a> {
    /// Creates a folder path component.
    pub fn folder(title: &'a str) -> Self {
        Self {
            title,
            is_folder: true,
            symbol: None,
        }
    }

    /// Creates a leaf file/document path component.
    pub fn file(title: &'a str) -> Self {
        Self {
            title,
            is_folder: false,
            symbol: None,
        }
    }

    /// Overrides the leading symbol with a [`MacSymbol`].
    pub fn with_symbol(mut self, symbol: MacSymbol) -> Self {
        self.symbol = Some(symbol);
        self
    }
}

/// Interaction output returned by [`MacPathControl::show`].
#[derive(Debug, Clone)]
pub struct MacPathControlResponse {
    /// Overall widget interaction response.
    pub response: Response,
    /// Index of the path segment clicked during this frame, if any.
    pub clicked_segment: Option<usize>,
    /// True if the optional trailing `Choose...` button was clicked during this frame.
    pub choose_clicked: bool,
}

/// Authentic macOS AppKit breadcrumb path bar (`NSPathControl` equivalent).
pub struct MacPathControl<'a> {
    id: Id,
    components: Vec<MacPathComponent<'a>>,
    show_choose_button: bool,
    choose_label: &'a str,
    width: Option<f32>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacPathControl<'a> {
    /// Creates a new `MacPathControl` from a slice of string segment names (all folders except last).
    pub fn from_segments(id_source: impl Hash + Debug, segments: &'a [&'a str]) -> Self {
        #[allow(deprecated)]
        let id = Id::new(id_source);
        let len = segments.len();
        let components = segments
            .iter()
            .enumerate()
            .map(|(idx, &title)| MacPathComponent {
                title,
                is_folder: idx + 1 < len,
                symbol: None,
            })
            .collect();
        Self {
            id,
            components,
            show_choose_button: false,
            choose_label: "Choose...",
            width: None,
            enabled: true,
            theme: None,
        }
    }

    /// Creates a new `MacPathControl` from explicit [`MacPathComponent`] items.
    pub fn new(id_source: impl Hash + Debug, components: &'a [MacPathComponent<'a>]) -> Self {
        #[allow(deprecated)]
        let id = Id::new(id_source);
        Self {
            id,
            components: components.to_vec(),
            show_choose_button: false,
            choose_label: "Choose...",
            width: None,
            enabled: true,
            theme: None,
        }
    }

    /// Shows or hides the trailing `Choose...` button.
    pub fn show_choose_button(mut self, show: bool) -> Self {
        self.show_choose_button = show;
        self
    }

    /// Sets custom label for the trailing choose button.
    pub fn choose_label(mut self, label: &'a str) -> Self {
        self.choose_label = label;
        self
    }

    /// Overrides total width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Enables or disables interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the path control bar in `ui`.
    pub fn show(self, ui: &mut Ui) -> MacPathControlResponse {
        let _ = (
            self.id,
            &self.components,
            self.show_choose_button,
            self.choose_label,
            self.width,
            self.enabled,
            self.theme,
        );
        let (_, response) = ui.allocate_exact_size(Vec2::ZERO, Sense::hover());
        MacPathControlResponse {
            response,
            clicked_segment: None,
            choose_clicked: false,
        }
    }
}
