//! Official Xcode 16.2 (`toolsVersion="23504"`) Cocoa `.xib` v3.0 XML Exporter (`REQ-013-05.1`, `REQ-013-05.5`).
//!
//! Transpiles any [`MacSceneIR`] into a 100% `ibtool`-compliant Xcode 16.2 `.xib` XML document
//! using genuine AppKit bottom-up (`isFlipped == false`) coordinates with zero hidden JSON shortcuts.

use crate::scene::ir::{MacSceneIR, NodeId, SceneNode, SceneNodeKind};

/// Exports `scene` into an official Xcode 16.2 (`toolsVersion="23504"`) Cocoa `.xib` v3.0 XML document.
pub fn scene_to_xcode16_xib_xml(scene: &MacSceneIR) -> String {
    let root = scene.nodes.first();
    let win_title = root.map(|n| escape_xml(&n.title)).unwrap_or_else(|| "Window".to_string());
    let root_id = root.map(|n| escape_xml(&n.ib_object_id)).unwrap_or_else(|| "root-view".to_string());
    let [w, h] = scene.design_size;

    let mut out = String::with_capacity(8192);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<document type=\"com.apple.InterfaceBuilder3.Cocoa.XIB\" version=\"3.0\" toolsVersion=\"23504\" targetRuntime=\"MacOSX.Cocoa\" propertyAccessControl=\"none\" useAutolayout=\"NO\" customObjectInstantitationMethod=\"direct\">\n");
    out.push_str("    <dependencies>\n");
    out.push_str("        <plugIn identifier=\"com.apple.InterfaceBuilder.CocoaPlugin\" version=\"23504\"/>\n");
    out.push_str("        <capability name=\"documents saved in the Xcode 8 format\" minToolsVersion=\"8.0\"/>\n");
    out.push_str("    </dependencies>\n");
    out.push_str("    <objects>\n");
    out.push_str("        <customObject id=\"-2\" userLabel=\"File's Owner\" customClass=\"NSWindowController\"/>\n");
    out.push_str("        <customObject id=\"-1\" userLabel=\"First Responder\" customClass=\"FirstResponder\"/>\n");
    out.push_str("        <customObject id=\"-3\" userLabel=\"Application\" customClass=\"NSObject\"/>\n");
    out.push_str(&format!(
        "        <window title=\"{win_title}\" allowsToolTipsWhenApplicationIsInactive=\"NO\" autorecalculatesKeyViewLoop=\"NO\" releasedWhenClosed=\"NO\" animationBehavior=\"default\" id=\"win-{root_id}\">\n"
    ));
    out.push_str(
        "            <windowStyleMask key=\"styleMask\" titled=\"YES\" closable=\"YES\" miniaturizable=\"YES\" resizable=\"YES\"/>\n",
    );
    out.push_str(&format!("            <rect key=\"contentRect\" x=\"196\" y=\"240\" width=\"{w:.0}\" height=\"{h:.0}\"/>\n"));
    out.push_str("            <rect key=\"screenRect\" x=\"0.0\" y=\"0.0\" width=\"1680\" height=\"1027\"/>\n");
    emit_root_view_xml(scene, &root_id, [w, h], false, "            ", &mut out);
    out.push_str("        </window>\n");
    out.push_str("    </objects>\n");
    out.push_str("</document>\n");
    out
}

/// Emits a `<view>` element (used for both `<view key="contentView">` in `.xib` and `<view key="view">` in `.storyboard`).
pub(crate) fn emit_root_view_xml(scene: &MacSceneIR, view_id: &str, [w, h]: [f32; 2], is_flipped: bool, indent: &str, out: &mut String) {
    let view_key = if indent.len() <= 12 { "contentView" } else { "view" };
    out.push_str(&format!("{indent}<view key=\"{view_key}\" identifier=\"{view_id}\" id=\"{view_id}\">\n"));
    out.push_str(&format!("{indent}    <rect key=\"frame\" x=\"0.0\" y=\"0.0\" width=\"{w:.0}\" height=\"{h:.0}\"/>\n"));
    out.push_str(&format!("{indent}    <autoresizingMask key=\"autoresizingMask\" widthSizable=\"YES\" heightSizable=\"YES\"/>\n"));
    let root_nid = scene.root;
    let direct_children: Vec<&SceneNode> =
        scene.nodes.iter().skip(1).filter(|n| n.parent == Some(root_nid) || n.parent.is_none()).collect();
    if !direct_children.is_empty() {
        out.push_str(&format!("{indent}    <subviews>\n"));
        for child in direct_children {
            emit_xib_node(scene, child, &format!("{indent}        "), out);
        }
        out.push_str(&format!("{indent}    </subviews>\n"));
    }
    if is_flipped {
        out.push_str(&format!("{indent}    <userDefinedRuntimeAttributes>\n"));
        out.push_str(&format!("{indent}        <userDefinedRuntimeAttribute type=\"string\" keyPath=\"ibFlipped\" value=\"YES\"/>\n"));
        out.push_str(&format!("{indent}    </userDefinedRuntimeAttributes>\n"));
    }
    out.push_str(&format!("{indent}</view>\n"));
}

