//! Generic `NSTabView` renderer for [`crate::scene::MacSceneIR::show`] (`REQ-013-07.2`).

use crate::controls::{
    MacControlSize, MacHelpButton, MacImageView, MacInlineActionButton, MacLabel, MacSearchField, MacSegmentedControl, MacTabStyle,
    MacTabView, MacTableColumn, MacTableView, MacTokenField, SegmentItem,
};
use crate::navigation::MacToolbar;
use crate::scene::ir::{SceneNode, SceneNodeKind};
use crate::scene::runtime::MacSceneDataSource;
use crate::theme::MacTheme;
use egui::{Align, Layout, Rect, Ui, UiBuilder};
use std::cell::RefCell;
use std::collections::BTreeSet;

pub(crate) fn render_window_titlebar_container(ui: &mut Ui, theme: &MacTheme, node: &SceneNode, rect: Rect) -> bool {
    let is_win =
        node.parent.is_none() && node.custom_class.as_deref().is_some_and(|c| c.ends_with("Window") || c.ends_with("WindowTemplate"));
    if is_win {
        let _ = MacToolbar::new("").id(("nib_win_tb", node.id.0)).theme(theme).show_in_rect(ui, rect, |_tb_ui| {});
        theme.register_surface_rect(ui.ctx(), rect, theme.window_background());
    }
    is_win
}

pub(crate) fn render_segmented_control_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    segments: &[String],
    (ctrl_size, enabled): (MacControlSize, bool),
) {
    if segments.is_empty() {
        return;
    }
    let fb_id = ui.scope_id().with(("nib_seg", node.id.0));
    let sel = node
        .value_binding
        .map(|b| data_source.get_i64(b).max(0) as usize)
        .unwrap_or_else(|| ui.data(|d| d.get_temp::<usize>(fb_id)).unwrap_or(0));
    let is_icon_strip = segments.iter().any(|s| {
        let base = s.split('|').next().unwrap_or(s.as_str());
        base.ends_with("Template") || base.ends_with("Icon") || base.starts_with("sidebar.")
    });
    let mut next_sel = None;
    let seg_w = rect.width() / (segments.len() as f32);
    if is_icon_strip {
        let active_idx = sel.min(segments.len().saturating_sub(1));
        if node.is_bordered || node.is_bezeled {
            let items: Vec<SegmentItem<'_>> =
                segments.iter().map(|s| SegmentItem::new("").with_width(seg_w).with_submenu(s.ends_with("|menu"))).collect();
            MacSegmentedControl::with_custom_segments(items, sel)
                .id_salt(("nib_seg_ctrl", node.id.0))
                .theme(theme)
                .control_size(ctrl_size)
                .enabled(enabled)
                .fixed_rect(rect)
                .show(ui, |idx| next_sel = Some(idx));
        } else {
            let resp = ui.interact(
                rect,
                ui.scope_id().with(("nib_unb_seg", node.id.0)),
                if enabled { egui::Sense::click() } else { egui::Sense::hover() },
            );
            if resp.clicked() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    let idx = (((pos.x - rect.min.x) / seg_w).floor().max(0.0) as usize).min(segments.len().saturating_sub(1));
                    next_sel = Some(idx);
                }
            }
        }
        let seg_tok = theme.segmented();
        let icon_pad = seg_tok.icon_padding.points();
        let painter = ui.painter_at(rect.intersect(ui.clip_rect()));
        for (i, raw) in segments.iter().enumerate() {
            let icon_name = raw.split('|').next().unwrap_or(raw.as_str());
            let slot_rect = Rect::from_min_size(egui::pos2(rect.min.x + (i as f32) * seg_w, rect.min.y), egui::vec2(seg_w, rect.height()));
            let tint = if !enabled {
                theme.tertiary_label()
            } else if i == active_idx {
                theme.label()
            } else {
                theme.secondary_label()
            };
            let ic_rect = if !node.is_bordered && !node.is_bezeled {
                slot_rect.shrink2(egui::vec2(icon_pad * 0.5, 0.0))
            } else {
                slot_rect.shrink(icon_pad)
            };
            if !data_source.paint_scene_icon_scaled(ui, theme, icon_name, ic_rect, tint, (node.image_scaling, node.symbol_point_size)) {
                MacImageView::from_resource_name(icon_name).theme(theme).template(true).tint(tint).scaling(crate::controls::MacImageScaling::from_nib_code(node.image_scaling)).symbol_point_size(node.symbol_point_size).fixed_rect(ic_rect).show(ui);
            }
            if raw.ends_with("|menu") && !node.is_bordered && !node.is_bezeled {
                let cx = slot_rect.right() - seg_tok.menu_chevron_inset_trailing.points();
                let tip_y = slot_rect.bottom() - seg_tok.menu_chevron_inset_bottom.points();
                let hw = seg_tok.menu_chevron_half_width.points();
                let ch = seg_tok.menu_chevron_height.points();
                let st = egui::Stroke::new(seg_tok.menu_chevron_stroke_width.points(), tint);
                painter.line_segment([egui::pos2(cx - hw, tip_y - ch), egui::pos2(cx, tip_y)], st);
                painter.line_segment([egui::pos2(cx, tip_y), egui::pos2(cx + hw, tip_y - ch)], st);
            }
        }
    } else {
        let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
        MacSegmentedControl::new(&refs, sel)
            .id_salt(("nib_seg_ctrl", node.id.0))
            .theme(theme)
            .control_size(ctrl_size)
            .enabled(enabled)
            .fixed_rect(rect)
            .show(ui, |idx| next_sel = Some(idx));
    }
    if let Some(new_idx) = next_sel {
        if let Some(b) = node.value_binding {
            data_source.set_i64(b, new_idx as i64);
        } else {
            ui.data_mut(|d| d.insert_temp(fb_id, new_idx));
        }
        if let Some(act) = node.action {
            data_source.on_action(act, node.id);
        }
    }
}

