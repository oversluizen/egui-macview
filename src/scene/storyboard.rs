//! Official Xcode 16.2 Cocoa `.storyboard` Multi-Scene & `<segue>` Frontend and Exporter (`REQ-013-05.3`).
//!
//! Parses and exports `<document type="com.apple.InterfaceBuilder3.Cocoa.Storyboard.XIB" version="3.0" toolsVersion="23504">`,
//! preserving multiple `<scene sceneID="...">` blocks, `<point key="canvasLocation" x="..." y="..."/>` 2D canvas positions,
//! and `<segue destination="..." kind="relationship|show|sheet|modal|popover" .../>` connections.

use crate::scene::ir::{MacSceneIR, SceneCompileError};
use crate::scene::xib_exporter::{emit_root_view_xml, escape_xml};
use crate::scene::xib_strings::XibXmlFrontend;
use serde::{Deserialize, Serialize};

/// Transition kind for a Cocoa Storyboard `<segue>` connection (`REQ-013-05.3`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegueKind {
    /// Containment relationship (`kind="relationship"`, e.g. `window.shadowedContentViewController`).
    Relationship,
    /// Primary content push/show transition (`kind="show"`).
    Show,
    /// Attached document/window sheet transition (`kind="sheet"`).
    Sheet,
    /// Modal dialog presentation (`kind="modal"`).
    Modal,
    /// Anchored popover presentation (`kind="popover"`).
    Popover,
    /// Custom `NSStoryboardSegue` subclass (`kind="custom"`).
    Custom(String),
}

impl SegueKind {
    /// Parses a Cocoa Storyboard `<segue kind="...">` attribute string.
    pub fn from_attr(raw: &str) -> Self {
        match raw {
            "relationship" => Self::Relationship,
            "show" => Self::Show,
            "sheet" => Self::Sheet,
            "modal" => Self::Modal,
            "popover" => Self::Popover,
            other => Self::Custom(other.to_string()),
        }
    }

    /// Returns the official Cocoa Storyboard XML attribute value for this segue kind.
    pub fn as_attr(&self) -> &str {
        match self {
            Self::Relationship => "relationship",
            Self::Show => "show",
            Self::Sheet => "sheet",
            Self::Modal => "modal",
            Self::Popover => "popover",
            Self::Custom(s) => s.as_str(),
        }
    }
}

/// Directed `<segue>` connection between two Storyboard controllers (`REQ-013-05.3`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryboardSegue {
    /// Interface Builder segue identifier (`id="..."`).
    pub id: String,
    /// Optional runtime segue identifier (`identifier="..."`).
    pub identifier: Option<String>,
    /// Source object ID (controller or button `id` that triggers the segue).
    pub source_object_id: String,
    /// Destination controller ID (`destination="..."`).
    pub destination: String,
    /// Segue presentation kind.
    pub kind: SegueKind,
    /// Optional relationship key path (e.g., `"window.shadowedContentViewController"`).
    pub relationship: Option<String>,
    /// Optional anchor view ID for popover segues (`popoverAnchorView="..."`).
    pub anchor_view: Option<String>,
}

/// Single `<scene sceneID="...">` artboard inside a [`MacStoryboardIR`] (`REQ-013-05.3`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoryboardScene {
    /// Scene identifier (`sceneID="..."`).
    pub scene_id: String,
    /// Primary controller ID (`<windowController id="...">` or `<viewController id="...">`).
    pub controller_id: String,
    /// Controller XML tag (`"windowController"` or `"viewController"`).
    pub controller_kind: String,
    /// Human-readable controller or window title.
    pub title: String,
    /// 2D artboard coordinates on the Storyboard canvas (`<point key="canvasLocation" x="..." y="..."/>`).
    pub canvas_location: [f32; 2],
    /// Lowered [`MacSceneIR`] for the scene's `<window>` or `<view key="view">`.
    pub scene_ir: MacSceneIR,
    /// Outgoing `<segue>` transitions originating inside this scene.
    pub segues: Vec<StoryboardSegue>,
}

