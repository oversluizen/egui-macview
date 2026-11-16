//! Extended AppKit/AVKit Widget Renderers for [`MacSceneIR::show`] (`SPEC-013` §4, `REQ-013-02P`).

use crate::controls::{
    MacCheckbox, MacCircularSlider, MacCollectionView, MacCollectionViewFlowLayout, MacColorWell, MacComboBox, MacControlSize, MacDate,
    MacDatePicker, MacDatePickerElementFlags, MacDatePickerStyle, MacDisclosureButton, MacDisclosureGroup, MacFontWeight, MacHelpButton,
    MacImageView, MacInlineActionButton, MacLabel, MacLabelColor, MacLevelIndicator, MacLevelIndicatorStyle, MacOriginAnchor,
    MacOriginMatrix, MacPlayerControlsStyle, MacPlayerView, MacProgressIndicator, MacRatingIndicator, MacRuleEditor, MacRuleRow, MacSlider,
    MacStepper, MacTabStyle, MacTabView, MacTableActionBar, MacTagColorPicker, MacTokenField,
};
use crate::scene::ir::{SceneNode, SceneNodeKind};
use crate::scene::runtime::MacSceneDataSource;
use crate::theme::MacTheme;
use egui::{Rect, Ui, UiBuilder};
use std::collections::BTreeSet;

pub(crate) fn render_label_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    ctrl_size: MacControlSize,
) {
    let dyn_txt = node.value_binding.map(|b| data_source.get_str(b)).unwrap_or("");
    let text = if dyn_txt.is_empty() { node.title.as_str() } else { dyn_txt };
    let meta = node.icon_name.as_deref().unwrap_or("");
    let weight = if meta.contains("semibold") {
        MacFontWeight::Semibold
    } else if meta.contains("bold") {
        MacFontWeight::Bold
    } else if meta.contains("medium") {
        MacFontWeight::Medium
    } else {
        MacFontWeight::Regular
    };
    let color = if meta.contains("secondary") {
        MacLabelColor::Secondary
    } else if meta.contains("tertiary") {
        MacLabelColor::Tertiary
    } else {
        MacLabelColor::Primary
    };
    let label_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() + 28.0, rect.height()));
    MacLabel::new(text).weight(weight).color(color).control_size(ctrl_size).theme(theme).fixed_rect(label_rect).show(ui);
}

pub(crate) fn render_checkbox_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    ctrl_size: MacControlSize,
    enabled: bool,
) {
    let fb_id = ui.scope_id().with(("nib_chk", node.id.0));
    let mut val = node
        .value_binding
        .map(|b| node.value_transformer.apply_bool(data_source.get_bool(b)))
        .unwrap_or_else(|| ui.data(|d| d.get_temp::<bool>(fb_id)).unwrap_or(false));
    let meta = node.icon_name.as_deref().unwrap_or("");
    let changed = if meta == "NSDisclosureButton" {
        let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_disc_btn", node.id.0)).max_rect(rect));
        c_ui.add_enabled_ui(enabled, |en_ui| en_ui.add(MacDisclosureButton::new(&mut val).control_size(ctrl_size).theme(theme)).changed())
            .inner
    } else {
        let is_mixed = meta == "NSMixedCheckbox" && !val;
        MacCheckbox::new(&mut val, node.title.as_str())
            .theme(theme)
            .control_size(ctrl_size)
            .mixed(is_mixed)
            .enabled(enabled)
            .fixed_rect(rect)
            .show(ui)
            .changed()
    };
    if changed && enabled {
        if let Some(b) = node.value_binding {
            data_source.set_bool(b, node.value_transformer.apply_bool(val));
        } else {
            ui.data_mut(|d| d.insert_temp(fb_id, val));
        }
        if let Some(act) = node.action {
            data_source.on_action(act, node.id);
        }
    }
}

