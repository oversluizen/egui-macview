//! Multi-select segmented control presentation (`NSSegmentedSwitchTrackingSelectAny`).

use super::item::{SegmentContent, SegmentItem};
use super::layout::compute_segmented_layout;
use super::storage::InlineBuf;
use super::MacSegmentedControl;
use crate::controls::MacControlSize;
use crate::theme::{alpha_composite, MacTheme};
use egui::{
    Align2, CornerRadius, Key, Modifiers, Pos2, Response, Role, Sense, Stroke, StrokeKind, Ui,
    WidgetInfo,
};

fn handle_multi_keyboard(
    ui: &mut Ui,
    response: &mut Response,
    segments: &InlineBuf<SegmentItem<'_>>,
    active: &mut [bool],
) -> Option<usize> {
    if !response.has_focus() || segments.is_empty() {
        return None;
    }
    let count = segments.len();
    let cursor_id = response.id.with("seg_multi_cursor");
    let mut cursor = ui
        .ctx()
        .data(|d| d.get_temp::<usize>(cursor_id))
        .unwrap_or(0)
        .min(count.saturating_sub(1));

    if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight)) {
        if let Some(next) = ((cursor + 1)..count).find(|&j| segments.get(j).is_some_and(|s| s.enabled)) {
            ui.ctx().data_mut(|d| d.insert_temp(cursor_id, next));
        }
        return None;
    }
    if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft)) {
        if let Some(prev) = (0..cursor).rev().find(|&j| segments.get(j).is_some_and(|s| s.enabled)) {
            ui.ctx().data_mut(|d| d.insert_temp(cursor_id, prev));
        }
        return None;
    }
    if ui.input_mut(|i| {
        i.consume_key(Modifiers::NONE, Key::Space) || i.consume_key(Modifiers::NONE, Key::Enter)
    }) {
        if !segments.get(cursor).is_some_and(|s| s.enabled) {
            cursor = (0..count).find(|&j| segments.get(j).is_some_and(|s| s.enabled))?;
        }
        if let Some(slot) = active.get_mut(cursor) {
            *slot = !*slot;
            response.mark_changed();
            return Some(cursor);
        }
    }
    None
}

pub(super) fn render_multi(
    control: MacSegmentedControl<'_>,
    ui: &mut Ui,
    active: &mut [bool],
) -> Option<usize> {
    let theme_cow = MacTheme::resolve(control.theme, ui.ctx());
    let theme: &MacTheme = &theme_cow;
    let seg = theme.segmented();
    let (tier, font_sz) = match control.control_size {
        MacControlSize::Regular | MacControlSize::Large => (&seg.regular, seg.font_size.points()),
        MacControlSize::Small | MacControlSize::CompactInfobox => {
            (&seg.small, seg.small_font_size.points())
        }
        MacControlSize::Mini => (&seg.mini, seg.mini_font_size.points()),
    };
    let height = control
        .fixed_rect
        .map(|r| r.height())
        .or(control.height)
        .unwrap_or_else(|| tier.height.points());
    let corner_radius = if control.small_square {
        seg.small_square_corner_radius.points()
    } else {
        tier.corner_radius.points()
    };
    let border_w = seg.border_width.points();
    let icon_pad = seg.icon_padding.points();
    let font_id = egui::FontId::proportional(font_sz);
    let enabled = control.enabled && ui.is_enabled();
    let sense = if enabled { Sense::click() } else { Sense::hover() };

    let layout = compute_segmented_layout(&control, theme, &font_id, height, sense, ui);
    let (rect, mut response, slots) = (layout.rect, layout.response, layout.slots);
    let count = control.segments.len().max(1);
    let a11y_group = control.a11y_label.clone();

    let mut toggled = if enabled {
        handle_multi_keyboard(ui, &mut response, &control.segments, active)
    } else {
        None
    };

    let alpha = if enabled { 1.0 } else { theme.disabled_opacity() };
    let surface_bg = theme.active_surface_at(ui, rect.center());
    let inactive_bg = theme.segmented_small_square_inactive_bg();
    let active_bg = theme.segmented_small_square_active_bg();
    let track_surface = alpha_composite(inactive_bg, surface_bg);
    let active_surface = alpha_composite(active_bg, surface_bg);
    let active_fg = theme.label_on(active_surface).gamma_multiply(alpha);
    let idle_fg = theme.secondary_label_on(track_surface).gamma_multiply(alpha);
    let disabled_fg = theme.tertiary_label_on(track_surface).gamma_multiply(alpha);

    let painter = ui.painter_at(rect);
    if enabled && response.has_focus() {
        theme.paint_focus_ring(&painter, rect, corner_radius);
    }
    let border_stroke = Stroke::new(border_w, theme.box_border().gamma_multiply(alpha));
    painter.rect(
        rect,
        CornerRadius::same(corner_radius as u8),
        inactive_bg.gamma_multiply(alpha),
        border_stroke,
        StrokeKind::Inside,
    );

    let hover_pos = response.hover_pos();
    let cursor_id = response.id.with("seg_multi_cursor");
    for (i, (seg_item, slot)) in control.segments.into_iter().zip(slots.iter().cloned()).enumerate() {
        let seg_rect = slot.rect;
        if hover_pos.is_some_and(|p| seg_rect.contains(p)) {
            if let Some(tip) = &seg_item.tooltip {
                response = response.on_hover_text(tip.as_ref());
            }
        }
        if enabled
            && seg_item.enabled
            && response.clicked()
            && response.interact_pointer_pos().is_some_and(|p| seg_rect.contains(p))
        {
            response.request_focus();
            ui.ctx().data_mut(|d| d.insert_temp(cursor_id, i));
            if let Some(flag) = active.get_mut(i) {
                *flag = !*flag;
                toggled = Some(i);
                response.mark_changed();
            }
        }
        let is_on = active.get(i).copied().unwrap_or(false);
        if is_on {
            painter.rect_filled(
                seg_rect.shrink(border_w),
                CornerRadius::ZERO,
                active_bg.gamma_multiply(alpha),
            );
        }
        let tint = if !seg_item.enabled {
            disabled_fg
        } else if is_on {
            active_fg
        } else {
            idle_fg
        };
        match seg_item.content {
            SegmentContent::Text(_) | SegmentContent::Symbol(_) => {
                if let Some(g) = slot.galley {
                    let pos = Align2::CENTER_CENTER.anchor_size(seg_rect.center(), g.size()).min;
                    painter.galley(pos, g, tint);
                }
            }
            SegmentContent::IconFn(icon_fn) => icon_fn(&painter, seg_rect.shrink(icon_pad), tint),
            SegmentContent::Custom(painter_fn) => painter_fn(&painter, seg_rect.shrink(icon_pad), tint),
        }
    }
    for i in 1..count {
        if let Some(slot) = slots.get(i) {
            painter.line_segment(
                [Pos2::new(slot.rect.min.x, rect.min.y), Pos2::new(slot.rect.min.x, rect.max.y)],
                border_stroke,
            );
        }
    }

    response.widget_info(|| {
        let mut info = WidgetInfo::new(Role::Toolbar);
        info.enabled = enabled;
        info.label = a11y_group.as_ref().map(|s| s.as_ref().to_owned());
        info
    });

    toggled
}
