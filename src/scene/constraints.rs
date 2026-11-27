//! Pre-compiled topological `NSLayoutConstraint` DAG schedule, `NSValueTransformer` evaluation,
//! and deterministic inline arithmetic expression evaluator (`SPEC-013 §2.2`, `§3.1.5`, `§4.1.3`).

use crate::scene::ir::NodeId;
use egui::{pos2, vec2, Rect};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Built-in AppKit `NSValueTransformer` truth table (`REQ-013-02E`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ValueTransformer {
    /// Identity transformer (passes value unchanged).
    #[default]
    Identity,
    /// `NSNegateBoolean` transformer (`b -> !b`).
    NegateBoolean,
    /// `NSIsNotNil` transformer (`!s.is_empty()`).
    IsNotNil,
    /// `NSIsNil` transformer (`s.is_empty()`).
    IsNil,
    /// Custom named application transformer.
    Custom(String),
}

impl ValueTransformer {
    /// Evaluates a boolean value through the transformer.
    #[inline]
    pub fn apply_bool(&self, val: bool) -> bool {
        match self {
            Self::NegateBoolean | Self::IsNil => !val,
            _ => val,
        }
    }

    /// Evaluates a string value into a boolean presence flag (`IsNotNil` / `IsNil`).
    #[inline]
    pub fn apply_str(&self, val: &str) -> bool {
        match self {
            Self::IsNotNil | Self::Identity | Self::Custom(_) => !val.is_empty(),
            Self::IsNil | Self::NegateBoolean => val.is_empty(),
        }
    }
}

/// AppKit `NSLayoutAttribute` codes (`1..=12`, `REQ-013-03B`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintAttr {
    /// Not an attribute (`0`, used for constant width/height constraints).
    NotAnAttribute,
    /// Left edge (`1`).
    Left,
    /// Right edge (`2`).
    Right,
    /// Top edge (`3`).
    Top,
    /// Bottom edge (`4`).
    Bottom,
    /// Leading edge (`5`, maps to `Left` in LTR).
    Leading,
    /// Trailing edge (`6`, maps to `Right` in LTR).
    Trailing,
    /// Width dimension (`7`).
    Width,
    /// Height dimension (`8`).
    Height,
    /// Horizontal center (`9`).
    CenterX,
    /// Vertical center (`10`).
    CenterY,
    /// Last baseline (`11`).
    LastBaseline,
    /// First baseline (`12`).
    FirstBaseline,
}

impl ConstraintAttr {
    /// Decodes an integer `NSLayoutAttribute` code from a binary `.nib` or `.xib`.
    pub const fn from_code(code: i32) -> Self {
        match code {
            1 => Self::Left,
            2 => Self::Right,
            3 => Self::Top,
            4 => Self::Bottom,
            5 => Self::Leading,
            6 => Self::Trailing,
            7 => Self::Width,
            8 => Self::Height,
            9 => Self::CenterX,
            10 => Self::CenterY,
            11 => Self::LastBaseline,
            12 => Self::FirstBaseline,
            _ => Self::NotAnAttribute,
        }
    }
}

/// AppKit `NSLayoutRelation` codes (`-1`, `0`, `+1`, `REQ-013-03B`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintRelation {
    /// Less than or equal (`-1`).
    LessThanOrEqual,
    /// Equal (`0`).
    Equal,
    /// Greater than or equal (`+1`).
    GreaterThanOrEqual,
}

/// Single `NSLayoutConstraint` specification ($A.\text{attr}_1 \;\text{rel}\; m \cdot B.\text{attr}_2 + c$).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutConstraintSpec {
    /// Target node $A$ (`None` represents container superview).
    pub first_item: Option<NodeId>,
    /// Target attribute on $A$.
    pub first_attr: ConstraintAttr,
    /// Relation operator (`<=`, `==`, `>=`).
    pub relation: ConstraintRelation,
    /// Reference node $B$ (`None` represents container superview or constant).
    pub second_item: Option<NodeId>,
    /// Reference attribute on $B$.
    pub second_attr: ConstraintAttr,
    /// Linear multiplier $m$.
    pub multiplier: f32,
    /// Constant offset $c$ in points.
    pub constant: f32,
    /// Constraint priority (`1..=1000`).
    pub priority: u16,
}

/// Pre-compiled topological evaluation schedule for `NSLayoutConstraint` chains (`REQ-013-03B`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CompiledConstraintSchedule {
    steps: Vec<LayoutConstraintSpec>,
}

