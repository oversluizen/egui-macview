//! Live `MacSceneIR` node mutation and re-normalization helpers (`REQ-013-02I`).

use crate::scene::ir::{MacSceneIR, NodeId, SceneNode, SceneNodeKind};
use crate::scene::normalizer::normalize_nib_alignment_rect;

impl MacSceneIR {
    /// Updates a node's `nib_frame` and live-renormalizes `alignment_rect`, `SemanticSectionGroup`s, and `SceneSearchIndex` (`REQ-013-02I`).
    pub fn set_node_nib_frame(&mut self, id: NodeId, new_nib_frame: [f32; 4]) -> bool {
        let h_parent = self.design_size[1];
        let root_flipped = self.nodes.first().is_some_and(|n| matches!(n.kind, SceneNodeKind::Container { is_flipped: true }));
        let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) else { return false };
        node.nib_frame = new_nib_frame;
        node.alignment_rect = normalize_nib_alignment_rect(&node.kind, node.mac_control_size(), new_nib_frame, h_parent, root_flipped);
        self.recompute_semantics();
        true
    }

    /// Inserts a new node under `parent` and re-normalizes semantic groups (`REQ-013-02I`).
    pub fn insert_node(&mut self, parent: NodeId, mut node: SceneNode) -> NodeId {
        let root_flipped = self.nodes.first().is_some_and(|n| matches!(n.kind, SceneNodeKind::Container { is_flipped: true }));
        node.parent = Some(parent);
        node.alignment_rect =
            normalize_nib_alignment_rect(&node.kind, node.mac_control_size(), node.nib_frame, self.design_size[1], root_flipped);
        let id = node.id;
        if let Some(p) = self.nodes.iter_mut().find(|n| n.id == parent) {
            p.children.push(id);
        }
        self.nodes.push(node);
        self.recompute_semantics();
        id
    }

    /// Removes a node from the scene and re-normalizes semantic groups (`REQ-013-02I`).
    pub fn remove_node(&mut self, id: NodeId) -> bool {
        let len_before = self.nodes.len();
        self.nodes.retain(|n| n.id != id);
        if self.nodes.len() == len_before {
            return false;
        }
        for n in &mut self.nodes {
            n.children.retain(|c| *c != id);
        }
        self.recompute_semantics();
        true
    }
}

pub(crate) fn solve_exact_apple_nib_layout(
    scene: &MacSceneIR,
    viewport_size: egui::Vec2,
    workspace_flags: u64,
    filter_mask: &crate::scene::normalizer::SceneFilterMask,
    out: &mut std::collections::BTreeMap<NodeId, egui::Rect>,
) {
    let [w0, _] = scene.design_size;
    let is_stack_root = scene.nodes.first().and_then(|n| n.stack_orientation).is_some();
    let resized_scene;
    let eff_scene = if is_stack_root && !scene.nib_constraints.is_empty() && (viewport_size.x.max(1.0) - w0.max(1.0)).abs() > 0.5 {
        let mut cloned = scene.clone();
        cloned.design_size[0] = viewport_size.x.max(1.0);
        cloned.solve_autolayout();
        resized_scene = cloned;
        &resized_scene
    } else {
        scene
    };
    let [ew0, eh0] = eff_scene.design_size;
    for node in &eff_scene.nodes {
        if !node.visibility.is_visible(workspace_flags) || !filter_mask.contains(node.id) {
            continue;
        }
        let [x, y, w, h] = crate::scene::normalizer::apply_autoresizing_mask(
            node.alignment_rect,
            node.autoresizing_mask,
            [ew0.max(1.0), eh0.max(1.0)],
            [viewport_size.x.max(1.0), viewport_size.y.max(1.0)],
        );
        out.insert(node.id, egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)));
    }
    if !eff_scene.constraints.steps().is_empty() {
        let c_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(viewport_size.x.max(1.0), viewport_size.y.max(1.0)));
        eff_scene.constraints.evaluate(c_rect, out);
    }
}

