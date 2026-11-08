//! `NSTableHeaderView` column definitions and header bar renderer (SPEC-011).

use crate::symbols::MacSymbol;
use crate::theme::MacTheme;
use egui::{Align, Align2, FontId, Id, Pos2, Rect, Sense, Stroke, Ui};

/// Sort order indicator for a [`MacTableColumn`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacSortDirection {
    /// Ascending sort order (`ChevronUp`).
    Ascending,
    /// Descending sort order (`ChevronDown`).
    Descending,
}

impl MacSortDirection {
    /// Returns the opposite sort direction (`Ascending` <-> `Descending`).
    pub fn toggled(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

/// Column definition for [`super::MacTableView`].
#[derive(Debug, Clone)]
pub struct MacTableColumn<'a> {
    /// Column header title.
    pub title: &'a str,
    /// Column width in points.
    pub width: f32,
    /// Horizontal cell alignment.
    pub alignment: Align,
    /// Optional active sort direction.
    pub sort_direction: Option<MacSortDirection>,
}

impl<'a> MacTableColumn<'a> {
    /// Creates a new column with the given `title` and `width` in points.
    pub fn new(title: &'a str, width: f32) -> Self {
        Self {
            title,
            width,
            alignment: Align::Min,
            sort_direction: None,
        }
    }

    /// Sets horizontal alignment for cells in this column.
    pub fn align(mut self, alignment: Align) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets the active sort indicator direction for this column.
    pub fn sort(mut self, direction: Option<MacSortDirection>) -> Self {
        self.sort_direction = direction;
        self
    }
}

/// Renders the `NSTableHeaderView` bar across `header_rect` and returns `(clicked_col, toggled_sort)`.
pub(crate) fn render_table_header(
    ui: &mut Ui,
    table_id: Id,
    theme: &MacTheme,
    header_rect: Rect,
    columns: &[MacTableColumn<'_>],
    enabled: bool,
) -> (Option<usize>, Option<(usize, MacSortDirection)>) {
    let tm = theme.table_view();
    let pad_h = tm.cell_padding_horizontal.points();
    let font_sz = tm.header_font_size.points();
    let chevron_sz = tm.sort_chevron_size.points();
    let chevron_gap = tm.sort_chevron_spacing.points();
    let sep_w = tm.separator_width.points();
    let div_inset = tm.header_divider_inset.points();

    let painter = ui.painter_at(header_rect);
    painter.rect_filled(header_rect, 0, theme.control());

    let mut cursor_x = header_rect.left();
    let mut clicked_header = None;
    let mut toggled_sort = None;

    for (col_idx, col) in columns.iter().enumerate() {
        let col_rect = Rect::from_min_max(
            Pos2::new(cursor_x, header_rect.top()),
            Pos2::new(
                (cursor_x + col.width).min(header_rect.right()),
                header_rect.bottom(),
            ),
        );
        let hdr_id = table_id.with(("hdr_col", col_idx));
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let resp = ui.interact(col_rect, hdr_id, sense);

        if resp.hovered() && enabled {
            painter.rect_filled(col_rect, 0, theme.control_background());
        }

        if resp.clicked() && enabled {
            clicked_header = Some(col_idx);
            let next_dir = col
                .sort_direction
                .map(MacSortDirection::toggled)
                .unwrap_or(MacSortDirection::Ascending);
            toggled_sort = Some((col_idx, next_dir));
        }

        let right_reserve = if col.sort_direction.is_some() {
            chevron_sz + pad_h
        } else {
            pad_h
        };
        let title_clip_rect = Rect::from_min_max(
            col_rect.min,
            Pos2::new(
                (col_rect.right() - right_reserve).max(col_rect.left()),
                col_rect.max.y,
            ),
        );
        let title_pos = Pos2::new(col_rect.left() + pad_h, col_rect.center().y);
        painter.with_clip_rect(title_clip_rect).text(
            title_pos,
            Align2::LEFT_CENTER,
            col.title,
            FontId::proportional(font_sz),
            theme.label(),
        );

        if let Some(dir) = col.sort_direction {
            let symbol = match dir {
                MacSortDirection::Ascending => MacSymbol::ChevronUp,
                MacSortDirection::Descending => MacSymbol::ChevronDown,
            };
            let chev_x = (col_rect.right() - pad_h - chevron_gap).max(col_rect.left() + pad_h);
            painter.text(
                Pos2::new(chev_x, col_rect.center().y),
                Align2::RIGHT_CENTER,
                symbol.as_str(),
                FontId::proportional(chevron_sz),
                theme.secondary_label(),
            );
        }

        cursor_x += col.width;
        if col_idx + 1 < columns.len() && cursor_x < header_rect.right() {
            painter.line_segment(
                [
                    Pos2::new(cursor_x, header_rect.top() + div_inset),
                    Pos2::new(cursor_x, header_rect.bottom() - div_inset),
                ],
                Stroke::new(sep_w, theme.separator()),
            );
        }
    }

    painter.line_segment(
        [
            Pos2::new(header_rect.left(), header_rect.bottom()),
            Pos2::new(header_rect.right(), header_rect.bottom()),
        ],
        Stroke::new(sep_w, theme.separator()),
    );

    (clicked_header, toggled_sort)
}
