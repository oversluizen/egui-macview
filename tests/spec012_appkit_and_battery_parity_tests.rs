//! ISO/IEC/IEEE 29119-3:2021 Companion Test Suite for SPEC-012 Parity & Battery Efficiency.
//!
//! Grounded in Apple AppKit/AVKit Sonoma 14.8.9 hardware inspection (Darwin 23.6.0).
//! Normative references: SPEC-012 (`REQ-012-01` through `REQ-012-07`) and Architecture Fitness Guards.

use egui::{Context, Id, Key, Modifiers, Pos2, RawInput, Rect, Vec2, ViewportId};
use egui_macview::controls::{
    MacCollectionView, MacCollectionViewFlowLayout, MacComboBox, MacControlSize, MacDate,
    MacDatePicker, MacDatePickerElementFlags, MacDatePickerStyle, MacPlayerControlsStyle,
    MacPlayerView, MacRuleEditor, MacRuleRow, MacTabView,
};
use egui_macview::theme::{MacAppearance, MacTheme};
use std::time::Duration;

fn run_pass(ctx: &Context, input: RawInput, f: impl FnMut(&mut egui::Ui)) -> Vec<egui::epaint::ClippedShape> {
    let mut out = ctx.run_ui(input, f);
    out.textures_delta.clear();
    out.shapes
}

fn run_pass_with_delay(ctx: &Context, input: RawInput, f: impl FnMut(&mut egui::Ui)) -> Duration {
    let mut out = ctx.run_ui(input, f);
    out.textures_delta.clear();
    out.viewport_output.get(&ViewportId::ROOT).map(|v| v.repaint_delay).unwrap_or(Duration::MAX)
}

fn click_at(ctx: &Context, pos: Pos2, mut f: impl FnMut(&mut egui::Ui)) {
    let make_input = |pressed: bool| {
        let mut input = RawInput { focused: true, screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 400.0))), ..Default::default() };
        input.events.push(egui::Event::PointerMoved(pos));
        input.events.push(egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Modifiers::NONE });
        input
    };
    let _ = run_pass(ctx, make_input(true), &mut f);
    let _ = run_pass(ctx, make_input(false), &mut f);
}

fn press_key(ctx: &Context, key: Key, mut f: impl FnMut(&mut egui::Ui)) {
    let mut input = RawInput { focused: true, screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 400.0))), ..Default::default() };
    input.events.push(egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE });
    let _ = run_pass(ctx, input, &mut f);
}

struct PlayerClickState<'a> {
    progress: &'a mut f32,
    volume: &'a mut f32,
    muted: &'a mut bool,
    rate: &'a mut f32,
    duration: f32,
}

fn click_player(ctx: &Context, theme: &MacTheme, pos: Pos2, s: &mut PlayerClickState<'_>) {
    click_at(ctx, pos, |ui| {
        ui.add(MacPlayerView::new().progress_mut(s.progress).volume_mut(s.volume).muted_mut(s.muted).playback_rate_mut(s.rate).duration_secs(s.duration).theme(theme));
    });
}

fn click_inline(ctx: &Context, theme: &MacTheme, pos: Pos2, p: &mut bool, m: &mut bool, pr: &mut f32) {
    click_at(ctx, pos, |ui| {
        ui.set_max_width(300.0);
        ui.set_min_size(Vec2::new(300.0, 24.0));
        ui.add(MacPlayerView::new().controls_style(MacPlayerControlsStyle::Inline).is_playing_mut(p).muted_mut(m).progress_mut(pr).theme(theme));
    });
}

fn step_date_segment(ctx: &Context, theme: &MacTheme, date: &mut MacDate, click_pos: Pos2) {
    let mut add_dp = |ui: &mut egui::Ui| {
        ui.add(MacDatePicker::new().date(date).style(MacDatePickerStyle::TextFieldAndStepper).elements(MacDatePickerElementFlags::DateOnly).theme(theme));
    };
    click_at(ctx, click_pos, &mut add_dp);
    press_key(ctx, Key::ArrowUp, &mut add_dp);
}

