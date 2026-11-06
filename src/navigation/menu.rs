//! Authentic macOS AppKit `NSMenu` and `NSMenuItem` Top Menu Bar.
//!
//! Normative Reference: SPEC-007 §2.
//! Grounded in macOS HIG keyboard-first accessibility and data-driven menus.

use crate::theme::MacTheme;
use egui::{
    Align2, Area, Color32, CornerRadius, FontId, Key, Order, Pos2, Rect, Sense, Stroke, StrokeKind,
    Ui, Vec2,
};

pub use super::item::{MacKeyboardShortcut, MacMenuItemKind, NSMenuItem};
pub use super::model::NSMenu;

/// Top application menu bar component.
pub struct MacMenuBar<'a> {
    /// Slice of top-level menus (e.g. File, Edit, View, Window, Help).
    pub menus: &'a [NSMenu],
    /// Optional dynamic macOS theme reference.
    pub theme: Option<&'a MacTheme>,
    /// Mutable reference to active open dropdown menu index.
    pub open_menu_idx: &'a mut Option<usize>,
    /// Mutable reference to currently highlighted item index in open menu.
    pub highlighted_item_idx: &'a mut Option<usize>,
}

impl<'a> MacMenuBar<'a> {
    /// Creates a new menu bar.
    pub fn new(
        menus: &'a [NSMenu],
        open_menu_idx: &'a mut Option<usize>,
        highlighted_item_idx: &'a mut Option<usize>,
    ) -> Self {
        Self {
            menus,
            theme: None,
            open_menu_idx,
            highlighted_item_idx,
        }
    }

    /// Sets the dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Processes keyboard shortcuts globally across all menus.
    pub fn handle_shortcuts(&self, input: &egui::InputState) -> Option<String> {
        for menu in self.menus {
            for item in &menu.items {
                if let MacMenuItemKind::Action {
                    id,
                    shortcut: Some(sc),
                } = &item.kind
                {
                    if item.enabled && sc.matches(input) {
                        return Some(id.clone());
                    }
                }
            }
        }
        None
    }

