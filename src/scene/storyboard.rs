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
