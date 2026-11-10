//! Authentic macOS AppKit Collection View (`NSCollectionView` equivalent, REQ-012-06).
//!
//! Grounded in Apple AppKit `NSCollectionView` and `NSCollectionViewFlowLayout`
//! with high-performance 2D viewport culling (`ui.clip_rect()`), 2D rubber-band
//! marquee selection, and spatial arrow key navigation.

use crate::controls::MacInsets;
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    CornerRadius, Id, Key, Modifiers, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    UiBuilder, Vec2,
};
use std::collections::BTreeSet;

/// Flow layout engine matching AppKit `NSCollectionViewFlowLayout`.
#[derive(Debug, Clone, PartialEq)]
pub struct MacCollectionViewFlowLayout {
    /// Nominal item dimensions in points ($50.0 \times 50.0\,\text{pt}$).
    pub item_size: Vec2,
    /// Vertical line spacing between rows ($10.0\,\text{pt}$).
    pub minimum_line_spacing: f32,
    /// Horizontal interitem spacing between columns ($10.0\,\text{pt}$).
    pub minimum_interitem_spacing: f32,
    /// Outer section padding insets ($0.0\,\text{pt}$).
    pub section_inset: MacInsets,
}

impl Default for MacCollectionViewFlowLayout {
    fn default() -> Self {
        let theme = MacTheme::default();
        let m = theme.collection_view();
        let sz = m.item_size.points();
        Self {
            item_size: Vec2::new(sz, sz),
            minimum_line_spacing: m.minimum_line_spacing.points(),
            minimum_interitem_spacing: m.minimum_interitem_spacing.points(),
            section_inset: MacInsets {
                left: m.section_inset.points(),
                right: m.section_inset.points(),
                top: m.section_inset.points(),
                bottom: m.section_inset.points(),
            },
        }
    }
}

impl MacCollectionViewFlowLayout {
    /// Computes column count fitting within available viewport width.
    ///
    /// Formula: $C = \lfloor (W_{\text{viewport}} + S_{\text{item}}) / (W_{\text{item}} + S_{\text{item}}) \rfloor$.
    pub fn compute_column_count(&self, available_width: f32) -> usize {
        let span = self.item_size.x + self.minimum_interitem_spacing;
        if span <= 0.0 {
            1
        } else {
            let cols = ((available_width + self.minimum_interitem_spacing) / span).floor();
            (cols as usize).max(1)
        }
    }

    /// Computes spatial 2D bounding rect for an item index.
    pub fn compute_item_rect(&self, origin: Pos2, index: usize, columns: usize) -> Rect {
        let cols = columns.max(1);
        let col = index % cols;
        let row = index / cols;

        let x = origin.x
            + self.section_inset.left
            + (col as f32) * (self.item_size.x + self.minimum_interitem_spacing);
        let y = origin.y
            + self.section_inset.top
            + (row as f32) * (self.item_size.y + self.minimum_line_spacing);

        Rect::from_min_size(Pos2::new(x, y), self.item_size)
    }
}

/// Authentic macOS Collection View (`NSCollectionView` equivalent).
pub struct MacCollectionView<'a, T> {
    items: &'a [T],
    selected_indices: &'a mut BTreeSet<usize>,
    layout: MacCollectionViewFlowLayout,
    theme: Option<&'a MacTheme>,
}

impl<'a, T> MacCollectionView<'a, T> {
    /// Creates a new collection view bound to an item slice and a mutable selection set.
    pub fn new(items: &'a [T], selected_indices: &'a mut BTreeSet<usize>) -> Self {
        Self {
            items,
            selected_indices,
            layout: MacCollectionViewFlowLayout::default(),
            theme: None,
        }
    }

    /// Overrides flow layout parameters.
    pub fn layout(mut self, layout: MacCollectionViewFlowLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Injects dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Shows the collection view and returns count of rendered items within the clip rect along with interaction response.
    pub fn show_with_response<F>(self, ui: &mut Ui, mut render_item: F) -> (usize, Response)
    where
        F: FnMut(&mut Ui, &T),
    {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.collection_view();

        let available_w = ui.available_width();
        let columns = self.layout.compute_column_count(available_w);
        let item_count = self.items.len();
        let rows = if item_count == 0 {
            0
        } else {
            item_count.div_ceil(columns)
        };

        let total_h = self.layout.section_inset.top
            + (rows as f32) * self.layout.item_size.y
            + ((rows.saturating_sub(1)) as f32) * self.layout.minimum_line_spacing
            + self.layout.section_inset.bottom;

        let desired_size = Vec2::new(available_w, total_h.max(self.layout.item_size.y));
        let (rect, mut response) =
            ui.allocate_exact_size(desired_size, Sense::click_and_drag() | Sense::focusable_noninteractive());
        let rect = subpixel_snap_rect(rect, ui.ctx().pixels_per_point());

        // Keyboard 2D spatial navigation when focused
        if response.has_focus() && !self.items.is_empty() {
            let last_selected = self.selected_indices.iter().copied().last().unwrap_or(0);
            let mut next_selected = None;

            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight)) {
                if last_selected + 1 < item_count {
                    next_selected = Some(last_selected + 1);
                }
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft)) {
                if last_selected > 0 {
                    next_selected = Some(last_selected - 1);
                }
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                if last_selected + columns < item_count {
                    next_selected = Some(last_selected + columns);
                }
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) && last_selected >= columns {
                next_selected = Some(last_selected - columns);
            }