impl CompiledConstraintSchedule {
    /// Compiles a list of `LayoutConstraintSpec` items into a topologically ordered 1-pass schedule.
    pub fn compile(mut specs: Vec<LayoutConstraintSpec>) -> Self {
        specs.sort_by_key(|c| {
            let dep_rank = if c.second_item.is_none() { 0 } else { 1 };
            let attr_rank = match c.first_attr {
                ConstraintAttr::Width | ConstraintAttr::Height => 0,
                ConstraintAttr::Left | ConstraintAttr::Leading | ConstraintAttr::Top => 1,
                _ => 2,
            };
            (dep_rank, attr_rank, std::cmp::Reverse(c.priority))
        });
        Self { steps: specs }
    }

    /// Returns the compiled constraint steps.
    pub fn steps(&self) -> &[LayoutConstraintSpec] {
        &self.steps
    }

    /// Evaluates the compiled constraint schedule in-place over `rects` within `container_rect`.
    pub fn evaluate(&self, container_rect: Rect, rects: &mut BTreeMap<NodeId, Rect>) {
        for step in &self.steps {
            let Some(target_id) = step.first_item else {
                continue;
            };
            let ref_rect = match step.second_item {
                Some(sec_id) => rects.get(&sec_id).copied().unwrap_or(container_rect),
                None => container_rect,
            };
            let rhs_base = read_attr(ref_rect, step.second_attr);
            let target_val = step.multiplier * rhs_base + step.constant;

            if let Some(cur) = rects.get_mut(&target_id) {
                apply_attr(cur, step.first_attr, step.relation, target_val);
            }
        }
    }
}

fn read_attr(r: Rect, attr: ConstraintAttr) -> f32 {
    match attr {
        ConstraintAttr::NotAnAttribute => 0.0,
        ConstraintAttr::Left | ConstraintAttr::Leading => r.min.x,
        ConstraintAttr::Right | ConstraintAttr::Trailing => r.max.x,
        ConstraintAttr::Top => r.min.y,
        ConstraintAttr::Bottom | ConstraintAttr::LastBaseline | ConstraintAttr::FirstBaseline => r.max.y,
        ConstraintAttr::Width => r.width(),
        ConstraintAttr::Height => r.height(),
        ConstraintAttr::CenterX => r.center().x,
        ConstraintAttr::CenterY => r.center().y,
    }
}

fn apply_attr(r: &mut Rect, attr: ConstraintAttr, rel: ConstraintRelation, val: f32) {
    let w = r.width();
    let h = r.height();
    let resolve = |cur: f32| match rel {
        ConstraintRelation::Equal => val,
        ConstraintRelation::LessThanOrEqual => cur.min(val),
        ConstraintRelation::GreaterThanOrEqual => cur.max(val),
    };
    match attr {
        ConstraintAttr::Left | ConstraintAttr::Leading => {
            let nx = resolve(r.min.x);
            *r = Rect::from_min_size(pos2(nx, r.min.y), vec2(w, h));
        }
        ConstraintAttr::Right | ConstraintAttr::Trailing => {
            let nr = resolve(r.max.x);
            *r = Rect::from_min_size(pos2(nr - w, r.min.y), vec2(w, h));
        }
        ConstraintAttr::Top => {
            let ny = resolve(r.min.y);
            *r = Rect::from_min_size(pos2(r.min.x, ny), vec2(w, h));
        }
        ConstraintAttr::Bottom | ConstraintAttr::LastBaseline | ConstraintAttr::FirstBaseline => {
            let nb = resolve(r.max.y);
            *r = Rect::from_min_size(pos2(r.min.x, nb - h), vec2(w, h));
        }
        ConstraintAttr::Width => {
            let nw = resolve(w).max(0.0);
            *r = Rect::from_min_size(r.min, vec2(nw, h));
        }
        ConstraintAttr::Height => {
            let nh = resolve(h).max(0.0);
            *r = Rect::from_min_size(r.min, vec2(w, nh));
        }
        ConstraintAttr::CenterX => {
            let cx = resolve(r.center().x);
            *r = Rect::from_min_size(pos2(cx - w * 0.5, r.min.y), vec2(w, h));
        }
        ConstraintAttr::CenterY => {
            let cy = resolve(r.center().y);
            *r = Rect::from_min_size(pos2(r.min.x, cy - h * 0.5), vec2(w, h));
        }
        ConstraintAttr::NotAnAttribute => {}
    }
}

