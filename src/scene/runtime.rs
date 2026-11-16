//! Layer 3: Dual-Mode Layout Solver (`ExactAppleNib` & `AdaptiveSemanticReflow`),
//! 120 FPS Renderer, and Zero-Allocation `MacSceneDataSource` Bridge (`SPEC-013 §4`).

use crate::controls::{
    MacBox, MacButtonCell, MacControlSize, MacInputCell, MacPopUpButton, MacRadioButton, MacSegmentedControl, MacSwitch, MacTextView,
    MacTokenField,
};
use crate::scene::ir::{ActionId, BindingId, LayoutPolicy, MacSceneIR, NodeId, SceneNode, SceneNodeKind};
pub use crate::scene::normalizer::apply_autoresizing_mask;
use crate::scene::runtime_widgets::{render_checkbox_node, render_extended_node, render_label_node, render_slider_node};
use crate::theme::MacTheme;
use egui::{pos2, vec2, Pos2, Rect, Ui, UiBuilder, Vec2};
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
            let [w0, h0] = scene.design_size;
            for node in &scene.nodes {
                if !node.visibility.is_visible(workspace_flags) || !filter_mask.contains(node.id) {
                    continue;
                }
                let [x, y, w, h] = apply_autoresizing_mask(
                    node.alignment_rect,
                    node.autoresizing_mask,
                    [w0.max(1.0), h0.max(1.0)],
                    [viewport_size.x.max(1.0), viewport_size.y.max(1.0)],
                );
                out.insert(node.id, Rect::from_min_size(pos2(x, y), vec2(w, h)));
            }
            if !scene.constraints.steps().is_empty() {
                scene.constraints.evaluate(Rect::from_min_size(Pos2::ZERO, viewport_size), &mut out);
            }
        }
        LayoutPolicy::AdaptiveSemanticReflow => {
            let is_horizontal_strip = scene.design_size[1] <= 48.0;
            if is_horizontal_strip {
                let mut cursor_x = 12.0_f32;
                for node in &scene.nodes {
                    if matches!(node.kind, SceneNodeKind::Container { .. }) {
                        continue;
                    }
                    if !node.visibility.is_visible(workspace_flags) || !filter_mask.contains(node.id) {
                        continue;
                    }
                    let [_, y, w, h] = node.alignment_rect;
                    out.insert(node.id, Rect::from_min_size(pos2(cursor_x, y), vec2(w, h)));
                    cursor_x += w + 8.0;
                }
            } else {
                let mut cursor_y = 16.0_f32;
                for node in &scene.nodes {
                    if matches!(node.kind, SceneNodeKind::Container { .. } | SceneNodeKind::Separator | SceneNodeKind::BoxContainer) {
                        continue;
                    }
                    if !node.visibility.is_visible(workspace_flags) || !filter_mask.contains(node.id) {
                        continue;
                    }
                    let [mut x, _, w, h] = node.alignment_rect;
                    if !matches!(node.kind, SceneNodeKind::Label) && x < 16.0 {
                        x = 16.0;
                    }
                    out.insert(node.id, Rect::from_min_size(pos2(x, cursor_y), vec2(w, h)));
                    let is_paired_label =
                        matches!(node.kind, SceneNodeKind::Label) && scene.labeled_pairs.iter().any(|p| p.label_node == node.id);
                    if !is_paired_label {
                        cursor_y += h + 10.0;
                    }
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
        let origin = ui.cursor().min.to_vec2();
        let avail_w = ui.available_width();
        let viewport = vec2(self.design_size[0].max(320.0), self.design_size[1].max(160.0));
        let scale_x = if policy == LayoutPolicy::ExactAppleNib
            && avail_w.is_finite()
            && (200.0..2400.0).contains(&avail_w)
            && self.design_size[0] >= 300.0
            && avail_w > viewport.x + 4.0
        {
            (avail_w / viewport.x).clamp(1.0, 1.85)
        } else {
            1.0
        };
        let solved = self.solve_layout(viewport, policy, workspace_flags, search_query);

        for node in &self.nodes {
            let Some(&local_rect) = solved.get(&node.id) else {
                continue;
            };
            let scaled_rect = Rect::from_min_size(
                pos2(local_rect.min.x * scale_x, local_rect.min.y),
                vec2(local_rect.width() * scale_x, local_rect.height()),
            );
            let rect = scaled_rect.translate(origin);
            let ctrl_size = data_source.resolve_control_size(node.id, &node.ib_object_id, node.mac_control_size());
            let enabled = data_source.is_node_enabled(node.id, &node.ib_object_id, true);

            match &node.kind {
                SceneNodeKind::Container { .. } => {}
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
                    let resp = MacButtonCell::new(node.title.as_str())
                        .theme(theme)
                        .selected(*is_default)
                        .control_size(ctrl_size)
                        .enabled(enabled)
                        .fixed_rect(rect)
                        .show(ui);
                    if resp.clicked() {
                        if let Some(act) = node.action {
                            data_source.on_action(act, node.id);
                        }
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
                    } else {
                        let fb_id = ui.scope_id().with(("nib_txt", node.id.0));
                        let mut text = node
                            .value_binding
                            .map(|b| data_source.get_str(b).to_string())
                            .unwrap_or_else(|| ui.data(|d| d.get_temp::<String>(fb_id)).unwrap_or_else(|| node.title.clone()));
                        let mut cell = MacInputCell::with_id(ui.scope_id().with(("nib_input", node.id.0)), &mut text)
                            .theme(theme)
                            .control_size(ctrl_size)
                            .enabled(enabled)
                            .fixed_rect(rect);
                        if let SceneNodeKind::NumberField { min, max, step, .. } = &node.kind {
                            cell = cell.numeric(min.unwrap_or(-100_000.0)..=max.unwrap_or(100_000.0), (*step).max(0.1));
                        }
                        if cell.show(ui).changed() {
                            if let Some(b) = node.value_binding {
                                if matches!(node.kind, SceneNodeKind::NumberField { .. }) {
                                    if let Some(v) = crate::scene::constraints::evaluate_arithmetic_expr(&text) {
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
                    let fb_id = ui.scope_id().with(("nib_seg", node.id.0));
                    let sel = node
                        .value_binding
                        .map(|b| data_source.get_i64(b).max(0) as usize)
                        .unwrap_or_else(|| ui.data(|d| d.get_temp::<usize>(fb_id)).unwrap_or(0));
                    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
                    if !refs.is_empty() {
                        let mut next_sel = None;
                        MacSegmentedControl::new(&refs, sel)
                            .theme(theme)
                            .control_size(ctrl_size)
                            .enabled(enabled)
                            .fixed_rect(rect)
                            .show(ui, |idx| next_sel = Some(idx));
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
        for node in &self.nodes {
            if let SceneNodeKind::CustomCanvasSlot { class_name } = &node.kind {
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

fn render_token_field_node(ui: &mut Ui, theme: &MacTheme, data_source: &mut dyn MacSceneDataSource, node: &SceneNode, rect: Rect, enabled: bool) {
    let raw = node.value_binding.map(|b| data_source.get_str(b).to_string()).unwrap_or_else(|| node.title.clone());
    let mut tokens: Vec<String> = raw.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
    let prev = tokens.clone();
    let draft_id = ui.scope_id().with(("nib_tok_draft", node.id.0));
    let mut draft = ui.data(|d| d.get_temp::<String>(draft_id)).unwrap_or_default();
    let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_tok_ui", node.id.0)).max_rect(rect));
    c_ui.add_enabled_ui(enabled, |en_ui| {
        MacTokenField::new((en_ui.scope_id(), "nib_tok", node.id.0), &mut tokens, &mut draft)
            .max_chars(18)
            .width(rect.width())
            .theme(theme)
            .show(en_ui);
    });
    ui.data_mut(|d| d.insert_temp(draft_id, draft));
    if tokens != prev && enabled {
        if let Some(b) = node.value_binding {
            data_source.set_str(b, &tokens.join(","));
        }
    }
}