            if let Some(new_idx) = next_selected {
                self.selected_indices.clear();
                self.selected_indices.insert(new_idx);
                response.mark_changed();
            }
        }

        let clip = ui.clip_rect();
        let painter = ui.painter_at(rect);
        let sel_corner_r = metrics.selection_corner_radius.points();
        let sel_stroke_w = metrics.selection_stroke_width.points();

        let mut rendered_count = 0usize;

        let row_pitch = self.layout.item_size.y + self.layout.minimum_line_spacing;
        let (first_idx, last_idx) = if row_pitch <= 0.0 || rows == 0 {
            (0, item_count)
        } else {
            let origin_y = rect.min.y + self.layout.section_inset.top;
            let rel_top = (clip.min.y - origin_y).max(0.0);
            let first_row = ((rel_top / row_pitch).floor() as usize).saturating_sub(1);
            let rel_bottom = (clip.max.y - origin_y).max(0.0);
            let last_row = (((rel_bottom / row_pitch).ceil() as usize) + 1).min(rows);
            (
                (first_row * columns).min(item_count),
                (last_row * columns).min(item_count),
            )
        };

        for (idx, item) in self.items[first_idx..last_idx]
            .iter()
            .enumerate()
            .map(|(i, it)| (first_idx + i, it))
        {
            let item_rect = self.layout.compute_item_rect(rect.min, idx, columns);

            // 2D Viewport culling oracle: strictly bypass if item does not intersect clip rect
            if !clip.intersects(item_rect) {
                continue;
            }

            rendered_count += 1;

            let is_selected = self.selected_indices.contains(&idx);
            let item_hovered = ui
                .input(|inp| inp.pointer.hover_pos())
                .is_some_and(|p| item_rect.contains(p));

            if item_hovered && ui.input(|inp| inp.pointer.primary_clicked()) {
                response.request_focus();
                if !ui.input(|inp| inp.modifiers.shift || inp.modifiers.command) {
                    self.selected_indices.clear();
                }
                self.selected_indices.insert(idx);
                response.mark_changed();
            }

            // Draw item background and selection ring
            let bg_color = if is_selected {
                theme.selected_control()
            } else if item_hovered {
                theme.control()
            } else {
                theme.control_background()
            };

            painter.rect(
                item_rect,
                CornerRadius::same(sel_corner_r as u8),
                bg_color,
                Stroke::new(1.0, theme.separator()),
                StrokeKind::Inside,
            );

            if is_selected {
                painter.rect_stroke(
                    item_rect,
                    CornerRadius::same(sel_corner_r as u8),
                    Stroke::new(sel_stroke_w, theme.control_accent()),
                    StrokeKind::Outside,
                );
            }

            // Render inner item
            let mut child_ui = ui.new_child(
                UiBuilder::new()
                    .id_salt(Id::NULL.with("coll_item").with(idx))
                    .max_rect(item_rect),
            );
            render_item(&mut child_ui, item);
        }

        // Rubber-band marquee selection during drag
        if response.dragged() {
            if let (Some(press_pos), Some(current_pos)) = (
                ui.input(|i| i.pointer.press_origin()),
                ui.input(|i| i.pointer.interact_pos()),
            ) {
                let marquee_rect = Rect::from_two_pos(press_pos, current_pos);
                let fill = theme
                    .selected_control()
                    .gamma_multiply(metrics.marquee_alpha_fill.points());
                let stroke = Stroke::new(
                    1.0,
                    theme
                        .control_accent()
                        .gamma_multiply(metrics.marquee_alpha_stroke.points()),
                );

                painter.rect(
                    marquee_rect,
                    CornerRadius::same(0),
                    fill,
                    stroke,
                    StrokeKind::Inside,
                );

                // Compute 2D AABB intersection with candidate row items
                let (m_first_idx, m_last_idx) = if row_pitch <= 0.0 || rows == 0 {
                    (0, item_count)
                } else {
                    let origin_y = rect.min.y + self.layout.section_inset.top;
                    let rel_top = (marquee_rect.min.y - origin_y).max(0.0);
                    let first_row = ((rel_top / row_pitch).floor() as usize).saturating_sub(1);
                    let rel_bottom = (marquee_rect.max.y - origin_y).max(0.0);
                    let last_row = (((rel_bottom / row_pitch).ceil() as usize) + 1).min(rows);
                    (
                        (first_row * columns).min(item_count),
                        (last_row * columns).min(item_count),
                    )
                };

                for idx in m_first_idx..m_last_idx {
                    let item_rect = self.layout.compute_item_rect(rect.min, idx, columns);
                    if marquee_rect.intersects(item_rect) {
                        self.selected_indices.insert(idx);
                        response.mark_changed();
                    }
                }
            }
        }

        (rendered_count, response)
    }

    /// Shows the collection view and returns count of rendered items within the clip rect.
    pub fn show_with_item_tracker<F>(self, ui: &mut Ui, render_item: F) -> usize
    where
        F: FnMut(&mut Ui, &T),
    {
        self.show_with_response(ui, render_item).0
    }

    /// Shows the collection view with default item presentation.
    pub fn show(self, ui: &mut Ui) -> Response {
        self.show_with_response(ui, |_ui, _item| {}).1
    }
}
