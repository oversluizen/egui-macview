//! COTS Integration & Behavioral Verification Suite for `egui-macview` v0.6.9 Pro-Inspector Controls.
//!
//! Covers:
//! 1. `MacOriginMatrix` (`3x3` 9-point transform origin / anchor pin grid, click & arrow keys).
//! 2. `MacTagColorPicker` (macOS Finder / AppKit color label swatch dots, 1x12, 2x6, custom palette, clear swatch).
//! 3. `MacTableActionBar` (`NSBezelStyleSmallSquare` `[ + | - | action ]` action strip).

use egui::{Color32, Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use egui_macview::{
    MacAppearance, MacControlSize, MacOriginAnchor, MacOriginMatrix, MacTableActionBar,
    MacTagColorPicker, MacTheme,
};

fn run_pass(ctx: &Context, input: RawInput, f: impl FnMut(&mut egui::Ui)) -> Vec<egui::epaint::ClippedShape> {
    let mut out = ctx.run_ui(input, f);
    out.textures_delta.clear();
    out.shapes
}

fn click_at(pos: Pos2, pressed: bool) -> RawInput {
    RawInput {
        events: vec![Event::PointerMoved(pos), Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE }],
        ..Default::default()
    }
}

fn key_press(key: Key) -> RawInput {
    RawInput { events: vec![Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE }], ..Default::default() }
}

