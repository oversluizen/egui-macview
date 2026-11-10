//! Authentic macOS AppKit Combo Box (`NSComboBox` equivalent, REQ-012-03).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].
//!
//! # Engineering Architecture & Apple AppKit Parity
//!
//! An authentic macOS combo box unites an editable text field (`NSTextField`) with a
//! right-anchored popover dropdown trigger (`NSComboBoxCell`), matching AppKit's
//! visual presentation and keyboard accessibility:
//!
//! - **Size-Variant Token Geometry**: Outer height ($24.0\,\text{pt}$ Regular, $20.0\,\text{pt}$ Small,
//!   $16.0\,\text{pt}$ Mini) and button width ($19.0\,\text{pt}$ Regular, $16.0\,\text{pt}$ Small,
//!   $14.0\,\text{pt}$ Mini) match `NSComboBox` metrics.
//! - **Dropdown Chevron**: Symmetrical double-chevron glyph (`ChevronUpDown`) centered inside
//!   the trigger button, highlighting on hover and press.
//! - **Boundary-Clamped Popover**: Displays up to 5 visible items ($18.0\,\text{pt}$ row height,
//!   $90.0\,\text{pt}$ max height) in an overlay `Area`. Automatically flips upward when
//!   approaching the bottom of the screen viewport.
//! - **Full Keyboard Navigation**:
//!   - `Alt + ArrowDown`: Expands the dropdown popover.
//!   - `ArrowDown` / `ArrowUp`: Navigates through filtered items with auto-scroll.
//!   - `Return`: Selects the highlighted item, writes to buffer, and dismisses popover.
//!   - `Escape`: Closes popover without altering buffer text.

use crate::controls::MacControlSize;
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    Area, CornerRadius, Id, Key, Modifiers, Order, Pos2, Rect, Response, ScrollArea, Sense, Stroke,
    StrokeKind, TextEdit, Ui, UiBuilder, Vec2,
};
use std::hash::Hash;

/// Computed layout geometry for the combo box text field and trigger button header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComboBoxHeaderLayout {
    /// Outer bounding rectangle.
    pub total_rect: Rect,
    /// Bounding rectangle for the right-hand dropdown trigger button.
    pub button_rect: Rect,
    /// Bounding rectangle for the editable text input field.
    pub text_rect: Rect,
    /// Resolved corner radius.
    pub corner_radius: f32,
}

/// Resolved geometry for the popover dropdown list, clamped to screen boundaries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComboBoxPopoverGeometry {
    /// Outer popover bounding box.
    pub popover_rect: Rect,
    /// Popover top-left screen position.
    pub popover_pos: Pos2,
    /// Single row height in points.
    pub item_height: f32,
    /// Total popover height in points.
    pub popover_height: f32,
    /// Total popover width in points.
    pub popover_width: f32,
}

