//! Authentic macOS AppKit Combo Box (`NSComboBox` equivalent, REQ-012-03).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].

use crate::controls::MacControlSize;
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    Area, CornerRadius, Id, Key, Modifiers, Order, Pos2, Rect, Response, ScrollArea, Sense, Stroke,
    StrokeKind, TextEdit, Ui, UiBuilder, Vec2,
};
use std::hash::Hash;

/// Authentic macOS AppKit Combo Box (`NSComboBox` equivalent).
///
/// Features an editable text field coupled with a right-anchored dropdown trigger,
/// keyboard navigation (Alt+Down, Up/Down arrows, Return, Escape), and a filterable popover list.
pub struct MacComboBox<'a, T: AsRef<str>> {
    id_salt: Id,
    text: &'a mut String,
    items: &'a [T],
    control_size: MacControlSize,
    theme: Option<&'a MacTheme>,
    width: Option<f32>,
}

impl<'a, T: AsRef<str>> MacComboBox<'a, T> {
    /// Creates a new combo box for an item slice and a mutable text buffer.
    pub fn new(id_salt: impl Hash + std::fmt::Debug, text: &'a mut String, items: &'a [T]) -> Self {
        Self {
            id_salt: Id::NULL.with(id_salt),
            text,
            items,
            control_size: MacControlSize::Regular,
            theme: None,
            width: None,
        }
    }