pub(crate) fn derive_autoresizing_masks_from_constraints(scene: &mut MacSceneIR) {
    use crate::scene::constraints::{ConstraintAttr, ConstraintRelation};

    if scene.nib_constraints.is_empty() {
        return;
    }
    let id_to_idx: std::collections::BTreeMap<NodeId, usize> = scene.nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
    let n_len = scene.nodes.len();
    let mut pin_l = vec![false; n_len];
    let mut pin_r = vec![false; n_len];
    let mut pin_cx = vec![false; n_len];
    let mut pin_t = vec![false; n_len];
    let mut pin_b = vec![false; n_len];
    let mut fix_w = vec![false; n_len];
    let mut fix_h = vec![false; n_len];

    for i in 1..n_len {
        if matches!(scene.nodes[i].kind, SceneNodeKind::Stepper { .. }) {
            fix_w[i] = true;
            fix_h[i] = true;
        }
    }
    for c in &scene.nib_constraints {
        if c.relation != ConstraintRelation::Equal {
            continue;
        }
        let Some(f_idx) = c.first_item.and_then(|id| id_to_idx.get(&id).copied()) else { continue };
        if c.second_item.is_none() {
            if c.first_attr == ConstraintAttr::Width {
                fix_w[f_idx] = true;
            } else if c.first_attr == ConstraintAttr::Height {
                fix_h[f_idx] = true;
            }
        }
    }

    for _ in 0..4 {
        for c in &scene.nib_constraints {
            if c.relation != ConstraintRelation::Equal {
                continue;
            }
            let (Some(f_id), Some(s_id)) = (c.first_item, c.second_item) else { continue };
            let (Some(&fi), Some(&si)) = (id_to_idx.get(&f_id), id_to_idx.get(&s_id)) else { continue };
            for &(a_me, me, a_other, other) in &[(c.first_attr, fi, c.second_attr, si), (c.second_attr, si, c.first_attr, fi)] {
                if me == 0 || scene.nodes[other].parent != Some(scene.nodes[me].id) {
                    continue;
                }
                match a_me {
                    ConstraintAttr::Left | ConstraintAttr::Leading if matches!(a_other, ConstraintAttr::Left | ConstraintAttr::Leading) => {
                        pin_l[other] = true;
                    }
                    ConstraintAttr::Right | ConstraintAttr::Trailing if matches!(a_other, ConstraintAttr::Right | ConstraintAttr::Trailing) => {
                        pin_r[other] = true;
                    }
                    ConstraintAttr::CenterX if a_other == ConstraintAttr::CenterX => {
                        pin_cx[other] = true;
                    }
                    _ => {}
                }
            }
        }
        for c in &scene.nib_constraints {
            if c.relation != ConstraintRelation::Equal {
                continue;
            }
            let (Some(f_id), Some(s_id)) = (c.first_item, c.second_item) else { continue };
            let (Some(&fi), Some(&si)) = (id_to_idx.get(&f_id), id_to_idx.get(&s_id)) else { continue };
            for &(a_me, me, a_other, other) in &[(c.first_attr, fi, c.second_attr, si), (c.second_attr, si, c.first_attr, fi)] {
                if me == 0 || scene.nodes[other].parent == Some(scene.nodes[me].id) {
                    continue;
                }
                let is_parent = scene.nodes[me].parent == Some(scene.nodes[other].id) || other == 0;
                let is_sib = scene.nodes[me].parent == scene.nodes[other].parent;
                if !is_parent && !is_sib {
                    continue;
                }
                let has_parent_l = scene.nib_constraints.iter().any(|pc| {
                    pc.relation == ConstraintRelation::Equal
                        && ((pc.first_item == Some(scene.nodes[me].id)
                            && matches!(pc.first_attr, ConstraintAttr::Left | ConstraintAttr::Leading)
                            && pc.second_item == scene.nodes[me].parent)
                            || (pc.second_item == Some(scene.nodes[me].id)
                                && matches!(pc.second_attr, ConstraintAttr::Left | ConstraintAttr::Leading)
                                && pc.first_item == scene.nodes[me].parent))
                });
                let has_parent_r = scene.nib_constraints.iter().any(|pc| {
                    pc.relation == ConstraintRelation::Equal
                        && ((pc.first_item == Some(scene.nodes[me].id)
                            && matches!(pc.first_attr, ConstraintAttr::Right | ConstraintAttr::Trailing)
                            && pc.second_item == scene.nodes[me].parent)
                            || (pc.second_item == Some(scene.nodes[me].id)
                                && matches!(pc.second_attr, ConstraintAttr::Right | ConstraintAttr::Trailing)
                                && pc.first_item == scene.nodes[me].parent))
                });
                match a_me {
                    ConstraintAttr::Left | ConstraintAttr::Leading => {
                        if (is_parent && matches!(a_other, ConstraintAttr::Left | ConstraintAttr::Leading))
                            || (is_sib && pin_l[other] && !has_parent_r && !pin_cx[other])
                        {
                            pin_l[me] = true;
                        } else if is_sib && pin_r[other] && !has_parent_l && !pin_cx[other] {
                            pin_r[me] = true;
                        }
                    }
                    ConstraintAttr::Right | ConstraintAttr::Trailing => {
                        if (is_parent && matches!(a_other, ConstraintAttr::Right | ConstraintAttr::Trailing))
                            || (is_sib && pin_r[other] && !has_parent_l && !pin_cx[other])
                        {
                            pin_r[me] = true;
                        } else if is_sib && pin_l[other] && !has_parent_r && !pin_cx[other] {
                            pin_l[me] = true;
                        }
                    }
                    ConstraintAttr::CenterX if (is_parent && a_other == ConstraintAttr::CenterX) || (is_sib && pin_cx[other]) => {
                        pin_cx[me] = true
                    }
                    ConstraintAttr::Top => {
                        if (is_parent && a_other == ConstraintAttr::Top)
                            || (is_sib && a_other == ConstraintAttr::Bottom && (pin_t[other] || pin_b[other]))
                        {
                            if is_parent || pin_t[other] {
                                pin_t[me] = true;
                            } else {
                                pin_b[me] = true;
                            }
                        }
                    }
                    ConstraintAttr::Bottom => {
                        if (is_parent && a_other == ConstraintAttr::Bottom)
                            || (is_sib && a_other == ConstraintAttr::Top && (pin_b[other] || pin_t[other]))
                        {
                            if is_parent || pin_b[other] {
                                pin_b[me] = true;
                            } else {
                                pin_t[me] = true;
                            }
                        }
                    }
                    ConstraintAttr::CenterY | ConstraintAttr::FirstBaseline | ConstraintAttr::LastBaseline if is_sib => {
                        if pin_t[other] {
                            pin_t[me] = true;
                        } else if pin_b[other] {
                            pin_b[me] = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let mut derived = vec![None::<u8>; n_len];
    for i in 1..n_len {
        if !(pin_l[i] || pin_r[i] || pin_cx[i] || fix_w[i] || pin_t[i] || pin_b[i] || fix_h[i]) {
            continue;
        }
        let h_bits = if pin_l[i] && pin_r[i] && !fix_w[i] {
            0x02
        } else if pin_cx[i] {
            0x05
        } else if pin_r[i] && !pin_l[i] {
            0x01
        } else {
            0x04
        };
        let v_bits = if pin_t[i]
            && pin_b[i]
            && !fix_h[i]
            && !matches!(
                scene.nodes[i].kind,
                SceneNodeKind::SearchField
                    | SceneNodeKind::TextField
                    | SceneNodeKind::NumberField { .. }
                    | SceneNodeKind::ComboBox { .. }
                    | SceneNodeKind::PopUpButton { .. }
                    | SceneNodeKind::PushButton { .. }
            ) {
            0x10
        } else if pin_b[i] && (!pin_t[i] || fix_h[i]) {
            0x20
        } else {
            0x08
        };
        derived[i] = Some(h_bits | v_bits);
    }

    for i in 1..n_len {
        let mut mask = derived[i];
        let mut cur_p = scene.nodes[i].parent;
        while let Some(pid) = cur_p {
            let Some(&pi) = id_to_idx.get(&pid) else { break };
            if pi == 0 {
                break;
            }
            if let Some(pm) = derived[pi] {
                mask = Some(match mask {
                    None => pm,
                    Some(cm) => {
                        let h = if (pm & 0x07) == 0x01 || (!pin_l[i] && !pin_r[i] && !pin_cx[i] && !fix_w[i]) { pm & 0x07 } else { cm & 0x07 };
                        let v = if ((pm & 0x38) == 0x20 && !pin_t[pi]) || (!pin_t[i] && !pin_b[i] && !fix_h[i]) { pm & 0x38 } else { cm & 0x38 };
                        h | v
                    }
                });
            }
            cur_p = scene.nodes[pi].parent;
        }
        if let Some(m) = mask.filter(|_| matches!(scene.nodes[i].autoresizing_mask, 0 | 12)) {
            scene.nodes[i].autoresizing_mask = m;
        }
    }
}