#[test]
fn test_v069_origin_matrix_anchors_click_and_keyboard_navigation() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let om = theme.origin_matrix();
    assert_eq!(
        (om.grid_size.points(), om.small_grid_size.points(), om.mini_grid_size.points(), om.cell_step.points(), om.small_cell_step.points(), om.mini_cell_step.points(), om.node_radius.points(), om.hover_node_radius.points(), om.active_node_radius.points()),
        (34.0, 28.0, 22.0, 12.0, 10.0, 8.0, 2.5, 3.0, 3.5)
    );
    for (idx, anchor) in MacOriginAnchor::ALL.iter().copied().enumerate() {
        assert_eq!((anchor.index(), MacOriginAnchor::from_index(idx), MacOriginAnchor::from_row_col(anchor.row(), anchor.col())), (idx, anchor, anchor));
    }
    assert_eq!(MacOriginAnchor::from_index(99), MacOriginAnchor::Center);

    let mut anchor = MacOriginAnchor::Center;
    let fixed = Rect::from_min_size(Pos2::new(40.0, 40.0), Vec2::splat(34.0));
    let tl_pos = MacOriginMatrix::node_center(fixed, MacOriginAnchor::TopLeft, om.cell_step.points());
    let shapes = run_pass(&ctx, RawInput { events: vec![Event::PointerMoved(tl_pos)], ..Default::default() }, |ui| {
        MacOriginMatrix::new(&mut anchor).theme(&theme).fixed_rect(fixed).show(ui);
    });
    let surface = theme.window_background();
    assert_eq!(shapes.iter().filter(|cs| matches!(&cs.shape, egui::Shape::LineSegment { .. })).count(), 4);
    assert_eq!(shapes.iter().filter(|cs| matches!(&cs.shape, egui::Shape::Circle(c) if c.stroke.width == 0.0 && c.fill != theme.control_accent())).count(), 8);
    assert!(shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Circle(c) if (c.radius - om.hover_node_radius.points()).abs() < 1e-3 && c.fill == theme.label_on(surface))));
    assert!(shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Circle(c) if (c.radius - om.active_node_radius.points()).abs() < 1e-3 && c.fill == theme.control_accent() && c.stroke.width == 0.0)));

    for target in MacOriginAnchor::ALL {
        let pos = MacOriginMatrix::node_center(fixed, target, om.cell_step.points());
        for pressed in [true, false] {
            run_pass(&ctx, click_at(pos, pressed), |ui| { MacOriginMatrix::new(&mut anchor).theme(&theme).fixed_rect(fixed).show(ui); });
        }
        assert_eq!(anchor, target);
    }

    for (key, expected) in [
        (Key::ArrowLeft, MacOriginAnchor::BottomCenter), (Key::ArrowLeft, MacOriginAnchor::BottomLeft), (Key::ArrowLeft, MacOriginAnchor::BottomLeft),
        (Key::ArrowUp, MacOriginAnchor::MiddleLeft), (Key::ArrowUp, MacOriginAnchor::TopLeft), (Key::ArrowUp, MacOriginAnchor::TopLeft),
        (Key::ArrowRight, MacOriginAnchor::TopCenter), (Key::ArrowDown, MacOriginAnchor::Center),
    ] {
        run_pass(&ctx, key_press(key), |ui| { MacOriginMatrix::new(&mut anchor).theme(&theme).fixed_rect(fixed).show(ui); });
        assert_eq!(anchor, expected);
    }

    // Test `from_index` normalization, distant/same-pin click guard, disabled guard, size variants, and fractional DPI (1.25, 1.5)
    let mut idx_binding = 99usize;
    let _ = run_pass(&ctx, RawInput::default(), |ui| { MacOriginMatrix::from_index(&mut idx_binding).theme(&theme).fixed_rect(fixed).show(ui); });
    assert_eq!(idx_binding, MacOriginAnchor::Center.index());
    let mut changed_clicks = 0usize;
    let wide_fixed = Rect::from_min_size(Pos2::new(40.0, 40.0), Vec2::new(120.0, 34.0));
    let far_pos = Pos2::new(wide_fixed.max.x - 1.0, wide_fixed.center().y);
    for (pos, en) in [(tl_pos, true), (tl_pos, true), (far_pos, true), (MacOriginMatrix::node_center(fixed, MacOriginAnchor::BottomRight, om.cell_step.points()), false)] {
        for pressed in [true, false] {
            run_pass(&ctx, click_at(pos, pressed), |ui| {
                let r = if pos == far_pos { wide_fixed } else { fixed };
                if MacOriginMatrix::from_index(&mut idx_binding).theme(&theme).enabled(en).fixed_rect(r).show(ui).changed() { changed_clicks += 1; }
            });
        }
    }
    assert_eq!((idx_binding, changed_clicks), (0, 1));
    for ppp in [1.25_f32, 1.5_f32] {
        ctx.set_pixels_per_point(ppp);
        let dpi_shapes = run_pass(&ctx, RawInput::default(), |ui| { MacOriginMatrix::new(&mut anchor).theme(&theme).fixed_rect(fixed).show(ui); });
        let centers: Vec<Pos2> = dpi_shapes.iter().filter_map(|cs| match &cs.shape { egui::Shape::Circle(c) => Some(c.center), _ => None }).collect();
        assert_eq!(centers.len(), 9);
        for cs in &dpi_shapes {
            if let egui::Shape::LineSegment { points, .. } = &cs.shape {
                assert!(centers.iter().any(|c| c.distance(points[0]) < 1e-4) && centers.iter().any(|c| c.distance(points[1]) < 1e-4));
            }
        }
    }
    ctx.set_pixels_per_point(1.0);
    let _ = run_pass(&ctx, RawInput::default(), |ui| {
        assert_eq!(MacOriginMatrix::new(&mut anchor).theme(&theme).show(ui).rect.size(), Vec2::splat(34.0));
        assert_eq!(MacOriginMatrix::new(&mut anchor).theme(&theme).control_size(MacControlSize::CompactInfobox).show(ui).rect.size(), Vec2::splat(28.0));
        assert_eq!(MacOriginMatrix::new(&mut anchor).theme(&theme).control_size(MacControlSize::Mini).show(ui).rect.size(), Vec2::splat(22.0));
    });
}

