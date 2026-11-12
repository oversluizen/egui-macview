//! Authentic macOS `NSSegmentedControl` segment descriptors and menu trigger events.

use crate::symbols::MacSymbol;
use egui::{Color32, Painter, Rect};
use std::borrow::Cow;
use std::sync::Arc;

/// Shared callback signature for stateful template-tinted custom vector segment icons.
pub type SegmentIconPainter<'a> = Arc<dyn Fn(&Painter, Rect, Color32) + 'a>;

/// Zero-allocation function pointer signature for stateless template-tinted vector segment icons.
pub type SegmentIconFn = fn(&Painter, Rect, Color32);

/// Visual payload rendered inside an individual segment of [`super::MacSegmentedControl`].
#[derive(Clone)]
pub enum SegmentContent<'a> {
    /// Borrowed or owned text label laid out with Apple HIG optical tracking.
    Text(Cow<'a, str>),
    /// Built-in monochrome [`MacSymbol`] glyph.
    Symbol(MacSymbol),
    /// Stateless function pointer icon (`fn(&Painter, Rect, Color32)`) with zero heap allocation.
    IconFn(SegmentIconFn),
    /// Stateful closure icon (`Arc<dyn Fn(&Painter, Rect, Color32) + 'a>`).
    Custom(SegmentIconPainter<'a>),
}

/// Per-segment configuration matching Apple AppKit `NSSegmentedControl` segment slots.
#[derive(Clone)]
pub struct SegmentItem<'a> {
    pub(crate) content: SegmentContent<'a>,
    pub(crate) width: Option<f32>,
    pub(crate) enabled: bool,
    pub(crate) has_menu: bool,
    pub(crate) shows_menu_indicator: bool,
    pub(crate) tooltip: Option<Cow<'a, str>>,
}

impl<'a> SegmentItem<'a> {
    const fn with_content(content: SegmentContent<'a>) -> Self {
        Self {
            content,
            width: None,
            enabled: true,
            has_menu: false,
            shows_menu_indicator: false,
            tooltip: None,
        }
    }

    pub(crate) fn accessible_cow(&self) -> Option<Cow<'a, str>> {
        match &self.content {
            SegmentContent::Text(s) if !s.is_empty() => Some(s.clone()),
            _ => self.tooltip.clone(),
        }
    }

    /// Creates an enabled segment displaying a borrowed or owned text label.
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self::with_content(SegmentContent::Text(label.into()))
    }

    /// Creates an enabled segment displaying a built-in [`MacSymbol`].
    pub const fn from_symbol(symbol: MacSymbol) -> Self {
        Self::with_content(SegmentContent::Symbol(symbol))
    }

    /// Creates an enabled segment rendered by a zero-allocation stateless icon function.
    pub const fn from_icon_fn(painter: SegmentIconFn) -> Self {
        Self::with_content(SegmentContent::IconFn(painter))
    }

    /// Creates an enabled segment rendered by a custom template-tinted vector closure.
    pub fn from_icon(painter: impl Fn(&Painter, Rect, Color32) + 'a) -> Self {
        Self::with_content(SegmentContent::Custom(Arc::new(painter)))
    }

    /// Assigns an explicit width in points (`setWidth(_:forSegment:)`).
    pub const fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Enables or disables interaction and dims tint for this segment (`setEnabled(_:forSegment:)`).
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Attaches or detaches menu trigger behavior (`setMenu(_:forSegment:)`) without altering indicator visibility.
    pub const fn with_menu(mut self, has_menu: bool) -> Self {
        self.has_menu = has_menu;
        self
    }

    /// Controls whether the trailing downward chevron indicator is drawn (`setShowsMenuIndicator(_:forSegment:)`).
    pub const fn with_shows_menu_indicator(mut self, shows: bool) -> Self {
        self.shows_menu_indicator = shows;
        self
    }

    /// Convenience builder setting both [`Self::with_menu`] and [`Self::with_shows_menu_indicator`].
    pub const fn with_submenu(mut self, enabled: bool) -> Self {
        self.has_menu = enabled;
        self.shows_menu_indicator = enabled;
        self
    }

    /// Attaches a hover tooltip (`setToolTip(_:forSegment:)`), borrowing `&'a str` without allocation.
    pub fn with_tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

impl<'a> From<&'a str> for SegmentItem<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl<'a> From<MacSymbol> for SegmentItem<'a> {
    fn from(sym: MacSymbol) -> Self {
        Self::from_symbol(sym)
    }
}

/// Gesture that opened a segment's attached menu (`NSSegmentedCell` menu activation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentMenuTrigger {
    /// Primary mouse button held past the `menu_press_hold_delay` token (`0.25s`).
    PressAndHold,
    /// Pointer dragged downward past the `menu_drag_threshold` token (`4.0pt`).
    PullDown,
    /// Right-click or `Control` + primary-click on a menu-bearing segment.
    SecondaryClick,
    /// Keyboard activation (`ArrowDown` or `Shift+F10`) while focused on a menu-bearing segment.
    Keyboard,
}

/// Interaction event emitted by [`super::MacSegmentedControl::show_with_events`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentEvent {
    /// Zero-based index of the interacted segment.
    pub index: usize,
    /// `None` when selecting the segment; `Some(trigger)` when opening its attached menu.
    pub menu_trigger: Option<SegmentMenuTrigger>,
    /// Screen-space bounding rectangle of the target segment for anchoring a menu or popover.
    pub rect: Rect,
}

impl SegmentEvent {
    /// Returns `true` when this event requests opening the segment's attached menu rather than selecting it.
    #[inline]
    pub const fn opens_menu(&self) -> bool {
        self.menu_trigger.is_some()
    }
}

/// Type alias for [`SegmentItem`].
pub type MacSegmentItem<'a> = SegmentItem<'a>;

/// Type alias for [`SegmentEvent`].
pub type MacSegmentEvent = SegmentEvent;
