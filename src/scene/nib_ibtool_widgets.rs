//! Native AppKit Widget Class & Cell Decoder for `ibtool --compile` `NIBArchive` (`REQ-013-05.2`, `REQ-013-05.5`).

use crate::scene::ir::{CustomClassMapper, NodeId, SceneNode, SceneNodeKind, VisibilityRule};
use crate::scene::nib_ibtool::{IbDecodeState, IbNibGraph, IbVal};
use crate::scene::nib_varint::decode_cflags_control_size;
use std::collections::BTreeMap;

pub(crate) fn build_ibtool_node(st: &mut IbDecodeState<'_>, obj_i: usize, nid: NodeId, parent_id: NodeId) -> SceneNode {
    let g = st.graph;
    let raw_cls = g.obj_class(obj_i);
    let custom_cls = g.prop_val(obj_i, "NSClassName").and_then(|v| g.resolve_str(v));
    let orig_cls = g.prop_val(obj_i, "NSOriginalClassName").and_then(|v| g.resolve_str(v));
    let base_cls = orig_cls.as_deref().unwrap_or(raw_cls);
    let effective_cls = custom_cls.as_deref().unwrap_or(raw_cls);
    let rattrs = st.runtime_attrs.get(&obj_i);

    let ib_id = ["NSReuseIdentifierKey", "NSUserInterfaceItemIdentifier", "NSIdentifier", "NSID"]
        .into_iter()
        .find_map(|k| g.prop_val(obj_i, k).and_then(|v| g.resolve_str(v)))
        .unwrap_or_else(|| format!("_NS:{obj_i}"));

    let mut frame = g.resolve_rect(obj_i).unwrap_or([0.0, 0.0, 120.0, 22.0]);
    let root_flipped = st.nodes.first().is_some_and(|n| matches!(n.kind, SceneNodeKind::Container { is_flipped: true }));
    if !root_flipped {
        if let Some(p_node) = st.nodes.iter().find(|n| n.id == parent_id) {
            if matches!(p_node.kind, SceneNodeKind::BoxContainer)
                && frame[1] < p_node.nib_frame[1]
                && frame[1] + frame[3] <= p_node.nib_frame[3]
            {
                frame[0] += p_node.nib_frame[0];
                frame[1] += p_node.nib_frame[1] + 22.0;
            } else if p_node.parent.is_some() && !matches!(p_node.kind, SceneNodeKind::BoxContainer) {
                frame[0] += p_node.nib_frame[0];
                frame[1] += p_node.nib_frame[1];
            }
        }
    }
    let mask = g.prop_val(obj_i, "NSvFlags").and_then(IbVal::as_u32).map(|v| (v & 0x3F) as u8).unwrap_or(0);
    let cell_idx = g.prop_val(obj_i, "NSCell").and_then(IbVal::as_obj_ref);
    let title_cell_idx = cell_idx.or_else(|| g.prop_val(obj_i, "NSTitleCell").and_then(IbVal::as_obj_ref));
    let mut title = title_cell_idx
        .and_then(|c| g.prop_val(c, "NSContents").or_else(|| g.prop_val(c, "NSTitle")).or_else(|| g.prop_val(c, "NSPlaceholderString")))
        .or_else(|| {
            g.prop_val(obj_i, "NSTitle").or_else(|| g.prop_val(obj_i, "NSContents")).or_else(|| g.prop_val(obj_i, "NSPlaceholderString"))
        })
        .and_then(|v| g.resolve_str(v))
        .filter(|s| !s.is_empty())
        .or_else(|| title_cell_idx.and_then(|c| g.prop_val(c, "NSPlaceholderString")).and_then(|v| g.resolve_str(v)).filter(|s| s == "--"))
        .or_else(|| rattrs.and_then(|m| m.get("title")).cloned())
        .unwrap_or_default();

    let cflags = cell_idx.and_then(|c| g.prop_val(c, "NSCellFlags")).and_then(IbVal::as_u32).unwrap_or(0);
    let font_pt = cell_idx
        .and_then(|c| g.prop_val(c, "NSSupport").or_else(|| g.prop_val(c, "NSFont")))
        .and_then(IbVal::as_obj_ref)
        .and_then(|f_obj| g.prop_val(f_obj, "NSSize"))
        .and_then(|v| g.resolve_f64(v))
        .unwrap_or(0.0);
    let pi_sz = g.prop_val(obj_i, "NSpiFlags").and_then(IbVal::as_u32).map(|f| (f >> 8) & 0x07).unwrap_or(0);
    let ctrl_sz_code = cell_idx
        .and_then(|c| g.prop_val(c, "NSControlSize").or_else(|| g.prop_val(c, "NSControlSize2")))
        .or_else(|| g.prop_val(obj_i, "NSControlSize").or_else(|| g.prop_val(obj_i, "NSControlSize2")))
        .and_then(IbVal::as_u32);
    let control_size = match ctrl_sz_code {
        Some(0) => "regular",
        Some(1) => "small",
        Some(2) => "mini",
        Some(3) => "large",
        _ if ((cflags >> 18) & 0x07) == 1 || pi_sz == 1 || (9.5..=11.5).contains(&font_pt) => "small",
        _ if ((cflags >> 18) & 0x07) == 2 || pi_sz == 2 || (1.0..9.5).contains(&font_pt) => "mini",
        _ => decode_cflags_control_size(cflags),
    }
    .to_string();

    let (kind, icon_name) =
        decode_ibtool_text_and_list_kind(g, obj_i, cell_idx, base_cls, effective_cls, &mut title, cflags, rattrs, st.mapper)
            .unwrap_or_else(|| {
                decode_ibtool_button_and_value_kind(g, obj_i, cell_idx, base_cls, effective_cls, &mut title, rattrs, st.mapper)
            });
    if matches!(kind, SceneNodeKind::RuleEditor) && frame[3] < 10.0 {
        let row_h = g.prop_val(obj_i, "NSRowHeight").and_then(|v| g.resolve_f64(v)).unwrap_or(25.0) as f32;
        frame[3] = row_h * 4.0 + 18.0;
    }

    let mut node = crate::scene::nib_frontend::mk_node(
        nid.0,
        &ib_id,
        Some(parent_id.0),
        vec![],
        kind,
        &title,
        &g.prop_val(obj_i, "NSToolTip").and_then(|v| g.resolve_str(v)).unwrap_or_default(),
        &control_size,
        frame,
        mask,
        None,
        None,
        icon_name.as_deref(),
        VisibilityRule::Always,
    );
    node.enabled = (cflags & 0x2000_0000) == 0;
    node.populate_ibtool_typed_fields(g, obj_i, cell_idx, cflags, font_pt, raw_cls, base_cls, effective_cls, custom_cls.as_deref());
    node
}

