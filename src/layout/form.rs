//! Authentic macOS AppKit Form Layout (`MacForm`).
//!
//! Compact wrapper around `egui::Grid` with AppKit token spacing,
//! trailing-aligned labels (column 0), and leading-aligned controls (column 1).

use crate::theme::MacTheme;
use egui::{vec2, Align, IdSalt, Layout, Response, Ui, Vec2, WidgetText};

/// Authentic macOS AppKit 2-column Form container.
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacForm, MacTheme, MacToggle};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let mut hardware_accel = true;
///
/// MacForm::new("preferences_form")
///     .theme(&theme)
///     .show(ui, |form| {
///         form.row_label("Hardware Acceleration:", |ui| {
///             ui.add(MacToggle::new(&mut hardware_accel).theme(&theme));
///         });
///     });
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacForm<'a> {
    id_salt: IdSalt,
    theme: Option<&'a MacTheme>,
    spacing: Option<Vec2>,
    min_col_width: Option<f32>,
}

impl<'a> MacForm<'a> {
    /// Creates a new `MacForm` with the specified id salt.
    pub fn new(id_salt: impl egui::AsIdSalt) -> Self {
        Self { id_salt: IdSalt::new(id_salt), theme: None, spacing: None, min_col_width: None }
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

    /// Overrides the minimum column width in points.
    pub fn min_col_width(mut self, min_col_width: f32) -> Self {
        self.min_col_width = Some(min_col_width);
        self
    }

    /// Renders the form contents with trailing labels and leading controls.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut MacFormUi<'_>) -> R) -> R {
        let spacing = self.spacing.unwrap_or_else(|| {
            let s = self
                .theme
                .map(|t| t.stack_view().default_spacing.points())
                .unwrap_or_else(|| MacTheme::from_ctx(ui.ctx()).stack_view().default_spacing.points());
            vec2(s, s)
        });

        let mut grid = egui::Grid::new(self.id_salt).num_columns(2).spacing(spacing);
        if let Some(min_w) = self.min_col_width {
            grid = grid.min_col_width(min_w);
        }
        grid.show(ui, |grid_ui| {
            let mut form_ui = MacFormUi { ui: grid_ui };
            add_contents(&mut form_ui)
        })
        .inner
    }
}

/// Form builder context for adding rows with trailing labels and leading controls.
pub struct MacFormUi<'a> {
    /// Underlying egui `Ui` instance.
    pub ui: &'a mut Ui,
}

impl<'a> MacFormUi<'a> {
    /// Adds a row with a trailing-aligned label (col 0) and leading-aligned control (col 1).
    /// Automatically paints a subtle full-width horizontal highlight band (`MacTheme::focused_row_fill`)
    /// when any child control in the row holds keyboard focus (`REQ-015-02`).
    pub fn row<R1, R2>(&mut self, label: impl FnOnce(&mut Ui) -> R1, control: impl FnOnce(&mut Ui) -> R2) -> (R1, R2) {
        let bg_idx = self.ui.painter().add(egui::Shape::Noop);
        let l_out = self.ui.with_layout(Layout::right_to_left(Align::Center), label);
        let c_out = self.ui.with_layout(Layout::left_to_right(Align::Center), control);
        let row_rect = l_out.response.rect.union(c_out.response.rect);
        if let Some(focused_id) = self.ui.ctx().memory(|m| m.focused()) {
            let row_has_focus = self.ui.ctx().read_response(focused_id).is_some_and(|r| r.rect.intersects(row_rect));
            if row_has_focus {
                let theme = MacTheme::from_ctx(self.ui.ctx());
                let full_x = self.ui.max_rect();
                let band_rect = egui::Rect::from_min_max(
                    egui::pos2(full_x.min.x.min(row_rect.min.x), row_rect.min.y - 1.0),
                    egui::pos2(full_x.max.x.max(row_rect.max.x), row_rect.max.y + 1.0),
                );
                self.ui.painter().set(bg_idx, egui::Shape::rect_filled(band_rect, egui::CornerRadius::same(2), theme.focused_row_fill()));
            }
        }
        self.ui.end_row();
        (l_out.inner, c_out.inner)
    }

    /// Convenience helper adding a text label and control row.
    pub fn row_label<R>(&mut self, label_text: impl Into<WidgetText>, control: impl FnOnce(&mut Ui) -> R) -> (Response, R) {
        let text = label_text.into();
        self.row(|ui| ui.label(text), control)
    }
}
