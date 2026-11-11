//! Filterable popover overlay and keyboard navigation for [`super::MacComboBox`].

use super::MacComboBox;
use crate::theme::MacTheme;
use egui::{
    Area, CornerRadius, FontId, Id, Key, Modifiers, Order, Pos2, Rect, Response, ScrollArea, Sense,
    Stroke, StrokeKind, Ui, Vec2,
};

/// Resolved geometry for the popover dropdown list, clamped to screen boundaries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComboBoxPopoverGeometry {
    /// Outer popover bounding box.
    pub popover_rect: Rect,
    /// Popover top-left screen position.
    pub popover_pos: Pos2,
    /// Single row height in points.
    pub item_height: f32,
    /// Total popover height in points.
    pub popover_height: f32,
    /// Total popover width in points.
    pub popover_width: f32,
}

pub(super) fn compute_popover_geometry<T: AsRef<str>>(
    combo: &MacComboBox<'_, T>,
    ui: &Ui,
    header_rect: Rect,
    item_count: usize,
) -> ComboBoxPopoverGeometry {
    let theme_cow = MacTheme::resolve(combo.theme, ui.ctx());
    let metrics = theme_cow.combo_box();
    let item_h = metrics.item_height.points();
    let visible_count = (metrics.visible_items.points() as usize).min(item_count);
    let pop_h = item_h * (visible_count as f32);
    let pop_w = header_rect.width();

    let screen_rect = ui
        .ctx()
        .input(|i| i.raw.screen_rect)
        .unwrap_or_else(|| ui.clip_rect());

    let gap = metrics.intercell_spacing_y.points();
    let pop_y = if header_rect.max.y + gap + pop_h > screen_rect.bottom()
        && header_rect.min.y - gap - pop_h >= screen_rect.top()
    {
        header_rect.min.y - gap - pop_h
    } else {
        header_rect.max.y + gap
    };

    let pop_pos = Pos2::new(header_rect.min.x, pop_y);
    let pop_rect = Rect::from_min_size(pop_pos, Vec2::new(pop_w, pop_h));

    ComboBoxPopoverGeometry {
        popover_rect: pop_rect,
        popover_pos: pop_pos,
        item_height: item_h,
        popover_height: pop_h,
        popover_width: pop_w,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_combo_popover<T: AsRef<str>>(
    combo: &mut MacComboBox<'_, T>,
    ui: &mut Ui,
    theme: &MacTheme,
    rect: Rect,
    corner_radius: f32,
    is_open: &mut bool,
    stored_sel_idx: Option<usize>,
    open_id: Id,
    selected_item_id: Id,
    pointer_pos: Option<Pos2>,
    response: &mut Response,
) {
    let metrics = theme.combo_box();
    let mut final_sel_idx = stored_sel_idx.unwrap_or(0);

    if *is_open {
        let current_text_lower = combo.text.to_lowercase();
        let is_exact_match = combo
            .items
            .iter()
            .any(|it| it.as_ref().eq_ignore_ascii_case(combo.text.as_str()));
        let filtered: Vec<(usize, &str)> = combo
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| (i, item.as_ref()))
            .filter(|(_, s)| {
                current_text_lower.is_empty()
                    || is_exact_match
                    || s.to_lowercase().contains(&current_text_lower)
            })
            .collect();

        let matching_idx = filtered
            .iter()
            .position(|(_, s)| s.eq_ignore_ascii_case(combo.text.as_str()))
            .unwrap_or(0);
        let mut sel_idx = stored_sel_idx
            .unwrap_or(matching_idx)
            .min(filtered.len().saturating_sub(1));

        let mut nav_changed = false;
        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            *is_open = false;
        } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
            if !filtered.is_empty() {
                sel_idx = (sel_idx + 1).min(filtered.len() - 1);
                nav_changed = true;
            }
        } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) {
            sel_idx = sel_idx.saturating_sub(1);
            nav_changed = true;
        } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
            if let Some((_, selected_str)) = filtered.get(sel_idx) {
                *combo.text = (*selected_str).to_string();
                response.mark_changed();
                *is_open = false;
            }
        }

        if *is_open && !filtered.is_empty() {
            let pop_geom = compute_popover_geometry(combo, ui, rect, filtered.len());
            let area = Area::new(combo.id_salt.with("popover_area"))
                .order(Order::Foreground)
                .fixed_pos(pop_geom.popover_pos);

            area.show(ui.ctx(), |pop_ui| {
                pop_ui.set_clip_rect(pop_geom.popover_rect);
                let pop_painter = pop_ui.painter_at(pop_geom.popover_rect);
                pop_painter.rect(
                    pop_geom.popover_rect,
                    CornerRadius::same(corner_radius as u8),
                    theme.control_background(),
                    Stroke::new(1.0, theme.separator()),
                    StrokeKind::Inside,
                );

                ScrollArea::vertical()
                    .max_height(pop_geom.popover_height)
                    .show(pop_ui, |scroll_ui| {
                        let body_font = scroll_ui
                            .style()
                            .text_styles
                            .get(&egui::TextStyle::Body)
                            .cloned()
                            .unwrap_or_else(|| FontId::proportional(theme.text_field().regular.font_size.points()));
                        for (idx, (_, item_str)) in filtered.iter().enumerate() {
                            let (row_rect, row_resp) = scroll_ui.allocate_exact_size(
                                Vec2::new(pop_geom.popover_width, pop_geom.item_height),
                                Sense::click(),
                            );

                            let is_selected = idx == sel_idx;
                            if is_selected && nav_changed {
                                row_resp.scroll_to_me(Some(egui::Align::Center));
                            }

                            let row_hovered = row_resp.hovered();
                            if row_hovered || is_selected {
                                scroll_ui.painter().rect_filled(
                                    row_rect,
                                    CornerRadius::same(0),
                                    theme.selected_control(),
                                );
                            }

                            let text_color = if row_hovered || is_selected {
                                theme.on_accent_text()
                            } else {
                                theme.label()
                            };

                            let galley = theme.layout_tracked_galley(
                                scroll_ui,
                                item_str,
                                body_font.clone(),
                                text_color,
                                f32::INFINITY,
                            );
                            scroll_ui.painter().galley(
                                Pos2::new(
                                    row_rect.min.x + metrics.intercell_spacing_x.points(),
                                    row_rect.center().y - galley.size().y * 0.5,
                                ),
                                galley,
                                text_color,
                            );

                            if row_resp.clicked() {
                                *combo.text = (*item_str).to_string();
                                response.mark_changed();
                                *is_open = false;
                            }
                        }
                    });
            });

            if ui.input(|i| i.pointer.primary_clicked()) {
                if let Some(pos) = pointer_pos {
                    if !rect.contains(pos) && !pop_geom.popover_rect.contains(pos) {
                        *is_open = false;
                    }
                }
            }
            final_sel_idx = sel_idx;
        }
    }

    ui.data_mut(|d| {
        d.insert_temp(open_id, *is_open);
        if *is_open {
            d.insert_temp(selected_item_id, final_sel_idx);
        } else {
            d.remove_temp::<usize>(selected_item_id);
        }
    });
}