/// Evaluates an inline arithmetic expression (`+`, `-`, `*`, `/`, parentheses, unary `-`)
/// for `NumberField` (`NSNumberFormatter`, `REQ-013-02E`).
pub fn evaluate_arithmetic_expr(input: &str) -> Option<f64> {
    struct Parser<'a> {
        bytes: &'a [u8],
        pos: usize,
    }
    impl<'a> Parser<'a> {
        fn skip_ws(&mut self) {
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            }
        }
        fn parse_expr(&mut self) -> Option<f64> {
            let mut lhs = self.parse_term()?;
            loop {
                self.skip_ws();
                if self.pos >= self.bytes.len() {
                    break;
                }
                match self.bytes[self.pos] {
                    b'+' => {
                        self.pos += 1;
                        lhs += self.parse_term()?;
                    }
                    b'-' => {
                        self.pos += 1;
                        lhs -= self.parse_term()?;
                    }
                    _ => break,
                }
            }
            Some(lhs)
        }
        fn parse_term(&mut self) -> Option<f64> {
            let mut lhs = self.parse_factor()?;
            loop {
                self.skip_ws();
                if self.pos >= self.bytes.len() {
                    break;
                }
                match self.bytes[self.pos] {
                    b'*' => {
                        self.pos += 1;
                        lhs *= self.parse_factor()?;
                    }
                    b'/' => {
                        self.pos += 1;
                        let rhs = self.parse_factor()?;
                        if rhs.abs() < 1e-12 {
                            return None;
                        }
                        lhs /= rhs;
                    }
                    _ => break,
                }
            }
            Some(lhs)
        }
        fn parse_factor(&mut self) -> Option<f64> {
            self.skip_ws();
            if self.pos >= self.bytes.len() {
                return None;
            }
            if self.bytes[self.pos] == b'+' {
                self.pos += 1;
                return self.parse_factor();
            }
            if self.bytes[self.pos] == b'-' {
                self.pos += 1;
                return self.parse_factor().map(|v| -v);
            }
            if self.bytes[self.pos] == b'(' {
                self.pos += 1;
                let val = self.parse_expr()?;
                self.skip_ws();
                if self.pos >= self.bytes.len() || self.bytes[self.pos] != b')' {
                    return None;
                }
                self.pos += 1;
                return Some(val);
            }
            let start = self.pos;
            while self.pos < self.bytes.len() && (self.bytes[self.pos].is_ascii_digit() || self.bytes[self.pos] == b'.') {
                self.pos += 1;
            }
            if self.pos == start {
                return None;
            }
            std::str::from_utf8(&self.bytes[start..self.pos]).ok()?.parse::<f64>().ok()
        }
    }

    let mut parser = Parser { bytes: input.as_bytes(), pos: 0 };
    let val = parser.parse_expr()?;
    parser.skip_ws();
    if parser.pos == parser.bytes.len() {
        Some(val)
    } else {
        None
    }
}

/// Returns the macOS AppKit `-[NSView alignmentRectInsets]` (`[top, left, bottom, right]` in points)
/// for a given [`crate::scene::ir::SceneNodeKind`], verified against live macOS AppKit telemetry
/// (`REQ-013-10.1` & `REQ-013-10.2`).
pub fn appkit_alignment_rect_insets(kind: &crate::scene::ir::SceneNodeKind) -> [f32; 4] {
    use crate::scene::ir::SceneNodeKind;
    match kind {
        SceneNodeKind::Label => [0.0, 2.0, 0.0, 2.0],
        SceneNodeKind::ComboBox { .. } => [1.0, 1.0, 2.0, 3.0],
        SceneNodeKind::Checkbox => [1.0, 2.0, 1.0, 0.0],
        SceneNodeKind::PushButton { .. } => [1.0, 1.0, 1.0, 1.0],
        _ => [0.0, 0.0, 0.0, 0.0],
    }
}