pub(crate) fn render_help_button_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    ctrl_size: MacControlSize,
    enabled: bool,
) {
    let pop_id = ui.scope_id().with(("nib_help_pop_open", node.id.0));
    let mut is_open = ui.data(|d| d.get_temp::<bool>(pop_id)).unwrap_or(false);
    let btn_resp = MacHelpButton::new().theme(theme).control_size(ctrl_size).enabled(enabled).fixed_rect(rect).show(ui);
    if btn_resp.clicked() {
        is_open = !is_open;
        if let Some(act) = node.action {
            data_source.on_action(act, node.id);
        }
    }
    if is_open {
        let pm = theme.popover_metrics();
        let pop_w = pm.full_popover_width.points();
        let pop_h = pm.compact_panel_height.points() + pm.arrow_height.points();
        let screen_min_x = ui.ctx().content_rect().min.x;
        let pop_x = (rect.center().x - pop_w * 0.5).max(screen_min_x + pm.inner_padding.points());
        let pop_rect = Rect::from_min_size(egui::pos2(pop_x, rect.max.y), egui::vec2(pop_w, pop_h));
        let help_txt = if node.tooltip.is_empty() { node.title.as_str() } else { node.tooltip.as_str() };
        egui::Area::new(pop_id.with("fg_area")).order(egui::Order::Foreground).fixed_pos(pop_rect.min).show(ui.ctx(), |area_ui| {
            crate::surfaces::MacPopover::new().theme(theme).arrow_edge(crate::surfaces::MacPopoverArrowEdge::Top).show(
                area_ui,
                pop_rect,
                |pop_ui| {
                    pop_ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                    pop_ui.set_max_width(pop_w - pm.inner_padding.points() * 2.0);
                    pop_ui.label(egui::RichText::new(help_txt).size(theme.label_metrics().small_font_size.points()).color(theme.label()));
                },
            );
        });
        let esc = ui.input(|i| i.key_pressed(egui::Key::Escape));
        let clicked_out = !btn_resp.clicked()
            && ui
                .input(|i| i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|p| !pop_rect.contains(p) && !rect.contains(p)));
        if esc || clicked_out {
            is_open = false;
        }
    }
    ui.data_mut(|d| d.insert_temp(pop_id, is_open));
}

