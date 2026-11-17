//! TabView & Scrollable CollectionView renderer for [`crate::scene::MacSceneIR::show`].

use crate::controls::{
    MacCollectionView, MacCollectionViewFlowLayout, MacControlSize, MacDate, MacDatePicker, MacDatePickerElementFlags,
    MacDatePickerStyle, MacLabel, MacLabelColor, MacPlayerControlsStyle, MacPlayerView, MacTabStyle, MacTabView,
};
use crate::scene::ir::SceneNode;
use crate::scene::runtime::MacSceneDataSource;
use crate::theme::MacTheme;
use egui::{Rect, Ui, UiBuilder};
use std::collections::BTreeSet;

pub(crate) fn render_tab_view_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    ctrl_size: MacControlSize,
) {
    let mut tab_idx = node.value_binding.map(|b| data_source.get_i64(b).max(0) as usize).unwrap_or(0);
    let tv_id = ui.scope_id().with(("nib_tv_state", node.id.0));
    let (mut playing, mut progress, mut grid_sel, mut date) = ui
        .data(|d| d.get_temp::<(bool, f32, BTreeSet<usize>, MacDate)>(tv_id))
        .unwrap_or_else(|| (false, 0.35, BTreeSet::new(), MacDate::default()));

    let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_tv_container", node.id.0)).max_rect(rect));
    c_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
    MacTabView::new(&mut tab_idx)
        .style(MacTabStyle::TopTabsBezelBorder)
        .control_size(ctrl_size)
        .theme(theme)
        .tab("AVPlayer HUD", |tab_ui| {
            tab_ui.vertical(|v_ui| {
                MacLabel::new("QuickTime Floating HUD Pod & Inline Transport:")
                    .color(MacLabelColor::Secondary)
                    .theme(theme)
                    .show(v_ui);
                v_ui.add(
                    MacPlayerView::new()
                        .controls_style(MacPlayerControlsStyle::Floating)
                        .is_playing_mut(&mut playing)
                        .progress_mut(&mut progress)
                        .theme(theme),
                );
            });
        })
        .tab("Collection Grid", |tab_ui| {
            let items: [usize; 60] = std::array::from_fn(|i| i);
            let mut clicked_item: Option<usize> = None;
            crate::layout::MacScrollView::new((tab_ui.scope_id(), "nib_tv_coll_scroll", node.id.0))
                .theme(theme)
                .vertical(true)
                .auto_hide(false)
                .max_width((rect.width() - 24.0).max(160.0))
                .max_height((rect.height() - 48.0).max(96.0))
                .show(tab_ui, |scroll_ui| {
                    let sel_copy = grid_sel.clone();
                    MacCollectionView::new(&items, &mut grid_sel)
                        .layout(MacCollectionViewFlowLayout::default())
                        .theme(theme)
                        .show_with_item_tracker(scroll_ui, |item_ui, item| {
                            let frac = ((*item % 12 + 1) as f32) / 12.0;
                            let (accent, sep) = (theme.control_accent(), theme.separator());
                            item_ui.vertical_centered(|c| {
                                if crate::controls::MacThumbnailCell::new()
                                    .theme(theme)
                                    .selected(sel_copy.contains(item))
                                    .square(26.0)
                                    .custom_draw(move |p, r| {
                                        let (ctr, rad) = (r.center(), r.width() * 0.38);
                                        p.circle_stroke(ctr, rad, egui::Stroke::new(1.0, sep));
                                        let end_a = -std::f32::consts::FRAC_PI_2 + frac * std::f32::consts::TAU;
                                        p.line_segment(
                                            [ctr, ctr + egui::vec2(end_a.cos() * rad, end_a.sin() * rad)],
                                            egui::Stroke::new(1.8, accent),
                                        );
                                        p.circle_filled(ctr + egui::vec2(end_a.cos() * rad, end_a.sin() * rad), 2.2, accent);
                                    })
                                    .show(c)
                                    .clicked()
                                {
                                    clicked_item = Some(*item);
                                }
                                MacLabel::new(format!("#{item}")).control_size(MacControlSize::Mini).theme(theme).show(c);
                            });
                        });
                });
            if let Some(chosen) = clicked_item {
                grid_sel = BTreeSet::from([chosen]);
            }
        })
        .tab("Clock & Calendar", |tab_ui| {
            tab_ui.add(
                MacDatePicker::new()
                    .date(&mut date)
                    .style(MacDatePickerStyle::ClockAndCalendar)
                    .elements(MacDatePickerElementFlags::DateAndTime)
                    .theme(theme),
            );
        })
        .show(&mut c_ui);

    if let Some(b) = node.value_binding {
        data_source.set_i64(b, tab_idx as i64);
    }
    ui.data_mut(|d| d.insert_temp(tv_id, (playing, progress, grid_sel, date)));
}
