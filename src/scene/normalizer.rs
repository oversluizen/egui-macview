//! Spatial Semantic Normalizer, Alignment-Rect Inset Normalizer, Custom Class Mapper,
//! and Host Contract Audit Types for `MacSceneIR` (`SPEC-013` §3.2, §3.5, §4.1.1).

use crate::scene::ir::{
    ActionId, BindingId, BindingValueType, FieldRowGroup, LabeledFieldPair, MacSceneIR, NodeId, SceneNode, SceneNodeKind,
    SemanticSectionGroup,
};
use crate::MacControlSize;
use std::collections::{BTreeMap, BTreeSet};

/// Configurable custom subclass mapper (`NSClassSwapper` resolution, `SPEC-013 §2.2` Mechanism 3).
#[derive(Debug, Clone, Default)]
pub struct CustomClassMapper {
    stepping_classes: BTreeSet<String>,
    origin_classes: BTreeSet<String>,
    inline_button_classes: BTreeSet<String>,
    color_well_classes: BTreeSet<String>,
}

impl CustomClassMapper {
    /// Creates a mapper pre-populated with common studio custom control subclass patterns.
    pub fn default_studio_mappings() -> Self {
        let mut m = Self::default();
        for name in ["SteppingTextField", "GSSteppingTextField", "CalculatingTextField"] {
            m.stepping_classes.insert(name.to_string());
        }
        for name in ["OriginPointControl", "GSOriginPointControl", "TransformGrid", "GSTransformGrid"] {
            m.origin_classes.insert(name.to_string());
        }
        for name in ["MouseOverButton", "GSMouseOverButton", "InlineActionButton", "GSInlineActionButton"] {
            m.inline_button_classes.insert(name.to_string());
        }
        for name in ["PopoverColorWell", "BFPopoverColorWell"] {
            m.color_well_classes.insert(name.to_string());
        }
        m
    }

    /// Returns true if `class_name` maps to a stepping `NumberField`.
    pub fn is_stepping_number_field(&self, class_name: &str) -> bool {
        self.stepping_classes.contains(class_name) || class_name.contains("SteppingTextField")
    }

    /// Returns true if `class_name` maps to an `OriginMatrix`.
    pub fn is_origin_matrix(&self, class_name: &str) -> bool {
        self.origin_classes.contains(class_name) || class_name.contains("OriginPoint") || class_name.contains("TransformGrid")
    }

    /// Returns true if `class_name` maps to an `InlineActionButton`.
    pub fn is_inline_button(&self, class_name: &str) -> bool {
        self.inline_button_classes.contains(class_name) || class_name.contains("MouseOverButton")
    }

    /// Returns true if `class_name` maps to a `ColorWell`.
    pub fn is_color_well(&self, class_name: &str) -> bool {
        self.color_well_classes.contains(class_name) || class_name.contains("ColorWell")
    }
}

/// Filter mask returned by `MacSceneIR::filter_by_search`.
#[derive(Debug, Clone, Default)]
pub struct SceneFilterMask {
    pub(crate) matched_nodes: BTreeSet<NodeId>,
}

impl SceneFilterMask {
    /// Returns the number of matched nodes.
    pub fn matched_node_count(&self) -> usize {
        self.matched_nodes.len()
    }

    /// Returns true if `node` matches the filter.
    pub fn contains(&self, node: NodeId) -> bool {
        self.matched_nodes.contains(&node)
    }
}

/// Host contract schema for compile-time / COTS test-time binding verification (`REQ-013-02` §3.5).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HostContractSchema {
    /// Registered bindings and their value types.
    pub bindings: BTreeMap<BindingId, BindingValueType>,
    /// Registered action selectors.
    pub actions: BTreeSet<ActionId>,
    /// Registered custom icon names.
    pub icons: BTreeSet<String>,
}