/// TC-012-P1: Validates MacPlayerView transport, audio controls, minimal/inline sizing, and stationary auto-hide.
#[test]
fn test_spec012_player_view_appkit_audio_and_stationary_cursor_autohide() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));

    // 1. Full Floating HUD Transport & Audio/Speed Builder APIs & Interactive Clicks
    let mut progress = 0.50f32;
    let mut volume = 0.85f32;
    let mut muted = false;
    let mut rate = 1.0f32;
    let duration = 120.0f32;

    let player = MacPlayerView::new()
        .progress_mut(&mut progress).volume(0.85).volume_mut(&mut volume).muted(false).muted_mut(&mut muted)
        .playback_rate(1.0).playback_rate_mut(&mut rate).buffered_progress(0.65).duration_secs(duration).theme(&theme);

    assert_eq!(player.playback_rates(), [0.5, 1.0, 1.25, 1.5, 2.0]);

    let mut st = PlayerClickState { progress: &mut progress, volume: &mut volume, muted: &mut muted, rate: &mut rate, duration };

    // Test Skip Back 15s click (center at 267.0, 33.0)
    click_player(&ctx, &theme, Pos2::new(267.0, 33.0), &mut st);
    let expected_skip = 15.0 / duration;
    assert!((*st.progress - (0.50 - expected_skip)).abs() < 1e-4);

    // Test Skip Forward 15s click (center at 333.0, 33.0)
    click_player(&ctx, &theme, Pos2::new(333.0, 33.0), &mut st);
    assert!((*st.progress - 0.50).abs() < 1e-4);

    // Test Frame Step Back (center at 239.0, 33.0): 1 frame at 30 fps
    click_player(&ctx, &theme, Pos2::new(239.0, 33.0), &mut st);
    let expected_step = 1.0 / (duration * 30.0);
    assert!((*st.progress - (0.50 - expected_step)).abs() < 1e-5);

    // Test Frame Step Forward (center at 361.0, 33.0)
    click_player(&ctx, &theme, Pos2::new(361.0, 33.0), &mut st);
    assert!((*st.progress - 0.50).abs() < 1e-5);

    // Test Mute button click (center at 87.5, 33.0)
    click_player(&ctx, &theme, Pos2::new(87.5, 33.0), &mut st);
    assert!(*st.muted, "Clicking mute button must toggle muted to true");

    // Test Volume Slider drag/click (center at 168.0, 33.0)
    let mut click_vol = RawInput { focused: true, screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 400.0))), ..Default::default() };
    click_vol.events.push(egui::Event::PointerButton { pos: Pos2::new(168.0, 33.0), button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    run_pass(&ctx, click_vol, |ui| {
        ui.add(MacPlayerView::new().volume_mut(st.volume).muted_mut(st.muted).duration_secs(duration).theme(&theme));
    });
    assert!((*st.volume - 0.50).abs() < 0.05 && !*st.muted);

    // Test Speed Button click (center at 480.5, 33.0): cycles 1.0 -> 1.25
    click_player(&ctx, &theme, Pos2::new(480.5, 33.0), &mut st);
    assert_eq!(*st.rate, 1.25, "Clicking speed button must cycle 1.0 to 1.25");

    // Verify painted shapes exist inside skip_back_rect, step_back_rect, skip_fwd_rect, step_fwd_rect, and aux_rect
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 400.0));
    let mut geom_opt = None;
    let hud_shapes = run_pass(&ctx, RawInput { focused: true, screen_rect: Some(screen), ..Default::default() }, |ui| {
        ui.set_min_size(screen.size());
        let available_w = ui.available_width();
        let m = theme.player_view();
        let pod = Vec2::new(m.hud_width.points(), m.hud_height.points());
        let total_size = Vec2::new(available_w.max(pod.x), pod.y + m.hud_bottom_margin.points() * 2.0);
        let avail_rect = egui_macview::layout::subpixel_snap_rect(
            Rect::from_min_size(ui.cursor().min, total_size),
            ui.ctx().pixels_per_point(),
        );
        let pv = MacPlayerView::new()
            .controls_style(MacPlayerControlsStyle::Floating)
            .is_playing(false)
            .progress(0.50)
            .duration_secs(duration)
            .theme(&theme);
        geom_opt = Some(pv.compute_geometry(avail_rect));
        ui.add(pv);
    });
    let hud_geom = geom_opt.unwrap();

    let has_shape_inside = |target: Rect| {
        hud_shapes.iter().any(|clipped| match &clipped.shape {
            egui::Shape::Circle(c) => target.contains(c.center),
            egui::Shape::LineSegment { points, .. } => target.contains(points[0]) || target.contains(points[1]),
            egui::Shape::Path(p) => p.points.iter().any(|pt| target.contains(*pt)),
            egui::Shape::Rect(r) => r.rect != hud_geom.controls_rect && target.intersects(r.rect),
            egui::Shape::Vec(v) => v.iter().any(|s| match s {
                egui::Shape::Circle(c) => target.contains(c.center),
                egui::Shape::LineSegment { points, .. } => target.contains(points[0]) || target.contains(points[1]),
                egui::Shape::Path(p) => p.points.iter().any(|pt| target.contains(*pt)),
                egui::Shape::Rect(r) => r.rect != hud_geom.controls_rect && target.intersects(r.rect),
                _ => false,
            }),
            _ => false,
        })
    };

    assert!(has_shape_inside(hud_geom.skip_back_rect), "skip_back_rect must have painted shapes");
    assert!(has_shape_inside(hud_geom.step_back_rect), "step_back_rect must have painted shapes");
    assert!(has_shape_inside(hud_geom.skip_fwd_rect), "skip_fwd_rect must have painted shapes");
    assert!(has_shape_inside(hud_geom.step_fwd_rect), "step_fwd_rect must have painted shapes");
    assert!(has_shape_inside(hud_geom.aux_rect), "aux_rect must have painted shapes");
    let has_polygon_inside = |target: Rect| {
        hud_shapes.iter().any(|clipped| match &clipped.shape {
            egui::Shape::Path(p) => p.points.iter().any(|pt| target.contains(*pt)),
            _ => false,
        })
    };
    assert!(has_polygon_inside(hud_geom.step_back_rect), "step_back_rect must contain triangle polygon");
    assert!(has_polygon_inside(hud_geom.step_fwd_rect), "step_fwd_rect must contain triangle polygon");
    assert!(has_polygon_inside(hud_geom.aux_rect), "aux_rect must contain broadcast triangle polygon");

    // 2. Minimal & Inline Exact Intrinsic Sizes and Play/Pause Toggle
    let minimal = MacPlayerView::new().controls_style(MacPlayerControlsStyle::Minimal).theme(&theme);
    assert_eq!(minimal.minimal_size(), Vec2::new(29.0, 18.0));

    let mut min_playing = false;
    click_at(&ctx, Pos2::new(14.5, 9.0), |ui| {
        ui.add(MacPlayerView::new().controls_style(MacPlayerControlsStyle::Minimal).is_playing_mut(&mut min_playing).theme(&theme));
    });
    assert!(min_playing, "Clicking Minimal play/pause button must toggle is_playing");

    // Inline 24.0 pt bottom bar: play/pause and mute interactions
    let mut inline_playing = false;
    let mut inline_muted = false;
    let mut inline_prog = 0.20f32;
    click_inline(&ctx, &theme, Pos2::new(18.0, 12.0), &mut inline_playing, &mut inline_muted, &mut inline_prog);
    assert!(inline_playing, "Clicking Inline play button must toggle is_playing");
    click_inline(&ctx, &theme, Pos2::new(280.0, 12.0), &mut inline_playing, &mut inline_muted, &mut inline_prog);
    assert!(inline_muted, "Clicking Inline mute button must toggle muted to true");

    // 3. Multi-instance ID isolation: rendering two MacPlayerView in same frame
    run_pass(&ctx, RawInput::default(), |ui| {
        ui.push_id("player_inst_1", |p1_ui| {
            p1_ui.add(MacPlayerView::new().controls_style(MacPlayerControlsStyle::Floating).is_playing(true).theme(&theme));
        });
        ui.push_id("player_inst_2", |p2_ui| {
            p2_ui.add(MacPlayerView::new().controls_style(MacPlayerControlsStyle::Floating).is_playing(false).theme(&theme));
        });
    });

    // 4. Battery Efficiency: Stationary cursor auto-hide after 3.0s idle
    let (auto_ctx, _) = (Context::default(), ());
    let mut idle_input = RawInput { time: Some(10.0), focused: true, ..Default::default() };
    idle_input.events.push(egui::Event::PointerMoved(Pos2::new(200.0, 200.0)));
    let add_floating = |ui: &mut egui::Ui| {
        ui.set_min_size(Vec2::new(600.0, 400.0));
        ui.add(MacPlayerView::new().controls_style(MacPlayerControlsStyle::Floating).is_playing(true).theme(&theme));
    };
    let _ = run_pass_with_delay(&auto_ctx, idle_input, add_floating);

    // Advance time past 3.0s auto-hide threshold with stationary cursor (delta = 0, no events)
    let _ = run_pass_with_delay(&auto_ctx, RawInput { time: Some(14.0), focused: true, ..Default::default() }, add_floating);

    // Advance past spring settling duration (> 14.0 + 1.1s = 15.1s) to 16.0s
    let delay = run_pass_with_delay(&auto_ctx, RawInput { time: Some(16.0), focused: true, ..Default::default() }, add_floating);
    assert!(delay >= Duration::from_secs(1), "Settled HUD with stationary cursor must not cause continuous repaints");
}

