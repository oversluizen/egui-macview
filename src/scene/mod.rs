//! Three-Layer UI Compiler Architecture (`SPEC-013`, Milestone `v0.7.0`).
//!
//! Provides multi-format ingestion frontends (`NibArchiveFrontend`, `XibXmlFrontend`,
//! `SwiftUiFrontend`, `StringsCatalog`, `MacSceneSpec`), a strongly typed intermediate
//! representation (`MacSceneIR`, `SpatialSemanticNormalizer`, `SceneSearchIndex`,
//! `SceneContractVerifier`, `SceneExporter`), and a dual-mode 120 FPS layout solver
//! (`LayoutPolicy::ExactAppleNib` and `LayoutPolicy::AdaptiveSemanticReflow`).

pub mod constraints;
pub mod exporter;
pub mod ir;
pub mod keyed_archiver;
pub mod nib_frontend;
pub mod nib_ibtool;
pub(crate) mod nib_ibtool_widgets;
pub mod nib_varint;
pub mod normalizer;
pub mod runtime;
pub(crate) mod runtime_tabs;
pub(crate) mod runtime_widgets;
pub(crate) mod stack_reflow;
pub mod storyboard;
pub mod swiftui;
pub mod xib_exporter;
pub mod xib_strings;

pub use constraints::{
    appkit_alignment_rect_insets, evaluate_arithmetic_expr, CompiledConstraintSchedule, ConstraintAttr, ConstraintRelation,
    LayoutConstraintSpec, ValueTransformer,
};
pub use exporter::{MacSceneSpec, SceneExporter};
pub use ir::{
    ActionId, BindingId, BindingValueType, ContractAuditReport, CustomClassMapper, FieldRowGroup, HostContractSchema, LabeledFieldPair,
    LayoutPolicy, MacSceneIR, MacTextAlignment, NodeId, SceneCompileError, SceneFilterMask, SceneNode, SceneNodeKind, SemanticSectionGroup,
    VisibilityRule,
};
pub use keyed_archiver::measure_appkit_label_text_width;
pub use nib_frontend::NibArchiveFrontend;
pub use runtime::MacSceneDataSource;
pub use storyboard::{MacStoryboardIR, SegueKind, StoryboardFrontend, StoryboardScene, StoryboardSegue};
pub use swiftui::SwiftUiFrontend;
pub use xib_strings::XibXmlFrontend;

/// Apple `<NibName>.strings` localization catalog (`"ObjectID.property" = "Localized";`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StringsCatalog {
    entries: std::collections::BTreeMap<(String, String), String>,
}

impl StringsCatalog {
    /// Parses an Apple `.strings` text file (`/* comments */` and `"ObjectID.prop" = "Value";` lines).
    pub fn from_str(source: &str) -> Result<Self, SceneCompileError> {
        let mut entries = std::collections::BTreeMap::new();
        for raw_line in source.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with("/*") || line.starts_with("//") {
                continue;
            }
            let Some((lhs, rhs)) = line.split_once('=') else {
                continue;
            };
            let key = unquote_strings_token(lhs.trim());
            let val = unquote_strings_token(rhs.trim().trim_end_matches(';').trim());
            if let Some((obj_id, prop)) = key.split_once('.') {
                entries.insert((obj_id.to_string(), prop.to_string()), val);
            }
        }
        Ok(Self { entries })
    }

    /// Parses either a binary/XML `plist` `.strings` file or a UTF-8 text `.strings` file.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SceneCompileError> {
        if let Ok(plist::Value::Dictionary(dict)) = plist::from_bytes::<plist::Value>(bytes) {
            let mut entries = std::collections::BTreeMap::new();
            for (k, v) in dict {
                if let (Some((obj_id, prop)), Some(s)) = (k.split_once('.'), v.as_string()) {
                    entries.insert((obj_id.to_string(), prop.to_string()), s.to_string());
                }
            }
            return Ok(Self { entries });
        }
        let text = std::str::from_utf8(bytes).map_err(|e| SceneCompileError(format!("Invalid UTF-8 in .strings: {e}")))?;
        Self::from_str(text)
    }

    /// Looks up a localized property for `(ib_object_id, property)`.
    pub fn get(&self, ib_object_id: &str, property: &str) -> Option<&str> {
        self.entries.get(&(ib_object_id.to_string(), property.to_string())).map(String::as_str)
    }
}

fn unquote_strings_token(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].replace("\\\"", "\"").replace("\\n", "\n")
    } else {
        trimmed.to_string()
    }
}

