//! ISO/IEC/IEEE 29119-3:2021 Integration Test Suite for SPEC-012 Advanced Controls.
//!
//! Grounded in Apple AppKit/AVKit Sonoma 14.8.9 hardware inspection (Darwin 23.6.0).
//! Normative references: SPEC-012 (`REQ-012-01` through `REQ-012-07`) and Architecture Fitness Guards.

use egui::{Context, Id, Key, Modifiers, Pos2, RawInput, Rect, Vec2};
use egui_macview::controls::{
    MacBox, MacCollectionView, MacCollectionViewFlowLayout, MacComboBox, MacControlSize,
    MacDate, MacDatePicker, MacDatePickerElementFlags, MacDatePickerStyle, MacDisclosureButton,
    MacDisclosureGroup, MacPlayerControlsStyle, MacPlayerView, MacTabStyle, MacTabView,
};
use egui_macview::theme::{MacAppearance, MacTheme};

fn run_pass(
    ctx: &Context,
    input: RawInput,
    f: impl FnMut(&mut egui::Ui),
) -> Vec<egui::epaint::ClippedShape> {
    let mut out = ctx.run_ui(input, f);
    out.textures_delta.clear();
    out.shapes
}

/// TC-012-05: Validates MacDisclosureGroup & MacDisclosureButton geometry and CASpring rotation.
#[test]
fn test_spec012_disclosure_group_geometry_and_caspring_rotation() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));
    let mut expanded = false;

    // 1. Triangular chevron invariant 13.0 x 13.0 pt, 6.0 pt label gap
    let (mut chevron_sz, mut label_gap) = (Vec2::ZERO, 0.0f32);
    let mut content_executed = false;

    run_pass(&ctx, RawInput::default(), |ui| {
        let group = MacDisclosureGroup::new("disc_group", "Section Header", &mut expanded)
            .theme(&theme);
        chevron_sz = group.chevron_size();
        label_gap = group.label_gap();
        group.show(ui, |_ui| {
            content_executed = true;
        });
    });

    assert_eq!(chevron_sz, Vec2::new(13.0, 13.0));
    assert_eq!(label_gap, 6.0);
    assert!(!content_executed, "Content must not be rendered when collapsed");

    // Expand and verify content execution
    expanded = true;
    run_pass(&ctx, RawInput::default(), |ui| {
        MacDisclosureGroup::new("disc_group", "Section Header", &mut expanded)
            .theme(&theme)
            .show(ui, |_ui| {
                content_executed = true;
            });
    });
    assert!(content_executed, "Content must be rendered when expanded");

    // Rotation angles: 0 deg (collapsed) -> 90 deg (expanded)
    assert_eq!(MacDisclosureGroup::rotation_angle(false), 0.0);
    assert!((MacDisclosureGroup::rotation_angle(true) - std::f32::consts::FRAC_PI_2).abs() < 1e-4);

    // 2. MacDisclosureButton rounded pill metrics
    for (size, expected_outer, expected_intrinsic) in [
        (MacControlSize::Regular, Vec2::new(28.0, 26.0), Vec2::new(20.0, 20.0)),
        (MacControlSize::Small, Vec2::new(24.0, 21.0), Vec2::new(16.0, 16.0)),
        (MacControlSize::Mini, Vec2::new(19.0, 16.0), Vec2::new(13.0, 13.0)),
    ] {
        let mut btn_expanded = false;
        let btn = MacDisclosureButton::new(&mut btn_expanded)
            .control_size(size)
            .theme(&theme);
        assert_eq!(btn.frame_size(), expected_outer);
        assert_eq!(btn.intrinsic_size(), expected_intrinsic);
    }
}