pub(crate) fn render_slider_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    bounds: (f64, f64, usize),
    (ctrl_size, enabled): (MacControlSize, bool),
) {
    let (min, max, tick_marks) = bounds;
    let fb_id = ui.scope_id().with(("nib_sld", node.id.0));
    let mut val = node
        .value_binding
        .map(|b| data_source.get_f64(b) as f32)
        .unwrap_or_else(|| ui.data(|d| d.get_temp::<f32>(fb_id)).unwrap_or(min as f32));
    let changed = match node.icon_name.as_deref() {
        Some("NSCircularSlider") => {
            let diam = rect.width().min(rect.height()).max(28.0);
            let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_circ_ui", node.id.0)).max_rect(rect));
            c_ui.add_enabled_ui(enabled, |en_ui| {
                MacCircularSlider::new(&mut val, (min as f32)..=(max as f32)).theme(theme).diameter(diam).show(en_ui).changed()
            })
            .inner
        }
        Some("NSRatingIndicator") => {
            let mut rating = val.round().clamp(0.0, max as f32) as u32;
            let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_rate_ui", node.id.0)).max_rect(rect));
            let r_ch = c_ui
                .add_enabled_ui(enabled, |en_ui| {
                    MacRatingIndicator::new(&mut rating, (max as u32).max(1)).theme(theme).show(en_ui).changed()
                })
                .inner;
            if r_ch {
                val = rating as f32;
            }
            r_ch
        }
        Some("NSLevelIndicator") => {
            let style = if tick_marks > 0 { MacLevelIndicatorStyle::Discrete } else { MacLevelIndicatorStyle::Continuous };
            let span = (max - min).max(1.0) as f32;
            let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_lvl_ui", node.id.0)).max_rect(rect));
            MacLevelIndicator::new(val, (min as f32)..=(max as f32))
                .style(style)
                .segments(tick_marks.max(12))
                .warning_value(min as f32 + span * 0.65)
                .critical_value(min as f32 + span * 0.85)
                .peak_hold((val + span * 0.08).min(max as f32))
                .length(rect.width())
                .theme(theme)
                .show(&mut c_ui);
            false
        }
        Some("NSProgressSpinning") => {
            let spin_t = ui.input(|i| i.time.fract());
            let prog_size = if matches!(ctrl_size, MacControlSize::Small | MacControlSize::Mini) {
                crate::controls::MacProgressControlSize::Small
            } else {
                crate::controls::MacProgressControlSize::Regular
            };
            let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_spin_ui", node.id.0)).max_rect(rect));
            MacProgressIndicator::spinning(spin_t).control_size(prog_size).theme(theme).show(&mut c_ui);
            false
        }
        Some("NSProgressBar") => {
            let frac = if max > 1.0 {
                (val / max as f32).clamp(0.0, 1.0)
            } else if val > 1.0 {
                (val / 100.0).clamp(0.0, 1.0)
            } else {
                val.clamp(0.0, 1.0)
            };
            let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_pbar_ui", node.id.0)).max_rect(rect));
            MacProgressIndicator::bar(frac).width(rect.width()).show_percentage(false).theme(theme).show(&mut c_ui);
            false
        }
        _ => MacSlider::new(&mut val, (min as f32)..=(max as f32))
            .theme(theme)
            .control_size(ctrl_size)
            .enabled(enabled)
            .tick_marks(tick_marks)
            .fixed_rect(rect)
            .show(ui)
            .changed(),
    };
    if changed && enabled {
        if let Some(b) = node.value_binding {
            data_source.set_f64(b, val as f64);
        } else {
            ui.data_mut(|d| d.insert_temp(fb_id, val));
        }
        if let Some(act) = node.action {
            data_source.on_action(act, node.id);
        }
    }
}