impl MacSceneIR {
    /// Applies a localized [`StringsCatalog`] to all matching nodes in $O(N)$ time, updates localized static `Label`
    /// intrinsic `alignmentRect` widths (`REQ-013-10.1` §3.8.4), and rebuilds [`MacSceneIR::rebuild_search_index`].
    pub fn apply_strings_catalog(&mut self, catalog: &StringsCatalog) {
        let root_flipped = self.nodes.first().is_some_and(|n| matches!(n.kind, SceneNodeKind::Container { is_flipped: true }));
        let design_h = self.design_size[1];
        for node in &mut self.nodes {
            let mut title_changed = false;
            if let Some(title) = catalog.get(&node.ib_object_id, "title").or_else(|| catalog.get(&node.ib_object_id, "stringValue")) {
                if node.title != title {
                    node.title = title.to_string();
                    title_changed = true;
                }
            } else if let Some(ph) = catalog.get(&node.ib_object_id, "placeholderString") {
                if node.title.is_empty() {
                    node.title = ph.to_string();
                }
            }
            if title_changed && matches!(node.kind, SceneNodeKind::Label) && node.value_binding.is_none() {
                let pt = node.font_size_pt.unwrap_or(match node.control_size.as_str() {
                    "mini" | "Mini" => 9.0,
                    "small" | "Small" => 11.0,
                    _ => 13.0,
                });
                let old_cx = node.nib_frame[0] + node.nib_frame[2] * 0.5;
                let new_align_w = keyed_archiver::measure_appkit_label_text_width(&node.title, pt);
                node.nib_frame[2] = new_align_w + 4.0;
                if node.text_alignment == crate::scene::ir::MacTextAlignment::Center {
                    node.nib_frame[0] = (old_cx - node.nib_frame[2] * 0.5).max(0.0);
                }
                node.alignment_rect =
                    normalizer::normalize_nib_alignment_rect(&node.kind, node.mac_control_size(), node.nib_frame, design_h, root_flipped);
            }
            if let Some(tip) = catalog.get(&node.ib_object_id, "ibShadowedToolTip") {
                node.tooltip = tip.to_string();
            }
        }
        for items in self.button_menus.values_mut() {
            for (mi_ib_id, mi_title, _, _) in items {
                if let Some(t) = catalog.get(mi_ib_id, "title") {
                    *mi_title = t.to_string();
                }
            }
        }
        self.solve_autolayout();
        self.rebuild_search_index();
    }

    /// Computes the live vertical height of an `NSStackView` scene (`stack_orientation == Some(1)`)
    /// after detaching any arranged subviews whose `is_hidden` or `hidden_binding` evaluates to `true` (`detachesHiddenViews`).
    pub fn effective_stack_height(&self, data_source: &dyn MacSceneDataSource) -> f32 {
        let Some(root) = self.nodes.first() else { return 0.0 };
        if root.stack_orientation != Some(1) || !root.detaches_hidden_views {
            return root.nib_frame[3];
        }
        let spacing = if root.stack_spacing > 0.0 { root.stack_spacing } else { 2.0 };
        let collapsed_h: f32 = root
            .children
            .iter()
            .filter_map(|&cid| self.nodes.iter().find(|n| n.id == cid))
            .filter(|row| row.is_hidden || row.hidden_binding.is_some_and(|hb| row.value_transformer.apply_bool(data_source.get_bool(hb))))
            .map(|row| row.nib_frame[3] + spacing)
            .sum();
        (root.nib_frame[3] - collapsed_h).max(1.0)
    }
}

/// Resolves and loads a `.nib` file or `.nib` directory bundle (`keyedobjects-101300.nib` / `keyedobjects.nib` / `objects-11.0+.nib`),
/// automatically applying a companion `../en.lproj/<Stem>.strings` or `./<Stem>.strings` catalog if present.
pub(crate) fn load_nib_from_path(path: &std::path::Path, mapper: &CustomClassMapper) -> Result<MacSceneIR, SceneCompileError> {
    load_nib_from_path_outlet(path, None, mapper)
}

