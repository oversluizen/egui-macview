//! Authentic macOS Right-Click Context Menu with Viewport Boundary Clamping.
//!
//! Normative Reference: SPEC-008B §3.
//! Enforces mathematical boundary clamping to guarantee zero clipping across screen/window edges.

use crate::navigation::menu::{MacMenuItemKind, NSMenuItem};
use crate::symbols::MacSymbol;
use crate::theme::MacTheme;
use egui::{
    Align2, Area, CornerRadius, FontId, Key, Modifiers, Order, Pos2, Rect, Sense, Stroke,
    StrokeKind, Ui, Vec2,
};

/// Clamps menu origin such that the menu rectangle remains strictly inside the viewport bounds.
pub fn clamp_menu_rect(click_pos: Pos2, menu_size: Vec2, viewport_rect: Rect) -> Rect {
    let effective_size = Vec2::new(
        menu_size.x.min(viewport_rect.width()),
        menu_size.y.min(viewport_rect.height()),
    );
    let mut x = click_pos.x;
    let mut y = click_pos.y;

    if x + effective_size.x > viewport_rect.max.x {
        x = viewport_rect.max.x - effective_size.x;
    }
    if y + effective_size.y > viewport_rect.max.y {
        y = viewport_rect.max.y - effective_size.y;
    }
    x = x.max(viewport_rect.min.x);
    y = y.max(viewport_rect.min.y);

    Rect::from_min_size(Pos2::new(x, y), effective_size)
}

/// Context Menu state and renderer.
pub struct MacContextMenu<'a> {
    /// Slice of menu items to display.
    pub items: &'a [NSMenuItem],
    /// Mutable reference to menu open state.
    pub is_open: &'a mut bool,
    /// Mutable reference to menu screen spawn position.
    pub spawn_pos: &'a mut Pos2,
    /// Mutable reference to currently highlighted menu item index.
    pub selected_idx: &'a mut Option<usize>,
    /// Optional macOS dynamic theme reference.
    pub theme: Option<&'a MacTheme>,
}

impl<'a> MacContextMenu<'a> {
    /// Creates a new context menu.
    pub fn new(
        items: &'a [NSMenuItem],
        is_open: &'a mut bool,
        spawn_pos: &'a mut Pos2,
        selected_idx: &'a mut Option<usize>,
    ) -> Self {
        Self {
            items,
            is_open,
            spawn_pos,
            selected_idx,
            theme: None,
        }
    }

    /// Sets the dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Checks a widget response (such as on a canvas card or outline row)
    /// and activates the context menu if secondary-clicked (right-clicked).
    pub fn attach_to(&mut self, response: &egui::Response) {
        if response.secondary_clicked() {
            *self.is_open = true;
            *self.spawn_pos = response
                .interact_pointer_pos()
                .unwrap_or_else(|| response.rect.left_top());
            *self.selected_idx = None;
        }
    }

    /// Closes the context menu.
    pub fn close(&mut self) {
        *self.is_open = false;
        *self.selected_idx = None;
    }

    fn step_selection(&mut self, forward: bool) {
        let items_len = self.items.len();
        if items_len == 0 {
            return;
        }
        let start = if forward {
            self.selected_idx.map_or(0, |idx| (idx + 1) % items_len)
        } else {
            self.selected_idx
                .map_or(items_len.saturating_sub(1), |idx| {
                    (idx + items_len - 1) % items_len
                })
        };
        for step in 0..items_len {
            let candidate = if forward {
                (start + step) % items_len
            } else {
                (start + items_len - (step % items_len)) % items_len
            };
            if let Some(item) = self.items.get(candidate) {
                if item.enabled && !matches!(item.kind, MacMenuItemKind::Separator) {
                    *self.selected_idx = Some(candidate);
                    break;
                }
            }
        }
    }

    fn activate_selected(&mut self) -> Option<String> {
        if let Some(idx) = *self.selected_idx {
            if let Some(item) = self.items.get(idx) {
                if item.enabled {
                    if let MacMenuItemKind::Action { id, .. } = &item.kind {
                        let action = id.clone();
                        self.close();
                        return Some(action);
                    }
                }
            }
        }
        None
    }

    /// Handles keyboard events (Arrow Down, Arrow Up, Enter, Escape).
    pub fn handle_input(&mut self, input: &egui::InputState) -> Option<String> {
        if !*self.is_open {
            return None;
        }
        if input.key_pressed(Key::Escape) {
            self.close();
            return None;
        }
        if input.key_pressed(Key::ArrowDown) {
            self.step_selection(true);
            return None;
        }
        if input.key_pressed(Key::ArrowUp) {
            self.step_selection(false);
            return None;
        }
        if input.key_pressed(Key::Enter) || input.key_pressed(Key::Space) {
            return self.activate_selected();
        }
        None
    }

