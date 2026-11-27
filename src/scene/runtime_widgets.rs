//! Extended AppKit/AVKit Widget Renderers for [`MacSceneIR::show`] (`SPEC-013` §4, `REQ-013-02P`).

use crate::controls::{
    MacCheckbox, MacCircularSlider, MacColorWell, MacComboBox, MacControlSize, MacDate, MacDatePicker, MacDatePickerElementFlags,
    MacDatePickerStyle, MacDisclosureButton, MacDisclosureGroup, MacImageView, MacLabel, MacLabelColor, MacLevelIndicator,
    MacLevelIndicatorStyle, MacOriginAnchor, MacOriginMatrix, MacProgressIndicator, MacRatingIndicator, MacRuleEditor, MacRuleRow,
    MacSlider, MacStepper, MacTableActionBar, MacTagColorPicker,
};
use crate::scene::ir::{SceneNode, SceneNodeKind};
use crate::scene::runtime::MacSceneDataSource;
use crate::scene::runtime_tabs::{
    render_help_button_node, render_inline_action_button_node, render_search_field_node, render_tab_view_node, render_table_view_node,
};
use crate::theme::MacTheme;
use egui::{Rect, Ui, UiBuilder};

pub(crate) fn render_label_node(
    ui: &mut Ui,
    theme: &MacTheme,
    data_source: &dyn MacSceneDataSource,
    node: &SceneNode,
    rect: Rect,
    ctrl_size: MacControlSize,
) {
    let formatted_pattern = node.display_pattern.as_ref().and_then(|pat| {
        if node.display_pattern_bindings.is_empty() {
            return None;
        }
        let mut s = pat.clone();
        let mut any_non_empty = false;
        for &(slot, bid, _) in &node.display_pattern_bindings {
            let val = data_source.get_str(bid);
            if !val.is_empty() {
                any_non_empty = true;
            }
            let token = format!("%{{value{slot}}}@");
            s = s.replace(&token, val);
        }
        any_non_empty.then_some(s)
    });
    let dyn_txt = formatted_pattern
        .as_deref()
        .unwrap_or_else(|| node.value_binding.map(|b| data_source.get_str(b)).unwrap_or(""));
    let text = if dyn_txt.is_empty() { node.title.as_str() } else { dyn_txt };
    let align = match node.text_alignment {
        crate::scene::ir::MacTextAlignment::Right => egui::Align::Max,
        crate::scene::ir::MacTextAlignment::Center => egui::Align::Center,
        _ => egui::Align::Min,
    };
    let lm = theme.label_metrics();
    let mini_max_pt = lm.mini_font_size.points() + 0.5;
    let small_max_pt = lm.small_font_size.points() + 0.5;
    let effective_size = match node.font_size_pt {
        Some(pt) if pt <= mini_max_pt => MacControlSize::Mini,
        Some(pt) if pt <= small_max_pt => MacControlSize::Small,
        _ => ctrl_size,
    };
    let label_rect = if matches!(align, egui::Align::Center) && node.value_binding.is_some() {
        let expand_w = rect.width().max(theme.popover_metrics().full_popover_width.points());
        egui::Rect::from_center_size(rect.center(), egui::vec2(expand_w, rect.height()))
    } else if matches!(align, egui::Align::Max | egui::Align::Center) {
        rect
    } else {
        let extra_w = theme.table_view().header_height.points();
        egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() + extra_w, rect.height()))
    };
    MacLabel::new(text)
        .weight(node.font_weight)
        .color(node.label_color)
        .alignment(align)
        .control_size(effective_size)
        .theme(theme)
        .fixed_rect(label_rect)
        .show(ui);
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
        let extra_w = theme.table_view().header_height.points();
        let chk_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() + extra_w, rect.height()));
        MacCheckbox::new(&mut val, node.title.as_str())
            .theme(theme)
            .control_size(ctrl_size)
            .mixed(is_mixed)
            .enabled(enabled)
            .fixed_rect(chk_rect)
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
            let min_diam = theme.table_view().header_height.points();
            let diam = rect.width().min(rect.height()).max(min_diam);
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
            let is_disc = data_source.resolve_level_discrete(node.id, &node.ib_object_id, tick_marks > 0);
            let has_thr = data_source.resolve_level_thresholds(node.id, &node.ib_object_id, true);
            let style = if is_disc { MacLevelIndicatorStyle::Discrete } else { MacLevelIndicatorStyle::Continuous };
            let span = (max - min).max(1.0) as f32;
            let step_frac = theme.slider().key_step_fraction.points();
            let warn_ratio = 0.5 + step_frac;
            let crit_ratio = 1.0 - step_frac;
            let peak_ratio = step_frac * 0.5;
            let def_segs = theme.level_indicator().default_segment_count.points() as usize;
            let (warn_v, crit_v) = if has_thr {
                (min as f32 + span * warn_ratio, min as f32 + span * crit_ratio)
            } else {
                (max as f32 + span, max as f32 + span * 2.0)
            };
            let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_lvl_ui", node.id.0)).max_rect(rect));
            MacLevelIndicator::new(val, (min as f32)..=(max as f32))
                .style(style)
                .segments(tick_marks.max(def_segs))
                .warning_value(warn_v)
                .critical_value(crit_v)
                .peak_hold((val + span * peak_ratio).min(max as f32))
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
            let pct_scale = theme.progress_indicator().percentage_scale.points();
            let frac = if max > 1.0 {
                (val / max as f32).clamp(0.0, 1.0)
            } else if val > 1.0 {
                (val / pct_scale).clamp(0.0, 1.0)
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
        SceneNodeKind::SearchField => {
            render_search_field_node(ui, theme, data_source, node, rect, enabled);
        }
        SceneNodeKind::ComboBox { options } => {
            let fb_id = ui.scope_id().with(("nib_cb_str", node.id.0));
            let mut text = node
                .value_binding
                .map(|b| data_source.get_str(b).to_string())
                .unwrap_or_else(|| ui.data(|d| d.get_temp::<String>(fb_id)).unwrap_or_else(|| node.title.clone()));
            let refs: Vec<&str> = options.iter().map(String::as_str).collect();
            let on_sidebar = theme.is_sidebar_surface(theme.active_surface_at(ui, rect.center()));
            let is_pill = on_sidebar || node.bezel_style == 1 || (!node.draws_background && node.is_bezeled);
            let changed = ui
                .add_enabled_ui(enabled, |en_ui| {
                    MacComboBox::new((en_ui.scope_id(), "nib_cb", node.id.0), &mut text, &refs)
                        .theme(theme)
                        .control_size(ctrl_size)
                        .recessed_pill(is_pill)
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
            if MacColorWell::new(&mut col)
                .id_salt((ui.scope_id(), "nib_cw", node.id.0))
                .fixed_rect(rect)
                .show_alpha(true)
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
        SceneNodeKind::TableView { columns } => {
            render_table_view_node(ui, theme, data_source, node, rect, columns);
        }
        SceneNodeKind::RuleEditor => {
            const DEFAULT_CRITERIA: &[&[&str]] = &[&["Key", "Value"], &["==", "!="], &["True", "False"]];
            let host_criteria = data_source.get_rule_criteria(node.id, &node.ib_object_id);
            let criteria = if host_criteria.is_empty() { DEFAULT_CRITERIA } else { host_criteria };
            let col_len = criteria.len().max(1);
            let re_id = ui.scope_id().with(("nib_re_rows", node.id.0));
            let r0: Vec<usize> = (0..col_len).map(|_| 0).collect();
            let r1: Vec<usize> = (0..col_len).map(|_| 1).collect();
            let mut rules =
                ui.data(|d| d.get_temp::<Vec<MacRuleRow>>(re_id)).unwrap_or_else(|| vec![MacRuleRow::new(r0), MacRuleRow::new(r1)]);
            let prev_len = rules.len();
            let mut c_ui = ui.new_child(UiBuilder::new().id_salt(("nib_re_ui", node.id.0)).max_rect(rect));
            c_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
            MacRuleEditor::new((c_ui.scope_id(), "nib_re", node.id.0), &mut rules, criteria).theme(theme).show(&mut c_ui);
            if rules.len() != prev_len {
                if let Some(b) = node.value_binding {
                    data_source.set_i64(b, rules.len() as i64);
                }
            }
            ui.data_mut(|d| d.insert_temp(re_id, rules));
        }
        SceneNodeKind::TableActionBar => {
            let items: Vec<String> = node.value_binding.map(|b| data_source.get_options(b).to_vec()).unwrap_or_default();
            let menu_id = ui.scope_id().with(("nib_tab_bar_menu", node.id.0));
            let (mut menu_open, mut menu_pos, mut menu_sel) =
                ui.data(|d| d.get_temp::<(bool, egui::Pos2, Option<usize>)>(menu_id)).unwrap_or((false, rect.left_bottom(), None));
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
                .with_action_menu(true)
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
            if resp.action_clicked {
                menu_open = true;
                menu_pos = rect.left_bottom();
            }
            let menu_items: Vec<crate::navigation::NSMenuItem> =
                items.iter().map(|lbl| crate::navigation::NSMenuItem::action(lbl.as_str(), lbl.as_str())).collect();
            if !menu_items.is_empty() {
                if let Some(_chosen) =
                    crate::feedback::MacContextMenu::new(&menu_items, &mut menu_open, &mut menu_pos, &mut menu_sel).theme(theme).show(ui)
                {
                    if let Some(b) = node.value_binding {
                        data_source.set_i64(b, 1);
                    }
                    if let Some(act) = node.action {
                        data_source.on_action(act, node.id);
                    }
                }
            }
            ui.data_mut(|d| d.insert_temp(menu_id, (menu_open, menu_pos, menu_sel)));
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
            render_help_button_node(ui, theme, data_source, node, rect, ctrl_size, enabled);
        }
        SceneNodeKind::InlineActionButton => {
            render_inline_action_button_node(ui, theme, data_source, node, rect, enabled);
        }
        SceneNodeKind::ImageView { icon_name, is_template } => {
            let tint = if *is_template { theme.secondary_label() } else { theme.label() };
            if !data_source.paint_scene_icon_scaled(ui, theme, icon_name, rect, tint, (node.image_scaling, node.symbol_point_size)) {
                MacImageView::from_resource_name(icon_name).theme(theme).template(*is_template).scaling(crate::controls::MacImageScaling::from_nib_code(node.image_scaling)).symbol_point_size(node.symbol_point_size).fixed_rect(rect).show(ui);
            }
        }
        SceneNodeKind::Stepper { step } => {
            let fb_id = ui.scope_id().with(("nib_stp", node.id.0));
            let mut val =
                node.value_binding.map(|b| data_source.get_f64(b)).unwrap_or_else(|| ui.data(|d| d.get_temp::<f64>(fb_id)).unwrap_or(0.0));
            let mut stp_ui = ui.new_child(UiBuilder::new().id_salt(("nib_stp_ui", node.id.0)).max_rect(rect));
            let bound = f64::from(i16::MAX);
            if MacStepper::new(&mut val, -bound..=bound, *step)
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
            let is_two_row = rect.height() >= theme.table_view().header_height.points()
                && rect.width() < theme.popover_metrics().compact_panel_width.points();
            let mut picker = MacTagColorPicker::new(&mut tag).theme(theme).enabled(enabled).fixed_rect(rect);
            if is_two_row {
                picker = picker.columns(6).rows(2).include_clear_swatch(false);
            }
            if picker.show(ui).changed() {
                if let Some(b) = node.value_binding {
                    data_source.set_i64(b, tag.map(|i| i as i64).unwrap_or(-1));
                } else {
                    ui.data_mut(|d| d.insert_temp(fb_id, tag));
                }
                if let Some(act) = node.action {
                    data_source.on_action(act, node.id);
                }
            }
        }
        _ => {}
    }
}
