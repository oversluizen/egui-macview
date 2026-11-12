//! Authentic AppKit `NSSegmentedControl` Vector Icon, Per-Segment Metrics & Menu Trigger Tests.
//!
//! Verifies clean-room Apple AppKit `NSSegmentedControl` parity (`SPEC-002` §2.3):
//! - Asymmetric per-segment widths (`setWidth(_:forSegment:)`), e.g. `[34, 33, 33, 33, 33, 33, 35]` (`234 x 24 pt`)
//! - Template vector closures (`SegmentItem::from_icon`) and `MacSymbol` tinting across active, inactive, and disabled states
//! - Per-segment enablement (`setEnabled(_:forSegment:)`) and keyboard arrow traversal skipping disabled segments
//! - Orthogonal `with_menu` (`setMenu(_:forSegment:)`) vs. `with_shows_menu_indicator` (`setShowsMenuIndicator(_:forSegment:)`)
//! - Press-and-hold (`250ms`), downward pull (`4pt`), and secondary-click menu triggers with `Duration::MAX` idle hygiene
//! - macOS Sonoma divider suppression adjacent to the active floating selection pill.

use egui::{pos2, Color32, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Shape, Vec2};
use egui_macview::{
    setup_macview_fonts, MacSegmentedControl, MacSymbol, MacTheme, SegmentEvent, SegmentItem,
    SegmentMenuTrigger,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

fn collect_line_segments(shapes: &[egui::epaint::ClippedShape]) -> Vec<[Pos2; 2]> {
    let mut out = Vec::new();
    for cs in shapes {
        collect_lines_from_shape(&cs.shape, &mut out);
    }
    out
}

fn collect_lines_from_shape(shape: &Shape, out: &mut Vec<[Pos2; 2]>) {
    match shape {
        Shape::LineSegment { points, .. } => out.push(*points),
        Shape::Vec(inner) => inner.iter().for_each(|s| collect_lines_from_shape(s, out)),
        _ => {}
    }
}

fn has_convex_polygon(shapes: &[egui::epaint::ClippedShape]) -> bool {
    fn check(shape: &Shape) -> bool {
        match shape {
            Shape::Path(p) => p.closed && p.points.len() == 3,
            Shape::Vec(inner) => inner.iter().any(check),
            _ => false,
        }
    }
    shapes.iter().any(|cs| check(&cs.shape))
}

fn make_input(time: f64, events: Vec<Event>) -> RawInput {
    RawInput {
        time: Some(time),
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0))),
        events,
        ..Default::default()
    }
}

fn pointer_events(pos: Pos2, button: PointerButton, pressed: bool) -> Vec<Event> {
    let mut evs = vec![Event::PointerMoved(pos)];
    evs.push(Event::PointerButton {
        pos,
        button,
        pressed,
        modifiers: Modifiers::NONE,
    });
    evs
}

#[test]
fn test_sonoma_segmented_control_custom_widths_and_total_width() {
    let widths = [34.0, 33.0, 33.0, 33.0, 33.0, 33.0, 35.0];
    assert_eq!(widths.iter().sum::<f32>(), 234.0);

    let segments: Vec<SegmentItem> = widths
        .iter()
        .enumerate()
        .map(|(i, &w)| SegmentItem::new(format!("S{i}")).with_width(w).with_submenu(i != 1))
        .collect();

    let ctx = egui::Context::default();
    let theme = MacTheme::default();
    let mut selected = 2;
    let mut output = ctx.run_ui(make_input(0.0, Vec::new()), |ui| {
        let resp = MacSegmentedControl::with_custom_segments(segments.clone(), selected)
            .theme(&theme)
            .height(24.0)
            .show(ui, |idx| selected = idx);
        assert_eq!(resp.rect.width(), 234.0);
        assert_eq!(resp.rect.height(), 24.0);
    });
    assert!(
        !has_convex_polygon(&output.shapes),
        "Segmented control must never emit closed triangle convex_polygon shapes"
    );
    output.textures_delta.clear();
}

