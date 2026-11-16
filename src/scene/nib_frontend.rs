//! Layer 1 Binary `.nib` Frontend (`NIBArchive\x01\x00\x00\x00` & `bplist00`, `SPEC-013 §2.1..2.2`).

use crate::scene::constraints::{CompiledConstraintSchedule, ValueTransformer};
use crate::scene::ir::{
    ActionId, BindingId, CustomClassMapper, MacSceneIR, NodeId, SceneCompileError, SceneNode, SceneNodeKind, VisibilityRule,
};
use crate::scene::nib_varint::write_nib_varint;

pub(crate) fn mk_node(
    id: u32,
    ib_id: &str,
    parent: Option<u32>,
    children: Vec<u32>,
    kind: SceneNodeKind,
    title: &str,
    tooltip: &str,
    size: &str,
    nib_frame: [f32; 4],
    mask: u8,
    binding: Option<(u32, &str)>,
    action: Option<(u32, &str)>,
    icon: Option<&str>,
    visibility: VisibilityRule,
) -> SceneNode {
    SceneNode {
        id: NodeId(id),
        ib_object_id: ib_id.into(),
        parent: parent.map(NodeId),
        children: children.into_iter().map(NodeId).collect(),
        kind,
        title: title.into(),
        tooltip: tooltip.into(),
        control_size: size.into(),
        nib_frame,
        alignment_rect: nib_frame,
        autoresizing_mask: mask,
        enabled: true,
        value_binding: binding.map(|(b, _)| BindingId(b)),
        binding_path: binding.map(|(_, p)| p.into()),
        value_transformer: ValueTransformer::Identity,
        action: action.map(|(a, _)| ActionId(a)),
        action_selector: action.map(|(_, s)| s.into()),
        icon_name: icon.map(Into::into),
        visibility,
        labeled_by: None,
    }
}

/// Binary `.nib` ingestion frontend supporting `NIBArchive` v1 and `NSKeyedArchiver` (`bplist00`).
pub struct NibArchiveFrontend;

impl NibArchiveFrontend {
    /// Decodes a binary `.nib` buffer (`NIBArchive\x01\x00\x00\x00` or `bplist00`) into [`MacSceneIR`].
    pub fn from_bytes(bytes: &[u8], mapper: &CustomClassMapper) -> Result<MacSceneIR, SceneCompileError> {
        if bytes.starts_with(b"NIBArchive\x01\x00\x00\x00") {
            Self::parse_nib_archive_v1(bytes, mapper)
        } else if bytes.starts_with(b"bplist00") {
            Self::parse_bplist_archive(bytes, mapper)
        } else {
            Err(SceneCompileError("Unsupported binary NIB header (expected NIBArchive v1 or bplist00)".to_string()))
        }
    }

    /// Loads a `.nib` file or `.nib` bundle directory (`keyedobjects-101300.nib` / `keyedobjects.nib`) from disk (`REQ-013-05.2`).
    pub fn from_path(path: &std::path::Path, mapper: &CustomClassMapper) -> Result<MacSceneIR, SceneCompileError> {
        crate::scene::nib_ibtool::load_nib_from_path(path, mapper)
    }

    fn parse_nib_archive_v1(bytes: &[u8], mapper: &CustomClassMapper) -> Result<MacSceneIR, SceneCompileError> {
        if bytes.len() < 50 {
            return Err(SceneCompileError("NIBArchive buffer shorter than 50-byte header".into()));
        }
        crate::scene::nib_varint::decode_real_nib_archive_v1(bytes, mapper)
    }

    fn parse_bplist_archive(bytes: &[u8], mapper: &CustomClassMapper) -> Result<MacSceneIR, SceneCompileError> {
        if bytes.len() < 16 {
            return Err(SceneCompileError("bplist00 buffer too short".into()));
        }
        crate::scene::nib_varint::decode_nskeyedarchiver_plist(bytes, mapper)
    }

    /// Builds a genuine binary `NIBArchive\x01\x00\x00\x00` stream with varints, keys, classes, values, and objects.
    pub fn build_real_varint_nib_archive() -> Vec<u8> {
        crate::scene::nib_varint::build_real_varint_nib_archive()
    }

    /// Builds a genuine `bplist00` `NSKeyedArchiver` byte stream via `plist::to_writer_binary`.
    pub fn build_real_nskeyedarchiver_bplist() -> Vec<u8> {
        crate::scene::nib_varint::build_real_nskeyedarchiver_bplist()
    }