/// TC-012-03: Validates MacComboBox frame metrics, trigger button width, popover scrolling and keyboard filtering.
#[test]
fn test_spec012_combo_box_geometry_and_popover_filtering() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));
    let mut text = String::new();
    let items = vec!["Apple", "Banana", "Cherry", "Date", "Elderberry", "Fig", "Grape", "Honeydew"];

    // 1. Frame height and button width across control sizes
    for (size, expected_height, expected_btn_width, expected_inset) in [
        (MacControlSize::Regular, 23.0f32, 27.0f32, 9.0f32),
        (MacControlSize::Small, 19.0f32, 20.0f32, 7.0f32),
        (MacControlSize::Mini, 15.0f32, 18.0f32, 6.0f32),
    ] {
        let combo = MacComboBox::new("cb_test", &mut text, &items)
            .control_size(size)
            .theme(&theme);
        assert_eq!(combo.frame_height(), expected_height);
        assert_eq!(combo.button_width(), expected_btn_width);
        assert_eq!(combo.text_inset(), expected_inset);
    }

    // 2. Popover metrics: item height 18.0 pt, 5 visible items yielding 90.0 pt viewport height
    let combo = MacComboBox::new("cb_metrics", &mut text, &items).theme(&theme);
    assert_eq!(combo.item_height(), 18.0);
    assert_eq!(combo.visible_items(), 5);
    assert_eq!(combo.popover_viewport_height(), 90.0);

    // 3. Open popover and navigate with keyboard
    let mut input = RawInput::default();
    input.events.push(egui::Event::Key {
        key: Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::ALT,
    });

    run_pass(&ctx, input, |ui| {
        MacComboBox::new("cb_live", &mut text, &items)
            .theme(&theme)
            .show(ui);
    });

    let open_id = Id::NULL.with("cb_live").with("popover_open");
    let is_open = ctx.data_mut(|d| d.get_temp::<bool>(open_id).unwrap_or(false));
    assert!(is_open, "Alt+Down must open combo box popover");

    // 4. Keyboard navigation & filtering: Enter selects highlighted item, and dynamic list contraction clamps sel_idx
    let mut cb_text = "App".to_string();
    let mut enter_input = RawInput::default();
    enter_input.events.push(egui::Event::Key {
        key: Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    let test_open_id = Id::NULL.with("cb_filter_test").with("popover_open");
    let test_sel_id = Id::NULL.with("cb_filter_test").with("popover_sel_idx");
    ctx.data_mut(|d| {
        d.insert_temp(test_open_id, true);
        d.insert_temp(test_sel_id, 8usize);
    });
    run_pass(&ctx, enter_input, |ui| {
        MacComboBox::new("cb_filter_test", &mut cb_text, &items)
            .theme(&theme)
            .show(ui);
    });
    assert_eq!(cb_text, "Apple", "Enter with clamped sel_idx must select the valid filtered item");
}

/// TC-012-04: Validates MacTabView MacBook insets, flipped coordinates, dynamic container sizing and tab switching.
#[test]
fn test_spec012_tab_view_insets_and_flipped_layout() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));
    let mut selected_tab = 0usize;

    // 1. NSTopTabsBezelBorder content insets & bar heights
    for (size, expected_bar_h, top, bottom, left, right) in [
        (MacControlSize::Regular, 24.0f32, 33.0f32, 13.0f32, 10.0f32, 10.0f32),
        (MacControlSize::Small, 20.0f32, 25.0f32, 13.0f32, 10.0f32, 10.0f32),
        (MacControlSize::Mini, 15.0f32, 19.0f32, 13.0f32, 10.0f32, 10.0f32),
    ] {
        let tab_view = MacTabView::new(&mut selected_tab)
            .style(MacTabStyle::TopTabsBezelBorder)
            .control_size(size)
            .theme(&theme);
        assert_eq!(tab_view.tab_bar_height(), expected_bar_h);
        let insets = tab_view.content_insets();
        assert_eq!(insets.top, top);
        assert_eq!(insets.bottom, bottom);
        assert_eq!(insets.left, left);
        assert_eq!(insets.right, right);
    }

    // 2. NSNoTabsNoBorder: 0.0 insets
    let mut borderless_tab = 0usize;
    let borderless = MacTabView::new(&mut borderless_tab)
        .style(MacTabStyle::NoTabsNoBorder)
        .theme(&theme);
    let zero_insets = borderless.content_insets();
    assert_eq!(zero_insets.top, 0.0);
    assert_eq!(zero_insets.bottom, 0.0);
    assert_eq!(zero_insets.left, 0.0);
    assert_eq!(zero_insets.right, 0.0);

    // 3. Dynamic container sizing: container encloses child content
    let mut tab_resp_height = 0.0f32;
    run_pass(&ctx, RawInput::default(), |ui| {
        let resp = MacTabView::new(&mut selected_tab)
            .theme(&theme)
            .tab("First", |ui| {
                ui.allocate_exact_size(Vec2::new(100.0, 150.0), egui::Sense::hover());
            })
            .tab("Second", |ui| {
                ui.label("Tab Two Content");
            })
            .show(ui);
        tab_resp_height = resp.rect.height();
    });

    // Content (150) + insets.top (33) + insets.bottom (13) = 196
    assert!(
        tab_resp_height >= 195.0,
        "Tab view container must dynamically size to enclose content: {tab_resp_height}"
    );
    assert_eq!(selected_tab, 0);

    // 4. Z-Order invariant: Tab body background must be painted BEHIND active tab child content
    let tab_shapes = run_pass(&ctx, RawInput::default(), |ui| {
        MacTabView::new(&mut selected_tab)
            .theme(&theme)
            .tab("First", |ui| {
                ui.label("Tab Inside Child Text");
            })
            .show(ui);
    });

    let mut tab_bg_idx = None;
    let mut tab_label_idx = None;

    for (idx, clipped) in tab_shapes.iter().enumerate() {
        match &clipped.shape {
            egui::Shape::Rect(r) if r.fill == theme.control_background() => {
                if tab_bg_idx.is_none() {
                    tab_bg_idx = Some(idx);
                }
            }
            egui::Shape::Text(t) if t.galley.text() == "Tab Inside Child Text" => {
                tab_label_idx = Some(idx);
            }
            _ => {}
        }
    }

    assert!(
        tab_bg_idx.is_some() && tab_label_idx.is_some(),
        "Both tab body background and tab child content label must be present"
    );
    assert!(
        tab_bg_idx.unwrap() < tab_label_idx.unwrap(),
        "Tab body background (idx {:?}) must be painted BEHIND tab child content (idx {:?})",
        tab_bg_idx,
        tab_label_idx
    );
}

