//! Authentic macOS Spotlight-Style Modal Command Palette (`Cmd+K` / `Ctrl+K`).
//!
//! Normative Reference: SPEC-007 §4.
//! Dimensions: Floating 480.0 x 280.0pt HUD with fuzzy query search and keyboard navigation.

use crate::theme::MacTheme;
use egui::{
    Align2, Area, Color32, CornerRadius, FontId, Key, Order, Pos2, Rect, ScrollArea, Sense, Stroke,
    StrokeKind, TextEdit, Vec2,
};

use super::item::MacKeyboardShortcut;

/// Default command palette width in points.
pub const COMMAND_PALETTE_WIDTH: f32 = 480.0;
/// Default command palette height in points.
pub const COMMAND_PALETTE_HEIGHT: f32 = 280.0;

/// Trait for types that can be converted into a shortcut string representation.
pub trait IntoShortcutString {
    /// Converts into a human-readable shortcut string.
    fn into_shortcut_string(self) -> String;
}

impl IntoShortcutString for String {
    fn into_shortcut_string(self) -> String {
        self
    }
}

impl IntoShortcutString for &str {
    fn into_shortcut_string(self) -> String {
        self.to_string()
    }
}

impl IntoShortcutString for MacKeyboardShortcut {
    fn into_shortcut_string(self) -> String {
        self.format_text()
    }
}

/// Individual registered command entry.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandPaletteItem {
    /// Action identifier returned when command is executed.
    pub id: String,
    /// Human-readable command title label.
    pub title: String,
    /// Optional explanatory subtitle.
    pub subtitle: Option<String>,
    /// Optional command group/category name (e.g. "File", "View").
    pub category: Option<String>,
    /// Optional keyboard shortcut display string (e.g. "Ctrl+S").
    pub shortcut: Option<String>,
    /// Keywords for query search matching.
    pub keywords: Vec<String>,
}

impl CommandPaletteItem {
    /// Constructs a basic command palette item.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            category: None,
            shortcut: None,
            keywords: Vec::new(),
        }
    }

    /// Sets the subtitle.
    pub fn subtitle(mut self, sub: impl Into<String>) -> Self {
        self.subtitle = Some(sub.into());
        self
    }

    /// Sets the category group.
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = Some(cat.into());
        self
    }

    /// Sets keyboard shortcut display string or MacKeyboardShortcut.
    pub fn shortcut(mut self, sc: impl IntoShortcutString) -> Self {
        self.shortcut = Some(sc.into_shortcut_string());
        self
    }

    /// Sets search keywords.
    pub fn keywords(mut self, kw: &[impl AsRef<str>]) -> Self {
        self.keywords = kw.iter().map(|s| s.as_ref().to_string()).collect();
        self
    }
}