fn decode_ibtool_text_and_list_kind(
    g: &IbNibGraph,
    obj_i: usize,
    cell_idx: Option<usize>,
    base_cls: &str,
    effective_cls: &str,
    title: &mut String,
    cflags: u32,
    rattrs: Option<&BTreeMap<String, String>>,
    mapper: &CustomClassMapper,
) -> Option<(SceneNodeKind, Option<String>)> {
    let explicit_icon = cell_idx
        .and_then(|c| g.prop_val(c, "NSImage").or_else(|| g.prop_val(c, "NSNormalImage")))
        .or_else(|| g.prop_val(obj_i, "NSImage"))
        .and_then(|v| g.resolve_str(v));
    let cell_cls = cell_idx.map(|c| g.obj_class(c)).unwrap_or_default();
    let draws_bg = cell_idx.and_then(|c| g.prop_val(c, "NSDrawsBackground")).and_then(IbVal::as_u32).unwrap_or(0) != 0;
    let fmt_idx =
        cell_idx.and_then(|c| g.prop_val(c, "NSFormatter")).or_else(|| g.prop_val(obj_i, "NSFormatter")).and_then(IbVal::as_obj_ref);
    let has_number_formatter = fmt_idx.is_some_and(|f| {
        g.obj_inherits(f, "NumberFormatter")
            || g.prop_val(f, "NS.min").is_some()
            || g.prop_val(f, "NS.max").is_some()
            || g.prop_val(f, "NS.attributes").is_some()
    });
    if has_number_formatter || mapper.is_stepping_number_field(effective_cls) {
        let attr_dict = fmt_idx.and_then(|f| g.prop_val(f, "NS.attributes")).and_then(IbVal::as_obj_ref);
        let min = fmt_idx.and_then(|f| g.prop_val(f, "NS.min").and_then(|v| g.resolve_f64(v))).or_else(|| attr_dict.and_then(|d| g.resolve_dict_f64(d, "minimum")));
        let max = fmt_idx.and_then(|f| g.prop_val(f, "NS.max").and_then(|v| g.resolve_f64(v))).or_else(|| attr_dict.and_then(|d| g.resolve_dict_f64(d, "maximum")));
        let step = rattrs.and_then(|m| m.get("step")).and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
        let shift_step = rattrs.and_then(|m| m.get("shiftStep")).and_then(|s| s.parse::<f64>().ok()).unwrap_or(10.0);
        let fmt_sfx = fmt_idx.and_then(|f| g.prop_val(f, "NS.positiveformat").or_else(|| g.prop_val(f, "NSPositiveFormat")).or_else(|| g.prop_val(f, "NSPositiveSuffix")))
            .and_then(|v| g.resolve_str(v)).map(|s| s.trim_start_matches(['0', '#', '.', ',', '-', '+']).to_string()).filter(|s| !s.is_empty());
        return Some((SceneNodeKind::NumberField { step, shift_step, min, max }, fmt_sfx.or(explicit_icon)));
    }
    if base_cls.contains("TokenField") || effective_cls.contains("TokenField") || g.obj_inherits(obj_i, "TokenField") {
        return Some((SceneNodeKind::TextField, Some("NSTokenField".into())));
    }
    if base_cls.contains("SearchField")
        || effective_cls.contains("SearchField")
        || cell_cls.contains("SearchField")
        || g.obj_inherits(obj_i, "SearchField")
    {
        if title.is_empty() {
            *title = "Search".to_string();
        }
        return Some((SceneNodeKind::SearchField, explicit_icon));
    }
    if base_cls.contains("TextField") || g.obj_inherits(obj_i, "TextField") || cell_cls.contains("TextFieldCell") {
        if (cflags & 0x40C0_0000) != 0 || draws_bg {
            return Some((SceneNodeKind::TextField, explicit_icon));
        }
        return Some((SceneNodeKind::Label, explicit_icon));
    }
    if base_cls.contains("PopUpButton") {
        let menu_idx = cell_idx.and_then(|c| g.prop_val(c, "NSMenu")).and_then(IbVal::as_obj_ref);
        let items_arr = menu_idx.and_then(|m| g.prop_val(m, "NSMenuItems")).and_then(IbVal::as_obj_ref);
        let options: Vec<String> = items_arr
            .map(|a| g.resolve_array_refs(a))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|mi| g.prop_val(mi, "NSTitle").and_then(|v| g.resolve_str(v)))
            .collect();
        if title.is_empty() {
            if let Some(first) = options.first() {
                *title = first.clone();
            }
        }
        return Some((SceneNodeKind::PopUpButton { options }, explicit_icon));
    }
    if base_cls.contains("ComboBox") {
        let list_arr = cell_idx.and_then(|c| g.prop_val(c, "NSPopUpListData")).and_then(IbVal::as_obj_ref);
        let options: Vec<String> = list_arr
            .map(|a| g.resolve_array_refs(a))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|si| g.resolve_str(&IbVal::ObjRef(si)))
            .collect();
        return Some((SceneNodeKind::ComboBox { options }, explicit_icon));
    }
    if base_cls.contains("SegmentedControl") {
        let seg_arr =
            cell_idx.and_then(|c| g.prop_val(c, "NSSegmentItems").or_else(|| g.prop_val(c, "NSSegmentImages"))).and_then(IbVal::as_obj_ref);
        let segments: Vec<String> = seg_arr
            .map(|a| g.resolve_array_refs(a))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|si| g.prop_val(si, "NSSegmentItemLabel").and_then(|v| g.resolve_str(v)))
            .collect();
        return Some((SceneNodeKind::SegmentedControl { segments }, explicit_icon));
    }
    if base_cls.contains("TabView") {
        let item_refs =
            g.prop_val(obj_i, "NSTabViewItems").and_then(IbVal::as_obj_ref).map(|a| g.resolve_array_refs(a)).unwrap_or_default();
        let tabs: Vec<String> = item_refs.iter().filter_map(|&ti| g.prop_val(ti, "NSLabel").and_then(|v| g.resolve_str(v))).collect();
        let tv_flags = g.prop_val(obj_i, "NStvFlags").and_then(IbVal::as_u32).unwrap_or(0);
        let tv_h = g.resolve_rect(obj_i).map(|r| r[3]).unwrap_or(0.0);
    let first_item_h = item_refs.first().and_then(|&ti| g.prop_val(ti, "NSView").and_then(IbVal::as_obj_ref)).and_then(|v| g.resolve_rect(v)).map(|r| r[3]).unwrap_or(0.0);
        let icon = explicit_icon.or_else(|| ((tv_flags & 0x07) >= 4 || (tv_h > 0.0 && first_item_h >= tv_h - 8.0)).then(|| "NSNoTabs".into()));
        return Some((SceneNodeKind::TabView { tabs }, icon));
    }
    if base_cls.contains("ScrollView") {
        if let Some(res) = crate::scene::nib_varint::decode_ibtool_scrollview_kind(g, obj_i, explicit_icon) {
            return Some(res);
        }
    }
    None
}

