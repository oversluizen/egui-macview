//! Layer 1 Declarative SwiftUI-Subset Importer (`SwiftUiFrontend`, `SPEC-013 §2.6` / `REQ-013-01E`).

use crate::scene::constraints::{CompiledConstraintSchedule, ValueTransformer};
use crate::scene::ir::{ActionId, BindingId, MacSceneIR, NodeId, SceneCompileError, SceneNode, SceneNodeKind, VisibilityRule};

/// Pure-Rust declarative SwiftUI-subset parser (`REQ-013-01E`).
pub struct SwiftUiFrontend;

impl SwiftUiFrontend {
    /// Parses a declarative SwiftUI source block (`Form`, `VStack`, `HStack`, `Section`,
    /// `LabeledContent`, `Toggle`, `TextField`, `Button`, `Picker`, `Slider`, `Stepper`,
    /// `Image`, `TextEditor`, `.controlSize(...)`, `.disabled(...)`, `.help(...)`, `.frame(...)`,
    /// and `$model.field` bindings) into a canonical [`MacSceneIR`].
    pub fn parse(source: &str) -> Result<MacSceneIR, SceneCompileError> {
        if source.trim().is_empty() {
            return Err(SceneCompileError("Empty SwiftUI source".into()));
        }

        let mut nodes = vec![SceneNode {
            id: NodeId(1),
            ib_object_id: "swiftui-root".into(),
            parent: None,
            children: vec![],
            kind: SceneNodeKind::Container { is_flipped: true },
            title: "SwiftUIView".into(),
            tooltip: String::new(),
            control_size: "regular".into(),
            nib_frame: [0.0, 0.0, 520.0, 360.0],
            alignment_rect: [0.0, 0.0, 520.0, 360.0],
            autoresizing_mask: 18,
            enabled: true,
            value_binding: None,
            binding_path: None,
            value_transformer: ValueTransformer::Identity,
            action: None,
            action_selector: None,
            icon_name: None,
            visibility: VisibilityRule::Always,
            labeled_by: None,
        }];

        let mut next_id = 2u32;
        let mut next_binding_auto = 1u32;
        let mut next_action = 1u32;
        let mut cursor_y = 24.0_f32;
        let mut section_count = 0usize;
        let mut in_hstack = false;
        let mut hstack_x = 24.0_f32;

        for raw_line in source.lines() {
            let line = raw_line.trim();
            if line.is_empty()
                || line == "{"
                || line.starts_with("Form")
                || line.starts_with("VStack")
                || line.starts_with("struct ")
                || line.starts_with("var body")
            {
                continue;
            }
            if line.starts_with("HStack") {
                in_hstack = true;
                hstack_x = 24.0;
                continue;
            }
            if line == "}" {
                if in_hstack {
                    in_hstack = false;
                    cursor_y += 32.0;
                }
                continue;
            }

            if line.starts_with("Section(") || line.starts_with("Divider()") {
                if section_count > 0 || line.starts_with("Divider()") {
                    let sep_id = NodeId(next_id);
                    next_id += 1;
                    nodes[0].children.push(sep_id);
                    nodes.push(SceneNode {
                        id: sep_id,
                        ib_object_id: format!("sep-{}", sep_id.0),
                        parent: Some(NodeId(1)),
                        children: vec![],
                        kind: SceneNodeKind::Separator,
                        title: String::new(),
                        tooltip: String::new(),
                        control_size: "regular".into(),
                        nib_frame: [20.0, cursor_y, 480.0, 2.0],
                        alignment_rect: [20.0, cursor_y, 480.0, 2.0],
                        autoresizing_mask: 2,
                        enabled: true,
                        value_binding: None,
                        binding_path: None,
                        value_transformer: ValueTransformer::Identity,
                        action: None,
                        action_selector: None,
                        icon_name: None,
                        visibility: VisibilityRule::Always,
                        labeled_by: None,
                    });
                    cursor_y += 18.0;
                }
                if line.starts_with("Section(") {
                    if let Some(sec_title) = extract_first_quoted(line) {
                        if !sec_title.starts_with("Section ") && sec_title != "General" {
                            let box_id = NodeId(next_id);
                            next_id += 1;
                            nodes[0].children.push(box_id);
                            nodes.push(SceneNode {
                                id: box_id,
                                ib_object_id: format!("box-{}", box_id.0),
                                parent: Some(NodeId(1)),
                                children: vec![],
                                kind: SceneNodeKind::BoxContainer,
                                title: sec_title,
                                tooltip: String::new(),
                                control_size: "regular".into(),
                                nib_frame: [12.0, cursor_y, 496.0, 120.0],
                                alignment_rect: [12.0, cursor_y, 496.0, 120.0],
                                autoresizing_mask: 2,
                                enabled: true,
                                value_binding: None,
                                binding_path: None,
                                value_transformer: ValueTransformer::Identity,
                                action: None,
                                action_selector: None,
                                icon_name: None,
                                visibility: VisibilityRule::Always,
                                labeled_by: None,
                            });
                        }
                    }
                    section_count += 1;
                }
                continue;
            }

            let (outer_label, inner_stmt) = if line.starts_with("LabeledContent(") {
                let lbl = extract_first_quoted(line).unwrap_or_default();
                let body = line.find('{').and_then(|s| line.rfind('}').map(|e| line[s + 1..e].trim())).unwrap_or(line);
                (Some(lbl), body)
            } else {
                (None, line)
            };

            if !is_control_statement(inner_stmt) {
                continue;
            }

            let (lbl_x, lbl_w, ctrl_x) = if in_hstack { (hstack_x, 24.0, hstack_x + 28.0) } else { (24.0, 100.0, 132.0) };

            if let Some(lbl_text) = outer_label {
                let lid = NodeId(next_id);
                next_id += 1;
                nodes[0].children.push(lid);
                nodes.push(SceneNode {
                    id: lid,
                    ib_object_id: format!("lbl-{}", lid.0),
                    parent: Some(NodeId(1)),
                    children: vec![],
                    kind: SceneNodeKind::Label,
                    title: lbl_text,
                    tooltip: String::new(),
                    control_size: extract_control_size(inner_stmt),
                    nib_frame: [lbl_x, cursor_y + 2.0, lbl_w, 16.0],
                    alignment_rect: [lbl_x, cursor_y + 2.0, lbl_w, 16.0],
                    autoresizing_mask: 0,
                    enabled: true,
                    value_binding: None,
                    binding_path: None,
                    value_transformer: ValueTransformer::Identity,
                    action: None,
                    action_selector: None,
                    icon_name: None,
                    visibility: VisibilityRule::Always,
                    labeled_by: None,
                });
            }

            let cid = NodeId(next_id);
            next_id += 1;
            nodes[0].children.push(cid);

            let title = extract_first_quoted(inner_stmt).unwrap_or_default();
            let control_size = extract_control_size(inner_stmt);
            let enabled = !inner_stmt.contains(".disabled(true)");
            let tooltip = extract_modifier_quoted(inner_stmt, ".help(").unwrap_or_default();
            let (w, h) = extract_frame_size(inner_stmt).unwrap_or((120.0, 22.0));
            let (value_binding, binding_path) = extract_binding(inner_stmt, &mut next_binding_auto);

            let (kind, action, action_selector, icon_name) = classify_swiftui_control(inner_stmt, &title, &mut next_action);

            nodes.push(SceneNode {
                id: cid,
                ib_object_id: format!("ctrl-{}", cid.0),
                parent: Some(NodeId(1)),
                children: vec![],
                kind,
                title,
                tooltip,
                control_size,
                nib_frame: [ctrl_x, cursor_y, w, h],
                alignment_rect: [ctrl_x, cursor_y, w, h],
                autoresizing_mask: 0,
                enabled,
                value_binding,
                binding_path,
                value_transformer: ValueTransformer::Identity,
                action,
                action_selector,
                icon_name,
                visibility: VisibilityRule::Always,
                labeled_by: None,
            });

            if in_hstack {
                hstack_x = ctrl_x + w + 24.0;
            } else {
                cursor_y += h + 12.0;
            }
        }

        Ok(MacSceneIR::from_nodes(NodeId(1), [520.0, 360.0], nodes, CompiledConstraintSchedule::default()))
    }
}

