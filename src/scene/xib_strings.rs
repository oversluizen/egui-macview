//! Layer 1 XML `.xib` Frontend (`XibXmlFrontend` via `roxmltree`) and Apple `.strings`
//! Localization Catalog (`StringsCatalog` with text & `plist` binary support, `SPEC-013 §2.3..2.4`).

use crate::scene::constraints::{CompiledConstraintSchedule, ConstraintAttr, ConstraintRelation, LayoutConstraintSpec, ValueTransformer};
use crate::scene::ir::{ActionId, BindingId, MacSceneIR, NodeId, SceneCompileError, SceneNode, SceneNodeKind, VisibilityRule};
use std::collections::BTreeMap;

/// Strips Cocoa controller prefixes (`arrangedObjects.`, `selection.`, `values.`) (`REQ-013-02C` §3.1.3).
pub fn flatten_controller_key_path(raw: &str) -> String {
    for prefix in ["arrangedObjects.", "selection.", "values."] {
        if let Some(stripped) = raw.strip_prefix(prefix) {
            return stripped.to_string();
        }
    }
    raw.to_string()
}

/// Apple `<NibName>.strings` localization catalog (`"ObjectID.property" = "Localized";`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StringsCatalog {
    entries: BTreeMap<(String, String), String>,
}

impl StringsCatalog {
    /// Parses an Apple `.strings` text file (`/* comments */` and `"ObjectID.prop" = "Value";` lines).
    pub fn from_str(source: &str) -> Result<Self, SceneCompileError> {
        let mut entries = BTreeMap::new();
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
            let mut entries = BTreeMap::new();
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

/// Xcode Interface Builder XML `.xib` importer powered by `roxmltree` (`REQ-013-01` §2.3).
pub struct XibXmlFrontend;

impl XibXmlFrontend {
    /// Parses an Xcode `.xib` XML string into a normalized [`MacSceneIR`] via a zero-allocation `roxmltree::Document`.
    pub fn from_xml_str(xml: &str) -> Result<MacSceneIR, SceneCompileError> {
        let doc = roxmltree::Document::parse(xml).map_err(|e| SceneCompileError(format!("Invalid XIB XML: {e}")))?;
        let root_elem = doc.root_element();
        if !root_elem.has_tag_name("document") {
            return Err(SceneCompileError("Missing <document> root in XIB XML".into()));
        }

        let root_node = crate::scene::nib_frontend::mk_node(
            1,
            "root-view",
            None,
            vec![],
            SceneNodeKind::Container { is_flipped: false },
            "Window",
            "",
            "regular",
            [0.0, 0.0, 480.0, 320.0],
            18,
            None,
            None,
            None,
            VisibilityRule::Always,
        );
        let mut nodes = vec![root_node];
        let (mut next_id, mut next_binding, mut next_action) = (2u32, 1u32, 1u32);
        let mut ib_to_node = BTreeMap::from([("root-view".to_string(), NodeId(1))]);
        let mut binding_by_path: BTreeMap<String, BindingId> = BTreeMap::new();

        for elem in doc.descendants().filter(|n| n.is_element()) {
            let tag = elem.tag_name().name();
            if tag == "view" && elem.attribute("key") == Some("contentView") {
                if elem.parent().map(|p| p.has_tag_name("window")).unwrap_or(false) {
                    let is_flipped = elem.attribute("flipped") == Some("YES") || find_runtime_attr(elem, "ibFlipped").as_deref() == Some("YES");
                    nodes[0].kind = SceneNodeKind::Container { is_flipped };
                    if let Some(w_title) = elem.parent().and_then(|p| p.attribute("title")) {
                        nodes[0].title = w_title.to_string();
                    }
                    if let Some(r) = find_child_rect(elem) {
                        nodes[0].nib_frame = r;
                        nodes[0].alignment_rect = r;
                    }
                    if let Some(id_attr) = elem.attribute("id") {
                        nodes[0].ib_object_id = id_attr.to_string();
                        ib_to_node.insert(id_attr.to_string(), NodeId(1));
                    }
                }
                continue;
            }
            if tag == "tableView" && has_enclosing_tag(elem, "scrollView") {
                continue;
            }
            let Some(kind) = classify_xib_element(elem, tag) else {
                continue;
            };
            let ib_id = elem.attribute("id").map(str::to_string).unwrap_or_else(|| next_id.to_string());
            let rect = find_child_rect(elem).unwrap_or([0.0, 0.0, 100.0, 22.0]);
            let cell = elem.children().find(|c| c.tag_name().name().ends_with("Cell"));
            let mut title = cell
                .and_then(|c| c.attribute("title"))
                .or_else(|| elem.attribute("title"))
                .map(str::to_string)
                .or_else(|| find_runtime_attr(elem, "ibTitle"))
                .unwrap_or_default();
            if matches!(kind, SceneNodeKind::HelpButton) && title.is_empty() {
                title = "?".to_string();
            }
            let control_size = cell
                .and_then(|c| c.attribute("controlSize"))
                .or_else(|| elem.attribute("controlSize"))
                .map(str::to_string)
                .or_else(|| find_runtime_attr(elem, "ibSize"))
                .unwrap_or_else(|| "regular".to_string());
            let tooltip = elem.attribute("toolTip").unwrap_or("").to_string();
            let mask = parse_xib_autoresizing(elem);
            let enabled = elem.attribute("enabled") != Some("NO");
            let icon_meta = extract_xib_icon_meta(elem, cell, &kind);
            let own_conns = elem
                .children()
                .find(|c| c.has_tag_name("connections"))
                .or_else(|| cell.and_then(|c| c.children().find(|ch| ch.has_tag_name("connections"))));
            let (value_binding, binding_path) = own_conns
                .and_then(|c| c.children().find(|d| d.has_tag_name("binding")))
                .and_then(|b| b.attribute("keyPath"))
                .map(|kp| {
                    let norm = flatten_controller_key_path(kp);
                    let bid = *binding_by_path.entry(norm.clone()).or_insert_with(|| {
                        let b = BindingId(next_binding);
                        next_binding += 1;
                        b
                    });
                    (Some(bid), Some(norm))
                })
                .unwrap_or((None, None));
            let (action, action_selector) = own_conns
                .and_then(|c| c.children().find(|d| d.has_tag_name("action")))
                .and_then(|a| a.attribute("selector"))
                .map(|sel| {
                    let aid = ActionId(next_action);
                    next_action += 1;
                    (Some(aid), Some(sel.to_string()))
                })
                .unwrap_or((None, None));
            let parent_id = find_enclosing_parent_node(elem, &ib_to_node).unwrap_or(NodeId(1));
            let root_flipped = matches!(nodes[0].kind, SceneNodeKind::Container { is_flipped: true });
            let mut rect = rect;
            if !root_flipped {
                if let Some(p_node) = nodes.iter().find(|n| n.id == parent_id) {
                    if matches!(p_node.kind, SceneNodeKind::BoxContainer)
                        && rect[1] < p_node.nib_frame[1]
                        && rect[1] + rect[3] <= p_node.nib_frame[3]
                    {
                        rect[0] += p_node.nib_frame[0];
                        rect[1] += p_node.nib_frame[1] + 22.0;
                    }
                }
            }
            let id = NodeId(next_id);
            next_id += 1;
            ib_to_node.insert(ib_id.clone(), id);
            if let Some(p_node) = nodes.iter_mut().find(|n| n.id == parent_id) {
                p_node.children.push(id);
            }
            nodes.push(SceneNode {
                id,
                ib_object_id: ib_id,
                parent: Some(parent_id),
                children: vec![],
                kind,
                title,
                tooltip,
                control_size,
                nib_frame: rect,
                alignment_rect: rect,
                autoresizing_mask: mask,
                enabled,
                value_binding,
                binding_path,
                value_transformer: ValueTransformer::Identity,
                action,
                action_selector,
                icon_name: icon_meta,
                visibility: VisibilityRule::Always,
                labeled_by: None,
            });
        }
        let schedule = parse_xib_constraints(&doc, &ib_to_node);
        let root_w = nodes[0].nib_frame[2].max(320.0);
        let root_h = nodes[0].nib_frame[3].max(200.0);
        Ok(MacSceneIR::from_nodes(NodeId(1), [root_w, root_h], nodes, schedule))
    }
}

fn find_runtime_attr(elem: roxmltree::Node<'_, '_>, key_path: &str) -> Option<String> {
    elem.children()
        .find(|c| c.has_tag_name("userDefinedRuntimeAttributes"))?
        .children()
        .find(|a| a.has_tag_name("userDefinedRuntimeAttribute") && a.attribute("keyPath") == Some(key_path))?
        .attribute("value")
        .map(str::to_string)
}

fn has_enclosing_tag(elem: roxmltree::Node<'_, '_>, target_tag: &str) -> bool {
    std::iter::successors(elem.parent(), |p| p.parent()).any(|p| p.has_tag_name(target_tag))
}

fn find_enclosing_parent_node(elem: roxmltree::Node<'_, '_>, ib_to_node: &BTreeMap<String, NodeId>) -> Option<NodeId> {
    std::iter::successors(elem.parent(), |p| p.parent()).find_map(|p| p.attribute("id").and_then(|id_str| ib_to_node.get(id_str)).copied())
}

fn extract_xib_icon_meta(elem: roxmltree::Node<'_, '_>, cell: Option<roxmltree::Node<'_, '_>>, kind: &SceneNodeKind) -> Option<String> {
    if let Some(ic) = find_runtime_attr(elem, "ibIcon") {
        return Some(ic);
    }
    if let Some(img) = cell.and_then(|c| c.attribute("image")).or_else(|| elem.attribute("image")) {
        return Some(img.to_string());
    }
    match kind {
        SceneNodeKind::Label => {
            let mfont = cell.and_then(|c| c.children().find(|ch| ch.has_tag_name("font"))).and_then(|f| f.attribute("metaFont")).unwrap_or("");
            let w_fallback = if mfont.contains("Semibold") { "semibold" } else if mfont.contains("Bold") { "bold" } else if mfont.contains("Medium") { "medium" } else { "regular" };
            let w = elem.attribute("fontWeight").unwrap_or(w_fallback);
            let cname = cell.and_then(|c| c.children().find(|ch| ch.has_tag_name("color"))).and_then(|cl| cl.attribute("name")).unwrap_or("");
            let c_fallback = if cname.contains("secondary") { "secondary" } else if cname.contains("tertiary") { "tertiary" } else { "primary" };
            let c = elem.attribute("textColor").unwrap_or(c_fallback);
            (w != "regular" || c != "primary").then(|| format!("NSLabel:{w}:{c}"))
        }
        SceneNodeKind::Checkbox => {
            if elem.attribute("bezelStyle") == Some("disclosure") || cell.and_then(|c| c.attribute("bezelStyle")) == Some("disclosure") {
                Some("NSDisclosureButton".into())
            } else if elem.attribute("allowsMixedState") == Some("YES") || cell.and_then(|c| c.attribute("allowsMixedState")) == Some("YES") {
                Some("NSMixedCheckbox".into())
            } else {
                None
            }
        }
        SceneNodeKind::PopUpButton { .. } if elem.attribute("accentBadge") == Some("YES") => Some("NSAccentBadge".into()),
        SceneNodeKind::Slider { .. }
            if elem.attribute("sliderType") == Some("circular") || cell.and_then(|c| c.attribute("sliderType")) == Some("circular") =>
        {
            Some("NSCircularSlider".into())
        }
        SceneNodeKind::Slider { .. } if elem.has_tag_name("levelIndicator") => {
            let is_rating = elem.attribute("style") == Some("rating")
                || elem.attribute("levelIndicatorStyle") == Some("rating")
                || cell.and_then(|c| c.attribute("levelIndicatorStyle")) == Some("rating");
            Some(if is_rating { "NSRatingIndicator".into() } else { "NSLevelIndicator".into() })
        }
        SceneNodeKind::Slider { .. } if elem.has_tag_name("progressIndicator") => {
            Some(if elem.attribute("style") == Some("spinning") { "NSProgressSpinning".into() } else { "NSProgressBar".into() })
        }
        SceneNodeKind::TextField if elem.has_tag_name("tokenField") => Some("NSTokenField".into()),
        SceneNodeKind::ColorWell if elem.attribute("supportsAlpha") == Some("YES") => Some("NSColorWellAlpha".into()),
        _ => None,
    }
}

fn classify_xib_element(elem: roxmltree::Node<'_, '_>, tag: &str) -> Option<SceneNodeKind> {
    let cell = elem.children().find(|c| c.tag_name().name().ends_with("Cell"));
    match tag {
        "textField" => {
            if elem.attribute("customClass").is_some_and(|c| c.contains("SteppingTextField") || c.contains("NumberField")) {
                let nf = cell.and_then(|c| c.children().find(|ch| ch.has_tag_name("numberFormatter")));
                let real_val = |k: &str| nf.and_then(|f| f.children().find(|r| r.has_tag_name("real") && r.attribute("key") == Some(k))).and_then(|r| r.attribute("value"));
                let min = elem.attribute("minValue").or_else(|| nf.and_then(|f| f.attribute("minimum"))).or_else(|| real_val("minimum")).and_then(|s| s.parse::<f64>().ok());
                let max = elem.attribute("maxValue").or_else(|| nf.and_then(|f| f.attribute("maximum"))).or_else(|| real_val("maximum")).and_then(|s| s.parse::<f64>().ok());
                let step = elem.attribute("step").and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
                let shift_step = elem.attribute("shiftStep").and_then(|s| s.parse::<f64>().ok()).unwrap_or(10.0);
                return Some(SceneNodeKind::NumberField { min, max, step, shift_step });
            }
            let editable = cell.and_then(|c| c.attribute("editable")).or_else(|| elem.attribute("editable")) == Some("YES");
            Some(if editable { SceneNodeKind::TextField } else { SceneNodeKind::Label })
        }
        "tokenField" => Some(SceneNodeKind::TextField),
        "customView" => {
            let cls = elem.attribute("customClass").unwrap_or("");
            if cls.contains("OriginPoint") || cls.contains("TransformGrid") {
                Some(SceneNodeKind::OriginMatrix)
            } else if cls.contains("ColorLabel") || cls.contains("TagColor") {
                Some(SceneNodeKind::TagColorPicker)
            } else if cls.contains("TableActionBar") {
                Some(SceneNodeKind::TableActionBar)
            } else if cls.contains("DisclosureGroup") {
                let add = elem.attribute("trailingAddButton") == Some("YES")
                    || find_runtime_attr(elem, "trailingAddButton").as_deref() == Some("YES");
                Some(SceneNodeKind::DisclosureGroup { trailing_add_button: add })
            } else if !cls.is_empty() {
                Some(SceneNodeKind::CustomCanvasSlot { class_name: cls.to_string() })
            } else {
                None
            }
        }
        "colorWell" => Some(SceneNodeKind::ColorWell),
        "stepper" => {
            let step = elem
                .attribute("increment")
                .or_else(|| cell.and_then(|c| c.attribute("increment")))
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(1.0);
            Some(SceneNodeKind::Stepper { step })
        }
        "searchField" => Some(SceneNodeKind::SearchField),
        "comboBox" => {
            let options: Vec<String> = elem
                .descendants()
                .filter_map(|d| {
                    if d.has_tag_name("string") {
                        d.text().map(str::to_string)
                    } else if d.has_tag_name("menuItem") {
                        d.attribute("title").map(str::to_string)
                    } else {
                        None
                    }
                })
                .collect();
            Some(SceneNodeKind::ComboBox { options })
        }
        "button" => {
            let bstyle = cell.and_then(|c| c.attribute("bezelStyle")).or_else(|| elem.attribute("bezelStyle")).unwrap_or("");
            let cls = elem.attribute("customClass").unwrap_or("");
            let btype = cell.and_then(|c| c.attribute("type")).or_else(|| elem.attribute("type")).unwrap_or("push");
            if bstyle == "helpButton" {
                Some(SceneNodeKind::HelpButton)
            } else if cls.contains("InlineActionButton") || cls.contains("MouseOverButton") {
                Some(SceneNodeKind::InlineActionButton)
            } else if btype == "switch" || btype == "check" || bstyle == "disclosure" {
                Some(SceneNodeKind::Checkbox)
            } else if btype == "radio" {
                Some(SceneNodeKind::RadioButton)
            } else {
                let key_eq = cell.and_then(|c| c.attribute("keyEquivalent")).unwrap_or("");
                Some(SceneNodeKind::PushButton { is_default: key_eq == "\r" || key_eq == "&#13;", is_cancel: key_eq == "\u{1b}" })
            }
        }
        "disclosureGroup" => {
            Some(SceneNodeKind::DisclosureGroup { trailing_add_button: elem.attribute("trailingAddButton") == Some("YES") })
        }
        "datePicker" => Some(SceneNodeKind::DatePicker {
            clock_and_calendar: elem.attribute("style") == Some("clockAndCalendar")
                || elem.attribute("datePickerStyle") == Some("clockAndCalendar")
                || cell.and_then(|c| c.attribute("datePickerStyle")) == Some("clockAndCalendar"),
        }),
        "tabView" => {
            let tabs = elem
                .descendants()
                .filter(|d| d.has_tag_name("tabViewItem"))
                .filter_map(|t| t.attribute("label").map(str::to_string))
                .collect();
            Some(SceneNodeKind::TabView { tabs })
        }
        "ruleEditor" => Some(SceneNodeKind::RuleEditor),
        "scrollView" => {
            if let Some(tv) = elem.descendants().find(|d| d.has_tag_name("tableView")) {
                let mut cols = Vec::new();
                for col in tv.descendants().filter(|d| d.has_tag_name("tableColumn")) {
                    let w = col.attribute("width").and_then(|s| s.parse::<f32>().ok()).unwrap_or(100.0);
                    let hdr = col
                        .children()
                        .find(|c| c.has_tag_name("tableHeaderCell"))
                        .and_then(|h| h.attribute("title"))
                        .or_else(|| col.attribute("identifier"))
                        .unwrap_or("Column")
                        .to_string();
                    cols.push((hdr, w));
                }
                return Some(SceneNodeKind::TableView { columns: cols });
            }
            None
        }
        "popUpButton" => {
            let options = elem
                .descendants()
                .filter(|d| d.has_tag_name("menuItem"))
                .filter_map(|m| m.attribute("title").map(str::to_string))
                .collect();
            Some(SceneNodeKind::PopUpButton { options })
        }
        "segmentedControl" => {
            let segments =
                elem.descendants().filter(|d| d.has_tag_name("segment")).filter_map(|s| s.attribute("label").map(str::to_string)).collect();
            Some(SceneNodeKind::SegmentedControl { segments })
        }
        "slider" | "levelIndicator" | "progressIndicator" => {
            let min = elem
                .attribute("minValue")
                .or_else(|| cell.and_then(|c| c.attribute("minValue")))
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let max = elem
                .attribute("maxValue")
                .or_else(|| cell.and_then(|c| c.attribute("maxValue")))
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(100.0);
            let tick_marks = elem
                .attribute("tickMarks")
                .or_else(|| elem.attribute("numberOfTickMarks"))
                .or_else(|| cell.and_then(|c| c.attribute("numberOfTickMarks")))
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            Some(SceneNodeKind::Slider { min, max, tick_marks })
        }
        "switch" => Some(SceneNodeKind::Switch),
        "imageView" => Some(SceneNodeKind::ImageView { icon_name: String::new(), is_template: true }),
        "textView" => Some(SceneNodeKind::TextView {
            monospace: elem.attribute("monospace") == Some("YES") || find_runtime_attr(elem, "monospace").as_deref() == Some("YES"),
            line_numbers: elem.attribute("lineNumbers") == Some("YES") || find_runtime_attr(elem, "lineNumbers").as_deref() == Some("YES"),
        }),
        "box" => Some(if elem.attribute("boxType") == Some("separator") { SceneNodeKind::Separator } else { SceneNodeKind::BoxContainer }),
        _ => None,
    }
}

fn parse_xib_constraints(doc: &roxmltree::Document<'_>, ib_to_node: &BTreeMap<String, NodeId>) -> CompiledConstraintSchedule {
    let mut specs = Vec::new();
    for c in doc.descendants().filter(|d| d.has_tag_name("constraint")) {
        let first_item = c.attribute("firstItem").and_then(|id| ib_to_node.get(id).copied());
        let second_item = c.attribute("secondItem").and_then(|id| ib_to_node.get(id).copied());
        let first_attr = parse_attr_name(c.attribute("firstAttribute").unwrap_or(""));
        let second_attr = parse_attr_name(c.attribute("secondAttribute").unwrap_or(""));
        let constant = c.attribute("constant").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
        if first_item.is_some() && first_attr != ConstraintAttr::NotAnAttribute {
            specs.push(LayoutConstraintSpec {
                first_item,
                first_attr,
                relation: ConstraintRelation::Equal,
                second_item,
                second_attr,
                multiplier: 1.0,
                constant,
                priority: 1000,
            });
        }
    }
    CompiledConstraintSchedule::compile(specs)
}

fn parse_attr_name(s: &str) -> ConstraintAttr {
    match s {
        "leading" => ConstraintAttr::Leading,
        "left" => ConstraintAttr::Left,
        "trailing" => ConstraintAttr::Trailing,
        "right" => ConstraintAttr::Right,
        "top" => ConstraintAttr::Top,
        "bottom" => ConstraintAttr::Bottom,
        "width" => ConstraintAttr::Width,
        "height" => ConstraintAttr::Height,
        "centerX" => ConstraintAttr::CenterX,
        "centerY" => ConstraintAttr::CenterY,
        "firstBaseline" => ConstraintAttr::FirstBaseline,
        "baseline" | "lastBaseline" => ConstraintAttr::LastBaseline,
        _ => ConstraintAttr::NotAnAttribute,
    }
}

fn find_child_rect(elem: roxmltree::Node<'_, '_>) -> Option<[f32; 4]> {
    let r = elem
        .children()
        .find(|c| c.has_tag_name("rect") && c.attribute("key") == Some("frame"))
        .or_else(|| elem.children().find(|c| c.has_tag_name("rect")))?;
    Some([
        r.attribute("x")?.parse().ok()?,
        r.attribute("y")?.parse().ok()?,
        r.attribute("width")?.parse().ok()?,
        r.attribute("height")?.parse().ok()?,
    ])
}

fn parse_xib_autoresizing(elem: roxmltree::Node<'_, '_>) -> u8 {
    let Some(am) = elem.children().find(|c| c.has_tag_name("autoresizingMask")) else { return 0 };
    [
        ("flexibleMinX", 0x01),
        ("widthSizable", 0x02),
        ("flexibleMaxX", 0x04),
        ("flexibleMinY", 0x08),
        ("heightSizable", 0x10),
        ("flexibleMaxY", 0x20),
    ]
    .into_iter()
    .fold(0u8, |acc, (k, bit)| if am.attribute(k) == Some("YES") { acc | bit } else { acc })
}