pub(crate) fn render_inline_action_button_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    enabled: bool,
) {
    let bound_on = node
        .value_binding
        .filter(|_| node.alternate_icon_name.is_some())
        .map(|b| node.value_transformer.apply_bool(data_source.get_bool(b)));
    let base_icon = node.icon_name.as_deref().or_else(|| node.action_selector.as_deref().map(|s| s.trim_end_matches(':')));
    let icon_cand = if bound_on == Some(true) {
        node.alternate_icon_name.as_deref().or(base_icon)
    } else {
        base_icon
    };
    if let Some(ic) = icon_cand {
        let id = ui.scope_id().with(("nib_inline_icon_btn", node.id.0));
        let sense = if enabled { egui::Sense::click() } else { egui::Sense::hover() };
        let resp = ui.interact(rect, id, sense);
        if enabled && (resp.is_pointer_button_down_on() || resp.hovered()) {
            let fill = if resp.is_pointer_button_down_on() {
                theme.inspector_pressed_fill()
            } else {
                theme.inspector_hover_fill()
            };
            let bezel_rect = rect.shrink(1.0);
            ui.painter_at(rect).rect_filled(bezel_rect, egui::CornerRadius::same(4), fill);
        }
        let is_on = (node.bezel_style & 0x80) != 0 || matches!(node.kind, SceneNodeKind::PushButton { is_default: true, .. });
        let tint = if !enabled {
            theme.tertiary_label()
        } else if is_on {
            theme.effective_control_accent(ui.ctx())
        } else {
            theme.secondary_label()
        };
        if !data_source.paint_scene_icon_scaled(ui, theme, ic, rect, tint, (node.image_scaling, node.symbol_point_size)) {
            if node.icon_name.is_some() {
                MacImageView::from_resource_name(ic).theme(theme).template(true).tint(tint).scaling(crate::controls::MacImageScaling::from_nib_code(node.image_scaling)).symbol_point_size(node.symbol_point_size).fixed_rect(rect).show(ui);
            } else if !node.title.is_empty() {
                let _ = MacInlineActionButton::new(node.title.as_str()).theme(theme).enabled(enabled).fixed_rect(rect).show(ui);
            }
        }
        if resp.clicked() && enabled {
            if let (Some(bid), Some(cur)) = (node.value_binding, bound_on) {
                data_source.set_bool(bid, node.value_transformer.apply_bool(!cur));
            }
            let pop_id = egui::Id::NULL.with(("nib_attached_btn_menu", node.id.0));
            let cur_open = ui.ctx().data(|d| d.get_temp::<(bool, Option<usize>)>(pop_id)).is_some_and(|(o, _)| o);
            ui.ctx().data_mut(|d| d.insert_temp(pop_id, (!cur_open, None::<usize>)));
            data_source.on_action(node.action.unwrap_or(crate::scene::ir::ActionId(0)), node.id);
        }
    } else if !node.title.is_empty()
        && MacInlineActionButton::new(node.title.as_str()).theme(theme).enabled(enabled).fixed_rect(rect).show(ui).clicked()
    {
        let pop_id = egui::Id::NULL.with(("nib_attached_btn_menu", node.id.0));
        let cur_open = ui.ctx().data(|d| d.get_temp::<(bool, Option<usize>)>(pop_id)).is_some_and(|(o, _)| o);
        ui.ctx().data_mut(|d| d.insert_temp(pop_id, (!cur_open, None::<usize>)));
        data_source.on_action(node.action.unwrap_or(crate::scene::ir::ActionId(0)), node.id);
    }
}

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
    let ic_name = node.icon_name.as_deref().unwrap_or("NSSearchMenuTemplate");
    let lead_rect = Rect::from_min_size(rect.min, egui::vec2(rect.height(), rect.height()));
    let has_host_icon = data_source.paint_scene_icon(ui, theme, ic_name, Rect::NOTHING, theme.placeholder_text());
    let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_srch_ui", node.id.0)).max_rect(rect));
    let changed = MacSearchField::new(&mut text)
        .id(c_ui.scope_id().with(("nib_srch", node.id.0)))
        .placeholder(placeholder)
        .width(rect.width())
        .height(rect.height())
        .enabled(enabled)
        .suppress_leading_icon(has_host_icon)
        .theme(theme)
        .show(&mut c_ui)
        .changed();
    if has_host_icon {
        let _ = data_source.paint_scene_icon(ui, theme, ic_name, lead_rect, theme.placeholder_text());
    }
    if changed && enabled {
        if let Some(b) = node.value_binding {
            data_source.set_str(b, &text);
        } else {
            ui.data_mut(|d| d.insert_temp(fb_id, text));
        }
    }
}

