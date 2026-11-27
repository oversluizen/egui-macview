//! Native Declarative `MacSceneSpec` Schema and `SceneExporter` (JSON & SwiftUI Transpiler, `SPEC-013 §2.5`, `§3.6`).

use crate::scene::ir::{MacSceneIR, NodeId, SceneCompileError, SceneNode, SceneNodeKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Human-readable, clean-room declarative scene specification (`SPEC-013 §2.5`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacSceneSpec {
    /// Schema version string (`"1.0"`).
    pub schema_version: String,
    /// Root scene node ID.
    pub root: NodeId,
    /// Design canvas dimensions `[width, height]`.
    pub design_size: [f32; 2],
    /// Lowered scene nodes.
    pub nodes: Vec<SceneNode>,
}

impl MacSceneSpec {
    /// Parses a `MacSceneSpec` from a JSON string.
    pub fn from_json_str(json: &str) -> Result<Self, SceneCompileError> {
        serde_json::from_str(json).map_err(|e| SceneCompileError(e.to_string()))
    }

    /// Converts this declarative `MacSceneSpec` into a normalized [`MacSceneIR`].
    pub fn into_scene_ir(self) -> MacSceneIR {
        MacSceneIR::from_nodes(self.root, self.design_size, self.nodes, crate::scene::constraints::CompiledConstraintSchedule::default())
    }
}

/// Exporter that transpiles a [`MacSceneIR`] into `MacSceneSpec` JSON or idiomatic SwiftUI source code (`REQ-013-02H`).
pub struct SceneExporter;

impl SceneExporter {
    /// Exports `scene` to a pretty-printed `MacSceneSpec` JSON string.
    pub fn to_json_string(scene: &MacSceneIR) -> Result<String, SceneCompileError> {
        let spec = MacSceneSpec {
            schema_version: "1.0".to_string(),
            root: scene.root,
            design_size: scene.design_size,
            nodes: scene.nodes.clone(),
        };
        serde_json::to_string_pretty(&spec).map_err(|e| SceneCompileError(e.to_string()))
    }

    /// Exports `scene` into an official Xcode 16.2 (`toolsVersion="23504"`) Cocoa `.xib` v3.0 XML document (`REQ-013-05.1`).
    pub fn to_xib_xml(scene: &MacSceneIR) -> String {
        crate::scene::xib_exporter::scene_to_xcode16_xib_xml(scene)
    }

    /// Decompiles and exports `scene` into clean, idiomatic SwiftUI source code using
    /// `SpatialSemanticNormalizer`'s `SemanticSectionGroup`s, `FieldRowGroup`s, and `LabeledFieldPair`s (`REQ-013-02H`).
    pub fn to_swiftui_source(scene: &MacSceneIR) -> String {
        let mut out = String::from("Form {\n");
        let mut emitted: BTreeSet<NodeId> = BTreeSet::new();
        for pair in &scene.labeled_pairs {
            emitted.insert(pair.label_node);
        }

        if scene.semantic_sections.is_empty() {
            out.push_str("    Section(\"General\") {\n");
            for node in &scene.nodes {
                if !emitted.contains(&node.id) {
                    emit_swiftui_node(scene, node, &mut out, 8);
                    emitted.insert(node.id);
                }
            }
            out.push_str("    }\n");
        } else {
            for sec in &scene.semantic_sections {
                out.push_str(&format!("    Section(\"{}\") {{\n", escape_swift(&sec.title)));
                for &nid in &sec.nodes {
                    if emitted.contains(&nid) {
                        continue;
                    }
                    if let Some(row) = scene.field_rows.iter().find(|r| r.nodes.contains(&nid)) {
                        let interactive_in_row: Vec<&SceneNode> = row
                            .nodes
                            .iter()
                            .filter(|&&id| !emitted.contains(&id))
                            .filter_map(|&id| scene.nodes.iter().find(|n| n.id == id))
                            .filter(|n| {
                                !matches!(n.kind, SceneNodeKind::Label | SceneNodeKind::Container { .. } | SceneNodeKind::Separator)
                            })
                            .collect();
                        if interactive_in_row.len() >= 2 {
                            out.push_str("        HStack {\n");
                            for rnode in interactive_in_row {
                                emit_swiftui_node(scene, rnode, &mut out, 12);
                                emitted.insert(rnode.id);
                            }
                            out.push_str("        }\n");
                            continue;
                        }
                    }
                    if let Some(node) = scene.nodes.iter().find(|n| n.id == nid) {
                        emit_swiftui_node(scene, node, &mut out, 8);
                        emitted.insert(node.id);
                    }
                }
                out.push_str("    }\n");
            }
        }
        out.push_str("}\n");
        out
    }
}

fn emit_swiftui_node(scene: &MacSceneIR, node: &SceneNode, out: &mut String, indent: usize) {
    if matches!(node.kind, SceneNodeKind::Container { .. } | SceneNodeKind::BoxContainer | SceneNodeKind::Separator) {
        return;
    }
    let pad = " ".repeat(indent);
    let binding_str = match (&node.binding_path, node.value_binding) {
        (Some(path), Some(bid)) => {
            let prefix = if path.contains('.') { "" } else { "model." };
            if bid.0 > 1 {
                format!("${prefix}{path}_b{}", bid.0)
            } else {
                format!("${prefix}{path}")
            }
        }
        (None, Some(bid)) => format!("$model.field_b{}", bid.0),
        _ => "$model.unbound".to_string(),
    };
    let [_, _, w, h] = node.alignment_rect;
    let modifiers = format!(
        ".controlSize(.{}){}{}.frame(width: {:.1}, height: {:.1})",
        node.control_size.to_lowercase(),
        if !node.enabled { ".disabled(true)" } else { "" },
        if !node.tooltip.is_empty() { format!(".help(\"{}\")", escape_swift(&node.tooltip)) } else { String::new() },
        w,
        h
    );

    let paired_label = node.labeled_by.and_then(|lid| scene.nodes.iter().find(|n| n.id == lid)).map(|l| l.title.as_str());

    let control_expr = match &node.kind {
        SceneNodeKind::Checkbox => {
            format!("Toggle(\"{}\", isOn: {}){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::RadioButton => {
            format!("Toggle(\"{}\", isOn: {}).toggleStyle(.radioGroup){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::Switch => {
            format!("Toggle(\"{}\", isOn: {}).toggleStyle(.switch){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::TextField | SceneNodeKind::NumberField { .. } => {
            format!("TextField(\"{}\", text: {}){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::PushButton { .. } | SceneNodeKind::HelpButton | SceneNodeKind::InlineActionButton => {
            let act = node.action_selector.as_deref().unwrap_or("performAction:");
            format!("Button(\"{}\", action: \"{}\"){}", escape_swift(&node.title), escape_swift(act), modifiers)
        }
        SceneNodeKind::PopUpButton { .. } => {
            format!("Picker(\"{}\", selection: {}){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::SegmentedControl { .. } => {
            format!("Picker(\"{}\", selection: {}).pickerStyle(.segmented){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::Slider { min, max, .. } => {
            if matches!(node.icon_name.as_deref(), Some("NSProgressBar" | "NSProgressSpinning")) {
                let style = if node.icon_name.as_deref() == Some("NSProgressSpinning") { ".progressViewStyle(.circular)" } else { "" };
                format!("ProgressView(\"{}\", value: {}){}{}", escape_swift(&node.title), binding_str, style, modifiers)
            } else if matches!(node.icon_name.as_deref(), Some("NSLevelIndicator" | "NSRatingIndicator")) {
                format!("Gauge(\"{}\", value: {}, in: {:.1}...{:.1}){}", escape_swift(&node.title), binding_str, min, max, modifiers)
            } else {
                format!("Slider(value: {}, in: {:.1}...{:.1}){}", binding_str, min, max, modifiers)
            }
        }
        SceneNodeKind::Stepper { step } => {
            format!("Stepper(\"{}\", value: {}, step: {:.1}){}", escape_swift(&node.title), binding_str, step, modifiers)
        }
        SceneNodeKind::ImageView { icon_name, .. } => {
            let sym = if !icon_name.is_empty() { icon_name.as_str() } else { node.title.as_str() };
            format!("Image(systemName: \"{}\"){}", escape_swift(sym), modifiers)
        }
        SceneNodeKind::TextView { .. } => {
            format!("TextEditor(text: {}){}", binding_str, modifiers)
        }
        SceneNodeKind::OriginMatrix => {
            format!("OriginPointPicker(selection: {}){}", binding_str, modifiers)
        }
        SceneNodeKind::ColorWell => {
            format!("ColorPicker(\"{}\", selection: {}){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::DatePicker { clock_and_calendar } => {
            let style = if *clock_and_calendar { ".datePickerStyle(.graphical)" } else { "" };
            format!("DatePicker(\"{}\", selection: {}){}{}", escape_swift(&node.title), binding_str, style, modifiers)
        }
        SceneNodeKind::TabView { .. } => {
            format!("TabView(\"{}\", selection: {}){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::DisclosureGroup { trailing_add_button } => {
            let add_mod = if *trailing_add_button { ".trailingAddButton(true)" } else { "" };
            format!("DisclosureGroup(\"{}\", isExpanded: {}){}{}", escape_swift(&node.title), binding_str, add_mod, modifiers)
        }
        SceneNodeKind::SearchField => {
            format!("SearchField(\"{}\", text: {}){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::ComboBox { .. } => {
            format!("ComboBox(\"{}\", text: {}){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::RuleEditor => {
            format!("RuleEditor(\"{}\", selection: {}){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::TableView { .. } => {
            format!("Table(\"{}\", selection: {}){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::TagColorPicker => {
            format!("TagColorPicker(selection: {}){}", binding_str, modifiers)
        }
        SceneNodeKind::TableActionBar => {
            format!("TableActionBar(selection: {}){}", binding_str, modifiers)
        }
        _ => {
            format!("Text(\"{}\"){}", escape_swift(&node.title), modifiers)
        }
    };

    if let Some(lbl) = paired_label {
        out.push_str(&format!("{pad}LabeledContent(\"{}\") {{ {control_expr} }}\n", escape_swift(lbl)));
    } else {
        out.push_str(&format!("{pad}{control_expr}\n"));
    }
}

fn escape_swift(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

impl crate::scene::ir::MacSceneIR {
    /// Updates `node.icon_name` during `awakeFromNib` (`REQ-013-10.1` §3.8.4).
    pub fn set_node_icon_name(&mut self, id_or_outlet: &str, icon_name: &str) -> bool {
        let Some(nid) = self.resolve_node_id(id_or_outlet) else { return false };
        let Some(node) = self.nodes.iter_mut().find(|n| n.id == nid) else { return false };
        node.icon_name = Some(icon_name.to_string());
        true
    }

    /// Assigns an explicit `binding_path` (and allocates a `value_binding` if absent) during `awakeFromNib`.
    pub fn set_node_binding_path(&mut self, id_or_outlet: &str, key_path: &str) -> bool {
        let Some(nid) = self.resolve_node_id(id_or_outlet) else { return false };
        let next_b = self.nodes.iter().filter_map(|n| n.value_binding).map(|b| b.0).max().unwrap_or(0) + 1;
        let Some(node) = self.nodes.iter_mut().find(|n| n.id == nid) else { return false };
        if node.value_binding.is_none() {
            node.value_binding = Some(crate::scene::ir::BindingId(next_b));
        }
        node.binding_path = Some(key_path.to_string());
        true
    }

    /// Updates `node.font_weight` and optional `node.font_name` during `awakeFromNib` (`REQ-013-10.1` §3.8.4).
    pub fn set_node_font_weight(&mut self, id_or_outlet: &str, weight: crate::controls::MacFontWeight, font_name: Option<&str>) -> bool {
        let Some(nid) = self.resolve_node_id(id_or_outlet) else { return false };
        let Some(node) = self.nodes.iter_mut().find(|n| n.id == nid) else { return false };
        node.font_weight = weight;
        if let Some(name) = font_name {
            node.font_name = Some(name.to_string());
        }
        true
    }

    /// Returns a mutable slice of all nodes in the scene arena for Stage-2 `initWithCoder:` / `awakeFromNib` class upgrades.
    pub fn nodes_mut(&mut self) -> &mut [SceneNode] {
        &mut self.nodes
    }
}

impl SceneNode {
    pub(crate) fn populate_xib_typed_fields(&mut self, elem: roxmltree::Node<'_, '_>, cell: Option<roxmltree::Node<'_, '_>>) {
        use crate::scene::ir::MacTextAlignment;
        let align_attr = cell.and_then(|c| c.attribute("alignment")).or_else(|| elem.attribute("alignment"));
        self.text_alignment = match align_attr {
            Some("right") => MacTextAlignment::Right,
            Some("center") => MacTextAlignment::Center,
            Some("justified") => MacTextAlignment::Justified,
            Some("natural") => MacTextAlignment::Natural,
            _ => MacTextAlignment::Left,
        };
        let bezel_attr = cell.and_then(|c| c.attribute("bezelStyle")).or_else(|| elem.attribute("bezelStyle"));
        self.bezel_style = match bezel_attr {
            Some("round" | "rounded" | "circular") => 1,
            Some("regularSquare") => 2,
            Some("disclosure") => 5,
            Some("shadowlessSquare") => 6,
            Some("texturedSquare") => 8,
            Some("helpButton") => 9,
            Some("smallSquare") => 10,
            Some("texturedRounded" | "recessed") => 11,
            Some("roundedDisclosure") => 12,
            Some("inline") => 15,
            _ => 0,
        };
        self.draws_background =
            cell.and_then(|c| c.attribute("drawsBackground")).or_else(|| elem.attribute("drawsBackground")) == Some("YES");
        self.is_bordered = cell.and_then(|c| c.attribute("bordered")).or_else(|| elem.attribute("bordered")) == Some("YES");
        self.is_bezeled =
            cell.and_then(|c| c.attribute("bezeled")).or_else(|| elem.attribute("bezeled")) == Some("YES") || self.bezel_style != 0;
        let default_ed = matches!(self.kind, SceneNodeKind::TextField | SceneNodeKind::NumberField { .. } | SceneNodeKind::SearchField);
        self.is_editable =
            cell.and_then(|c| c.attribute("editable")).or_else(|| elem.attribute("editable")).map(|v| v == "YES").unwrap_or(default_ed);
        self.is_selectable = cell
            .and_then(|c| c.attribute("selectable"))
            .or_else(|| elem.attribute("selectable"))
            .map(|v| v == "YES")
            .unwrap_or(self.is_editable);
        let font_elem = cell
            .and_then(|c| c.children().find(|ch| ch.has_tag_name("font")))
            .or_else(|| elem.children().find(|ch| ch.has_tag_name("font")));
        self.font_size_pt = font_elem.and_then(|fe| {
            fe.attribute("size").and_then(|s| s.parse::<f32>().ok()).or_else(|| match fe.attribute("metaFont") {
                Some("smallSystem") => Some(11.0),
                Some("miniSystem") => Some(9.0),
                Some(m) if m.starts_with("system") => Some(13.0),
                _ => None,
            })
        });
        let mfont = font_elem.and_then(|f| f.attribute("metaFont")).unwrap_or("");
        let fname = font_elem.and_then(|f| f.attribute("name")).or_else(|| (!mfont.is_empty()).then_some(mfont));
        self.font_name = fname.map(str::to_string);
        let w_str = elem.attribute("fontWeight").unwrap_or_else(|| fname.unwrap_or(mfont));
        self.font_weight = if w_str.contains("Semibold") || w_str.contains("semibold") {
            crate::controls::MacFontWeight::Semibold
        } else if w_str.contains("Bold") || w_str.contains("bold") {
            crate::controls::MacFontWeight::Bold
        } else if w_str.contains("Medium") || w_str.contains("medium") {
            crate::controls::MacFontWeight::Medium
        } else if w_str.contains("Light") || w_str.contains("light") {
            crate::controls::MacFontWeight::Light
        } else {
            crate::controls::MacFontWeight::Regular
        };
        let cname = cell.and_then(|c| c.children().find(|ch| ch.has_tag_name("color"))).and_then(|cl| cl.attribute("name")).unwrap_or("");
        let c_str = elem.attribute("textColor").unwrap_or(cname);
        self.label_color = if c_str.contains("secondary") {
            crate::controls::MacLabelColor::Secondary
        } else if c_str.contains("tertiary") {
            crate::controls::MacLabelColor::Tertiary
        } else {
            crate::controls::MacLabelColor::Primary
        };
        self.custom_class = elem.attribute("customClass").map(str::to_string);
        self.cell_class = cell.and_then(|c| c.attribute("customClass")).map(str::to_string);
        self.is_hidden = elem.attribute("hidden") == Some("YES");
        if elem.has_tag_name("stackView") || self.custom_class.as_deref() == Some("NSStackView") {
            self.stack_orientation = Some(if elem.attribute("orientation") == Some("vertical") { 1 } else { 0 });
            self.stack_spacing = elem.attribute("spacing").and_then(|s| s.parse::<f32>().ok()).unwrap_or(2.0);
            self.detaches_hidden_views = elem.attribute("detachesHiddenViews").map(|v| v == "YES").unwrap_or(true);
        }
        if elem.has_tag_name("visualEffectView") || self.custom_class.as_deref() == Some("NSVisualEffectView") {
            self.draws_background = true;
            self.custom_class.get_or_insert_with(|| "NSVisualEffectView".into());
            let mat_code: u8 = match elem.attribute("material") {
                Some("titlebar") => 3,
                Some("menu") => 5,
                Some("popover") => 6,
                Some("headerView") => 10,
                Some("sheet") => 11,
                Some("windowBackground") => 12,
                Some("hudWindow") => 13,
                Some("contentBackground") => 18,
                Some("underWindowBackground") => 21,
                Some("underPageBackground") => 22,
                _ => 7,
            };
            let behind_bit = if elem.attribute("blendingMode") == Some("behindWindow") { 0x80u8 } else { 0u8 };
            self.bezel_style = mat_code | behind_bit;
        }
    }

    pub(crate) fn populate_ibtool_typed_fields(
        &mut self,
        g: &crate::scene::nib_ibtool::IbNibGraph,
        obj_i: usize,
        cell_idx: Option<usize>,
        cflags: u32,
        font_pt: f64,
        raw_cls: &str,
        base_cls: &str,
        effective_cls: &str,
        custom_cls: Option<&str>,
    ) {
        use crate::scene::ir::MacTextAlignment;
        use crate::scene::nib_ibtool::IbVal;
        let vflags = g.prop_val(obj_i, "NSvFlags").and_then(IbVal::as_u32).unwrap_or(0);
        self.is_hidden = (vflags & 0x8000_0000) != 0;
        let cflags2 = cell_idx.and_then(|c| g.prop_val(c, "NSCellFlags2")).and_then(IbVal::as_u32).unwrap_or(0);
        let btn_flags2 = cell_idx.and_then(|c| g.prop_val(c, "NSButtonFlags2")).and_then(IbVal::as_u32).unwrap_or(0);
        let align_code = cell_idx
            .and_then(|c| g.prop_val(c, "NSAlignment"))
            .or_else(|| g.prop_val(obj_i, "NSAlignment"))
            .and_then(IbVal::as_u32)
            .unwrap_or_else(|| {
                let a2 = (cflags2 >> 26) & 0x07;
                if matches!(a2, 1..=4) {
                    a2
                } else {
                    (cflags >> 2) & 0x07
                }
            });
        self.text_alignment = match align_code {
            1 => MacTextAlignment::Right,
            2 => MacTextAlignment::Center,
            3 => MacTextAlignment::Justified,
            4 => MacTextAlignment::Natural,
            _ => MacTextAlignment::Left,
        };
        self.cell_class = cell_idx.and_then(|c| {
            g.prop_val(c, "NSClassName").and_then(|v| g.resolve_str(v)).or_else(|| {
                let rc = g.obj_class(c);
                (!rc.is_empty()).then(|| rc.to_string())
            })
        });
        let raw_bezel = cell_idx
            .and_then(|c| g.prop_val(c, "NSBezelStyle").or_else(|| g.prop_val(c, "NSTextBezelStyle")))
            .or_else(|| g.prop_val(obj_i, "NSBezelStyle"))
            .and_then(IbVal::as_u32)
            .unwrap_or(btn_flags2 & 0x1F) as u8;
        self.draws_background = cell_idx
            .and_then(|c| g.prop_val(c, "NSDrawsBackground"))
            .and_then(IbVal::as_u32)
            .map(|v| v != 0)
            .unwrap_or((cflags & 0x0400_0000) != 0);
        self.is_bordered = (cflags & 0x0080_0000) != 0;
        self.is_bezeled = (cflags & 0x0040_0000) != 0;
        self.is_editable = (cflags & 0x4008_0000) != 0;
        self.is_selectable = (cflags & 0x0004_0000) != 0 || self.is_editable;
        let state_on = !effective_cls.contains("MenuButton")
            && ((cflags & 0x8000_0000) != 0
                || cell_idx
                    .and_then(|c| g.prop_val(c, "NSContents").or_else(|| g.prop_val(c, "NSState")))
                    .is_some_and(|v| g.resolve_str(v).as_deref() == Some("1") || v.as_u32() == Some(1)));
        let alt_img = cell_idx.and_then(|c| g.prop_val(c, "NSAlternateImage")).and_then(|v| g.resolve_str(v)).unwrap_or_default();
        if !alt_img.is_empty() {
            self.alternate_icon_name = Some(alt_img.clone());
        }
        let highlight_on = state_on && (alt_img.ends_with("Highlight") || (alt_img.is_empty() && raw_bezel == 1));
        let base_bezel = if raw_bezel == 0 && !self.draws_background && self.is_bezeled { 1 } else { raw_bezel };
        self.bezel_style = if base_cls.contains("Button") && highlight_on { base_bezel | 0x80 } else { base_bezel };
        let font_ref = cell_idx
            .and_then(|c| g.prop_val(c, "NSSupport").or_else(|| g.prop_val(c, "NSFont")))
            .or_else(|| g.prop_val(obj_i, "NSSupport").or_else(|| g.prop_val(obj_i, "NSFont")));
        let font_name = font_ref.and_then(|v| g.resolve_str(v)).unwrap_or_default();
        let desc_wt = font_ref
            .and_then(IbVal::as_obj_ref)
            .and_then(|fi| g.prop_val(fi, "NSFontDescriptor").or_else(|| g.prop_val(fi, "NSFontDescriptorAttributes")))
            .and_then(IbVal::as_obj_ref)
            .and_then(|di| g.prop_val(di, "NSFontDescriptorAttributes").and_then(IbVal::as_obj_ref).or(Some(di)))
            .and_then(|attr_d| {
                g.resolve_dict_f64(attr_d, "NSFontWeight").or_else(|| {
                    g.resolve_array_refs(attr_d)
                        .into_iter()
                        .find_map(|sub| g.resolve_dict_f64(sub, "NSFontWeight").or_else(|| g.resolve_dict_f64(sub, "NSFontWeightTrait")))
                })
            });
        self.font_name = (!font_name.is_empty()).then(|| font_name.clone());
        self.font_weight =
            if font_name.contains("Demi") || font_name.contains("Semibold") || desc_wt.is_some_and(|w| (0.25..0.38).contains(&w)) {
                crate::controls::MacFontWeight::Semibold
            } else if font_name.contains("Bold") || desc_wt.is_some_and(|w| w >= 0.38) {
                crate::controls::MacFontWeight::Bold
            } else if font_name.contains("Medium") || desc_wt.is_some_and(|w| (0.15..0.25).contains(&w)) {
                crate::controls::MacFontWeight::Medium
            } else if font_name.contains("Light") || font_name.contains("Thin") || desc_wt.is_some_and(|w| w <= -0.2) {
                crate::controls::MacFontWeight::Light
            } else {
                crate::controls::MacFontWeight::Regular
            };
        let color_name = cell_idx.and_then(|c| g.prop_val(c, "NSTextColor")).and_then(|v| g.resolve_str(v)).unwrap_or_default();
        self.label_color = if color_name.contains("secondary") {
            crate::controls::MacLabelColor::Secondary
        } else if color_name.contains("tertiary") {
            crate::controls::MacLabelColor::Tertiary
        } else {
            crate::controls::MacLabelColor::Primary
        };
        self.font_size_pt = (font_pt > 0.0).then_some(font_pt as f32);
        self.custom_class = custom_cls.map(str::to_string).or_else(|| (raw_cls != base_cls).then(|| raw_cls.to_string()));
        self.image_scaling = cell_idx
            .and_then(|c| g.prop_val(c, "NSScale").or_else(|| g.prop_val(c, "NSImageScaling")))
            .or_else(|| g.prop_val(obj_i, "NSScale").or_else(|| g.prop_val(obj_i, "NSImageScaling")))
            .and_then(IbVal::as_u32)
            .unwrap_or(0) as u8;
        self.symbol_point_size = g
            .prop_val(obj_i, "IBNSShadowedSymbolConfiguration")
            .or_else(|| cell_idx.and_then(|c| g.prop_val(c, "IBNSShadowedSymbolConfiguration").or_else(|| g.prop_val(c, "NSSymbolConfiguration"))))
            .and_then(IbVal::as_obj_ref)
            .and_then(|sc| g.prop_val(sc, "NSPointSize").or_else(|| g.prop_val(sc, "NSSize")))
            .and_then(|v| g.resolve_f64(v))
            .filter(|&pt| pt > 0.0)
            .map(|pt| pt as f32);
        if g.obj_inherits(obj_i, "StackView") || base_cls.contains("StackView") || effective_cls.contains("StackView") {
            self.stack_orientation = Some(g.prop_val(obj_i, "NSStackViewOrientation").and_then(IbVal::as_u32).unwrap_or(0) as u8);
            self.stack_spacing = g.prop_val(obj_i, "NSStackViewSpacing").and_then(|v| g.resolve_f64(v)).unwrap_or(2.0) as f32;
            self.detaches_hidden_views =
                g.prop_val(obj_i, "NSStackViewDetachesHiddenViews").and_then(IbVal::as_u32).map(|v| v != 0).unwrap_or(true);
        }
        if g.obj_inherits(obj_i, "VisualEffectView") || base_cls.contains("VisualEffectView") || effective_cls.contains("VisualEffectView")
        {
            self.draws_background = true;
            self.custom_class.get_or_insert_with(|| "NSVisualEffectView".into());
            let mat = g
                .prop_val(obj_i, "NSVisualEffectViewMaterial")
                .or_else(|| g.prop_val(obj_i, "IBVisualEffectViewExternalMaterial"))
                .or_else(|| g.prop_val(obj_i, "NSMaterial"))
                .and_then(IbVal::as_u32)
                .unwrap_or(7);
            let blending_opt =
                g.prop_val(obj_i, "NSVisualEffectViewBlendingMode").or_else(|| g.prop_val(obj_i, "NSBlendingMode")).and_then(IbVal::as_u32);
            let is_behind = match blending_opt {
                Some(b) => b == 0,
                None => mat == 7 || mat == 21,
            };
            self.bezel_style = ((mat as u8).max(1) & 0x7F) | if is_behind { 0x80 } else { 0 };
        }
    }
}