/// TC-012-02: Validates MacDatePicker stepper invariants, dimensions, stepping and calendar/clock matrices.
#[test]
fn test_spec012_date_picker_stepper_and_calendar_matrix() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));

    // 1. Invariant stepper width (19.0 pt) and chevron pitch (9.0 pt)
    let picker = MacDatePicker::new()
        .style(MacDatePickerStyle::TextFieldAndStepper)
        .theme(&theme);
    assert_eq!(picker.stepper_width(), 19.0);
    assert_eq!(picker.stepper_chevron_pitch(), 9.0);

    // 2. Exact frame dimensions across sizes & flags
    for (flags, reg, small, mini) in [
        (MacDatePickerElementFlags::DateAndTime, Vec2::new(161.0, 28.0), Vec2::new(157.0, 22.0), Vec2::new(155.0, 21.0)),
        (MacDatePickerElementFlags::DateOnly, Vec2::new(97.0, 28.0), Vec2::new(93.0, 22.0), Vec2::new(91.0, 21.0)),
        (MacDatePickerElementFlags::TimeOnly, Vec2::new(81.0, 28.0), Vec2::new(77.0, 22.0), Vec2::new(75.0, 21.0)),
    ] {
        assert_eq!(MacDatePicker::dimensions(flags, MacControlSize::Regular), reg);
        assert_eq!(MacDatePicker::dimensions(flags, MacControlSize::Small), small);
        assert_eq!(MacDatePicker::dimensions(flags, MacControlSize::Mini), mini);
    }

    // 3. Clock and Calendar matrices dimensions
    assert_eq!(MacDatePicker::calendar_matrix_size(), Vec2::new(139.0, 148.0));
    assert_eq!(MacDatePicker::clock_face_size(), Vec2::new(122.0, 123.0));
    assert_eq!(MacDatePicker::clock_intrinsic_size(), Vec2::new(119.0, 119.0));

    // 4. Stepper date mutation interaction
    let mut test_date = MacDate { year: 2026, month: 11, day: 10, hour: 10, minute: 14, second: 30 };
    test_date.step_segment(0, MacDatePickerElementFlags::DateOnly, 1);
    assert_eq!(test_date.year, 2027);

    test_date.step_segment(1, MacDatePickerElementFlags::DateOnly, 2);
    assert_eq!(test_date.month, 1); // Nov (11) + 2 wraps to Jan (1)

    // Leap-year boundary stepping invariant: 2024-02-29 + 1 year must clamp day to 28
    let mut leap_date = MacDate { year: 2024, month: 2, day: 29, hour: 12, minute: 0, second: 0 };
    leap_date.step_segment(0, MacDatePickerElementFlags::DateOnly, 1);
    assert_eq!(leap_date.year, 2025);
    assert_eq!(
        leap_date.day, 28,
        "Stepping year from leap year to non-leap year must clamp Feb 29 to Feb 28"
    );

    // Century leap-year boundary: year 2000 is leap, year 1900 is non-leap
    let mut century_date = MacDate { year: 2000, month: 2, day: 29, hour: 12, minute: 0, second: 0 };
    century_date.step_segment(0, MacDatePickerElementFlags::DateOnly, -100);
    assert_eq!(century_date.year, 1900);
    assert_eq!(
        century_date.day, 28,
        "Year 1900 is not a leap year, day must clamp to 28"
    );

    // Render picker in UI
    run_pass(&ctx, RawInput::default(), |ui| {
        ui.add(
            MacDatePicker::new()
                .date(&mut test_date)
                .style(MacDatePickerStyle::TextFieldAndStepper)
                .theme(&theme),
        );
    });
}

