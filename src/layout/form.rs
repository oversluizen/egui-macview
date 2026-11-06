//! Authentic macOS AppKit Form Layout (`MacForm`).
//!
//! Compact wrapper around `egui::Grid` with AppKit token spacing,
//! trailing-aligned labels (column 0), and leading-aligned controls (column 1).

use crate::theme::MacTheme;
use egui::{vec2, Align, IdSalt, Layout, Response, Ui, Vec2, WidgetText};

/// Authentic macOS AppKit 2-column Form container.
pub struct MacForm<'a> {
    id_salt: IdSalt,
    theme: Option<&'a MacTheme>,
    spacing: Option<Vec2>,
}

impl<'a> MacForm<'a> {
    /// Creates a new `MacForm` with the specified id salt.
    pub fn new(id_salt: impl egui::AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            theme: None,
            spacing: None,
        }
    }

    /// Sets the dynamic theme to derive AppKit token spacing.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Overrides inter-item spacing between columns and rows in points.
    pub fn spacing(mut self, spacing: impl Into<Vec2>) -> Self {
        self.spacing = Some(spacing.into());
        self
    }

    /// Renders the form contents with trailing labels and leading controls.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut MacFormUi<'_>) -> R) -> R {
        let spacing = self.spacing.unwrap_or_else(|| {
            let s = self
                .theme
                .map(|t| t.stack_view().default_spacing.points())
                .unwrap_or_else(|| ui.spacing().item_spacing.x);
            vec2(s, s)
        });

        let mut result = None;
        egui::Grid::new(self.id_salt)
            .num_columns(2)
            .spacing(spacing)
            .show(ui, |grid_ui| {
                let mut form_ui = MacFormUi { ui: grid_ui };
                result = Some(add_contents(&mut form_ui));
            });
        result.expect("MacForm grid rendered")
    }
}

/// Form builder context for adding rows with trailing labels and leading controls.
pub struct MacFormUi<'a> {
    /// Underlying egui `Ui` instance.
    pub ui: &'a mut Ui,
}

impl<'a> MacFormUi<'a> {
    /// Adds a row with a trailing-aligned label (col 0) and leading-aligned control (col 1).
    pub fn row<R1, R2>(
        &mut self,
        label: impl FnOnce(&mut Ui) -> R1,
        control: impl FnOnce(&mut Ui) -> R2,
    ) -> (R1, R2) {
        let r1 = self
            .ui
            .with_layout(Layout::right_to_left(Align::Center), label)
            .inner;
        let r2 = self
            .ui
            .with_layout(Layout::left_to_right(Align::Center), control)
            .inner;
        self.ui.end_row();
        (r1, r2)
    }

    /// Convenience helper adding a text label and control row.
    pub fn row_label<R>(
        &mut self,
        label_text: impl Into<WidgetText>,
        control: impl FnOnce(&mut Ui) -> R,
    ) -> (Response, R) {
        let text = label_text.into();
        self.row(|ui| ui.label(text), control)
    }
}