/// Multi-scene Xcode Cocoa `.storyboard` Intermediate Representation (`REQ-013-05.3`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacStoryboardIR {
    /// Xcode tools version (`"23504"` for Xcode 16.2).
    pub tools_version: String,
    /// Entry controller ID (`initialViewController="..."`).
    pub initial_view_controller: Option<String>,
    /// Ordered Storyboard scenes (`<scenes><scene .../></scenes>`).
    pub scenes: Vec<StoryboardScene>,
}

/// Xcode Interface Builder Cocoa `.storyboard` parser and exporter (`REQ-013-05.3`).
pub struct StoryboardFrontend;

impl StoryboardFrontend {
    /// Parses an Xcode Cocoa `.storyboard` XML string into a [`MacStoryboardIR`].
    pub fn from_xml_str(xml: &str) -> Result<MacStoryboardIR, SceneCompileError> {
        let doc = roxmltree::Document::parse(xml).map_err(|e| SceneCompileError(format!("Invalid Storyboard XML: {e}")))?;
        let root = doc.root_element();
        if !root.has_tag_name("document") {
            return Err(SceneCompileError("Missing <document> root in Storyboard XML".into()));
        }
        let tools_version = root.attribute("toolsVersion").unwrap_or("23504").to_string();
        let initial_view_controller = root.attribute("initialViewController").map(str::to_string);

        let mut scenes = Vec::new();
        for scene_node in root.descendants().filter(|n| n.is_element() && n.has_tag_name("scene")) {
            let scene_id = scene_node.attribute("sceneID").unwrap_or("scene-0").to_string();
            let canvas_location = scene_node
                .children()
                .find(|c| c.has_tag_name("point") && c.attribute("key") == Some("canvasLocation"))
                .map(|pt| {
                    let x = pt.attribute("x").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                    let y = pt.attribute("y").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                    [x, y]
                })
                .unwrap_or([0.0, 0.0]);

            let ctrl_elem = scene_node.descendants().find(|n| {
                n.is_element()
                    && matches!(n.tag_name().name(), "windowController" | "viewController" | "splitViewController" | "tabViewController")
            });
            let controller_kind = ctrl_elem.map(|c| c.tag_name().name().to_string()).unwrap_or_else(|| "viewController".to_string());
            let controller_id = ctrl_elem.and_then(|c| c.attribute("id")).unwrap_or("vc-0").to_string();
            let title = ctrl_elem
                .and_then(|c| c.attribute("title"))
                .or_else(|| scene_node.descendants().find(|d| d.has_tag_name("window")).and_then(|w| w.attribute("title")))
                .unwrap_or(&scene_id)
                .to_string();

            let mut segues = Vec::new();
            for seg in scene_node.descendants().filter(|d| d.is_element() && d.has_tag_name("segue")) {
                let src_id =
                    seg.parent().and_then(|conns| conns.parent()).and_then(|src| src.attribute("id")).unwrap_or(&controller_id).to_string();
                segues.push(StoryboardSegue {
                    id: seg.attribute("id").unwrap_or("seg-0").to_string(),
                    identifier: seg.attribute("identifier").map(str::to_string),
                    source_object_id: src_id,
                    destination: seg.attribute("destination").unwrap_or("").to_string(),
                    kind: SegueKind::from_attr(seg.attribute("kind").unwrap_or("show")),
                    relationship: seg.attribute("relationship").map(str::to_string),
                    anchor_view: seg.attribute("popoverAnchorView").map(str::to_string),
                });
            }

            let scene_slice = &xml[scene_node.range()];
            let wrapped_xib = wrap_storyboard_scene_as_xib(scene_slice, &title);
            let scene_ir = XibXmlFrontend::from_xml_str(&wrapped_xib)?;

            scenes.push(StoryboardScene { scene_id, controller_id, controller_kind, title, canvas_location, scene_ir, segues });
        }

        if scenes.is_empty() {
            return Err(SceneCompileError("Storyboard contains zero <scene> blocks".into()));
        }
        Ok(MacStoryboardIR { tools_version, initial_view_controller, scenes })
    }