pub(crate) fn render_extended_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    ctrl_size: MacControlSize,
    enabled: bool,
) {
    match &node.kind {
        SceneNodeKind::ComboBox { options } => {
            let fb_id = ui.scope_id().with(("nib_cb_str", node.id.0));
            let mut text = node
                .value_binding
                .map(|b| data_source.get_str(b).to_string())
                .unwrap_or_else(|| ui.data(|d| d.get_temp::<String>(fb_id)).unwrap_or_else(|| node.title.clone()));
            let refs: Vec<&str> = options.iter().map(String::as_str).collect();
            let changed = ui
                .add_enabled_ui(enabled, |en_ui| {
                    MacComboBox::new((en_ui.scope_id(), "nib_cb", node.id.0), &mut text, &refs)
                        .theme(theme)
                        .control_size(ctrl_size)
                        .fixed_rect(rect)
                        .show(en_ui)
                        .changed()
                })
                .inner;
            if changed && enabled {
                if let Some(b) = node.value_binding {
                    data_source.set_str(b, &text);
                } else {
                    ui.data_mut(|d| d.insert_temp(fb_id, text));
                }
            }
        }
        SceneNodeKind::ColorWell => {
            let mut col = node.value_binding.and_then(|b| data_source.get_color(b)).unwrap_or_else(|| theme.control_accent());
            let has_alpha = node.icon_name.as_deref() == Some("NSColorWellAlpha");
            if MacColorWell::new(&mut col)
                .id_salt((ui.scope_id(), "nib_cw", node.id.0))
                .fixed_rect(rect)
                .show_alpha(has_alpha)
                .enabled(enabled)
                .theme(theme)
                .show(ui)
                .changed()
                && enabled
            {
                if let Some(b) = node.value_binding {
                    data_source.set_color(b, col);
                }
                if let Some(act) = node.action {
                    data_source.on_action(act, node.id);
                }
            }
        }
        SceneNodeKind::DisclosureGroup { trailing_add_button } => {
            let mut open = node.value_binding.map(|b| data_source.get_bool(b)).unwrap_or(true);
            let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_dg_ui", node.id.0)).max_rect(rect));
            let resp = MacDisclosureGroup::new((c_ui.scope_id(), "nib_dg", node.id.0), node.title.as_str(), &mut open)
                .theme(theme)
                .with_trailing_add_button(*trailing_add_button)
                .show(&mut c_ui, |_| {});
            if let Some(b) = node.value_binding {
                data_source.set_bool(b, open);
            }
            if resp.trailing_clicked {
                if let Some(act) = node.action {
                    data_source.on_action(act, node.id);
                }
            }
        }
        SceneNodeKind::DatePicker { clock_and_calendar } => {
            let dp_id = ui.scope_id().with(("nib_dp_val", node.id.0));
            let mut date = ui.data(|d| d.get_temp::<MacDate>(dp_id)).unwrap_or_default();
            if let Some(bound_day) = node.value_binding.map(|b| data_source.get_i64(b)).filter(|d| (1..=31).contains(d)) {
                date.day = bound_day as u32;
            }
            let prev_day = date.day;
            let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_dp_ui", node.id.0)).max_rect(rect));
            c_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
            let styles: &[MacDatePickerStyle] = if *clock_and_calendar {
                &[MacDatePickerStyle::TextFieldAndStepper, MacDatePickerStyle::ClockAndCalendar]
            } else {
                &[MacDatePickerStyle::TextFieldAndStepper]
            };
            for &st in styles {
                c_ui.add(
                    MacDatePicker::new()
                        .date(&mut date)
                        .style(st)
                        .elements(MacDatePickerElementFlags::DateAndTime)
                        .control_size(ctrl_size)
                        .theme(theme),
                );
            }
            if let Some(b) = node.value_binding.filter(|_| date.day != prev_day) {
                data_source.set_i64(b, date.day as i64);
            }
            ui.data_mut(|d| d.insert_temp(dp_id, date));
        }
        SceneNodeKind::TabView { .. } => {
            render_tab_view_node(ui, theme, data_source, node, rect, ctrl_size);
        }
        SceneNodeKind::RuleEditor => {
            let re_id = ui.scope_id().with(("nib_re_rows", node.id.0));
            let mut rules = ui
                .data(|d| d.get_temp::<Vec<MacRuleRow>>(re_id))
                .unwrap_or_else(|| vec![MacRuleRow::new(vec![0, 0, 0]), MacRuleRow::new(vec![1, 1, 1])]);
            let prev_len = rules.len();
            const CRITERIA: &[&[&str]] = &[
                &["Name", "Kind", "Created Date", "File Size"],
                &["contains", "is exactly", "begins with", "ends with"],
                &["ProRes", "MOV", "HEVC", "PNG"],
            ];
            let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_re_ui", node.id.0)).max_rect(rect));
            c_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
            MacRuleEditor::new((c_ui.scope_id(), "nib_re", node.id.0), &mut rules, CRITERIA).theme(theme).show(&mut c_ui);
            if rules.len() != prev_len {
                if let Some(b) = node.value_binding {
                    data_source.set_i64(b, rules.len() as i64);
                }
            }
            ui.data_mut(|d| d.insert_temp(re_id, rules));
        }
        SceneNodeKind::TableActionBar => {
            let items: Vec<String> = node
                .value_binding
                .map(|b| data_source.get_options(b).to_vec())
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| vec!["Transform Layer".into(), "Color Tag".into()]);
            let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_tab_bar_ui", node.id.0)).max_rect(rect));
            c_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
            c_ui.horizontal(|h_ui| {
                for item in &items {
                    MacLabel::new(item.as_str())
                        .control_size(MacControlSize::Small)
                        .color(MacLabelColor::Secondary)
                        .theme(theme)
                        .show(h_ui);
                }
            });
            let resp = MacTableActionBar::new((c_ui.scope_id(), "nib_tab_bar", node.id.0))
                .theme(theme)
                .can_remove(!items.is_empty())
                .show(&mut c_ui);
            if resp.add_clicked || resp.remove_clicked {
                if let Some(b) = node.value_binding {
                    data_source.set_i64(b, if resp.add_clicked { 1 } else { -1 });
                }
                if let Some(act) = node.action {
                    data_source.on_action(act, node.id);
                }
            }
        }
        SceneNodeKind::OriginMatrix => {
            let fb_id = ui.scope_id().with(("nib_org", node.id.0));
            let idx = node
                .value_binding
                .map(|b| data_source.get_i64(b).clamp(0, 8) as usize)
                .unwrap_or_else(|| ui.data(|d| d.get_temp::<usize>(fb_id)).unwrap_or(4));
            let mut anchor = MacOriginAnchor::ALL[idx];
            if MacOriginMatrix::new(&mut anchor).theme(theme).control_size(ctrl_size).enabled(enabled).fixed_rect(rect).show(ui).changed() {
                let new_idx = MacOriginAnchor::ALL.iter().position(|&a| a == anchor).unwrap_or(4);
                if let Some(b) = node.value_binding {
                    data_source.set_i64(b, new_idx as i64);
                } else {
                    ui.data_mut(|d| d.insert_temp(fb_id, new_idx));
                }
                if let Some(act) = node.action {
                    data_source.on_action(act, node.id);
                }
            }
        }
        SceneNodeKind::HelpButton => {
            if MacHelpButton::new().theme(theme).control_size(ctrl_size).enabled(enabled).fixed_rect(rect).show(ui).clicked() {
                if let Some(act) = node.action {
                    data_source.on_action(act, node.id);
                }
            }
        }
        SceneNodeKind::InlineActionButton => {
            if MacInlineActionButton::new(node.title.as_str()).theme(theme).enabled(enabled).fixed_rect(rect).show(ui).clicked() {
                if let Some(act) = node.action {
                    data_source.on_action(act, node.id);
                }
            }
        }
        SceneNodeKind::ImageView { icon_name, is_template } => {
            MacImageView::from_resource_name(icon_name).theme(theme).template(*is_template).fixed_rect(rect).show(ui);
        }
        SceneNodeKind::Stepper { step } => {
            let fb_id = ui.scope_id().with(("nib_stp", node.id.0));
            let mut val =
                node.value_binding.map(|b| data_source.get_f64(b)).unwrap_or_else(|| ui.data(|d| d.get_temp::<f64>(fb_id)).unwrap_or(0.0));
            let mut stp_ui = ui.new_child(UiBuilder::new().id_salt(("nib_stp_ui", node.id.0)).max_rect(rect));
            if MacStepper::new(&mut val, -10_000.0..=10_000.0, *step)
                .width(rect.width())
                .height(rect.height())
                .theme(theme)
                .enabled(enabled)
                .show(&mut stp_ui)
                .changed()
            {
                if let Some(b) = node.value_binding {
                    data_source.set_f64(b, val);
                } else {
                    ui.data_mut(|d| d.insert_temp(fb_id, val));
                }
                if let Some(act) = node.action {
                    data_source.on_action(act, node.id);
                }
            }
        }
        SceneNodeKind::TagColorPicker => {
            let fb_id = ui.scope_id().with(("nib_tag", node.id.0));
            let mut tag = node
                .value_binding
                .and_then(|b| (data_source.get_i64(b) >= 0).then(|| data_source.get_i64(b) as usize))
                .or_else(|| ui.data(|d| d.get_temp::<Option<usize>>(fb_id)).flatten());
            if MacTagColorPicker::new(&mut tag).theme(theme).enabled(enabled).fixed_rect(rect).show(ui).changed() {
                if let Some(b) = node.value_binding {
                    data_source.set_i64(b, tag.map(|i| i as i64).unwrap_or(-1));
                } else {
                    ui.data_mut(|d| d.insert_temp(fb_id, tag));
                }
            }
        }
        _ => {}
    }
}