#[test]
fn test_v069_tag_color_picker_swatches_layouts_and_custom_palette() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let tm = theme.tag_color_picker();
    assert_eq!(
        (tm.dot_diameter.points(), tm.small_dot_diameter.points(), tm.mini_dot_diameter.points(), tm.dot_spacing.points()),
        (12.0, 10.0, 8.0, 5.0)
    );
    assert_eq!(theme.tag_label_palette().len(), 11);

    let mut selected: Option<usize> = None;
    let fixed_1x12 = Rect::from_min_size(Pos2::new(20.0, 20.0), Vec2::new(220.0, 16.0));
    let (diam, spacing) = (tm.dot_diameter.points(), tm.dot_spacing.points());

    let p0 = MacTagColorPicker::swatch_center(fixed_1x12, 0, 12, diam, spacing);
    let clear_pos = MacTagColorPicker::swatch_center(fixed_1x12, 11, 12, diam, spacing);
    let init_shapes = run_pass(&ctx, RawInput { events: vec![Event::PointerMoved(p0)], ..Default::default() }, |ui| {
        MacTagColorPicker::new(&mut selected).theme(&theme).columns(12).fixed_rect(fixed_1x12).show(ui);
    });
    assert!(!init_shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Circle(c) if c.center.distance(clear_pos) < 1.0 && c.radius > diam * 0.5 + 0.2)));
    assert!(init_shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Circle(c) if c.center.distance(clear_pos) < 1.0 && c.stroke.color == theme.label_on(theme.window_background()))));
    assert!(init_shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Circle(c) if c.center.distance(p0) < 1.0 && c.radius > diam * 0.5 + 0.5)));

    // Click swatch 0 (Red) and swatch 7 (Blue) in 1x12 layout
    let mut sel_shapes = Vec::new();
    for idx in [0usize, 7usize] {
        let p = MacTagColorPicker::swatch_center(fixed_1x12, idx, 12, diam, spacing);
        for pressed in [true, false] {
            sel_shapes = run_pass(&ctx, click_at(p, pressed), |ui| {
                MacTagColorPicker::new(&mut selected).theme(&theme).columns(12).fixed_rect(fixed_1x12).show(ui);
            });
        }
        assert_eq!(selected, Some(idx));
    }
    let p7 = MacTagColorPicker::swatch_center(fixed_1x12, 7, 12, diam, spacing);
    let check_segments = sel_shapes.iter().filter(|cs| matches!(&cs.shape, egui::Shape::LineSegment { points, stroke } if points[0].distance(p7) <= diam * 0.5 && stroke.color == theme.on_accent_text())).count();
    assert_eq!(check_segments, 2, "Selected color tag swatch must draw a 2-segment high-contrast vector checkmark inside the swatch");
    assert!(sel_shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Circle(c) if c.center.distance(p7) < 1.0 && c.fill != Color32::TRANSPARENT && (c.radius - diam * 0.5).abs() < 1e-3)), "Hovered selected swatch must not expand into outer selection ring");
    for pressed in [true, false] {
        run_pass(&ctx, click_at(clear_pos, pressed), |ui| {
            MacTagColorPicker::new(&mut selected).theme(&theme).columns(12).fixed_rect(fixed_1x12).show(ui);
        });
    }
    assert_eq!(selected, None);

    // 2x6 layout + custom palette (including pure white for high-contrast dark checkmark) + disabled state
    let custom = [
        Color32::from_rgb(255, 255, 255),
        Color32::from_rgb(0, 255, 0),
        Color32::from_rgb(0, 0, 255),
    ];
    let fixed_2x6 = Rect::from_min_size(Pos2::new(20.0, 60.0), Vec2::new(120.0, 34.0));
    let _ = run_pass(&ctx, RawInput::default(), |ui| {
        MacTagColorPicker::new(&mut selected).theme(&theme).columns(6).fixed_rect(fixed_2x6).show(ui);
    });
    let p_row2 = MacTagColorPicker::swatch_center(fixed_2x6, 8, 6, diam, spacing);
    for pressed in [true, false] {
        run_pass(&ctx, click_at(p_row2, pressed), |ui| {
            MacTagColorPicker::new(&mut selected).theme(&theme).columns(6).fixed_rect(fixed_2x6).show(ui);
        });
    }
    assert_eq!(selected, Some(8));

    // Custom 3-color palette: click slot 0 (pure white) and verify dark checkmark contrast, then slot 1
    let p_custom_0 = MacTagColorPicker::swatch_center(fixed_2x6, 0, 6, diam, spacing);
    let mut white_shapes = Vec::new();
    for pressed in [true, false] {
        white_shapes = run_pass(&ctx, click_at(p_custom_0, pressed), |ui| {
            MacTagColorPicker::new(&mut selected).theme(&theme).palette(&custom).columns(6).fixed_rect(fixed_2x6).show(ui);
        });
    }
    let dark_chk = egui_macview::theme::MacBackgroundStyle::Light.primary_text_color(&theme);
    assert_eq!(white_shapes.iter().filter(|cs| matches!(&cs.shape, egui::Shape::LineSegment { points, stroke } if points[0].distance(p_custom_0) <= diam * 0.5 && stroke.color == dark_chk)).count(), 2);
    let p_custom_1 = MacTagColorPicker::swatch_center(fixed_2x6, 1, 6, diam, spacing);
    for pressed in [true, false] {
        run_pass(&ctx, click_at(p_custom_1, pressed), |ui| {
            MacTagColorPicker::new(&mut selected).theme(&theme).palette(&custom).columns(6).fixed_rect(fixed_2x6).show(ui);
        });
    }
    assert_eq!(selected, Some(1));

    // Disabled ignores clicks, distant empty-space clicks in wide fixed_rect are ignored, and 0-slot picker allocates 0 height
    let p_first = MacTagColorPicker::swatch_center(fixed_2x6, 0, 6, diam, spacing);
    let p_far = Pos2::new(fixed_2x6.max.x - 1.0, fixed_2x6.max.y - 1.0);
    for pressed in [true, false] {
        run_pass(&ctx, click_at(p_first, pressed), |ui| {
            MacTagColorPicker::new(&mut selected).theme(&theme).palette(&custom).enabled(false).fixed_rect(fixed_2x6).show(ui);
        });
        run_pass(&ctx, click_at(p_far, pressed), |ui| {
            MacTagColorPicker::new(&mut selected).theme(&theme).palette(&custom).columns(6).fixed_rect(fixed_2x6).show(ui);
        });
    }
    assert_eq!(selected, Some(1));
    let mut empty_h = 1.0_f32;
    let _ = run_pass(&ctx, RawInput::default(), |ui| {
        empty_h = MacTagColorPicker::new(&mut selected).theme(&theme).palette(&[]).include_clear_swatch(false).show(ui).rect.height();
    });
    assert_eq!(empty_h, 0.0);
}

