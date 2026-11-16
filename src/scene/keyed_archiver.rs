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