fn classify_swiftui_control(
    inner_stmt: &str,
    title: &str,
    next_action: &mut u32,
) -> (SceneNodeKind, Option<ActionId>, Option<String>, Option<String>) {
    if inner_stmt.starts_with("Toggle(") {
        let kind = if inner_stmt.contains(".toggleStyle(.switch)") {
            SceneNodeKind::Switch
        } else if inner_stmt.contains(".toggleStyle(.radioGroup)") {
            SceneNodeKind::RadioButton
        } else {
            SceneNodeKind::Checkbox
        };
        (kind, None, None, None)
    } else if inner_stmt.starts_with("TextField(") {
        let kind = if inner_stmt.contains("format: .number") {
            SceneNodeKind::NumberField { step: 1.0, shift_step: 10.0, min: None, max: None }
        } else {
            SceneNodeKind::TextField
        };
        (kind, None, None, None)
    } else if inner_stmt.starts_with("Button(") {
        let act_str = extract_modifier_quoted(inner_stmt, "action: ").unwrap_or_else(|| "buttonAction:".into());
        let aid = ActionId(*next_action);
        *next_action += 1;
        (
            SceneNodeKind::PushButton {
                is_default: inner_stmt.contains(".defaultAction"),
                is_cancel: inner_stmt.contains(".cancelAction"),
            },
            Some(aid),
            Some(act_str),
            None,
        )
    } else if inner_stmt.starts_with("Picker(") {
        let kind = if inner_stmt.contains(".pickerStyle(.segmented)") {
            SceneNodeKind::SegmentedControl { segments: vec![] }
        } else {
            SceneNodeKind::PopUpButton { options: vec![] }
        };
        (kind, None, None, None)
    } else if inner_stmt.starts_with("Slider(") {
        (SceneNodeKind::Slider { min: 0.0, max: 100.0, tick_marks: 0 }, None, None, None)
    } else if inner_stmt.starts_with("Stepper(") {
        (SceneNodeKind::Stepper { step: 1.0 }, None, None, None)
    } else if inner_stmt.starts_with("Image(") {
        (SceneNodeKind::ImageView { icon_name: title.to_string(), is_template: true }, None, None, Some(title.to_string()))
    } else if inner_stmt.starts_with("TextEditor(") {
        (SceneNodeKind::TextView { monospace: false, line_numbers: false }, None, None, None)
    } else if inner_stmt.starts_with("OriginPointPicker(") {
        (SceneNodeKind::OriginMatrix, None, None, None)
    } else if inner_stmt.starts_with("ColorPicker(") {
        (SceneNodeKind::ColorWell, None, None, None)
    } else if inner_stmt.starts_with("DatePicker(") {
        (SceneNodeKind::DatePicker { clock_and_calendar: inner_stmt.contains(".datePickerStyle(.graphical)") }, None, None, None)
    } else if inner_stmt.starts_with("TabView(") {
        (SceneNodeKind::TabView { tabs: vec![] }, None, None, None)
    } else if inner_stmt.starts_with("DisclosureGroup(") {
        (SceneNodeKind::DisclosureGroup { trailing_add_button: inner_stmt.contains(".trailingAddButton(true)") }, None, None, None)
    } else if inner_stmt.starts_with("SearchField(") {
        (SceneNodeKind::SearchField, None, None, None)
    } else if inner_stmt.starts_with("ComboBox(") {
        (SceneNodeKind::ComboBox { options: vec![] }, None, None, None)
    } else if inner_stmt.starts_with("RuleEditor(") {
        (SceneNodeKind::RuleEditor, None, None, None)
    } else if inner_stmt.starts_with("Table(") {
        (SceneNodeKind::TableView { columns: vec![] }, None, None, None)
    } else if inner_stmt.starts_with("TagColorPicker(") {
        (SceneNodeKind::TagColorPicker, None, None, None)
    } else if inner_stmt.starts_with("TableActionBar(") {
        (SceneNodeKind::TableActionBar, None, None, None)
    } else if inner_stmt.starts_with("ProgressView(") {
        let icon = if inner_stmt.contains(".progressViewStyle(.circular)") { "NSProgressSpinning" } else { "NSProgressBar" };
        (SceneNodeKind::Slider { min: 0.0, max: 100.0, tick_marks: 0 }, None, None, Some(icon.into()))
    } else if inner_stmt.starts_with("Gauge(") {
        (SceneNodeKind::Slider { min: 0.0, max: 10.0, tick_marks: 0 }, None, None, Some("NSLevelIndicator".into()))
    } else {
        (SceneNodeKind::Label, None, None, None)
    }
}