/// Authentic macOS AppKit Combo Box (`NSComboBox` equivalent).
///
/// Features an editable text field coupled with a right-anchored dropdown trigger,
/// keyboard navigation (Alt+Down, Up/Down arrows, Return, Escape), and a filterable popover list.
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacComboBox, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let mut font_name = String::from("SF Pro Text");
/// let fonts = ["Helvetica Neue", "Menlo", "SF Mono", "SF Pro Display", "SF Pro Text"];
///
/// MacComboBox::new("font_combo", &mut font_name, &fonts)
///     .width(180.0)
///     .theme(&theme)
///     .show(ui);
/// # });
/// # out.textures_delta.clear();
/// ```
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
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let m = theme_cow.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_height.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_height.points(),
            MacControlSize::Mini => m.mini_height.points(),
        }
    }

    /// Resolves right dropdown trigger button width in points.
    pub fn button_width(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let m = theme_cow.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_button_width.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_button_width.points(),
            MacControlSize::Mini => m.mini_button_width.points(),
        }
    }

    /// Resolves left text padding inset in points.
    pub fn text_inset(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let m = theme_cow.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_inset.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_inset.points(),
            MacControlSize::Mini => m.mini_inset.points(),
        }
    }

    /// Resolves corner radius in points.
    pub fn corner_radius(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let m = theme_cow.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_corner_radius.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_corner_radius.points(),
            MacControlSize::Mini => m.mini_corner_radius.points(),
        }
    }

    /// Resolves popover single item row height in points ($18.0\,\text{pt}$).
    pub fn item_height(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.combo_box().item_height.points()
    }

    /// Resolves visible items count before scrolling ($5$).
    pub fn visible_items(&self) -> usize {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.combo_box().visible_items.points() as usize
    }

    /// Resolves popover viewport height before scrolling ($90.0\,\text{pt}$).
    pub fn popover_viewport_height(&self) -> f32 {
        self.item_height() * (self.visible_items() as f32)
    }

    /// Phase 1: Computes the layout rectangles of the outer frame, text area, and trigger button.
    pub fn compute_header_layout(&self, ui: &Ui) -> ComboBoxHeaderLayout {
        let height = self.frame_height();
        let btn_w = self.button_width();
        let corner_r = self.corner_radius();
        let desired_w = self.width.unwrap_or_else(|| ui.available_width().max(btn_w * 2.0));

        let rect = Rect::from_min_size(ui.cursor().min, Vec2::new(desired_w, height));
        let rect = subpixel_snap_rect(rect, ui.ctx().pixels_per_point());

        let text_inset = self.text_inset();
        let btn_rect = Rect::from_min_max(Pos2::new(rect.max.x - btn_w, rect.min.y), rect.max);
        let text_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + text_inset, rect.min.y),
            Pos2::new(rect.max.x - btn_w, rect.max.y),
        );

        ComboBoxHeaderLayout {
            total_rect: rect,
            button_rect: btn_rect,
            text_rect,
            corner_radius: corner_r,
        }
    }

    /// Phase 2: Computes popover dimensions and screen-edge boundary clamping (with upward-flip detection).
    pub fn compute_popover_geometry(
        &self,
        ui: &Ui,
        header_rect: Rect,
        item_count: usize,
    ) -> ComboBoxPopoverGeometry {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let metrics = theme_cow.combo_box();
        let item_h = metrics.item_height.points();
        let visible_count = (metrics.visible_items.points() as usize).min(item_count);
        let pop_h = item_h * (visible_count as f32);
        let pop_w = header_rect.width();

        let screen_rect = ui
            .ctx()
            .input(|i| i.raw.screen_rect)
            .unwrap_or_else(|| ui.clip_rect());

        // Screen-edge boundary detection: flip upward if dropdown exceeds bottom margin
        let pop_y = if header_rect.max.y + 2.0 + pop_h > screen_rect.bottom()
            && header_rect.min.y - 2.0 - pop_h >= screen_rect.top()
        {
            header_rect.min.y - 2.0 - pop_h
        } else {
            header_rect.max.y + 2.0
        };

        let pop_pos = Pos2::new(header_rect.min.x, pop_y);
        let pop_rect = Rect::from_min_size(pop_pos, Vec2::new(pop_w, pop_h));

        ComboBoxPopoverGeometry {
            popover_rect: pop_rect,
            popover_pos: pop_pos,
            item_height: item_h,
            popover_height: pop_h,
            popover_width: pop_w,
        }
    }

    /// Shows the combo box widget.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.combo_box();

        let layout = self.compute_header_layout(ui);
        let (rect, mut response) = ui.allocate_exact_size(layout.total_rect.size(), Sense::click());
        let rect = subpixel_snap_rect(rect, ui.ctx().pixels_per_point());

        let open_id = self.id_salt.with("popover_open");
        let mut is_open = ui.data_mut(|d| d.get_temp::<bool>(open_id).unwrap_or(false));
        let selected_item_id = self.id_salt.with("popover_sel_idx");
        let stored_sel_idx = ui.data_mut(|d| d.get_temp::<usize>(selected_item_id));
        let mut final_sel_idx = stored_sel_idx.unwrap_or(0);

        let btn_w = self.button_width();
        let text_inset = self.text_inset();
        let btn_rect = Rect::from_min_max(Pos2::new(rect.max.x - btn_w, rect.min.y), rect.max);
        let text_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + text_inset, rect.min.y),
            Pos2::new(rect.max.x - btn_w, rect.max.y),
        );

        if ui.input_mut(|i| i.consume_key(Modifiers::ALT, Key::ArrowDown))
            || ui.input(|i| i.modifiers.alt && i.key_pressed(Key::ArrowDown))
        {
            is_open = true;
        }

        // Render Frame Background and Border
        let painter = ui.painter_at(rect);
        let border_stroke = if response.has_focus() {
            Stroke::new(1.0, theme.control_accent())
        } else {
            Stroke::new(1.0, theme.separator())
        };
        painter.rect(
            rect,
            CornerRadius::same(layout.corner_radius as u8),
            theme.control_background(),
            border_stroke,
            StrokeKind::Inside,
        );

        // Separator hairline between text field and button
        painter.line_segment(
            [Pos2::new(btn_rect.min.x, rect.min.y), Pos2::new(btn_rect.min.x, rect.max.y)],
            Stroke::new(1.0, theme.separator()),
        );

        // Trigger Button Highlight & Chevron Glyph
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let btn_hovered = pointer_pos.is_some_and(|p| btn_rect.contains(p));
        let btn_down = btn_hovered && ui.input(|i| i.pointer.primary_down());

        let btn_radius = CornerRadius {
            nw: 0,
            sw: 0,
            ne: layout.corner_radius as u8,
            se: layout.corner_radius as u8,
        };

        if btn_down {
            painter.rect_filled(btn_rect, btn_radius, theme.selected_control());
        } else if btn_hovered {
            painter.rect_filled(btn_rect, btn_radius, theme.control());
        }

        if btn_hovered && ui.input(|i| i.pointer.primary_clicked()) {
            is_open = !is_open;
        }

        // Draw ChevronUpDown Glyphs
        let (cx, cy) = (btn_rect.center().x, btn_rect.center().y);
        let half_ch = (btn_w * 0.5) * 0.5;
        let chevron_color = if btn_hovered { theme.label() } else { theme.secondary_label() };

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

        // Text input field
        let mut child_ui = ui.new_child(
            UiBuilder::new()
                .id_salt(self.id_salt.with("text_edit"))
                .max_rect(text_rect),
        );
        let edit_res = child_ui.add_sized(text_rect.size(), TextEdit::singleline(self.text).frame(egui::Frame::NONE));
        if edit_res.changed() {
            response.mark_changed();
            is_open = true;
        }

        // Phase 3: Keyboard navigation & item filtering
        if is_open {
            let current_text_lower = self.text.to_lowercase();
            let is_exact_match = self
                .items
                .iter()
                .any(|it| it.as_ref().eq_ignore_ascii_case(self.text.as_str()));
            let filtered: Vec<(usize, &str)> = self
                .items
                .iter()
                .enumerate()
                .map(|(i, item)| (i, item.as_ref()))
                .filter(|(_, s)| {
                    current_text_lower.is_empty()
                        || is_exact_match
                        || s.to_lowercase().contains(&current_text_lower)
                })
                .collect();

            let matching_idx = filtered
                .iter()
                .position(|(_, s)| s.eq_ignore_ascii_case(self.text.as_str()))
                .unwrap_or(0);
            let mut sel_idx = stored_sel_idx
                .unwrap_or(matching_idx)
                .min(filtered.len().saturating_sub(1));

            let mut nav_changed = false;
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                is_open = false;
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                if !filtered.is_empty() {
                    sel_idx = (sel_idx + 1).min(filtered.len() - 1);
                    nav_changed = true;
                }
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) {
                sel_idx = sel_idx.saturating_sub(1);
                nav_changed = true;
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
                if let Some((_, selected_str)) = filtered.get(sel_idx) {
                    *self.text = (*selected_str).to_string();
                    response.mark_changed();
                    is_open = false;
                }
            }

            // Popover rendering
            if is_open && !filtered.is_empty() {
                let pop_geom = self.compute_popover_geometry(ui, rect, filtered.len());
                let area = Area::new(self.id_salt.with("popover_area"))
                    .order(Order::Foreground)
                    .fixed_pos(pop_geom.popover_pos);

                area.show(ui.ctx(), |pop_ui| {
                    pop_ui.set_clip_rect(pop_geom.popover_rect);
                    let pop_painter = pop_ui.painter_at(pop_geom.popover_rect);
                    pop_painter.rect(
                        pop_geom.popover_rect,
                        CornerRadius::same(layout.corner_radius as u8),
                        theme.control_background(),
                        Stroke::new(1.0, theme.separator()),
                        StrokeKind::Inside,
                    );

                    ScrollArea::vertical()
                        .max_height(pop_geom.popover_height)
                        .show(pop_ui, |scroll_ui| {
                            for (idx, (_, item_str)) in filtered.iter().enumerate() {
                                let (row_rect, row_resp) = scroll_ui
                                    .allocate_exact_size(Vec2::new(pop_geom.popover_width, pop_geom.item_height), Sense::click());

                                let is_selected = idx == sel_idx;
                                if is_selected && nav_changed {
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
                        if !rect.contains(pos) && !pop_geom.popover_rect.contains(pos) {
                            is_open = false;
                        }
                    }
                }
                final_sel_idx = sel_idx;
            }
        }

        ui.data_mut(|d| {
            d.insert_temp(open_id, is_open);
            if is_open {
                d.insert_temp(selected_item_id, final_sel_idx);
            } else {
                d.remove_temp::<usize>(selected_item_id);
            }
        });

        response
    }
}