#[test]
fn test_v069_table_action_bar_add_remove_action_and_strip_modes() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let ab = theme.table_action_bar();
    assert_eq!(
        (ab.bar_height.points(), ab.small_bar_height.points(), ab.mini_bar_height.points(), ab.button_width.points(), ab.corner_radius.points()),
        (22.0, 18.0, 15.0, 24.0, 1.0)
    );

    let bar_rect = Rect::from_min_size(Pos2::new(20.0, 20.0), Vec2::new(240.0, 22.0));
    let btn_w = ab.button_width.points();
    let act_w = ab.action_button_width.points();
    let add_pos = MacTableActionBar::button_center(bar_rect, 0, btn_w);
    let rem_pos = MacTableActionBar::button_center(bar_rect, 1, btn_w);
    let act_pos = MacTableActionBar::action_button_center(bar_rect, btn_w, act_w);
    assert_eq!(act_pos, Pos2::new(20.0 + 48.0 + 16.0, 31.0));

    // When with_action_menu(false), the divider at min.x + 2 * btn_w must be painted once, not twice
    let no_act_shapes = run_pass(&ctx, RawInput::default(), |ui| {
        MacTableActionBar::new("no_act_bar").theme(&theme).with_action_menu(false).fixed_rect(bar_rect).show(ui);
    });
    let div_48_count = no_act_shapes.iter().filter(|cs| matches!(&cs.shape, egui::Shape::LineSegment { points, .. } if (points[0].x - (bar_rect.min.x + btn_w * 2.0)).abs() < 1e-3 && (points[1].x - (bar_rect.min.x + btn_w * 2.0)).abs() < 1e-3)).count();
    assert_eq!(div_48_count, 1, "Trailing cluster divider at 2 * btn_w must be drawn exactly once");

    let (mut adds, mut rems, mut acts, mut changed_count) = (0usize, 0usize, 0usize, 0usize);
    let bar_shapes = run_pass(&ctx, RawInput::default(), |ui| {
        MacTableActionBar::new("test_bar").theme(&theme).with_action_menu(true).fixed_rect(bar_rect).show(ui);
    });
    assert!(bar_shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Rect(r) if r.fill == theme.box_background())) && bar_shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Rect(r) if r.fill == theme.control())));
    assert!(bar_shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Text(t) if t.galley.text() == egui_macview::MacSymbol::Gear.as_str())));
    for pos in [add_pos, rem_pos, act_pos] {
        for pressed in [true, false] {
            run_pass(&ctx, click_at(pos, pressed), |ui| {
                let r = MacTableActionBar::new("test_bar").theme(&theme).with_action_menu(true).fixed_rect(bar_rect).show(ui);
                if r.add_clicked { adds += 1; }
                if r.remove_clicked { rems += 1; }
                if r.action_clicked { acts += 1; }
                if r.response.changed() { changed_count += 1; }
            });
        }
    }
    assert_eq!((adds, rems, acts, changed_count), (1, 1, 1, 3));

    // When can_remove(false) or narrow fixed_rect (width < cluster_w in Small/Mini), clicks outside/disabled do not fire
    let narrow_rect = Rect::from_min_size(Pos2::new(20.0, 60.0), Vec2::new(34.0, 15.0));
    let outside_act_pos = Pos2::new(narrow_rect.min.x + 50.0, narrow_rect.center().y);
    for pressed in [true, false] {
        run_pass(&ctx, click_at(rem_pos, pressed), |ui| {
            let r = MacTableActionBar::new("test_bar_no_rem").theme(&theme).can_remove(false).fixed_rect(bar_rect).show(ui);
            assert!(!r.remove_clicked);
        });
        let narrow_shapes = run_pass(&ctx, click_at(outside_act_pos, pressed), |ui| {
            let r = MacTableActionBar::new("narrow_mini").theme(&theme).control_size(MacControlSize::Mini).with_action_menu(true).fixed_rect(narrow_rect).show(ui);
            assert!(!r.action_clicked);
        });
        assert!(!narrow_shapes.iter().any(|cs| matches!(&cs.shape, egui::Shape::Text(t) if t.galley.text() == egui_macview::MacSymbol::Gear.as_str())));
    }

    // Compact cluster width when fill_remaining_strip(false) vs full strip width
    let (mut compact_w, mut act_compact_w, mut small_h) = (0.0_f32, 0.0_f32, 0.0_f32);
    let _ = run_pass(&ctx, RawInput::default(), |ui| {
        compact_w = MacTableActionBar::new("compact_bar").theme(&theme).fill_remaining_strip(false).show(ui).response.rect.width();
        act_compact_w = MacTableActionBar::new("act_compact_bar").theme(&theme).with_action_menu(true).fill_remaining_strip(false).show(ui).response.rect.width();
        small_h = MacTableActionBar::new("small_bar").theme(&theme).control_size(MacControlSize::Small).fill_remaining_strip(false).show(ui).response.rect.height();
    });
    assert!((compact_w - btn_w * 2.0).abs() < 0.5);
    assert!((act_compact_w - (btn_w * 2.0 + ab.action_button_width.points())).abs() < 0.5);
    assert!((small_h - ab.small_bar_height.points()).abs() < 0.5);
}