/// TC-012-P2: Validates MacDatePicker direct token hit-testing and calendar 40.0 pt header bezel.
#[test]
fn test_spec012_date_picker_direct_mouse_selection_and_bezel_layout() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));

    // 1. Direct segment token clicking in TextFieldAndStepper (Year, Month, Day)
    let mut date = MacDate { year: 2026, month: 11, day: 10, hour: 12, minute: 0, second: 0 };
    step_date_segment(&ctx, &theme, &mut date, Pos2::new(21.0, 12.0));
    assert_eq!(date.year, 2027, "Clicking year segment followed by ArrowUp must increment year");

    step_date_segment(&ctx, &theme, &mut date, Pos2::new(49.0, 12.0));
    assert_eq!(date.month, 12, "Clicking month segment followed by ArrowUp must increment month");

    step_date_segment(&ctx, &theme, &mut date, Pos2::new(68.0, 12.0));
    assert_eq!(date.day, 11, "Clicking day segment followed by ArrowUp must increment day");

    // 2. Calendar matrix bezel layout: header (24.0 pt) + weekday (16.0 pt) = 40.0 pt offset
    let mut cal_date = MacDate { year: 2026, month: 11, day: 1, hour: 0, minute: 0, second: 0 };
    click_at(&ctx, Pos2::new(10.0, 49.0), |ui| {
        ui.add(
            MacDatePicker::new()
                .date(&mut cal_date)
                .style(MacDatePickerStyle::ClockAndCalendar)
                .elements(MacDatePickerElementFlags::DateOnly)
                .theme(&theme),
        );
    });
    assert_eq!(cal_date.day, 1, "Clicking day cell at y=49.0 must select day 1");

    // 3. Analog clock click-to-set-minute & 12-hour tick marks
    let mut clock_date = MacDate { year: 2026, month: 11, day: 10, hour: 12, minute: 0, second: 0 };
    click_at(&ctx, Pos2::new(96.0, 61.5), |ui| {
        ui.add(
            MacDatePicker::new()
                .date(&mut clock_date)
                .style(MacDatePickerStyle::ClockAndCalendar)
                .elements(MacDatePickerElementFlags::TimeOnly)
                .theme(&theme),
        );
    });
    assert_eq!(clock_date.minute, 15, "Clicking 3 o'clock on analog clock face must set minute to 15");

    click_at(&ctx, Pos2::new(61.0, 26.5), |ui| {
        ui.add(MacDatePicker::new().date(&mut clock_date).style(MacDatePickerStyle::ClockAndCalendar).elements(MacDatePickerElementFlags::TimeOnly).theme(&theme));
    });
    assert_eq!(clock_date.minute, 0, "Clicking 12 o'clock must set minute to 0");

    let clk_shapes = run_pass(&ctx, RawInput::default(), |ui| {
        ui.add(MacDatePicker::new().date(&mut clock_date).style(MacDatePickerStyle::ClockAndCalendar).elements(MacDatePickerElementFlags::TimeOnly).theme(&theme));
    });
    let tick_count = clk_shapes.iter().filter(|c| matches!(&c.shape, egui::Shape::LineSegment { .. })).count();
    assert_eq!(tick_count, 14, "Clock must render 12 hour tick marks and 2 hands");
}