    /// Exports a [`MacStoryboardIR`] into an Xcode 16.2 (`toolsVersion="23504"`) Cocoa `.storyboard` XML document.
    pub fn to_xml_str(storyboard: &MacStoryboardIR) -> String {
        let mut out = String::with_capacity(8192);
        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        let init_attr = storyboard
            .initial_view_controller
            .as_deref()
            .map(|id| format!(" initialViewController=\"{}\"", escape_xml(id)))
            .unwrap_or_default();
        out.push_str(&format!(
            "<document type=\"com.apple.InterfaceBuilder3.Cocoa.Storyboard.XIB\" version=\"3.0\" toolsVersion=\"23504\" targetRuntime=\"MacOSX.Cocoa\" propertyAccessControl=\"none\" useAutolayout=\"NO\"{init_attr}>\n"
        ));
        out.push_str("    <dependencies>\n");
        out.push_str("        <plugIn identifier=\"com.apple.InterfaceBuilder.CocoaPlugin\" version=\"23504\"/>\n");
        out.push_str("        <capability name=\"documents saved in the Xcode 8 format\" minToolsVersion=\"8.0\"/>\n");
        out.push_str("    </dependencies>\n");
        out.push_str("    <scenes>\n");

        for sc in &storyboard.scenes {
            let sid = escape_xml(&sc.scene_id);
            let cid = escape_xml(&sc.controller_id);
            let ctitle = escape_xml(&sc.title);
            let [w, h] = sc.scene_ir.design_size;
            let root_id = sc.scene_ir.nodes.first().map(|n| escape_xml(&n.ib_object_id)).unwrap_or_else(|| format!("view-{cid}"));
            out.push_str(&format!("        <scene sceneID=\"{sid}\">\n"));
            out.push_str("            <objects>\n");

            if sc.controller_kind == "windowController" {
                out.push_str(&format!("                <windowController id=\"{cid}\" sceneMemberID=\"viewController\">\n"));
                out.push_str(&format!(
                    "                    <window key=\"window\" title=\"{ctitle}\" allowsToolTipsWhenApplicationIsInactive=\"NO\" autorecalculatesKeyViewLoop=\"NO\" releasedWhenClosed=\"NO\" visibleAtLaunch=\"NO\" animationBehavior=\"default\" id=\"win-{cid}\">\n"
                ));
                out.push_str("                        <windowStyleMask key=\"styleMask\" titled=\"YES\" closable=\"YES\" miniaturizable=\"YES\" resizable=\"YES\"/>\n");
                out.push_str(&format!(
                    "                        <rect key=\"contentRect\" x=\"196\" y=\"240\" width=\"{w:.0}\" height=\"{h:.0}\"/>\n"
                ));
                out.push_str("                        <rect key=\"screenRect\" x=\"0.0\" y=\"0.0\" width=\"1680\" height=\"1027\"/>\n");
                emit_root_view_xml(&sc.scene_ir, &root_id, [w, h], false, "                        ", &mut out);
                out.push_str("                    </window>\n");
                emit_storyboard_segues(&sc.segues, "                    ", &mut out);
                out.push_str("                </windowController>\n");
            } else {
                out.push_str(&format!(
                    "                <viewController title=\"{ctitle}\" id=\"{cid}\" sceneMemberID=\"viewController\">\n"
                ));
                emit_root_view_xml(&sc.scene_ir, &root_id, [w, h], false, "                    ", &mut out);
                emit_storyboard_segues(&sc.segues, "                    ", &mut out);
                out.push_str("                </viewController>\n");
            }

            out.push_str(&format!(
                "                <customObject id=\"fr-{cid}\" userLabel=\"First Responder\" customClass=\"NSResponder\" sceneMemberID=\"firstResponder\"/>\n"
            ));
            out.push_str("            </objects>\n");
            out.push_str(&format!(
                "            <point key=\"canvasLocation\" x=\"{:.0}\" y=\"{:.0}\"/>\n",
                sc.canvas_location[0], sc.canvas_location[1]
            ));
            out.push_str("        </scene>\n");
        }

        out.push_str("    </scenes>\n");
        out.push_str("</document>\n");
        out
    }
}

