//! Visual rendering pipeline and `AccessKit` semantics for [`super::MacSegmentedControl`].

use super::interaction::{handle_segmented_input, InteractionConfig};
use super::item::{SegmentContent, SegmentEvent};
use super::layout::compute_segmented_layout;
use super::MacSegmentedControl;
use crate::controls::MacControlSize;
use crate::layout::subpixel_snap_pos;
use crate::theme::{alpha_composite, MacTheme};
use egui::{
    Align2, Color32, CornerRadius, Galley, Painter, Pos2, Rect, Response, Role, Sense, Stroke,
    StrokeKind, Ui, Vec2, WidgetInfo,
};
use std::sync::Arc;

fn paint_menu_chevron(painter: &Painter, theme: &MacTheme, seg_rect: Rect, tint: Color32, ppp: f32) {
    let seg = theme.segmented();
    let cx = seg_rect.right() - seg.menu_chevron_inset_trailing.points();
    let tip_y = seg_rect.bottom() - seg.menu_chevron_inset_bottom.points();
    let hw = seg.menu_chevron_half_width.points();
    let ch = seg.menu_chevron_height.points();
    let stroke = Stroke::new(seg.menu_chevron_stroke_width.points(), tint);
    let left_pt = subpixel_snap_pos(Pos2::new(cx - hw, tip_y - ch), ppp);
    let tip_pt = subpixel_snap_pos(Pos2::new(cx, tip_y), ppp);
    let right_pt = subpixel_snap_pos(Pos2::new(cx + hw, tip_y - ch), ppp);
    painter.line_segment([left_pt, tip_pt], stroke);
    painter.line_segment([tip_pt, right_pt], stroke);
}

fn paint_segment_content(
    painter: &Painter,
    seg_rect: Rect,
    icon_pad: f32,
    tint: Color32,
    content: SegmentContent<'_>,
    galley: Option<Arc<Galley>>,
) {
    match content {
        SegmentContent::Text(_) | SegmentContent::Symbol(_) => {
            if let Some(g) = galley {
                let pos = Align2::CENTER_CENTER.anchor_size(seg_rect.center(), g.size()).min;
                painter.galley(pos, g, tint);
            }
        }
        SegmentContent::IconFn(icon_fn) => icon_fn(painter, seg_rect.shrink(icon_pad), tint),
        SegmentContent::Custom(painter_fn) => painter_fn(painter, seg_rect.shrink(icon_pad), tint),
    }
}