    /// Sets the control size (`Regular`, `Small`, or `Mini`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Injects dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Overrides default width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Resolves outer frame height in points.
    pub fn frame_height(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        let m = theme_cow.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_height.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_height.points(),
            MacControlSize::Mini => m.mini_height.points(),
        }
    }

    /// Resolves right dropdown trigger button width in points.
    pub fn button_width(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        let m = theme_cow.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_button_width.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_button_width.points(),
            MacControlSize::Mini => m.mini_button_width.points(),
        }
    }

    /// Resolves left text padding inset in points.
    pub fn text_inset(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        let m = theme_cow.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_inset.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_inset.points(),
            MacControlSize::Mini => m.mini_inset.points(),
        }
    }

    /// Resolves corner radius in points.
    pub fn corner_radius(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        let m = theme_cow.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_corner_radius.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_corner_radius.points(),
            MacControlSize::Mini => m.mini_corner_radius.points(),
        }
    }

    /// Resolves popover single item row height in points ($18.0\,\text{pt}$).
    pub fn item_height(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        theme_cow.combo_box().item_height.points()
    }

    /// Resolves visible items count before scrolling ($5$).
    pub fn visible_items(&self) -> usize {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        theme_cow.combo_box().visible_items.points() as usize
    }

    /// Resolves popover viewport height before scrolling ($90.0\,\text{pt}$).
    pub fn popover_viewport_height(&self) -> f32 {
        self.item_height() * (self.visible_items() as f32)
    }

    /// Shows the combo box widget.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.combo_box();

        let height = self.frame_height();
        let btn_w = self.button_width();
        let corner_r = self.corner_radius();
        let desired_w = self.width.unwrap_or_else(|| {
            ui.available_width().max(btn_w * 2.0)
        });

        let desired_size = Vec2::new(desired_w, height);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
        let rect = subpixel_snap_rect(rect, ui.ctx().pixels_per_point());

        let open_id = self.id_salt.with("popover_open");
        let mut is_open = ui.data_mut(|d| d.get_temp::<bool>(open_id).unwrap_or(false));
        let selected_item_id = self.id_salt.with("popover_sel_idx");
        let mut sel_idx = ui.data_mut(|d| d.get_temp::<usize>(selected_item_id).unwrap_or(0));

        let btn_rect = Rect::from_min_max(
            Pos2::new(rect.max.x - btn_w, rect.min.y),
            rect.max,
        );
        let text_rect = Rect::from_min_max(
            rect.min,
            Pos2::new(rect.max.x - btn_w, rect.max.y),
        );

        if ui.input_mut(|i| i.consume_key(Modifiers::ALT, Key::ArrowDown))
            || ui.input(|i| i.modifiers.alt && i.key_pressed(Key::ArrowDown))
        {
            is_open = true;
        }

        // Draw combo box frame background and border
        let painter = ui.painter_at(rect);
        let bg = theme.control_background();
        let border_stroke = if response.has_focus() {
            Stroke::new(1.0, theme.control_accent())
        } else {
            Stroke::new(1.0, theme.separator())
        };

        painter.rect(
            rect,
            CornerRadius::same(corner_r as u8),
            bg,
            border_stroke,
            StrokeKind::Inside,
        );

        // Draw separator hairline between text field and button
        painter.line_segment(
            [
                Pos2::new(btn_rect.min.x, rect.min.y),
                Pos2::new(btn_rect.min.x, rect.max.y),
            ],
            Stroke::new(1.0, theme.separator()),
        );

        // Draw trigger button background
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let btn_hovered = pointer_pos.is_some_and(|p| btn_rect.contains(p));
        let btn_down = btn_hovered && ui.input(|i| i.pointer.primary_down());

        if btn_down {
            painter.rect_filled(
                btn_rect,
                CornerRadius::same(corner_r as u8),
                theme.selected_control(),
            );
        } else if btn_hovered {
            painter.rect_filled(
                btn_rect,
                CornerRadius::same(corner_r as u8),
                theme.control(),
            );
        }

        if btn_hovered && ui.input(|i| i.pointer.primary_clicked()) {
            is_open = !is_open;
        }

        // Draw trigger chevron glyph (ChevronUpDown)
        let cx = btn_rect.center().x;
        let cy = btn_rect.center().y;
        let half_ch = (btn_w * 0.5) * 0.5;
        let chevron_color = if btn_hovered {
            theme.label()
        } else {
            theme.secondary_label()
        };

        let up_p1 = Pos2::new(cx - half_ch, cy - 2.0);
        let up_p2 = Pos2::new(cx, cy - half_ch - 2.0);
        let up_p3 = Pos2::new(cx + half_ch, cy - 2.0);
        painter.line_segment([up_p1, up_p2], Stroke::new(1.0, chevron_color));
        painter.line_segment([up_p2, up_p3], Stroke::new(1.0, chevron_color));

        let down_p1 = Pos2::new(cx - half_ch, cy + 2.0);
        let down_p2 = Pos2::new(cx, cy + half_ch + 2.0);
        let down_p3 = Pos2::new(cx + half_ch, cy + 2.0);
        painter.line_segment([down_p1, down_p2], Stroke::new(1.0, chevron_color));
        painter.line_segment([down_p2, down_p3], Stroke::new(1.0, chevron_color));

        // Editable text input inside text_rect
        let mut child_ui = ui.new_child(
            UiBuilder::new()
                .id_salt(self.id_salt.with("text_edit"))
                .max_rect(text_rect),
        );
        let edit_res = child_ui.add_sized(
            text_rect.size(),
            TextEdit::singleline(self.text)
                .frame(egui::Frame::NONE),
        );
        if edit_res.changed() {
            response.mark_changed();
            is_open = true;
        }

        // Filter items matching typed substring
        let current_text_lower = self.text.to_lowercase();
        let filtered: Vec<(usize, &str)> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| (i, item.as_ref()))
            .filter(|(_, s)| current_text_lower.is_empty() || s.to_lowercase().contains(&current_text_lower))
            .collect();

        // Keyboard navigation when open
        if is_open {
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                is_open = false;
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                if !filtered.is_empty() {
                    sel_idx = (sel_idx + 1).min(filtered.len() - 1);
                }
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) {
                sel_idx = sel_idx.saturating_sub(1);
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
                if let Some((_, selected_str)) = filtered.get(sel_idx) {
                    *self.text = (*selected_str).to_string();
                    response.mark_changed();
                    is_open = false;
                }
            }
        }

        // Render popup list if open
        if is_open && !filtered.is_empty() {
            let item_h = metrics.item_height.points();
            let visible_count = (metrics.visible_items.points() as usize).min(filtered.len());
            let pop_h = item_h * (visible_count as f32);
            let pop_w = rect.width();

            let screen_rect = ui
                .ctx()
                .input(|i| i.raw.screen_rect)
                .unwrap_or_else(|| ui.clip_rect());

            let pop_y = if rect.max.y + 2.0 + pop_h > screen_rect.bottom()
                && rect.min.y - 2.0 - pop_h >= screen_rect.top()
            {
                rect.min.y - 2.0 - pop_h
            } else {
                rect.max.y + 2.0
            };
            let pop_pos = Pos2::new(rect.min.x, pop_y);
            let pop_rect = Rect::from_min_size(pop_pos, Vec2::new(pop_w, pop_h));

            let area = Area::new(self.id_salt.with("popover_area"))
                .order(Order::Foreground)
                .fixed_pos(pop_pos);

            area.show(ui.ctx(), |pop_ui| {
                pop_ui.set_clip_rect(pop_rect);
                let pop_painter = pop_ui.painter_at(pop_rect);
                pop_painter.rect(
                    pop_rect,
                    CornerRadius::same(corner_r as u8),
                    theme.control_background(),
                    Stroke::new(1.0, theme.separator()),
                    StrokeKind::Inside,
                );

                ScrollArea::vertical()
                    .max_height(pop_h)
                    .show(pop_ui, |scroll_ui| {
                        for (idx, (_, item_str)) in filtered.iter().enumerate() {
                            let (row_rect, row_resp) = scroll_ui
                                .allocate_exact_size(Vec2::new(pop_w, item_h), Sense::click());

                            let is_selected = idx == sel_idx;
                            if is_selected {
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

                            let galley = scroll_ui.painter().layout_no_wrap(
                                item_str.to_string(),
                                scroll_ui.style().text_styles.get(&egui::TextStyle::Body).cloned().unwrap_or_default(),
                                text_color,
                            );
                            scroll_ui.painter().galley(
                                Pos2::new(row_rect.min.x + metrics.intercell_spacing_x.points(), row_rect.center().y - galley.size().y * 0.5),
                                galley,
                                text_color,
                            );

                            if row_resp.clicked() {
                                *self.text = (*item_str).to_string();
                                response.mark_changed();
                                is_open = false;
                            }
                        }
                    });
            });

            // Click outside to dismiss
            if ui.input(|i| i.pointer.primary_clicked()) {
                if let Some(pos) = pointer_pos {
                    if !rect.contains(pos) && !pop_rect.contains(pos) {
                        is_open = false;
                    }
                }
            }
        }

        ui.data_mut(|d| {
            d.insert_temp(open_id, is_open);
            d.insert_temp(selected_item_id, sel_idx);
        });

        response
    }
}
