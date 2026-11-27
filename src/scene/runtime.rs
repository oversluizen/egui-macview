//! Layer 3: Dual-Mode Layout Solver (`ExactAppleNib` & `AdaptiveSemanticReflow`),
//! 120 FPS Renderer, and Zero-Allocation `MacSceneDataSource` Bridge (`SPEC-013 §4`).

use crate::controls::{MacBox, MacButtonCell, MacControlSize, MacInputCell, MacPopUpButton, MacRadioButton, MacSwitch, MacTextView};
use crate::scene::ir::{ActionId, BindingId, LayoutPolicy, MacSceneIR, NodeId, SceneNode, SceneNodeKind};
pub use crate::scene::normalizer::apply_autoresizing_mask;
use crate::scene::runtime_tabs::{render_segmented_control_node, render_token_field_node, render_window_titlebar_container};
use crate::scene::runtime_widgets::{render_checkbox_node, render_extended_node, render_label_node, render_slider_node};
use crate::theme::MacTheme;
use egui::{pos2, vec2, Rect, Ui, Vec2};
use std::collections::BTreeMap;

/// Zero-allocation host state and action bridge for [`MacSceneIR`] (`REQ-013-03` §4.3).
pub trait MacSceneDataSource {
    /// Reads a boolean binding value.
    fn get_bool(&self, id: BindingId) -> bool;
    /// Writes a boolean binding value.
    fn set_bool(&mut self, id: BindingId, val: bool);
    /// Reads a 64-bit floating-point binding value.
    fn get_f64(&self, id: BindingId) -> f64;
    /// Writes a 64-bit floating-point binding value.
    fn set_f64(&mut self, id: BindingId, val: f64);
    /// Reads a 64-bit integer binding value.
    fn get_i64(&self, id: BindingId) -> i64;
    /// Writes a 64-bit integer binding value.
    fn set_i64(&mut self, id: BindingId, val: i64);
    /// Reads a string slice binding value.
    fn get_str(&self, id: BindingId) -> &str;
    /// Writes a string binding value.
    fn set_str(&mut self, id: BindingId, val: &str);
    /// Returns dynamic option items for a `PopUpButton` or `ComboBox` binding.
    fn get_options(&self, id: BindingId) -> &[String];
    /// Dispatches an action selector triggered by `sender`.
    fn on_action(&mut self, action: ActionId, sender: NodeId);
    /// Renders an application-defined custom canvas slot (`NSClassSwapper` view).
    fn render_custom_slot(&mut self, ui: &mut Ui, theme: &MacTheme, slot_class: &str, rect: Rect);
    /// Reads a color binding value (`NSColorWell`).
    fn get_color(&self, _id: BindingId) -> Option<egui::Color32> {
        None
    }
    /// Writes a color binding value (`NSColorWell`).
    fn set_color(&mut self, _id: BindingId, _val: egui::Color32) {}
    /// Queries whether `node_id` (`ib_object_id`) is currently enabled (`enabledBinding` inheritance).
    fn is_node_enabled(&self, _id: NodeId, _ib_object_id: &str, default_enabled: bool) -> bool {
        default_enabled
    }
    /// Resolves the control size for `node_id` (`ib_object_id`) when dynamically bound.
    fn resolve_control_size(&self, _id: NodeId, _ib_object_id: &str, default_size: MacControlSize) -> MacControlSize {
        default_size
    }
    /// Resolves whether a level indicator node renders discrete segments (`true`) or a continuous bar (`false`).
    fn resolve_level_discrete(&self, _id: NodeId, _ib_object_id: &str, default_discrete: bool) -> bool {
        default_discrete
    }
    /// Resolves whether a level indicator node applies warning/critical color thresholds (`true`) or uniform color (`false`).
    fn resolve_level_thresholds(&self, _id: NodeId, _ib_object_id: &str, default_thresholds: bool) -> bool {
        default_thresholds
    }
    /// Returns the criteria slices for an `NSRuleEditor` node (`row_column -> options`).
    fn get_rule_criteria(&self, _id: NodeId, _ib_object_id: &str) -> &[&[&str]] {
        &[]
    }
    /// Renders the body of an active `<tabViewItem>` inside an `NSTabView` node.
    fn render_tab_item(&mut self, _ui: &mut Ui, _theme: &MacTheme, _id: NodeId, _ib_object_id: &str, _tab_idx: usize, _rect: Rect) {}
    /// Optional host icon painter (`SPEC-013 §3.1.6` Tier 2); returns `true` when the host painted `icon_name` inside `rect`.
    fn paint_scene_icon(&mut self, _ui: &mut Ui, _theme: &MacTheme, _icon_name: &str, _rect: Rect, _tint: egui::Color32) -> bool {
        false
    }
    /// Optional host icon painter with explicit AppKit `NSImageScaling` (`0..=3`) and `NSImageSymbolConfiguration.NSPointSize`.
    fn paint_scene_icon_scaled(&mut self, ui: &mut Ui, theme: &MacTheme, icon_name: &str, rect: Rect, tint: egui::Color32, (_image_scaling, _symbol_point_size): (u8, Option<f32>)) -> bool {
        self.paint_scene_icon(ui, theme, icon_name, rect, tint)
    }
    /// Optional host data-source renderer for `NSTableView` / `NSOutlineView` nodes (`SceneNodeKind::TableView`).
    /// Returns `true` when the host rendered the table or outline view inside `rect`.
    fn render_table_or_outline(&mut self, _ui: &mut Ui, _theme: &MacTheme, _node_meta: (NodeId, &str, bool), _columns: &[(String, f32)], _rect: Rect) -> bool {
        false
    }
    /// Resolves the active tab index for an `NSTabView` node (`SceneNodeKind::TabView`).
    fn resolve_active_tab(&self, _id: NodeId, _ib_object_id: &str, default_tab: usize) -> usize {
        default_tab
    }
    /// Resolves an optional custom capsule tint color for an `NSTokenField` node (`REQ-013-10.5`).
    fn resolve_token_capsule_tint(&self, _id: NodeId, _ib_object_id: &str, _theme: &MacTheme) -> Option<egui::Color32> {
        None
    }
}

