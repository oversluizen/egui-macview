//! Stack-inline single-pass layout and `Galley` shaping for [`super::MacSegmentedControl`].

use super::item::SegmentContent;
use super::storage::InlineBuf;
use super::MacSegmentedControl;
use crate::theme::MacTheme;
use egui::{Color32, FontId, Galley, Pos2, Rect, Response, Sense, Ui, Vec2};
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct SegLayoutSlot {
    pub rect: Rect,
    pub galley: Option<Arc<Galley>>,
}

pub(super) struct SegmentedLayout {
    pub rect: Rect,
    pub response: Response,
    pub slots: InlineBuf<SegLayoutSlot>,
    pub uniform_widths: bool,
}

pub(super) fn compute_segmented_layout(
    control: &MacSegmentedControl<'_>,
    theme: &MacTheme,
    font_id: &FontId,
    height: f32,
    sense: Sense,
    ui: &mut Ui,
) -> SegmentedLayout {
    let seg = theme.segmented();
    let padding_h = seg.padding_horizontal.points();

    let mut shaped_galleys: InlineBuf<Option<Arc<Galley>>> = InlineBuf::new();
    let mut max_content_w = seg.item_width.points();
    let mut explicit_sum = 0.0_f32;
    let mut unspec_count = 0_usize;

    for item in control.segments.iter() {
        let galley = match &item.content {
            SegmentContent::Text(s) if !s.is_empty() => Some(theme.layout_tracked_galley(
                ui,
                s,
                font_id.clone(),
                Color32::PLACEHOLDER,
                f32::INFINITY,
            )),
            SegmentContent::Symbol(sym) => Some(theme.layout_tracked_galley(
                ui,
                sym.as_str(),
                font_id.clone(),
                Color32::PLACEHOLDER,
                f32::INFINITY,
            )),
            SegmentContent::Text(_) | SegmentContent::IconFn(_) | SegmentContent::Custom(_) => None,
        };
        if let Some(w) = item.width {
            explicit_sum += w;
        } else {
            unspec_count += 1;
            if let Some(g) = &galley {
                max_content_w = max_content_w.max(g.size().x + padding_h * 2.0);
            }
        }
        shaped_galleys.push(galley);
    }

    let (rect, response, fallback_w) = if let Some(fixed) = control.fixed_rect {
        let id = if let Some(salt) = control.id_salt {
            ui.make_persistent_id(salt)
        } else {
            let auto = ui.next_auto_id().with("mac_segmented_fixed");
            ui.skip_ahead_auto_ids(1);
            auto
        };
        let resp = ui.interact(fixed, id, sense);
        let rem = (fixed.width() - explicit_sum).max(0.0);
        let fw = if unspec_count > 0 { rem / unspec_count as f32 } else { 0.0 };
        (fixed, resp, fw)
    } else {
        let total_w = explicit_sum + (unspec_count as f32 * max_content_w);
        let (r, resp) = if let Some(salt) = control.id_salt {
            let (_, r) = ui.allocate_space(Vec2::new(total_w, height));
            let resp = ui.interact(r, ui.make_persistent_id(salt), sense);
            (r, resp)
        } else {
            ui.allocate_exact_size(Vec2::new(total_w, height), sense)
        };
        (r, resp, max_content_w)
    };

    let mut slots: InlineBuf<SegLayoutSlot> = InlineBuf::new();
    let mut cur_x = rect.min.x;
    let mut first_w: Option<f32> = None;
    let mut uniform_widths = true;

    if control.segments.is_empty() {
        slots.push(SegLayoutSlot { rect, galley: None });
    } else {
        for (item, galley) in control.segments.iter().zip(shaped_galleys) {
            let w = item.width.unwrap_or(fallback_w);
            if let Some(w0) = first_w {
                if (w - w0).abs() != 0.0 {
                    uniform_widths = false;
                }
            } else {
                first_w = Some(w);
            }
            let seg_rect = Rect::from_min_size(Pos2::new(cur_x, rect.min.y), Vec2::new(w, rect.height()));
            slots.push(SegLayoutSlot { rect: seg_rect, galley });
            cur_x += w;
        }
    }

    SegmentedLayout {
        rect,
        response,
        slots,
        uniform_widths,
    }
}
