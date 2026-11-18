//! Real Apple `NIBArchive` v1 Varint Decoder and `plist` `NSKeyedArchiver` (`bplist00`)
//! Object-Graph Decoder with `NSControl`+`NSCell` Fusion (`SPEC-013` §2.1..2.2, §3.1.2..3.1.5).

use crate::scene::constraints::{CompiledConstraintSchedule, ValueTransformer};
use crate::scene::ir::{
    ActionId, BindingId, CustomClassMapper, MacSceneIR, NodeId, SceneCompileError, SceneNode, SceneNodeKind, VisibilityRule,
};
pub use crate::scene::keyed_archiver::{build_real_nskeyedarchiver_bplist, decode_nskeyedarchiver_plist};
use crate::scene::nib_frontend::mk_node;
use crate::scene::xib_strings::flatten_controller_key_path;

/// Reads an Apple `NIBArchive` 7-bit varint where bit 7 (`0x80`) marks the LAST byte.
pub fn read_nib_varint(bytes: &[u8], cursor: &mut usize) -> Option<u32> {
    let mut result = 0u32;
    let mut shift = 0u32;
    for _ in 0..5 {
        let b = *bytes.get(*cursor)?;
        *cursor += 1;
        result |= ((b & 0x7F) as u32) << shift;
        if (b & 0x80) != 0 {
            return Some(result);
        }
        shift += 7;
    }
    None
}

/// Writes an Apple `NIBArchive` 7-bit varint where bit 7 (`0x80`) marks the LAST byte.
pub fn write_nib_varint(buf: &mut Vec<u8>, mut val: u32) {
    while val >= 0x80 {
        buf.push((val & 0x7F) as u8);
        val >>= 7;
    }
    buf.push((val as u8) | 0x80);
}

/// Decodes `NSCell` `cFlags` bits `18..=20` into control size string (`SPEC-013 §3.1.2`).
pub fn decode_cflags_control_size(cflags: u32) -> &'static str {
    match (cflags >> 18) & 0x07 {
        1 => "small",
        2 => "mini",
        3 => "large",
        _ => "regular",
    }
}