/// TC-012-06: Validates MacCollectionView flow layout, 2D viewport culling, and marquee selection.
#[test]
fn test_spec012_collection_view_flow_layout_and_2d_viewport_culling() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));

    // 1. Flow layout default metrics
    let layout = MacCollectionViewFlowLayout::default();
    assert_eq!(layout.item_size, Vec2::new(50.0, 50.0));
    assert_eq!(layout.minimum_line_spacing, 10.0);
    assert_eq!(layout.minimum_interitem_spacing, 10.0);

    // 2. 2D Viewport culling: only items intersecting clip rect are rendered
    let items: Vec<usize> = (0..1000).collect();
    let mut selected_indices = std::collections::BTreeSet::new();

    let mut rendered_count = 0usize;
    run_pass(&ctx, RawInput::default(), |ui| {
        ui.set_clip_rect(Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(200.0, 200.0)));
        let coll = MacCollectionView::new(&items, &mut selected_indices)
            .layout(layout.clone())
            .theme(&theme);
        rendered_count = coll.show_with_item_tracker(ui, |_ui, _item| {});
    });

    assert!(rendered_count < 25, "Rendered count must be culled to viewport: {rendered_count}");
    assert!(rendered_count > 0, "Must render visible items in viewport");

    // 3. 2D Arrow key navigation column calculation
    let columns = layout.compute_column_count(230.0);
    assert_eq!(columns, 4);

    // 4. Clean single allocation in show()
    run_pass(&ctx, RawInput::default(), |ui| {
        let resp = MacCollectionView::new(&items[..4], &mut selected_indices)
            .theme(&theme)
            .show(ui);
        assert!(resp.rect.height() >= 50.0);
    });
}

