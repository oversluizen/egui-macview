//! COTS Integration & Behavioral Verification Suite for `egui-macview` v0.6.5 AppKit NIB Parity.

use egui::{
    Align, Color32, Context, Event, FontFamily, FontId, Modifiers, PointerButton, Pos2, RawInput,
    Rect, Vec2,
};
use egui_macview::{
    setup_macview_fonts, setup_symbols_font, MacAppearance, MacBox, MacButtonCell, MacCheckbox,
    MacComboBox, MacControlSize, MacDisclosureGroup, MacFontWeight, MacHelpButton,
    MacInlineActionButton, MacInputCell, MacLabel, MacLabelColor, MacOutlineView, MacPopUpButton,
    MacSegmentedControl, MacTableColumn, MacTableView, MacTheme, INTER_REGULAR_FONT_BYTES,
    INTER_REGULAR_FONT_NAME, INTER_SEMIBOLD_FONT_BYTES, INTER_SEMIBOLD_FONT_NAME,
};
use std::collections::BTreeSet;

fn run_pass(ctx: &Context, input: RawInput, f: impl FnMut(&mut egui::Ui)) -> Vec<egui::epaint::ClippedShape> {
    let mut out = ctx.run_ui(input, f);
    out.textures_delta.clear();
    out.shapes
}

fn click_at(pos: Pos2, pressed: bool) -> RawInput {
    RawInput {
        events: vec![
            Event::PointerMoved(pos),
            Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE },
        ],
        ..Default::default()
    }
}

fn collect_text_shapes<'a>(shape: &'a egui::epaint::Shape, out: &mut Vec<&'a egui::epaint::TextShape>) {
    match shape {
        egui::epaint::Shape::Text(t) => out.push(t),
        egui::epaint::Shape::Vec(v) => {
            for s in v {
                collect_text_shapes(s, out);
            }
        }
        _ => {}
    }
}

#[test]
fn test_v065_optical_tracking_galley_and_controls() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    assert!((theme.tracking_for_font_size(13.0) - (-0.08)).abs() < 1e-4);
    assert!((theme.tracking_for_font_size(11.0) - 0.06).abs() < 1e-4);
    assert!((theme.tracking_for_font_size(12.0) - 0.0).abs() < 1e-4);
    assert!((theme.tracking_for_font_size(9.0) - 0.17).abs() < 1e-4);
    assert!((theme.tracking_for_font_size(15.0) - (-0.23)).abs() < 1e-4);

    let fmt_13 = theme.tracked_text_format(FontId::proportional(13.0), theme.label());
    assert!((fmt_13.extra_letter_spacing - (-0.08)).abs() < 1e-4);
    let fmt_11 = theme.tracked_text_format(FontId::proportional(11.0), theme.label());
    assert!((fmt_11.extra_letter_spacing - 0.06).abs() < 1e-4);

    let (mut checked, mut popup_idx, mut disc_open, mut tree_exp) = (true, 0usize, true, true);
    let mut input_val = "TrackedInput".to_string();
    let mut combo_val = "TrackedCombo".to_string();
    let mut table_sel = BTreeSet::new();

    let shapes = run_pass(&ctx, RawInput::default(), |ui| {
        let g13 = theme.layout_tracked_galley(ui, "Typography Parity Sample", FontId::proportional(13.0), theme.label(), f32::INFINITY);
        let g13_untracked = ui.painter().layout_no_wrap("Typography Parity Sample".to_string(), FontId::proportional(13.0), theme.label());
        assert!(g13.size().x < g13_untracked.size().x);

        let g11 = theme.layout_tracked_galley(ui, "Small Label Sample", FontId::proportional(11.0), theme.label(), f32::INFINITY);
        let g11_untracked = ui.painter().layout_no_wrap("Small Label Sample".to_string(), FontId::proportional(11.0), theme.label());
        assert!(g11.size().x > g11_untracked.size().x);

        ui.add(MacButtonCell::new("TrackedBtn").theme(&theme));
        MacPopUpButton::new(&mut popup_idx, &["TrackedPop"]).theme(&theme).show(ui);
        MacInputCell::new("trk_in", &mut input_val).theme(&theme).show(ui);
        MacComboBox::new("trk_cb", &mut combo_val, &["TrackedCombo"]).theme(&theme).show(ui);
        MacCheckbox::new(&mut checked, "TrackedCheck").theme(&theme).show(ui);
        MacLabel::new("TrackedLbl").theme(&theme).show(ui);
        MacSegmentedControl::new(&["SegOne", "SegTwo"], 0).theme(&theme).show(ui, |_| {});
        MacDisclosureGroup::new("trk_disc", "TrackedSection", &mut disc_open).theme(&theme).show(ui, |u| { u.label("Body"); });
        MacBox::new("TrackedBox").theme(&theme).show(ui, |u| { u.label("Inside"); });
        MacOutlineView::new("trk_out").theme(&theme).show(ui, |tree| {
            tree.row("r1", 0, false, &mut tree_exp, false, |row| { row.label("TrackedRow"); });
        });
        let cols = [MacTableColumn::new("TrackedHdr", 120.0)];
        MacTableView::new("trk_tbl", &cols, 1, &mut table_sel).theme(&theme).show(ui, |u, _, _| { u.label("Cell"); });
    });

    let mut texts = Vec::new();
    for cs in &shapes {
        collect_text_shapes(&cs.shape, &mut texts);
    }

    for expected_label in [
        "TrackedBtn", "TrackedPop", "TrackedInput", "TrackedCombo", "TrackedCheck",
        "TrackedLbl", "SegOne", "TrackedSection", "TrackedBox", "TrackedRow", "TrackedHdr",
    ] {
        let shape = texts.iter().find(|t| t.galley.text() == expected_label).unwrap();
        let font_sz = shape.galley.job.sections[0].format.font_id.size;
        let spacing = shape.galley.job.sections[0].format.extra_letter_spacing;
        let expected_tracking = theme.tracking_for_font_size(font_sz);
        assert!((spacing - expected_tracking).abs() < 1e-4, "Tracking mismatch on {expected_label}");
        if (font_sz - 12.0).abs() > 0.1 {
            assert!(spacing.abs() > 1e-4, "Missing tracking on {expected_label}");
        }
    }
}