pub(crate) fn render_token_field_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    enabled: bool,
) {
    let raw = node.value_binding.map(|b| data_source.get_str(b).to_string()).unwrap_or_else(|| node.title.clone());
    let mut tokens: Vec<String> = raw.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
    let prev = tokens.clone();
    let draft_id = ui.scope_id().with(("nib_tok_draft", node.id.0));
    let mut draft = ui.data(|d| d.get_temp::<String>(draft_id)).unwrap_or_default();
    let on_sidebar = theme.is_sidebar_surface(theme.active_surface_at(ui, rect.center()));
    let is_pill = (on_sidebar && node.is_bezeled) || node.bezel_style == 1 || (!node.draws_background && node.is_bezeled);
    let cap_tint = data_source.resolve_token_capsule_tint(node.id, &node.ib_object_id, theme).unwrap_or_else(|| theme.control_accent());
    let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_tok_ui", node.id.0)).max_rect(rect));
    c_ui.add_enabled_ui(enabled, |en_ui| {
        let mut tf = MacTokenField::new((en_ui.scope_id(), "nib_tok", node.id.0), &mut tokens, &mut draft)
            .recessed_pill(is_pill)
            .capsule_tint(cap_tint)
            .fixed_rect(rect)
            .theme(theme);
        if let Some(pt) = node.font_size_pt {
            tf = tf.font_size(pt);
        }
        tf.show(en_ui);
    });
    ui.data_mut(|d| d.insert_temp(draft_id, draft));
    if tokens != prev && enabled {
        if let Some(b) = node.value_binding {
            data_source.set_str(b, &tokens.join(","));
        }
    }
}