    /// Renders the context menu if open, returning action ID if an item was executed.
    pub fn show(mut self, ui: &mut Ui) -> Option<String> {
        if !*self.is_open {
            return None;
        }

        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::from_ctx(ui.ctx());
                &fallback_theme
            }
        };

        let key_action = ui.input_mut(|i| {
            if i.consume_key(Modifiers::NONE, Key::Escape) {
                self.close();
                return None;
            }
            if i.consume_key(Modifiers::NONE, Key::ArrowDown) {
                self.step_selection(true);
                return None;
            }
            if i.consume_key(Modifiers::NONE, Key::ArrowUp) {
                self.step_selection(false);
                return None;
            }
            if i.consume_key(Modifiers::NONE, Key::Enter)
                || i.consume_key(Modifiers::NONE, Key::Space)
            {
                return self.activate_selected();
            }
            None
        });
        if let Some(action) = key_action {
            return Some(action);
        }
        if !*self.is_open {
            return None;
        }

        let cm = theme.context_menu_metrics();
        let item_height = cm.item_height.points();
        let padding = cm.padding.points();
        let padding_h = cm.item_padding_x.points();
        let min_width = cm.min_width.points();
        let corner_r = cm.corner_radius.points() as u8;
        let item_corner_r = cm.row_corner_radius.points() as u8;
        let border_w = theme.popover_metrics().border_width.points();
        let min_clearance = cm.shortcut_clearance.points();
        let font_id = FontId::proportional(cm.font_size.points());
        let on_accent = theme.on_accent_text();
        let has_any_checked = self.items.iter().any(|it| it.checked);
        let check_col_w = padding_h * 2.0;
        let lead_offset = if has_any_checked {
            check_col_w
        } else {
            padding_h
        };

        let mut max_item_w = 0.0f32;
        for item in self.items {
            if matches!(item.kind, MacMenuItemKind::Separator) {
                continue;
            }
            let title_w = ui
                .painter()
                .layout_no_wrap(item.title.clone(), font_id.clone(), on_accent)
                .size()
                .x;
            let sc_w = match &item.kind {
                MacMenuItemKind::Action {
                    shortcut: Some(sc), ..
                } => {
                    ui.painter()
                        .layout_no_wrap(sc.format_text(), font_id.clone(), on_accent)
                        .size()
                        .x
                }
                _ => 0.0,
            };
            let item_w = if sc_w > 0.0 {
                lead_offset + padding_h + title_w + min_clearance + sc_w
            } else {
                lead_offset + padding_h + title_w
            };
            max_item_w = max_item_w.max(item_w);
        }
        let menu_width = min_width.max(max_item_w + padding * 2.0);
        let total_h = (self.items.len() as f32) * item_height + padding * 2.0;

        let viewport_rect = ui
            .ctx()
            .input(|i| i.raw.screen_rect)
            .unwrap_or_else(|| ui.clip_rect());
        let menu_rect = clamp_menu_rect(
            *self.spawn_pos,
            Vec2::new(menu_width, total_h),
            viewport_rect,
        );

        let clicked_outside = ui.input(|i| {
            for ev in &i.events {
                if let egui::Event::PointerButton {
                    pos, pressed: true, ..
                } = ev
                {
                    if !menu_rect.contains(*pos) {
                        return true;
                    }
                }
            }
            false
        });
        if clicked_outside {
            self.close();
            return None;
        }

        let mut triggered = None;
        let area_id = ui.scope_id().with("mac_context_menu_area");

        Area::new(area_id)
            .order(Order::Foreground)
            .fixed_pos(menu_rect.min)
            .show(ui.ctx(), |menu_ui| {
                let _ = menu_ui.allocate_rect(menu_rect, Sense::click());
                let painter = menu_ui.painter();

                painter.rect(
                    menu_rect,
                    CornerRadius::same(corner_r),
                    theme.window_background(),
                    Stroke::new(border_w, theme.separator()),
                    StrokeKind::Inside,
                );

                for (idx, item) in self.items.iter().enumerate() {
                    let item_y = menu_rect.top() + padding + (idx as f32) * item_height;
                    let item_rect = Rect::from_min_size(
                        Pos2::new(menu_rect.left() + padding, item_y),
                        Vec2::new(menu_rect.width() - padding * 2.0, item_height),
                    );

                    if matches!(item.kind, MacMenuItemKind::Separator) {
                        painter.line_segment(
                            [
                                Pos2::new(item_rect.left(), item_rect.center().y),
                                Pos2::new(item_rect.right(), item_rect.center().y),
                            ],
                            Stroke::new(border_w, theme.separator()),
                        );
                        continue;
                    }

                    let is_highlighted = *self.selected_idx == Some(idx);
                    let resp = menu_ui.interact(item_rect, area_id.with(idx), Sense::click());

                    if resp.hovered() && item.enabled {
                        *self.selected_idx = Some(idx);
                    }

                    if is_highlighted && item.enabled {
                        painter.rect(
                            item_rect,
                            CornerRadius::same(item_corner_r),
                            theme.control_accent(),
                            Stroke::NONE,
                            StrokeKind::Inside,
                        );
                    }

                    let fg = if !item.enabled {
                        theme.placeholder_text()
                    } else if is_highlighted {
                        on_accent
                    } else {
                        theme.label()
                    };

                    if item.checked {
                        painter.text(
                            Pos2::new(item_rect.left() + lead_offset * 0.5, item_rect.center().y),
                            Align2::CENTER_CENTER,
                            MacSymbol::Checkmark.as_str(),
                            font_id.clone(),
                            fg,
                        );
                    }

                    painter.text(
                        Pos2::new(item_rect.left() + lead_offset, item_rect.center().y),
                        Align2::LEFT_CENTER,
                        &item.title,
                        font_id.clone(),
                        fg,
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
                            fg,
                        );
                    }

                    if resp.clicked() && item.enabled {
                        if let MacMenuItemKind::Action { id, .. } = &item.kind {
                            triggered = Some(id.clone());
                            self.close();
                        }
                    }
                }
            });

        triggered
    }
}
