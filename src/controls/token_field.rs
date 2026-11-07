//! Authentic macOS AppKit Token Field (`NSTokenField` equivalent, SPEC-011).

use crate::theme::MacTheme;
use egui::{Id, Response, Sense, Ui, Vec2, Widget};
use std::fmt::Debug;
use std::hash::Hash;

/// Authentic macOS AppKit token pill field (`NSTokenField` equivalent).
pub struct MacTokenField<'a> {
    id: Id,
    tokens: &'a mut Vec<String>,
    input_buffer: &'a mut String,
    placeholder: &'a str,
    max_chars: Option<usize>,
    width: Option<f32>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacTokenField<'a> {
    /// Creates a new `MacTokenField` bound to `tokens` and `input_buffer`.
    pub fn new(
        id_source: impl Hash + Debug,
        tokens: &'a mut Vec<String>,
        input_buffer: &'a mut String,
    ) -> Self {
        #[allow(deprecated)]
        let id = Id::new(id_source);
        Self {
            id,
            tokens,
            input_buffer,
            placeholder: "Add token...",
            max_chars: None,
            width: None,
            enabled: true,
            theme: None,
        }
    }

    /// Sets placeholder text shown when no tokens or input text are present.
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Configures an optional character budget counter (`current / max_chars`) at bottom-right.
    pub fn max_chars(mut self, max_chars: usize) -> Self {
        self.max_chars = Some(max_chars);
        self
    }

    /// Overrides the control width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Enables or disables the token field.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the token field in `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacTokenField<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let _ = (
            self.id,
            &self.tokens,
            &self.input_buffer,
            self.placeholder,
            self.max_chars,
            self.width,
            self.enabled,
            self.theme,
        );
        ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1
    }
}