fn emit_storyboard_segues(segues: &[StoryboardSegue], indent: &str, out: &mut String) {
    if segues.is_empty() {
        return;
    }
    out.push_str(&format!("{indent}<connections>\n"));
    for sg in segues {
        let id_attr = sg.identifier.as_deref().map(|i| format!(" identifier=\"{}\"", escape_xml(i))).unwrap_or_default();
        let rel_attr = sg.relationship.as_deref().map(|r| format!(" relationship=\"{}\"", escape_xml(r))).unwrap_or_default();
        let anc_attr = sg
            .anchor_view
            .as_deref()
            .map(|a| format!(" popoverAnchorView=\"{}\" popoverBehavior=\"t\" preferredEdge=\"maxX\"", escape_xml(a)))
            .unwrap_or_default();
        out.push_str(&format!(
            "{indent}    <segue destination=\"{}\" kind=\"{}\"{id_attr}{rel_attr}{anc_attr} id=\"{}\"/>\n",
            escape_xml(&sg.destination),
            escape_xml(sg.kind.as_attr()),
            escape_xml(&sg.id)
        ));
    }
    out.push_str(&format!("{indent}</connections>\n"));
}

fn wrap_storyboard_scene_as_xib(scene_xml: &str, title: &str) -> String {
    if scene_xml.contains("<window ") {
        format!("<document type=\"com.apple.InterfaceBuilder3.Cocoa.XIB\" version=\"3.0\">{scene_xml}</document>")
    } else {
        let converted = scene_xml.replacen("key=\"view\"", "key=\"contentView\"", 1);
        format!(
            "<document type=\"com.apple.InterfaceBuilder3.Cocoa.XIB\" version=\"3.0\"><objects><window title=\"{}\">{converted}</window></objects></document>",
            escape_xml(title)
        )
    }
}

impl MacSceneIR {
    /// Returns the ordered set of all interactive `BindingId`s in the scene.
    pub fn interactive_binding_ids(&self) -> Vec<crate::scene::ir::BindingId> {
        let mut set = std::collections::BTreeSet::new();
        for n in &self.nodes {
            if let Some(b) = n.value_binding {
                set.insert(b);
            }
        }
        set.into_iter().collect()
    }

    /// Dynamically mounts a child `MacSceneIR` under `parent_or_slot` (`addSubview:` equivalent).
    pub fn mount_subscene(&mut self, parent_or_slot: crate::scene::ir::NodeId, child_scene: &MacSceneIR) {
        use crate::scene::ir::NodeId;
        let id_offset = self.nodes.iter().map(|n| n.id.0).max().unwrap_or(0) + 100;
        let child_root_new = NodeId(child_scene.root.0 + id_offset);
        if let Some(p) = self.nodes.iter_mut().find(|n| n.id == parent_or_slot) {
            p.children.push(child_root_new);
        }
        for cn in &child_scene.nodes {
            let mut cloned = cn.clone();
            cloned.id = NodeId(cn.id.0 + id_offset);
            cloned.parent = cn.parent.map(|p| NodeId(p.0 + id_offset)).or(Some(parent_or_slot));
            cloned.children = cn.children.iter().map(|c| NodeId(c.0 + id_offset)).collect();
            self.nodes.push(cloned);
        }
        self.rebuild_search_index();
    }
}