impl HostContractSchema {
    /// Creates an empty host contract schema.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Verification report returned by `MacSceneIR::verify_contract`.
#[derive(Debug, Clone, Default)]
pub struct ContractAuditReport {
    /// Unbound or missing `BindingId`s.
    pub missing_bindings: Vec<BindingId>,
    /// Unregistered `ActionId`s.
    pub missing_actions: Vec<ActionId>,
    /// Unresolved custom icon resource names.
    pub unresolved_icons: Vec<String>,
}

impl ContractAuditReport {
    /// Returns true when 100% of scene bindings, actions, and icons are satisfied.
    pub fn is_ok(&self) -> bool {
        self.missing_bindings.is_empty() && self.missing_actions.is_empty() && self.unresolved_icons.is_empty()
    }
}

/// Normalizes an AppKit `NSFrame` into a visual `alignmentRect` (`REQ-013-03A` §4.1.1).
pub fn normalize_nib_alignment_rect(
    kind: &SceneNodeKind,
    size: MacControlSize,
    nib_frame: [f32; 4],
    parent_h: f32,
    parent_is_flipped: bool,
) -> [f32; 4] {
    let [mut x, mut y, mut w, mut h] = nib_frame;
    if !parent_is_flipped && parent_h > 0.0 {
        y = parent_h - (y + h);
    }
    match kind {
        SceneNodeKind::PushButton { .. } => {
            if h >= 38.0 {
                x += 6.0;
                y += 6.0;
                w = (w - 12.0).max(24.0);
                h = 28.0;
            } else if h >= 30.0 {
                x += 6.0;
                y += (h - 20.0) * 0.5;
                w = (w - 12.0).max(24.0);
                h = 20.0;
            } else {
                let target_h = match size {
                    MacControlSize::Large => 28.0,
                    MacControlSize::Regular | MacControlSize::CompactInfobox => 20.0,
                    MacControlSize::Small => 17.0,
                    MacControlSize::Mini => 14.0,
                };
                if h > target_h {
                    y += (h - target_h) * 0.5;
                    h = target_h;
                }
            }
        }
        SceneNodeKind::PopUpButton { .. } => {
            if h >= 25.0 {
                x += 3.0;
                w = (w - 6.0).max(24.0);
                y += (h - 20.0) * 0.5;
                h = 20.0;
            }
        }
        SceneNodeKind::Checkbox | SceneNodeKind::RadioButton if x < 0.0 => {
            x += 2.0;
            w = (w - 2.0).max(16.0);
        }
        _ => {}
    }
    [x, y, w, h]
}

pub(crate) fn run_spatial_semantic_normalizer(
    nodes: &mut [SceneNode],
) -> (Vec<LabeledFieldPair>, Vec<FieldRowGroup>, Vec<SemanticSectionGroup>) {
    let mut pairs = Vec::new();
    let mut rows = Vec::new();

    let labels: Vec<(NodeId, [f32; 4], String)> =
        nodes.iter().filter(|n| matches!(n.kind, SceneNodeKind::Label)).map(|n| (n.id, n.alignment_rect, n.title.clone())).collect();

    for node in nodes.iter_mut() {
        let is_interactive = !matches!(
            node.kind,
            SceneNodeKind::Container { .. } | SceneNodeKind::BoxContainer | SceneNodeKind::Separator | SceneNodeKind::Label
        );
        if !is_interactive {
            continue;
        }
        let [cx, cy, _, ch] = node.alignment_rect;
        let c_mid_y = cy + ch * 0.5;
        let mut best_lbl: Option<(NodeId, f32)> = None;
        for (lid, [lx, ly, lw, lh], _) in &labels {
            let l_mid_y = ly + lh * 0.5;
            let dy = (c_mid_y - l_mid_y).abs();
            let dx = cx - (lx + lw);
            if dy <= 8.0 && cx > *lx && (-32.0..=64.0).contains(&dx) {
                let score = dy + dx.abs() * 0.25;
                if best_lbl.is_none_or(|(_, b)| score < b) {
                    best_lbl = Some((*lid, score));
                }
            }
        }
        if let Some((lid, _)) = best_lbl {
            node.labeled_by = Some(lid);
            pairs.push(LabeledFieldPair { label_node: lid, control_node: node.id });
        }
    }

    let mut sep_ys: Vec<f32> = nodes.iter().filter(|n| matches!(n.kind, SceneNodeKind::Separator)).map(|n| n.alignment_rect[1]).collect();
    sep_ys.sort_by(|a, b| a.total_cmp(b));

    let mut boundaries = vec![-1e6_f32];
    boundaries.extend(sep_ys);
    boundaries.push(1e6_f32);

    let mut sections = Vec::new();
    for (sec_idx, win) in boundaries.windows(2).enumerate() {
        let (y_min, y_max) = (win[0], win[1]);
        let mut box_title: Option<String> = None;
        let mut sec_nodes = Vec::new();
        for n in nodes.iter() {
            let ny = n.alignment_rect[1];
            if ny <= y_min || ny > y_max {
                continue;
            }
            if matches!(n.kind, SceneNodeKind::BoxContainer) {
                if !n.title.is_empty() && box_title.is_none() {
                    box_title = Some(n.title.clone());
                }
                continue;
            }
            if matches!(n.kind, SceneNodeKind::Container { .. } | SceneNodeKind::Separator) {
                continue;
            }
            sec_nodes.push(n.id);
        }
        if !sec_nodes.is_empty() {
            let sec_pairs: Vec<LabeledFieldPair> = pairs.iter().filter(|p| sec_nodes.contains(&p.control_node)).cloned().collect();
            let mut visited = BTreeSet::new();
            for &nid in &sec_nodes {
                if visited.contains(&nid) {
                    continue;
                }
                let base_y = nodes.iter().find(|n| n.id == nid).map(|n| n.alignment_rect[1]).unwrap_or(0.0);
                let mut row_ids = Vec::new();
                for &other_id in &sec_nodes {
                    if !visited.contains(&other_id) {
                        let oy = nodes.iter().find(|n| n.id == other_id).map(|n| n.alignment_rect[1]).unwrap_or(0.0);
                        if (oy - base_y).abs() <= 8.0 {
                            visited.insert(other_id);
                            row_ids.push(other_id);
                        }
                    }
                }
                if !row_ids.is_empty() {
                    rows.push(FieldRowGroup { nodes: row_ids });
                }
            }
            sections.push(SemanticSectionGroup {
                title: box_title.unwrap_or_else(|| format!("Section {}", sec_idx + 1)),
                nodes: sec_nodes,
                pairs: sec_pairs,
            });
        }
    }
    (pairs, rows, sections)
}

/// Evaluates AppKit's 6-bit `NSAutoresizingMaskOptions` (`resizeWithOldSuperviewSize:`, `SPEC-013 §4.1.2`).
pub fn apply_autoresizing_mask(rect0: [f32; 4], mask: u8, old_parent: [f32; 2], new_parent: [f32; 2]) -> [f32; 4] {
    let [x0, y0, w0, h0] = rect0;
    let (x, w) = solve_axis(x0, w0, old_parent[0], new_parent[0], (mask & 0x01) != 0, (mask & 0x02) != 0, (mask & 0x04) != 0);
    let (y, h) = solve_axis(y0, h0, old_parent[1], new_parent[1], (mask & 0x20) != 0, (mask & 0x10) != 0, (mask & 0x08) != 0);
    [x, y, w, h]
}

fn solve_axis(pos0: f32, len0: f32, old_total: f32, new_total: f32, flex_min: bool, flex_size: bool, flex_max: bool) -> (f32, f32) {
    let delta = new_total - old_total;
    if delta.abs() < 1e-4 || (!flex_min && !flex_size && !flex_max) {
        return (pos0, len0);
    }
    let min_m = pos0.max(0.0);
    let sz = len0.max(0.0);
    let max_m = (old_total - (pos0 + len0)).max(0.0);
    let mut flex_sum = 0.0_f32;
    let mut flex_count = 0usize;
    for (active, m) in [(flex_min, min_m), (flex_size, sz), (flex_max, max_m)] {
        if active {
            flex_sum += m;
            flex_count += 1;
        }
    }
    let share = |orig: f32, active: bool| -> f32 {
        if !active {
            0.0
        } else if flex_sum > 1e-4 {
            delta * (orig / flex_sum)
        } else {
            delta / (flex_count as f32)
        }
    };
    (pos0 + share(min_m, flex_min), (len0 + share(sz, flex_size)).max(1.0))
}

impl MacSceneIR {
    /// Verifies that all bindings, actions, and icons in this scene match `schema` (`REQ-013-02` §3.5).
    pub fn verify_contract(&self, schema: &HostContractSchema) -> ContractAuditReport {
        let mut report = ContractAuditReport::default();
        for n in &self.nodes {
            if let Some(b) = n.value_binding {
                if !schema.bindings.contains_key(&b) {
                    report.missing_bindings.push(b);
                }
            }
            if let Some(a) = n.action {
                if !schema.actions.contains(&a) {
                    report.missing_actions.push(a);
                }
            }
            if let Some(ref icon) = n.icon_name {
                if !icon.starts_with("NS") && !schema.icons.contains(icon) {
                    report.unresolved_icons.push(icon.clone());
                }
            }
        }
        report
    }