fn is_control_statement(s: &str) -> bool {
    s.starts_with("Toggle(")
        || s.starts_with("TextField(")
        || s.starts_with("Button(")
        || s.starts_with("Picker(")
        || s.starts_with("Slider(")
        || s.starts_with("Stepper(")
        || s.starts_with("Image(")
        || s.starts_with("TextEditor(")
        || s.starts_with("OriginPointPicker(")
        || s.starts_with("ColorPicker(")
        || s.starts_with("DatePicker(")
        || s.starts_with("TabView(")
        || s.starts_with("DisclosureGroup(")
        || s.starts_with("SearchField(")
        || s.starts_with("ComboBox(")
        || s.starts_with("RuleEditor(")
        || s.starts_with("Table(")
        || s.starts_with("TagColorPicker(")
        || s.starts_with("TableActionBar(")
        || s.starts_with("ProgressView(")
        || s.starts_with("Gauge(")
        || s.starts_with("Text(")
}

fn extract_first_quoted(s: &str) -> Option<String> {
    let start = s.find('"')? + 1;
    let rest = &s[start..];
    let end = rest.find('"')?;
    Some(rest[..end].replace("\\\"", "\""))
}

fn extract_modifier_quoted(s: &str, prefix: &str) -> Option<String> {
    let idx = s.find(prefix)? + prefix.len();
    extract_first_quoted(&s[idx..])
}