pub(crate) fn unroll_ibtool_nsmatrix(
    st: &mut crate::scene::nib_ibtool::IbDecodeState<'_>,
    child_obj: usize,
    parent_node_id: crate::scene::ir::NodeId,
) {
    use crate::scene::ir::{NodeId, SceneNodeKind, VisibilityRule};
    use crate::scene::nib_frontend::mk_node;
    use crate::scene::nib_ibtool::IbVal;
    let mut m_frame = st.graph.resolve_rect(child_obj).unwrap_or([0.0, 0.0, 170.0, 52.0]);
    let root_flipped = st.nodes.first().is_some_and(|n| matches!(n.kind, SceneNodeKind::Container { is_flipped: true }));
    if !root_flipped {
        if let Some(p_node) = st.nodes.iter().find(|n| n.id == parent_node_id) {
            if p_node.parent.is_some() {
                m_frame[0] += p_node.nib_frame[0];
                m_frame[1] += p_node.nib_frame[1];
            }
        }
    }
    let cells =
        st.graph.prop_val(child_obj, "NSCells").and_then(IbVal::as_obj_ref).map(|a| st.graph.resolve_array_refs(a)).unwrap_or_default();
    let matrix_cell_cls = cells.iter().find_map(|&c| st.graph.prop_val(c, "NSClassName").and_then(|v| st.graph.resolve_str(v)));
    if matrix_cell_cls.as_deref().is_some_and(|s| s.contains("ColorLabel")) {
        let ib_id = ["NSReuseIdentifierKey", "NSUserInterfaceItemIdentifier", "NSIdentifier", "NSID"]
            .into_iter()
            .find_map(|k| st.graph.prop_val(child_obj, k).and_then(|v| st.graph.resolve_str(v)))
            .unwrap_or_else(|| format!("_NS:{child_obj}"));
        let nid = NodeId(st.next_id);
        st.next_id += 1;
        let mut t_node = mk_node(
            nid.0,
            &ib_id,
            Some(parent_node_id.0),
            vec![],
            SceneNodeKind::TagColorPicker,
            "",
            "",
            "small",
            m_frame,
            0,
            None,
            None,
            None,
            VisibilityRule::Always,
        );
        t_node.custom_class = Some("NSMatrix".into());
        t_node.cell_class = matrix_cell_cls;
        if let Some(p_node) = st.nodes.iter_mut().find(|n| n.id == parent_node_id) {
            p_node.children.push(nid);
        }
        st.obj_to_node_idx.insert(child_obj, st.nodes.len());
        st.nodes.push(t_node);
        return;
    }
    let row_cnt = cells.len().max(1);
    let row_h = (m_frame[3] / row_cnt as f32).max(16.0);
    for (r_idx, cell_obj) in cells.into_iter().enumerate() {
        let nid = NodeId(st.next_id);
        st.next_id += 1;
        let cell_title = st
            .graph
            .prop_val(cell_obj, "NSContents")
            .or_else(|| st.graph.prop_val(cell_obj, "NSTitle"))
            .and_then(|v| st.graph.resolve_str(v))
            .unwrap_or_default();
        let cell_y = m_frame[1] + ((row_cnt - 1 - r_idx) as f32) * row_h;
        let cell_rect = [m_frame[0], cell_y, m_frame[2], row_h];
        let r_node = mk_node(
            nid.0,
            &format!("nib-matrix-{child_obj}-{r_idx}"),
            Some(parent_node_id.0),
            vec![],
            SceneNodeKind::RadioButton,
            &cell_title,
            "",
            "regular",
            cell_rect,
            0,
            None,
            None,
            None,
            VisibilityRule::Always,
        );
        if let Some(p_node) = st.nodes.iter_mut().find(|n| n.id == parent_node_id) {
            p_node.children.push(nid);
        }
        st.obj_to_node_idx.insert(cell_obj, st.nodes.len());
        st.nodes.push(r_node);
    }
}

