//! Authentic macOS AppKit Table View (`NSTableView` + `NSTableHeaderView`, SPEC-011).
//!
//! Grounded in Apple AppKit `NSTableView` and W3C DTCG tokens via [`MacTheme`].
//! Features multi-column headers with clickable sort indicators, alternating row backgrounds,
//! single/Shift-range/Cmd-toggle row selection, and consumed keyboard arrow navigation.

pub mod header;

pub use header::{MacSortDirection, MacTableColumn};

use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    CornerRadius, Id, Key, Layout, Modifiers, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    UiBuilder, Vec2,
};
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::hash::Hash;

/// Output returned by [`MacTableView::show`].
#[derive(Debug, Clone)]
pub struct MacTableViewResponse {
    /// Overall widget interaction response.
    pub response: Response,
    /// Index of a column header clicked during this frame, if any.
    pub clicked_header: Option<usize>,
    /// Toggled sort direction for the clicked header, if any.
    pub toggled_sort: Option<(usize, MacSortDirection)>,
    /// Index of the row clicked during this frame, if any.
    pub clicked_row: Option<usize>,
    /// True if the row selection set changed during this frame.
    pub selection_changed: bool,
}

/// Authentic macOS AppKit multi-column data table (`NSTableView` equivalent).
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacSortDirection, MacTableColumn, MacTableView, MacTheme};
/// use std::collections::BTreeSet;
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut selection = BTreeSet::from([0]);
/// let cols = [
///     MacTableColumn::new("Name", 140.0).sort(Some(MacSortDirection::Ascending)),
///     MacTableColumn::new("Duration", 80.0),
/// ];
///
/// MacTableView::new("media_table", &cols, 3, &mut selection)
///     .theme(&theme)
///     .uses_alternating_row_backgrounds(true)
///     .show(ui, |cell_ui, row, col| {
///         cell_ui.label(format!("Cell ({row}, {col})"));
///     });
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacTableView<'a> {
    id: Id,
    columns: &'a [MacTableColumn<'a>],
    row_count: usize,
    selection: &'a mut BTreeSet<usize>,
    uses_alternating_row_backgrounds: bool,
    row_height: Option<f32>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacTableView<'a> {
    /// Creates a new `MacTableView` with `columns`, `row_count`, and mutable `selection` set.
    pub fn new(
        id_source: impl Hash + Debug,
        columns: &'a [MacTableColumn<'a>],
        row_count: usize,
        selection: &'a mut BTreeSet<usize>,
    ) -> Self {
        #[allow(deprecated)]
        let id = Id::new(id_source);
        Self {
            id,
            columns,
            row_count,
            selection,
            uses_alternating_row_backgrounds: true,
            row_height: None,
            enabled: true,
            theme: None,
        }
    }

    /// Configures `usesAlternatingRowBackgroundColors`.
    pub fn uses_alternating_row_backgrounds(mut self, alternating: bool) -> Self {
        self.uses_alternating_row_backgrounds = alternating;
        self
    }

    /// Overrides default row height in points.
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = Some(height);
        self
    }

    /// Enables or disables interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the table view and invokes `cell_ui(ui, row_index, col_index)` for each cell.
    pub fn show(
        self,
        ui: &mut Ui,
        mut cell_ui: impl FnMut(&mut Ui, usize, usize),
    ) -> MacTableViewResponse {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let tm = theme.table_view();
        let hdr_h = tm.header_height.points();
        let row_h = self.row_height.unwrap_or_else(|| tm.row_height.points());
        let pad_h = tm.cell_padding_horizontal.points();
        let pad_v = tm.cell_padding_vertical.points();
        let sep_w = tm.separator_width.points();
        let outer_r = CornerRadius::same(tm.corner_radius.points() as u8);
        let row_r = CornerRadius::same(tm.row_corner_radius.points() as u8);

        let cols_w: f32 = self.columns.iter().map(|c| c.width).sum();
        let total_w = cols_w.max(ui.available_width());
        let total_h = hdr_h + (self.row_count as f32) * row_h;

        let (raw_rect, mut response) =
            ui.allocate_exact_size(Vec2::new(total_w, total_h), Sense::hover());
        let rect = subpixel_snap_rect(raw_rect, ui.ctx().pixels_per_point());

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, outer_r, theme.control_background());

        let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), hdr_h));
        let (clicked_header, toggled_sort) = header::render_table_header(
            ui,
            self.id,
            theme,
            header_rect,
            self.columns,
            self.enabled,
        );

        let anchor_id = self.id.with("selection_anchor");
        let mut anchor = ui
            .data(|d| d.get_temp::<usize>(anchor_id))
            .or_else(|| self.selection.iter().next().copied())
            .unwrap_or(0);

        let mut clicked_row = None;
        let mut selection_changed = false;

        // Keyboard ArrowUp / ArrowDown navigation when enabled and a row is selected or focused
        if self.enabled && self.row_count > 0 && (!self.selection.is_empty() || response.has_focus())
        {
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                let cur = self.selection.iter().next_back().copied().unwrap_or(0);
                let next = (cur + 1).min(self.row_count - 1);
                self.selection.clear();
                self.selection.insert(next);
                anchor = next;
                selection_changed = true;
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) {
                let cur = self.selection.iter().next().copied().unwrap_or(0);
                let prev = cur.saturating_sub(1);
                self.selection.clear();
                self.selection.insert(prev);
                anchor = prev;
                selection_changed = true;
            }
        }

        let body_top = header_rect.bottom();

        for row_idx in 0..self.row_count {
            let row_y = body_top + (row_idx as f32) * row_h;
            let row_rect =
                Rect::from_min_size(Pos2::new(rect.left(), row_y), Vec2::new(rect.width(), row_h));
            let row_id = self.id.with(("row", row_idx));
            let row_sense = if self.enabled {
                Sense::click()
            } else {
                Sense::hover()
            };
            let row_resp = ui.interact(row_rect, row_id, row_sense);

            if row_resp.clicked() && self.enabled {
                clicked_row = Some(row_idx);
                ui.memory_mut(|m| m.request_focus(response.id));
                let mods = ui.input(|i| {
                    let mut m = i.modifiers;
                    for ev in &i.events {
                        if let egui::Event::PointerButton { modifiers, .. } = ev {
                            m |= *modifiers;
                        }
                    }
                    m
                });
                if mods.shift {
                    let lo = anchor.min(row_idx);
                    let hi = anchor.max(row_idx);
                    self.selection.clear();
                    for idx in lo..=hi {
                        self.selection.insert(idx);
                    }
                } else if mods.command || mods.ctrl {
                    if !self.selection.remove(&row_idx) {
                        self.selection.insert(row_idx);
                    }
                    anchor = row_idx;
                } else {
                    self.selection.clear();
                    self.selection.insert(row_idx);
                    anchor = row_idx;
                }
                selection_changed = true;
            }

            let is_selected = self.selection.contains(&row_idx);
            let row_painter = ui.painter_at(row_rect);
            if is_selected {
                let sel_bg = if self.enabled {
                    theme.selected_control()
                } else {
                    theme.selected_control_inactive()
                };
                row_painter.rect_filled(row_rect, row_r, sel_bg);
            } else if row_resp.hovered() && self.enabled {
                row_painter.rect_filled(row_rect, row_r, theme.control());
            } else if self.uses_alternating_row_backgrounds {
                let alt_bg = theme.alternating_content_background(row_idx);
                row_painter.rect_filled(row_rect, 0, alt_bg);
            }

            let mut col_x = row_rect.left();
            for (col_idx, col) in self.columns.iter().enumerate() {
                let cell_rect = Rect::from_min_max(
                    Pos2::new(col_x + pad_h, row_rect.top() + pad_v),
                    Pos2::new(
                        (col_x + col.width - pad_h).max(col_x + pad_h),
                        row_rect.bottom() - pad_v,
                    ),
                );
                let layout = Layout::left_to_right(egui::Align::Center).with_main_align(col.alignment);
                let mut child_ui = ui.new_child(
                    UiBuilder::new()
                        .max_rect(cell_rect)
                        .layout(layout)
                        .id_salt(("cell", row_idx, col_idx)),
                );
                cell_ui(&mut child_ui, row_idx, col_idx);
                col_x += col.width;
            }
        }

        ui.data_mut(|d| d.insert_temp(anchor_id, anchor));

        ui.painter_at(rect).rect_stroke(
            rect,
            outer_r,
            Stroke::new(sep_w, theme.separator()),
            StrokeKind::Inside,
        );

        if selection_changed || clicked_header.is_some() {
            response.mark_changed();
        }

        MacTableViewResponse {
            response,
            clicked_header,
            toggled_sort,
            clicked_row,
            selection_changed,
        }
    }
}
