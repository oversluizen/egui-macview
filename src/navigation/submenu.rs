//! Authentic macOS cascading submenu popover rendering engine.
//!
//! Normative Reference: SPEC-007 §2.3.
//! Grounded in Single Responsibility Principle to maintain modularity ceilings.

use egui::{
    Align2, Area, CornerRadius, FontId, Id, Order, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2,
};

use super::item::MacMenuItemKind;
use super::model::NSMenu;
use crate::theme::MacTheme;

/// Navigation result when processing keyboard input inside a cascading submenu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmenuKeyboardResult {
    /// Action identifier triggered by Enter or Space.
    pub action: Option<String>,
    /// Updated child highlight index.
    pub new_sub_highlight: Option<usize>,
    /// Whether keyboard input was consumed by this submenu level.
    pub consumed: bool,
}

/// Advances or decrements index to the next enabled non-separator item in `items`.
pub(crate) fn step_enabled_menu_item(
    items: &[super::item::NSMenuItem],
    curr: Option<usize>,
    forward: bool,
) -> Option<usize> {
    let len = items.len();
    if len == 0 {
        return None;
    }
    let start = if forward {
        curr.map_or(0, |c| (c + 1) % len)
    } else {
        curr.map_or(len.saturating_sub(1), |c| (c + len - 1) % len)
    };
    for step in 0..len {
        let candidate = if forward {
            (start + step) % len
        } else {
            (start + len - (step % len)) % len
        };
        if let Some(it) = items.get(candidate) {
            if it.enabled && !matches!(it.kind, MacMenuItemKind::Separator) {
                return Some(candidate);
            }
        }
    }
    None
}

/// Advances or decrements index to the next enabled non-separator item in a submenu.
pub fn step_submenu_highlight(
    child_menu: &NSMenu,
    curr: Option<usize>,
    forward: bool,
) -> Option<usize> {
    step_enabled_menu_item(&child_menu.items, curr, forward)
}

/// Processes keyboard navigation events targeting an active submenu.
pub fn handle_submenu_input(
    child_menu: &NSMenu,
    sub_highlight: Option<usize>,
    input: &egui::InputState,
) -> SubmenuKeyboardResult {
    if let Some(curr) = sub_highlight {
        if input.key_pressed(egui::Key::Escape) || input.key_pressed(egui::Key::ArrowLeft) {
            return SubmenuKeyboardResult {
                action: None,
                new_sub_highlight: None,
                consumed: true,
            };
        }
        if input.key_pressed(egui::Key::ArrowDown) {
            let next = step_submenu_highlight(child_menu, Some(curr), true);
            return SubmenuKeyboardResult {
                action: None,
                new_sub_highlight: next.or(Some(curr)),
                consumed: true,
            };
        }
        if input.key_pressed(egui::Key::ArrowUp) {
            let prev = step_submenu_highlight(child_menu, Some(curr), false);
            return SubmenuKeyboardResult {
                action: None,
                new_sub_highlight: prev.or(Some(curr)),
                consumed: true,
            };
        }
        if input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space) {
            if let Some(item) = child_menu.items.get(curr) {
                if item.enabled {
                    if let MacMenuItemKind::Action { id, .. } = &item.kind {
                        return SubmenuKeyboardResult {
                            action: Some(id.clone()),
                            new_sub_highlight: None,
                            consumed: true,
                        };
                    }
                }
            }
            return SubmenuKeyboardResult {
                action: None,
                new_sub_highlight: Some(curr),
                consumed: true,
            };
        }
    }
    SubmenuKeyboardResult {
        action: None,
        new_sub_highlight: sub_highlight,
        consumed: false,
    }
}

/// Pre-laid-out text galleys for a single non-separator menu item (`Color32::PLACEHOLDER`).
pub(crate) type MenuItemGalleys = (
    std::sync::Arc<egui::Galley>,
    Option<std::sync::Arc<egui::Galley>>,
);