fn decode_ibtool_button_and_value_kind(
    g: &IbNibGraph,
    obj_i: usize,
    cell_idx: Option<usize>,
    base_cls: &str,
    effective_cls: &str,
    title: &mut String,
    rattrs: Option<&BTreeMap<String, String>>,
    mapper: &CustomClassMapper,
) -> (SceneNodeKind, Option<String>) {
    let explicit_icon = cell_idx
        .and_then(|c| {
            g.prop_val(c, "NSImage")
                .or_else(|| g.prop_val(c, "NSNormalImage"))
                .or_else(|| g.prop_val(c, "NSAlternateImage"))
                .or_else(|| g.prop_val(c, "NSOnImage"))
        })
        .or_else(|| g.prop_val(obj_i, "NSImage").or_else(|| g.prop_val(obj_i, "NSNormalImage")))
        .and_then(|v| g.resolve_str(v));
    if base_cls.contains("Button") {
        let cflags = cell_idx.and_then(|c| g.prop_val(c, "NSCellFlags")).and_then(IbVal::as_u32).unwrap_or(0);
        let cflags2 = cell_idx.and_then(|c| g.prop_val(c, "NSCellFlags2")).and_then(IbVal::as_u32).unwrap_or(0);
        let btn_flags2 = cell_idx.and_then(|c| g.prop_val(c, "NSButtonFlags2")).and_then(IbVal::as_u32).unwrap_or(0);
        let bezel = cell_idx.and_then(|c| g.prop_val(c, "NSBezelStyle")).and_then(IbVal::as_u32).unwrap_or(btn_flags2 & 0x1F);
        if explicit_icon.is_some() && matches!(title.as_str(), "0" | "1") {
            title.clear();
        }
        if effective_cls.contains("InlineActionButton") || (mapper.is_inline_button(effective_cls) && bezel != 1) {
            return (SceneNodeKind::InlineActionButton, explicit_icon);
        }
        let norm_img = cell_idx
            .and_then(|c| {
                g.prop_val(c, "NSNormalImage").or_else(|| g.prop_val(c, "NSOnImage")).or_else(|| g.prop_val(c, "NSAlternateImage"))
            })
            .and_then(|v| g.resolve_str(v))
            .unwrap_or_default();
        let has_mixed = cell_idx.and_then(|c| g.prop_val(c, "NSMixedImage")).is_some()
            || (cflags2 & 0x0100_0000) != 0
            || (btn_flags2 & 0x1000_0000) != 0;
        if bezel == 9 {
            if title.is_empty() {
                *title = "?".into();
            }
            return (SceneNodeKind::HelpButton, explicit_icon);
        }
        if (bezel == 11
            || effective_cls.contains("MenuButton")
            || (mapper.is_inline_button(effective_cls) && bezel != 1)
            || mapper.is_origin_matrix(effective_cls)
            || (bezel == 2 && explicit_icon.is_some() && !norm_img.contains("Switch") && !norm_img.contains("Radio") && (cflags & 0x00C0_0000) == 0))
            && title.is_empty()
            && !norm_img.contains("Switch")
            && !norm_img.contains("Radio")
        {
            return (SceneNodeKind::InlineActionButton, explicit_icon);
        }
        if bezel == 5 || bezel == 14 {
            return (SceneNodeKind::Checkbox, explicit_icon.or_else(|| Some("NSDisclosureButton".into())));
        }
        if norm_img.contains("Radio") {
            return (SceneNodeKind::RadioButton, None);
        }
        if norm_img.contains("Switch") || (bezel == 2 && (explicit_icon.is_none() || !title.is_empty())) {
            let icon = has_mixed.then(|| "NSMixedCheckbox".into()).or_else(|| explicit_icon.filter(|ic| ic != "NSSwitch"));
            return (SceneNodeKind::Checkbox, icon);
        }
        let key_equiv = cell_idx.and_then(|c| g.prop_val(c, "NSKeyEquivalent")).and_then(|v| g.resolve_str(v)).unwrap_or_default();
        return (SceneNodeKind::PushButton { is_default: key_equiv == "\r", is_cancel: key_equiv == "\u{1b}" }, explicit_icon);
    }
    if base_cls.contains("Slider") || base_cls.contains("LevelIndicator") || base_cls.contains("ProgressIndicator") {
        let min = cell_idx
            .and_then(|c| g.prop_val(c, "NSMinValue"))
            .or_else(|| g.prop_val(obj_i, "NSMinValue"))
            .and_then(|v| g.resolve_f64(v))
            .unwrap_or(0.0);
        let max = cell_idx
            .and_then(|c| g.prop_val(c, "NSMaxValue"))
            .or_else(|| g.prop_val(obj_i, "NSMaxValue"))
            .and_then(|v| g.resolve_f64(v))
            .unwrap_or(if base_cls.contains("ProgressIndicator") {
                1.0
            } else if base_cls.contains("LevelIndicator") {
                if min < 0.0 {
                    0.0
                } else {
                    5.0
                }
            } else {
                100.0
            });
        let ticks = cell_idx.and_then(|c| g.prop_val(c, "NSNumberOfTickMarks")).and_then(IbVal::as_u32).unwrap_or(0) as usize;
        let icon = explicit_icon.or_else(|| {
            if base_cls.contains("LevelIndicator") {
                let lstyle = cell_idx.and_then(|c| g.prop_val(c, "NSIndicatorStyle")).and_then(IbVal::as_u32).unwrap_or(0);
                Some(if lstyle == 3 { "NSRatingIndicator".into() } else { "NSLevelIndicator".into() })
            } else if base_cls.contains("ProgressIndicator") {
                let pi_flags = g.prop_val(obj_i, "NSpiFlags").and_then(IbVal::as_u32).unwrap_or(0);
                Some(if (pi_flags & 0x1000) != 0 { "NSProgressSpinning".into() } else { "NSProgressBar".into() })
            } else {
                (cell_idx.and_then(|c| g.prop_val(c, "NSSliderType")).and_then(IbVal::as_u32).unwrap_or(0) == 1)
                    .then(|| "NSCircularSlider".into())
            }
        });
        return (SceneNodeKind::Slider { min, max, tick_marks: ticks }, icon);
    }
    if base_cls.contains("Stepper") {
        let step = cell_idx
            .and_then(|c| g.prop_val(c, "NSIncrement"))
            .or_else(|| g.prop_val(obj_i, "NSIncrement"))
            .and_then(|v| g.resolve_f64(v))
            .unwrap_or(1.0);
        return (SceneNodeKind::Stepper { step }, explicit_icon);
    }
    if base_cls.contains("ColorWell") {
        return (SceneNodeKind::ColorWell, explicit_icon);
    }
    if base_cls.contains("DatePicker") {
        let dtype = cell_idx.and_then(|c| g.prop_val(c, "NSDatePickerType")).and_then(IbVal::as_u32).unwrap_or(0);
        return (
            SceneNodeKind::DatePicker { clock_and_calendar: explicit_icon.as_deref() == Some("clockAndCalendar") || dtype != 0 },
            None,
        );
    }
    if base_cls.contains("RuleEditor") {
        return (SceneNodeKind::RuleEditor, explicit_icon);
    }
    if base_cls.contains("TextView") {
        let mono = rattrs.and_then(|m| m.get("monospace")).is_some_and(|v| v == "YES");
        let lines = rattrs.and_then(|m| m.get("lineNumbers")).is_some_and(|v| v == "YES");
        return (SceneNodeKind::TextView { monospace: mono, line_numbers: lines }, explicit_icon);
    }
    if base_cls.contains("PathControl") || effective_cls.contains("PathControl") {
        let comp_arr = cell_idx
            .and_then(|c| g.prop_val(c, "NSPathComponentCells"))
            .or_else(|| g.prop_val(obj_i, "NSPathComponentCells"))
            .and_then(IbVal::as_obj_ref);
        let comps: Vec<String> = comp_arr
            .map(|a| g.resolve_array_refs(a))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|pc| g.prop_val(pc, "NSContents").or_else(|| g.prop_val(pc, "NSTitle")).and_then(|v| g.resolve_str(v)))
            .collect();
        if !comps.is_empty() {
            *title = comps.join(" / ");
        } else if title.is_empty() {
            if let Some(url_str) =
                cell_idx.and_then(|c| g.prop_val(c, "NSURL")).or_else(|| g.prop_val(obj_i, "NSURL")).and_then(|v| g.resolve_str(v))
            {
                *title = url_str;
            }
        }
        return (SceneNodeKind::Label, explicit_icon);
    }
    if base_cls.contains("Box") {
        let btype = g.prop_val(obj_i, "NSBoxType").and_then(IbVal::as_u32).unwrap_or(0);
        if btype == 2 {
            title.clear();
            return (SceneNodeKind::Separator, explicit_icon);
        }
        return (SceneNodeKind::BoxContainer, explicit_icon);
    }
    if base_cls.contains("ImageView") || effective_cls.contains("ImageView") || g.obj_inherits(obj_i, "ImageView") {
        let ic = explicit_icon.clone().unwrap_or_else(|| title.clone());
        let is_tmpl = ic.ends_with("Template") || ic.starts_with("NS") || !ic.contains('/');
        return (SceneNodeKind::ImageView { icon_name: ic, is_template: is_tmpl }, explicit_icon);
    }
    if mapper.is_origin_matrix(effective_cls) {
        return (SceneNodeKind::OriginMatrix, explicit_icon);
    }
    if effective_cls.contains("ColorLabel") || effective_cls.contains("TagColor") {
        return (SceneNodeKind::TagColorPicker, explicit_icon);
    }
    if effective_cls.contains("TableActionBar") {
        return (SceneNodeKind::TableActionBar, explicit_icon);
    }
    if effective_cls.contains("DisclosureGroup") {
        let add_btn = rattrs.and_then(|m| m.get("trailingAddButton")).is_some_and(|v| v == "YES");
        return (SceneNodeKind::DisclosureGroup { trailing_add_button: add_btn }, explicit_icon);
    }
    if base_cls.contains("VisualEffectView") || effective_cls.contains("VisualEffectView") || g.obj_inherits(obj_i, "VisualEffectView") {
        return (SceneNodeKind::Container { is_flipped: false }, explicit_icon);
    }
    (SceneNodeKind::CustomCanvasSlot { class_name: effective_cls.to_string() }, explicit_icon)
}

