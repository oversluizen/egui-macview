//! Authentic macOS AppKit `NSMenu` and `NSMenuItem` Top Menu Bar.
//!
//! Normative Reference: SPEC-007 §2.
//! Grounded in macOS HIG keyboard-first accessibility and data-driven menus.

use crate::symbols::MacSymbol;
use crate::theme::MacTheme;
use egui::{
    Align2, Area, CornerRadius, FontId, Id, Order, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
};

pub use super::item::{MacKeyboardShortcut, MacMenuItemKind, NSMenuItem};
pub use super::model::NSMenu;

/// Top application menu bar component.
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacMenuBar, MacTheme, NSMenu, NSMenuItem};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let menus = [
///     NSMenu::with_items("File", vec![NSMenuItem::action("new_doc", "New Document")]),
///     NSMenu::with_items("Edit", vec![NSMenuItem::action("undo", "Undo")]),
/// ];
/// let mut open_menu = None;
/// let mut highlighted_item = None;
///
/// MacMenuBar::new(&menus, &mut open_menu, &mut highlighted_item)
///     .theme(&theme)
///     .show(ui);
/// # });
/// # out.textures_delta.clear();
/// ```
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

    /// Processes keyboard shortcuts globally across all menus and submenus.
    pub fn handle_shortcuts(&self, input: &egui::InputState) -> Option<String> {
        self.menus.iter().find_map(|m| m.find_shortcut(input))
    }

    fn step_highlighted(&mut self, forward: bool) {
        let Some(menu) = (*self.open_menu_idx).and_then(|idx| self.menus.get(idx)) else {
            return;
        };
        let curr = Some(self.highlighted_item_idx.unwrap_or(0));
        if let Some(next) = super::submenu::step_enabled_menu_item(&menu.items, curr, forward) {
            *self.highlighted_item_idx = Some(next);
        }
    }

    /// Navigates to the next enabled item in the currently open menu.
    pub fn navigate_next(&mut self) {
        self.step_highlighted(true);
    }

    /// Navigates to the previous enabled item in the currently open menu.
    pub fn navigate_prev(&mut self) {
        self.step_highlighted(false);
    }

    /// Closes all open menus.
    pub fn close(&mut self) {
        *self.open_menu_idx = None;
        *self.highlighted_item_idx = None;
    }

    /// Executes the currently highlighted menu item, returning its action ID.
    pub fn activate_highlighted(&mut self) -> Option<String> {
        let (m_idx, i_idx) = ((*self.open_menu_idx)?, (*self.highlighted_item_idx)?);
        let item = self.menus.get(m_idx)?.items.get(i_idx)?;
        if item.enabled {
            if let MacMenuItemKind::Action { id, .. } = &item.kind {
                let action = id.clone();
                self.close();
                return Some(action);
            }
        }
        None
    }

    /// Handles keyboard navigation events when a menu is open.
    pub fn handle_navigation_input(&mut self, input: &egui::InputState) -> Option<String> {
        let mut sub = None;
        super::submenu::handle_menu_bar_navigation(
            self.menus,
            self.open_menu_idx,
            self.highlighted_item_idx,
            &mut sub,
            input,
        )
    }

    /// Returns the currently active submenu highlight index, if any.
    pub fn submenu_highlight(ctx: &egui::Context) -> Option<usize> {
        let id = Id::NULL.with("mac_menu_bar_submenu_highlight");
        ctx.data(|d| d.get_temp(id))
    }

    /// Renders the menu bar into the given UI, returning triggered action if any.
    pub fn show(mut self, ui: &mut Ui) -> Option<String> {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;

        let sub_temp_id = Id::NULL.with("mac_menu_bar_submenu_highlight");
        let was_open = self.open_menu_idx.is_some();
        let mut sub_highlight: Option<usize> = if was_open {
            ui.ctx().data(|d| d.get_temp(sub_temp_id))
        } else {
            None
        };

        // Guard global shortcut interception when a text editor has keyboard focus
        if !ui.ctx().egui_wants_keyboard_input() && ui.memory(|m| m.focused().is_none()) {
            if let Some(action) =
                ui.input_mut(|i| self.menus.iter().find_map(|m| m.consume_shortcut(i)))
            {
                self.close();
                if was_open {
                    ui.ctx().data_mut(|d| d.remove_temp::<usize>(sub_temp_id));
                }
                return Some(action);
            }
        }

        // Process menu and submenu keyboard navigation (consuming handled navigation keys)
        if was_open {
            let nav_action = ui.input_mut(|i| {
                let act = super::submenu::handle_menu_bar_navigation(
                    self.menus,
                    self.open_menu_idx,
                    self.highlighted_item_idx,
                    &mut sub_highlight,
                    i,
                );
                for nav_key in [
                    egui::Key::Escape,
                    egui::Key::ArrowDown,
                    egui::Key::ArrowUp,
                    egui::Key::ArrowLeft,
                    egui::Key::ArrowRight,
                    egui::Key::Enter,
                    egui::Key::Space,
                ] {
                    let _ = i.consume_key(egui::Modifiers::NONE, nav_key);
                }
                act
            });
            if let Some(action) = nav_action {
                ui.ctx().data_mut(|d| d.remove_temp::<usize>(sub_temp_id));
                return Some(action);
            }

            if let Some(s_idx) = sub_highlight {
                ui.ctx().data_mut(|d| d.insert_temp(sub_temp_id, s_idx));
            } else {
                ui.ctx().data_mut(|d| d.remove_temp::<usize>(sub_temp_id));
            }
        }

        let mb_metrics = theme.menu_bar_metrics();
        let bar_height = mb_metrics.bar_height.points();
        let font_id = FontId::proportional(mb_metrics.font_size.points());
        let padding_h = mb_metrics.item_padding_x.points();
        let item_corner_r = mb_metrics.bar_corner_radius.points() as u8;
        let border_w = theme.popover_metrics().border_width.points();
        let on_accent = theme.on_accent_text();

        let mut triggered_action = None;
        let mut open_rect = None;
        let mut bar_union_rect = Rect::NOTHING;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (idx, menu) in self.menus.iter().enumerate() {
                let is_open = *self.open_menu_idx == Some(idx);
                let text = &menu.title;
                let text_color = if is_open { on_accent } else { theme.label() };

                let galley = ui.painter().layout_no_wrap(
                    text.clone(),
                    font_id.clone(),
                    egui::Color32::PLACEHOLDER,
                );
                let item_width = galley.size().x + padding_h * 2.0;
                let (rect, resp) =
                    ui.allocate_exact_size(Vec2::new(item_width, bar_height), Sense::click());
                bar_union_rect = bar_union_rect.union(rect);

                if is_open {
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same(item_corner_r),
                        theme.control_accent(),
                    );
                    open_rect = Some(rect);
                } else if resp.hovered() {
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same(item_corner_r),
                        theme.control(),
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
                let item_height = mb_metrics.dropdown_item_height.points();
                let menu_padding = mb_metrics.dropdown_padding_y.points();
                let base_min_width = mb_metrics.dropdown_min_width.points().max(anchor.width());
                let total_height = (menu.items.len() as f32) * item_height + menu_padding * 2.0;
                let min_clearance = mb_metrics.shortcut_clearance.points();
                let check_col_w = mb_metrics.dropdown_padding_x.points() * 2.0;
                let has_any_checked = menu.items.iter().any(|it| it.checked);
                let lead_offset = if has_any_checked {
                    check_col_w
                } else {
                    padding_h
                };

                let (max_item_w, item_galleys) = super::submenu::layout_menu_items(
                    ui.painter(),
                    &menu.items,
                    &font_id,
                    lead_offset,
                    padding_h,
                    min_clearance,
                );

                let menu_width = base_min_width.max(max_item_w + menu_padding * 2.0);
                let popup_pos = Pos2::new(anchor.left(), anchor.bottom());
                let menu_rect = Rect::from_min_size(popup_pos, Vec2::new(menu_width, total_height));
                let area_id = ui.scope_id().with("menu_dropdown_area").with(m_idx);
                let mut active_sub_rect: Option<Rect> = None;

                Area::new(area_id)
                    .order(Order::Foreground)
                    .fixed_pos(popup_pos)
                    .show(ui.ctx(), |pop_ui| {
                        let _ = pop_ui.allocate_rect(menu_rect, Sense::hover());
                        let painter = pop_ui.painter();
                        let drop_corner_r = mb_metrics.dropdown_corner_radius.points() as u8;

                        painter.rect(
                            menu_rect,
                            CornerRadius::same(drop_corner_r),
                            theme.window_background(),
                            Stroke::new(border_w, theme.separator()),
                            StrokeKind::Inside,
                        );

                        for ((i_idx, item), galleys) in
                            menu.items.iter().enumerate().zip(item_galleys)
                        {
                            let item_y = popup_pos.y + menu_padding + (i_idx as f32) * item_height;
                            let item_rect = Rect::from_min_size(
                                Pos2::new(popup_pos.x + menu_padding, item_y),
                                Vec2::new(menu_width - menu_padding * 2.0, item_height),
                            );

                            let Some((title_galley, trailing_galley)) = galleys else {
                                painter.line_segment(
                                    [
                                        Pos2::new(item_rect.left(), item_rect.center().y),
                                        Pos2::new(item_rect.right(), item_rect.center().y),
                                    ],
                                    Stroke::new(border_w, theme.separator()),
                                );
                                continue;
                            };

                            let is_highlighted = *self.highlighted_item_idx == Some(i_idx);
                            let resp =
                                pop_ui.interact(item_rect, area_id.with(i_idx), Sense::click());

                            if resp.hovered() && item.enabled {
                                *self.highlighted_item_idx = Some(i_idx);
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

                            let item_text_color = if !item.enabled {
                                theme.placeholder_text()
                            } else if is_highlighted {
                                on_accent
                            } else {
                                theme.label()
                            };

                            if item.checked {
                                painter.text(
                                    Pos2::new(
                                        item_rect.left() + lead_offset * 0.5,
                                        item_rect.center().y,
                                    ),
                                    Align2::CENTER_CENTER,
                                    MacSymbol::Checkmark.as_str(),
                                    font_id.clone(),
                                    item_text_color,
                                );
                            }

                            let title_pos = Align2::LEFT_CENTER
                                .anchor_size(
                                    Pos2::new(item_rect.left() + lead_offset, item_rect.center().y),
                                    title_galley.size(),
                                )
                                .min;
                            painter.galley(title_pos, title_galley, item_text_color);

                            if let Some(tg) = trailing_galley {
                                let tr_pos = Align2::RIGHT_CENTER
                                    .anchor_size(
                                        Pos2::new(
                                            item_rect.right() - padding_h,
                                            item_rect.center().y,
                                        ),
                                        tg.size(),
                                    )
                                    .min;
                                painter.galley(tr_pos, tg, item_text_color);
                            }

                            if resp.clicked() && item.enabled {
                                if let MacMenuItemKind::Action { id, .. } = &item.kind {
                                    triggered_action = Some(id.clone());
                                    self.close();
                                }
                            }

                            if let MacMenuItemKind::Submenu { menu: child_menu } = &item.kind {
                                let sub_is_active = is_highlighted && sub_highlight.is_some();
                                if (is_highlighted || resp.hovered() || sub_is_active)
                                    && item.enabled
                                {
                                    let sub_id = area_id.with("submenu").with(i_idx);
                                    let sub_pos = Pos2::new(popup_pos.x + menu_width, item_y);
                                    let child_hl =
                                        if is_highlighted { sub_highlight } else { None };
                                    active_sub_rect = Some(Rect::from_min_size(
                                        sub_pos,
                                        Vec2::new(
                                            base_min_width,
                                            (child_menu.items.len() as f32) * item_height
                                                + menu_padding * 2.0,
                                        ),
                                    ));
                                    if let Some(act) = super::submenu::render_submenu_popover(
                                        child_menu,
                                        sub_id,
                                        sub_pos,
                                        theme,
                                        pop_ui.ctx(),
                                        child_hl,
                                    ) {
                                        triggered_action = Some(act);
                                        self.close();
                                    }
                                }
                            }
                        }
                    });

                // Dismiss open dropdown if pointer clicked outside bar, dropdown, and submenu
                let clicked_outside = ui.input(|i| {
                    if i.pointer.any_pressed() || i.pointer.any_click() {
                        if let Some(pos) =
                            i.pointer.interact_pos().or_else(|| i.pointer.latest_pos())
                        {
                            let in_bar = bar_union_rect.contains(pos);
                            let in_menu = menu_rect.contains(pos);
                            let in_sub = active_sub_rect.is_some_and(|r| r.contains(pos));
                            return !in_bar && !in_menu && !in_sub;
                        }
                    }
                    false
                });
                if clicked_outside {
                    self.close();
                }
            }
        }

        triggered_action
    }
}
