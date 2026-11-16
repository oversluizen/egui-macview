//! Real Xcode 16.2 `ibtool --compile` Binary `NIBArchive` (`NSIBObjectData`) & Bundle Decoder (`REQ-013-05.2`, `REQ-013-05.5`).
//!
//! Decodes genuine Apple `ibtool --compile` `.nib` files 100% from native AppKit objects
//! (`NSWindowTemplate`, `NSView`, `NSTextField`+`NSTextFieldCell`+`NSNumberFormatter`, `NSButton`+`NSButtonCell`,
//! `NSPopUpButton`+`NSMenu`, `NSSegmentedControl`+`NSSegmentItem`, `NSSlider`+`NSSliderCell`, `NSStepper`,
//! `NSColorWell`, `NSComboBox`, `NSDatePicker`, `NSTabView`, `NSRuleEditor`, `NSLevelIndicator`,
//! `NSProgressIndicator`, `NSTokenField`, `NSTextView`, `NSBox`, `NSCustomView`, `NSClassSwapper`).

use crate::scene::constraints::{CompiledConstraintSchedule, ValueTransformer};
use crate::scene::ir::{
    ActionId, BindingId, CustomClassMapper, MacSceneIR, NodeId, SceneCompileError, SceneNode, SceneNodeKind, VisibilityRule,
};
use crate::scene::nib_ibtool_widgets::build_ibtool_node;
use crate::scene::nib_varint::read_nib_varint;
use crate::scene::xib_strings::flatten_controller_key_path;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) enum IbVal {
    Int(i64),
    Float(f64),
    Bytes(Vec<u8>),
    ObjRef(usize),
    Nil,
}

impl IbVal {
    pub(crate) fn as_u32(&self) -> Option<u32> {
        if let Self::Int(v) = self { Some(*v as u32) } else { None }
    }
    pub(crate) fn as_obj_ref(&self) -> Option<usize> {
        if let Self::ObjRef(idx) = self { Some(*idx) } else { None }
    }
    pub(crate) fn as_nsrect(&self) -> Option<[f32; 4]> {
        let Self::Bytes(b) = self else { return None };
        if b.len() != 17 || b[0] != 6 {
            return None;
        }
        Some([
            f32::from_le_bytes(b[1..5].try_into().ok()?),
            f32::from_le_bytes(b[5..9].try_into().ok()?),
            f32::from_le_bytes(b[9..13].try_into().ok()?),
            f32::from_le_bytes(b[13..17].try_into().ok()?),
        ])
    }
}

pub(crate) struct IbNibGraph {
    pub keys: Vec<String>,
    pub classes: Vec<String>,
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
        for _ in 0..cls_count {
            let len = read_nib_varint(bytes, &mut cur).ok_or_else(|| SceneCompileError("Bad cls varint".into()))? as usize;
            let extra = read_nib_varint(bytes, &mut cur).unwrap_or(0) as usize;
            cur += extra * 4;
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
        Ok(Self { keys, classes, values, objects })
    }

    pub fn obj_class(&self, obj_idx: usize) -> &str {
        self.objects.get(obj_idx).and_then(|&(c, _, _)| self.classes.get(c)).map(String::as_str).unwrap_or("")
    }

    pub fn obj_props(&self, obj_idx: usize) -> &[(usize, IbVal)] {
        let Some(&(_, vs, vl)) = self.objects.get(obj_idx) else { return &[] };
        self.values.get(vs..vs.saturating_add(vl)).unwrap_or(&[])
    }

    pub fn prop_val(&self, obj_idx: usize, key_name: &str) -> Option<&IbVal> {
        self.obj_props(obj_idx)
            .iter()
            .find(|(k_idx, _)| self.keys.get(*k_idx).map(String::as_str) == Some(key_name))
            .map(|(_, val)| val)
    }

