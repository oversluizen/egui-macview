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
        SceneNodeKind::Switch => {
            format!("Toggle(\"{}\", isOn: {}).toggleStyle(.switch){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::TextField | SceneNodeKind::NumberField { .. } => {
            format!("TextField(\"{}\", text: {}){}", escape_swift(&node.title), binding_str, modifiers)
        }
        SceneNodeKind::PushButton { .. } => {
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
            format!("Slider(value: {}, in: {:.1}...{:.1}){}", binding_str, min, max, modifiers)
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