#[test]
fn test_segmented_control_vector_icon_items_receive_appkit_state_tints() {
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    setup_macview_fonts(&mut fonts);
    ctx.set_fonts(fonts);

    let theme = MacTheme::default();
    let recorded: Rc<RefCell<Vec<(usize, Rect, Color32)>>> = Rc::new(RefCell::new(Vec::new()));

    let mut output = ctx.run_ui(make_input(0.0, Vec::new()), |ui| {
        let (r0, r1, r2) = (Rc::clone(&recorded), Rc::clone(&recorded), Rc::clone(&recorded));
        let segments = vec![
            SegmentItem::from_icon(move |_p, rect, col| r0.borrow_mut().push((0, rect, col)))
                .with_width(34.0)
                .with_tooltip("Active Tool"),
            SegmentItem::from_icon(move |_p, rect, col| r1.borrow_mut().push((1, rect, col)))
                .with_width(33.0)
                .with_tooltip("Inactive Tool"),
            SegmentItem::from_icon(move |_p, rect, col| r2.borrow_mut().push((2, rect, col)))
                .with_width(35.0)
                .enabled(false)
                .with_tooltip("Disabled Tool"),
            SegmentItem::from_symbol(MacSymbol::Search).with_width(33.0),
        ];
        let _ = MacSegmentedControl::with_custom_segments(segments, 0)
            .theme(&theme)
            .height(24.0)
            .show(ui, |_| {});
    });
    output.textures_delta.clear();

    let calls = recorded.borrow();
    assert_eq!(calls.len(), 3);
    assert!(calls[0].1.width() > 20.0 && calls[0].1.width() < 34.0);
    assert_ne!(calls[0].2, calls[1].2, "Active tint must differ from inactive tint");
    assert_ne!(calls[1].2, calls[2].2, "Disabled tint must differ from inactive tint");
}

#[test]
fn test_segmented_control_orthogonal_menu_vs_shows_menu_indicator() {
    let ctx = egui::Context::default();
    let theme = MacTheme::default();
    let bar_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(80.0, 24.0));

    let segs_hidden = vec![
        SegmentItem::new("Action")
            .with_width(40.0)
            .with_menu(true)
            .with_shows_menu_indicator(false),
        SegmentItem::new("Plain").with_width(40.0).with_menu(false),
    ];
    let mut out_a = ctx.run_ui(make_input(0.0, Vec::new()), |ui| {
        let _ = MacSegmentedControl::with_custom_segments(segs_hidden.clone(), 0)
            .theme(&theme)
            .fixed_rect(bar_rect)
            .show_with_events(ui, |_| {});
    });
    let lines_a = collect_line_segments(&out_a.shapes);
    out_a.textures_delta.clear();

    let segs_visible = vec![
        SegmentItem::new("Action")
            .with_width(40.0)
            .with_menu(true)
            .with_shows_menu_indicator(true),
        SegmentItem::new("Plain").with_width(40.0).with_menu(false),
    ];
    let mut out_b = ctx.run_ui(make_input(0.1, Vec::new()), |ui| {
        let _ = MacSegmentedControl::with_custom_segments(segs_visible.clone(), 0)
            .theme(&theme)
            .fixed_rect(bar_rect)
            .show_with_events(ui, |_| {});
    });
    let lines_b = collect_line_segments(&out_b.shapes);
    out_b.textures_delta.clear();
    assert_eq!(
        lines_b.len(),
        lines_a.len() + 2,
        "shows_menu_indicator(true) must add exactly 2 analytical chevron line segments"
    );

    let mut events: Vec<SegmentEvent> = Vec::new();
    for (idx, pos) in [pos2(15.0, 12.0), pos2(60.0, 12.0)].into_iter().enumerate() {
        let t = 1.0 + idx as f64;
        for pressed in [true, false] {
            let mut o = ctx.run_ui(
                make_input(t + if pressed { 0.0 } else { 0.05 }, pointer_events(pos, PointerButton::Secondary, pressed)),
                |ui| {
                    let _ = MacSegmentedControl::with_custom_segments(segs_hidden.clone(), 0)
                        .theme(&theme)
                        .fixed_rect(bar_rect)
                        .show_with_events(ui, |ev| events.push(ev));
                },
            );
            o.textures_delta.clear();
        }
        assert_eq!(events.len(), 1);
    }
    assert_eq!(events[0].index, 0);
    assert_eq!(events[0].menu_trigger, Some(SegmentMenuTrigger::SecondaryClick));
    assert!(events[0].opens_menu());
}