fn extract_control_size(s: &str) -> String {
    if s.contains(".controlSize(.small)") {
        "small".into()
    } else if s.contains(".controlSize(.mini)") {
        "mini".into()
    } else {
        "regular".into()
    }
}

fn extract_frame_size(s: &str) -> Option<(f32, f32)> {
    let idx = s.find(".frame(")?;
    let rest = &s[idx..];
    let w_idx = rest.find("width:")? + "width:".len();
    let w_str: String = rest[w_idx..].trim_start().chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    let h_idx = rest.find("height:")? + "height:".len();
    let h_str: String = rest[h_idx..].trim_start().chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    Some((w_str.parse().ok()?, h_str.parse().ok()?))
}

fn extract_binding(s: &str, next_auto: &mut u32) -> (Option<BindingId>, Option<String>) {
    let Some(dollar_idx) = s.find('$') else {
        return (None, None);
    };
    let rest = &s[dollar_idx + 1..];
    let token: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_' || *c == '#').collect();
    if token.is_empty() || token == "model.unbound" {
        return (None, None);
    }
    if let Some((path, id_str)) = token.rsplit_once("_b") {
        if let Ok(id_num) = id_str.parse::<u32>() {
            return (Some(BindingId(id_num)), Some(path.to_string()));
        }
    }
    let bid = BindingId(*next_auto);
    *next_auto += 1;
    (Some(bid), Some(token))
}