#[test]
fn test_v065_input_cell_control_sizes_fixed_rect_and_alignment() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let tf = theme.text_field();

    assert_eq!(tf.regular.height.points(), 22.0);
    assert_eq!(tf.regular.font_size.points(), 13.0);
    assert_eq!(tf.regular.padding_horizontal.points(), 7.0);
    assert_eq!(tf.regular.padding_vertical.as_ref().unwrap().points(), 2.0);
    assert_eq!(tf.regular.first_baseline_offset_from_top.points(), 15.0);
    assert_eq!(tf.regular.baseline_offset_from_bottom.points(), 6.0);
    assert_eq!(tf.small.height.points(), 19.0);
    assert_eq!(tf.small.font_size.points(), 11.0);
    assert_eq!(tf.small.padding_horizontal.points(), 5.0);
    assert_eq!(tf.mini.font_size.points(), 9.0);
    assert_eq!(tf.mini.padding_horizontal.points(), 4.0);
    assert_eq!(tf.compact_infobox.height.points(), 18.0);

    let mut v_default = "Default22".to_string();
    let mut v_reg = "AutoReg".to_string();
    let mut v_sm = "AutoSm".to_string();
    let mut v_hud = "AutoHud".to_string();
    let mut v_mini = "AutoMini".to_string();
    let mut v_left = "42.0".to_string();
    let mut v_right = "42.0".to_string();
    let mut default_h = 0.0;

    let shapes = run_pass(&ctx, RawInput::default(), |ui| {
        default_h = MacInputCell::new("def_cell", &mut v_default).theme(&theme).show(ui).rect.height();
        MacInputCell::new("r_reg", &mut v_reg).theme(&theme).fixed_rect(Rect::from_min_size(Pos2::new(10.0, 40.0), Vec2::new(100.0, 22.0))).show(ui);
        MacInputCell::new("r_sm", &mut v_sm).theme(&theme).fixed_rect(Rect::from_min_size(Pos2::new(10.0, 70.0), Vec2::new(100.0, 19.0))).show(ui);
        MacInputCell::new("r_hud", &mut v_hud).theme(&theme).fixed_rect(Rect::from_min_size(Pos2::new(10.0, 100.0), Vec2::new(100.0, 18.0))).show(ui);
        MacInputCell::new("r_mini", &mut v_mini).theme(&theme).fixed_rect(Rect::from_min_size(Pos2::new(10.0, 130.0), Vec2::new(100.0, 16.0))).show(ui);
        MacInputCell::new("align_l", &mut v_left).theme(&theme).fixed_rect(Rect::from_min_size(Pos2::new(140.0, 40.0), Vec2::new(100.0, 22.0))).text_align(Align::LEFT).show(ui);
        MacInputCell::new("align_r", &mut v_right).theme(&theme).fixed_rect(Rect::from_min_size(Pos2::new(140.0, 70.0), Vec2::new(100.0, 22.0))).text_align(Align::RIGHT).show(ui);
    });

    assert_eq!(default_h, 22.0);
    let mut texts = Vec::new();
    for cs in &shapes {
        collect_text_shapes(&cs.shape, &mut texts);
    }
    let font_size_of = |label: &str| -> f32 {
        texts.iter().find(|t| t.galley.text() == label).unwrap().galley.job.sections[0].format.font_id.size
    };
    assert_eq!(font_size_of("AutoReg"), 13.0);
    assert_eq!(font_size_of("AutoSm"), 11.0);
    assert_eq!(font_size_of("AutoHud"), 11.0);
    assert_eq!(font_size_of("AutoMini"), 9.0);
    let matching_42: Vec<_> = texts.iter().filter(|t| t.galley.text() == "42.0").collect();
    assert!(matching_42[1].pos.x > matching_42[0].pos.x + 20.0);
}

