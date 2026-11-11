//! Authentic macOS NSTextFieldCell primitive with Tab-chain and Transactional Buffer.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].
//! Fully data-driven: zero hardcoded colors, radiuses, or padding metrics.

mod editing;

use crate::theme::MacTheme;
use egui::{Align, Id, Rect, Response, Ui};
use std::ops::RangeInclusive;

/// Standard AppKit `NSControlSize` variants for `MacInputCell` and AppKit NIB controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacControlSize {
    /// `NSControlSizeRegular` (`22pt` height, `13pt` font).
    #[default]
    Regular,
    /// `NSControlSizeSmall` (`19pt` height, `11pt` font).
    Small,
    /// `NSControlSizeMini` (`15pt`–`16pt` height, `9pt` font).
    Mini,
    /// `NSControlSizeLarge` (`28pt` height, `16pt` font).
    Large,
    /// Dense inspector / HUD infobox cell (`18pt` height, `11pt` font).
    CompactInfobox,
}

/// Authentic macOS text entry cell (AppKit `NSTextFieldCell` style).
///
/// Features transactional input buffering, Escape rollback, Return confirmation,
/// deterministic Tab-chain cyclic focus navigation, automatic NIB height-based
/// `MacControlSize` inference, AppKit baseline centering, and horizontal text alignment.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacInputCell, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut text = String::from("Hello macOS");
///
/// MacInputCell::new("input_demo", &mut text)
///     .theme(&theme)
///     .placeholder("Type here...")
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacInputCell<'a> {
    pub(super) id: Id,
    pub(super) value: &'a mut String,
    pub(super) theme: Option<&'a MacTheme>,
    pub(super) width: Option<f32>,
    pub(super) height: Option<f32>,
    pub(super) font_size: Option<f32>,
    pub(super) corner_radius: Option<f32>,
    pub(super) control_size: MacControlSize,
    pub(super) explicit_control_size: bool,
    pub(super) fixed_rect: Option<Rect>,
    pub(super) text_align: Align,
    pub(super) enabled: bool,
    pub(super) next_focus_id: Option<Id>,
    pub(super) prev_focus_id: Option<Id>,
    pub(super) placeholder: Option<&'a str>,
    pub(super) numeric_config: Option<(RangeInclusive<f64>, f64)>,
}

impl<'a> MacInputCell<'a> {
    /// Creates a new `MacInputCell` bound to `value` and identified by `id_source`.
    pub fn new(id_source: impl std::hash::Hash + std::fmt::Debug, value: &'a mut String) -> Self {
        let id = Id::NULL.with(id_source);
        Self::with_id(id, value)
    }

    /// Creates a cell directly with an existing `egui::Id` without double-hashing.
    pub fn with_id(id: Id, value: &'a mut String) -> Self {
        Self {
            id,
            value,
            theme: None,
            width: None,
            height: None,
            font_size: None,
            corner_radius: None,
            control_size: MacControlSize::Regular,
            explicit_control_size: false,
            fixed_rect: None,
            text_align: Align::Min,
            enabled: true,
            next_focus_id: None,
            prev_focus_id: None,
            placeholder: None,
            numeric_config: None,
        }
    }

    /// Configures numeric arrow-key stepping (`ArrowUp`/`ArrowDown` by `±step`,
    /// `Shift` by `±10 * step`, `Cmd`/`Ctrl` by `±100 * step`, clamped to `range`).
    pub fn numeric(mut self, range: RangeInclusive<f64>, step: f64) -> Self {
        self.numeric_config = Some((range, step));
        self
    }

    /// Sets the AppKit control size preset (`Regular`, `Small`, `Mini`, `Large`, `CompactInfobox`).
    pub fn control_size(mut self, control_size: MacControlSize) -> Self {
        self.control_size = control_size;
        self.explicit_control_size = true;
        self
    }

    /// Pins the input cell to an exact NIB rectangle.
    ///
    /// When `.control_size(...)` was not explicitly overridden, `MacControlSize` is automatically
    /// inferred from `rect.height()` (`>= 21.0pt` -> `Regular`, `18.5..21.0pt` -> `Small`,
    /// `17.0..18.5pt` -> `CompactInfobox`, `< 17.0pt` -> `Mini`).
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Sets horizontal text alignment inside the cell (`Align::Min` left or `Align::Max` right).
    pub fn text_align(mut self, align: Align) -> Self {
        self.text_align = align;
        self
    }

    /// Returns the resolved [`MacControlSize`] for this cell (accounting for `.fixed_rect(rect)` auto-detection).
    pub fn resolved_control_size(&self) -> MacControlSize {
        if self.explicit_control_size {
            return self.control_size;
        }
        let Some(rect) = self.fixed_rect else {
            return self.control_size;
        };
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let tf = theme_cow.text_field();
        let h = rect.height();
        if h >= tf.auto_size_regular_min_height.points() {
            MacControlSize::Regular
        } else if h >= tf.auto_size_small_min_height.points() {
            MacControlSize::Small
        } else if h >= tf.auto_size_compact_min_height.points() {
            MacControlSize::CompactInfobox
        } else {
            MacControlSize::Mini
        }
    }

    /// Enables or disables user interaction on the input cell.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Overrides cell width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides cell height in points (defaults to token text field height).
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Overrides font size in points.
    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = Some(font_size);
        self
    }

    /// Overrides corner radius in points.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Sets placeholder text displayed when the text field is empty and unfocused.
    pub fn placeholder(mut self, text: &'a str) -> Self {
        self.placeholder = Some(text);
        self
    }

    /// Sets the deterministic Tab navigation target.
    pub fn next_focus(mut self, id: Id) -> Self {
        self.next_focus_id = Some(id);
        self
    }

    /// Sets the deterministic Shift+Tab navigation target.
    pub fn prev_focus(mut self, id: Id) -> Self {
        self.prev_focus_id = Some(id);
        self
    }

    /// Renders the input cell into `ui` and handles keyboard/focus interaction.
    pub fn show(self, ui: &mut Ui) -> Response {
        editing::show_input_cell(self, ui)
    }
}
