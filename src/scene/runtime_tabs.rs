//! Generic `NSTabView` renderer for [`crate::scene::MacSceneIR::show`] (`REQ-013-07.2`).

use crate::controls::{MacControlSize, MacTabStyle, MacTabView};
use crate::scene::ir::{SceneNode, SceneNodeKind};
use crate::scene::runtime::MacSceneDataSource;
use crate::theme::MacTheme;
use egui::{Rect, Ui, UiBuilder};
use std::cell::RefCell;

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
