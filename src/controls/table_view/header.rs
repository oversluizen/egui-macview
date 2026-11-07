//! `NSTableHeaderView` column definitions and sort direction primitives (SPEC-011).

use egui::Align;

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
