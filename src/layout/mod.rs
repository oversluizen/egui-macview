//! Layout engines: `MacFixedView` (absolute pt), `MacSplitView` (NSSplitView),
//! `MacScrollView` (NSScrollView), `MacStackView` (NSStackView), and `MacForm` (NSForm).

pub mod form;
pub mod scroll_view;
pub mod scrollbar;
pub mod split_view;
pub mod stack_view;

pub use form::{MacForm, MacFormUi};
pub use scroll_view::{MacScrollView, MacScrollViewOutput};
pub use scrollbar::render_overlay_thumb;
pub use split_view::{MacSplitOrientation, MacSplitView};
pub use stack_view::{
    MacStackAlignment, MacStackOrientation, MacStackUi, MacStackView, MacStackViewOutput,
};

use egui::{Pos2, Rect, Response, Sense, Ui, Vec2};

/// Snaps a logical coordinate to the nearest physical hardware pixel boundary
/// according to the Subpixel Snap Criterion (SPEC-001 §2.1 A):
///
/// $$x_{\text{pixel}} = \frac{\lfloor x_{\text{logical}} \cdot s + 0.5 \rfloor}{s}$$
///
/// # Example
///
/// ```rust
/// use egui_macview::layout::subpixel_snap;
///
/// let snapped = subpixel_snap(10.333, 2.0); // 200% scaling
/// assert_eq!(snapped, 10.5);
/// ```
#[inline]
pub fn subpixel_snap(logical: f32, scale_factor: f32) -> f32 {
    let s = scale_factor.max(f32::EPSILON);
    (logical * s + 0.5).floor() / s
}

/// Snaps a 2D logical point `Pos2` to hardware pixel boundaries.
///
/// # Example
///
/// ```rust
/// use egui_macview::layout::subpixel_snap_pos;
/// use egui::Pos2;
///
/// let snapped = subpixel_snap_pos(Pos2::new(10.25, 20.75), 1.0);
/// assert_eq!(snapped, Pos2::new(10.0, 21.0));
/// ```
#[inline]
pub fn subpixel_snap_pos(pos: Pos2, scale_factor: f32) -> Pos2 {
    Pos2::new(
        subpixel_snap(pos.x, scale_factor),
        subpixel_snap(pos.y, scale_factor),
    )
}

/// Snaps a logical `Rect` to hardware pixel boundaries.
///
/// # Example
///
/// ```rust
/// use egui_macview::layout::subpixel_snap_rect;
/// use egui::{Pos2, Rect};
///
/// let rect = Rect::from_min_max(Pos2::new(0.4, 0.4), Pos2::new(10.6, 20.6));
/// let snapped = subpixel_snap_rect(rect, 1.0);
/// assert_eq!(snapped.min, Pos2::new(0.0, 0.0));
/// assert_eq!(snapped.max, Pos2::new(11.0, 21.0));
/// ```
#[inline]
pub fn subpixel_snap_rect(rect: Rect, scale_factor: f32) -> Rect {
    let min = subpixel_snap_pos(rect.min, scale_factor);
    let max = subpixel_snap_pos(rect.max, scale_factor);
    Rect::from_min_max(min, max)
}

/// A logical geometry frame in AppKit points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacFrame {
    /// Top-left origin point in logical points.
    pub origin: Pos2,
    /// Width and height dimensions in logical points.
    pub size: Vec2,
}

impl MacFrame {
    /// Constructs a `MacFrame` with the given origin `(x, y)` and size `(w, h)`.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            origin: Pos2::new(x, y),
            size: Vec2::new(w, h),
        }
    }

    /// Converts frame into an `egui::Rect`.
    pub fn to_rect(&self) -> Rect {
        Rect::from_min_size(self.origin, self.size)
    }
}

/// Absolute pt coordinate container based on macOS NIB geometries.
pub struct MacFixedView<'a> {
    /// Underlying egui `Ui` context.
    pub ui: &'a mut Ui,
    /// Absolute bounding rectangle of the fixed container.
    pub container_rect: Rect,
    subview_count: usize,
}

impl<'a> MacFixedView<'a> {
    /// Creates a new `MacFixedView` wrapping `ui` and bounded by `container_rect`.
    pub fn new(ui: &'a mut Ui, container_rect: Rect) -> Self {
        Self {
            ui,
            container_rect,
            subview_count: 0,
        }
    }

    /// Allocates an exact subview at relative NIB coordinates (dx, dy, w, h).
    pub fn subview_rect(&self, rel_origin: Pos2, size: Vec2) -> Rect {
        Rect::from_min_size(
            Pos2::new(
                self.container_rect.left() + rel_origin.x,
                self.container_rect.top() + rel_origin.y,
            ),
            size,
        )
    }

    /// Executes a closure inside a subview allocated at relative NIB coordinates.
    pub fn subview<R>(
        &mut self,
        rel_origin: Pos2,
        size: Vec2,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        let idx = self.subview_count;
        self.subview_count += 1;
        let rect = self.subview_rect(rel_origin, size);
        let mut child = self.ui.new_child(
            egui::UiBuilder::new()
                .id_salt(("fixed_subview", idx))
                .max_rect(rect),
        );
        child.set_clip_rect(rect.intersect(self.ui.clip_rect()));
        add_contents(&mut child)
    }

    /// Allocates the container rect response after placing all subviews.
    pub fn finish(self) -> Response {
        self.ui.allocate_rect(self.container_rect, Sense::hover())
    }
}

/// Deterministic horizontal row container with fixed height and spacing budget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacRow {
    /// Vertical offset of the row in container coordinates.
    pub y_offset: f32,
    /// Fixed row height in points.
    pub height: f32,
    /// Horizontal cursor advancing as cells are allocated.
    pub cursor_x: f32,
    /// Spacing between adjacent cells in points.
    pub item_spacing: f32,
    /// Total available container width in points.
    pub container_width: f32,
}

impl MacRow {
    /// Creates a new `MacRow` container.
    pub fn new(y_offset: f32, height: f32, container_width: f32, item_spacing: f32) -> Self {
        Self {
            y_offset,
            height,
            cursor_x: 0.0,
            item_spacing,
            container_width,
        }
    }

    /// Allocates next cell of width `width`, advancing cursor with spacing.
    /// Returns relative bounds `(x, y, width, height)`.
    pub fn allocate_cell(&mut self, width: f32) -> (f32, f32, f32, f32) {
        let x = self.cursor_x;
        let y = self.y_offset;
        let h = self.height;
        self.cursor_x += width + self.item_spacing;
        (x, y, width, h)
    }

    /// Returns remaining horizontal budget in points.
    pub fn remaining_width(&self) -> f32 {
        (self.container_width - self.cursor_x).max(0.0)
    }
}
