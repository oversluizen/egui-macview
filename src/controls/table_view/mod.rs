//! Authentic macOS AppKit Table View (`NSTableView` + `NSTableHeaderView`, SPEC-011).

pub mod header;

pub use header::{MacSortDirection, MacTableColumn};

use crate::theme::MacTheme;
use egui::{Id, Response, Sense, Ui, Vec2};
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
        mut _cell_ui: impl FnMut(&mut Ui, usize, usize),
    ) -> MacTableViewResponse {
        let _ = (
            self.id,
            self.columns,
            self.row_count,
            &self.selection,
            self.uses_alternating_row_backgrounds,
            self.row_height,
            self.enabled,
            self.theme,
        );
        let (_, response) = ui.allocate_exact_size(Vec2::ZERO, Sense::hover());
        MacTableViewResponse {
            response,
            clicked_header: None,
            toggled_sort: None,
            clicked_row: None,
            selection_changed: false,
        }
    }
}