pub(super) fn render_segmented(
    control: MacSegmentedControl<'_>,
    ui: &mut Ui,
    mut on_event: impl FnMut(SegmentEvent),
) -> Response {
    let theme_cow = MacTheme::resolve(control.theme, ui.ctx());
    let theme: &MacTheme = &theme_cow;
    let seg = theme.segmented();
    let (tier, font_sz) = match control.control_size {
        MacControlSize::Regular | MacControlSize::Large => (&seg.regular, seg.font_size.points()),
        MacControlSize::Small | MacControlSize::CompactInfobox => (&seg.small, seg.small_font_size.points()),
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
    let enabled = control.enabled && ui.is_enabled();
    let sense = if enabled { Sense::click_and_drag() } else { Sense::hover() };
    let font_id = egui::FontId::proportional(font_sz);

    let layout = compute_segmented_layout(&control, theme, &font_id, height, sense, ui);
    let (rect, mut response, slots) = (layout.rect, layout.response, layout.slots);
    let count = control.segments.len().max(1);
    let mut active_idx = control.selected_idx.min(count.saturating_sub(1));

    let momentary = control.momentary;
    let pressed_seg = handle_segmented_input(
        ui,
        &mut response,
        &control.segments,
        &slots,
        &mut active_idx,
        InteractionConfig {
            enabled,
            momentary,
            hold_delay: seg.menu_press_hold_delay.points() as f64,
            drag_threshold: seg.menu_drag_threshold.points(),
        },
        &mut on_event,
    );

    let a11y_group = control.a11y_label.clone();
    let a11y_active = (!momentary)
        .then(|| control.segments.get(active_idx).and_then(|s| s.accessible_cow()))
        .flatten();

    let alpha_factor = if enabled { 1.0 } else { theme.disabled_opacity() };
    let surface_bg = theme.active_surface_at(ui, rect.center());
    let painter = ui.painter_at(rect);
    if enabled && response.has_focus() {
        theme.paint_focus_ring(&painter, rect, corner_radius);
    }

    let bg_fill = if control.small_square {
        theme.segmented_small_square_inactive_bg()
    } else {
        theme.control_background()
    };
    let border_col = if control.small_square { theme.box_border() } else { theme.separator() };
    let border_width = seg.border_width.points();
    let indicator_inset = seg.indicator_inset.points();
    let indicator_radius = tier.indicator_radius.points();

    painter.rect(
        rect,
        CornerRadius::same(corner_radius as u8),
        bg_fill.gamma_multiply(alpha_factor),
        Stroke::new(border_width, border_col.gamma_multiply(alpha_factor)),
        StrokeKind::Inside,
    );

    let div_inset = seg.divider_vertical_inset.points();
    for i in 1..count {
        let suppress_divider = !control.small_square
            && if momentary {
                pressed_seg.is_some_and(|p| i == p || i == p + 1)
            } else {
                i == active_idx || i == active_idx + 1
            };
        if suppress_divider {
            continue;
        }
        if let Some(slot) = slots.get(i) {
            let (y0, y1) = if control.small_square {
                (rect.min.y, rect.max.y)
            } else {
                (rect.min.y + div_inset, rect.max.y - div_inset)
            };
            if y1 > y0 {
                painter.line_segment(
                    [Pos2::new(slot.rect.min.x, y0), Pos2::new(slot.rect.min.x, y1)],
                    Stroke::new(border_width, border_col.gamma_multiply(alpha_factor)),
                );
            }
        }
    }

    let track_surface = alpha_composite(bg_fill, surface_bg);
    let pill_surface = if momentary {
        track_surface
    } else {
        let target_rect = slots.get(active_idx).map_or(rect, |s| s.rect);
        let spring = theme.motion().swiftui_snappy.to_spring();
        let anim_x = spring.animate_f32(ui.ctx(), response.id.with("seg_caspring_x"), target_rect.min.x - rect.min.x);
        let anim_w = if layout.uniform_widths {
            target_rect.width()
        } else {
            spring.animate_f32(ui.ctx(), response.id.with("seg_caspring_w"), target_rect.width())
        };
        let animated_seg_rect = Rect::from_min_size(Pos2::new(rect.min.x + anim_x, rect.min.y), Vec2::new(anim_w, rect.height()));

        if control.small_square {
            let active_bg = theme.segmented_small_square_active_bg();
            painter.rect_filled(animated_seg_rect.shrink(border_width), CornerRadius::ZERO, active_bg.gamma_multiply(alpha_factor));
            alpha_composite(active_bg, surface_bg)
        } else {
            let pill_rect = animated_seg_rect.shrink(indicator_inset);
            let pill_bg = theme.control();
            painter.rect_filled(
                pill_rect.translate(Vec2::new(0.0, border_width)),
                CornerRadius::same(indicator_radius as u8),
                theme.drop_shadow().gamma_multiply(alpha_factor),
            );
            painter.rect(
                pill_rect,
                CornerRadius::same(indicator_radius as u8),
                pill_bg.gamma_multiply(alpha_factor),
                Stroke::new(border_width, theme.separator().gamma_multiply(alpha_factor)),
                StrokeKind::Inside,
            );
            alpha_composite(pill_bg, surface_bg)
        }
    };

    if let Some(p_idx) = pressed_seg.filter(|&p| momentary || p != active_idx) {
        if let Some(p_slot) = slots.get(p_idx) {
            let (hl_rect, hl_radius) = if control.small_square {
                (p_slot.rect.shrink(border_width), CornerRadius::ZERO)
            } else {
                (p_slot.rect.shrink(indicator_inset), CornerRadius::same(indicator_radius as u8))
            };
            let hl_alpha = seg.pressed_highlight_opacity.points() * alpha_factor;
            painter.rect_filled(hl_rect, hl_radius, theme.separator().gamma_multiply(hl_alpha));
        }
    }

    let active_fg = theme.label_on(pill_surface).gamma_multiply(alpha_factor);
    let hover_fg = theme.label_on(track_surface).gamma_multiply(alpha_factor);
    let idle_fg = theme.secondary_label_on(track_surface).gamma_multiply(alpha_factor);
    let disabled_fg = theme.tertiary_label_on(track_surface).gamma_multiply(alpha_factor);
    let hover_pos = response.hover_pos();
    let ppp = ui.ctx().pixels_per_point();
    let icon_pad = seg.icon_padding.points();

    for (i, (seg_item, slot)) in control.segments.into_iter().zip(slots).enumerate() {
        let seg_rect = slot.rect;
        let is_hovered = hover_pos.is_some_and(|p| seg_rect.contains(p));
        if is_hovered {
            if let Some(tip) = &seg_item.tooltip {
                response = response.on_hover_text(tip.as_ref());
            }
        }
        let tint = if !seg_item.enabled {
            disabled_fg
        } else if !momentary && i == active_idx {
            active_fg
        } else if pressed_seg == Some(i) || (enabled && is_hovered) {
            hover_fg
        } else {
            idle_fg
        };

        paint_segment_content(&painter, seg_rect, icon_pad, tint, seg_item.content, slot.galley);

        if seg_item.shows_menu_indicator {
            paint_menu_chevron(&painter, theme, seg_rect, tint, ppp);
        }
    }

    response.widget_info(|| {
        let mut info = WidgetInfo::new(if momentary { Role::Toolbar } else { Role::RadioGroup });
        info.enabled = enabled;
        if !momentary {
            info.selected = Some(true);
            info.current_text_value = a11y_active.as_ref().map(|s| s.as_ref().to_owned());
        }
        info.label = a11y_group
            .as_ref()
            .or(a11y_active.as_ref())
            .map(|s| s.as_ref().to_owned());
        info
    });

    response
}