    /// Encodes a slice of [`SceneNode`]s into a genuine binary `NIBArchive\x01\x00\x00\x00` varint stream (`REQ-013-05.5`).
    pub fn encode_nodes_to_nib_archive(nodes: &[SceneNode]) -> Vec<u8> {
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
            "NSUserInterfaceItemIdentifier",
            "NSToolTip",
            "NSImageName",
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
            "NSBoxSeparator",
            "NSSwitch",
        ];
        let mut cls_sec = Vec::new();
        for c in cls_names {
            write_nib_varint(&mut cls_sec, (c.len() + 1) as u32);
            write_nib_varint(&mut cls_sec, 0);
            cls_sec.extend_from_slice(c.as_bytes());
            cls_sec.push(0);
        }

        let mut vals_sec = Vec::new();
        let mut objs: Vec<(u32, u32, u32)> = Vec::new();
        let mut val_count = 0u32;

        let push_bytes = |vals: &mut Vec<u8>, vc: &mut u32, k: u32, b: &[u8]| {
            write_nib_varint(vals, k);
            vals.push(8u8);
            write_nib_varint(vals, b.len() as u32);
            vals.extend_from_slice(b);
            *vc += 1;
        };
        let push_int = |vals: &mut Vec<u8>, vc: &mut u32, k: u32, v: i32| {
            write_nib_varint(vals, k);
            vals.push(2u8);
            vals.extend_from_slice(&v.to_le_bytes());
            *vc += 1;
        };
        let push_ref = |vals: &mut Vec<u8>, vc: &mut u32, k: u32, obj_idx: u32| {
            write_nib_varint(vals, k);
            vals.push(10u8);
            vals.extend_from_slice(&obj_idx.to_le_bytes());
            *vc += 1;
        };
        let rect_bytes = |r: [f32; 4]| -> Vec<u8> {
            let mut b = vec![6u8];
            for f in r {
                b.extend_from_slice(&f.to_le_bytes());
            }
            b
        };

        let mut pending_cells: Vec<(u32, u32, u32, String)> = Vec::new();
        let mut pending_conns: Vec<(u32, u32, String, bool)> = Vec::new();
        let base_cell_idx = nodes.len() as u32;

        for (i, n) in nodes.iter().enumerate() {
            let v_start = val_count;
            let (cls_idx, custom_cls_opt, cell_cls_opt) = match &n.kind {
                SceneNodeKind::Container { .. } | SceneNodeKind::BoxContainer => (0u32, None, None),
                SceneNodeKind::Separator => (8u32, None, None),
                SceneNodeKind::Label => (1u32, None, Some(4u32)),
                SceneNodeKind::TextField => (1u32, None, Some(4u32)),
                SceneNodeKind::NumberField { .. } => (2u32, Some("SteppingTextField"), Some(4u32)),
                SceneNodeKind::OriginMatrix => (2u32, Some("OriginPointControl"), None),
                SceneNodeKind::Checkbox | SceneNodeKind::PushButton { .. } | SceneNodeKind::RadioButton => (3u32, None, Some(5u32)),
                SceneNodeKind::Switch => (9u32, None, None),
                SceneNodeKind::CustomCanvasSlot { class_name } => (2u32, Some(class_name.as_str()), Some(4u32)),
                _ => (1u32, None, Some(4u32)),
            };
            push_bytes(&mut vals_sec, &mut val_count, 0, &rect_bytes(n.nib_frame));
            let title_str = if matches!(n.kind, SceneNodeKind::Label) && !n.title.ends_with(':') && !n.title.is_empty() {
                format!("{}:", n.title)
            } else {
                n.title.clone()
            };
            if !title_str.is_empty() {
                push_bytes(&mut vals_sec, &mut val_count, 1, title_str.as_bytes());
            }
            if let Some(ccls) = custom_cls_opt {
                push_bytes(&mut vals_sec, &mut val_count, 2, ccls.as_bytes());
            }
            let unflipped_bit = if i == 0 && matches!(n.kind, SceneNodeKind::Container { is_flipped: false }) { 0x80i32 } else { 0 };
            if n.autoresizing_mask != 0 || unflipped_bit != 0 {
                push_int(&mut vals_sec, &mut val_count, 4, (n.autoresizing_mask as i32) | unflipped_bit);
            }
            push_bytes(&mut vals_sec, &mut val_count, 11, n.ib_object_id.as_bytes());
            if !n.tooltip.is_empty() {
                push_bytes(&mut vals_sec, &mut val_count, 12, n.tooltip.as_bytes());
            }
            if let Some(ref ic) = n.icon_name {
                push_bytes(&mut vals_sec, &mut val_count, 13, ic.as_bytes());
            }
            if let Some(c_cls) = cell_cls_opt {
                let cell_obj_idx = base_cell_idx + pending_cells.len() as u32;
                push_ref(&mut vals_sec, &mut val_count, 3, cell_obj_idx);
                let sz_bits = match n.control_size.as_str() {
                    "small" => 1u32 << 18,
                    "mini" => 2u32 << 18,
                    _ => 0u32,
                };
                let b_flags = if matches!(n.kind, SceneNodeKind::Checkbox) { 0x0080_0000 } else { 0 };
                let ke = match n.kind {
                    SceneNodeKind::PushButton { is_default: true, .. } => "\r".to_string(),
                    SceneNodeKind::PushButton { is_cancel: true, .. } => "\u{1b}".to_string(),
                    _ => String::new(),
                };
                pending_cells.push((c_cls, sz_bits, b_flags, ke));
            }
            if let Some(ref kp) = n.binding_path {
                pending_conns.push((7u32, i as u32, kp.clone(), false));
            }
            if let Some(ref sel) = n.action_selector {
                pending_conns.push((6u32, i as u32, sel.clone(), true));
            }
            objs.push((cls_idx, v_start, val_count - v_start));
        }

