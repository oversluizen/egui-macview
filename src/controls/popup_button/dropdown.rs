//! Floating menu overlay handler for [`super::MacPopUpButton`].

use crate::theme::MacTheme;
use egui::{
    Align2, Area, CornerRadius, Galley, Key, Modifiers, Order, Pos2, Rect, Response, Sense, Stroke,
    StrokeKind, Ui, Vec2,
};
use std::sync::Arc;

pub(super) type PopupItemsCache = (f32, Vec<Arc<Galley>>);

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_popup_dropdown(
    ui: &mut Ui,
    theme: &MacTheme,
    rect: Rect,
    response: &mut Response,
    selected_index: &mut usize,
    items: &[&str],
    enabled: bool,
    desired_width: f32,
    height: f32,
    corner_radius: f32,
    current_galley: &Arc<Galley>,
    cached_items: &mut Option<PopupItemsCache>,
    layout_items: impl Fn(&Ui, &Arc<Galley>, &mut Option<PopupItemsCache>) -> PopupItemsCache,
) {
    let metrics = theme.popup_button();
    let is_open_id = response.id.with("mac_popup_is_open");
    let mut is_popup_open = ui.data(|d| d.get_temp::<bool>(is_open_id).unwrap_or(false));

    if enabled {
        if response.clicked() {
            response.request_focus();
            is_popup_open = !is_popup_open;
            ui.data_mut(|d| d.insert_temp(is_open_id, is_popup_open));
        } else if response.has_focus()
            && (ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Space))
                || ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)))
        {
            is_popup_open = !is_popup_open;
            ui.data_mut(|d| d.insert_temp(is_open_id, is_popup_open));
        }
    }

    if enabled && is_popup_open {
        let (auto_width, item_galleys) = layout_items(ui, current_galley, cached_items);
        let menu_width = desired_width
            .max(auto_width)
            .max(metrics.menu_min_width.points());
        let item_height = height.max(metrics.menu_item_height.points());
        let menu_padding = metrics.menu_item_padding.points();
        let total_menu_height = item_height * (items.len() as f32) + menu_padding * 2.0;

        let screen_rect = ui
            .ctx()
            .input(|i| i.raw.screen_rect)
            .unwrap_or_else(|| ui.clip_rect());
        let popup_gap = metrics.popup_gap.points();
        let popup_y = if rect.bottom() + popup_gap + total_menu_height > screen_rect.bottom()
            && rect.top() - popup_gap - total_menu_height >= screen_rect.top()
        {
            rect.top() - popup_gap - total_menu_height
        } else {
            rect.bottom() + popup_gap
        };
        let popup_x = if rect.left() + menu_width > screen_rect.right() {
            (screen_rect.right() - menu_width).max(screen_rect.left())
        } else {
            rect.left()
        };
        let popup_pos = Pos2::new(popup_x, popup_y);
        let area_id = response.id.with("mac_popup_area");
        let mut item_selected_this_frame = false;

        Area::new(area_id)
            .order(Order::Foreground)
            .sense(Sense::click())
            .fixed_pos(popup_pos)
            .show(ui.ctx(), |menu_ui| {
                let (menu_rect, _bg_resp) = menu_ui.allocate_exact_size(
                    Vec2::new(menu_width, total_menu_height),
                    Sense::click(),
                );
                if menu_ui.rect_contains_pointer(menu_rect) {
                    menu_ui.input_mut(|i| {
                        i.smooth_scroll_delta = Vec2::ZERO;
                    });
                }
                let menu_painter = menu_ui.painter_at(menu_rect);

                menu_painter.rect(
                    menu_rect,
                    CornerRadius::same(corner_radius as u8),
                    theme.window_background(),
                    Stroke::new(metrics.border_width.points(), theme.separator()),
                    StrokeKind::Inside,
                );

                let item_margin = metrics.menu_item_padding.points();
                for (idx, item_galley) in item_galleys.into_iter().enumerate() {
                    let item_rect = Rect::from_min_size(
                        Pos2::new(
                            menu_rect.left() + item_margin,
                            menu_rect.top() + item_margin + (idx as f32) * item_height,
                        ),
                        Vec2::new(menu_width - item_margin * 2.0, item_height),
                    );
                    let item_resp =
                        menu_ui.interact(item_rect, area_id.with(idx), Sense::click());
                    let is_hovered = item_resp.hovered();
                    let is_active = idx == *selected_index;

                    if is_hovered || is_active {
                        let item_bg = if is_hovered {
                            theme.control_accent()
                        } else {
                            theme.control()
                        };
                        menu_painter.rect_filled(
                            item_rect,
                            CornerRadius::same(corner_radius as u8),
                            item_bg,
                        );
                    }

                    let item_text_color = if is_hovered {
                        theme.on_accent_text()
                    } else {
                        theme.label()
                    };

                    let item_text_pos = Align2::LEFT_CENTER
                        .anchor_size(
                            Pos2::new(item_rect.left() + menu_padding, item_rect.center().y),
                            item_galley.size(),
                        )
                        .min;
                    menu_painter.galley(item_text_pos, item_galley, item_text_color);

                    if item_resp.clicked() {
                        *selected_index = idx;
                        item_selected_this_frame = true;
                        menu_ui.data_mut(|d| d.insert_temp(is_open_id, false));
                        response.mark_changed();
                    }
                }
            });

        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            ui.data_mut(|d| d.insert_temp(is_open_id, false));
        } else if !item_selected_this_frame
            && ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary))
            && !response.clicked()
        {
            let popup_bounds =
                Rect::from_min_size(popup_pos, Vec2::new(menu_width, total_menu_height));
            if !ui.rect_contains_pointer(popup_bounds) {
                ui.data_mut(|d| d.insert_temp(is_open_id, false));
            }
        }
    }
}