/// Lays out each menu item once with `Color32::PLACEHOLDER` and returns `(max_item_width, galleys)`.
pub(crate) fn layout_menu_items(
    painter: &egui::Painter,
    items: &[crate::navigation::item::NSMenuItem],
    font_id: &FontId,
    lead_offset: f32,
    padding_h: f32,
    min_clearance: f32,
) -> (f32, Vec<Option<MenuItemGalleys>>) {
    let mut max_item_w = 0.0f32;
    let mut laid_out = Vec::with_capacity(items.len());
    for item in items {
        if matches!(item.kind, MacMenuItemKind::Separator) {
            laid_out.push(None);
            continue;
        }
        let title_galley = painter.layout_no_wrap(
            item.title.clone(),
            font_id.clone(),
            egui::Color32::PLACEHOLDER,
        );
        let title_w = title_galley.size().x;
        let trailing_galley = match &item.kind {
            MacMenuItemKind::Action {
                shortcut: Some(sc), ..
            } => Some(painter.layout_no_wrap(
                sc.format_text(),
                font_id.clone(),
                egui::Color32::PLACEHOLDER,
            )),
            MacMenuItemKind::Submenu { .. } => Some(painter.layout_no_wrap(
                "›".to_owned(),
                font_id.clone(),
                egui::Color32::PLACEHOLDER,
            )),
            _ => None,
        };
        let trailing_w = trailing_galley.as_ref().map_or(0.0, |g| g.size().x);
        let item_w = if trailing_w > 0.0 {
            lead_offset + padding_h + title_w + min_clearance + trailing_w * 2.0
        } else {
            lead_offset + padding_h + title_w
        };
        max_item_w = max_item_w.max(item_w);
        laid_out.push(Some((title_galley, trailing_galley)));
    }
    (max_item_w, laid_out)
}