pub(crate) fn render_token_field_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    enabled: bool,
) {
    let raw = node.value_binding.map(|b| data_source.get_str(b).to_string()).unwrap_or_else(|| node.title.clone());
    let mut tokens: Vec<String> = raw.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
    let prev = tokens.clone();
    let draft_id = ui.scope_id().with(("nib_tok_draft", node.id.0));
    let mut draft = ui.data(|d| d.get_temp::<String>(draft_id)).unwrap_or_default();
    let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_tok_ui", node.id.0)).max_rect(rect));
    c_ui.add_enabled_ui(enabled, |en_ui| {
        MacTokenField::new((en_ui.scope_id(), "nib_tok", node.id.0), &mut tokens, &mut draft)
            .max_chars(18)
            .width(rect.width())
            .theme(theme)
            .show(en_ui);
    });
    ui.data_mut(|d| d.insert_temp(draft_id, draft));
    if tokens != prev && enabled {
        if let Some(b) = node.value_binding {
            data_source.set_str(b, &tokens.join(","));
        }
    }
}

fn render_tab_view_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &mut dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    ctrl_size: MacControlSize,
) {
    let mut tab_idx = node.value_binding.map(|b| data_source.get_i64(b).max(0) as usize).unwrap_or(0);
    let tv_id = ui.scope_id().with(("nib_tv_state", node.id.0));
    let (mut playing, mut progress, mut grid_sel, mut date) = ui
        .data(|d| d.get_temp::<(bool, f32, BTreeSet<usize>, MacDate)>(tv_id))
        .unwrap_or_else(|| (false, 0.35, BTreeSet::new(), MacDate::default()));

    let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_tv_container", node.id.0)).max_rect(rect));
    c_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
    MacTabView::new(&mut tab_idx)
        .style(MacTabStyle::TopTabsBezelBorder)
        .control_size(ctrl_size)
        .theme(theme)
        .tab("AVPlayer HUD", |tab_ui| {
            tab_ui.vertical(|v_ui| {
                MacLabel::new("QuickTime Floating HUD Pod & Inline Transport:").color(MacLabelColor::Secondary).theme(theme).show(v_ui);
                v_ui.add(
                    MacPlayerView::new()
                        .controls_style(MacPlayerControlsStyle::Floating)
                        .is_playing_mut(&mut playing)
                        .progress_mut(&mut progress)
                        .theme(theme),
                );
            });
        })
        .tab("Collection Grid", |tab_ui| {
            let items: [usize; 60] = std::array::from_fn(|i| i);
            MacCollectionView::new(&items, &mut grid_sel)
                .layout(MacCollectionViewFlowLayout::default())
                .theme(theme)
                .show_with_item_tracker(tab_ui, |item_ui, item| {
                    item_ui.centered_and_justified(|c| {
                        MacLabel::new(format!("#{item}")).control_size(MacControlSize::Small).theme(theme).show(c);
                    });
                });
        })
        .tab("Clock & Calendar", |tab_ui| {
            tab_ui.add(
                MacDatePicker::new()
                    .date(&mut date)
                    .style(MacDatePickerStyle::ClockAndCalendar)
                    .elements(MacDatePickerElementFlags::DateAndTime)
                    .theme(theme),
            );
        })
        .show(&mut c_ui);

    if let Some(b) = node.value_binding {
        data_source.set_i64(b, tab_idx as i64);
    }
    ui.data_mut(|d| d.insert_temp(tv_id, (playing, progress, grid_sel, date)));
}