pub(crate) fn resolve_horizontal_stack_views(scene: &mut MacSceneIR) {
    if scene.nodes.first().and_then(|n| n.stack_orientation) == Some(1) {
        return;
    }
    let h_stacks: Vec<(usize, f32, f32, bool, Vec<crate::scene::ir::NodeId>)> = scene.nodes.iter().enumerate()
        .filter(|&(i, n)| {
            i > 0 && n.stack_orientation == Some(0) && n.children.len() >= 2
                && n.children.iter().filter_map(|cid| scene.nodes.iter().find(|cn| cn.id == *cid))
                    .filter(|cn| !cn.is_hidden && (cn.nib_frame[0] - n.nib_frame[0]).abs() <= 0.5).count() >= 2
        })
        .map(|(i, n)| (i, n.nib_frame[0], n.nib_frame[2], n.detaches_hidden_views, n.children.clone()))
        .collect();
    for (st_i, st_x, st_w, detaches, children) in h_stacks {
        let base_x = if scene.nodes[st_i].parent.is_some() { st_x } else { 0.0 };
        let vis_indices: Vec<usize> = children.iter().filter_map(|cid| scene.nodes.iter().position(|n| n.id == *cid))
            .filter(|&ci| !(detaches && scene.nodes[ci].is_hidden)).collect();
        if vis_indices.len() >= 2 {
            let sum_w: f32 = vis_indices.iter().map(|&ci| scene.nodes[ci].nib_frame[2]).sum();
            let sp = if scene.nodes[st_i].stack_spacing > 0.0 { scene.nodes[st_i].stack_spacing }
                else if st_w > sum_w { (st_w - sum_w) / ((vis_indices.len() - 1) as f32) } else { 2.0 };
            let mut cur_x = base_x;
            for ci in vis_indices {
                scene.nodes[ci].nib_frame[0] = cur_x.round();
                scene.nodes[ci].alignment_rect =
                    crate::scene::normalizer::normalize_nib_alignment_rect(&scene.nodes[ci].kind, scene.nodes[ci].mac_control_size(), scene.nodes[ci].nib_frame, scene.design_size[1], false);
                cur_x += scene.nodes[ci].nib_frame[2] + sp;
            }
        }
    }
}

