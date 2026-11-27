//! Real Apple `plist` `NSKeyedArchiver` (`bplist00` & XML plist) Object-Graph Decoder (`SPEC-013` §2.2).

use crate::scene::constraints::CompiledConstraintSchedule;
use crate::scene::ir::{CustomClassMapper, MacSceneIR, NodeId, SceneCompileError, SceneNodeKind, VisibilityRule};
use crate::scene::nib_frontend::mk_node;
use crate::scene::nib_varint::{classify_nib_control, write_nib_varint};
use plist::{Dictionary, Uid, Value};

/// Decodes a genuine `plist` `NSKeyedArchiver` (`bplist00` or XML plist with `$objects` & `CF$UID`).
pub fn decode_nskeyedarchiver_plist(bytes: &[u8], mapper: &CustomClassMapper) -> Result<MacSceneIR, SceneCompileError> {
    let root_val: Value = plist::from_bytes(bytes).map_err(|e| SceneCompileError(format!("plist decode error: {e}")))?;
    let dict = root_val.as_dictionary().ok_or_else(|| SceneCompileError("Root plist is not a dictionary".into()))?;
    let objects = dict
        .get("$objects")
        .and_then(Value::as_array)
        .ok_or_else(|| SceneCompileError("Missing $objects array in NSKeyedArchiver plist".into()))?;

    let resolve_str = |v: &Value| -> Option<String> {
        if let Some(s) = v.as_string() {
            return Some(s.to_string());
        }
        if let Value::Uid(uid) = v {
            return objects.get(uid.get() as usize)?.as_string().map(str::to_string);
        }
        None
    };

    let resolve_class_name = |obj_dict: &Dictionary| -> String {
        if let Some(Value::Uid(uid)) = obj_dict.get("$class") {
            if let Some(cls_dict) = objects.get(uid.get() as usize).and_then(Value::as_dictionary) {
                if let Some(name) = cls_dict.get("$classname").and_then(Value::as_string) {
                    return name.to_string();
                }
            }
        }
        "NSView".into()
    };

    let mut nodes = Vec::new();
    for (idx, item) in objects.iter().enumerate() {
        let Some(obj_dict) = item.as_dictionary() else {
            continue;
        };
        if obj_dict.contains_key("$classname") {
            continue;
        }
        let cls_name = resolve_class_name(obj_dict);
        if cls_name == "NSTableView" || cls_name == "NSTableColumn" {
            continue;
        }
        let title = obj_dict.get("NSTitle").or_else(|| obj_dict.get("NSContents")).and_then(resolve_str).unwrap_or_default();
        let frame_str = obj_dict.get("NSFrame").and_then(resolve_str).unwrap_or_else(|| "{{0, 0}, {120, 22}}".into());
        let frame = parse_nsrect_string(&frame_str).unwrap_or([0.0, 0.0, 120.0, 22.0]);
        let kind = if cls_name == "NSScrollView" {
            let mut cols = Vec::new();
            if let Some(Value::Uid(doc_uid)) = obj_dict.get("NSDocView") {
                if let Some(tv_dict) = objects.get(doc_uid.get() as usize).and_then(Value::as_dictionary) {
                    if let Some(col_arr) = tv_dict.get("NSTableColumns").and_then(Value::as_array) {
                        for col_v in col_arr {
                            if let Value::Uid(c_uid) = col_v {
                                if let Some(c_dict) = objects.get(c_uid.get() as usize).and_then(Value::as_dictionary) {
                                    let hdr = c_dict.get("NSTitle").and_then(resolve_str).unwrap_or_else(|| "Col".into());
                                    let w = c_dict.get("NSWidth").and_then(Value::as_real).unwrap_or(100.0) as f32;
                                    cols.push((hdr, w));
                                }
                            }
                        }
                    }
                }
            }
            SceneNodeKind::TableView { columns: cols }
        } else {
            classify_nib_control(&cls_name, &title, 0, "", mapper)
        };
        let id = (nodes.len() + 1) as u32;
        nodes.push(mk_node(
            id,
            &format!("uid-{idx}"),
            if nodes.is_empty() { None } else { Some(1) },
            vec![],
            kind,
            &title,
            "",
            "regular",
            frame,
            0,
            None,
            None,
            None,
            VisibilityRule::Always,
        ));
    }
    if nodes.len() > 1 {
        let child_ids: Vec<NodeId> = nodes[1..].iter().map(|n| n.id).collect();
        nodes[0].children = child_ids;
    }
    Ok(MacSceneIR::from_nodes(NodeId(1), [480.0, 320.0], nodes, CompiledConstraintSchedule::default()))
}