pub(crate) fn solve_scene_layout(
    scene: &MacSceneIR,
    viewport_size: Vec2,
    policy: LayoutPolicy,
    workspace_flags: u64,
    search_query: &str,
) -> BTreeMap<NodeId, Rect> {
    let filter_mask = scene.filter_by_search(search_query);
    let mut out = BTreeMap::new();

    match policy {
        LayoutPolicy::ExactAppleNib => {
            crate::scene::stack_reflow::solve_exact_apple_nib_layout(scene, viewport_size, workspace_flags, &filter_mask, &mut out);
        }
        LayoutPolicy::AdaptiveSemanticReflow => {
            let tokens = crate::tokens::MacDesignTokens::default_shared();
            let tb = &tokens.macview.metrics.toolbar;
            let sv = crate::tokens::StackViewMetrics::default();
            let strip_h = tb.height_unified.points();
            let is_horizontal_strip = scene.design_size[1] <= strip_h;
            if is_horizontal_strip {
                let mut cursor_x = tb.edge_padding.points();
                let item_gap = sv.default_spacing.points();
                for node in &scene.nodes {
                    if matches!(node.kind, SceneNodeKind::Container { .. }) {
                        continue;
                    }
                    if !node.visibility.is_visible(workspace_flags) || !filter_mask.contains(node.id) {
                        continue;
                    }
                    let [_, y, w, h] = node.alignment_rect;
                    out.insert(node.id, Rect::from_min_size(pos2(cursor_x, y), vec2(w, h)));
                    cursor_x += w + item_gap;
                }
            } else {
                let sv = crate::tokens::StackViewMetrics::default();
                let pad_x = sv.edge_padding.points() * 2.0;
                let row_gap_x = sv.default_spacing.points();
                let row_gap_y = sv.default_spacing.points() + 2.0;
                let def_row_h = sv.default_item_height.points();
                let col_align_x = crate::tokens::DatePickerMetrics::default().calendar_width.points();
                let canvas_skip_h = crate::tokens::DatePickerMetrics::default().clock_size.points();
                let baseline_tol = sv.default_spacing.points() - 2.0;
                let max_x = (viewport_size.x - pad_x).max(col_align_x);
                let mut visible_nodes: Vec<&SceneNode> = scene
                    .nodes
                    .iter()
                    .filter(|node| {
                        !matches!(node.kind, SceneNodeKind::Container { .. } | SceneNodeKind::Separator | SceneNodeKind::BoxContainer)
                            && !(matches!(node.kind, SceneNodeKind::CustomCanvasSlot { .. }) && node.alignment_rect[3] >= canvas_skip_h)
                            && node.visibility.is_visible(workspace_flags)
                            && filter_mask.contains(node.id)
                    })
                    .collect();
                visible_nodes.sort_by(|a, b| {
                    let ya = a.alignment_rect[1];
                    let yb = b.alignment_rect[1];
                    if (ya - yb).abs() <= baseline_tol {
                        a.alignment_rect[0].partial_cmp(&b.alignment_rect[0]).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        ya.partial_cmp(&yb).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });

                let mut cursor_y = pad_x;
                let mut idx = 0usize;
                while idx < visible_nodes.len() {
                    let row_anchor_y = visible_nodes[idx].alignment_rect[1];
                    let mut row_end = idx + 1;
                    while row_end < visible_nodes.len() && (visible_nodes[row_end].alignment_rect[1] - row_anchor_y).abs() <= baseline_tol {
                        row_end += 1;
                    }
                    let row = &visible_nodes[idx..row_end];
                    let row_h = row.iter().map(|n| n.alignment_rect[3]).fold(def_row_h, f32::max);
                    let mut row_x = pad_x;
                    for (col_i, node) in row.iter().enumerate() {
                        let [_, _, w, h] = node.alignment_rect;
                        if col_i == 1
                            && matches!(row[0].kind, SceneNodeKind::Label)
                            && !matches!(node.kind, SceneNodeKind::Label)
                            && row_x < col_align_x
                        {
                            row_x = col_align_x;
                        }
                        if row_x > pad_x && row_x + w > max_x {
                            cursor_y += row_h + row_gap_x;
                            row_x = pad_x;
                        }
                        let y_aligned = cursor_y + (row_h - h).max(0.0) * 0.5;
                        out.insert(node.id, Rect::from_min_size(pos2(row_x, y_aligned), vec2(w, h)));
                        row_x += w + row_gap_x;
                    }
                    cursor_y += row_h + row_gap_y;
                    idx = row_end;
                }
            }
        }
    }

    out
}

impl MacSceneIR {
    /// Renders the compiled scene inside `ui` using `policy` and `data_source` (`REQ-013-03`, `REQ-013-02P`).
    pub fn show(
        &self,
        ui: &mut Ui,
        theme: &MacTheme,
        data_source: &mut dyn MacSceneDataSource,
        policy: LayoutPolicy,
        workspace_flags: u64,
        search_query: &str,
    ) {
        let pm = theme.popover_metrics();
        let min_vp_w = pm.full_popover_width.points();
        let min_vp_h = pm.compact_panel_width.points();
        let viewport = match policy {
            LayoutPolicy::ExactAppleNib => vec2(self.design_size[0].max(1.0), self.design_size[1].max(1.0)),
            LayoutPolicy::AdaptiveSemanticReflow => vec2(self.design_size[0].max(min_vp_w), self.design_size[1].max(min_vp_h)),
        };
        let target_rect = Rect::from_min_size(ui.cursor().min, viewport);
        self.show_in_rect(ui, target_rect, theme, data_source, policy, workspace_flags, search_query);
    }

    /// Renders the compiled scene inside an explicit `target_rect`, applying AppKit `NSAutoresizingMaskOptions`
    /// (`resizeWithOldSuperviewSize:`) from `self.design_size` to `target_rect.size()` (`REQ-013-03`).
    pub fn show_in_rect(
        &self,
        ui: &mut Ui,
        target_rect: Rect,
        theme: &MacTheme,
        data_source: &mut dyn MacSceneDataSource,
        policy: LayoutPolicy,
        workspace_flags: u64,
        search_query: &str,
    ) {
        let origin = target_rect.min.to_vec2();
        let viewport = vec2(target_rect.width().max(1.0), target_rect.height().max(1.0));
        let scale_x = 1.0;
        let solved = self.solve_layout_cached(ui.ctx(), viewport, policy, workspace_flags, search_query);

        let mut deferred_inline_buttons = Vec::new();
        let (hidden_ids, y_shifts) = self.compute_hidden_and_y_shifts(data_source);
        for node in &self.nodes {
            if !hidden_ids.contains(&node.id)
                && matches!(&node.kind, SceneNodeKind::ImageView { icon_name, .. } if icon_name.contains("Background"))
            {
                if let Some(&local_rect) = solved.get(&node.id) {
                    let y_off = y_shifts.get(&node.id).copied().unwrap_or(0.0);
                    let rect = Rect::from_min_size(
                        pos2(local_rect.min.x * scale_x, local_rect.min.y + y_off),
                        vec2(local_rect.width() * scale_x, local_rect.height()),
                    )
                    .translate(origin);
                    let ctrl_size = data_source.resolve_control_size(node.id, &node.ib_object_id, node.mac_control_size());
                    render_extended_node(ui, theme, data_source, node, rect, ctrl_size, true);
                }
            }
        }
        for node in &self.nodes {
            if hidden_ids.contains(&node.id)
                || matches!(&node.kind, SceneNodeKind::ImageView { icon_name, .. } if icon_name.contains("Background"))
            {
                continue;
            }
            if let crate::scene::ir::VisibilityRule::WhenTabSelected { tab_view, tab_index } = node.visibility {
                let tv_node = self.nodes.iter().find(|n| n.id == tab_view);
                let default_tab = tv_node
                    .and_then(|tv| tv.value_binding)
                    .map(|b| data_source.get_i64(b).max(0) as usize)
                    .unwrap_or_else(|| ui.data(|d| d.get_temp::<usize>(ui.scope_id().with(("nib_tv_sel", tab_view.0)))).unwrap_or(0));
                let tv_ib = tv_node.map(|n| n.ib_object_id.as_str()).unwrap_or("");
                if data_source.resolve_active_tab(tab_view, tv_ib, default_tab) != tab_index {
                    continue;
                }
            }
            let Some(&local_rect) = solved.get(&node.id) else {
                continue;
            };
            let y_off = y_shifts.get(&node.id).copied().unwrap_or(0.0);
            let scaled_rect = Rect::from_min_size(
                pos2(local_rect.min.x * scale_x, local_rect.min.y + y_off),
                vec2(local_rect.width() * scale_x, local_rect.height()),
            );
            let rect = scaled_rect.translate(origin);
            let ctrl_size = data_source.resolve_control_size(node.id, &node.ib_object_id, node.mac_control_size());
            let default_en =
                node.enabled_binding.map(|eb| node.enabled_transformer.apply_bool(data_source.get_bool(eb))).unwrap_or(node.enabled);
            let enabled = data_source.is_node_enabled(node.id, &node.ib_object_id, default_en);
            if matches!(node.kind, SceneNodeKind::InlineActionButton) {
                deferred_inline_buttons.push((node, rect, ctrl_size, enabled));
                continue;
            }

            match &node.kind {
                SceneNodeKind::Container { .. } => {
                    if !render_window_titlebar_container(ui, theme, node, rect) {
                        if let Some(mat_code) = node.visual_effect_material() {
                            let mat_bg = theme.visual_effect_surface_color(mat_code, node.visual_effect_is_behind_window());
                            ui.painter().rect_filled(rect, 0.0, mat_bg);
                            theme.register_surface_rect(ui.ctx(), rect, mat_bg);
                        }
                    }
                }
                SceneNodeKind::BoxContainer => {
                    MacBox::new(node.title.as_str()).theme(theme).paint_at(ui, rect);
                }
                SceneNodeKind::Separator => {
                    MacBox::separator_at(ui, rect, theme);
                }
                SceneNodeKind::Label => {
                    render_label_node(ui, theme, data_source, node, rect, ctrl_size);
                }
                SceneNodeKind::PushButton { is_default, .. } => {
                    let unb = !node.is_bordered && !node.is_bezeled && node.icon_name.is_some();
                    let resp = if unb {
                        ui.interact(rect, ui.scope_id().with(("nib_unb_btn", node.id.0, node.ib_object_id.as_str())), egui::Sense::click())
                    } else {
                        MacButtonCell::new(node.title.as_str()).id_salt(("nib_btn_cell", node.id.0)).theme(theme).selected(*is_default).control_size(ctrl_size).enabled(enabled).fixed_rect(rect).show(ui)
                    };
                    if let Some(ic) = node.icon_name.as_ref().filter(|_| node.title.is_empty() || unb) {
                        let is_active = *is_default || (unb && (node.bezel_style & 0x80) != 0);
                        let tint = match (unb, is_active) {
                            (true, true) => theme.effective_control_accent(ui.ctx()),
                            (true, false) => theme.secondary_label(),
                            (false, true) => theme.on_accent_text(),
                            (false, false) => theme.label(),
                        };
                        if !data_source.paint_scene_icon_scaled(ui, theme, ic, rect, tint, (node.image_scaling, node.symbol_point_size)) {
                            crate::controls::MacImageView::from_resource_name(ic).theme(theme).template(true).scaling(crate::controls::MacImageScaling::from_nib_code(node.image_scaling)).symbol_point_size(node.symbol_point_size).fixed_rect(rect).show(ui);
                        }
                    }
                    if resp.clicked() {
                        if self.button_menus.contains_key(&node.id) {
                            let pop_id = egui::Id::NULL.with(("nib_attached_btn_menu", node.id.0));
                            let cur_open = ui.ctx().data(|d| d.get_temp::<(bool, Option<usize>)>(pop_id)).is_some_and(|(o, _)| o);
                            ui.ctx().data_mut(|d| d.insert_temp(pop_id, (!cur_open, None::<usize>)));
                        }
                        data_source.on_action(node.action.unwrap_or(ActionId(0)), node.id);
                    }
                }
                SceneNodeKind::Checkbox => {
                    render_checkbox_node(ui, theme, data_source, node, rect, ctrl_size, enabled);
                }
                SceneNodeKind::RadioButton => {
                    let fb_id = ui.scope_id().with(("nib_rad", node.id.0));
                    let mut val = node
                        .value_binding
                        .map(|b| data_source.get_bool(b))
                        .unwrap_or_else(|| ui.data(|d| d.get_temp::<bool>(fb_id)).unwrap_or(false));
                    if MacRadioButton::new(&mut val, node.title.as_str())
                        .theme(theme)
                        .control_size(ctrl_size)
                        .enabled(enabled)
                        .fixed_rect(rect)
                        .show(ui)
                        .changed()
                    {
                        if let Some(b) = node.value_binding {
                            data_source.set_bool(b, val);
                        } else {
                            ui.data_mut(|d| d.insert_temp(fb_id, val));
                        }
                        if let Some(act) = node.action {
                            data_source.on_action(act, node.id);
                        }
                    }
                }
                SceneNodeKind::Switch => {
                    let fb_id = ui.scope_id().with(("nib_sw", node.id.0));
                    let mut val = node
                        .value_binding
                        .map(|b| data_source.get_bool(b))
                        .unwrap_or_else(|| ui.data(|d| d.get_temp::<bool>(fb_id)).unwrap_or(false));
                    if MacSwitch::new(&mut val).theme(theme).control_size(ctrl_size).enabled(enabled).fixed_rect(rect).show(ui).changed() {
                        if let Some(b) = node.value_binding {
                            data_source.set_bool(b, val);
                        } else {
                            ui.data_mut(|d| d.insert_temp(fb_id, val));
                        }
                        if let Some(act) = node.action {
                            data_source.on_action(act, node.id);
                        }
                    }
                }
                SceneNodeKind::TextField | SceneNodeKind::NumberField { .. } => {
                    if node.icon_name.as_deref() == Some("NSTokenField") {
                        render_token_field_node(ui, theme, data_source, node, rect, enabled);
                    } else if (!node.is_editable && !node.draws_background && !node.is_bezeled && !node.is_bordered) || node.title == "--" {
                        render_label_node(ui, theme, data_source, node, rect, ctrl_size);
                    } else {
                        let fb_id = ui.scope_id().with(("nib_txt", node.id.0));
                        let mut text = node
                            .value_binding
                            .map(|b| data_source.get_str(b).to_string())
                            .unwrap_or_else(|| ui.data(|d| d.get_temp::<String>(fb_id)).unwrap_or_else(|| node.title.clone()));
                        if matches!(node.kind, SceneNodeKind::NumberField { .. }) {
                            if let Some(sfx) = node.icon_name.as_ref().filter(|s| matches!(s.as_str(), "%" | "°")) {
                                if !text.is_empty() && !text.ends_with(sfx.as_str()) {
                                    text.push_str(sfx);
                                }
                            }
                        }
                        let t_align = match node.text_alignment {
                            crate::scene::ir::MacTextAlignment::Right => egui::Align::Max,
                            crate::scene::ir::MacTextAlignment::Center => egui::Align::Center,
                            _ => egui::Align::Min,
                        };
                        let on_sidebar = theme.is_sidebar_surface(theme.active_surface_at(ui, rect.center()));
                        let is_round_bezel = node.bezel_style == 1
                            || node.cell_class.as_deref().is_some_and(|c| c.contains("RoundTextFieldCell"))
                            || (!node.draws_background && node.is_bezeled);
                        let is_pill = (on_sidebar && node.is_bezeled) || is_round_bezel;
                        let is_mono = node.font_name.as_deref().is_some_and(|n| n.contains("Mono"));
                        let is_alt = node.font_name.as_deref() == Some(".SFNS-Regular");
                        let mut cell = MacInputCell::with_id(ui.scope_id().with(("nib_input", node.id.0)), &mut text)
                            .theme(theme)
                            .control_size(ctrl_size)
                            .text_align(t_align)
                            .font_weight(node.font_weight)
                            .monospace(is_mono)
                            .stylistic_alt(is_alt)
                            .recessed_pill(is_pill)
                            .enabled(enabled)
                            .fixed_rect(rect);
                        if !is_round_bezel && node.draws_background {
                            cell = cell.corner_radius(theme.text_field().regular.corner_radius.points());
                        }
                        if !node.title.is_empty() && node.value_binding.is_some() {
                            cell = cell.placeholder(node.title.as_str());
                        }
                        if let SceneNodeKind::NumberField { min, max, step, .. } = &node.kind {
                            let bound = f64::from(u16::MAX);
                            let min_step = f64::from(theme.slider().key_step_fraction.points());
                            cell = cell.numeric(min.unwrap_or(-bound)..=max.unwrap_or(bound), (*step).max(min_step));
                        }
                        if cell.show(ui).changed() {
                            if let Some(b) = node.value_binding {
                                if matches!(node.kind, SceneNodeKind::NumberField { .. }) {
                                    let clean_num = text.trim_end_matches(['%', '°']);
                                    if let Some(v) = crate::scene::constraints::evaluate_arithmetic_expr(clean_num) {
                                        data_source.set_f64(b, v);
                                    }
                                }
                                data_source.set_str(b, &text);
                            } else {
                                ui.data_mut(|d| d.insert_temp(fb_id, text));
                            }
                        }
                    }
                }
                SceneNodeKind::TextView { monospace, line_numbers } => {
                    let fb_id = ui.scope_id().with(("nib_tv", node.id.0));
                    let mut text = node
                        .value_binding
                        .map(|b| data_source.get_str(b).to_string())
                        .unwrap_or_else(|| ui.data(|d| d.get_temp::<String>(fb_id)).unwrap_or_else(|| node.title.clone()));
                    if MacTextView::new((ui.scope_id(), "nib_tv_w", node.id.0), &mut text)
                        .theme(theme)
                        .monospace(*monospace)
                        .line_numbers(*line_numbers)
                        .fixed_rect(rect)
                        .show(ui)
                        .changed()
                    {
                        if let Some(b) = node.value_binding {
                            data_source.set_str(b, &text);
                        } else {
                            ui.data_mut(|d| d.insert_temp(fb_id, text));
                        }
                    }
                }
                SceneNodeKind::PopUpButton { options } => {
                    let fb_id = ui.scope_id().with(("nib_pop", node.id.0));
                    let mut sel = node
                        .value_binding
                        .map(|b| data_source.get_i64(b).max(0) as usize)
                        .unwrap_or_else(|| ui.data(|d| d.get_temp::<usize>(fb_id)).unwrap_or(0));
                    let refs: Vec<&str> =
                        if options.is_empty() { vec![node.title.as_str()] } else { options.iter().map(String::as_str).collect() };
                    let badge = matches!(node.icon_name.as_deref(), Some("accent_badge" | "NSAccentBadge"));
                    if MacPopUpButton::new(&mut sel, &refs)
                        .theme(theme)
                        .control_size(ctrl_size)
                        .accent_badge(badge)
                        .enabled(enabled)
                        .fixed_rect(rect)
                        .show(ui)
                        .changed()
                    {
                        if let Some(b) = node.value_binding {
                            data_source.set_i64(b, sel as i64);
                        } else {
                            ui.data_mut(|d| d.insert_temp(fb_id, sel));
                        }
                        if let Some(act) = node.action {
                            data_source.on_action(act, node.id);
                        }
                    }
                }
                SceneNodeKind::SegmentedControl { segments } => {
                    render_segmented_control_node(ui, theme, data_source, node, rect, segments, (ctrl_size, enabled));
                }
                SceneNodeKind::Slider { min, max, tick_marks } => {
                    render_slider_node(ui, theme, data_source, node, rect, (*min, *max, *tick_marks), (ctrl_size, enabled));
                }
                SceneNodeKind::CustomCanvasSlot { .. } => {}
                _ => {
                    render_extended_node(ui, theme, data_source, node, rect, ctrl_size, enabled);
                }
            }
        }
        for (node, rect, ctrl_size, enabled) in deferred_inline_buttons {
            render_extended_node(ui, theme, data_source, node, rect, ctrl_size, enabled);
        }
        self.render_attached_button_menus(ui, theme, data_source, &solved, origin);
        for node in &self.nodes {
            if let SceneNodeKind::CustomCanvasSlot { class_name } = &node.kind {
                if let crate::scene::ir::VisibilityRule::WhenTabSelected { tab_view, tab_index } = node.visibility {
                    let tv_node = self.nodes.iter().find(|n| n.id == tab_view);
                    let default_tab = tv_node
                        .and_then(|tv| tv.value_binding)
                        .map(|b| data_source.get_i64(b).max(0) as usize)
                        .unwrap_or_else(|| ui.data(|d| d.get_temp::<usize>(ui.scope_id().with(("nib_tv_sel", tab_view.0)))).unwrap_or(0));
                    let tv_ib = tv_node.map(|n| n.ib_object_id.as_str()).unwrap_or("");
                    if data_source.resolve_active_tab(tab_view, tv_ib, default_tab) != tab_index {
                        continue;
                    }
                }
                if let Some(&local_rect) = solved.get(&node.id) {
                    let scaled_rect = Rect::from_min_size(
                        pos2(local_rect.min.x * scale_x, local_rect.min.y),
                        vec2(local_rect.width() * scale_x, local_rect.height()),
                    );
                    data_source.render_custom_slot(ui, theme, class_name, scaled_rect.translate(origin));
                }
            }
        }
        ui.advance_cursor_after_rect(Rect::from_min_size(origin.to_pos2(), vec2(viewport.x * scale_x, viewport.y)));
    }
}