pub(crate) fn collect_ibtool_subviews(
    st: &mut crate::scene::nib_ibtool::IbDecodeState<'_>,
    parent_view_obj: usize,
    parent_node_id: crate::scene::ir::NodeId,
) {
    use crate::scene::ir::{NodeId, SceneNodeKind, VisibilityRule};
    use crate::scene::nib_ibtool::IbVal;
    use crate::scene::nib_ibtool_widgets::build_ibtool_node;

    let subviews = st
        .graph
        .prop_val(parent_view_obj, "NSSubviews")
        .and_then(IbVal::as_obj_ref)
        .map(|arr| st.graph.resolve_array_refs(arr))
        .unwrap_or_default();
    for child_obj in subviews {
        if st.graph.obj_class(child_obj) == "NSMatrix" {
            unroll_ibtool_nsmatrix(st, child_obj, parent_node_id);
            continue;
        }
        let nid = NodeId(st.next_id);
        st.next_id += 1;
        let node = build_ibtool_node(st, child_obj, nid, parent_node_id);
        let is_box = matches!(node.kind, SceneNodeKind::BoxContainer);
        let raw_c = st.graph.obj_class(child_obj);
        let custom_c = st.graph.prop_val(child_obj, "NSClassName").and_then(|v| st.graph.resolve_str(v)).unwrap_or_default();
        let orig_c = st.graph.prop_val(child_obj, "NSOriginalClassName").and_then(|v| st.graph.resolve_str(v)).unwrap_or_default();
        let eff_c = if !custom_c.is_empty() {
            custom_c.as_str()
        } else if !orig_c.is_empty() {
            orig_c.as_str()
        } else {
            raw_c
        };
        let is_container_view =
            (matches!(eff_c, "NSView" | "NSCustomView" | "NSSplitView" | "NSStackView" | "NSStackViewContainer" | "NSClipView")
                || (eff_c.ends_with("View")
                    && !eff_c.contains("Table")
                    && !eff_c.contains("Outline")
                    && !eff_c.contains("Text")
                    && !eff_c.contains("Image")
                    && !eff_c.contains("Tab")
                    && !eff_c.contains("Scroll")))
                && st.graph.prop_val(child_obj, "NSSubviews").is_some();
        if let Some(p_node) = st.nodes.iter_mut().find(|n| n.id == parent_node_id) {
            p_node.children.push(nid);
        }
        let cur_node_idx = st.nodes.len();
        st.obj_to_node_idx.insert(child_obj, cur_node_idx);
        st.nodes.push(node);
        if is_box {
            if let Some(cv_obj) = st.graph.prop_val(child_obj, "NSContentView").and_then(IbVal::as_obj_ref) {
                st.obj_to_node_idx.insert(cv_obj, cur_node_idx);
                collect_ibtool_subviews(st, cv_obj, nid);
            } else if st.graph.prop_val(child_obj, "NSSubviews").is_some() {
                collect_ibtool_subviews(st, child_obj, nid);
            }
        } else if is_container_view {
            collect_ibtool_subviews(st, child_obj, nid);
        } else if eff_c.contains("ScrollView")
            && !matches!(st.nodes.last().map(|n| &n.kind), Some(SceneNodeKind::TableView { .. } | SceneNodeKind::CustomCanvasSlot { .. }))
        {
            if let Some(doc_obj) = crate::scene::nib_varint::resolve_scroll_doc_view(st.graph, child_obj) {
                st.obj_to_node_idx.insert(doc_obj, cur_node_idx);
                collect_ibtool_subviews(st, doc_obj, nid);
            } else if st.graph.prop_val(child_obj, "NSSubviews").is_some() {
                collect_ibtool_subviews(st, child_obj, nid);
            }
        } else if matches!(st.nodes.last().map(|n| &n.kind), Some(SceneNodeKind::TabView { .. })) {
            if let Some(tab_arr) = st.graph.prop_val(child_obj, "NSTabViewItems").and_then(IbVal::as_obj_ref) {
                for (tab_index, ti) in st.graph.resolve_array_refs(tab_arr).into_iter().enumerate() {
                    if let Some(tv_view) = st.graph.prop_val(ti, "NSView").and_then(IbVal::as_obj_ref) {
                        let tv_cls = st.graph.obj_class(tv_view);
                        let tv_orig =
                            st.graph.prop_val(tv_view, "NSOriginalClassName").and_then(|v| st.graph.resolve_str(v)).unwrap_or_default();
                        let start_idx = st.nodes.len();
                        if tv_cls.contains("ScrollView") || tv_orig.contains("ScrollView") || tv_cls == "NSClassSwapper" {
                            let sub_id = NodeId(st.next_id);
                            st.next_id += 1;
                            let mut sub_node = build_ibtool_node(st, tv_view, sub_id, nid);
                            sub_node.visibility = VisibilityRule::WhenTabSelected { tab_view: nid, tab_index };
                            if let Some(p_node) = st.nodes.iter_mut().find(|n| n.id == nid) {
                                p_node.children.push(sub_id);
                            }
                            let sub_idx = st.nodes.len();
                            st.obj_to_node_idx.insert(tv_view, sub_idx);
                            if let Some(doc_obj) = crate::scene::nib_varint::resolve_scroll_doc_view(st.graph, tv_view) {
                                st.obj_to_node_idx.insert(doc_obj, sub_idx);
                            }
                            st.nodes.push(sub_node);
                        } else {
                            let p_rect = st.nodes.iter().find(|n| n.id == nid).map(|n| n.nib_frame).unwrap_or_default();
                            st.obj_to_node_idx.insert(tv_view, cur_node_idx);
                            let before_tab = st.nodes.len();
                            collect_ibtool_subviews(st, tv_view, nid);
                            for sn in &mut st.nodes[before_tab..] {
                                if sn.parent == Some(nid) && sn.nib_frame[0] == 0.0 && sn.nib_frame[1] == 0.0 {
                                    sn.nib_frame[0] = p_rect[0];
                                    sn.nib_frame[1] = p_rect[1];
                                }
                            }
                        }
                        for sub_node in &mut st.nodes[start_idx..] {
                            if matches!(sub_node.visibility, VisibilityRule::Always) {
                                sub_node.visibility = VisibilityRule::WhenTabSelected { tab_view: nid, tab_index };
                            }
                        }
                    }
                }
            }
        }
    }
}