    pub fn resolve_str(&self, val: &IbVal) -> Option<String> {
        match val {
            IbVal::Bytes(b) => std::str::from_utf8(b.strip_suffix(&[0]).unwrap_or(b.as_slice())).ok().map(str::to_string),
            IbVal::ObjRef(idx) => {
                if let Some(IbVal::Bytes(b)) = self.prop_val(*idx, "NS.bytes") {
                    return std::str::from_utf8(b.strip_suffix(&[0]).unwrap_or(b.as_slice())).ok().map(str::to_string);
                }
                ["NSResourceName", "NSContents", "NSImageName", "NSColorName", "NSName"]
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
                if let Some(r) = v.as_nsrect().or_else(|| self.resolve_str(v).and_then(|s| parse_brace_rect(&s))) {
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

pub(crate) fn parse_brace_rect(raw: &str) -> Option<[f32; 4]> {
    let cleaned: String = raw.chars().filter(|&c| c != '{' && c != '}').collect();
    let nums: Vec<f32> = cleaned.split(',').filter_map(|p| p.trim().parse::<f32>().ok()).collect();
    match nums.as_slice() {
        [x, y, w, h] => Some([*x, *y, *w, *h]),
        [w, h] => Some([0.0, 0.0, *w, *h]),
        _ => None,
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
            let k_refs = graph.prop_val(obj_i, "NSKeyPaths").and_then(IbVal::as_obj_ref).map(|a| graph.resolve_array_refs(a)).unwrap_or_default();
            let v_refs = graph.prop_val(obj_i, "NSValues").and_then(IbVal::as_obj_ref).map(|a| graph.resolve_array_refs(a)).unwrap_or_default();
            let entry = runtime_attrs.entry(target).or_default();
            for (kr, vr) in k_refs.into_iter().zip(v_refs) {
                if let (Some(k), Some(v)) = (graph.resolve_str(&IbVal::ObjRef(kr)), graph.resolve_str(&IbVal::ObjRef(vr))) {
                    entry.insert(k, v);
                }
            }
        }
    }

    let win_idx = (0..graph.objects.len()).find(|&i| graph.obj_class(i) == "NSWindowTemplate");
    let root_view_idx = win_idx
        .and_then(|w| graph.prop_val(w, "NSWindowView").and_then(IbVal::as_obj_ref))
        .or_else(|| (0..graph.objects.len()).find(|&i| graph.obj_class(i) == "NSView" && graph.prop_val(i, "NSSuperview").is_none()))
        .unwrap_or(0);
    let win_title = win_idx
        .and_then(|w| graph.prop_val(w, "NSWindowTitle").and_then(|v| graph.resolve_str(v)))
        .unwrap_or_else(|| "Window".to_string());
    let root_ib_id = ["NSReuseIdentifierKey", "NSUserInterfaceItemIdentifier", "NSIdentifier", "NSID"]
        .into_iter()
        .find_map(|k| graph.prop_val(root_view_idx, k).and_then(|v| graph.resolve_str(v)))
        .or_else(|| {
            graph.obj_props(root_view_idx).iter().find_map(|(_, v)| {
                graph.resolve_str(v).filter(|s| !s.is_empty() && !s.starts_with('{') && !s.starts_with("NS"))
            })
        })
        .unwrap_or_else(|| "root-view".to_string());
    let root_rect = graph.resolve_rect(root_view_idx).unwrap_or([0.0, 0.0, 480.0, 320.0]);
    let root_flipped = runtime_attrs.get(&root_view_idx).and_then(|m| m.get("ibFlipped")).is_some_and(|v| v == "YES");

    let root_node = SceneNode {
        id: NodeId(1),
        ib_object_id: root_ib_id,
        parent: None,
        children: vec![],
        kind: SceneNodeKind::Container { is_flipped: root_flipped },
        title: win_title,
        tooltip: String::new(),
        control_size: "regular".into(),
        nib_frame: root_rect,
        alignment_rect: root_rect,
        autoresizing_mask: 18,
        enabled: true,
        value_binding: None,
        binding_path: None,
        value_transformer: ValueTransformer::Identity,
        action: None,
        action_selector: None,
        icon_name: None,
        visibility: VisibilityRule::Always,
        labeled_by: None,
    };

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

    let root_w = st.nodes[0].nib_frame[2].max(320.0);
    let root_h = st.nodes[0].nib_frame[3].max(200.0);
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
        let nid = NodeId(st.next_id);
        st.next_id += 1;
        let node = build_ibtool_node(st, child_obj, nid, parent_node_id);
        let is_box = matches!(node.kind, SceneNodeKind::BoxContainer);
        if let Some(p_node) = st.nodes.iter_mut().find(|n| n.id == parent_node_id) {
            p_node.children.push(nid);
        }
        st.obj_to_node_idx.insert(child_obj, st.nodes.len());
        st.nodes.push(node);
        if is_box {
            if let Some(cv_obj) = st.graph.prop_val(child_obj, "NSContentView").and_then(IbVal::as_obj_ref) {
                collect_ibtool_subviews(st, cv_obj, nid);
            }
        }
    }
}

fn wire_ibtool_connectors(st: &mut IbDecodeState<'_>) {
    for obj_i in 0..st.graph.objects.len() {
        let cls = st.graph.obj_class(obj_i);
        if cls != "NSNibControlConnector" && cls != "NSNibBindingConnector" {
            continue;
        }
        let src_obj = st.graph.prop_val(obj_i, "NSSource").or_else(|| st.graph.prop_val(obj_i, "NSObject")).and_then(IbVal::as_obj_ref);
        let Some(&n_idx) = src_obj.and_then(|o| st.obj_to_node_idx.get(&o)) else { continue };
        if cls == "NSNibControlConnector" && st.nodes[n_idx].action_selector.is_none() {
            if let Some(sel) = st.graph.prop_val(obj_i, "NSLabel").and_then(|v| st.graph.resolve_str(v)) {
                st.nodes[n_idx].action = Some(ActionId(st.next_action));
                st.nodes[n_idx].action_selector = Some(sel);
                st.next_action += 1;
            }
        } else if cls == "NSNibBindingConnector" && st.nodes[n_idx].binding_path.is_none() {
            if let Some(raw_kp) = st.graph.prop_val(obj_i, "NSKeyPath").and_then(|v| st.graph.resolve_str(v)) {
                let norm = flatten_controller_key_path(&raw_kp);
                let bid = *st.binding_by_path.entry(norm.clone()).or_insert_with(|| {
                    let b = BindingId(st.next_binding);
                    st.next_binding += 1;
                    b
                });
                st.nodes[n_idx].value_binding = Some(bid);
                st.nodes[n_idx].binding_path = Some(norm);
            }
        }
    }
}

/// Resolves and loads a `.nib` file or `.nib` directory bundle (`keyedobjects-101300.nib` / `keyedobjects.nib` / `objects-11.0+.nib`).
pub fn load_nib_from_path(path: &Path, mapper: &CustomClassMapper) -> Result<MacSceneIR, SceneCompileError> {
    if path.is_dir() {
        for candidate in ["keyedobjects-101300.nib", "keyedobjects.nib", "objects-11.0+.nib", "objects.nib"] {
            let sub = path.join(candidate);
            if sub.is_file() {
                let bytes = std::fs::read(&sub).map_err(|e| SceneCompileError(format!("Failed reading {}: {e}", sub.display())))?;
                return crate::scene::NibArchiveFrontend::from_bytes(&bytes, mapper);
            }
        }
        return Err(SceneCompileError(format!("No inner .nib stream found in bundle directory {}", path.display())));
    }
    let bytes = std::fs::read(path).map_err(|e| SceneCompileError(format!("Failed reading {}: {e}", path.display())))?;
    crate::scene::NibArchiveFrontend::from_bytes(&bytes, mapper)
}