pub(crate) fn load_nib_from_path_outlet(
    path: &std::path::Path,
    outlet_label: Option<&str>,
    mapper: &CustomClassMapper,
) -> Result<MacSceneIR, SceneCompileError> {
    let parse_fn = |bytes: &[u8]| -> Result<MacSceneIR, SceneCompileError> {
        if let Some(lbl) = outlet_label {
            nib_ibtool::IbNibGraph::decode_outlet_root(bytes, lbl, mapper)
        } else {
            NibArchiveFrontend::from_bytes(bytes, mapper)
        }
    };
    let mut scene = if path.is_dir() {
        let mut loaded = None;
        for candidate in ["keyedobjects-101300.nib", "keyedobjects.nib", "objects-11.0+.nib", "objects.nib"] {
            let sub = path.join(candidate);
            if sub.is_file() {
                let bytes = std::fs::read(&sub).map_err(|e| SceneCompileError(format!("Failed reading {}: {e}", sub.display())))?;
                loaded = Some(parse_fn(&bytes)?);
                break;
            }
        }
        loaded.ok_or_else(|| SceneCompileError(format!("No inner .nib stream found in bundle directory {}", path.display())))?
    } else {
        let bytes = std::fs::read(path).map_err(|e| SceneCompileError(format!("Failed reading {}: {e}", path.display())))?;
        parse_fn(&bytes)?
    };
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        let strings_name = format!("{stem}.strings");
        let mut candidates = Vec::new();
        if let Some(parent) = path.parent() {
            candidates.push(parent.join(&strings_name));
            candidates.push(parent.join("en.lproj").join(&strings_name));
            if let Some(grandparent) = parent.parent() {
                candidates.push(grandparent.join("en.lproj").join(&strings_name));
            }
        }
        for cand in candidates {
            if cand.is_file() {
                if let Ok(sbytes) = std::fs::read(&cand) {
                    if let Ok(cat) = StringsCatalog::from_bytes(&sbytes) {
                        scene.apply_strings_catalog(&cat);
                        break;
                    }
                }
            }
        }
        if let Some(parent) = path.parent() {
            for loc_cand in [
                parent.join("en.lproj").join("Localizable.strings"),
                parent.parent().unwrap_or(parent).join("en.lproj").join("Localizable.strings"),
            ] {
                if loc_cand.is_file() {
                    if let Ok(sbytes) = std::fs::read(&loc_cand) {
                        if apply_localizable_plist_bytes(&mut scene, &sbytes) {
                            break;
                        }
                    }
                }
            }
        }
    }
    Ok(scene)
}