    /// Derives a matching `HostContractSchema` from the scene's declared bindings, actions, and icons.
    pub fn derive_matching_schema(&self) -> HostContractSchema {
        let mut schema = HostContractSchema::new();
        for n in &self.nodes {
            if let Some(b) = n.value_binding {
                let ty = match n.kind {
                    SceneNodeKind::Checkbox | SceneNodeKind::RadioButton | SceneNodeKind::Switch => BindingValueType::Bool,
                    SceneNodeKind::NumberField { .. } | SceneNodeKind::Slider { .. } => BindingValueType::F64,
                    SceneNodeKind::PopUpButton { .. } | SceneNodeKind::SegmentedControl { .. } | SceneNodeKind::Stepper { .. } => {
                        BindingValueType::I64
                    }
                    _ => BindingValueType::String,
                };
                schema.bindings.insert(b, ty);
            }
            if let Some(a) = n.action {
                schema.actions.insert(a);
            }
            if let Some(ref icon) = n.icon_name {
                schema.icons.insert(icon.clone());
            }
        }
        schema
    }

    /// Validates structural, topological, and search-index invariants on `MacSceneIR` (`REQ-013-02`).
    pub fn validate_invariants(&self) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        for n in &self.nodes {
            if !ids.insert(n.id) {
                return Err(format!("Duplicate NodeId {:?}", n.id));
            }
        }
        if !ids.contains(&self.root) {
            return Err(format!("Root NodeId {:?} missing from arena", self.root));
        }
        for n in &self.nodes {
            if let Some(p) = n.parent {
                if !ids.contains(&p) {
                    return Err(format!("Node {:?} references missing parent {:?}", n.id, p));
                }
            }
            for &c in &n.children {
                if !ids.contains(&c) {
                    return Err(format!("Node {:?} references missing child {:?}", n.id, c));
                }
            }
            if let Some(lbl) = n.labeled_by {
                if !ids.contains(&lbl) {
                    return Err(format!("Node {:?} references missing label {:?}", n.id, lbl));
                }
            }
        }
        if self.search_index.len() != self.nodes.len() {
            return Err("SceneSearchIndex length out of sync with nodes".into());
        }
        Ok(())
    }
}
