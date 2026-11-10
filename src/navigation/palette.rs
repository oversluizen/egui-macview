//! Authentic macOS Spotlight-Style Modal Command Palette (`Cmd+K` / `Ctrl+K`).
//!
//! Normative Reference: SPEC-007 §4.
//! Dimensions: Floating 480.0 x 280.0pt HUD with fuzzy query search and keyboard navigation.

use crate::theme::MacTheme;
use egui::{
    Align, Align2, Area, CornerRadius, FontId, Frame, Id, Key, Layout, Modifiers, Order, Pos2,
    Rect, ScrollArea, Sense, Stroke, StrokeKind, TextEdit, UiBuilder, Vec2,
};

pub use super::fuzzy::{CommandPaletteItem, IntoShortcutString};
pub use crate::tokens::metrics::navigation::{COMMAND_PALETTE_HEIGHT, COMMAND_PALETTE_WIDTH};

/// Floating modal command palette state and renderer.
///
/// # Example
/// ```rust
/// use egui_macview::{CommandPaletteItem, MacAppearance, MacCommandPalette, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |_ui| {
/// let theme = MacTheme::new(MacAppearance::DarkAqua);
/// let mut palette = MacCommandPalette::new(vec![
///     CommandPaletteItem::new("toggle_theme", "Switch Theme").shortcut("⌘T"),
///     CommandPaletteItem::new("export_pdf", "Export as PDF").shortcut("⌘E"),
/// ])
/// .open(true);
///
/// if let Some(cmd_id) = palette.show(&ctx, &theme) {
///     // Handle triggered command `cmd_id`
/// }
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacCommandPalette {
    /// Whether the command palette is currently open/visible.
    pub open: bool,
    /// Search query string.
    pub query: String,
    /// Selected index in filtered results.
    pub selected_index: usize,
    /// Registered commands.
    pub items: Vec<CommandPaletteItem>,
    /// Width in points.
    pub width: f32,
    /// Height in points.
    pub height: f32,
}

impl Default for MacCommandPalette {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl MacCommandPalette {
    /// Creates a command palette with a collection of available commands.
    pub fn new(items: Vec<CommandPaletteItem>) -> Self {
        Self {
            open: false,
            query: String::new(),
            selected_index: 0,
            items,
            width: COMMAND_PALETTE_WIDTH,
            height: COMMAND_PALETTE_HEIGHT,
        }
    }

    /// Returns whether the command palette is currently visible.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Builder method to set visibility.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Closes the command palette.
    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.selected_index = 0;
    }

