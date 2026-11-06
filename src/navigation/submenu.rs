//! Authentic macOS cascading submenu popover rendering engine.
//!
//! Normative Reference: SPEC-007 §2.3.
//! Grounded in Single Responsibility Principle to maintain GL-1015 modularity ceilings.

use egui::{
    Align2, Area, Color32, CornerRadius, FontId, Id, Order, Pos2, Rect, Sense, Stroke, StrokeKind,
    Vec2,
};

use super::item::MacMenuItemKind;
use super::model::NSMenu;
use crate::theme::MacTheme;

/// Renders a cascading submenu popover adjacent to its parent menu item.
pub fn render_submenu_popover(
    child_menu: &NSMenu,
    sub_id: Id,
    sub_pos: Pos2,
    theme: &MacTheme,
    ctx: &egui::Context,
) -> Option<String> {
    let metrics = theme.popup_button();
    let item_height = metrics.menu_item_height.points();
    let menu_padding = metrics.menu_item_padding.points();
    let padding_h = metrics.padding_horizontal.points();
    let sub_min_w = metrics.menu_min_width.points();
    let sub_total_h = (child_menu.items.len() as f32) * item_height + menu_padding * 2.0;
    let corner_r = theme.popover_metrics().corner_radius.points() as u8;
    let font_id = FontId::proportional(metrics.font_size.points());

    let mut triggered = None;

    Area::new(sub_id)
        .order(Order::Foreground)
        .fixed_pos(sub_pos)
        .show(ctx, |sub_ui| {
            let sub_rect = Rect::from_min_size(sub_pos, Vec2::new(sub_min_w, sub_total_h));
            let painter = sub_ui.painter();
            painter.rect(
                sub_rect,
                CornerRadius::same(corner_r),
                theme.window_background(),
                Stroke::new(1.0, theme.separator()),
                StrokeKind::Inside,
            );

            for (s_idx, sub_item) in child_menu.items.iter().enumerate() {
                let s_y = sub_pos.y + menu_padding + (s_idx as f32) * item_height;
                let s_rect = Rect::from_min_size(
                    Pos2::new(sub_pos.x + menu_padding, s_y),
                    Vec2::new(sub_min_w - menu_padding * 2.0, item_height),
                );

                if matches!(sub_item.kind, MacMenuItemKind::Separator) {
                    painter.line_segment(
                        [
                            Pos2::new(s_rect.left(), s_rect.center().y),
                            Pos2::new(s_rect.right(), s_rect.center().y),
                        ],
                        Stroke::new(1.0, theme.separator()),
                    );
                    continue;
                }

                let s_resp = sub_ui.interact(s_rect, sub_id.with(s_idx), Sense::click());
                let s_hover = s_resp.hovered() && sub_item.enabled;

                if s_hover {
                    painter.rect(
                        s_rect,
                        CornerRadius::same(corner_r),
                        theme.control_accent(),
                        Stroke::NONE,
                        StrokeKind::Inside,
                    );
                }

                let s_color = if !sub_item.enabled {
                    theme.placeholder_text()
                } else if s_hover {
                    Color32::WHITE
                } else {
                    theme.label()
                };

                painter.text(
                    Pos2::new(s_rect.left() + padding_h, s_rect.center().y),
                    Align2::LEFT_CENTER,
                    &sub_item.title,
                    font_id.clone(),
                    s_color,
                );

                if let MacMenuItemKind::Action {
                    shortcut: Some(sc), ..
                } = &sub_item.kind
                {
                    painter.text(
                        Pos2::new(s_rect.right() - padding_h, s_rect.center().y),
                        Align2::RIGHT_CENTER,
                        sc.format_text(),
                        font_id.clone(),
                        s_color,
                    );
                }

                if s_resp.clicked() && sub_item.enabled {
                    if let MacMenuItemKind::Action { id, .. } = &sub_item.kind {
                        triggered = Some(id.clone());
                    }
                }
            }
        });

    triggered
}
