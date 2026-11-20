//! Real Xcode 16.2 `ibtool --compile` Binary `NIBArchive` (`NSIBObjectData`) & Bundle Decoder (`REQ-013-05.2`, `REQ-013-05.5`).
//!
//! Decodes genuine Apple `ibtool --compile` `.nib` files 100% from native AppKit objects
//! (`NSWindowTemplate`, `NSView`, `NSTextField`+`NSTextFieldCell`+`NSNumberFormatter`, `NSButton`+`NSButtonCell`,
//! `NSPopUpButton`+`NSMenu`, `NSSegmentedControl`+`NSSegmentItem`, `NSSlider`+`NSSliderCell`, `NSStepper`,
//! `NSColorWell`, `NSComboBox`, `NSDatePicker`, `NSTabView`, `NSRuleEditor`, `NSLevelIndicator`,
//! `NSProgressIndicator`, `NSTokenField`, `NSTextView`, `NSBox`, `NSCustomView`, `NSClassSwapper`).

use crate::scene::constraints::CompiledConstraintSchedule;
use crate::scene::ir::{
    BindingId, CustomClassMapper, MacSceneIR, NodeId, SceneCompileError, SceneNode, SceneNodeKind, VisibilityRule,
};
use crate::scene::nib_ibtool_widgets::{build_ibtool_node, wire_ibtool_connectors};
use crate::scene::nib_varint::read_nib_varint;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) enum IbVal {
    Int(i64),
    Float(f64),
    Bytes(Vec<u8>),
    ObjRef(usize),
    Nil,
}

impl IbVal {
    pub(crate) fn as_str(&self) -> Option<&str> {
        let Self::Bytes(b) = self else { return None };
        std::str::from_utf8(b.strip_suffix(&[0]).unwrap_or(b.as_slice())).ok()
    }
    pub(crate) fn as_u32(&self) -> Option<u32> {
        if let Self::Int(v) = self {
            Some(*v as u32)
        } else {
            None
        }
    }
    pub(crate) fn as_obj_ref(&self) -> Option<usize> {
        if let Self::ObjRef(idx) = self {
            Some(*idx)
        } else {
            None
        }
    }
    pub(crate) fn as_nsrect(&self) -> Option<[f32; 4]> {
        let Self::Bytes(b) = self else { return None };
        if b.first().copied() != Some(6) {
            return None;
        }
        let f32_at = |o: usize| b.get(o..o + 4).and_then(|s| s.try_into().ok()).map(f32::from_le_bytes);
        let f64_at = |o: usize| b.get(o..o + 8).and_then(|s| s.try_into().ok()).map(|a| f64::from_le_bytes(a) as f32);
        match b.len() {
            17 => {
                let (x, y, w, h) = (f32_at(1)?, f32_at(5)?, f32_at(9)?, f32_at(13)?);
                let (dw, dh) = (f64_at(1)?, f64_at(9)?);
                let is_f64_sz = (1.0..=10000.0).contains(&dw) && (1.0..=10000.0).contains(&dh) && (w == 0.0 || h == 0.0 || (x.abs() < 1e-10 && y != 0.0));
                Some(if is_f64_sz { [0.0, 0.0, dw, dh] } else { [x, y, w, h] })
            }
            33 => Some([f64_at(1)?, f64_at(9)?, f64_at(17)?, f64_at(25)?]),
            9 => Some([0.0, 0.0, f32_at(1)?, f32_at(5)?]),
            _ => None,
        }
    }
}

pub(crate) struct IbNibGraph {
    pub keys: Vec<String>,
    pub classes: Vec<String>,
    pub class_super_ids: Vec<Vec<u32>>,
    pub values: Vec<(usize, IbVal)>,
    pub objects: Vec<(usize, usize, usize)>,
}