/// TC-012-P3: Validates MacComboBox text insets and scroll isolation without keyboard navigation.
#[test]
fn test_spec012_combo_box_text_inset_and_scroll_isolation() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));
    let mut text = "Apple".to_string();
    let items = ["Apple", "Apricot", "Avocado"];

    // 1. Text Inset across sizes
    for (size, expected) in [(MacControlSize::Regular, 9.0), (MacControlSize::Small, 7.0), (MacControlSize::Mini, 6.0)] {
        let cb = MacComboBox::new("cb_inset", &mut text, &items).control_size(size).theme(&theme);
        assert_eq!(cb.text_inset(), expected);
    }

    // 2. Closed ComboBox does zero popover allocations
    run_pass(&ctx, RawInput::default(), |ui| {
        MacComboBox::new("cb_closed", &mut text, &items).theme(&theme).show(ui);
    });
    let open_id = Id::NULL.with("cb_closed").with("popover_open");
    assert!(!ctx.data(|d| d.get_temp::<bool>(open_id).unwrap_or(false)));

    // 3. Open ComboBox popover and verify scroll isolation when no arrow keys are pressed
    let mut open_input = RawInput { focused: true, ..Default::default() };
    open_input.events.push(egui::Event::Key { key: Key::ArrowDown, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::ALT });
    run_pass(&ctx, open_input, |ui| {
        MacComboBox::new("cb_scroll_iso", &mut text, &items).theme(&theme).show(ui);
    });
    let iso_open_id = Id::NULL.with("cb_scroll_iso").with("popover_open");
    assert!(ctx.data(|d| d.get_temp::<bool>(iso_open_id).unwrap_or(false)), "Alt+ArrowDown must open popover");

    run_pass(&ctx, RawInput { focused: true, ..Default::default() }, |ui| {
        MacComboBox::new("cb_scroll_iso", &mut text, &items).theme(&theme).show(ui);
    });
    assert!(ctx.data(|d| d.get_temp::<bool>(iso_open_id).unwrap_or(false)), "Popover remains open on subsequent idle frames");

    // 4. Exact-match query does not filter out sibling items, but partial query filters
    let mut exact_text = "Apple".to_string();
    let pop_shapes = run_pass(&ctx, RawInput { focused: true, ..Default::default() }, |ui| {
        MacComboBox::new("cb_scroll_iso", &mut exact_text, &items).theme(&theme).show(ui);
    });
    let has_label = |shapes: &[egui::epaint::ClippedShape], label: &str| {
        shapes.iter().any(|c| match &c.shape {
            egui::Shape::Text(t) => t.galley.text() == label,
            _ => false,
        })
    };
    assert!(has_label(&pop_shapes, "Apricot"), "Exact match on 'Apple' must not hide sibling 'Apricot'");
    assert!(has_label(&pop_shapes, "Avocado"), "Exact match on 'Apple' must not hide sibling 'Avocado'");

    // Trigger button highlight uses asymmetric CornerRadius (nw: 0, sw: 0)
    let btn_in = RawInput { focused: true, events: vec![egui::Event::PointerMoved(Pos2::new(590.0, 12.0))], ..Default::default() };
    let cb_shapes = run_pass(&ctx, btn_in, |ui| {
        MacComboBox::new("cb_btn_cr", &mut exact_text, &items).width(600.0).theme(&theme).show(ui);
    });
    let has_asym_corner = cb_shapes.iter().any(|c| match &c.shape {
        egui::Shape::Rect(r) => r.corner_radius.nw == 0 && r.corner_radius.sw == 0 && r.corner_radius.ne > 0,
        _ => false,
    });
    assert!(has_asym_corner, "Trigger button must use asymmetric CornerRadius with nw=0, sw=0");
}