pub(crate) fn solve_scene_autolayout_2d(scene: &mut MacSceneIR) {
    use crate::scene::constraints::{ConstraintAttr, ConstraintRelation};
    use crate::scene::ir::{NodeId, SceneNode, SceneNodeKind};
    use crate::scene::normalizer::normalize_nib_alignment_rect;
    use std::collections::BTreeMap;

    if scene.nib_constraints.is_empty()
        && scene.nodes.first().and_then(|n| n.stack_orientation).is_none()
        && !scene.nodes.iter().skip(1).any(|n| n.stack_orientation == Some(0))
    {
        return;
    }

    let root_w = scene.design_size[0].max(1.0);
    let is_stack_root = scene.nodes.first().and_then(|n| n.stack_orientation).is_some();
    let id_to_idx: BTreeMap<NodeId, usize> = scene.nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
    let is_super = |child: NodeId, other: NodeId, nodes: &[SceneNode]| -> bool {
        id_to_idx.get(&child).and_then(|&ci| nodes[ci].parent) == Some(other)
    };
    let h_insets = |node: &SceneNode| -> (f32, f32) {
        if matches!(node.kind, SceneNodeKind::Label)
            && !is_stack_root
            && !scene.nib_constraints.iter().any(|c| c.first_item == Some(node.id))
        {
            return (0.0, 0.0);
        }
        let [_, l, _, r] = node.node_alignment_rect_insets();
        (l, r)
    };
    let align_x = |node: &SceneNode| node.nib_frame[0] + h_insets(node).0;
    let align_w = |node: &SceneNode| (node.nib_frame[2] - h_insets(node).0 - h_insets(node).1).max(0.0);
    let set_ax = |node: &mut SceneNode, ax: f32| node.nib_frame[0] = (ax - h_insets(node).0).max(0.0);
    let set_aw = |node: &mut SceneNode, aw: f32| {
        let (l, r) = h_insets(node);
        node.nib_frame[2] = aw.max(0.0) + l + r;
    };

    if is_stack_root {
        let old_w = scene.nodes.first().map(|n| n.nib_frame[2]).unwrap_or(root_w);
        for n in &mut scene.nodes {
            let spans_full_w = n.nib_frame[0].abs() <= 0.5 && (n.nib_frame[2] - old_w).abs() <= 1.0;
            if matches!(n.kind, SceneNodeKind::Container { .. } | SceneNodeKind::TabView { .. })
                || n.stack_orientation.is_some()
                || spans_full_w
            {
                n.nib_frame[0] = 0.0;
                n.nib_frame[2] = root_w;
            }
        }
    }

    let mut width_consts: BTreeMap<NodeId, f32> = BTreeMap::new();
    let mut equal_width_pairs: Vec<(NodeId, NodeId)> = Vec::new();
    let mut centerx_consts: BTreeMap<NodeId, (NodeId, f32)> = BTreeMap::new();
    let mut leading_consts: BTreeMap<NodeId, (Option<NodeId>, f32)> = BTreeMap::new();
    let mut trailing_consts: BTreeMap<NodeId, f32> = BTreeMap::new();
    let mut equal_lead_pairs: Vec<(NodeId, NodeId)> = Vec::new();
    let mut equal_trail_pairs: Vec<(NodeId, NodeId)> = Vec::new();

    for c in &scene.nib_constraints {
        let Some(f_id) = c.first_item else { continue };
        let s_opt = c.second_item;
        let (a1, a2, c_val) = (c.first_attr, c.second_attr, c.constant);
        if a1 == ConstraintAttr::Width && s_opt.is_none() && c.relation == ConstraintRelation::Equal {
            width_consts.insert(f_id, c_val);
        } else if a1 == ConstraintAttr::Width && a2 == ConstraintAttr::Width && c.relation == ConstraintRelation::Equal {
            if let Some(s_id) = s_opt.filter(|_| c_val.abs() <= 0.5) {
                equal_width_pairs.push((f_id, s_id));
            }
        } else if a1 == ConstraintAttr::CenterX && a2 == ConstraintAttr::CenterX && c.relation == ConstraintRelation::Equal {
            if let Some(s_id) = s_opt {
                if is_super(f_id, s_id, &scene.nodes) {
                    centerx_consts.insert(f_id, (s_id, c_val));
                } else if is_super(s_id, f_id, &scene.nodes) {
                    centerx_consts.insert(s_id, (f_id, -c_val));
                }
            }
        } else if matches!(a1, ConstraintAttr::Left | ConstraintAttr::Leading)
            && matches!(a2, ConstraintAttr::Left | ConstraintAttr::Leading)
            && c.relation == ConstraintRelation::Equal
        {
            if let Some(s_id) = s_opt {
                if is_super(f_id, s_id, &scene.nodes) {
                    leading_consts.insert(f_id, (None, c_val));
                } else if is_super(s_id, f_id, &scene.nodes) {
                    leading_consts.insert(s_id, (None, -c_val));
                } else if c_val.abs() <= 0.5 {
                    equal_lead_pairs.push((f_id, s_id));
                }
            } else {
                leading_consts.insert(f_id, (None, c_val));
            }
        } else if matches!(a1, ConstraintAttr::Left | ConstraintAttr::Leading)
            && matches!(a2, ConstraintAttr::Right | ConstraintAttr::Trailing)
            && c.relation == ConstraintRelation::Equal
        {
            if let Some(s_id) = s_opt {
                leading_consts.insert(f_id, (Some(s_id), c_val));
            }
        } else if matches!(a1, ConstraintAttr::Right | ConstraintAttr::Trailing)
            && matches!(a2, ConstraintAttr::Right | ConstraintAttr::Trailing)
            && (c.relation == ConstraintRelation::Equal || is_stack_root)
        {
            if let Some(s_id) = s_opt {
                if is_super(f_id, s_id, &scene.nodes) {
                    trailing_consts.insert(f_id, if c_val <= 0.0 { -c_val } else { c_val });
                } else if is_super(s_id, f_id, &scene.nodes) {
                    trailing_consts.insert(s_id, if c_val >= 0.0 { c_val } else { -c_val });
                } else if c_val.abs() <= 0.5 {
                    equal_trail_pairs.push((f_id, s_id));
                }
            } else {
                trailing_consts.insert(f_id, c_val.abs());
            }
        }
    }

    if is_stack_root || !equal_lead_pairs.is_empty() {
        for (&nid, &w) in &width_consts {
            if let Some(&idx) = id_to_idx.get(&nid) {
                if !matches!(scene.nodes[idx].kind, SceneNodeKind::Container { .. }) {
                    set_aw(&mut scene.nodes[idx], w);
                }
            }
        }
        for (&nid, &(p_id, c_off)) in &centerx_consts {
            if let Some(&idx) = id_to_idx.get(&nid) {
                let (p_x, p_w) = id_to_idx
                    .get(&p_id)
                    .map(|&pi| {
                        if scene.nodes[pi].parent.is_some() {
                            (scene.nodes[pi].nib_frame[0], scene.nodes[pi].nib_frame[2])
                        } else {
                            (0.0, root_w)
                        }
                    })
                    .unwrap_or((0.0, root_w));
                let aw = align_w(&scene.nodes[idx]);
                set_ax(&mut scene.nodes[idx], (p_x + (p_w - aw) * 0.5 + c_off).round());
            }
        }
        for (&nid, &(sib_opt, lead)) in &leading_consts {
            if let Some(&idx) = id_to_idx.get(&nid).filter(|&&i| {
                sib_opt.is_none() && !matches!(scene.nodes[i].kind, SceneNodeKind::Container { .. } | SceneNodeKind::TabView { .. })
            }) {
                let p_x = scene.nodes[idx]
                    .parent
                    .and_then(|pid| id_to_idx.get(&pid).map(|&pi| &scene.nodes[pi]))
                    .map(|pn| if pn.parent.is_some() { pn.nib_frame[0] } else { 0.0 })
                    .unwrap_or(0.0);
                set_ax(&mut scene.nodes[idx], p_x + lead);
            }
        }
        for (&nid, &(sib_opt, gap)) in &leading_consts {
            if let Some(sib_id) = sib_opt {
                if let (Some(&n_idx), Some(&s_idx)) = (id_to_idx.get(&nid), id_to_idx.get(&sib_id)) {
                    if centerx_consts.contains_key(&nid) {
                        let target_right = align_x(&scene.nodes[n_idx]) - gap;
                        if width_consts.contains_key(&sib_id) || !leading_consts.contains_key(&sib_id) {
                            let sib_aw = align_w(&scene.nodes[s_idx]);
                            set_ax(&mut scene.nodes[s_idx], target_right - sib_aw);
                        } else {
                            let sib_ax = align_x(&scene.nodes[s_idx]);
                            set_aw(&mut scene.nodes[s_idx], (target_right - sib_ax).max(0.0));
                        }
                    } else {
                        let sib_ar_right = align_x(&scene.nodes[s_idx]) + align_w(&scene.nodes[s_idx]);
                        set_ax(&mut scene.nodes[n_idx], sib_ar_right + gap);
                    }
                }
            }
        }
        for _ in 0..equal_lead_pairs.len() {
            for &(f_id, s_id) in &equal_lead_pairs {
                if let (Some(&f_idx), Some(&s_idx)) = (id_to_idx.get(&f_id), id_to_idx.get(&s_id)) {
                    let max_ax = align_x(&scene.nodes[f_idx]).max(align_x(&scene.nodes[s_idx]));
                    set_ax(&mut scene.nodes[f_idx], max_ax);
                    set_ax(&mut scene.nodes[s_idx], max_ax);
                }
            }
        }
        for (&nid, &(sib_opt, gap)) in &leading_consts {
            if let Some(sib_id) = sib_opt {
                if !centerx_consts.contains_key(&nid) {
                    if let (Some(&n_idx), Some(&s_idx)) = (id_to_idx.get(&nid), id_to_idx.get(&sib_id)) {
                        let req_sib_w = align_x(&scene.nodes[n_idx]) - gap - align_x(&scene.nodes[s_idx]);
                        if req_sib_w > align_w(&scene.nodes[s_idx]) {
                            set_aw(&mut scene.nodes[s_idx], req_sib_w);
                        }
                    }
                }
            }
        }
        for &(f_id, s_id) in &equal_width_pairs {
            if let (Some(&f_idx), Some(&s_idx)) = (id_to_idx.get(&f_id), id_to_idx.get(&s_id)) {
                if let (Some(&(Some(lead_sib), gap)), Some(&trail)) = (leading_consts.get(&f_id), trailing_consts.get(&f_id)) {
                    if lead_sib == s_id {
                        let s_ax = align_x(&scene.nodes[s_idx]);
                        let each_aw = ((root_w - trail - s_ax - gap) * 0.5).round();
                        set_aw(&mut scene.nodes[s_idx], each_aw);
                        set_ax(&mut scene.nodes[f_idx], s_ax + each_aw + gap);
                        set_aw(&mut scene.nodes[f_idx], each_aw);
                    }
                }
            }
        }
        for (&nid, &trail) in &trailing_consts {
            if let Some(&n_idx) = id_to_idx.get(&nid) {
                let solved_by_adj_equal_width = equal_width_pairs.iter().any(|&(f, s)| {
                    f == nid && leading_consts.get(&f).is_some_and(|&(sib, _)| sib == Some(s)) && trailing_consts.contains_key(&f)
                });
                if matches!(scene.nodes[n_idx].kind, SceneNodeKind::Container { .. } | SceneNodeKind::TabView { .. })
                    || solved_by_adj_equal_width
                {
                    continue;
                }
                let p_right = scene.nodes[n_idx]
                    .parent
                    .and_then(|pid| id_to_idx.get(&pid).map(|&pi| &scene.nodes[pi]))
                    .map(|pn| if pn.parent.is_some() { pn.nib_frame[0] + pn.nib_frame[2] } else { root_w })
                    .unwrap_or(root_w);
                if width_consts.contains_key(&nid) || !leading_consts.contains_key(&nid) {
                    let aw = align_w(&scene.nodes[n_idx]);
                    set_ax(&mut scene.nodes[n_idx], p_right - trail - aw);
                } else {
                    let new_aw = p_right - trail - align_x(&scene.nodes[n_idx]);
                    if new_aw >= 16.0 {
                        set_aw(&mut scene.nodes[n_idx], new_aw);
                    }
                }
            }
        }
        for (f_id, s_id) in equal_trail_pairs {
            if let (Some(&f_idx), Some(&s_idx)) = (id_to_idx.get(&f_id), id_to_idx.get(&s_id)) {
                let s_ar_right = align_x(&scene.nodes[s_idx]) + align_w(&scene.nodes[s_idx]);
                let new_aw = s_ar_right - align_x(&scene.nodes[f_idx]);
                if new_aw >= 16.0 {
                    set_aw(&mut scene.nodes[f_idx], new_aw);
                }
            }
        }
    }

    crate::scene::keyed_archiver::resolve_scene_vertical_stack_layout(scene);
    crate::scene::stack_reflow::derive_autoresizing_masks_from_constraints(scene);

    let root_flipped = scene.nodes.first().is_some_and(|n| matches!(n.kind, SceneNodeKind::Container { is_flipped: true }));
    let design_h = scene.design_size[1];
    for node in &mut scene.nodes {
        let flip = if matches!(node.kind, SceneNodeKind::Container { .. }) { true } else { root_flipped };
        node.alignment_rect = normalize_nib_alignment_rect(&node.kind, node.mac_control_size(), node.nib_frame, design_h, flip);
    }
}