impl IbNibGraph {
    pub fn parse(bytes: &[u8]) -> Result<Self, SceneCompileError> {
        if bytes.len() < 52 {
            return Err(SceneCompileError("NIBArchive shorter than 52 bytes".into()));
        }
        let read_u32 = |off: usize| -> u32 { u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap_or([0; 4])) };
        let (obj_count, obj_offset) = (read_u32(18) as usize, read_u32(22) as usize);
        let (key_count, key_offset) = (read_u32(26) as usize, read_u32(30) as usize);
        let (val_count, val_offset) = (read_u32(34) as usize, read_u32(38) as usize);
        let (cls_count, cls_offset) = (read_u32(42) as usize, read_u32(46) as usize);
        if obj_count == 0 || key_offset >= bytes.len() || val_offset >= bytes.len() || cls_offset >= bytes.len() {
            return Err(SceneCompileError("Invalid NIBArchive section offsets".into()));
        }

        let mut cur = key_offset;
        let mut keys = Vec::with_capacity(key_count);
        for _ in 0..key_count {
            let len = read_nib_varint(bytes, &mut cur).ok_or_else(|| SceneCompileError("Bad key varint".into()))? as usize;
            let slice = bytes.get(cur..cur + len).ok_or_else(|| SceneCompileError("Key OOB".into()))?;
            keys.push(String::from_utf8_lossy(slice).to_string());
            cur += len;
        }

        cur = cls_offset;
        let mut classes = Vec::with_capacity(cls_count);
        let mut class_super_ids = Vec::with_capacity(cls_count);
        for _ in 0..cls_count {
            let len = read_nib_varint(bytes, &mut cur).ok_or_else(|| SceneCompileError("Bad cls varint".into()))? as usize;
            let extra = read_nib_varint(bytes, &mut cur).unwrap_or(0) as usize;
            let mut supers = Vec::with_capacity(extra);
            for _ in 0..extra {
                let sid = bytes.get(cur..cur + 4).and_then(|s| s.try_into().ok()).map(u32::from_le_bytes).unwrap_or(0);
                supers.push(sid);
                cur += 4;
            }
            class_super_ids.push(supers);
            let raw = bytes.get(cur..cur + len).ok_or_else(|| SceneCompileError("Class OOB".into()))?;
            classes.push(String::from_utf8_lossy(raw.strip_suffix(&[0]).unwrap_or(raw)).to_string());
            cur += len;
        }

        cur = val_offset;
        let mut values = Vec::with_capacity(val_count);
        for _ in 0..val_count {
            let key_idx = read_nib_varint(bytes, &mut cur).ok_or_else(|| SceneCompileError("Bad val key".into()))? as usize;
            let vtype = *bytes.get(cur).ok_or_else(|| SceneCompileError("Bad val type".into()))?;
            cur += 1;
            let val = decode_nib_value_entry(bytes, &mut cur, vtype)?;
            values.push((key_idx, val));
        }