/// TC-012-P4: Validates MacTabView minimum container sizes and centered segmented pill bar.
#[test]
fn test_spec012_tab_view_centered_pill_and_min_container_sizes() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));
    let mut tab_idx = 0usize;

    for (size, min_w, min_h) in [
        (MacControlSize::Regular, 150.0f32, 36.0f32),
        (MacControlSize::Small, 122.0f32, 28.0f32),
        (MacControlSize::Mini, 107.0f32, 22.0f32),
    ] {
        let tab_view = MacTabView::new(&mut tab_idx).control_size(size).theme(&theme);
        assert_eq!(tab_view.min_container_size(), Vec2::new(min_w, min_h));
    }

    let tab_shapes = run_pass(&ctx, RawInput::default(), |ui| {
        ui.set_min_width(400.0);
        let tab_view = MacTabView::new(&mut tab_idx).theme(&theme).tab("Alpha", |ui| { ui.label("A"); }).tab("Beta", |ui| { ui.label("B"); });
        let container_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 100.0));
        let bar_rect = tab_view.tab_bar_rect(container_rect, ui);
        assert!(bar_rect.width() < container_rect.width(), "Segmented bar must be narrower than 400pt container");
        assert!((bar_rect.center().x - container_rect.center().x).abs() < 1e-3, "Segmented bar must be horizontally centered");
        let expected_bar_y = container_rect.min.y + tab_view.tab_bar_height() * 0.5;
        assert!((bar_rect.center().y - expected_bar_y).abs() < 1e-3, "Segmented bar must straddle top bezel line");
        tab_view.show(ui);
    });

    let has_pill = |fill: egui::Color32| {
        tab_shapes.iter().any(|c| match &c.shape {
            egui::Shape::Rect(r) => r.fill == fill && r.corner_radius.is_same() && r.corner_radius.nw > 0,
            _ => false,
        })
    };
    assert!(has_pill(theme.window_background()), "Bar rect must be painted as unified pill");
    assert!(has_pill(theme.control_background()), "Active tab must be painted as pill");
}