        for (c_cls, cflags, bflags, ke) in pending_cells {
            let v_start = val_count;
            push_int(&mut vals_sec, &mut val_count, 5, cflags as i32);
            if bflags != 0 {
                push_int(&mut vals_sec, &mut val_count, 6, bflags as i32);
            }
            if !ke.is_empty() {
                push_bytes(&mut vals_sec, &mut val_count, 7, ke.as_bytes());
            }
            objs.push((c_cls, v_start, val_count - v_start));
        }

        for (conn_cls, src_idx, str_val, is_action) in pending_conns {
            let v_start = val_count;
            push_ref(&mut vals_sec, &mut val_count, 8, src_idx);
            push_bytes(&mut vals_sec, &mut val_count, if is_action { 9 } else { 10 }, str_val.as_bytes());
            objs.push((conn_cls, v_start, val_count - v_start));
        }

        let mut objs_sec = Vec::new();
        for (c, vs, vl) in &objs {
            write_nib_varint(&mut objs_sec, *c);
            write_nib_varint(&mut objs_sec, *vs);
            write_nib_varint(&mut objs_sec, *vl);
        }

        let obj_off = 52u32;
        let key_off = obj_off + objs_sec.len() as u32;
        let val_off = key_off + keys_sec.len() as u32;
        let cls_off = val_off + vals_sec.len() as u32;