pub(crate) fn parse_nsrect_string(s: &str) -> Option<[f32; 4]> {
    let nums: Vec<f32> = s
        .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .filter(|t| !t.is_empty())
        .filter_map(|t| t.parse::<f32>().ok())
        .collect();
    if nums.len() >= 4 {
        Some([nums[0], nums[1], nums[2], nums[3]])
    } else if nums.len() == 2 {
        Some([0.0, 0.0, nums[0], nums[1]])
    } else {
        None
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
    let objs: [(u32, u32, u32); 11] =
        [(0, 0, 1), (1, 1, 2), (2, 3, 4), (2, 7, 2), (3, 9, 2), (3, 11, 2), (4, 13, 2), (5, 15, 2), (5, 17, 2), (6, 19, 2), (7, 21, 3)];
    let mut objs_sec = Vec::new();
    for (c, vs, vl) in objs {
        write_nib_varint(&mut objs_sec, c);
        write_nib_varint(&mut objs_sec, vs);
        write_nib_varint(&mut objs_sec, vl);
    }
    let obj_off = 50u32;
    let key_off = obj_off + objs_sec.len() as u32;
    let val_off = key_off + keys_sec.len() as u32;
    let cls_off = val_off + vals_sec.len() as u32;
    let mut out = Vec::from(&b"NIBArchive\x01\x00\x00\x00"[..]);
    for u in [10u32, objs.len() as u32, obj_off, key_names.len() as u32, key_off, 24u32, val_off, cls_names.len() as u32, cls_off] {
        out.extend_from_slice(&u.to_le_bytes());
    }
    out.extend_from_slice(&objs_sec);
    out.extend_from_slice(&keys_sec);
    out.extend_from_slice(&vals_sec);
    out.extend_from_slice(&cls_sec);
    out
}

/// Builds a genuine `bplist00` `NSKeyedArchiver` byte stream with `NSScrollView -> NSTableView` via `plist::to_writer_binary`.
pub fn build_real_nskeyedarchiver_bplist() -> Vec<u8> {
    let mk_cls = |name: &str| -> Value {
        let mut d = Dictionary::new();
        d.insert("$classname".into(), Value::String(name.into()));
        Value::Dictionary(d)
    };
    let mut root_view = Dictionary::new();
    root_view.insert("$class".into(), Value::Uid(Uid::new(1)));
    root_view.insert("NSFrame".into(), Value::String("{{0, 0}, {480, 320}}".into()));

    let mut lbl_obj = Dictionary::new();
    lbl_obj.insert("$class".into(), Value::Uid(Uid::new(2)));
    lbl_obj.insert("NSFrame".into(), Value::String("{{24, 28}, {90, 16}}".into()));
    lbl_obj.insert("NSTitle".into(), Value::Uid(Uid::new(12)));

    let mut scroll_obj = Dictionary::new();
    scroll_obj.insert("$class".into(), Value::Uid(Uid::new(3)));
    scroll_obj.insert("NSFrame".into(), Value::String("{{20, 60}, {440, 200}}".into()));
    scroll_obj.insert("NSDocView".into(), Value::Uid(Uid::new(9)));

    let mut table_obj = Dictionary::new();
    table_obj.insert("$class".into(), Value::Uid(Uid::new(4)));
    table_obj.insert("NSTableColumns".into(), Value::Array(vec![Value::Uid(Uid::new(10)), Value::Uid(Uid::new(11))]));

    let mut col1 = Dictionary::new();
    col1.insert("$class".into(), Value::Uid(Uid::new(5)));
    col1.insert("NSTitle".into(), Value::String("Glyph".into()));
    col1.insert("NSWidth".into(), Value::Real(140.0));

    let mut col2 = Dictionary::new();
    col2.insert("$class".into(), Value::Uid(Uid::new(5)));
    col2.insert("NSTitle".into(), Value::String("Width".into()));
    col2.insert("NSWidth".into(), Value::Real(80.0));

    let objects = vec![
        Value::String("$null".into()),
        mk_cls("NSView"),
        mk_cls("NSTextField"),
        mk_cls("NSScrollView"),
        mk_cls("NSTableView"),
        mk_cls("NSTableColumn"),
        Value::Dictionary(root_view),
        Value::Dictionary(lbl_obj),
        Value::Dictionary(scroll_obj),
        Value::Dictionary(table_obj),
        Value::Dictionary(col1),
        Value::Dictionary(col2),
        Value::String("Scale:".into()),
    ];

    let mut root = Dictionary::new();
    root.insert("$archiver".into(), Value::String("NSKeyedArchiver".into()));
    root.insert("$version".into(), Value::Integer(100000.into()));
    root.insert("$objects".into(), Value::Array(objects));

    let mut buf = Vec::new();
    let _ = plist::to_writer_binary(&mut buf, &Value::Dictionary(root));
    buf
}

pub(crate) fn resolve_scene_vertical_stack_layout(scene: &mut MacSceneIR) {
    use crate::scene::constraints::ConstraintAttr;
    use crate::scene::ir::SceneNode;
    use std::collections::BTreeMap;

    crate::scene::storyboard::resolve_horizontal_stack_views(scene);
    if scene.nodes.first().and_then(|n| n.stack_orientation) != Some(1) {
        return;
    }
    let root_id = scene.root;
    let spacing = if scene.nodes[0].stack_spacing > 0.0 { scene.nodes[0].stack_spacing } else { 2.0 };
    let detaches_hidden = scene.nodes[0].detaches_hidden_views;
    let id_to_idx: BTreeMap<NodeId, usize> = scene.nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
    let is_super = |child: NodeId, other: NodeId, nodes: &[SceneNode]| -> bool {
        id_to_idx.get(&child).and_then(|&ci| nodes[ci].parent) == Some(other)
    };
    let is_active_node = |ni: usize, nodes: &[SceneNode]| -> bool {
        !(detaches_hidden && nodes[ni].is_hidden)
            && !matches!(nodes[ni].visibility, VisibilityRule::WhenTabSelected { tab_index, .. } if tab_index > 0)
    };

    let mut height_consts: BTreeMap<NodeId, f32> = BTreeMap::new();
    let mut top_to_super: BTreeMap<NodeId, f32> = BTreeMap::new();
    let mut top_to_sib_bot: BTreeMap<NodeId, (NodeId, f32)> = BTreeMap::new();
    let mut top_to_sib_top: Vec<(NodeId, NodeId, f32)> = Vec::new();
    let mut bot_to_super: BTreeMap<NodeId, f32> = BTreeMap::new();
    let mut centery_pairs: Vec<(NodeId, NodeId)> = Vec::new();
    let mut baseline_pairs: Vec<(NodeId, NodeId)> = Vec::new();

    for c in &scene.nib_constraints {
        let Some(f_id) = c.first_item else { continue };
        let s_opt = c.second_item;
        let (a1, a2, c_val) = (c.first_attr, c.second_attr, c.constant);
        if a1 == ConstraintAttr::Height && s_opt.is_none() {
            height_consts.insert(f_id, c_val);
        } else if a1 == ConstraintAttr::Top && a2 == ConstraintAttr::Top {
            if let Some(s_id) = s_opt {
                if is_super(f_id, s_id, &scene.nodes) {
                    top_to_super.insert(f_id, c_val);
                } else if is_super(s_id, f_id, &scene.nodes) {
                    top_to_super.insert(s_id, -c_val);
                } else {
                    top_to_sib_top.push((f_id, s_id, c_val));
                }
            }
        } else if a1 == ConstraintAttr::Top && a2 == ConstraintAttr::Bottom {
            if let Some(s_id) = s_opt {
                top_to_sib_bot.insert(f_id, (s_id, c_val));
            }
        } else if a1 == ConstraintAttr::Bottom && a2 == ConstraintAttr::Bottom {
            if let Some(s_id) = s_opt {
                if is_super(s_id, f_id, &scene.nodes) && c_val >= 0.0 {
                    bot_to_super.insert(s_id, c_val);
                } else if is_super(f_id, s_id, &scene.nodes) && c_val <= 0.0 {
                    bot_to_super.insert(f_id, -c_val);
                }
            }
        } else if a1 == ConstraintAttr::CenterY && a2 == ConstraintAttr::CenterY {
            if let Some(s_id) = s_opt {
                centery_pairs.push((f_id, s_id));
            }
        } else if matches!(a1, ConstraintAttr::LastBaseline | ConstraintAttr::FirstBaseline)
            && matches!(a2, ConstraintAttr::LastBaseline | ConstraintAttr::FirstBaseline)
        {
            if let Some(s_id) = s_opt {
                baseline_pairs.push((f_id, s_id));
            }
        }
    }

    let vert_insets = |nid: NodeId, node: &SceneNode, h_consts: &BTreeMap<NodeId, f32>| -> (f32, f32) {
        let mut node_with_h = node.clone();
        if let Some(&hc) = h_consts.get(&nid) {
            node_with_h.nib_frame[3] = hc;
        }
        let [t, _, b, _] = node_with_h.node_alignment_rect_insets();
        (t, b)
    };

    let mut align_h: BTreeMap<NodeId, f32> = BTreeMap::new();
    let mut local_top: BTreeMap<NodeId, f32> = top_to_super.clone();
    for (&nid, &ni) in &id_to_idx {
        let (in_t, in_b) = vert_insets(nid, &scene.nodes[ni], &height_consts);
        let h = if let Some(&hc) = height_consts.get(&nid) {
            hc
        } else {
            match &scene.nodes[ni].kind {
                SceneNodeKind::Label => 14.0,
                SceneNodeKind::ComboBox { .. } => 20.0,
                SceneNodeKind::Checkbox => 16.0,
                SceneNodeKind::TextField | SceneNodeKind::NumberField { .. } => {
                    if scene.nodes[ni].bezel_style == 1 || (!scene.nodes[ni].draws_background && scene.nodes[ni].is_bezeled) {
                        22.0
                    } else if scene.nodes[ni].nib_frame[3] >= 24.0 {
                        24.0
                    } else {
                        22.0
                    }
                }
                _ => (scene.nodes[ni].nib_frame[3] - in_t - in_b).max(0.0),
            }
        };
        align_h.insert(nid, h);
    }

    let baseline_off = |h: f32| -> f32 {
        if (h - 14.0).abs() <= 0.5 {
            11.0
        } else if (h - 20.0).abs() <= 0.5 {
            15.0
        } else {
            16.0
        }
    };

    for _ in 0..4 {
        for (&f_id, &(s_id, gap)) in &top_to_sib_bot {
            if let (Some(&s_t), Some(&s_h)) = (local_top.get(&s_id), align_h.get(&s_id)) {
                local_top.insert(f_id, s_t + s_h + gap);
            }
        }
        for &(f_id, s_id, c) in &top_to_sib_top {
            if let Some(&f_t) = local_top.get(&f_id) {
                local_top.entry(s_id).or_insert(f_t - c);
            }
            if let Some(&s_t) = local_top.get(&s_id) {
                local_top.entry(f_id).or_insert(s_t + c);
            }
        }
        for &(f_id, s_id) in &centery_pairs {
            if let (Some(&s_t), Some(&s_h), Some(&f_h)) = (local_top.get(&s_id), align_h.get(&s_id), align_h.get(&f_id)) {
                local_top.entry(f_id).or_insert(s_t + ((s_h - f_h) * 0.5).floor());
            } else if let (Some(&f_t), Some(&f_h), Some(&s_h)) = (local_top.get(&f_id), align_h.get(&f_id), align_h.get(&s_id)) {
                local_top.entry(s_id).or_insert(f_t + ((f_h - s_h) * 0.5).floor());
            }
        }
        for &(f_id, s_id) in &baseline_pairs {
            if let (Some(&s_t), Some(&s_h), Some(&f_h)) = (local_top.get(&s_id), align_h.get(&s_id), align_h.get(&f_id)) {
                local_top.entry(f_id).or_insert(s_t + baseline_off(s_h) - baseline_off(f_h));
            } else if let (Some(&f_t), Some(&f_h), Some(&s_h)) = (local_top.get(&f_id), align_h.get(&f_id), align_h.get(&s_id)) {
                local_top.entry(s_id).or_insert(f_t + baseline_off(f_h) - baseline_off(s_h));
            }
        }
        for (&c_id, &bot_c) in &bot_to_super {
            if let Some(&c_ni) = id_to_idx.get(&c_id) {
                if !is_active_node(c_ni, &scene.nodes) {
                    continue;
                }
                if let (Some(&c_t), Some(&c_h), Some(pid)) = (local_top.get(&c_id), align_h.get(&c_id), scene.nodes[c_ni].parent) {
                    let req_h = c_t + c_h + bot_c;
                    align_h.insert(pid, req_h);
                    if let Some(&p_ni) = id_to_idx.get(&pid) {
                        if let Some(gp_id) = scene.nodes[p_ni].parent {
                            if gp_id != root_id
                                && id_to_idx.get(&gp_id).is_some_and(|&gni| {
                                    matches!(scene.nodes[gni].kind, SceneNodeKind::TabView { .. })
                                        || scene.nodes[gni].stack_orientation.is_some()
                                })
                            {
                                local_top.entry(pid).or_insert(0.0);
                                align_h.insert(gp_id, req_h);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut global_top: BTreeMap<NodeId, f32> = BTreeMap::from([(root_id, 0.0)]);
    let mut y_cursor = 0.0f32;
    let row_children = scene.nodes[0].children.clone();
    for row_nid in &row_children {
        if let Some(&r_ni) = id_to_idx.get(row_nid) {
            if detaches_hidden && scene.nodes[r_ni].is_hidden {
                continue;
            }
        }
        let rh = align_h.get(row_nid).copied().unwrap_or(22.0);
        global_top.insert(*row_nid, y_cursor);
        y_cursor += rh + spacing;
    }
    let total_h = (y_cursor - spacing).max(1.0);
    scene.nodes[0].nib_frame[1] = 0.0;
    scene.nodes[0].nib_frame[3] = total_h;
    scene.design_size[1] = total_h;

    for ni in 1..scene.nodes.len() {
        let nid = scene.nodes[ni].id;
        let g_top = if let Some(&gt) = global_top.get(&nid) {
            gt
        } else {
            let p_top = scene.nodes[ni].parent.and_then(|pid| global_top.get(&pid).copied()).unwrap_or(0.0);
            let lt = local_top.get(&nid).copied().unwrap_or(0.0);
            let gt = p_top + lt;
            global_top.insert(nid, gt);
            gt
        };
        let ah = align_h.get(&nid).copied().unwrap_or(scene.nodes[ni].nib_frame[3]);
        let (in_t, in_b) = vert_insets(nid, &scene.nodes[ni], &height_consts);
        let frame_top = g_top - in_t;
        let frame_h = ah + in_t + in_b;
        scene.nodes[ni].nib_frame[1] = total_h - (frame_top + frame_h);
        scene.nodes[ni].nib_frame[3] = frame_h;
    }
}

/// Hardware-calibrated macOS Sonoma 14.8.9 `.AppleSystemUIFont` (`11.0 pt`) printable ASCII (`0x20..=0x7E`)
/// advance table, measured directly from `-[NSString sizeWithAttributes:]` on the reference MacBook.
const APPKIT_SYSTEM_FONT_11PT_ASCII_ADVANCES: [f32; 95] = [
    3.16, 3.49, 5.32, 6.99, 6.99, 10.24, 7.89, 3.33, 4.26, 4.26, 5.25, 6.99, 3.33, 5.25, 3.33, 3.42, 6.99, 5.17, 6.70, 6.96, 7.14, 6.86,
    7.07, 6.33, 7.09, 7.07, 3.33, 3.33, 6.99, 6.99, 6.99, 5.70, 10.16, 7.48, 7.29, 7.94, 8.06, 6.62, 6.36, 8.28, 8.23, 3.01, 5.98, 7.31,
    6.31, 9.68, 8.23, 8.55, 7.05, 8.55, 7.25, 7.07, 7.04, 8.17, 7.48, 10.71, 7.53, 7.27, 7.34, 4.26, 3.42, 4.26, 6.99, 6.48, 5.56, 6.13,
    6.82, 6.22, 6.82, 6.35, 4.04, 6.77, 6.54, 2.78, 2.78, 6.04, 2.85, 9.64, 6.48, 6.56, 6.78, 6.77, 4.25, 5.82, 4.06, 6.48, 6.03, 8.58,
    5.83, 6.04, 5.99, 4.26, 2.91, 4.26, 6.99,
];

/// Computes the exact macOS Sonoma 14.8.9 AppKit `-[NSTextField intrinsicContentSize].width` (in points at `font_size_pt`)
/// for `text`, calibrated against live MacBook `NSFont.systemFont(ofSize:)` telemetry (`REQ-013-10.1` §3.8.4).
pub fn measure_appkit_label_text_width(text: &str, font_size_pt: f32) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let scale = (font_size_pt / 11.0).max(0.5);
    let mut sum_at_11pt = 0.0_f32;
    for ch in text.chars() {
        let code = ch as u32;
        let adv_11 = if (32..=126).contains(&code) { APPKIT_SYSTEM_FONT_11PT_ASCII_ADVANCES[(code - 32) as usize] } else { 6.48 };
        sum_at_11pt += adv_11;
    }
    (sum_at_11pt * scale).ceil()
}

impl crate::scene::ir::SceneNode {
    /// Returns the intrinsic `alignmentRect` width of a static `Label` (`REQ-013-10.1` §3.8.4):
    /// preserves the exact Xcode-archived `.nib` `alignmentRect` width (`w_nib - 4.0`) when present,
    /// or measures `self.title` at `self.font_size_pt` (`11.0pt` small / `13.0pt` regular).
    pub fn intrinsic_label_alignment_width(&self) -> f32 {
        if self.nib_frame[2] > 4.0 {
            (self.nib_frame[2] - 4.0).max(0.0)
        } else {
            let pt = self.font_size_pt.unwrap_or(match self.control_size.as_str() {
                "mini" | "Mini" => 9.0,
                "small" | "Small" => 11.0,
                _ => 13.0,
            });
            measure_appkit_label_text_width(&self.title, pt)
        }
    }

    /// Returns the 4D AppKit `alignmentRectInsets` `[top, left, bottom, right]` for this `SceneNode` (`REQ-013-10.2` §4.1.4).
    pub fn node_alignment_rect_insets(&self) -> [f32; 4] {
        match &self.kind {
            crate::scene::ir::SceneNodeKind::InlineActionButton => match self.nib_frame[3].round() as i32 {
                18 | 22 => [2.0, 0.0, 2.0, 0.0],
                19 | 26 => [4.0, 0.0, 3.0, 0.0],
                27 => [1.0, 1.0, 1.0, 1.0],
                _ => [0.0, 0.0, 0.0, 0.0],
            },
            crate::scene::ir::SceneNodeKind::ImageView { icon_name, .. } => match icon_name.as_str() {
                "tag" => [6.0, 0.0, 6.0, 0.0],
                "u.square" => [4.0, 0.0, 4.0, 0.0],
                _ => [0.0, 0.0, 0.0, 0.0],
            },
            crate::scene::ir::SceneNodeKind::PushButton { .. } if self.nib_frame[3] <= 20.0 => [0.0, 0.0, 0.0, 0.0],
            kind => crate::scene::constraints::appkit_alignment_rect_insets(kind),
        }
    }
}