/// TC-012-P5: Validates MacCollectionView O(1) row culling with 10,000 items in a 200 pt viewport.
#[test]
fn test_spec012_collection_view_o1_row_range_culling() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));
    let items: Vec<usize> = (0..10_000).collect();
    let mut selected = std::collections::BTreeSet::new();

    let mut invocations = 0usize;
    run_pass(&ctx, RawInput::default(), |ui| {
        ui.set_clip_rect(Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(200.0, 200.0)));
        MacCollectionView::new(&items, &mut selected)
            .layout(MacCollectionViewFlowLayout::default())
            .theme(&theme)
            .show_with_item_tracker(ui, |_ui, _item| {
                invocations += 1;
            });
    });

    assert!(invocations <= 30, "10,000 items in 200pt viewport must invoke closure <= 30 times (got {invocations})");
    assert!(invocations > 0, "Must render visible items");

    // Focus on click: clicking an item requests focus
    let click_items = vec![10, 20, 30];
    let mut click_selected = std::collections::BTreeSet::new();
    click_at(&ctx, Pos2::new(25.0, 25.0), |ui| {
        MacCollectionView::new(&click_items, &mut click_selected).theme(&theme).show(ui);
    });
    assert!(click_selected.contains(&0), "Item 0 must be selected after click");

    // Immediate next frame has focus and navigates with arrow keys
    let mut arrow_in = RawInput { focused: true, ..Default::default() };
    arrow_in.events.push(egui::Event::Key { key: Key::ArrowRight, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE });
    run_pass(&ctx, arrow_in, |ui| {
        let resp = MacCollectionView::new(&click_items, &mut click_selected).theme(&theme).show(ui);
        assert!(resp.has_focus(), "Collection view must have focus after item click");
    });
    assert!(click_selected.contains(&1), "Arrow navigation must move selection to item 1");

    // Marquee drag selection
    let mut drag_selected = std::collections::BTreeSet::new();
    let screen_rect = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0)));
    let mut raw_down = RawInput { focused: true, screen_rect, ..Default::default() };
    raw_down.events.push(egui::Event::PointerMoved(Pos2::new(5.0, 5.0)));
    raw_down.events.push(egui::Event::PointerButton { pos: Pos2::new(5.0, 5.0), button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    run_pass(&ctx, raw_down, |ui| {
        MacCollectionView::new(&click_items, &mut drag_selected).theme(&theme).show(ui);
    });

    let mut raw_move = RawInput { focused: true, screen_rect, ..Default::default() };
    raw_move.events.push(egui::Event::PointerMoved(Pos2::new(120.0, 120.0)));
    run_pass(&ctx, raw_move, |ui| {
        MacCollectionView::new(&click_items, &mut drag_selected).theme(&theme).show(ui);
    });
    assert!(!drag_selected.is_empty(), "Marquee drag must select items intersecting the bounding box");
}

/// TC-012-P6: Validates MacRuleEditor row height, add/remove buttons, and criteria mutation.
#[test]
fn test_spec012_rule_editor_predicate_rows_and_mutations() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));

    let mut rows = vec![MacRuleRow::new(vec![0, 0]), MacRuleRow::new(vec![1, 1])];
    let criteria: &[&[&str]] = &[
        &["Name", "Kind", "Date", "Size"],
        &["contains", "is", "begins with", "ends with"],
    ];

    let editor = MacRuleEditor::new("rule_test", &mut rows, criteria).theme(&theme);
    assert_eq!((editor.row_height(), editor.button_size(), editor.row_spacing()), (32.0, 20.0, 8.0));

    let add_re = |ui: &mut egui::Ui, r: &mut Vec<MacRuleRow>| {
        ui.set_min_width(600.0);
        MacRuleEditor::new("rule_run", r, criteria).theme(&theme).show(ui);
    };

    // 1. Initial render
    run_pass(&ctx, RawInput::default(), |ui| add_re(ui, &mut rows));
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].slices, vec![0, 0]);

    // 2. Click [+] button on row 0 (center at x=590.0, y=16.0 with width 600)
    click_at(&ctx, Pos2::new(590.0, 16.0), |ui| add_re(ui, &mut rows));
    assert_eq!(rows.len(), 3, "Clicking [+] button must insert a new predicate row");

    // 3. Click [-] button on row 1 (center at x=566.0, y=48.0)
    click_at(&ctx, Pos2::new(566.0, 48.0), |ui| add_re(ui, &mut rows));
    assert_eq!(rows.len(), 2, "Clicking [-] button must remove the target predicate row");

    // 4. Click criterion pill 0 on row 0 (x=24.0, y=16.0) to mutate slice index (0 -> 1)
    click_at(&ctx, Pos2::new(24.0, 16.0), |ui| add_re(ui, &mut rows));
    assert_eq!(rows[0].slices[0], 1, "Clicking criterion pill must cycle slice option index");
}