fn emit_xib_node(scene: &MacSceneIR, node: &SceneNode, indent: &str, out: &mut String) {
    let id = sanitize_ib_id(&node.ib_object_id, node.id);
    let title = escape_xml(&node.title);
    let cs = node.control_size.as_str();
    let cs_attr = if cs == "small" || cs == "mini" { format!(" controlSize=\"{cs}\"") } else { String::new() };
    let en_attr = if !node.enabled { " enabled=\"NO\"" } else { "" };
    let tip_attr = if !node.tooltip.is_empty() { format!(" toolTip=\"{}\"", escape_xml(&node.tooltip)) } else { String::new() };
    let src_flipped = scene.nodes.first().is_some_and(|n| matches!(n.kind, SceneNodeKind::Container { is_flipped: true }));
    let [x, raw_y, w, h] = node.nib_frame;
    let y = if src_flipped { (scene.design_size[1] - (raw_y + h)).max(0.0) } else { raw_y };
    let rect_line = format!("{indent}    <rect key=\"frame\" x=\"{x:.0}\" y=\"{y:.0}\" width=\"{w:.0}\" height=\"{h:.0}\"/>\n");
    let mask_line = format_autoresizing_xml(node.autoresizing_mask, &format!("{indent}    "));
    let inspectables = build_inspectable_attrs(node, [x, y, w, h]);

    match &node.kind {
        SceneNodeKind::BoxContainer => {
            out.push_str(&format!("{indent}<box identifier=\"{id}\" title=\"{title}\"{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"));
            let bw = (w - 8.0).max(16.0);
            let bh = (h - 20.0).max(16.0);
            out.push_str(&format!("{indent}    <view key=\"contentView\" id=\"cv-{id}\">\n"));
            out.push_str(&format!("{indent}        <rect key=\"frame\" x=\"4\" y=\"5\" width=\"{bw:.0}\" height=\"{bh:.0}\"/>\n"));
            out.push_str(&format!(
                "{indent}        <autoresizingMask key=\"autoresizingMask\" widthSizable=\"YES\" heightSizable=\"YES\"/>\n"
            ));
            let box_children: Vec<&SceneNode> = scene.nodes.iter().filter(|n| n.parent == Some(node.id)).collect();
            if !box_children.is_empty() {
                out.push_str(&format!("{indent}        <subviews>\n"));
                for bc in box_children {
                    emit_xib_node(scene, bc, &format!("{indent}            "), out);
                }
                out.push_str(&format!("{indent}        </subviews>\n"));
            }
            out.push_str(&format!("{indent}    </view>\n"));
            emit_connections_and_inspectables(node, &id, &inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</box>\n"));
        }
        SceneNodeKind::Separator => {
            out.push_str(&format!(
                "{indent}<box identifier=\"{id}\" verticalHuggingPriority=\"750\" boxType=\"separator\" id=\"{id}\">\n{rect_line}{mask_line}"
            ));
            emit_connections_and_inspectables(node, &id, &inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</box>\n"));
        }
        SceneNodeKind::Label => {
            out.push_str(&format!(
                "{indent}<textField identifier=\"{id}\" horizontalHuggingPriority=\"251\" verticalHuggingPriority=\"750\"{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
            ));
            out.push_str(&format!(
                "{indent}    <textFieldCell key=\"cell\"{cs_attr} lineBreakMode=\"clipping\" editable=\"NO\" title=\"{title}\" id=\"cell-{id}\">\n"
            ));
            let icon = node.icon_name.as_deref().unwrap_or("");
            let mfont = if icon.contains("semibold") || icon.contains("bold") {
                "systemSemibold"
            } else if icon.contains("medium") {
                "systemMedium"
            } else {
                "system"
            };
            out.push_str(&format!("{indent}        <font key=\"font\" metaFont=\"{mfont}\"/>\n"));
            if icon.contains("secondary") {
                out.push_str(&format!(
                    "{indent}        <color key=\"textColor\" name=\"secondaryLabelColor\" catalog=\"System\" colorSpace=\"catalog\"/>\n"
                ));
            } else if icon.contains("tertiary") {
                out.push_str(&format!(
                    "{indent}        <color key=\"textColor\" name=\"tertiaryLabelColor\" catalog=\"System\" colorSpace=\"catalog\"/>\n"
                ));
            }
            out.push_str(&format!("{indent}    </textFieldCell>\n"));
            emit_connections_and_inspectables(node, &id, &inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</textField>\n"));
        }
        SceneNodeKind::TextField => {
            if node.icon_name.as_deref() == Some("NSTokenField") {
                out.push_str(&format!(
                    "{indent}<tokenField identifier=\"{id}\" verticalHuggingPriority=\"750\"{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
                ));
                out.push_str(&format!(
                    "{indent}    <tokenFieldCell key=\"cell\"{cs_attr} selectable=\"YES\" editable=\"YES\" borderStyle=\"bezel\" alignment=\"left\" title=\"{title}\" drawsBackground=\"YES\" allowsEditingTextAttributes=\"NO\" id=\"cell-{id}\">\n"
                ));
                out.push_str(&format!("{indent}        <font key=\"font\" metaFont=\"system\"/>\n{indent}    </tokenFieldCell>\n"));
                emit_connections_and_inspectables(node, &id, &inspectables, &format!("{indent}    "), out);
                out.push_str(&format!("{indent}</tokenField>\n"));
            } else {
                out.push_str(&format!(
                    "{indent}<textField identifier=\"{id}\" verticalHuggingPriority=\"750\"{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
                ));
                out.push_str(&format!(
                    "{indent}    <textFieldCell key=\"cell\"{cs_attr} scrollable=\"YES\" lineBreakMode=\"clipping\" selectable=\"YES\" editable=\"YES\" sendsActionOnEndEditing=\"YES\" borderStyle=\"bezel\" title=\"{title}\" drawsBackground=\"YES\" id=\"cell-{id}\">\n"
                ));
                out.push_str(&format!("{indent}        <font key=\"font\" metaFont=\"system\"/>\n{indent}    </textFieldCell>\n"));
                emit_connections_and_inspectables(node, &id, &inspectables, &format!("{indent}    "), out);
                out.push_str(&format!("{indent}</textField>\n"));
            }
        }
        SceneNodeKind::NumberField { min, max, .. } => {
            out.push_str(&format!(
                "{indent}<textField identifier=\"{id}\" verticalHuggingPriority=\"750\" customClass=\"GSSteppingTextField\"{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
            ));
            out.push_str(&format!(
                "{indent}    <textFieldCell key=\"cell\"{cs_attr} scrollable=\"YES\" lineBreakMode=\"clipping\" selectable=\"YES\" editable=\"YES\" sendsActionOnEndEditing=\"YES\" state=\"on\" borderStyle=\"bezel\" title=\"{title}\" drawsBackground=\"YES\" id=\"cell-{id}\">\n"
            ));
            out.push_str(&format!(
                "{indent}        <numberFormatter key=\"formatter\" formatterBehavior=\"default10_4\" numberStyle=\"decimal\" minimumIntegerDigits=\"1\" maximumIntegerDigits=\"2000000000\" maximumFractionDigits=\"3\" id=\"fmt-{id}\">\n"
            ));
            if let Some(mn) = min {
                out.push_str(&format!("{indent}            <real key=\"minimum\" value=\"{mn}\"/>\n"));
            }
            if let Some(mx) = max {
                out.push_str(&format!("{indent}            <real key=\"maximum\" value=\"{mx}\"/>\n"));
            }
            out.push_str(&format!("{indent}        </numberFormatter>\n"));
            out.push_str(&format!("{indent}        <font key=\"font\" metaFont=\"system\"/>\n{indent}    </textFieldCell>\n"));
            emit_connections_and_inspectables(node, &id, &inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</textField>\n"));
        }
        SceneNodeKind::PushButton { is_default, is_cancel } => {
            let ke = if *is_default {
                " keyEquivalent=\"&#13;\""
            } else if *is_cancel {
                " keyEquivalent=\"&#27;\""
            } else {
                ""
            };
            out.push_str(&format!(
                "{indent}<button identifier=\"{id}\" verticalHuggingPriority=\"750\"{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
            ));
            out.push_str(&format!(
                "{indent}    <buttonCell key=\"cell\"{cs_attr} type=\"push\" title=\"{title}\" bezelStyle=\"rounded\" alignment=\"center\" borderStyle=\"border\" imageScaling=\"proportionallyDown\" inset=\"2\"{ke} id=\"cell-{id}\">\n"
            ));
            out.push_str(&format!(
                "{indent}        <behavior key=\"behavior\" pushIn=\"YES\" lightByBackground=\"YES\" lightByGray=\"YES\"/>\n{indent}        <font key=\"font\" metaFont=\"system\"/>\n{indent}    </buttonCell>\n"
            ));
            emit_connections_and_inspectables(node, &id, &inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</button>\n"));
        }
        SceneNodeKind::Checkbox
        | SceneNodeKind::RadioButton
        | SceneNodeKind::Switch
        | SceneNodeKind::HelpButton
        | SceneNodeKind::InlineActionButton => {
            let ctx = XibNodeCtx {
                id: &id,
                title: &title,
                cs_attr: &cs_attr,
                en_attr,
                tip_attr: &tip_attr,
                rect_line: &rect_line,
                mask_line: &mask_line,
                inspectables: &inspectables,
                indent,
            };
            emit_button_variant_xml(node, &ctx, out);
        }
        _ => {
            let ctx = XibNodeCtx {
                id: &id,
                title: &title,
                cs_attr: &cs_attr,
                en_attr,
                tip_attr: &tip_attr,
                rect_line: &rect_line,
                mask_line: &mask_line,
                inspectables: &inspectables,
                indent,
            };
            emit_complex_widget_xml(node, &ctx, out);
        }
    }
}

struct XibNodeCtx<'a> {
    id: &'a str,
    title: &'a str,
    cs_attr: &'a str,
    en_attr: &'a str,
    tip_attr: &'a str,
    rect_line: &'a str,
    mask_line: &'a str,
    inspectables: &'a [(String, String)],
    indent: &'a str,
}

fn emit_button_variant_xml(node: &SceneNode, ctx: &XibNodeCtx<'_>, out: &mut String) {
    let XibNodeCtx { id, title, cs_attr, en_attr, tip_attr, rect_line, mask_line, inspectables, indent } = *ctx;
    let icon = node.icon_name.as_deref().unwrap_or("");
    let (cls_attr, btype, bstyle, extra_cell) = match &node.kind {
        SceneNodeKind::HelpButton => ("", "help", "helpButton", ""),
        SceneNodeKind::InlineActionButton => (" customClass=\"NSInlineActionButton\"", "inline", "inline", ""),
        SceneNodeKind::RadioButton => ("", "radio", "regularSquare", " imagePosition=\"left\""),
        SceneNodeKind::Switch => ("", "switch", "regularSquare", " imagePosition=\"left\" state=\"on\""),
        SceneNodeKind::Checkbox if icon == "NSDisclosureButton" => ("", "disclosureTriangle", "disclosure", ""),
        SceneNodeKind::Checkbox if icon == "NSMixedCheckbox" => {
            ("", "check", "regularSquare", " imagePosition=\"left\" state=\"on\" allowsMixedState=\"YES\"")
        }
        _ => ("", "check", "regularSquare", " imagePosition=\"left\" state=\"on\""),
    };
    out.push_str(&format!(
        "{indent}<button identifier=\"{id}\" verticalHuggingPriority=\"750\"{cls_attr}{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
    ));
    out.push_str(&format!(
        "{indent}    <buttonCell key=\"cell\"{cs_attr} type=\"{btype}\" title=\"{title}\" bezelStyle=\"{bstyle}\"{extra_cell} inset=\"2\" id=\"cell-{id}\">\n"
    ));
    out.push_str(&format!("{indent}        <font key=\"font\" metaFont=\"system\"/>\n{indent}    </buttonCell>\n"));
    emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
    out.push_str(&format!("{indent}</button>\n"));
}

fn emit_complex_widget_xml(node: &SceneNode, ctx: &XibNodeCtx<'_>, out: &mut String) {
    let XibNodeCtx { id, title, cs_attr, en_attr, tip_attr, rect_line, mask_line, inspectables, indent } = *ctx;
    let [_, _, w, h] = node.nib_frame;
    match &node.kind {
        SceneNodeKind::PopUpButton { options } => {
            let first = options.first().map(|s| escape_xml(s)).unwrap_or_else(|| title.to_string());
            out.push_str(&format!(
                "{indent}<popUpButton identifier=\"{id}\" verticalHuggingPriority=\"750\"{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
            ));
            out.push_str(&format!(
                "{indent}    <popUpButtonCell key=\"cell\"{cs_attr} type=\"push\" title=\"{first}\" bezelStyle=\"rounded\" alignment=\"left\" lineBreakMode=\"truncatingTail\" state=\"on\" borderStyle=\"borderAndBezel\" imageScaling=\"proportionallyDown\" inset=\"2\" id=\"cell-{id}\">\n"
            ));
            out.push_str(&format!("{indent}        <behavior key=\"behavior\" lightByBackground=\"YES\" lightByGray=\"YES\"/>\n"));
            out.push_str(&format!("{indent}        <font key=\"font\" metaFont=\"menu\"/>\n"));
            out.push_str(&format!("{indent}        <menu key=\"menu\" id=\"menu-{id}\">\n{indent}            <items>\n"));
            for (i, opt) in options.iter().enumerate() {
                let st = if i == 0 { " state=\"on\"" } else { "" };
                out.push_str(&format!("{indent}                <menuItem title=\"{}\"{st} id=\"mi-{id}-{i}\"/>\n", escape_xml(opt)));
            }
            out.push_str(&format!("{indent}            </items>\n{indent}        </menu>\n{indent}    </popUpButtonCell>\n"));
            emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</popUpButton>\n"));
        }
        SceneNodeKind::ComboBox { options } => {
            out.push_str(&format!(
                "{indent}<comboBox identifier=\"{id}\" verticalHuggingPriority=\"750\"{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
            ));
            out.push_str(&format!(
                "{indent}    <comboBoxCell key=\"cell\"{cs_attr} scrollable=\"YES\" lineBreakMode=\"clipping\" selectable=\"YES\" editable=\"YES\" sendsActionOnEndEditing=\"YES\" borderStyle=\"bezel\" title=\"{title}\" drawsBackground=\"YES\" completes=\"NO\" numberOfVisibleItems=\"5\" id=\"cell-{id}\">\n"
            ));
            out.push_str(&format!("{indent}        <font key=\"font\" metaFont=\"system\"/>\n{indent}        <objectValues>\n"));
            for opt in options {
                out.push_str(&format!("{indent}            <string>{}</string>\n", escape_xml(opt)));
            }
            out.push_str(&format!("{indent}        </objectValues>\n{indent}    </comboBoxCell>\n"));
            emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</comboBox>\n"));
        }
        SceneNodeKind::SegmentedControl { segments } => {
            out.push_str(&format!(
                "{indent}<segmentedControl identifier=\"{id}\" verticalHuggingPriority=\"750\"{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
            ));
            out.push_str(&format!(
                "{indent}    <segmentedCell key=\"cell\"{cs_attr} borderStyle=\"border\" alignment=\"left\" style=\"rounded\" trackingMode=\"selectOne\" id=\"cell-{id}\">\n"
            ));
            out.push_str(&format!("{indent}        <font key=\"font\" metaFont=\"system\"/>\n{indent}        <segments>\n"));
            for (i, seg) in segments.iter().enumerate() {
                let sel = if i == 0 { " selected=\"YES\"" } else { "" };
                out.push_str(&format!("{indent}            <segment label=\"{}\"{sel}/>\n", escape_xml(seg)));
            }
            out.push_str(&format!("{indent}        </segments>\n{indent}    </segmentedCell>\n"));
            emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</segmentedControl>\n"));
        }
        SceneNodeKind::Slider { min, max, tick_marks } => {
            let icon = node.icon_name.as_deref().unwrap_or("");
            if icon == "NSRatingIndicator" || icon == "NSLevelIndicator" {
                let lstyle = if icon == "NSRatingIndicator" { "rating" } else { "continuousCapacity" };
                out.push_str(&format!(
                    "{indent}<levelIndicator identifier=\"{id}\" verticalHuggingPriority=\"750\"{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
                ));
                out.push_str(&format!(
                    "{indent}    <levelIndicatorCell key=\"cell\"{cs_attr} alignment=\"left\" doubleValue=\"{min}\" minValue=\"{min}\" maxValue=\"{max}\" numberOfTickMarks=\"{tick_marks}\" levelIndicatorStyle=\"{lstyle}\" id=\"cell-{id}\"/>\n"
                ));
                emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
                out.push_str(&format!("{indent}</levelIndicator>\n"));
            } else if icon == "NSProgressSpinning" || icon == "NSProgressBar" {
                let pstyle = if icon == "NSProgressSpinning" { "spinning" } else { "bar" };
                out.push_str(&format!(
                    "{indent}<progressIndicator identifier=\"{id}\"{cs_attr} minValue=\"{min}\" maxValue=\"{max}\" doubleValue=\"{min}\" style=\"{pstyle}\" id=\"{id}\">\n{rect_line}{mask_line}"
                ));
                emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
                out.push_str(&format!("{indent}</progressIndicator>\n"));
            } else {
                let stype = if icon == "NSCircularSlider" { "circular" } else { "linear" };
                out.push_str(&format!(
                    "{indent}<slider identifier=\"{id}\" verticalHuggingPriority=\"750\"{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
                ));
                out.push_str(&format!(
                    "{indent}    <sliderCell key=\"cell\"{cs_attr} state=\"on\" alignment=\"left\" minValue=\"{min}\" maxValue=\"{max}\" doubleValue=\"{min}\" tickMarkPosition=\"above\" numberOfTickMarks=\"{tick_marks}\" sliderType=\"{stype}\" id=\"cell-{id}\"/>\n"
                ));
                emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
                out.push_str(&format!("{indent}</slider>\n"));
            }
        }
        SceneNodeKind::Stepper { step } => {
            out.push_str(&format!(
                "{indent}<stepper identifier=\"{id}\" horizontalHuggingPriority=\"750\" verticalHuggingPriority=\"750\"{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
            ));
            out.push_str(&format!(
                "{indent}    <stepperCell key=\"cell\"{cs_attr} continuous=\"YES\" alignment=\"left\" increment=\"{step}\" minValue=\"-10000\" maxValue=\"10000\" id=\"cell-{id}\"/>\n"
            ));
            emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</stepper>\n"));
        }
        SceneNodeKind::ColorWell => {
            out.push_str(&format!("{indent}<colorWell identifier=\"{id}\"{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"));
            out.push_str(&format!(
                "{indent}    <color key=\"color\" red=\"0.0\" green=\"0.47843137250000001\" blue=\"1\" alpha=\"1\" colorSpace=\"calibratedRGB\"/>\n"
            ));
            emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</colorWell>\n"));
        }
        SceneNodeKind::DatePicker { clock_and_calendar } => {
            let dstyle = if *clock_and_calendar { "clockAndCalendar" } else { "textFieldAndStepper" };
            out.push_str(&format!(
                "{indent}<datePicker identifier=\"{id}\" verticalHuggingPriority=\"750\"{cs_attr}{en_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
            ));
            out.push_str(&format!(
                "{indent}    <datePickerCell key=\"cell\"{cs_attr} borderStyle=\"bezel\" alignment=\"left\" datePickerStyle=\"{dstyle}\" id=\"cell-{id}\">\n"
            ));
            out.push_str(&format!(
                "{indent}        <font key=\"font\" metaFont=\"system\"/>\n{indent}        <date key=\"date\" timeIntervalSinceReferenceDate=\"784987200\"/>\n{indent}    </datePickerCell>\n"
            ));
            emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</datePicker>\n"));
        }
        SceneNodeKind::TabView { tabs } => {
            out.push_str(&format!("{indent}<tabView identifier=\"{id}\"{cs_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"));
            out.push_str(&format!("{indent}    <font key=\"font\" metaFont=\"system\"/>\n{indent}    <tabViewItems>\n"));
            let tw = (w - 20.0).max(40.0);
            let th = (h - 46.0).max(40.0);
            for (i, tab) in tabs.iter().enumerate() {
                out.push_str(&format!(
                    "{indent}        <tabViewItem label=\"{}\" identifier=\"{i}\" id=\"tvi-{id}-{i}\">\n",
                    escape_xml(tab)
                ));
                out.push_str(&format!("{indent}            <view key=\"view\" id=\"tvv-{id}-{i}\">\n"));
                out.push_str(&format!(
                    "{indent}                <rect key=\"frame\" x=\"10\" y=\"33\" width=\"{tw:.0}\" height=\"{th:.0}\"/>\n"
                ));
                out.push_str(&format!(
                    "{indent}                <autoresizingMask key=\"autoresizingMask\" widthSizable=\"YES\" heightSizable=\"YES\"/>\n"
                ));
                out.push_str(&format!("{indent}            </view>\n{indent}        </tabViewItem>\n"));
            }
            out.push_str(&format!("{indent}    </tabViewItems>\n"));
            emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</tabView>\n"));
        }
        SceneNodeKind::RuleEditor => {
            out.push_str(&format!(
                "{indent}<ruleEditor identifier=\"{id}\" nestingMode=\"list\" canRemoveAllRows=\"NO\" rowHeight=\"25\"{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
            ));
            emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</ruleEditor>\n"));
        }
        SceneNodeKind::TextView { .. } => {
            out.push_str(&format!(
                "{indent}<textView identifier=\"{id}\" importsGraphics=\"NO\" richText=\"NO\" verticallyResizable=\"YES\"{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}"
            ));
            out.push_str(&format!("{indent}    <color key=\"textColor\" name=\"textColor\" catalog=\"System\" colorSpace=\"catalog\"/>\n"));
            out.push_str(&format!(
                "{indent}    <color key=\"backgroundColor\" name=\"textBackgroundColor\" catalog=\"System\" colorSpace=\"catalog\"/>\n"
            ));
            out.push_str(&format!("{indent}    <size key=\"minSize\" width=\"{w:.0}\" height=\"{h:.0}\"/>\n"));
            out.push_str(&format!("{indent}    <size key=\"maxSize\" width=\"10000\" height=\"10000000\"/>\n"));
            emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</textView>\n"));
        }
        SceneNodeKind::TableView { columns } => {
            out.push_str(&format!(
                "{indent}<scrollView identifier=\"{id}\" autohidesScrollers=\"YES\" horizontalLineScroll=\"19\" horizontalPageScroll=\"10\" verticalLineScroll=\"19\" verticalPageScroll=\"10\" usesPredominantAxisScrolling=\"NO\" id=\"{id}\">\n{rect_line}{mask_line}"
            ));
            out.push_str(&format!("{indent}    <clipView key=\"contentView\" id=\"clip-{id}\">\n"));
            out.push_str(&format!("{indent}        <rect key=\"frame\" x=\"1\" y=\"1\" width=\"{w:.0}\" height=\"{h:.0}\"/>\n"));
            out.push_str(&format!("{indent}        <autoresizingMask key=\"autoresizingMask\" widthSizable=\"YES\" heightSizable=\"YES\"/>\n{indent}        <subviews>\n"));
            out.push_str(&format!(
                "{indent}            <tableView verticalHuggingPriority=\"750\" id=\"tbl-{id}\">\n{indent}                <tableColumns>\n"
            ));
            for (ci, (col_title, col_w)) in columns.iter().enumerate() {
                out.push_str(&format!(
                    "{indent}                    <tableColumn identifier=\"col-{ci}\" width=\"{col_w:.0}\" id=\"tc-{id}-{ci}\">\n"
                ));
                out.push_str(&format!(
                    "{indent}                        <tableHeaderCell key=\"headerCell\" lineBreakMode=\"truncatingTail\" borderStyle=\"border\" title=\"{}\"/>\n",
                    escape_xml(col_title)
                ));
                out.push_str(&format!("{indent}                    </tableColumn>\n"));
            }
            out.push_str(&format!("{indent}                </tableColumns>\n{indent}            </tableView>\n{indent}        </subviews>\n{indent}    </clipView>\n"));
            emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</scrollView>\n"));
        }
        _ => {
            let cls = match &node.kind {
                SceneNodeKind::OriginMatrix => "GSOriginPointControl",
                SceneNodeKind::TagColorPicker => "ColorLabelView",
                SceneNodeKind::TableActionBar => "NSTableActionBar",
                SceneNodeKind::DisclosureGroup { .. } => "NSDisclosureGroup",
                SceneNodeKind::CustomCanvasSlot { class_name } => class_name.as_str(),
                _ => "NSCustomView",
            };
            out.push_str(&format!(
                "{indent}<customView identifier=\"{id}\" customClass=\"{}\"{cs_attr}{tip_attr} id=\"{id}\">\n{rect_line}{mask_line}",
                escape_xml(cls)
            ));
            emit_connections_and_inspectables(node, id, inspectables, &format!("{indent}    "), out);
            out.push_str(&format!("{indent}</customView>\n"));
        }
    }
}

fn emit_connections_and_inspectables(node: &SceneNode, id: &str, inspectables: &[(String, String)], indent: &str, out: &mut String) {
    if node.binding_path.is_some() || node.action_selector.is_some() {
        out.push_str(&format!("{indent}<connections>\n"));
        if let Some(ref kp) = node.binding_path {
            let bname = match node.kind {
                SceneNodeKind::PopUpButton { .. } | SceneNodeKind::SegmentedControl { .. } | SceneNodeKind::TabView { .. } => {
                    "selectedIndex"
                }
                _ => "value",
            };
            let full_kp = if kp.contains('.') { kp.clone() } else { format!("values.{kp}") };
            out.push_str(&format!(
                "{indent}    <binding destination=\"-2\" name=\"{bname}\" keyPath=\"{}\" id=\"b-{id}\"/>\n",
                escape_xml(&full_kp)
            ));
        }
        if let Some(ref sel) = node.action_selector {
            out.push_str(&format!("{indent}    <action selector=\"{}\" target=\"-2\" id=\"a-{id}\"/>\n", escape_xml(sel)));
        }
        out.push_str(&format!("{indent}</connections>\n"));
    }
    if !inspectables.is_empty() {
        out.push_str(&format!("{indent}<userDefinedRuntimeAttributes>\n"));
        for (k, v) in inspectables {
            out.push_str(&format!(
                "{indent}    <userDefinedRuntimeAttribute type=\"string\" keyPath=\"{}\" value=\"{}\"/>\n",
                escape_xml(k),
                escape_xml(v)
            ));
        }
        out.push_str(&format!("{indent}</userDefinedRuntimeAttributes>\n"));
    }
}

fn build_inspectable_attrs(node: &SceneNode, _rect: [f32; 4]) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    if let SceneNodeKind::DisclosureGroup { trailing_add_button } = node.kind {
        if !node.title.is_empty() {
            attrs.push(("title".into(), node.title.clone()));
        }
        if trailing_add_button {
            attrs.push(("trailingAddButton".into(), "YES".into()));
        }
    }
    if let SceneNodeKind::TextView { monospace, line_numbers } = node.kind {
        if monospace {
            attrs.push(("monospace".into(), "YES".into()));
        }
        if line_numbers {
            attrs.push(("lineNumbers".into(), "YES".into()));
        }
    }
    attrs
}

fn format_autoresizing_xml(mask: u8, indent: &str) -> String {
    let mut attrs = String::new();
    if (mask & 0x01) != 0 {
        attrs.push_str(" flexibleMinX=\"YES\"");
    }
    if (mask & 0x02) != 0 {
        attrs.push_str(" widthSizable=\"YES\"");
    }
    if (mask & 0x04) != 0 {
        attrs.push_str(" flexibleMaxX=\"YES\"");
    }
    if (mask & 0x08) != 0 {
        attrs.push_str(" flexibleMinY=\"YES\"");
    }
    if (mask & 0x10) != 0 {
        attrs.push_str(" heightSizable=\"YES\"");
    }
    if (mask & 0x20) != 0 {
        attrs.push_str(" flexibleMaxY=\"YES\"");
    }
    if attrs.is_empty() {
        attrs.push_str(" flexibleMaxX=\"YES\" flexibleMinY=\"YES\"");
    }
    format!("{indent}<autoresizingMask key=\"autoresizingMask\"{attrs}/>\n")
}

fn sanitize_ib_id(raw: &str, fallback: NodeId) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        format!("node-{}", fallback.0)
    } else {
        escape_xml(trimmed)
    }
}

pub(crate) fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
