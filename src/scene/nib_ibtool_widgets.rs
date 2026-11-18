//! Native AppKit Widget Class & Cell Decoder for `ibtool --compile` `NIBArchive` (`REQ-013-05.2`, `REQ-013-05.5`).

use crate::scene::constraints::ValueTransformer;
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
        .unwrap_or_else(|| format!("nib-{obj_i}"));

    let mut frame = g.resolve_rect(obj_i).unwrap_or([0.0, 0.0, 120.0, 22.0]);
    let mask = g.prop_val(obj_i, "NSvFlags").and_then(IbVal::as_u32).map(|v| (v & 0x3F) as u8).unwrap_or(0);
    let cell_idx = g.prop_val(obj_i, "NSCell").and_then(IbVal::as_obj_ref);
    let title_cell_idx = cell_idx.or_else(|| g.prop_val(obj_i, "NSTitleCell").and_then(IbVal::as_obj_ref));
    let mut title = title_cell_idx
        .and_then(|c| g.prop_val(c, "NSContents").or_else(|| g.prop_val(c, "NSTitle")))
        .or_else(|| g.prop_val(obj_i, "NSTitle").or_else(|| g.prop_val(obj_i, "NSContents")))
        .and_then(|v| g.resolve_str(v))
        .or_else(|| rattrs.and_then(|m| m.get("title")).cloned())
        .unwrap_or_default();

    let cflags = cell_idx.and_then(|c| g.prop_val(c, "NSCellFlags")).and_then(IbVal::as_u32).unwrap_or(0);
    let cflags2 = cell_idx.and_then(|c| g.prop_val(c, "NSCellFlags2")).and_then(IbVal::as_u32).unwrap_or(0);
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
        _ if ((cflags2 >> 26) & 0x07) == 1 || ((cflags >> 18) & 0x07) == 1 || pi_sz == 1 => "small",
        _ if ((cflags2 >> 26) & 0x07) == 2 || ((cflags >> 18) & 0x07) == 2 || pi_sz == 2 => "mini",
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

    SceneNode {
        id: nid,
        ib_object_id: ib_id,
        parent: Some(parent_id),
        children: vec![],
        kind,
        title,
        tooltip: g.prop_val(obj_i, "NSToolTip").and_then(|v| g.resolve_str(v)).unwrap_or_default(),
        control_size,
        nib_frame: frame,
        alignment_rect: frame,
        autoresizing_mask: mask,
        enabled: (cflags & 0x2000_0000) == 0,
        value_binding: None,
        binding_path: None,
        value_transformer: ValueTransformer::Identity,
        action: None,
        action_selector: None,
        icon_name,
        visibility: VisibilityRule::Always,
        labeled_by: None,
    }
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
    if effective_cls.contains("SteppingTextField") || mapper.is_stepping_number_field(effective_cls) {
        let fmt_idx = cell_idx.and_then(|c| g.prop_val(c, "NSFormatter")).and_then(IbVal::as_obj_ref);
        let attr_dict = fmt_idx.and_then(|f| g.prop_val(f, "NS.attributes")).and_then(IbVal::as_obj_ref);
        let min = fmt_idx
            .and_then(|f| g.prop_val(f, "NS.min").and_then(|v| g.resolve_f64(v)))
            .or_else(|| attr_dict.and_then(|d| g.resolve_dict_f64(d, "minimum")));
        let max = fmt_idx
            .and_then(|f| g.prop_val(f, "NS.max").and_then(|v| g.resolve_f64(v)))
            .or_else(|| attr_dict.and_then(|d| g.resolve_dict_f64(d, "maximum")));
        let step = rattrs.and_then(|m| m.get("step")).and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
        let shift_step = rattrs.and_then(|m| m.get("shiftStep")).and_then(|s| s.parse::<f64>().ok()).unwrap_or(10.0);
        return Some((SceneNodeKind::NumberField { step, shift_step, min, max }, explicit_icon));
    }
    if base_cls.contains("TokenField") || effective_cls.contains("TokenField") {
        return Some((SceneNodeKind::TextField, Some("NSTokenField".into())));
    }
    if base_cls.contains("TextField") {
        let draws_bg = cell_idx.and_then(|c| g.prop_val(c, "NSDrawsBackground")).and_then(IbVal::as_u32).unwrap_or(0) != 0;
        if (cflags & 0x4000_0000) != 0 || draws_bg {
            return Some((SceneNodeKind::TextField, explicit_icon));
        }
        let font_name = cell_idx
            .and_then(|c| g.prop_val(c, "NSSupport").or_else(|| g.prop_val(c, "NSFont")))
            .and_then(|v| g.resolve_str(v))
            .unwrap_or_default();
        let color_name = cell_idx.and_then(|c| g.prop_val(c, "NSTextColor")).and_then(|v| g.resolve_str(v)).unwrap_or_default();
        let w = rattrs.and_then(|m| m.get("fontWeight")).map(String::as_str).unwrap_or_else(|| {
            if font_name.contains("Demi") || font_name.contains("Semibold") {
                "semibold"
            } else if font_name.contains("Bold") {
                "bold"
            } else if font_name.contains("Medium") {
                "medium"
            } else {
                "regular"
            }
        });
        let c = rattrs.and_then(|m| m.get("textColor")).map(String::as_str).unwrap_or_else(|| {
            if color_name.contains("secondary") {
                "secondary"
            } else if color_name.contains("tertiary") {
                "tertiary"
            } else {
                "primary"
            }
        });
        let icon = explicit_icon.or_else(|| (w != "regular" || c != "primary").then(|| format!("NSLabel:{w}:{c}")));
        return Some((SceneNodeKind::Label, icon));
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
        let tab_arr = g.prop_val(obj_i, "NSTabViewItems").and_then(IbVal::as_obj_ref);
        let tabs: Vec<String> = tab_arr
            .map(|a| g.resolve_array_refs(a))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|ti| g.prop_val(ti, "NSLabel").and_then(|v| g.resolve_str(v)))
            .collect();
        return Some((SceneNodeKind::TabView { tabs }, explicit_icon));
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
        .and_then(|c| g.prop_val(c, "NSImage"))
        .or_else(|| g.prop_val(obj_i, "NSImage"))
        .and_then(|v| g.resolve_str(v));
    if base_cls.contains("Button") {
        if effective_cls.contains("InlineActionButton") || mapper.is_inline_button(effective_cls) {
            return (SceneNodeKind::InlineActionButton, explicit_icon);
        }
        let cflags2 = cell_idx.and_then(|c| g.prop_val(c, "NSCellFlags2")).and_then(IbVal::as_u32).unwrap_or(0);
        let btn_flags2 = cell_idx.and_then(|c| g.prop_val(c, "NSButtonFlags2")).and_then(IbVal::as_u32).unwrap_or(0);
        let bezel = cell_idx.and_then(|c| g.prop_val(c, "NSBezelStyle")).and_then(IbVal::as_u32).unwrap_or(btn_flags2 & 0x1F);
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
        if bezel == 5 || bezel == 14 {
            return (SceneNodeKind::Checkbox, explicit_icon.or_else(|| Some("NSDisclosureButton".into())));
        }
        if norm_img.contains("Radio") {
            return (SceneNodeKind::RadioButton, explicit_icon);
        }
        if norm_img.contains("Switch") || bezel == 2 {
            let icon = explicit_icon.or_else(|| has_mixed.then(|| "NSMixedCheckbox".into()));
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
            .unwrap_or_else(|| {
                if base_cls.contains("ProgressIndicator") {
                    1.0
                } else if base_cls.contains("LevelIndicator") {
                    if min < 0.0 {
                        0.0
                    } else {
                        5.0
                    }
                } else {
                    100.0
                }
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
                let stype = cell_idx.and_then(|c| g.prop_val(c, "NSSliderType")).and_then(IbVal::as_u32).unwrap_or(0);
                (stype == 1).then(|| "NSCircularSlider".into())
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
    if base_cls.contains("Box") {
        let btype = g.prop_val(obj_i, "NSBoxType").and_then(IbVal::as_u32).unwrap_or(0);
        if btype == 2 {
            title.clear();
            return (SceneNodeKind::Separator, explicit_icon);
        }
        return (SceneNodeKind::BoxContainer, explicit_icon);
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
    (SceneNodeKind::CustomCanvasSlot { class_name: effective_cls.to_string() }, explicit_icon)
}