        cur = obj_offset;
        let mut objects = Vec::with_capacity(obj_count);
        for _ in 0..obj_count {
            let cls_idx = read_nib_varint(bytes, &mut cur).ok_or_else(|| SceneCompileError("Bad obj cls".into()))? as usize;
            let val_start = read_nib_varint(bytes, &mut cur).ok_or_else(|| SceneCompileError("Bad obj val_start".into()))? as usize;
            let val_len = read_nib_varint(bytes, &mut cur).ok_or_else(|| SceneCompileError("Bad obj val_len".into()))? as usize;
            objects.push((cls_idx, val_start, val_len));
        }
        Ok(Self { keys, classes, class_super_ids, values, objects })
    }

    pub fn obj_class(&self, obj_idx: usize) -> &str {
        self.objects.get(obj_idx).and_then(|&(c, _, _)| self.classes.get(c)).map(String::as_str).unwrap_or("")
    }

    pub fn obj_inherits(&self, obj_idx: usize, needle: &str) -> bool {
        let Some(&(cls_i, _, _)) = self.objects.get(obj_idx) else { return false };
        if self.classes.get(cls_i).is_some_and(|c| c.contains(needle)) {
            return true;
        }
        self.class_super_ids
            .get(cls_i)
            .is_some_and(|supers| supers.iter().any(|&sid| self.classes.get(sid as usize).is_some_and(|c| c.contains(needle))))
    }

    pub fn obj_props(&self, obj_idx: usize) -> &[(usize, IbVal)] {
        let Some(&(_, vs, vl)) = self.objects.get(obj_idx) else { return &[] };
        self.values.get(vs..vs.saturating_add(vl)).unwrap_or(&[])
    }

    pub fn prop_val(&self, obj_idx: usize, key_name: &str) -> Option<&IbVal> {
        self.obj_props(obj_idx).iter().find(|(k_idx, _)| self.keys.get(*k_idx).map(String::as_str) == Some(key_name)).map(|(_, val)| val)
    }

    pub fn resolve_str(&self, val: &IbVal) -> Option<String> {
        match val {
            IbVal::Bytes(b) => std::str::from_utf8(b.strip_suffix(&[0]).unwrap_or(b.as_slice())).ok().map(str::to_string),
            IbVal::ObjRef(idx) => {
                if let Some(IbVal::Bytes(b)) = self.prop_val(*idx, "NS.bytes") {
                    return std::str::from_utf8(b.strip_suffix(&[0]).unwrap_or(b.as_slice())).ok().map(str::to_string);
                }
                [
                    "NS.string",
                    "NSString",
                    "NSResourceName",
                    "NSContents",
                    "NSTitle",
                    "NSImageName",
                    "NSColorName",
                    "NSName",
                ]
                .into_iter()
                .find_map(|k| self.prop_val(*idx, k).and_then(|v| self.resolve_str(v)))
            }
            _ => None,
        }
    }

    pub fn resolve_f64(&self, val: &IbVal) -> Option<f64> {
        match val {
            IbVal::Float(f) => Some(*f),
            IbVal::Int(i) => Some(*i as f64),
            IbVal::ObjRef(idx) => ["NS.dblval", "NS.intval", "NS.boolval"]
                .into_iter()
                .find_map(|k| self.prop_val(*idx, k).and_then(|v| self.resolve_f64(v)))
                .or_else(|| self.prop_val(*idx, "NS.bytes").and_then(|v| self.resolve_str(v)).and_then(|s| s.trim().parse::<f64>().ok())),
            _ => None,
        }
    }

    pub fn resolve_array_refs(&self, arr_idx: usize) -> Vec<usize> {
        self.obj_props(arr_idx)
            .iter()
            .filter(|(k_idx, _)| matches!(self.keys.get(*k_idx).map(String::as_str), Some("UINibEncoderEmptyKey" | "NS.objects")))
            .filter_map(|(_, val)| val.as_obj_ref())
            .collect()
    }

    pub fn resolve_dict_f64(&self, dict_idx: usize, target_key: &str) -> Option<f64> {
        for pair in self.resolve_array_refs(dict_idx).chunks(2) {
            if pair.len() == 2 {
                if self.resolve_str(&IbVal::ObjRef(pair[0])).as_deref() == Some(target_key) {
                    return self.resolve_f64(&IbVal::ObjRef(pair[1]));
                }
                if self.resolve_str(&IbVal::ObjRef(pair[1])).as_deref() == Some(target_key) {
                    return self.resolve_f64(&IbVal::ObjRef(pair[0]));
                }
            }
        }
        None
    }

    pub fn resolve_rect(&self, obj_idx: usize) -> Option<[f32; 4]> {
        for (key, size_only) in [("NSFrame", false), ("NSFrameSize", true), ("NSWindowRect", false)] {
            if let Some(v) = self.prop_val(obj_idx, key) {
                if let Some(r) = v.as_nsrect().or_else(|| self.resolve_str(v).and_then(|s| crate::scene::parse_brace_rect(&s))) {
                    return Some(if size_only { [0.0, 0.0, r[2], r[3]] } else { r });
                }
            }
        }
        None
    }
}