    /// Navigates to the next enabled item in the currently open menu.
    pub fn navigate_next(&mut self) {
        if let Some(m_idx) = *self.open_menu_idx {
            if let Some(menu) = self.menus.get(m_idx) {
                let items_len = menu.items.len();
                if items_len == 0 {
                    return;
                }
                let curr = self.highlighted_item_idx.unwrap_or(0);
                for step in 1..=items_len {
                    let next = (curr + step) % items_len;
                    if let Some(item) = menu.items.get(next) {
                        if item.enabled && !matches!(item.kind, MacMenuItemKind::Separator) {
                            *self.highlighted_item_idx = Some(next);
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Navigates to the previous enabled item in the currently open menu.
    pub fn navigate_prev(&mut self) {
        if let Some(m_idx) = *self.open_menu_idx {
            if let Some(menu) = self.menus.get(m_idx) {
                let items_len = menu.items.len();
                if items_len == 0 {
                    return;
                }
                let curr = self.highlighted_item_idx.unwrap_or(0);
                for step in 1..=items_len {
                    let prev = (curr + items_len - (step % items_len)) % items_len;
                    if let Some(item) = menu.items.get(prev) {
                        if item.enabled && !matches!(item.kind, MacMenuItemKind::Separator) {
                            *self.highlighted_item_idx = Some(prev);
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Closes all open menus.
    pub fn close(&mut self) {
        *self.open_menu_idx = None;
        *self.highlighted_item_idx = None;
    }

    /// Executes the currently highlighted menu item, returning its action ID.
    pub fn activate_highlighted(&mut self) -> Option<String> {
        if let (Some(m_idx), Some(i_idx)) = (*self.open_menu_idx, *self.highlighted_item_idx) {
            if let Some(menu) = self.menus.get(m_idx) {
                if let Some(item) = menu.items.get(i_idx) {
                    if item.enabled {
                        if let MacMenuItemKind::Action { id, .. } = &item.kind {
                            let action = id.clone();
                            self.close();
                            return Some(action);
                        }
                    }
                }
            }
        }
        None
    }

    /// Handles keyboard navigation events when a menu is open.
    pub fn handle_navigation_input(&mut self, input: &egui::InputState) -> Option<String> {
        if self.open_menu_idx.is_none() {
            return None;
        }

        if input.key_pressed(Key::Escape) {
            self.close();
            return None;
        }

        if input.key_pressed(Key::ArrowDown) {
            self.navigate_next();
            return None;
        }

        if input.key_pressed(Key::ArrowUp) {
            self.navigate_prev();
            return None;
        }

        if input.key_pressed(Key::ArrowRight) {
            if let Some(m_idx) = *self.open_menu_idx {
                let next_m = (m_idx + 1) % self.menus.len();
                *self.open_menu_idx = Some(next_m);
                *self.highlighted_item_idx = Some(0);
            }
            return None;
        }

        if input.key_pressed(Key::ArrowLeft) {
            if let Some(m_idx) = *self.open_menu_idx {
                let prev_m = (m_idx + self.menus.len() - 1) % self.menus.len();
                *self.open_menu_idx = Some(prev_m);
                *self.highlighted_item_idx = Some(0);
            }
            return None;
        }

        if input.key_pressed(Key::Enter) || input.key_pressed(Key::Space) {
            return self.activate_highlighted();
        }

        None
    }

    /// Renders the menu bar into the given UI, returning triggered action if any.
    pub fn show(mut self, ui: &mut Ui) -> Option<String> {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        // First check shortcuts
        if let Some(action) = ui.input(|i| self.handle_shortcuts(i)) {
            self.close();
            return Some(action);
        }

        // Then check active navigation keys
        if let Some(action) = ui.input(|i| self.handle_navigation_input(i)) {
            return Some(action);
        }

        let bar_height = theme.popup_button().height.points();
        let font_id = FontId::proportional(theme.popup_button().font_size.points());
        let padding_h = theme.popup_button().padding_horizontal.points();

        let mut triggered_action = None;
        let mut open_rect = None;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (idx, menu) in self.menus.iter().enumerate() {
                let is_open = *self.open_menu_idx == Some(idx);
                let text = &menu.title;
                let text_color = if is_open {
                    Color32::WHITE
                } else {
                    theme.label()
                };

                let galley = ui
                    .painter()
                    .layout_no_wrap(text.clone(), font_id.clone(), text_color);
                let item_width = galley.size().x + padding_h * 2.0;
                let (rect, resp) =
                    ui.allocate_exact_size(Vec2::new(item_width, bar_height), Sense::click());

                if is_open {
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(
                            theme
                                .control_metrics()
                                .button
                                .regular
                                .corner_radius
                                .points() as u8,
                        ),
                        theme.control_accent(),
                        Stroke::NONE,
                        StrokeKind::Inside,
                    );
                    open_rect = Some(rect);
                } else if resp.hovered() {
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(
                            theme
                                .control_metrics()
                                .button
                                .regular
                                .corner_radius
                                .points() as u8,
                        ),
                        theme.control(),
                        Stroke::NONE,
                        StrokeKind::Inside,
                    );
                }

                ui.painter().galley(
                    Pos2::new(
                        rect.min.x + padding_h,
                        rect.center().y - galley.size().y * 0.5,
                    ),
                    galley,
                    text_color,
                );

                if resp.clicked() {
                    if is_open {
                        self.close();
                    } else {
                        *self.open_menu_idx = Some(idx);
                        *self.highlighted_item_idx = Some(0);
                    }
                }
            }
        });

        // If a menu is open, render its dropdown items in Foreground order
        if let (Some(m_idx), Some(anchor)) = (*self.open_menu_idx, open_rect) {
            if let Some(menu) = self.menus.get(m_idx) {
                let metrics = theme.popup_button();
                let item_height = metrics.menu_item_height.points();
                let menu_padding = metrics.menu_item_padding.points();
                let min_width = metrics.menu_min_width.points().max(anchor.width());
                let total_height = (menu.items.len() as f32) * item_height + menu_padding * 2.0;

                let popup_pos = Pos2::new(anchor.left(), anchor.bottom());
                let area_id = ui.scope_id().with("menu_dropdown_area").with(m_idx);

                Area::new(area_id)
                    .order(Order::Foreground)
                    .fixed_pos(popup_pos)
                    .show(ui.ctx(), |pop_ui| {
                        let menu_rect =
                            Rect::from_min_size(popup_pos, Vec2::new(min_width, total_height));
                        let painter = pop_ui.painter();
                        let corner_r = theme.popover_metrics().corner_radius.points() as u8;

                        painter.rect(
                            menu_rect,
                            CornerRadius::same(corner_r),
                            theme.window_background(),
                            Stroke::new(1.0, theme.separator()),
                            StrokeKind::Inside,
                        );

                        for (i_idx, item) in menu.items.iter().enumerate() {
                            let item_y = popup_pos.y + menu_padding + (i_idx as f32) * item_height;
                            let item_rect = Rect::from_min_size(
                                Pos2::new(popup_pos.x + menu_padding, item_y),
                                Vec2::new(min_width - menu_padding * 2.0, item_height),
                            );

                            if matches!(item.kind, MacMenuItemKind::Separator) {
                                painter.line_segment(
                                    [
                                        Pos2::new(item_rect.left(), item_rect.center().y),
                                        Pos2::new(item_rect.right(), item_rect.center().y),
                                    ],
                                    Stroke::new(1.0, theme.separator()),
                                );
                                continue;
                            }

                            let is_highlighted = *self.highlighted_item_idx == Some(i_idx);
                            let resp =
                                pop_ui.interact(item_rect, area_id.with(i_idx), Sense::click());

                            if resp.hovered() && item.enabled {
                                *self.highlighted_item_idx = Some(i_idx);
                            }

                            if is_highlighted && item.enabled {
                                painter.rect(
                                    item_rect,
                                    CornerRadius::same(corner_r),
                                    theme.control_accent(),
                                    Stroke::NONE,
                                    StrokeKind::Inside,
                                );
                            }

                            let item_text_color = if !item.enabled {
                                theme.placeholder_text()
                            } else if is_highlighted {
                                Color32::WHITE
                            } else {
                                theme.label()
                            };

                            painter.text(
                                Pos2::new(item_rect.left() + padding_h, item_rect.center().y),
                                Align2::LEFT_CENTER,
                                &item.title,
                                font_id.clone(),
                                item_text_color,
                            );

                            if let MacMenuItemKind::Action {
                                shortcut: Some(sc), ..
                            } = &item.kind
                            {
                                painter.text(
                                    Pos2::new(item_rect.right() - padding_h, item_rect.center().y),
                                    Align2::RIGHT_CENTER,
                                    sc.format_text(),
                                    font_id.clone(),
                                    item_text_color,
                                );
                            }

                            if resp.clicked() && item.enabled {
                                if let MacMenuItemKind::Action { id, .. } = &item.kind {
                                    triggered_action = Some(id.clone());
                                    self.close();
                                }
                            }
                        }
                    });
            }
        }

        triggered_action
    }
}