        let mut out = Vec::from(&b"NIBArchive\x01\x00\x00\x00"[..]);
        for u in [
            0u32,
            objs.len() as u32,
            obj_off,
            key_names.len() as u32,
            key_off,
            val_count,
            val_off,
            cls_names.len() as u32,
            cls_off,
        ] {
            out.extend_from_slice(&u.to_le_bytes());
        }
        out.resize(52, 0);
        out.extend_from_slice(&objs_sec);
        out.extend_from_slice(&keys_sec);
        out.extend_from_slice(&vals_sec);
        out.extend_from_slice(&cls_sec);
        out
    }

    fn default_fixture_nodes() -> Vec<SceneNode> {
        vec![
            mk_node(
                1,
                "root",
                None,
                vec![2, 3, 4, 5, 6],
                SceneNodeKind::Container { is_flipped: true },
                "Window",
                "",
                "regular",
                [0.0, 0.0, 480.0, 320.0],
                18,
                None,
                None,
                None,
                VisibilityRule::Always,
            ),
            mk_node(
                2,
                "10",
                Some(1),
                vec![],
                SceneNodeKind::Label,
                "Scale:",
                "",
                "regular",
                [20.0, 24.0, 60.0, 16.0],
                0,
                None,
                None,
                None,
                VisibilityRule::Always,
            ),
            mk_node(
                3,
                "11",
                Some(1),
                vec![],
                SceneNodeKind::CustomCanvasSlot { class_name: "SteppingTextField".into() },
                "100",
                "Scale percentage",
                "regular",
                [88.0, 21.0, 72.0, 22.0],
                2,
                Some((1, "values.scalePercent")),
                None,
                None,
                VisibilityRule::Always,
            ),
            mk_node(
                4,
                "12",
                Some(1),
                vec![],
                SceneNodeKind::CustomCanvasSlot { class_name: "OriginPointControl".into() },
                "Origin",
                "Transform origin anchor",
                "regular",
                [180.0, 18.0, 34.0, 34.0],
                0,
                Some((2, "values.originAnchor")),
                None,
                None,
                VisibilityRule::Always,
            ),
            mk_node(
                5,
                "13",
                Some(1),
                vec![],
                SceneNodeKind::Checkbox,
                "Snap to Grid",
                "Snap points to grid",
                "small",
                [20.0, 64.0, 140.0, 16.0],
                0,
                Some((3, "values.snapToGrid")),
                None,
                None,
                VisibilityRule::Always,
            ),
            mk_node(
                6,
                "14",
                Some(1),
                vec![],
                SceneNodeKind::PushButton { is_default: true, is_cancel: false },
                "Apply",
                "Apply transformation",
                "regular",
                [-6.0, -6.0, 131.0, 40.0],
                9,
                None,
                Some((1, "applyTransform:")),
                Some("NSAddTemplate"),
                VisibilityRule::Always,
            ),
        ]
    }

    /// Builds a valid binary `NIBArchive\x01\x00\x00\x00` fixture buffer for COTS verification.
    pub fn build_synthetic_nib_fixture() -> Vec<u8> {
        Self::encode_nodes_to_nib_archive(&Self::default_fixture_nodes())
    }

    /// Builds a valid binary `bplist00` fixture buffer for COTS verification.
    pub fn build_synthetic_bplist_fixture() -> Vec<u8> {
        Self::build_real_nskeyedarchiver_bplist()
    }

    /// Builds a multi-section binary `NIBArchive` fixture with `NSBoxSeparator` dividers and baseline-paired labels.
    pub fn build_synthetic_preferences_nib() -> Vec<u8> {
        let nodes = vec![
            mk_node(
                1,
                "pref-root",
                None,
                vec![2, 3, 4, 5, 6],
                SceneNodeKind::Container { is_flipped: true },
                "Preferences",
                "",
                "regular",
                [0.0, 0.0, 520.0, 360.0],
                18,
                None,
                None,
                None,
                VisibilityRule::Always,
            ),
            mk_node(
                2,
                "201",
                Some(1),
                vec![],
                SceneNodeKind::Label,
                "Grid Spacing:",
                "",
                "regular",
                [24.0, 28.0, 100.0, 16.0],
                0,
                None,
                None,
                None,
                VisibilityRule::Always,
            ),
            mk_node(
                3,
                "202",
                Some(1),
                vec![],
                SceneNodeKind::TextField,
                "10",
                "Grid unit spacing",
                "regular",
                [132.0, 25.0, 80.0, 22.0],
                0,
                Some((10, "defaults.gridSpacing")),
                None,
                None,
                VisibilityRule::Always,
            ),
            mk_node(
                4,
                "203",
                Some(1),
                vec![],
                SceneNodeKind::Separator,
                "",
                "",
                "regular",
                [20.0, 70.0, 480.0, 2.0],
                2,
                None,
                None,
                None,
                VisibilityRule::Always,
            ),
            mk_node(
                5,
                "204",
                Some(1),
                vec![],
                SceneNodeKind::Label,
                "Snap Distance:",
                "",
                "small",
                [24.0, 96.0, 100.0, 16.0],
                0,
                None,
                None,
                None,
                VisibilityRule::Always,
            ),
            mk_node(
                6,
                "205",
                Some(1),
                vec![],
                SceneNodeKind::Checkbox,
                "Enable Snap",
                "Snap cursor to guidelines",
                "small",
                [132.0, 95.0, 140.0, 18.0],
                0,
                Some((11, "defaults.enableSnap")),
                Some((10, "toggleSnap:")),
                Some("NSActionTemplate"),
                VisibilityRule::Always,
            ),
        ];
        Self::encode_nodes_to_nib_archive(&nodes)
    }

    /// Builds a toolbar scene containing context-dependent tools guarded by `VisibilityRule::WhenWorkspaceFlagsContain`.
    pub fn build_synthetic_toolbar_scene() -> MacSceneIR {
        const MODE_DRAWING: u64 = 1 << 0;
        const MODE_INSTRUCTOR: u64 = 1 << 1;

        let nodes = vec![
            mk_node(
                1,
                "tb-1",
                None,
                vec![],
                SceneNodeKind::PushButton { is_default: false, is_cancel: false },
                "Select",
                "Select tool",
                "regular",
                [12.0, 6.0, 36.0, 26.0],
                0,
                None,
                Some((1, "selectTool:")),
                None,
                VisibilityRule::WhenWorkspaceFlagsContain(MODE_DRAWING),
            ),
            mk_node(
                2,
                "tb-2",
                None,
                vec![],
                SceneNodeKind::PushButton { is_default: false, is_cancel: false },
                "Instructor",
                "TrueType Instructor tool",
                "regular",
                [56.0, 6.0, 36.0, 26.0],
                0,
                None,
                Some((2, "instructorTool:")),
                None,
                VisibilityRule::WhenWorkspaceFlagsContain(MODE_INSTRUCTOR),
            ),
            mk_node(
                3,
                "tb-3",
                None,
                vec![],
                SceneNodeKind::PushButton { is_default: false, is_cancel: false },
                "Pen",
                "Pen drawing tool",
                "regular",
                [100.0, 6.0, 36.0, 26.0],
                0,
                None,
                Some((3, "penTool:")),
                None,
                VisibilityRule::WhenWorkspaceFlagsContain(MODE_DRAWING),
            ),
        ];
        MacSceneIR::from_nodes(NodeId(1), [600.0, 38.0], nodes, CompiledConstraintSchedule::default())
    }
}