/// Decodes a genuine binary `NIBArchive\x01\x00\x00\x00` stream with varint tables & `NSCell` fusion.
pub fn decode_real_nib_archive_v1(bytes: &[u8], mapper: &CustomClassMapper) -> Result<MacSceneIR, SceneCompileError> {
    let graph = crate::scene::nib_ibtool::IbNibGraph::parse(bytes)?;
    if graph.classes.iter().any(|c| c == "NSIBObjectData") {
        return crate::scene::nib_ibtool::decode_ibtool_nib_archive(&graph, mapper);
    }
    let crate::scene::nib_ibtool::IbNibGraph { keys, classes, values, objects } = graph;

    let mut nodes: Vec<SceneNode> = Vec::new();
    let mut obj_to_node_idx: Vec<Option<usize>> = vec![None; objects.len()];
    let mut next_binding = 1u32;
    let mut next_action = 1u32;
    let mut binding_by_path: std::collections::BTreeMap<String, BindingId> = std::collections::BTreeMap::new();

    for (obj_i, &(cls_idx, v_start, v_len)) in objects.iter().enumerate() {
        let raw_cls = classes.get(cls_idx).map(String::as_str).unwrap_or("NSView");
        if raw_cls.ends_with("Cell") || raw_cls.contains("Connector") {
            continue;
        }
        let props = values.get(v_start..v_start.saturating_add(v_len)).unwrap_or(&[]);
        let mut frame = [0.0_f32, 0.0, 120.0, 22.0];
        let mut title = String::new();
        let mut tooltip = String::new();
        let mut ib_id_opt: Option<String> = None;
        let mut icon_opt: Option<String> = None;
        let mut custom_cls: Option<String> = None;
        let mut cell_ref: Option<usize> = None;
        let mut mask = 0u8;
        let mut is_unflipped = false;

        for (k_idx, val) in props {
            let k_name = keys.get(*k_idx).map(String::as_str).unwrap_or("");
            match k_name {
                "NSFrame" | "NSFrameSize" => {
                    if let Some(r) = val.as_nsrect() {
                        frame = r;
                    }
                }
                "NSTitle" | "NSContents" => {
                    if let Some(s) = val.as_str() {
                        title = s.to_string();
                    }
                }
                "NSToolTip" => {
                    if let Some(s) = val.as_str() {
                        tooltip = s.to_string();
                    }
                }
                "NSUserInterfaceItemIdentifier" => {
                    if let Some(s) = val.as_str() {
                        ib_id_opt = Some(s.to_string());
                    }
                }
                "NSImageName" => {
                    if let Some(s) = val.as_str() {
                        icon_opt = Some(s.to_string());
                    }
                }
                "NSClassName" => {
                    if let Some(s) = val.as_str() {
                        custom_cls = Some(s.to_string());
                    }
                }
                "NSCell" => cell_ref = val.as_obj_ref(),
                "NSvFlags" => {
                    if let Some(v) = val.as_u32() {
                        mask = (v & 0x3F) as u8;
                        is_unflipped = (v & 0x80) != 0;
                    }
                }
                _ => {}
            }
        }

        let mut cflags = 0u32;
        let mut btn_flags = 0u32;
        let mut key_equiv = String::new();
        if let Some(c_idx) = cell_ref {
            if let Some(&(_, cv_start, cv_len)) = objects.get(c_idx) {
                for (ck_idx, cval) in values.get(cv_start..cv_start.saturating_add(cv_len)).unwrap_or(&[]) {
                    match keys.get(*ck_idx).map(String::as_str).unwrap_or("") {
                        "NSContents" | "NSTitle" => {
                            if let Some(s) = cval.as_str() {
                                title = s.to_string();
                            }
                        }
                        "NSCellFlags" | "cFlags" => {
                            if let Some(v) = cval.as_u32() {
                                cflags = v;
                            }
                        }
                        "NSButtonFlags" => {
                            if let Some(v) = cval.as_u32() {
                                btn_flags = v;
                            }
                        }
                        "NSKeyEquivalent" => {
                            if let Some(s) = cval.as_str() {
                                key_equiv = s.to_string();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let effective_cls = custom_cls.as_deref().unwrap_or(raw_cls);
        let size_str = decode_cflags_control_size(cflags);
        let mut kind = classify_nib_control(effective_cls, &title, btn_flags, &key_equiv, mapper);
        if nodes.is_empty() && is_unflipped {
            kind = SceneNodeKind::Container { is_flipped: false };
        }
        let id = (nodes.len() + 1) as u32;
        let ib_id = ib_id_opt.unwrap_or_else(|| format!("nib-{obj_i}"));
        obj_to_node_idx[obj_i] = Some(nodes.len());
        nodes.push(mk_node(
            id,
            &ib_id,
            if nodes.is_empty() { None } else { Some(1) },
            vec![],
            kind,
            &title,
            &tooltip,
            size_str,
            frame,
            mask,
            None,
            None,
            icon_opt.as_deref(),
            VisibilityRule::Always,
        ));
    }

    for &(cls_idx, v_start, v_len) in &objects {
        let cls_name = classes.get(cls_idx).map(String::as_str).unwrap_or("");
        if !cls_name.contains("Connector") {
            continue;
        }
        let props = values.get(v_start..v_start.saturating_add(v_len)).unwrap_or(&[]);
        let mut src_obj: Option<usize> = None;
        let mut dst_obj: Option<usize> = None;
        let mut label_str = String::new();
        let mut key_path = String::new();
        let mut vtrans = ValueTransformer::Identity;
        for (k_idx, val) in props {
            match keys.get(*k_idx).map(String::as_str).unwrap_or("") {
                "NSSource" => src_obj = val.as_obj_ref(),
                "NSDestination" => dst_obj = val.as_obj_ref(),
                "NSLabel" => {
                    if let Some(s) = val.as_str() {
                        label_str = s.to_string();
                    }
                }
                "NSKeyPath" => {
                    if let Some(s) = val.as_str() {
                        key_path = flatten_controller_key_path(s);
                    }
                }
                "NSValueTransformerName" if val.as_str() == Some("NSNegateBoolean") => {
                    vtrans = ValueTransformer::NegateBoolean;
                }
                _ => {}
            }
        }
        if cls_name == "NSNibControlConnector" {
            if let Some(n_idx) = src_obj.and_then(|o| obj_to_node_idx.get(o).copied().flatten()) {
                nodes[n_idx].action = Some(ActionId(next_action));
                nodes[n_idx].action_selector = Some(label_str);
                next_action += 1;
            }
        } else if cls_name == "NSNibBindingConnector" {
            if let Some(n_idx) = src_obj.or(dst_obj).and_then(|o| obj_to_node_idx.get(o).copied().flatten()) {
                let bid = *binding_by_path.entry(key_path.clone()).or_insert_with(|| {
                    let b = BindingId(next_binding);
                    next_binding += 1;
                    b
                });
                nodes[n_idx].value_binding = Some(bid);
                nodes[n_idx].binding_path = Some(key_path);
                nodes[n_idx].value_transformer = vtrans;
            }
        }
    }

    if nodes.len() > 1 {
        let child_ids: Vec<NodeId> = nodes[1..].iter().map(|n| n.id).collect();
        nodes[0].children = child_ids;
    }
    let root_w = nodes.first().map(|n| n.nib_frame[2]).unwrap_or(480.0).max(320.0);
    let root_h = nodes.first().map(|n| n.nib_frame[3]).unwrap_or(320.0).max(200.0);
    Ok(MacSceneIR::from_nodes(NodeId(1), [root_w, root_h], nodes, CompiledConstraintSchedule::default()))
}

pub(crate) fn classify_nib_control(
    cls_name: &str,
    title: &str,
    btn_flags: u32,
    key_equiv: &str,
    mapper: &CustomClassMapper,
) -> SceneNodeKind {
    if mapper.is_stepping_number_field(cls_name) {
        SceneNodeKind::NumberField { step: 1.0, shift_step: 10.0, min: None, max: None }
    } else if mapper.is_origin_matrix(cls_name) {
        SceneNodeKind::OriginMatrix
    } else if cls_name.contains("Separator") {
        SceneNodeKind::Separator
    } else if cls_name.contains("Button") {
        if (btn_flags & 0x0080_0000) != 0 {
            SceneNodeKind::Checkbox
        } else {
            SceneNodeKind::PushButton {
                is_default: key_equiv == "\r" || title == "OK" || title == "Apply",
                is_cancel: key_equiv == "\u{1b}",
            }
        }
    } else if cls_name.contains("TextField") {
        if title.ends_with(':') {
            SceneNodeKind::Label
        } else {
            SceneNodeKind::TextField
        }
    } else if cls_name.contains("Switch") {
        SceneNodeKind::Switch
    } else {
        SceneNodeKind::Container { is_flipped: true }
    }
}

/// Builds a genuine binary `NIBArchive\x01\x00\x00\x00` stream with varints, `NSCell` fusion, `NSClassSwapper`, and connectors.
pub fn build_real_varint_nib_archive() -> Vec<u8> {
    let key_names = [
        "NSFrame",
        "NSTitle",
        "NSClassName",
        "NSCell",
        "NSvFlags",
        "NSCellFlags",
        "NSButtonFlags",
        "NSKeyEquivalent",
        "NSSource",
        "NSLabel",
        "NSKeyPath",
        "NSValueTransformerName",
    ];
    let mut keys_sec = Vec::new();
    for k in key_names {
        write_nib_varint(&mut keys_sec, k.len() as u32);
        keys_sec.extend_from_slice(k.as_bytes());
    }

    let cls_names = [
        "NSView",
        "NSTextField",
        "NSClassSwapper",
        "NSButton",
        "NSTextFieldCell",
        "NSButtonCell",
        "NSNibControlConnector",
        "NSNibBindingConnector",
    ];
    let mut cls_sec = Vec::new();
    for c in cls_names {
        write_nib_varint(&mut cls_sec, (c.len() + 1) as u32);
        write_nib_varint(&mut cls_sec, 0);
        cls_sec.extend_from_slice(c.as_bytes());
        cls_sec.push(0);
    }

    let mut vals_sec = Vec::new();
    let push_bytes_val = |vals: &mut Vec<u8>, k: u32, b: &[u8]| {
        write_nib_varint(vals, k);
        vals.push(8u8);
        write_nib_varint(vals, b.len() as u32);
        vals.extend_from_slice(b);
    };
    let push_int_val = |vals: &mut Vec<u8>, k: u32, v: i32| {
        write_nib_varint(vals, k);
        vals.push(2u8);
        vals.extend_from_slice(&v.to_le_bytes());
    };
    let push_ref_val = |vals: &mut Vec<u8>, k: u32, obj_idx: u32| {
        write_nib_varint(vals, k);
        vals.push(10u8);
        vals.extend_from_slice(&obj_idx.to_le_bytes());
    };
    let rect_bytes = |r: [f32; 4]| -> Vec<u8> {
        let mut b = vec![6u8];
        for f in r {
            b.extend_from_slice(&f.to_le_bytes());
        }
        b
    };

    push_bytes_val(&mut vals_sec, 0, &rect_bytes([0.0, 0.0, 480.0, 320.0]));
    push_bytes_val(&mut vals_sec, 0, &rect_bytes([24.0, 28.0, 90.0, 16.0]));
    push_bytes_val(&mut vals_sec, 1, b"Scale:\0");
    push_bytes_val(&mut vals_sec, 0, &rect_bytes([88.0, 21.0, 72.0, 22.0]));
    push_bytes_val(&mut vals_sec, 2, b"SteppingTextField\0");
    push_int_val(&mut vals_sec, 4, 2);
    push_ref_val(&mut vals_sec, 3, 6);
    push_bytes_val(&mut vals_sec, 0, &rect_bytes([180.0, 18.0, 34.0, 34.0]));
    push_bytes_val(&mut vals_sec, 2, b"OriginPointControl\0");
    push_bytes_val(&mut vals_sec, 0, &rect_bytes([20.0, 64.0, 140.0, 16.0]));
    push_ref_val(&mut vals_sec, 3, 7);
    push_bytes_val(&mut vals_sec, 0, &rect_bytes([-6.0, -6.0, 131.0, 40.0]));
    push_ref_val(&mut vals_sec, 3, 8);
    push_bytes_val(&mut vals_sec, 1, b"100\0");
    push_int_val(&mut vals_sec, 5, 1 << 18);
    push_bytes_val(&mut vals_sec, 1, b"Snap to Grid\0");
    push_int_val(&mut vals_sec, 6, 0x0080_0000);
    push_bytes_val(&mut vals_sec, 1, b"Apply\0");
    push_bytes_val(&mut vals_sec, 7, b"\r\0");
    push_ref_val(&mut vals_sec, 8, 5);
    push_bytes_val(&mut vals_sec, 9, b"applyTransform:\0");
    push_ref_val(&mut vals_sec, 8, 2);
    push_bytes_val(&mut vals_sec, 10, b"arrangedObjects.scalePercent\0");
    push_bytes_val(&mut vals_sec, 11, b"NSNegateBoolean\0");
    push_ref_val(&mut vals_sec, 8, 3);
    push_bytes_val(&mut vals_sec, 10, b"values.originAnchor\0");
    push_ref_val(&mut vals_sec, 8, 4);
    push_bytes_val(&mut vals_sec, 10, b"values.snapToPixelGrid\0");

    let objs: [(u32, u32, u32); 13] = [
        (0, 0, 1),
        (1, 1, 2),
        (2, 3, 4),
        (2, 7, 2),
        (3, 9, 2),
        (3, 11, 2),
        (4, 13, 2),
        (5, 15, 2),
        (5, 17, 2),
        (6, 19, 2),
        (7, 21, 3),
        (7, 24, 2),
        (7, 26, 2),
    ];
    let mut objs_sec = Vec::new();
    for (c, vs, vl) in objs {
        write_nib_varint(&mut objs_sec, c);
        write_nib_varint(&mut objs_sec, vs);
        write_nib_varint(&mut objs_sec, vl);
    }

    let obj_off = 52u32;
    let key_off = obj_off + objs_sec.len() as u32;
    let val_off = key_off + keys_sec.len() as u32;
    let cls_off = val_off + vals_sec.len() as u32;

    let mut out = Vec::from(&b"NIBArchive\x01\x00\x00\x00"[..]);
    for u in [0u32, objs.len() as u32, obj_off, key_names.len() as u32, key_off, 28u32, val_off, cls_names.len() as u32, cls_off] {
        out.extend_from_slice(&u.to_le_bytes());
    }
    out.resize(52, 0);
    out.extend_from_slice(&objs_sec);
    out.extend_from_slice(&keys_sec);
    out.extend_from_slice(&vals_sec);
    out.extend_from_slice(&cls_sec);
    out
}
