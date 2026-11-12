//! Authentic macOS `NSSegmentedControl` primitive.
//!
//! Grounded in Apple macOS Human Interface Guidelines (HIG) and W3C DTCG tokens via [`MacTheme`].
//! Fully data-driven: zero hardcoded metrics or ad-hoc token arithmetic.

mod interaction;
mod item;
mod layout;
mod multi;
mod render;
mod storage;

pub use item::{
    MacSegmentEvent, MacSegmentItem, SegmentContent, SegmentEvent, SegmentIconFn,
    SegmentIconPainter, SegmentItem, SegmentMenuTrigger,
};

use crate::controls::MacControlSize;
use crate::symbols::MacSymbol;
use crate::theme::MacTheme;
use egui::{Rect, Response, Ui};
use std::borrow::Cow;
use storage::{InlineBuf, INLINE_CAP};

/// Authentic macOS segmented pill selector (`NSSegmentedControl` equivalent).
///
/// Supports `CASpring` sliding pill physics, per-segment widths, template-tinted vector icons,
/// per-segment enablement, `isHighlighted` pressed feedback, Sonoma active-pill divider suppression,
/// `AccessKit` (`egui::Role::RadioGroup` / `egui::Role::Toolbar`) semantics, stack-inline storage,
/// and AppKit menu triggers (`PressAndHold`, `PullDown`, `SecondaryClick`, `Keyboard`).
///
/// # Example
/// ```rust
/// use egui_macview::{MacSegmentedControl, MacTheme};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut selected = 1;
/// MacSegmentedControl::new(&["Day", "Week", "Month", "Year"], selected)
///     .accessibility_label("Calendar Scale")
///     .theme(&theme)
///     .show(ui, |i| selected = i);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacSegmentedControl<'a> {
    pub(crate) segments: InlineBuf<SegmentItem<'a>>,
    pub(crate) selected_idx: usize,
    pub(crate) enabled: bool,
    pub(crate) theme: Option<&'a MacTheme>,
    pub(crate) height: Option<f32>,
    pub(crate) control_size: MacControlSize,
    pub(crate) small_square: bool,
    pub(crate) momentary: bool,
    pub(crate) fixed_rect: Option<Rect>,
    pub(crate) id_salt: Option<egui::Id>,
    pub(crate) a11y_label: Option<Cow<'a, str>>,
}

impl<'a> MacSegmentedControl<'a> {
    /// Maximum number of segments stored inline on the stack (`8`) before spilling to heap storage.
    pub const INLINE_SEGMENT_CAPACITY: usize = INLINE_CAP;

    /// Creates a segmented control from borrowed text option labels and currently selected index.
    pub fn new(options: &'a [&'a str], selected_idx: usize) -> Self {
        Self::with_custom_segments(options.iter().map(|&opt| SegmentItem::new(opt)), selected_idx)
    }

    /// Creates a segmented control from any array or iterator of [`SegmentItem`] descriptors.
    pub fn with_custom_segments(
        segments: impl IntoIterator<Item = SegmentItem<'a>>,
        selected_idx: usize,
    ) -> Self {
        Self {
            segments: segments.into_iter().collect(),
            selected_idx,
            enabled: true,
            theme: None,
            height: None,
            control_size: MacControlSize::Regular,
            small_square: false,
            momentary: false,
            fixed_rect: None,
            id_salt: None,
            a11y_label: None,
        }
    }

    /// Creates a momentary segmented action strip (`NSSegmentedSwitchTrackingMomentary`) with no persistent active pill.
    pub fn momentary(segments: impl IntoIterator<Item = SegmentItem<'a>>) -> Self {
        let mut ctrl = Self::with_custom_segments(segments, 0);
        ctrl.momentary = true;
        ctrl
    }

    /// Creates a segmented control from a slice of [`MacSymbol`] icons.
    pub fn with_icons(symbols: &'a [MacSymbol], selected_idx: usize) -> Self {
        Self::with_custom_segments(symbols.iter().map(|&sym| SegmentItem::from_symbol(sym)), selected_idx)
    }

    /// Returns `true` when the segment descriptors reside inline on the stack without heap allocation.
    #[inline]
    pub const fn is_inline_allocated(&self) -> bool {
        self.segments.is_inline()
    }

    /// Explicitly binds an ID salt so spring physics and gesture state stay invariant across conditional peer layouts.
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = Some(egui::Id::NULL.with(salt));
        self
    }

    /// Sets the `NSAccessibility` / `AccessKit` group label announced by assistive technologies.
    pub fn accessibility_label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    /// Enables or disables the entire segmented control.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects an explicit [`MacTheme`] reference.
    pub const fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Overrides control height in points (defaults to size-tier segmented height token).
    pub const fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the AppKit control size (`Regular` `22pt`, `Small` `18pt`, or `Mini` `15pt`).
    pub const fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Configures AppKit `NSSegmentedStyleSmallSquare` (`2.0pt` corner radius and full-height dividers).
    pub const fn small_square(mut self) -> Self {
        self.small_square = true;
        self
    }

    /// Pins the segmented control to an exact NIB rectangle.
    pub const fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Renders a multi-select segmented strip (`NSSegmentedSwitchTrackingSelectAny`).
    pub fn show_multi(self, ui: &mut Ui, active: &mut [bool]) -> Option<usize> {
        multi::render_multi(self, ui, active)
    }

    /// Renders the segmented control and invokes `on_select` when a primary selection occurs.
    pub fn show(self, ui: &mut Ui, mut on_select: impl FnMut(usize)) -> Response {
        self.show_with_events(ui, |ev| {
            if !ev.opens_menu() {
                on_select(ev.index);
            }
        })
    }

    /// Renders the segmented control and dispatches [`SegmentEvent`] values for both selection and menu triggers.
    pub fn show_with_events(self, ui: &mut Ui, on_event: impl FnMut(SegmentEvent)) -> Response {
        render::render_segmented(self, ui, on_event)
    }

    /// Renders a momentary segmented strip (`NSSegmentedSwitchTrackingMomentary`) and invokes `on_action` on click/activation.
    pub fn show_momentary(mut self, ui: &mut Ui, mut on_action: impl FnMut(usize)) -> Response {
        self.momentary = true;
        self.show_with_events(ui, |ev| {
            if !ev.opens_menu() {
                on_action(ev.index);
            }
        })
    }

    /// Renders a momentary segmented strip (`NSSegmentedSwitchTrackingMomentary`) and dispatches both click and menu [`SegmentEvent`]s.
    pub fn show_momentary_with_events(
        mut self,
        ui: &mut Ui,
        on_event: impl FnMut(SegmentEvent),
    ) -> Response {
        self.momentary = true;
        render::render_segmented(self, ui, on_event)
    }
}