pub(crate) fn apply_localizable_plist_bytes(scene: &mut MacSceneIR, sbytes: &[u8]) -> bool {
    let Ok(plist::Value::Dictionary(dict)) = plist::from_bytes::<plist::Value>(sbytes) else {
        return false;
    };
    let root_flipped = scene.nodes.first().is_some_and(|n| matches!(n.kind, SceneNodeKind::Container { is_flipped: true }));
    let design_h = scene.design_size[1];
    for node in &mut scene.nodes {
        if let Some(val) = dict.get(&node.title).and_then(plist::Value::as_string) {
            let new_title = val.replace('’', "'");
            if node.title != new_title {
                node.title = new_title;
                if matches!(node.kind, SceneNodeKind::Label) && node.value_binding.is_none() {
                    let pt = node.font_size_pt.unwrap_or(match node.control_size.as_str() {
                        "mini" | "Mini" => 9.0,
                        "small" | "Small" => 11.0,
                        _ => 13.0,
                    });
                    let old_cx = node.nib_frame[0] + node.nib_frame[2] * 0.5;
                    node.nib_frame[2] = keyed_archiver::measure_appkit_label_text_width(&node.title, pt) + 4.0;
                    if node.text_alignment == crate::scene::ir::MacTextAlignment::Center {
                        node.nib_frame[0] = (old_cx - node.nib_frame[2] * 0.5).max(0.0);
                    }
                    node.alignment_rect = normalizer::normalize_nib_alignment_rect(
                        &node.kind,
                        node.mac_control_size(),
                        node.nib_frame,
                        design_h,
                        root_flipped,
                    );
                }
            }
        }
    }
    for items in scene.button_menus.values_mut() {
        for (_, mi_title, _, _) in items {
            if let Some(val) = dict.get(mi_title).and_then(plist::Value::as_string) {
                *mi_title = val.replace('’', "'");
            }
        }
    }
    scene.solve_autolayout();
    scene.rebuild_search_index();
    true
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

pub(crate) fn decode_menu_bar_from_nib_path(path: &std::path::Path) -> Result<Vec<crate::navigation::NSMenu>, SceneCompileError> {
    if path.is_dir() {
        for candidate in ["keyedobjects-101300.nib", "keyedobjects.nib", "objects-11.0+.nib", "objects.nib"] {
            let sub = path.join(candidate);
            if sub.is_file() {
                let bytes = std::fs::read(&sub).map_err(|e| SceneCompileError(format!("Failed reading {}: {e}", sub.display())))?;
                return decode_menu_bar_from_nib_bytes(&bytes);
            }
        }
        Err(SceneCompileError(format!("No inner .nib stream found in bundle directory {}", path.display())))
    } else {
        let bytes = std::fs::read(path).map_err(|e| SceneCompileError(format!("Failed reading {}: {e}", path.display())))?;
        decode_menu_bar_from_nib_bytes(&bytes)
    }
}

pub(crate) fn decode_menu_bar_from_nib_bytes(bytes: &[u8]) -> Result<Vec<crate::navigation::NSMenu>, SceneCompileError> {
    if !bytes.starts_with(b"NIBArchive\x01\x00\x00\x00") {
        return Ok(Vec::new());
    }
    let g = nib_ibtool::IbNibGraph::parse(bytes)?;
    let mut item_actions = std::collections::BTreeMap::new();
    for obj_i in 0..g.objects.len() {
        if g.obj_class(obj_i) == "NSNibControlConnector" {
            let src = g.prop_val(obj_i, "NSSource").or_else(|| g.prop_val(obj_i, "NSObject")).and_then(nib_ibtool::IbVal::as_obj_ref);
            let sel = g.prop_val(obj_i, "NSLabel").and_then(|v| g.resolve_str(v));
            if let (Some(s), Some(l)) = (src, sel) {
                item_actions.insert(s, l);
            }
        }
    }
    let root_menu = (0..g.objects.len())
        .find(|&i| {
            g.obj_inherits(i, "NSMenu") && matches!(g.prop_val(i, "NSName").and_then(|v| g.resolve_str(v)).as_deref(), Some("_NSMainMenu"))
        })
        .or_else(|| {
            (0..g.objects.len()).filter(|&i| g.obj_inherits(i, "NSMenu")).max_by_key(|&i| {
                g.prop_val(i, "NSMenuItems").and_then(nib_ibtool::IbVal::as_obj_ref).map(|a| g.resolve_array_refs(a).len()).unwrap_or(0)
            })
        });
    let Some(main_idx) = root_menu else { return Ok(Vec::new()) };
    let top_items =
        g.prop_val(main_idx, "NSMenuItems").and_then(nib_ibtool::IbVal::as_obj_ref).map(|a| g.resolve_array_refs(a)).unwrap_or_default();
    let mut menus = Vec::new();
    for mi in top_items {
        if let Some(sub_idx) = g.prop_val(mi, "NSSubmenu").and_then(nib_ibtool::IbVal::as_obj_ref) {
            let title =
                g.prop_val(sub_idx, "NSTitle").or_else(|| g.prop_val(mi, "NSTitle")).and_then(|v| g.resolve_str(v)).unwrap_or_default();
            if !title.is_empty() {
                menus.push(build_nib_nsmenu(&g, sub_idx, &title, &item_actions, 0));
            }
        }
    }
    Ok(menus)
}

fn build_nib_nsmenu(
    g: &nib_ibtool::IbNibGraph,
    menu_idx: usize,
    title: &str,
    actions: &std::collections::BTreeMap<usize, String>,
    depth: usize,
) -> crate::navigation::NSMenu {
    use crate::navigation::{MacKeyboardShortcut, NSMenu, NSMenuItem};
    let mut items = Vec::new();
    if depth > 4 {
        return NSMenu::with_items(title, items);
    }
    let item_refs =
        g.prop_val(menu_idx, "NSMenuItems").and_then(nib_ibtool::IbVal::as_obj_ref).map(|a| g.resolve_array_refs(a)).unwrap_or_default();
    for mi in item_refs {
        let is_sep = g.prop_val(mi, "NSIsSeparator").and_then(nib_ibtool::IbVal::as_u32).unwrap_or(0) != 0;
        if is_sep {
            items.push(NSMenuItem::separator());
            continue;
        }
        let item_title = g.prop_val(mi, "NSTitle").and_then(|v| g.resolve_str(v)).unwrap_or_default();
        if let Some(sub_idx) = g.prop_val(mi, "NSSubmenu").and_then(nib_ibtool::IbVal::as_obj_ref) {
            let sub_title = if item_title.is_empty() {
                g.prop_val(sub_idx, "NSTitle").and_then(|v| g.resolve_str(v)).unwrap_or_default()
            } else {
                item_title.clone()
            };
            let sub = build_nib_nsmenu(g, sub_idx, &sub_title, actions, depth + 1);
            items.push(NSMenuItem::submenu(sub_title, sub));
            continue;
        }
        if item_title.is_empty() {
            continue;
        }
        let act_id = actions.get(&mi).cloned().unwrap_or_else(|| item_title.clone());
        let ke = g.prop_val(mi, "NSKeyEquiv").and_then(|v| g.resolve_str(v)).unwrap_or_default();
        let mod_mask = g.prop_val(mi, "NSKeyEquivModMask").and_then(nib_ibtool::IbVal::as_u32).unwrap_or(0x0010_0000);
        if let Some(key) = xib_strings::parse_nib_key_equiv(&ke) {
            let is_upper = ke.chars().next().is_some_and(|c| c.is_ascii_uppercase());
            let sc = MacKeyboardShortcut {
                key,
                command: (mod_mask & 0x0010_0000) != 0 || (mod_mask & 0x0004_0000) != 0,
                shift: (mod_mask & 0x0002_0000) != 0 || is_upper,
                alt: (mod_mask & 0x0008_0000) != 0,
            };
            items.push(NSMenuItem::action_with_shortcut(item_title, act_id, sc));
        } else {
            items.push(NSMenuItem::action(item_title, act_id));
        }
    }
    NSMenu::with_items(title, items)
}

pub(crate) fn wire_ibtool_connectors(st: &mut nib_ibtool::IbDecodeState<'_>) {
    use nib_ibtool::IbVal;
    for obj_i in 0..st.graph.objects.len() {
        let cls = st.graph.obj_class(obj_i);
        if cls == "NSIBHelpConnector" {
            let dst_obj =
                st.graph.prop_val(obj_i, "NSDestination").or_else(|| st.graph.prop_val(obj_i, "NSObject")).and_then(IbVal::as_obj_ref);
            if let Some(&n_idx) = dst_obj.and_then(|o| st.obj_to_node_idx.get(&o)) {
                if st.nodes[n_idx].tooltip.is_empty() {
                    if let Some(tip) = st.graph.prop_val(obj_i, "NSMarker").and_then(|v| st.graph.resolve_str(v)) {
                        st.nodes[n_idx].tooltip = tip;
                    }
                }
            }
            continue;
        }
        if cls == "NSNibOutletConnector" {
            nib_varint::wire_ibtool_outlet_and_menu(st, obj_i);
            continue;
        }
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
        } else if cls == "NSNibBindingConnector" {
            let bind_attr = st.graph.prop_val(obj_i, "NSBinding").and_then(|v| st.graph.resolve_str(v)).unwrap_or_default();
            let resolve_vt = |g: &nib_ibtool::IbNibGraph, oi: usize| -> Option<crate::scene::constraints::ValueTransformer> {
                g.prop_val(oi, "NSValueTransformerName")
                    .and_then(|v| g.resolve_str(v))
                    .or_else(|| {
                        g.prop_val(oi, "NSOptions")
                            .and_then(IbVal::as_obj_ref)
                            .and_then(|d| g.resolve_dict_str(d, "NSValueTransformerName"))
                    })
                    .map(|vt| match vt.as_str() {
                        "NSNegateBoolean" => crate::scene::constraints::ValueTransformer::NegateBoolean,
                        "NSIsNotNil" => crate::scene::constraints::ValueTransformer::IsNotNil,
                        "NSIsNil" => crate::scene::constraints::ValueTransformer::IsNil,
                        _ => crate::scene::constraints::ValueTransformer::Custom(vt),
                    })
            };
            if bind_attr == "hidden" {
                if let Some(raw_kp) = st.graph.prop_val(obj_i, "NSKeyPath").and_then(|v| st.graph.resolve_str(v)) {
                    let norm = xib_strings::flatten_controller_key_path(&raw_kp);
                    let bid = *st.binding_by_path.entry(norm.clone()).or_insert_with(|| {
                        let b = BindingId(st.next_binding);
                        st.next_binding += 1;
                        b
                    });
                    st.nodes[n_idx].hidden_binding = Some(bid);
                    st.nodes[n_idx].hidden_path = Some(norm);
                    if let Some(vt) = resolve_vt(st.graph, obj_i) {
                        st.nodes[n_idx].value_transformer = vt;
                    }
                }
                continue;
            }
            if bind_attr == "enabled" {
                if let Some(raw_kp) = st.graph.prop_val(obj_i, "NSKeyPath").and_then(|v| st.graph.resolve_str(v)) {
                    let norm = xib_strings::flatten_controller_key_path(&raw_kp);
                    let bid = *st.binding_by_path.entry(norm.clone()).or_insert_with(|| {
                        let b = BindingId(st.next_binding);
                        st.next_binding += 1;
                        b
                    });
                    st.nodes[n_idx].enabled_binding = Some(bid);
                    st.nodes[n_idx].enabled_path = Some(norm);
                    if let Some(vt) = resolve_vt(st.graph, obj_i) {
                        st.nodes[n_idx].enabled_transformer = vt;
                    }
                }
            }
            if matches!(bind_attr.as_str(), "minValue" | "maxValue") {
                continue;
            }
            if let Some(slot_num) = bind_attr.strip_prefix("displayPatternValue").and_then(|s| s.parse::<u8>().ok()) {
                if let Some(raw_kp) = st.graph.prop_val(obj_i, "NSKeyPath").and_then(|v| st.graph.resolve_str(v)) {
                    let norm = xib_strings::flatten_controller_key_path(&raw_kp);
                    let bid = *st.binding_by_path.entry(norm.clone()).or_insert_with(|| {
                        let b = BindingId(st.next_binding);
                        st.next_binding += 1;
                        b
                    });
                    if st.nodes[n_idx].display_pattern.is_none() {
                        st.nodes[n_idx].display_pattern = st
                            .graph
                            .prop_val(obj_i, "NSOptions")
                            .and_then(IbVal::as_obj_ref)
                            .and_then(|d| st.graph.resolve_dict_str(d, "NSDisplayPattern"));
                    }
                    if !st.nodes[n_idx].display_pattern_bindings.iter().any(|(s, _, _)| *s == slot_num) {
                        st.nodes[n_idx].display_pattern_bindings.push((slot_num, bid, norm.clone()));
                        st.nodes[n_idx].display_pattern_bindings.sort_by_key(|(s, _, _)| *s);
                    }
                    if slot_num == 1 {
                        st.nodes[n_idx].value_binding = Some(bid);
                        st.nodes[n_idx].binding_path = Some(norm);
                    }
                }
                continue;
            }
            let is_primary_val = matches!(
                bind_attr.as_str(),
                "value"
                    | "selectedValue"
                    | "selectedObject"
                    | "selectedIndex"
                    | "selectedTag"
                    | "doubleValue"
                    | "intValue"
                    | "stringValue"
            );
            if st.nodes[n_idx].binding_path.is_none() || is_primary_val {
                if let Some(raw_kp) = st.graph.prop_val(obj_i, "NSKeyPath").and_then(|v| st.graph.resolve_str(v)) {
                    let norm = xib_strings::flatten_controller_key_path(&raw_kp);
                    let bid = *st.binding_by_path.entry(norm.clone()).or_insert_with(|| {
                        let b = BindingId(st.next_binding);
                        st.next_binding += 1;
                        b
                    });
                    st.nodes[n_idx].value_binding = Some(bid);
                    st.nodes[n_idx].binding_path = Some(norm);
                    if let Some(vt) = resolve_vt(st.graph, obj_i) {
                        st.nodes[n_idx].value_transformer = vt;
                    }
                    if is_primary_val
                        && matches!(st.nodes[n_idx].kind, SceneNodeKind::Label)
                        && st.nodes[n_idx].title.is_empty()
                        && st.nodes[n_idx].nib_frame[3] >= 19.0
                    {
                        st.nodes[n_idx].kind = SceneNodeKind::TextField;
                    }
                }
            }
        }
    }
    for node in &mut st.nodes {
        if node.value_binding.is_none()
            && matches!(
                node.kind,
                SceneNodeKind::NumberField { .. } | SceneNodeKind::TextField | SceneNodeKind::TagColorPicker | SceneNodeKind::Slider { .. }
            )
        {
            if let Some(ref sel) = node.action_selector {
                let bind_key = if sel == "setDimension:" && !node.ib_object_id.is_empty() && !node.ib_object_id.starts_with("_NS:") {
                    format!("{sel}{}", node.ib_object_id)
                } else {
                    sel.clone()
                };
                let bid = *st.binding_by_path.entry(bind_key.clone()).or_insert_with(|| {
                    let b = BindingId(st.next_binding);
                    st.next_binding += 1;
                    b
                });
                node.value_binding = Some(bid);
                if node.binding_path.is_none() {
                    node.binding_path = Some(bind_key);
                }
            }
        }
    }
    nib_varint::normalize_percentage_stepper_increments(&mut st.nodes);
}