/// Renders a cascading submenu popover adjacent to its parent menu item.
pub fn render_submenu_popover(
    child_menu: &NSMenu,
    sub_id: Id,
    sub_pos: Pos2,
    theme: &MacTheme,
    ctx: &egui::Context,
    highlighted_idx: Option<usize>,
) -> Option<String> {
    let mb_metrics = theme.menu_bar_metrics();
    let item_height = mb_metrics.dropdown_item_height.points();
    let menu_padding = mb_metrics.dropdown_padding_y.points();
    let padding_h = mb_metrics.item_padding_x.points();
    let base_sub_min_w = mb_metrics.dropdown_min_width.points();
    let sub_total_h = (child_menu.items.len() as f32) * item_height + menu_padding * 2.0;
    let drop_corner_r = mb_metrics.dropdown_corner_radius.points() as u8;
    let item_corner_r = mb_metrics.bar_corner_radius.points() as u8;
    let border_w = theme.popover_metrics().border_width.points();
    let min_clearance = mb_metrics.shortcut_clearance.points();
    let font_id = FontId::proportional(mb_metrics.font_size.points());
    let on_accent = theme.on_accent_text();

    let mut triggered = None;

    Area::new(sub_id)
        .order(Order::Foreground)
        .fixed_pos(sub_pos)
        .show(ctx, |sub_ui| {
            let (max_item_w, item_galleys) = layout_menu_items(
                sub_ui.painter(),
                &child_menu.items,
                &font_id,
                padding_h,
                padding_h,
                min_clearance,
            );

            let sub_menu_w = base_sub_min_w.max(max_item_w + menu_padding * 2.0);
            let sub_rect = Rect::from_min_size(sub_pos, Vec2::new(sub_menu_w, sub_total_h));
            let _ = sub_ui.allocate_rect(sub_rect, Sense::hover());
            let painter = sub_ui.painter();
            painter.rect(
                sub_rect,
                CornerRadius::same(drop_corner_r),
                theme.window_background(),
                Stroke::new(border_w, theme.separator()),
                StrokeKind::Inside,
            );

            for ((s_idx, sub_item), galleys) in
                child_menu.items.iter().enumerate().zip(item_galleys)
            {
                let s_y = sub_pos.y + menu_padding + (s_idx as f32) * item_height;
                let s_rect = Rect::from_min_size(
                    Pos2::new(sub_pos.x + menu_padding, s_y),
                    Vec2::new(sub_menu_w - menu_padding * 2.0, item_height),
                );

                let Some((title_galley, trailing_galley)) = galleys else {
                    painter.line_segment(
                        [
                            Pos2::new(s_rect.left(), s_rect.center().y),
                            Pos2::new(s_rect.right(), s_rect.center().y),
                        ],
                        Stroke::new(border_w, theme.separator()),
                    );
                    continue;
                };

                let s_resp = sub_ui.interact(s_rect, sub_id.with(s_idx), Sense::click());
                let is_highlighted = highlighted_idx == Some(s_idx);
                let s_active = (s_resp.hovered() || is_highlighted) && sub_item.enabled;

                if s_active {
                    painter.rect(
                        s_rect,
                        CornerRadius::same(item_corner_r),
                        theme.control_accent(),
                        Stroke::NONE,
                        StrokeKind::Inside,
                    );
                }

                let s_color = if !sub_item.enabled {
                    theme.placeholder_text()
                } else if s_active {
                    on_accent
                } else {
                    theme.label()
                };

                let title_pos = Align2::LEFT_CENTER
                    .anchor_size(
                        Pos2::new(s_rect.left() + padding_h, s_rect.center().y),
                        title_galley.size(),
                    )
                    .min;
                painter.galley(title_pos, title_galley, s_color);

                if let Some(tg) = trailing_galley {
                    let tr_pos = Align2::RIGHT_CENTER
                        .anchor_size(
                            Pos2::new(s_rect.right() - padding_h, s_rect.center().y),
                            tg.size(),
                        )
                        .min;
                    painter.galley(tr_pos, tg, s_color);
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

/// Orchestrates keyboard navigation across top-level menus and cascading submenus.
pub fn handle_menu_bar_navigation(
    menus: &[NSMenu],
    open_menu_idx: &mut Option<usize>,
    highlighted_item_idx: &mut Option<usize>,
    sub_highlight: &mut Option<usize>,
    input: &egui::InputState,
) -> Option<String> {
    let m_idx = (*open_menu_idx)?;
    let menu = menus.get(m_idx)?;

    let current_is_sub = highlighted_item_idx
        .and_then(|i| menu.items.get(i))
        .and_then(|it| match &it.kind {
            MacMenuItemKind::Submenu { menu: c } => Some(c),
            _ => None,
        });

    if let Some(child_menu) = current_is_sub {
        let res = handle_submenu_input(child_menu, *sub_highlight, input);
        if res.consumed {
            *sub_highlight = res.new_sub_highlight;
            if let Some(act) = res.action {
                *open_menu_idx = None;
                *highlighted_item_idx = None;
                *sub_highlight = None;
                return Some(act);
            }
            return None;
        } else if sub_highlight.is_none() && input.key_pressed(egui::Key::ArrowRight) {
            *sub_highlight = step_submenu_highlight(child_menu, None, true);
            return None;
        }
    }

    if sub_highlight.is_none() {
        if input.key_pressed(egui::Key::Escape) {
            *open_menu_idx = None;
            *highlighted_item_idx = None;
            return None;
        }
        if input.key_pressed(egui::Key::ArrowRight) {
            let next_m = (m_idx + 1) % menus.len();
            *open_menu_idx = Some(next_m);
            *highlighted_item_idx = Some(0);
            return None;
        }
        if input.key_pressed(egui::Key::ArrowLeft) {
            let prev_m = (m_idx + menus.len() - 1) % menus.len();
            *open_menu_idx = Some(prev_m);
            *highlighted_item_idx = Some(0);
            return None;
        }
        if input.key_pressed(egui::Key::ArrowDown) {
            if let Some(next) =
                step_enabled_menu_item(&menu.items, Some(highlighted_item_idx.unwrap_or(0)), true)
            {
                *highlighted_item_idx = Some(next);
            }
            return None;
        }
        if input.key_pressed(egui::Key::ArrowUp) {
            if let Some(prev) =
                step_enabled_menu_item(&menu.items, Some(highlighted_item_idx.unwrap_or(0)), false)
            {
                *highlighted_item_idx = Some(prev);
            }
            return None;
        }
        if input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space) {
            if let Some(i_idx) = *highlighted_item_idx {
                if let Some(item) = menu.items.get(i_idx) {
                    if item.enabled {
                        if let MacMenuItemKind::Action { id, .. } = &item.kind {
                            let act = id.clone();
                            *open_menu_idx = None;
                            *highlighted_item_idx = None;
                            return Some(act);
                        }
                    }
                }
            }
        }
    }

    None
}