fn decode_nib_value_entry(bytes: &[u8], cur: &mut usize, vtype: u8) -> Result<IbVal, SceneCompileError> {
    match vtype {
        0 => {
            let v = *bytes.get(*cur).unwrap_or(&0) as i8 as i64;
            *cur += 1;
            Ok(IbVal::Int(v))
        }
        1 => {
            let v = i16::from_le_bytes(bytes.get(*cur..*cur + 2).and_then(|s| s.try_into().ok()).unwrap_or([0; 2])) as i64;
            *cur += 2;
            Ok(IbVal::Int(v))
        }
        2 => {
            let v = i32::from_le_bytes(bytes.get(*cur..*cur + 4).and_then(|s| s.try_into().ok()).unwrap_or([0; 4])) as i64;
            *cur += 4;
            Ok(IbVal::Int(v))
        }
        3 => {
            let v = i64::from_le_bytes(bytes.get(*cur..*cur + 8).and_then(|s| s.try_into().ok()).unwrap_or([0; 8]));
            *cur += 8;
            Ok(IbVal::Int(v))
        }
        4 => Ok(IbVal::Int(0)),
        5 => Ok(IbVal::Int(1)),
        6 => {
            let v = f32::from_le_bytes(bytes.get(*cur..*cur + 4).and_then(|s| s.try_into().ok()).unwrap_or([0; 4])) as f64;
            *cur += 4;
            Ok(IbVal::Float(v))
        }
        7 => {
            let v = f64::from_le_bytes(bytes.get(*cur..*cur + 8).and_then(|s| s.try_into().ok()).unwrap_or([0; 8]));
            *cur += 8;
            Ok(IbVal::Float(v))
        }
        8 => {
            let len = read_nib_varint(bytes, cur).unwrap_or(0) as usize;
            let data = bytes.get(*cur..*cur + len).unwrap_or(&[]).to_vec();
            *cur += len;
            Ok(IbVal::Bytes(data))
        }
        9 => Ok(IbVal::Nil),
        10 => {
            let idx = u32::from_le_bytes(bytes.get(*cur..*cur + 4).and_then(|s| s.try_into().ok()).unwrap_or([0; 4])) as usize;
            *cur += 4;
            Ok(IbVal::ObjRef(idx))
        }
        _ => Err(SceneCompileError(format!("Unknown NIB value type {vtype}"))),
    }
}

pub(crate) struct IbDecodeState<'a> {
    pub graph: &'a IbNibGraph,
    pub runtime_attrs: BTreeMap<usize, BTreeMap<String, String>>,
    pub mapper: &'a CustomClassMapper,
    pub nodes: Vec<SceneNode>,
    pub obj_to_node_idx: BTreeMap<usize, usize>,
    pub next_id: u32,
    pub next_binding: u32,
    pub next_action: u32,
    pub binding_by_path: BTreeMap<String, BindingId>,
}

