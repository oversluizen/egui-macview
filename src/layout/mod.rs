//! Layout engines: `MacFixedView` (absolute pt) and `MacFlexView` (springs/struts).

use egui::{Pos2, Rect, Ui, Vec2};

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