#[test]
fn test_v065_popup_button_double_chevron_and_nib_frame_visual_clamping() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let pb = theme.popup_button();

    assert_eq!(pb.chevron_gap_vertical.points(), 1.75);
    assert_eq!(pb.chevron_half_width.points(), 2.5);
    assert_eq!(pb.chevron_height.points(), 2.25);
    assert_eq!(pb.chevron_stroke_width.points(), 1.25);
    assert_eq!(pb.regular_visual_height.points(), 20.0);

    let mut sel = 0usize;
    let nib_frame = Rect::from_min_size(Pos2::new(20.0, 20.0), Vec2::new(140.0, 25.0));
    let shapes = run_pass(&ctx, RawInput::default(), |ui| {
        MacPopUpButton::new(&mut sel, &["Item A", "Item B"]).theme(&theme).control_size(MacControlSize::Regular).fixed_rect(nib_frame).show(ui);
    });

    let bezel_rect = shapes.iter().find_map(|cs| match &cs.shape {
        egui::Shape::Rect(r) if r.fill == theme.control_background() => Some(r.rect),
        _ => None,
    }).unwrap();
    assert!((bezel_rect.height() - 20.0).abs() < 0.5);

    let cy = bezel_rect.center().y;
    let lines: Vec<_> = shapes.iter().filter_map(|cs| match &cs.shape {
        egui::Shape::LineSegment { points, stroke } => Some((*points, *stroke)),
        _ => None,
    }).collect();
    assert!(lines.len() >= 4);
    for (_, stroke) in &lines {
        assert_eq!(stroke.color, theme.label());
    }
    let upper: Vec<_> = lines.iter().filter(|(pts, _)| pts[0].y < cy && pts[1].y < cy).collect();
    let lower: Vec<_> = lines.iter().filter(|(pts, _)| pts[0].y > cy && pts[1].y > cy).collect();
    assert_eq!(upper.len(), 2);
    assert_eq!(lower.len(), 2);
}

#[test]
fn test_v065_mac_checkbox_box_surfaces_combo_badge_and_disclosure() {
    let ctx = Context::default();
    let aqua = MacTheme::new(MacAppearance::Aqua);
    let dark = MacTheme::new(MacAppearance::DarkAqua);

    assert_eq!(aqua.box_background(), Color32::from_rgb(0xE5, 0xE5, 0xE5));
    assert_eq!(aqua.box_border(), Color32::from_rgb(0xD8, 0xD8, 0xD8));
    assert_eq!(dark.box_background(), Color32::from_rgb(0x23, 0x25, 0x28));
    assert_eq!(dark.box_border(), Color32::from_rgb(0x3A, 0x3D, 0x42));
    assert_eq!(aqua.checkbox().regular.box_size.points(), 14.0);
    assert_eq!(aqua.checkbox().small.box_size.points(), 12.0);
    assert_eq!(aqua.checkbox().mini.box_size.points(), 10.0);

    let (mut c_off, mut c_on, mut c_mix, mut is_open) = (false, true, false, true);
    let mut cb_text = "Sonoma".to_string();
    let mut off_rect = Rect::NOTHING;
    let mut disc_rect = Rect::NOTHING;

    let shapes = run_pass(&ctx, RawInput::default(), |ui| {
        off_rect = MacCheckbox::new(&mut c_off, "Unchecked").theme(&aqua).show(ui).rect;
        MacCheckbox::new(&mut c_on, "Checked").theme(&aqua).control_size(MacControlSize::Small).show(ui);
        MacCheckbox::new(&mut c_mix, "Mixed").theme(&aqua).control_size(MacControlSize::Mini).mixed(true).show(ui);
        MacBox::new("Metrics").theme(&aqua).fixed_rect(Rect::from_min_size(Pos2::new(20.0, 90.0), Vec2::new(200.0, 50.0))).show(ui, |b_ui| { b_ui.label("Inside"); });
        MacComboBox::new("cb_nib", &mut cb_text, &["Sonoma", "Sequoia"]).theme(&aqua).fixed_rect(Rect::from_min_size(Pos2::new(20.0, 150.0), Vec2::new(160.0, 23.0))).show(ui);
    });

    assert!(shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Rect(r) if r.fill == aqua.box_background())));
    assert!(shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Rect(r) if r.fill == aqua.control_accent() && (r.rect.width() - 17.0).abs() < 1.5)));

    for pressed in [true, false] {
        run_pass(&ctx, click_at(off_rect.center(), pressed), |ui| {
            MacCheckbox::new(&mut c_off, "Unchecked").theme(&aqua).show(ui);
        });
    }
    assert!(c_off);

    run_pass(&ctx, RawInput::default(), |ui| {
        disc_rect = MacDisclosureGroup::new("axes_sec", "Axes", &mut is_open).theme(&aqua).with_trailing_add_button(true).show(ui, |u| { u.label("Weight Axis"); }).response.rect;
    });
    let plus_pos = Pos2::new(disc_rect.right() - 8.0, disc_rect.center().y);
    let mut trailing_clicked = false;
    for pressed in [true, false] {
        run_pass(&ctx, click_at(plus_pos, pressed), |ui| {
            let r = MacDisclosureGroup::new("axes_sec", "Axes", &mut is_open).theme(&aqua).with_trailing_add_button(true).show(ui, |u| { u.label("Weight Axis"); });
            if !pressed {
                trailing_clicked = r.trailing_clicked;
            }
        });
    }
    assert!(trailing_clicked);
    assert!(is_open);
}