/// Decodes a genuine Xcode 16.2 `ibtool --compile` `NIBArchive` containing `NSIBObjectData`.
pub(crate) fn decode_ibtool_nib_archive(graph: &IbNibGraph, mapper: &CustomClassMapper) -> Result<MacSceneIR, SceneCompileError> {
    let mut runtime_attrs: BTreeMap<usize, BTreeMap<String, String>> = BTreeMap::new();
    for obj_i in 0..graph.objects.len() {
        if graph.obj_class(obj_i) == "NSIBUserDefinedRuntimeAttributesConnector" {
            let Some(target) = graph.prop_val(obj_i, "NSObject").and_then(IbVal::as_obj_ref) else { continue };
            let k_refs =
                graph.prop_val(obj_i, "NSKeyPaths").and_then(IbVal::as_obj_ref).map(|a| graph.resolve_array_refs(a)).unwrap_or_default();
            let v_refs =
                graph.prop_val(obj_i, "NSValues").and_then(IbVal::as_obj_ref).map(|a| graph.resolve_array_refs(a)).unwrap_or_default();
            let entry = runtime_attrs.entry(target).or_default();
            for (kr, vr) in k_refs.into_iter().zip(v_refs) {
                if let (Some(k), Some(v)) = (graph.resolve_str(&IbVal::ObjRef(kr)), graph.resolve_str(&IbVal::ObjRef(vr))) {
                    entry.insert(k, v);
                }
            }
        }
    }

    let win_idx = (0..graph.objects.len()).find(|&i| graph.obj_class(i) == "NSWindowTemplate");
    let count_descendants = |start_idx: usize| -> usize {
        let mut total = 0usize;
        let mut stack = vec![start_idx];
        while let Some(curr) = stack.pop() {
            if let Some(arr) = graph.prop_val(curr, "NSSubviews").and_then(IbVal::as_obj_ref) {
                let subs = graph.resolve_array_refs(arr);
                total += subs.len();
                stack.extend(subs);
            }
            if let Some(doc) = graph.prop_val(curr, "NSDocView").and_then(IbVal::as_obj_ref) {
                stack.push(doc);
            }
            if let Some(cv) = graph.prop_val(curr, "NSContentView").and_then(IbVal::as_obj_ref) {
                stack.push(cv);
            }
        }
        total
    };
    let best_free = (0..graph.objects.len())
        .filter(|&i| graph.prop_val(i, "NSSuperview").is_none() && graph.prop_val(i, "NSSubviews").is_some())
        .max_by_key(|&i| count_descendants(i));
    let win_view = win_idx.and_then(|w| graph.prop_val(w, "NSWindowView").and_then(IbVal::as_obj_ref));
    let root_view_idx = if best_free.map(count_descendants).unwrap_or(0) > win_view.map(count_descendants).unwrap_or(0) {
        best_free.or(win_view)
    } else {
        win_view.or(best_free)
    }
    .or_else(|| (0..graph.objects.len()).find(|&i| graph.obj_class(i).ends_with("View") && graph.prop_val(i, "NSSuperview").is_none()))
    .unwrap_or(0);
    let win_title =
        win_idx.and_then(|w| graph.prop_val(w, "NSWindowTitle").and_then(|v| graph.resolve_str(v))).unwrap_or_else(|| "Window".to_string());
    let root_ib_id = ["NSReuseIdentifierKey", "NSUserInterfaceItemIdentifier", "NSIdentifier", "NSID"]
        .into_iter()
        .find_map(|k| graph.prop_val(root_view_idx, k).and_then(|v| graph.resolve_str(v)))
        .or_else(|| {
            graph
                .obj_props(root_view_idx)
                .iter()
                .find_map(|(_, v)| graph.resolve_str(v).filter(|s| !s.is_empty() && !s.starts_with('{') && !s.starts_with("NS")))
        })
        .unwrap_or_else(|| "root-view".to_string());
    let root_rect = graph.resolve_rect(root_view_idx).unwrap_or([0.0, 0.0, 480.0, 320.0]);
    let root_flipped = runtime_attrs.get(&root_view_idx).and_then(|m| m.get("ibFlipped")).is_some_and(|v| v == "YES");

    let root_node = crate::scene::nib_frontend::mk_node(
        1,
        &root_ib_id,
        None,
        vec![],
        SceneNodeKind::Container { is_flipped: root_flipped },
        &win_title,
        "",
        "regular",
        root_rect,
        18,
        None,
        None,
        None,
        VisibilityRule::Always,
    );

    let mut st = IbDecodeState {
        graph,
        runtime_attrs,
        mapper,
        nodes: vec![root_node],
        obj_to_node_idx: BTreeMap::from([(root_view_idx, 0)]),
        next_id: 2,
        next_binding: 1,
        next_action: 1,
        binding_by_path: BTreeMap::new(),
    };
    collect_ibtool_subviews(&mut st, root_view_idx, NodeId(1));
    wire_ibtool_connectors(&mut st);

    let root_w = st.nodes[0].nib_frame[2].max(1.0);
    let root_h = st.nodes[0].nib_frame[3].max(1.0);
    Ok(MacSceneIR::from_nodes(NodeId(1), [root_w, root_h], st.nodes, CompiledConstraintSchedule::default()))
}

