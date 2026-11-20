//! Generic `NSTabView` renderer for [`crate::scene::MacSceneIR::show`] (`REQ-013-07.2`).

use crate::controls::{MacControlSize, MacLabel, MacSearchField, MacTabStyle, MacTabView, MacTableColumn, MacTableView};
use crate::scene::ir::{SceneNode, SceneNodeKind};
use crate::scene::runtime::MacSceneDataSource;
use crate::theme::MacTheme;
use egui::{Align, Layout, Rect, Ui, UiBuilder};
use std::cell::RefCell;
use std::collections::BTreeSet;

pub(crate) fn render_search_field_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    enabled: bool,
) {
    let fb_id = ui.scope_id().with(("nib_srch_str", node.id.0));
    let mut text = node
        .value_binding
        .map(|b| data_source.get_str(b).to_string())
        .unwrap_or_else(|| ui.data(|d| d.get_temp::<String>(fb_id)).unwrap_or_default());
    let placeholder = if node.title.is_empty() { "Search" } else { node.title.as_str() };
    let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_srch_ui", node.id.0)).max_rect(rect));
    let changed = MacSearchField::new(&mut text)
        .id(c_ui.scope_id().with(("nib_srch", node.id.0)))
        .placeholder(placeholder)
        .width(rect.width())
        .height(rect.height())
        .enabled(enabled)
        .theme(theme)
        .show(&mut c_ui)
        .changed();
    if changed && enabled {
        if let Some(b) = node.value_binding {
            data_source.set_str(b, &text);
        } else {
            ui.data_mut(|d| d.insert_temp(fb_id, text));
        }
    }
}

pub(crate) fn render_table_view_node(
    ui: &mut Ui,
    theme: &MacTheme,
    node: &SceneNode,
    rect: Rect,
    columns: &[(String, f32)],
) {
    let mut sel_set = BTreeSet::from([0usize]);
    let min_col_w = theme.table_view().header_height.points();
    let cols: Vec<MacTableColumn<'_>> = if columns.is_empty() {
        vec![MacTableColumn::new("", rect.width())]
    } else {
        columns
            .iter()
            .map(|(title, w)| MacTableColumn::new(title.as_str(), (*w).max(min_col_w)))
            .collect()
    };
    let row_count = usize::from(!node.title.is_empty()).max(1);
    let mut c_ui = ui.new_child(
        UiBuilder::new()
            .id_salt(("nib_tbl_ui", node.id.0))
            .max_rect(rect)
            .layout(Layout::top_down(Align::Min)),
    );
    let _ = MacTableView::new(("nib_tbl", node.id.0), &cols, row_count, &mut sel_set)
        .theme(theme)
        .uses_alternating_row_backgrounds(false)
        .show(&mut c_ui, |cell_ui, _row, col| {
            if col == 0 && !node.title.is_empty() {
                MacLabel::new(node.title.as_str())
                    .control_size(MacControlSize::Small)
                    .theme(theme)
                    .show(cell_ui);
            }
        });
}

pub(crate) fn render_tab_view_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    ctrl_size: MacControlSize,
) {
    let fb_id = ui.scope_id().with(("nib_tv_sel", node.id.0));
    let mut tab_idx = node
        .value_binding
        .map(|b| data_source.get_i64(b).max(0) as usize)
        .unwrap_or_else(|| ui.data(|d| d.get_temp::<usize>(fb_id)).unwrap_or(0));

    let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_tv_container", node.id.0)).max_rect(rect));
    c_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
    let tv_metrics = theme.tab_view();
    let pad_h = tv_metrics.regular_inset_horizontal.points() * 2.0;
    let bar_h = tv_metrics.regular_bar_height.points() + tv_metrics.regular_inset_top.points();
    let min_w = tv_metrics.regular_min_width.points();
    let min_h = tv_metrics.regular_bar_height.points();
    let body_size = egui::vec2((rect.width() - pad_h).max(min_w), (rect.height() - bar_h).max(min_h));
    let tabs = if let SceneNodeKind::TabView { tabs } = &node.kind { tabs.as_slice() } else { &[] };

    let ds_cell = RefCell::new(data_source);
    let ds_ref = &ds_cell;
    let nid = node.id;
    let ib_id = node.ib_object_id.as_str();
    let mut tv = MacTabView::new(&mut tab_idx).style(MacTabStyle::TopTabsBezelBorder).control_size(ctrl_size).theme(theme);
    for (idx, lbl) in tabs.iter().enumerate() {
        tv = tv.tab(lbl.as_str(), move |tab_ui| {
            let body_rect = Rect::from_min_size(tab_ui.cursor().min, body_size);
            ds_ref.borrow_mut().render_tab_item(tab_ui, theme, nid, ib_id, idx, body_rect);
        });
    }
    tv.show(&mut c_ui);

    if let Some(b) = node.value_binding {
        ds_cell.into_inner().set_i64(b, tab_idx as i64);
    } else {
        ui.data_mut(|d| d.insert_temp(fb_id, tab_idx));
    }
}
