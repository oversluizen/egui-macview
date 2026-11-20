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
pub mod storyboard;
pub mod swiftui;
pub mod xib_exporter;
pub mod xib_strings;

pub use constraints::{
    evaluate_arithmetic_expr, CompiledConstraintSchedule, ConstraintAttr, ConstraintRelation, LayoutConstraintSpec, ValueTransformer,
};
pub use exporter::{MacSceneSpec, SceneExporter};
pub use ir::{
    ActionId, BindingId, BindingValueType, ContractAuditReport, CustomClassMapper, FieldRowGroup, HostContractSchema, LabeledFieldPair,
    LayoutPolicy, MacSceneIR, NodeId, SceneCompileError, SceneFilterMask, SceneNode, SceneNodeKind, SemanticSectionGroup, VisibilityRule,
};
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
    /// Applies a localized [`StringsCatalog`] to all matching nodes in $O(N)$ time and rebuilds [`MacSceneIR::rebuild_search_index`].
    pub fn apply_strings_catalog(&mut self, catalog: &StringsCatalog) {
        for node in &mut self.nodes {
            if let Some(title) = catalog.get(&node.ib_object_id, "title") {
                node.title = title.to_string();
            }
            if let Some(tip) = catalog.get(&node.ib_object_id, "ibShadowedToolTip") {
                node.tooltip = tip.to_string();
            }
        }
        self.rebuild_search_index();
    }
}

/// Resolves and loads a `.nib` file or `.nib` directory bundle (`keyedobjects-101300.nib` / `keyedobjects.nib` / `objects-11.0+.nib`),
/// automatically applying a companion `../en.lproj/<Stem>.strings` or `./<Stem>.strings` catalog if present.
pub(crate) fn load_nib_from_path(path: &std::path::Path, mapper: &CustomClassMapper) -> Result<MacSceneIR, SceneCompileError> {
    let mut scene = if path.is_dir() {
        let mut loaded = None;
        for candidate in ["keyedobjects-101300.nib", "keyedobjects.nib", "objects-11.0+.nib", "objects.nib"] {
            let sub = path.join(candidate);
            if sub.is_file() {
                let bytes = std::fs::read(&sub).map_err(|e| SceneCompileError(format!("Failed reading {}: {e}", sub.display())))?;
                loaded = Some(NibArchiveFrontend::from_bytes(&bytes, mapper)?);
                break;
            }
        }
        loaded.ok_or_else(|| SceneCompileError(format!("No inner .nib stream found in bundle directory {}", path.display())))?
    } else {
        let bytes = std::fs::read(path).map_err(|e| SceneCompileError(format!("Failed reading {}: {e}", path.display())))?;
        NibArchiveFrontend::from_bytes(&bytes, mapper)?
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
                        if let Ok(plist::Value::Dictionary(dict)) = plist::from_bytes::<plist::Value>(&sbytes) {
                            for node in &mut scene.nodes {
                                if let Some(val) = dict.get(&node.title).and_then(plist::Value::as_string) {
                                    node.title = val.replace('’', "'");
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
    Ok(scene)
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
    let root_menu = (0..g.objects.len()).find(|&i| {
        g.obj_inherits(i, "NSMenu")
            && matches!(g.prop_val(i, "NSName").and_then(|v| g.resolve_str(v)).as_deref(), Some("_NSMainMenu"))
    }).or_else(|| {
        (0..g.objects.len())
            .filter(|&i| g.obj_inherits(i, "NSMenu"))
            .max_by_key(|&i| g.prop_val(i, "NSMenuItems").and_then(nib_ibtool::IbVal::as_obj_ref).map(|a| g.resolve_array_refs(a).len()).unwrap_or(0))
    });
    let Some(main_idx) = root_menu else { return Ok(Vec::new()) };
    let top_items = g.prop_val(main_idx, "NSMenuItems").and_then(nib_ibtool::IbVal::as_obj_ref).map(|a| g.resolve_array_refs(a)).unwrap_or_default();
    let mut menus = Vec::new();
    for mi in top_items {
        if let Some(sub_idx) = g.prop_val(mi, "NSSubmenu").and_then(nib_ibtool::IbVal::as_obj_ref) {
            let title = g
                .prop_val(sub_idx, "NSTitle")
                .or_else(|| g.prop_val(mi, "NSTitle"))
                .and_then(|v| g.resolve_str(v))
                .unwrap_or_default();
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
    let item_refs = g.prop_val(menu_idx, "NSMenuItems").and_then(nib_ibtool::IbVal::as_obj_ref).map(|a| g.resolve_array_refs(a)).unwrap_or_default();
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
        if let Some(key) = parse_nib_key_equiv(&ke) {
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

fn parse_nib_key_equiv(ke: &str) -> Option<egui::Key> {
    let ch = ke.chars().next()?.to_ascii_uppercase();
    egui::Key::from_name(&ch.to_string()).or(match ch {
        '+' | '=' => Some(egui::Key::Plus),
        '-' => Some(egui::Key::Minus),
        ',' => Some(egui::Key::Comma),
        '.' => Some(egui::Key::Period),
        '/' => Some(egui::Key::Slash),
        '\r' | '\n' => Some(egui::Key::Enter),
        '\u{8}' | '\u{7f}' => Some(egui::Key::Backspace),
        ' ' => Some(egui::Key::Space),
        _ => None,
    })
}