pub(crate) fn render_table_view_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    columns: &[(String, f32)],
) {
    let is_outline = node.icon_name.as_deref() == Some("NSOutlineView");
    if data_source.render_table_or_outline(ui, theme, (node.id, &node.ib_object_id, is_outline), columns, rect) {
        return;
    }
    let mut sel_set = BTreeSet::from([0usize]);
    let min_col_w = theme.table_view().header_height.points();
    let cols: Vec<MacTableColumn<'_>> = if columns.is_empty() {
        vec![MacTableColumn::new("", rect.width())]
    } else {
        columns.iter().map(|(title, w)| MacTableColumn::new(title.as_str(), (*w).max(min_col_w))).collect()
    };
    let row_count = usize::from(!node.title.is_empty()).max(1);
    let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_tbl_ui", node.id.0)).max_rect(rect).layout(Layout::top_down(Align::Min)));
    let _ = MacTableView::new(("nib_tbl", node.id.0), &cols, row_count, &mut sel_set)
        .theme(theme)
        .uses_alternating_row_backgrounds(false)
        .show(&mut c_ui, |cell_ui, _row, col| {
            if col == 0 && !node.title.is_empty() {
                MacLabel::new(node.title.as_str()).control_size(MacControlSize::Small).theme(theme).show(cell_ui);
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
    if node.icon_name.as_deref() == Some("NSNoTabs") {
        let active = data_source.resolve_active_tab(node.id, &node.ib_object_id, tab_idx);
        data_source.render_tab_item(ui, theme, node.id, &node.ib_object_id, active, rect);
        return;
    }

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

impl crate::scene::ir::MacSceneIR {
    /// Returns the cached solved layout (`Arc<BTreeMap<NodeId, Rect>>`) for `(viewport_size, policy, workspace_flags)`
    /// stored in `ctx` temporary memory when `search_query` is empty (`REQ-013-10.8`), avoiding redundant per-frame
    /// `BTreeMap` allocations and constraint evaluations at 120/240 FPS when the viewport is unchanged.
    pub fn solve_layout_cached(
        &self,
        ctx: &egui::Context,
        viewport_size: egui::Vec2,
        policy: crate::scene::ir::LayoutPolicy,
        workspace_flags: u64,
        search_query: &str,
    ) -> std::sync::Arc<std::collections::BTreeMap<crate::scene::ir::NodeId, Rect>> {
        if !search_query.is_empty() {
            return std::sync::Arc::new(self.solve_layout(viewport_size, policy, workspace_flags, search_query));
        }
        let first_ib = self.nodes.first().map(|n| n.ib_object_id.as_str()).unwrap_or("");
        let cache_id = egui::Id::unique((
            "macview_solved_layout_cache",
            self.root.0,
            self.nodes.len(),
            first_ib,
            self.design_size[0].to_bits(),
            self.design_size[1].to_bits(),
            viewport_size.x.to_bits(),
            viewport_size.y.to_bits(),
            policy == crate::scene::ir::LayoutPolicy::ExactAppleNib,
            workspace_flags,
        ));
        if let Some(hit) = ctx.data(|d| d.get_temp::<std::sync::Arc<std::collections::BTreeMap<crate::scene::ir::NodeId, Rect>>>(cache_id))
        {
            return hit;
        }
        let arc = std::sync::Arc::new(self.solve_layout(viewport_size, policy, workspace_flags, search_query));
        ctx.data_mut(|d| d.insert_temp(cache_id, std::sync::Arc::clone(&arc)));
        arc
    }

    /// Propagates the solved intrinsic height (`child_stack_height`) of a mounted subscene into `slot_outlet`
    /// and its vertical `NSLayoutConstraint` chain (`REQ-013-10.2`),
    /// re-normalizing `alignment_rect` without hardcoded frame coordinates.
    pub fn sync_mounted_vertical_slot_height(&mut self, slot_outlet: &str, child_stack_height: f32) -> bool {
        let Some(slot_id) = self.resolve_node_id(slot_outlet) else { return false };
        let Some(slot_idx) = self.nodes.iter().position(|n| n.id == slot_id) else { return false };
        let slot_bottom = self.nodes[slot_idx].nib_frame[1];
        let old_slot_h = self.nodes[slot_idx].nib_frame[3];
        let Some(wrap_id) = self.nodes[slot_idx].parent else { return false };
        let Some(wrap_idx) = self.nodes.iter().position(|n| n.id == wrap_id) else { return false };
        let old_wrap_h = self.nodes[wrap_idx].nib_frame[3];
        let top_pad = (old_wrap_h - (slot_bottom + old_slot_h)).max(0.0);
        let new_slot_h = (child_stack_height - slot_bottom + top_pad * 0.5).max(1.0);
        let new_wrap_h = slot_bottom + new_slot_h + top_pad;
        let delta_h = old_wrap_h - new_wrap_h;
        if delta_h.abs() <= 0.5 {
            return false;
        }
        self.nodes[slot_idx].nib_frame[3] = new_slot_h;
        self.nodes[wrap_idx].nib_frame[3] = new_wrap_h;
        let above_sib = self.nib_constraints.iter().find_map(|c| {
            use crate::scene::constraints::ConstraintAttr;
            if c.first_item == Some(wrap_id) && c.first_attr == ConstraintAttr::Top && c.second_attr == ConstraintAttr::Bottom {
                c.second_item
            } else if c.second_item == Some(wrap_id) && c.second_attr == ConstraintAttr::Top && c.first_attr == ConstraintAttr::Bottom {
                c.first_item
            } else {
                None
            }
        });
        if let Some(above_node) = above_sib.and_then(|aid| self.nodes.iter_mut().find(|n| n.id == aid)) {
            above_node.nib_frame[1] = new_wrap_h;
            above_node.nib_frame[3] += delta_h;
        }
        self.solve_autolayout();
        true
    }

    pub(crate) fn compute_hidden_and_y_shifts(
        &self,
        data_source: &dyn MacSceneDataSource,
    ) -> (std::collections::BTreeSet<crate::scene::ir::NodeId>, std::collections::BTreeMap<crate::scene::ir::NodeId, f32>) {
        crate::scene::nib_ibtool::compute_scene_hidden_and_y_shifts(self, data_source)
    }

    pub(crate) fn render_attached_button_menus(
        &self,
        ui: &mut Ui,
        theme: &MacTheme,
        data_source: &mut dyn MacSceneDataSource,
        solved: &std::collections::BTreeMap<crate::scene::ir::NodeId, Rect>,
        origin: egui::Vec2,
    ) {
        crate::scene::nib_frontend::render_scene_attached_button_menus(self, ui, theme, data_source, solved, origin);
    }

    pub(crate) fn recompute_semantics(&mut self) {
        for n in &mut self.nodes {
            n.labeled_by = None;
        }
        let (pairs, rows, sections) = crate::scene::normalizer::run_spatial_semantic_normalizer(&mut self.nodes);
        self.labeled_pairs = pairs;
        self.field_rows = rows;
        self.semantic_sections = sections;
        self.rebuild_search_index();
    }
}
