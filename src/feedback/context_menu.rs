//! Authentic macOS Right-Click Context Menu with Viewport Boundary Clamping.
//!
//! Normative Reference: SPEC-008 §3.
//! Enforces mathematical boundary clamping to guarantee zero clipping across screen/window edges.

use crate::navigation::menu::NSMenuItem;
use crate::theme::MacTheme;
use egui::{
    Align2, Area, Color32, CornerRadius, FontId, Key, Order, Pos2, Rect, Sense, Stroke, StrokeKind,
    Ui, Vec2,
};

/// Clamps menu origin such that the menu rectangle remains strictly inside the viewport bounds.
pub fn clamp_menu_rect(click_pos: Pos2, menu_size: Vec2, viewport_rect: Rect) -> Rect {
    let effective_size = Vec2::new(
        menu_size.x.min(viewport_rect.width()),
        menu_size.y.min(viewport_rect.height()),
    );
    let mut x = click_pos.x;
    let mut y = click_pos.y;

    // Clamp right edge
    if x + effective_size.x > viewport_rect.max.x {
        x = viewport_rect.max.x - effective_size.x;
    }
    // Clamp bottom edge
    if y + effective_size.y > viewport_rect.max.y {
        y = viewport_rect.max.y - effective_size.y;
    }
    // Clamp left and top edges
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

    /// Handles keyboard events (Arrow Down, Arrow Up, Enter, Escape).
    pub fn handle_input(&mut self, input: &egui::InputState) -> Option<String> {
        if !*self.is_open {
            return None;
        }

        if input.key_pressed(Key::Escape) {
            self.close();
            return None;
        }

        let items_len = self.items.len();
        if items_len == 0 {
            return None;
        }

        if input.key_pressed(Key::ArrowDown) {
            let start = self.selected_idx.map_or(0, |idx| (idx + 1) % items_len);
            for step in 0..items_len {
                let candidate = (start + step) % items_len;
                if let Some(item) = self.items.get(candidate) {
                    if item.enabled
                        && !matches!(
                            item.kind,
                            crate::navigation::menu::MacMenuItemKind::Separator
                        )
                    {
                        *self.selected_idx = Some(candidate);
                        break;
                    }
                }
            }
            return None;
        }

        if input.key_pressed(Key::ArrowUp) {
            let start = self
                .selected_idx
                .map_or(items_len.saturating_sub(1), |idx| {
                    (idx + items_len - 1) % items_len
                });
            for step in 0..items_len {
                let candidate = (start + items_len - (step % items_len)) % items_len;
                if let Some(item) = self.items.get(candidate) {
                    if item.enabled
                        && !matches!(
                            item.kind,
                            crate::navigation::menu::MacMenuItemKind::Separator
                        )
                    {
                        *self.selected_idx = Some(candidate);
                        break;
                    }
                }
            }
            return None;
        }

        if input.key_pressed(Key::Enter) || input.key_pressed(Key::Space) {
            if let Some(idx) = *self.selected_idx {
                if let Some(item) = self.items.get(idx) {
                    if item.enabled {
                        if let crate::navigation::menu::MacMenuItemKind::Action { id, .. } =
                            &item.kind
                        {
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

    /// Renders the context menu if open, returning action ID if an item was executed.
    pub fn show(mut self, ui: &mut Ui) -> Option<String> {
        if !*self.is_open {
            return None;
        }

        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        if let Some(action) = ui.input(|i| self.handle_input(i)) {
            return Some(action);
        }

        let item_height = 22.0f32;
        let padding = 4.0f32;
        let min_width = 140.0f32;
        let total_h = (self.items.len() as f32) * item_height + padding * 2.0;

        let viewport_rect = ui
            .ctx()
            .input(|i| i.raw.screen_rect)
            .unwrap_or_else(|| ui.clip_rect());
        let menu_rect = clamp_menu_rect(
            *self.spawn_pos,
            Vec2::new(min_width, total_h),
            viewport_rect,
        );

        let mut triggered = None;
        let area_id = ui.scope_id().with("mac_context_menu_area");

        Area::new(area_id)
            .order(Order::Foreground)
            .fixed_pos(menu_rect.min)
            .show(ui.ctx(), |menu_ui| {
                let painter = menu_ui.painter();
                let corner_r = 6u8;

                painter.rect(
                    menu_rect,
                    CornerRadius::same(corner_r),
                    theme.window_background(),
                    Stroke::new(1.0, theme.separator()),
                    StrokeKind::Inside,
                );

                let font_id = FontId::proportional(12.0);

                for (idx, item) in self.items.iter().enumerate() {
                    let item_y = menu_rect.top() + padding + (idx as f32) * item_height;
                    let item_rect = Rect::from_min_size(
                        Pos2::new(menu_rect.left() + padding, item_y),
                        Vec2::new(menu_rect.width() - padding * 2.0, item_height),
                    );

                    if matches!(
                        item.kind,
                        crate::navigation::menu::MacMenuItemKind::Separator
                    ) {
                        painter.line_segment(
                            [
                                Pos2::new(item_rect.left(), item_rect.center().y),
                                Pos2::new(item_rect.right(), item_rect.center().y),
                            ],
                            Stroke::new(1.0, theme.separator()),
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
                            CornerRadius::same(4),
                            theme.control_accent(),
                            Stroke::NONE,
                            StrokeKind::Inside,
                        );
                    }

                    let fg = if !item.enabled {
                        theme.placeholder_text()
                    } else if is_highlighted {
                        Color32::WHITE
                    } else {
                        theme.label()
                    };

                    painter.text(
                        Pos2::new(item_rect.left() + 8.0, item_rect.center().y),
                        Align2::LEFT_CENTER,
                        &item.title,
                        font_id.clone(),
                        fg,
                    );

                    if let crate::navigation::menu::MacMenuItemKind::Action {
                        shortcut: Some(sc),
                        ..
                    } = &item.kind
                    {
                        painter.text(
                            Pos2::new(item_rect.right() - 8.0, item_rect.center().y),
                            Align2::RIGHT_CENTER,
                            sc.format_text(),
                            font_id.clone(),
                            fg,
                        );
                    }

                    if resp.clicked() && item.enabled {
                        if let crate::navigation::menu::MacMenuItemKind::Action { id, .. } =
                            &item.kind
                        {
                            triggered = Some(id.clone());
                            self.close();
                        }
                    }
                }
            });

        triggered
    }
}