#[test]
fn test_segmented_control_press_and_hold_and_pull_down_open_menu_with_zero_idle_repaint() {
    let ctx = egui::Context::default();
    let theme = MacTheme::default();
    let segs = vec![
        SegmentItem::new("Draw").with_width(36.0).with_submenu(true),
        SegmentItem::new("Text").with_width(36.0).with_submenu(false),
    ];
    let bar_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(72.0, 24.0));
    let mut events: Vec<SegmentEvent> = Vec::new();

    let run_step = |time: f64, evs: Vec<Event>, out_events: &mut Vec<SegmentEvent>| -> Duration {
        let mut out = ctx.run_ui(make_input(time, evs), |ui| {
            let _ = MacSegmentedControl::with_custom_segments(segs.clone(), 0)
                .theme(&theme)
                .fixed_rect(bar_rect)
                .show_with_events(ui, |ev| out_events.push(ev));
        });
        out.textures_delta.clear();
        out.viewport_output[&egui::ViewportId::ROOT].repaint_delay
    };

    // 1. Press at t=1.0s on menu segment 0 -> schedules wake-up for hold threshold, no event yet
    let d1 = run_step(1.0, pointer_events(pos2(16.0, 12.0), PointerButton::Primary, true), &mut events);
    assert!(events.is_empty());
    assert!(d1 < Duration::from_secs(1), "Holding menu segment must schedule wake-up before 250ms");

    // 2. Held at t=1.30s (>= 250ms) -> fires PressAndHold once
    let _ = run_step(1.30, Vec::new(), &mut events);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].menu_trigger, Some(SegmentMenuTrigger::PressAndHold));

    // 3. Release at t=1.35s -> no duplicate selection event
    let _ = run_step(1.35, pointer_events(pos2(16.0, 12.0), PointerButton::Primary, false), &mut events);
    assert_eq!(events.len(), 1, "Release after PressAndHold must not fire a duplicate selection");

    // 4. Downward pull >= 4pt immediately fires PullDown
    events.clear();
    let _ = run_step(2.0, pointer_events(pos2(16.0, 8.0), PointerButton::Primary, true), &mut events);
    let _ = run_step(2.03, vec![Event::PointerMoved(pos2(16.0, 14.0))], &mut events);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].menu_trigger, Some(SegmentMenuTrigger::PullDown));
    let _ = run_step(2.06, pointer_events(pos2(16.0, 14.0), PointerButton::Primary, false), &mut events);

    // 5. Let spring settle and verify idle repaint_delay == Duration::MAX
    for t in [3.0, 4.0, 5.0] {
        let _ = run_step(t, Vec::new(), &mut events);
    }
    let idle_delay = run_step(6.0, Vec::new(), &mut events);
    assert_eq!(idle_delay, Duration::MAX, "Idle segmented control must settle to Duration::MAX");
}

#[test]
fn test_segmented_control_per_segment_enabled_and_keyboard_skip() {
    let ctx = egui::Context::default();
    let theme = MacTheme::default();
    let segs = vec![
        SegmentItem::new("First").with_width(40.0),
        SegmentItem::new("Disabled").with_width(40.0).enabled(false),
        SegmentItem::new("Third").with_width(40.0),
    ];
    let bar_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(120.0, 24.0));
    let mut selected = 0usize;

    let step = |time: f64, evs: Vec<Event>, sel: &mut usize| {
        let mut out = ctx.run_ui(make_input(time, evs), |ui| {
            let _ = MacSegmentedControl::with_custom_segments(segs.clone(), *sel)
                .theme(&theme)
                .fixed_rect(bar_rect)
                .show(ui, |idx| *sel = idx);
        });
        out.textures_delta.clear();
    };

    // Click disabled segment 1 -> ignored
    step(1.0, pointer_events(pos2(60.0, 12.0), PointerButton::Primary, true), &mut selected);
    step(1.05, pointer_events(pos2(60.0, 12.0), PointerButton::Primary, false), &mut selected);
    assert_eq!(selected, 0);

    // Click segment 0 to focus, then press ArrowRight -> skips disabled segment 1 and selects 2
    step(2.0, pointer_events(pos2(20.0, 12.0), PointerButton::Primary, true), &mut selected);
    step(2.05, pointer_events(pos2(20.0, 12.0), PointerButton::Primary, false), &mut selected);
    step(
        2.10,
        vec![Event::Key {
            key: Key::ArrowRight,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        &mut selected,
    );
    assert_eq!(selected, 2, "ArrowRight must skip disabled segment 1 and select segment 2");
}

#[test]
fn test_segmented_control_sonoma_divider_suppression_adjacent_to_active_pill() {
    let ctx = egui::Context::default();
    let theme = MacTheme::default();
    let bar_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(120.0, 24.0));

    let mut out = ctx.run_ui(make_input(0.0, Vec::new()), |ui| {
        let segs = vec![
            SegmentItem::new("A").with_width(30.0),
            SegmentItem::new("B").with_width(30.0),
            SegmentItem::new("C").with_width(30.0),
            SegmentItem::new("D").with_width(30.0),
        ];
        let _ = MacSegmentedControl::with_custom_segments(segs, 1)
            .theme(&theme)
            .fixed_rect(bar_rect)
            .show(ui, |_| {});
    });
    let vertical_dividers: Vec<f32> = collect_line_segments(&out.shapes)
        .into_iter()
        .filter(|[a, b]| (a.x - b.x).abs() < 0.01 && (b.y - a.y).abs() > 8.0)
        .map(|[a, _]| a.x)
        .collect();
    out.textures_delta.clear();

    assert_eq!(
        vertical_dividers,
        vec![90.0],
        "Sonoma rounded segmented control must suppress dividers at x=30 and x=60 adjacent to active pill 1, drawing only x=90"
    );
}