pub(crate) fn collect_ibtool_constraint_specs(st: &IbDecodeState<'_>) -> Vec<crate::scene::constraints::LayoutConstraintSpec> {
    use crate::scene::constraints::{ConstraintAttr, ConstraintRelation, LayoutConstraintSpec};
    let mut specs = Vec::new();
    for obj_i in 0..st.graph.objects.len() {
        let cls = st.graph.obj_class(obj_i);
        if !cls.contains("Constraint") && !st.graph.obj_inherits(obj_i, "LayoutConstraint") {
            continue;
        }
        let resolve_item = |raw: Option<usize>| -> Option<NodeId> {
            raw.and_then(|o| {
                st.obj_to_node_idx.get(&o).copied().or_else(|| {
                    if !st.graph.obj_inherits(o, "LayoutGuide") {
                        return None;
                    }
                    let other = st
                        .graph
                        .prop_val(obj_i, "NSFirstItem")
                        .and_then(IbVal::as_obj_ref)
                        .filter(|&x| x != o)
                        .or_else(|| st.graph.prop_val(obj_i, "NSSecondItem").and_then(IbVal::as_obj_ref).filter(|&x| x != o));
                    other
                        .and_then(|ot| st.graph.prop_val(ot, "NSSuperview").and_then(IbVal::as_obj_ref))
                        .and_then(|sv| st.obj_to_node_idx.get(&sv).copied())
                        .or_else(|| {
                            (0..st.graph.objects.len()).find_map(|p| {
                                (st.graph.prop_val(p, "IBNSSafeAreaLayoutGuide").and_then(IbVal::as_obj_ref) == Some(o)
                                    || st.graph.prop_val(p, "IBNSLayoutMarginsGuide").and_then(IbVal::as_obj_ref) == Some(o)
                                    || st
                                        .graph
                                        .prop_val(p, "NSViewLayoutGuides")
                                        .and_then(IbVal::as_obj_ref)
                                        .is_some_and(|a| st.graph.resolve_array_refs(a).contains(&o)))
                                .then(|| st.obj_to_node_idx.get(&p).copied())?
                            })
                        })
                })
            })
            .map(|ni| st.nodes[ni].id)
        };
        let first_item = resolve_item(st.graph.prop_val(obj_i, "NSFirstItem").and_then(IbVal::as_obj_ref));
        let raw_second = st.graph.prop_val(obj_i, "NSSecondItem").and_then(IbVal::as_obj_ref);
        let second_item = resolve_item(raw_second);
        if raw_second.is_some() && second_item.is_none() {
            continue;
        }
        let a1 = st.graph.prop_val(obj_i, "NSFirstAttribute").or_else(|| st.graph.prop_val(obj_i, "NSFirstAttributeV2")).and_then(IbVal::as_u32).unwrap_or(0) as i32;
        let a2 = st.graph.prop_val(obj_i, "NSSecondAttribute").or_else(|| st.graph.prop_val(obj_i, "NSSecondAttributeV2")).and_then(IbVal::as_u32).unwrap_or(0) as i32;
        let has_standard_sym = st.graph.prop_val(obj_i, "NSSymbolicConstant").is_some();
        let constant = st.graph.prop_val(obj_i, "NSConstant").or_else(|| st.graph.prop_val(obj_i, "NSConstantV2")).and_then(|v| st.graph.resolve_f64(v)).unwrap_or(if has_standard_sym { 8.0 } else { 0.0 }) as f32;
        let rel_code = st.graph.prop_val(obj_i, "NSRelation").and_then(IbVal::as_u32).unwrap_or(0) as i32;
        let relation = match rel_code {
            r if r < 0 => ConstraintRelation::LessThanOrEqual,
            r if r > 0 => ConstraintRelation::GreaterThanOrEqual,
            _ => ConstraintRelation::Equal,
        };
        let multiplier = st.graph.prop_val(obj_i, "NSMultiplier").and_then(|v| st.graph.resolve_f64(v)).unwrap_or(1.0) as f32;
        let priority = st.graph.prop_val(obj_i, "NSPriority").and_then(IbVal::as_u32).unwrap_or(1000).clamp(1, 1000) as u16;
        if first_item.is_some() && a1 != 0 {
            specs.push(LayoutConstraintSpec {
                first_item,
                first_attr: ConstraintAttr::from_code(a1),
                relation,
                second_item,
                second_attr: ConstraintAttr::from_code(a2),
                multiplier,
                constant,
                priority,
            });
        }
    }
    specs
}
