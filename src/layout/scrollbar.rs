//! Authentic macOS Floating Overlay Scrollbar Thumb primitives.
//!
//! Handles subpixel-snapped geometric computation, corner clearance isolation,
//! interaction tracking, and dynamic opacity rendering for floating overlay scrollbars.

use crate::layout::subpixel_snap_rect;
use egui::{Color32, CornerRadius, Id, Pos2, Rect, Sense, Stroke, StrokeKind, Ui};

/// Parameters for rendering an authentic floating overlay scrollbar thumb.
#[allow(clippy::too_many_arguments)]
pub fn render_overlay_thumb(
    ui: &mut Ui,
    painter: &egui::Painter,
    id: Id,
    viewport: Rect,
    view_span: f32,
    content_span: f32,
    lead: f32,
    trail: f32,
    inset: f32,
    thickness: f32,
    radius: f32,
    min_len: f32,
    min_visible_len: f32,
    offset: &mut f32,
    max_scroll: f32,
    is_vertical: bool,
    alpha: f32,
    color_fn: &impl Fn(bool, bool) -> Color32,
    active_id: Id,
    now: f64,
) -> bool {
    let ratio = (view_span / content_span).clamp(0.0, 1.0);
    let avail = (view_span - lead - trail).max(0.0);
    let thumb_len = (avail * ratio).max(min_len.min(avail)).min(avail);
    let travel = (avail - thumb_len).max(0.0);
    let frac = if max_scroll > 0.0 {
        *offset / max_scroll
    } else {
        0.0
    };

    if thumb_len < min_visible_len {
        return false;
    }

    let raw = if is_vertical {
        let top = viewport.top() + lead + frac * travel;
        let right = viewport.right() - inset;
        Rect::from_min_max(
            Pos2::new(right - thickness, top),
            Pos2::new(right, top + thumb_len),
        )
    } else {
        let left = viewport.left() + lead + frac * travel;
        let bottom = viewport.bottom() - inset;
        Rect::from_min_max(
            Pos2::new(left, bottom - thickness),
            Pos2::new(left + thumb_len, bottom),
        )
    };
    let thumb_rect = subpixel_snap_rect(raw, ui.pixels_per_point());

    let resp = ui.interact(thumb_rect, id, Sense::click_and_drag());
    if resp.hovered() || resp.dragged() {
        ui.data_mut(|d| d.insert_temp(active_id, now));
        ui.ctx().request_repaint();
    }
    let mut dragged = false;
    if resp.dragged() && travel > 0.0 {
        let dy = if is_vertical {
            resp.drag_delta().y
        } else {
            resp.drag_delta().x
        };
        *offset = (*offset + dy * (max_scroll / travel)).clamp(0.0, max_scroll);
        dragged = true;
    }

    if alpha > f32::EPSILON {
        let r = radius.min(thumb_len * 0.5).min(thickness * 0.5);
        painter.rect(
            thumb_rect,
            CornerRadius::same(r as u8),
            color_fn(resp.hovered(), resp.dragged()),
            Stroke::NONE,
            StrokeKind::Inside,
        );
    }

    dragged
}