    /// Toggles visibility between open and closed.
    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open = true;
            self.query.clear();
            self.selected_index = 0;
        }
    }

    /// Sets the search query programmatically.
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.selected_index = 0;
    }

    /// Returns current query string.
    #[inline]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns the currently selected item index.
    #[inline]
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Computes a fuzzy match score for pattern against text.
    #[inline]
    pub fn fuzzy_match_score(pattern: &str, text: &str) -> Option<i32> {
        super::fuzzy::fuzzy_match_score(pattern, text)
    }

    /// Filters items matching a search query via fuzzy ranking.
    pub fn filter_items<'a>(
        items: &'a [CommandPaletteItem],
        query: &str,
    ) -> Vec<&'a CommandPaletteItem> {
        let q = query.trim();
        if q.is_empty() {
            return items.iter().collect();
        }
        let mut scored: Vec<(i32, usize, &CommandPaletteItem)> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, it)| {
                let mut best_score = None;
                for target in std::iter::once(&it.title)
                    .chain(it.subtitle.as_ref())
                    .chain(it.category.as_ref())
                    .chain(it.keywords.iter())
                {
                    if let Some(s) = Self::fuzzy_match_score(q, target) {
                        best_score = Some(best_score.map_or(s, |curr: i32| curr.max(s)));
                    }
                }
                best_score.map(|s| (s, idx, it))
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        scored.into_iter().map(|(_, _, it)| it).collect()
    }

    /// Returns filtered items matching the active search query via fuzzy ranking.
    pub fn filtered_items(&self) -> Vec<&CommandPaletteItem> {
        Self::filter_items(&self.items, &self.query)
    }

    /// Moves selection to next filtered item.
    pub fn select_next(&mut self) {
        let count = self.filtered_items().len();
        if count > 0 && self.selected_index + 1 < count {
            self.selected_index += 1;
        }
    }

    /// Moves selection to previous filtered item.
    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Executes the currently selected item, returning its action ID and closing the palette.
    pub fn execute_selected(&mut self) -> Option<String> {
        let filtered = self.filtered_items();
        if let Some(item) = filtered.get(self.selected_index) {
            let action_id = item.id.clone();
            self.close();
            Some(action_id)
        } else {
            None
        }
    }

    /// Handles keyboard events (`Cmd+K` / `Ctrl+K`, Arrows, Enter, Escape).
    pub fn handle_input(&mut self, input: &egui::InputState) -> Option<String> {
        let cmd = input.modifiers.command || input.modifiers.ctrl;
        if cmd && input.key_pressed(Key::K) {
            self.toggle();
            return None;
        }
        if !self.open {
            return None;
        }
        if input.key_pressed(Key::Escape) {
            if !self.query.is_empty() {
                self.query.clear();
                self.selected_index = 0;
            } else {
                self.close();
            }
            return None;
        }
        if input.key_pressed(Key::ArrowDown) {
            self.select_next();
            return None;
        }
        if input.key_pressed(Key::ArrowUp) {
            self.select_prev();
            return None;
        }
        if input.key_pressed(Key::Enter) {
            return self.execute_selected();
        }
        None
    }

    /// Renders the modal command palette window, returning action ID if triggered.
    pub fn show(&mut self, ctx: &egui::Context, theme: &MacTheme) -> Option<String> {
        let cmd_toggled = ctx.input_mut(|i| {
            let cmd = i.modifiers.command || i.modifiers.ctrl;
            if cmd && i.key_pressed(Key::K) {
                let mods = i.modifiers;
                i.consume_key(mods, Key::K);
                return true;
            }
            false
        });
        if cmd_toggled {
            self.toggle();
            return None;
        }

        if !self.open {
            return None;
        }

        let key_action = ctx.input_mut(|i| {
            if i.consume_key(Modifiers::NONE, Key::Escape) {
                if !self.query.is_empty() {
                    self.query.clear();
                    self.selected_index = 0;
                } else {
                    self.close();
                }
                return None;
            }
            if i.consume_key(Modifiers::NONE, Key::ArrowDown) {
                self.select_next();
                return None;
            }
            if i.consume_key(Modifiers::NONE, Key::ArrowUp) {
                self.select_prev();
                return None;
            }
            if i.consume_key(Modifiers::NONE, Key::Enter) {
                return self.execute_selected();
            }
            None
        });
        if let Some(action) = key_action {
            return Some(action);
        }
        if !self.open {
            return None;
        }

        let pal = theme.palette_metrics();
        let border_w = theme.popover_metrics().border_width.points();
        let corner_r = pal.corner_radius.points() as u8;
        let header_h = pal.search_bar_height.points();
        let search_margin = pal.search_padding_x.points() * 0.5;
        let search_font_pt = pal.query_font_size.points();
        let row_height = pal.row_height.points();
        let row_corner_r = pal.row_corner_radius.points() as u8;
        let row_inset_x = pal.row_inset.points();
        let row_inset_y = border_w;
        let padding_h = pal.row_padding_x.points();
        let title_font_pt = pal.item_font_size.points();
        let badge_font_pt = pal.category_font_size.points();
        let cat_offset_x = pal.row_padding_x.points() * 2.0;
        let on_accent = theme.on_accent_text();

        let fallback_screen = Vec2::new(self.width * 2.0, self.height * 2.0);
        let screen_rect = ctx
            .input(|i| i.raw.screen_rect)
            .unwrap_or(Rect::from_min_size(Pos2::ZERO, fallback_screen));
        let palette_rect =
            Rect::from_center_size(screen_rect.center(), Vec2::new(self.width, self.height));

        let mut should_close = false;
        let mut backdrop_clicked = false;
        let mut triggered_action = None;

        let backdrop_id = Id::NULL.with("command_palette_backdrop");
        Area::new(backdrop_id)
            .order(Order::Middle)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                let resp = ui.allocate_response(screen_rect.size(), Sense::click());
                ui.painter()
                    .rect_filled(screen_rect, CornerRadius::ZERO, theme.backdrop_dim());
                if resp.clicked() {
                    backdrop_clicked = true;
                }
            });

        if backdrop_clicked {
            self.close();
            return None;
        }

        let modal_id = Id::NULL.with("command_palette_modal");
        Area::new(modal_id)
            .order(Order::Foreground)
            .fixed_pos(palette_rect.min)
            .show(ctx, |ui| {
                let _ = ui.allocate_rect(palette_rect, Sense::click());
                let painter = ui.painter();

                painter.rect(
                    palette_rect,
                    CornerRadius::same(corner_r),
                    theme.window_background(),
                    Stroke::new(border_w, theme.separator()),
                    StrokeKind::Inside,
                );

                let header_rect = Rect::from_min_size(
                    palette_rect.min,
                    Vec2::new(palette_rect.width(), header_h),
                );
                painter.line_segment(
                    [
                        Pos2::new(header_rect.left(), header_rect.bottom()),
                        Pos2::new(header_rect.right(), header_rect.bottom()),
                    ],
                    Stroke::new(border_w, theme.separator()),
                );

                let search_box_rect = header_rect.shrink(search_margin);
                let mut search_ui = ui.new_child(
                    UiBuilder::new()
                        .id_salt("palette_search_ui")
                        .max_rect(search_box_rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );

                let text_edit = TextEdit::singleline(&mut self.query)
                    .hint_text("Type a command or search...")
                    .font(FontId::proportional(search_font_pt))
                    .frame(Frame::NONE)
                    .desired_width(search_box_rect.width());

                let edit_resp = search_ui.add(text_edit);
                edit_resp.request_focus();

                let results_rect = Rect::from_min_max(
                    Pos2::new(palette_rect.min.x, header_rect.bottom() + border_w),
                    palette_rect.max,
                );

                let filtered = Self::filter_items(&self.items, &self.query);
                let filtered_count = filtered.len();
                if self.selected_index >= filtered_count && filtered_count > 0 {
                    self.selected_index = filtered_count - 1;
                }
                let active_selected_idx = self.selected_index;

                let mut list_ui = ui.new_child(
                    UiBuilder::new()
                        .id_salt("palette_list_ui")
                        .max_rect(results_rect)
                        .layout(Layout::top_down(Align::Min)),
                );

                ScrollArea::vertical()
                    .max_height(results_rect.height())
                    .show(&mut list_ui, |scroll_ui| {
                        for (idx, item) in filtered.iter().enumerate() {
                            let (row_rect, resp) = scroll_ui.allocate_exact_size(
                                Vec2::new(results_rect.width(), row_height),
                                Sense::click(),
                            );

                            let is_selected = active_selected_idx == idx;
                            let highlight_rect =
                                row_rect.shrink2(Vec2::new(row_inset_x, row_inset_y));
                            if is_selected {
                                scroll_ui.painter().rect(
                                    highlight_rect,
                                    CornerRadius::same(row_corner_r),
                                    theme.control_accent(),
                                    Stroke::NONE,
                                    StrokeKind::Inside,
                                );
                            } else if resp.hovered() {
                                scroll_ui.painter().rect(
                                    highlight_rect,
                                    CornerRadius::same(row_corner_r),
                                    theme.control(),
                                    Stroke::NONE,
                                    StrokeKind::Inside,
                                );
                            }

                            let text_color = if is_selected {
                                on_accent
                            } else {
                                theme.label()
                            };
                            let sec_color = if is_selected {
                                on_accent
                            } else {
                                theme.secondary_label()
                            };

                            scroll_ui.painter().text(
                                Pos2::new(row_rect.left() + padding_h, row_rect.center().y),
                                Align2::LEFT_CENTER,
                                &item.title,
                                FontId::proportional(title_font_pt),
                                text_color,
                            );

                            if let Some(cat) = &item.category {
                                scroll_ui.painter().text(
                                    Pos2::new(
                                        row_rect.center().x + cat_offset_x,
                                        row_rect.center().y,
                                    ),
                                    Align2::LEFT_CENTER,
                                    cat,
                                    FontId::proportional(badge_font_pt),
                                    sec_color,
                                );
                            }

                            if let Some(sc) = &item.shortcut {
                                scroll_ui.painter().text(
                                    Pos2::new(row_rect.right() - padding_h, row_rect.center().y),
                                    Align2::RIGHT_CENTER,
                                    sc,
                                    FontId::monospace(badge_font_pt),
                                    sec_color,
                                );
                            }

                            if resp.clicked() {
                                triggered_action = Some(item.id.clone());
                                should_close = true;
                            }
                        }
                    });
            });

        if should_close {
            self.close();
        }

        triggered_action
    }
}
