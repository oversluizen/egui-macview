//! State 6 (`v0.6.9`) Pro-Inspector Controls showcase (`MacOriginMatrix`, `MacTagColorPicker`, `MacTableActionBar`).

use egui::{Align, CornerRadius, Layout, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use egui_macview::{
    controls::{
        MacControlSize, MacFontWeight, MacLabel, MacLabelColor, MacOriginAnchor, MacOriginMatrix,
        MacTableActionBar, MacTagColorPicker,
    },
    theme::MacTheme,
    MacToastManager,
};
use std::cell::RefCell;

/// Renders the `v0.6.9` Pro-Inspector controls (`MacOriginMatrix`, `MacTagColorPicker`, `MacTableActionBar`).
pub fn render_state6_pro_controls(
    ui: &mut Ui,
    theme: &MacTheme,
    origin_anchor: &mut MacOriginAnchor,
    tag_color: &mut Option<usize>,
    action_items: &mut Vec<String>,
    toasts: &RefCell<MacToastManager>,
) {
    ui.vertical(|pro_ui| {
        MacLabel::new("Pro-Inspector Controls (v0.6.9): Origin Matrix, Tag Color Dots & Table Action Bar")
            .weight(MacFontWeight::Semibold)
            .theme(theme)
            .show(pro_ui);
        pro_ui.add_space(4.0);

        let row_h = theme.origin_matrix().grid_size.points();
        pro_ui.allocate_ui_with_layout(
            Vec2::new(pro_ui.available_width(), row_h),
            Layout::left_to_right(Align::Center),
            |row_ui| {
                row_ui.set_min_height(row_h);
                MacLabel::new("Origin:")
                    .color(MacLabelColor::Secondary)
                    .theme(theme)
                    .show(row_ui);
                if MacOriginMatrix::new(origin_anchor)
                    .control_size(MacControlSize::Regular)
                    .theme(theme)
                    .show(row_ui)
                    .changed()
                {
                    toasts.borrow_mut().info(format!("Origin {origin_anchor:?}"));
                }
                row_ui.add_space(10.0);
                MacLabel::new("Tag Label:")
                    .color(MacLabelColor::Secondary)
                    .theme(theme)
                    .show(row_ui);
                if MacTagColorPicker::new(tag_color)
                    .columns(12)
                    .control_size(MacControlSize::Regular)
                    .theme(theme)
                    .show(row_ui)
                    .changed()
                {
                    toasts.borrow_mut().info(format!("Tag {tag_color:?}"));
                }
            },
        );

        pro_ui.add_space(6.0);

        // Compact bordered AppKit list/table preview docked flush above MacTableActionBar
        let palette = theme.tag_label_palette();
        let active_tag_col = tag_color.and_then(|i| palette.get(i).copied());
        let item_row_h = 20.0_f32;
        let visible_rows = action_items.len().max(1);
        let list_h = (visible_rows as f32) * item_row_h;
        let list_w = pro_ui.available_width();
        let prev_spacing_y = pro_ui.spacing().item_spacing.y;
        pro_ui.spacing_mut().item_spacing.y = 0.0;

        let (list_rect, _) = pro_ui.allocate_exact_size(Vec2::new(list_w, list_h), Sense::hover());
        let painter = pro_ui.painter_at(list_rect);
        painter.rect_filled(list_rect, CornerRadius::same(1), theme.control_background());
        for idx in 0..visible_rows {
            let y_top = list_rect.top() + (idx as f32) * item_row_h;
            let r_rect = Rect::from_min_size(Pos2::new(list_rect.left(), y_top), Vec2::new(list_w, item_row_h));
            if idx % 2 == 1 {
                painter.rect_filled(r_rect, CornerRadius::ZERO, theme.box_background());
            }
            let dot_pos = Pos2::new(r_rect.left() + 12.0, r_rect.center().y);
            if let Some(c) = active_tag_col {
                painter.circle(dot_pos, 4.0, c, Stroke::new(1.0, theme.tag_swatch_border()));
            } else {
                painter.circle_stroke(dot_pos, 4.0, Stroke::new(1.0, theme.secondary_label()));
            }
            let label_rect = Rect::from_min_max(Pos2::new(r_rect.left() + 24.0, y_top), Pos2::new(r_rect.right() - 8.0, y_top + item_row_h));
            let mut row_child = pro_ui.new_child(
                egui::UiBuilder::new()
                    .id_salt(("v069_list_row", idx))
                    .max_rect(label_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            if let Some(item_text) = action_items.get(idx) {
                MacLabel::new(item_text.as_str()).control_size(MacControlSize::Small).theme(theme).show(&mut row_child);
                row_child.add_space(8.0);
                MacLabel::new(format!("{origin_anchor:?}"))
                    .control_size(MacControlSize::Mini)
                    .color(MacLabelColor::Secondary)
                    .theme(theme)
                    .show(&mut row_child);
            } else {
                MacLabel::new("Empty list — click '+' below")
                    .control_size(MacControlSize::Small)
                    .color(MacLabelColor::Secondary)
                    .theme(theme)
                    .show(&mut row_child);
            }
        }
        painter.rect_stroke(list_rect, CornerRadius::same(1), Stroke::new(1.0, theme.separator()), StrokeKind::Inside);

        let bar_resp = MacTableActionBar::new("gallery_v069_table_action_bar")
            .can_add(true)
            .can_remove(!action_items.is_empty())
            .with_action_menu(true)
            .fill_remaining_strip(true)
            .theme(theme)
            .show(pro_ui);
        pro_ui.spacing_mut().item_spacing.y = prev_spacing_y;

        if bar_resp.add_clicked {
            let next_idx = action_items.len() + 1;
            action_items.push(format!("Item #{next_idx}"));
            toasts.borrow_mut().success("Added table item");
        }
        if bar_resp.remove_clicked {
            action_items.pop();
            toasts.borrow_mut().info("Removed table item");
        }
        if bar_resp.action_clicked {
            toasts.borrow_mut().info("Table action menu clicked");
        }
    });
}
