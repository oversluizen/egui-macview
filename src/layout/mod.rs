//! Layout engines: `MacFixedView` (absolute pt) and `MacFlexView` (springs/struts).

use egui::{Pos2, Rect, Ui, Vec2};

/// Snaps a logical coordinate to the nearest physical hardware pixel boundary
/// according to the Subpixel Snap Criterion (SPEC-001 §2.1 A):
///
/// $$x_{\text{pixel}} = \frac{\lfloor x_{\text{logical}} \cdot s + 0.5 \rfloor}{s}$$
#[inline]
pub fn subpixel_snap(logical: f32, scale_factor: f32) -> f32 {
    let s = scale_factor.max(0.0001);
    (logical * s + 0.5).floor() / s
}

/// Snaps a 2D logical point `Pos2` to hardware pixel boundaries.
#[inline]
pub fn subpixel_snap_pos(pos: Pos2, scale_factor: f32) -> Pos2 {
    Pos2::new(
        subpixel_snap(pos.x, scale_factor),
        subpixel_snap(pos.y, scale_factor),
    )
}

/// Snaps a logical `Rect` to hardware pixel boundaries.
#[inline]
pub fn subpixel_snap_rect(rect: Rect, scale_factor: f32) -> Rect {
    let min = subpixel_snap_pos(rect.min, scale_factor);
    let max = subpixel_snap_pos(rect.max, scale_factor);
    Rect::from_min_max(min, max)
}

/// A logical geometry frame in AppKit points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacFrame {
    pub origin: Pos2,
    pub size: Vec2,
}

impl MacFrame {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            origin: Pos2::new(x, y),
            size: Vec2::new(w, h),
        }
    }

    pub fn to_rect(&self) -> Rect {
        Rect::from_min_size(self.origin, self.size)
    }
}

/// Absolute pt coordinate container based on macOS NIB geometries.
pub struct MacFixedView<'a> {
    pub ui: &'a mut Ui,
    pub container_rect: Rect,
}

impl<'a> MacFixedView<'a> {
    pub fn new(ui: &'a mut Ui, container_rect: Rect) -> Self {
        Self { ui, container_rect }
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
}

/// Flexible container supporting auto-resizing masks and fill splits.
pub struct MacFlexView<'a> {
    pub ui: &'a mut Ui,
    pub available_rect: Rect,
}

impl<'a> MacFlexView<'a> {
    pub fn new(ui: &'a mut Ui, available_rect: Rect) -> Self {
        Self { ui, available_rect }
    }
}

/// Deterministic horizontal row container with fixed height and spacing budget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacRow {
    pub y_offset: f32,
    pub height: f32,
    pub cursor_x: f32,
    pub item_spacing: f32,
    pub container_width: f32,
}

impl MacRow {
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