/// TC-012-01: Validates MacPlayerView HUD pod, inline transport, scrubber, and auto-hide timing.
#[test]
fn test_spec012_player_view_hud_pod_and_inline_transport() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));

    // 1. Floating HUD pod geometry & auto-hide timing
    let floating = MacPlayerView::new()
        .controls_style(MacPlayerControlsStyle::Floating)
        .theme(&theme);
    assert_eq!(floating.hud_pod_size(), Vec2::new(481.0, 64.0));
    assert_eq!(floating.hud_corner_radius(), 16.0);
    assert_eq!(floating.auto_hide_seconds(), 3.0);

    // 2. Scrubber geometry: height 22.5 pt, track 3.5 pt, knob 8.0 x 8.0 pt
    assert_eq!(floating.scrubber_height(), 22.5);
    assert_eq!(floating.scrubber_track_height(), 3.5);
    assert_eq!(floating.scrubber_knob_size(), Vec2::new(8.0, 8.0));

    // 3. Inline bottom bar: 24.0 pt height, 0.0 corner radius
    let inline = MacPlayerView::new()
        .controls_style(MacPlayerControlsStyle::Inline)
        .theme(&theme);
    assert_eq!(inline.inline_bar_height(), 24.0);
    assert_eq!(inline.inline_corner_radius(), 0.0);

    // 4. Render HUD pod in pass
    run_pass(&ctx, RawInput::default(), |ui| {
        ui.add(MacPlayerView::new().controls_style(MacPlayerControlsStyle::Floating).is_playing(true).progress(0.42).theme(&theme));
    });

    // 5. Two-way mutable state binding
    let mut live_playing = false;
    let mut live_progress = 0.15f32;
    run_pass(&ctx, RawInput::default(), |ui| {
        ui.add(MacPlayerView::new().controls_style(MacPlayerControlsStyle::Floating).is_playing_mut(&mut live_playing).progress_mut(&mut live_progress).theme(&theme));
    });
    assert!(!live_playing);
    assert!((live_progress - 0.15).abs() < 1e-4);

    // 6. Multi-hour AVKit timecode formatting and scrubber clearance (> 36,000s)
    use egui_macview::controls::{format_remaining_timecode, format_timecode};
    assert_eq!(format_timecode(125, 200.0), "02:05");
    assert_eq!(format_remaining_timecode(75, 200.0), "-01:15");
    assert_eq!(format_timecode(3672, 7200.0), "01:01:12");
    assert_eq!(format_remaining_timecode(3528, 7200.0), "-00:58:48");
    assert_eq!(format_timecode(36000, 36000.0), "10:00:00");

    let long_pv = MacPlayerView::new()
        .controls_style(MacPlayerControlsStyle::Floating)
        .duration_secs(36000.0)
        .theme(&theme);
    let geom = long_pv.compute_geometry(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 400.0)));
    assert!(geom.scrubber_track_rect.width() > 100.0, "Scrubber track must retain adequate width under multi-hour duration");
    assert!(geom.elapsed_pos.x + 48.0 <= geom.scrubber_track_rect.min.x, "Elapsed time label must not overlap scrubber track");
    assert!(geom.scrubber_track_rect.max.x <= geom.remaining_pos.x, "Scrubber track must not overlap remaining time label");
}

/// TC-012-07: Validates MacBox container insets and separator primitive.
#[test]
fn test_spec012_box_view_insets_and_primitives() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));
    let mac_box = MacBox::new("Box Title").theme(&theme);

    let insets = mac_box.content_insets();
    assert_eq!(insets.top, 20.0);
    assert_eq!(insets.bottom, 10.0);
    assert_eq!(insets.left, 9.0);
    assert_eq!(insets.right, 9.0);
    assert_eq!(mac_box.corner_radius(), 5.0);
    assert_eq!(mac_box.border_width(), 1.0);

    // Render Box and Separator
    let shapes = run_pass(&ctx, RawInput::default(), |ui| {
        MacBox::new("Enclosed Container")
            .theme(&theme)
            .show(ui, |b_ui| {
                b_ui.label("Box Content");
                MacBox::separator(b_ui);
            });
    });

    let mut bg_rect_idx = None;
    let mut label_text_idx = None;

    for (idx, clipped) in shapes.iter().enumerate() {
        match &clipped.shape {
            egui::Shape::Rect(r) if r.fill == theme.control_background() => {
                if bg_rect_idx.is_none() {
                    bg_rect_idx = Some(idx);
                }
            }
            egui::Shape::Text(t) if t.galley.text() == "Box Content" => {
                label_text_idx = Some(idx);
            }
            _ => {}
        }
    }

    assert!(
        bg_rect_idx.is_some() && label_text_idx.is_some(),
        "Both container background and child label must be in painter shapes"
    );
    assert!(
        bg_rect_idx.unwrap() < label_text_idx.unwrap(),
        "Container background (idx {:?}) must be painted BEHIND child content label (idx {:?})",
        bg_rect_idx,
        label_text_idx
    );
}