fn collect_ibtool_subviews(st: &mut IbDecodeState<'_>, parent_view_obj: usize, parent_node_id: NodeId) {
    let subviews = st
        .graph
        .prop_val(parent_view_obj, "NSSubviews")
        .and_then(IbVal::as_obj_ref)
        .map(|arr| st.graph.resolve_array_refs(arr))
        .unwrap_or_default();

    for child_obj in subviews {
        if st.graph.obj_class(child_obj) == "NSMatrix" {
            let m_frame = st.graph.resolve_rect(child_obj).unwrap_or([0.0, 0.0, 170.0, 52.0]);
            let cells = st
                .graph
                .prop_val(child_obj, "NSCells")
                .and_then(IbVal::as_obj_ref)
                .map(|a| st.graph.resolve_array_refs(a))
                .unwrap_or_default();
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
                let r_node = crate::scene::nib_frontend::mk_node(
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
            continue;
        }
        let nid = NodeId(st.next_id);
        st.next_id += 1;
        let node = build_ibtool_node(st, child_obj, nid, parent_node_id);
        let is_box = matches!(node.kind, SceneNodeKind::BoxContainer);
        let raw_c = st.graph.obj_class(child_obj);
        let custom_c = st.graph.prop_val(child_obj, "NSClassName").and_then(|v| st.graph.resolve_str(v)).unwrap_or_default();
        let orig_c = st.graph.prop_val(child_obj, "NSOriginalClassName").and_then(|v| st.graph.resolve_str(v)).unwrap_or_default();
        let eff_c = if !custom_c.is_empty() { custom_c.as_str() } else if !orig_c.is_empty() { orig_c.as_str() } else { raw_c };
        let is_container_view = (matches!(
            eff_c,
            "NSView" | "NSCustomView" | "NSSplitView" | "NSStackView" | "NSStackViewContainer" | "NSClipView"
        ) || (eff_c.ends_with("View") && !eff_c.contains("Table") && !eff_c.contains("Outline") && !eff_c.contains("Text") && !eff_c.contains("Image") && !eff_c.contains("Tab") && !eff_c.contains("Scroll")))
            && st.graph.prop_val(child_obj, "NSSubviews").is_some();
        if let Some(p_node) = st.nodes.iter_mut().find(|n| n.id == parent_node_id) {
            p_node.children.push(nid);
        }
        st.obj_to_node_idx.insert(child_obj, st.nodes.len());
        st.nodes.push(node);
        if is_box {
            if let Some(cv_obj) = st.graph.prop_val(child_obj, "NSContentView").and_then(IbVal::as_obj_ref) {
                collect_ibtool_subviews(st, cv_obj, nid);
            } else if st.graph.prop_val(child_obj, "NSSubviews").is_some() {
                collect_ibtool_subviews(st, child_obj, nid);
            }
        } else if is_container_view {
            collect_ibtool_subviews(st, child_obj, nid);
        } else if eff_c.contains("ScrollView") && !matches!(st.nodes.last().map(|n| &n.kind), Some(SceneNodeKind::TableView { .. })) {
            let doc_opt = st
                .graph
                .prop_val(child_obj, "NSDocView")
                .and_then(IbVal::as_obj_ref)
                .or_else(|| {
                    st.graph
                        .prop_val(child_obj, "NSContentView")
                        .and_then(IbVal::as_obj_ref)
                        .and_then(|cv| st.graph.prop_val(cv, "NSDocView").and_then(IbVal::as_obj_ref))
                })
                .or_else(|| {
                    st.graph
                        .prop_val(child_obj, "NSSubviews")
                        .and_then(IbVal::as_obj_ref)
                        .into_iter()
                        .flat_map(|a| st.graph.resolve_array_refs(a))
                        .find_map(|sub| {
                            st.graph.prop_val(sub, "NSDocView").and_then(IbVal::as_obj_ref).or_else(|| {
                                st.graph
                                    .prop_val(sub, "NSSubviews")
                                    .and_then(IbVal::as_obj_ref)
                                    .and_then(|ca| st.graph.resolve_array_refs(ca).into_iter().next())
                            })
                        })
                });
            if let Some(doc_obj) = doc_opt {
                collect_ibtool_subviews(st, doc_obj, nid);
            } else if st.graph.prop_val(child_obj, "NSSubviews").is_some() {
                collect_ibtool_subviews(st, child_obj, nid);
            }
        } else if matches!(st.nodes.last().map(|n| &n.kind), Some(SceneNodeKind::TabView { .. })) {
            if let Some(tab_arr) = st.graph.prop_val(child_obj, "NSTabViewItems").and_then(IbVal::as_obj_ref) {
                for ti in st.graph.resolve_array_refs(tab_arr) {
                    if let Some(tv_view) = st.graph.prop_val(ti, "NSView").and_then(IbVal::as_obj_ref) {
                        collect_ibtool_subviews(st, tv_view, nid);
                    }
                }
            }
        }
    }
}