#[test]
fn test_v065_inter_and_cupertino_fonts_and_cross_platform_baseline_alignment() {
    assert!(INTER_REGULAR_FONT_BYTES.len() > 200_000);
    assert!(INTER_SEMIBOLD_FONT_BYTES.len() > 200_000);

    let mut fonts = egui::FontDefinitions::default();
    setup_macview_fonts(&mut fonts);
    setup_symbols_font(&mut fonts);
    assert!(fonts.font_data.contains_key(INTER_REGULAR_FONT_NAME));
    assert!(fonts.font_data.contains_key(INTER_SEMIBOLD_FONT_NAME));
    assert!(fonts.font_data.contains_key("cupertino_icons"));
    assert_eq!(fonts.families[&FontFamily::Proportional][0], INTER_REGULAR_FONT_NAME);

    let ctx = Context::default();
    ctx.set_fonts(fonts);
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut val = "1920".to_string();

    let shapes = run_pass(&ctx, RawInput::default(), |ui| {
        MacLabel::new("Width:").theme(&theme).weight(MacFontWeight::Semibold).color(MacLabelColor::Primary).alignment(Align::RIGHT)
            .fixed_rect(Rect::from_min_size(Pos2::new(20.0, 43.0), Vec2::new(60.0, 16.0))).show(ui);
        MacInputCell::new("w_in", &mut val).theme(&theme)
            .fixed_rect(Rect::from_min_size(Pos2::new(88.0, 41.0), Vec2::new(80.0, 22.0))).show(ui);
        MacHelpButton::new().theme(&theme)
            .fixed_rect(Rect::from_min_size(Pos2::new(180.0, 40.0), Vec2::new(25.0, 25.0))).show(ui);
        MacInlineActionButton::new("+").theme(&theme)
            .fixed_rect(Rect::from_min_size(Pos2::new(212.0, 44.0), Vec2::new(16.0, 16.0))).show(ui);
    });

    let mut texts = Vec::new();
    for cs in &shapes {
        collect_text_shapes(&cs.shape, &mut texts);
    }
    let lbl_shape = texts.iter().find(|t| t.galley.text() == "Width:").unwrap();
    let in_shape = texts.iter().find(|t| t.galley.text() == "1920").unwrap();
    assert_eq!(lbl_shape.galley.job.sections[0].format.font_id.family, FontFamily::Name(INTER_SEMIBOLD_FONT_NAME.into()));
    assert!((lbl_shape.pos.y - in_shape.pos.y).abs() <= 0.5);

    let circles: Vec<_> = shapes.iter().filter_map(|cs| match &cs.shape {
        egui::Shape::Circle(c) => Some(c.radius),
        _ => None,
    }).collect();
    assert!(circles.iter().any(|&r| (r - 10.0).abs() < 0.5));
    assert!(circles.iter().any(|&r| (r - 8.0).abs() < 0.5));
}