/// Floating modal command palette state and renderer.
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
    /// Returns `Some(score)` if pattern is a subsequence of text (case-insensitive), or `None` otherwise.
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

        // Sort descending by score, tie-break by original index
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

    /// Handles keyboard events (Cmd+K / Ctrl+K, Arrows, Enter, Escape).
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
            self.close();
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
        if !self.open {
            return None;
        }

        let mut triggered_action = None;

        // Check global shortcuts first
        if let Some(action) = ctx.input(|i| self.handle_input(i)) {
            return Some(action);
        }

        let screen_rect = ctx
            .input(|i| i.raw.screen_rect)
            .unwrap_or(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0)));
        let palette_rect =
            Rect::from_center_size(screen_rect.center(), Vec2::new(self.width, self.height));

        let mut should_close = false;
        let mut backdrop_clicked = false;

        // Backdrop dimmer
        #[allow(deprecated)]
        let backdrop_id = egui::Id::new("command_palette_backdrop");
        Area::new(backdrop_id)
            .order(Order::Middle)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                let resp = ui.allocate_response(screen_rect.size(), Sense::click());
                ui.painter().rect_filled(
                    screen_rect,
                    CornerRadius::ZERO,
                    Color32::from_black_alpha(100),
                );
                if resp.clicked() {
                    backdrop_clicked = true;
                }
            });

        if backdrop_clicked {
            self.close();
            return None;
        }

        // Floating Palette Modal
        #[allow(deprecated)]
        let modal_id = egui::Id::new("command_palette_modal");
        Area::new(modal_id)
            .order(Order::Foreground)
            .fixed_pos(palette_rect.min)
            .show(ctx, |ui| {
                let painter = ui.painter();
                let corner_r = 10u8;

                // Window background and stroke
                painter.rect(
                    palette_rect,
                    CornerRadius::same(corner_r),
                    theme.window_background(),
                    Stroke::new(1.0, theme.separator()),
                    StrokeKind::Inside,
                );

                let header_h = 36.0f32;
                let header_rect = Rect::from_min_size(
                    palette_rect.min,
                    Vec2::new(palette_rect.width(), header_h),
                );

                // Bottom divider under search bar
                painter.line_segment(
                    [
                        Pos2::new(header_rect.left(), header_rect.bottom()),
                        Pos2::new(header_rect.right(), header_rect.bottom()),
                    ],
                    Stroke::new(1.0, theme.separator()),
                );

                // Search field text edit
                let search_margin = 8.0f32;
                let search_box_rect = header_rect.shrink(search_margin);
                let mut search_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(search_box_rect)
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                );

                let text_edit = TextEdit::singleline(&mut self.query)
                    .hint_text("Type a command or search...")
                    .font(FontId::proportional(14.0))
                    .frame(egui::Frame::NONE)
                    .desired_width(search_box_rect.width());

                let edit_resp = search_ui.add(text_edit);
                edit_resp.request_focus();

                // Results list
                let results_rect = Rect::from_min_max(
                    Pos2::new(palette_rect.min.x, header_rect.bottom() + 1.0),
                    palette_rect.max,
                );

                let filtered_count = Self::filter_items(&self.items, &self.query).len();

                if self.selected_index >= filtered_count && filtered_count > 0 {
                    self.selected_index = filtered_count - 1;
                }
                let active_selected_idx = self.selected_index;

                let filtered = Self::filter_items(&self.items, &self.query);

                let row_height = 28.0f32;
                let padding_h = 12.0f32;

                let mut list_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(results_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
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
                            if is_selected {
                                scroll_ui.painter().rect(
                                    row_rect.shrink2(Vec2::new(4.0, 1.0)),
                                    CornerRadius::same(5),
                                    theme.control_accent(),
                                    Stroke::NONE,
                                    StrokeKind::Inside,
                                );
                            } else if resp.hovered() {
                                scroll_ui.painter().rect(
                                    row_rect.shrink2(Vec2::new(4.0, 1.0)),
                                    CornerRadius::same(5),
                                    theme.control(),
                                    Stroke::NONE,
                                    StrokeKind::Inside,
                                );
                            }

                            let text_color = if is_selected {
                                Color32::WHITE
                            } else {
                                theme.label()
                            };

                            let sec_color = if is_selected {
                                Color32::WHITE
                            } else {
                                theme.secondary_label()
                            };

                            // Title
                            scroll_ui.painter().text(
                                Pos2::new(row_rect.left() + padding_h, row_rect.center().y),
                                Align2::LEFT_CENTER,
                                &item.title,
                                FontId::proportional(12.5),
                                text_color,
                            );

                            // Category badge
                            if let Some(cat) = &item.category {
                                scroll_ui.painter().text(
                                    Pos2::new(row_rect.center().x + 20.0, row_rect.center().y),
                                    Align2::LEFT_CENTER,
                                    cat,
                                    FontId::proportional(10.5),
                                    sec_color,
                                );
                            }

                            // Shortcut badge
                            if let Some(sc) = &item.shortcut {
                                scroll_ui.painter().text(
                                    Pos2::new(row_rect.right() - padding_h, row_rect.center().y),
                                    Align2::RIGHT_CENTER,
                                    sc,
                                    FontId::monospace(10.5),
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
