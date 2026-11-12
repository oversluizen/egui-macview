//! Accessibility (`AccessKit` / WCAG 4.1.2), Keyboard Menu/Multi Parity, Pressed Highlight & Stack-Inline COTS Tests.
//!
//! Verifies Tier-1 industry standards for `MacSegmentedControl` (`SPEC-002` §2.3):
//! 1. `egui::WidgetInfo` (`Role::RadioGroup` & `Role::Toolbar`, `.accessibility_label(...)`, active segment value).
//! 2. Full keyboard parity: `ArrowDown` / `Shift+F10` emits `SegmentMenuTrigger::Keyboard` on menu segments,
//!    and `show_multi` supports focus ring + `ArrowLeft`/`ArrowRight` (skipping disabled) + `Space`/`Enter` toggle.
//! 3. `NSSegmentedCell.isHighlighted` visual pressed state on inactive segments with drag-out cancellation.
//! 4. Stack-inline storage (`INLINE_SEGMENT_CAPACITY == 8`) accepting `[SegmentItem; N]` arrays with zero heap `Vec` allocation.

use egui::{
    output::OutputEvent, pos2, Color32, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect,
    Role, Shape, Vec2,
};
use egui_macview::{MacSegmentedControl, MacTheme, SegmentEvent, SegmentItem, SegmentMenuTrigger};
use std::cell::RefCell;
use std::rc::Rc;

fn make_input(time: f64, events: Vec<Event>) -> RawInput {
    RawInput {
        time: Some(time),
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0))),
        events,
        ..Default::default()
    }
}

fn count_rect_shapes(shapes: &[egui::epaint::ClippedShape]) -> usize {
    fn visit(s: &Shape) -> usize {
        match s {
            Shape::Rect(_) => 1,
            Shape::Vec(v) => v.iter().map(visit).sum(),
            _ => 0,
        }
    }
    shapes.iter().map(|cs| visit(&cs.shape)).sum()
}

#[test]
fn test_segmented_accesskit_role_and_screen_reader_metadata() {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let theme = MacTheme::default();
    let bar_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(80.0, 24.0));
    let mut selected = 0usize;

    // Warm frame 0
    let mut o0 = ctx.run_ui(make_input(0.0, Vec::new()), |ui| {
        let _ = MacSegmentedControl::new(&["Select", "Pen"], selected)
            .accessibility_label("Tools")
            .theme(&theme)
            .fixed_rect(bar_rect)
            .show(ui, |i| selected = i);
    });
    o0.textures_delta.clear();
    assert!(
        o0.platform_output.accesskit_update.is_some(),
        "AccessKit tree update must be emitted when enabled"
    );

    // Click segment 1 ("Pen") at x = 60 -> emits ValueChanged / Clicked with Role::RadioGroup, label "Tools", text_value "Pen"
    let mut o1 = ctx.run_ui(
        make_input(
            0.1,
            vec![
                Event::PointerMoved(pos2(60.0, 12.0)),
                Event::PointerButton {
                    pos: pos2(60.0, 12.0),
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        ),
        |ui| {
            let _ = MacSegmentedControl::new(&["Select", "Pen"], selected)
                .accessibility_label("Tools")
                .theme(&theme)
                .fixed_rect(bar_rect)
                .show(ui, |i| selected = i);
        },
    );
    o1.textures_delta.clear();

    let mut o2 = ctx.run_ui(
        make_input(
            0.15,
            vec![Event::PointerButton {
                pos: pos2(60.0, 12.0),
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        ),
        |ui| {
            let _ = MacSegmentedControl::new(&["Select", "Pen"], selected)
                .accessibility_label("Tools")
                .theme(&theme)
                .fixed_rect(bar_rect)
                .show(ui, |i| selected = i);
        },
    );
    o2.textures_delta.clear();
    assert_eq!(selected, 1);

    let radio_info = o2
        .platform_output
        .events
        .iter()
        .find_map(|ev| match ev {
            OutputEvent::Clicked(info) | OutputEvent::ValueChanged(info) => Some(info),
            _ => None,
        })
        .expect("Segment selection must emit an accessibility OutputEvent");
    assert_eq!(radio_info.role, Role::RadioGroup);
    assert_eq!(radio_info.label.as_deref(), Some("Tools"));
    assert_eq!(radio_info.current_text_value.as_deref(), Some("Pen"));
}

#[test]
fn test_segmented_keyboard_menu_trigger_and_multi_select_keyboard_parity() {
    let ctx = egui::Context::default();
    let theme = MacTheme::default();
    let bar_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(80.0, 24.0));
    let mut events: Vec<SegmentEvent> = Vec::new();
    let mut selected = 0usize;

    let run_single = |time: f64, evs: Vec<Event>, sel: &mut usize, out_ev: &mut Vec<SegmentEvent>| {
        let mut out = ctx.run_ui(make_input(time, evs), |ui| {
            let items = [
                SegmentItem::new("Draw").with_width(40.0).with_submenu(true),
                SegmentItem::new("Text").with_width(40.0).with_submenu(false),
            ];
            let _ = MacSegmentedControl::with_custom_segments(items, *sel)
                .theme(&theme)
                .fixed_rect(bar_rect)
                .show_with_events(ui, |ev| {
                    if !ev.opens_menu() {
                        *sel = ev.index;
                    }
                    out_ev.push(ev);
                });
        });
        out.textures_delta.clear();
    };

    // Click segment 0 to focus
    run_single(
        0.1,
        vec![
            Event::PointerMoved(pos2(20.0, 12.0)),
            Event::PointerButton { pos: pos2(20.0, 12.0), button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
        ],
        &mut selected,
        &mut events,
    );
    run_single(
        0.15,
        vec![Event::PointerButton { pos: pos2(20.0, 12.0), button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }],
        &mut selected,
        &mut events,
    );
    events.clear();

    // Press ArrowDown on focused menu segment 0 -> emits SegmentMenuTrigger::Keyboard
    run_single(
        0.2,
        vec![Event::Key { key: Key::ArrowDown, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE }],
        &mut selected,
        &mut events,
    );
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].index, 0);
    assert_eq!(events[0].menu_trigger, Some(SegmentMenuTrigger::Keyboard));

    // Move ArrowRight to segment 1 (has_menu == false), press ArrowDown -> must NOT emit menu event
    events.clear();
    run_single(
        0.3,
        vec![Event::Key { key: Key::ArrowRight, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE }],
        &mut selected,
        &mut events,
    );
    assert_eq!(selected, 1);
    events.clear();
    run_single(
        0.4,
        vec![Event::Key { key: Key::ArrowDown, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE }],
        &mut selected,
        &mut events,
    );
    assert!(events.is_empty(), "ArrowDown on non-menu segment must not emit a menu trigger");

    // Multi-select keyboard navigation: click segment 0 to focus, ArrowRight skips disabled 1 -> lands on 2, Space toggles 2
    let ctx_m = egui::Context::default();
    let multi_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(120.0, 22.0));
    let mut active = [false, false, false];
    let mut last_toggled = None;
    let run_multi = |time: f64, evs: Vec<Event>, act: &mut [bool; 3], toggled: &mut Option<usize>| {
        let mut out = ctx_m.run_ui(make_input(time, evs), |ui| {
            let items = [
                SegmentItem::new("Bold").with_width(40.0),
                SegmentItem::new("Locked").with_width(40.0).enabled(false),
                SegmentItem::new("Underline").with_width(40.0),
            ];
            *toggled = MacSegmentedControl::with_custom_segments(items, 0)
                .small_square()
                .accessibility_label("Text Styles")
                .theme(&theme)
                .fixed_rect(multi_rect)
                .show_multi(ui, act);
        });
        out.textures_delta.clear();
    };

    run_multi(0.0, Vec::new(), &mut active, &mut last_toggled);
    run_multi(
        0.1,
        vec![
            Event::PointerMoved(pos2(20.0, 11.0)),
            Event::PointerButton { pos: pos2(20.0, 11.0), button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
            Event::PointerButton { pos: pos2(20.0, 11.0), button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE },
        ],
        &mut active,
        &mut last_toggled,
    );
    assert_eq!(active, [true, false, false]);

    run_multi(
        0.2,
        vec![Event::Key { key: Key::ArrowRight, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE }],
        &mut active,
        &mut last_toggled,
    );
    run_multi(
        0.3,
        vec![Event::Key { key: Key::Space, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE }],
        &mut active,
        &mut last_toggled,
    );
    assert_eq!(last_toggled, Some(2));
    assert_eq!(active, [true, false, true], "ArrowRight + Space in show_multi must skip disabled 1 and toggle segment 2");
}

#[test]
fn test_segmented_pressed_highlight_and_drag_out_cancellation() {
    let ctx = egui::Context::default();
    let theme = MacTheme::default();
    let bar_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(80.0, 24.0));
    let seg1_tint: Rc<RefCell<Color32>> = Rc::new(RefCell::new(Color32::TRANSPARENT));
    let mut selected = 0usize;

    let run_frame = |time: f64, evs: Vec<Event>, sel: &mut usize| -> (usize, Color32) {
        let mut out = ctx.run_ui(make_input(time, evs), |ui| {
            let t_ref = Rc::clone(&seg1_tint);
            let items = [
                SegmentItem::new("One").with_width(40.0),
                SegmentItem::from_icon(move |_p, _r, col| *t_ref.borrow_mut() = col).with_width(40.0),
            ];
            let _ = MacSegmentedControl::with_custom_segments(items, *sel)
                .theme(&theme)
                .fixed_rect(bar_rect)
                .show(ui, |i| *sel = i);
        });
        let rects = count_rect_shapes(&out.shapes);
        out.textures_delta.clear();
        (rects, *seg1_tint.borrow())
    };

    let (rects_idle, tint_idle) = run_frame(0.0, Vec::new(), &mut selected);

    // Press down on inactive segment 1 at (60, 12) -> draws depressed highlight rect and primary tint
    let (rects_pressed, tint_pressed) = run_frame(
        0.1,
        vec![
            Event::PointerMoved(pos2(60.0, 12.0)),
            Event::PointerButton { pos: pos2(60.0, 12.0), button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
        ],
        &mut selected,
    );
    assert!(rects_pressed > rects_idle, "Pressing inactive segment must render isHighlighted backdrop rect");
    assert_ne!(tint_pressed, tint_idle, "Pressing inactive segment must elevate icon/label tint");

    // Drag outside segment 1 to (60, 80) while still holding primary -> cancels highlight visually (focus ring remains)
    let (rects_outside, tint_outside) = run_frame(0.15, vec![Event::PointerMoved(pos2(60.0, 80.0))], &mut selected);
    assert_eq!(
        rects_outside + 1,
        rects_pressed,
        "Dragging outside segment must cancel the depressed highlight rect"
    );
    assert_eq!(tint_outside, tint_idle, "Dragging outside segment must restore idle tint");

    // Release outside -> must NOT select segment 1
    let _ = run_frame(
        0.2,
        vec![Event::PointerButton { pos: pos2(60.0, 80.0), button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }],
        &mut selected,
    );
    assert_eq!(selected, 0, "Releasing outside pressed segment must cancel selection");
}

#[test]
fn test_segmented_stack_inline_array_zero_vec_api_and_large_fallback() {
    assert_eq!(MacSegmentedControl::INLINE_SEGMENT_CAPACITY, 8);

    let inline_ctrl = MacSegmentedControl::with_custom_segments(
        [SegmentItem::new("A"), SegmentItem::new("B"), SegmentItem::new("C")],
        1,
    );
    assert!(inline_ctrl.is_inline_allocated(), "<= 8 segments must reside inline on the stack");

    let heap_items: Vec<SegmentItem> = (0..10).map(|i| SegmentItem::new(format!("S{i}")).with_width(24.0)).collect();
    let heap_ctrl = MacSegmentedControl::with_custom_segments(heap_items, 9);
    assert!(!heap_ctrl.is_inline_allocated(), "> 8 segments must spill cleanly to heap storage");

    let ctx = egui::Context::default();
    let theme = MacTheme::default();
    let mut out = ctx.run_ui(make_input(0.0, Vec::new()), |ui| {
        let items: Vec<SegmentItem> = (0..10).map(|i| SegmentItem::new(format!("S{i}")).with_width(24.0)).collect();
        let resp = MacSegmentedControl::with_custom_segments(items, 9).theme(&theme).show(ui, |_| {});
        assert_eq!(resp.rect.width(), 240.0);
    });
    out.textures_delta.clear();
}

#[test]
fn test_segmented_momentary_tracking_and_id_salt_stability() {
    let ctx = egui::Context::default();
    let theme = MacTheme::default();
    let bar_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(80.0, 24.0));
    let actions: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(Vec::new()));
    let menus: Rc<RefCell<Vec<(usize, SegmentMenuTrigger)>>> = Rc::new(RefCell::new(Vec::new()));

    // 1. Verify .id_salt(...) keeps widget Id identical even when conditional peer widgets appear
    let mut id_without_peer = egui::Id::NULL;
    let mut id_with_peer = egui::Id::NULL;
    let mut o1 = ctx.run_ui(make_input(0.0, Vec::new()), |ui| {
        id_without_peer = MacSegmentedControl::new(&["Back", "Forward"], 0)
            .id_salt("history_nav")
            .theme(&theme)
            .show(ui, |_| {})
            .id;
    });
    o1.textures_delta.clear();
    let mut o2 = ctx.run_ui(make_input(0.02, Vec::new()), |ui| {
        ui.label("Dynamic Leading Status");
        id_with_peer = MacSegmentedControl::new(&["Back", "Forward"], 0)
            .id_salt("history_nav")
            .theme(&theme)
            .show(ui, |_| {})
            .id;
    });
    o2.textures_delta.clear();
    assert_eq!(id_without_peer, id_with_peer, "id_salt must stabilize widget Id across conditional peers");

    // 2. Verify momentary tracking: 1 rect at rest (track only, NO active pill), 2 rects while pressed, fires on release & returns to 1 rect
    let run_momentary = |time: f64, evs: Vec<Event>| -> usize {
        let act_ref = Rc::clone(&actions);
        let menu_ref = Rc::clone(&menus);
        let mut out = ctx.run_ui(make_input(time, evs), |ui| {
            let items = [
                SegmentItem::new("<").with_width(40.0).with_menu(true),
                SegmentItem::new(">").with_width(40.0),
            ];
            let _ = MacSegmentedControl::momentary(items)
                .id_salt("momentary_nav")
                .accessibility_label("History Navigation")
                .theme(&theme)
                .fixed_rect(bar_rect)
                .show_momentary_with_events(ui, |ev| {
                    if let Some(trig) = ev.menu_trigger {
                        menu_ref.borrow_mut().push((ev.index, trig));
                    } else {
                        act_ref.borrow_mut().push(ev.index);
                    }
                });
        });
        let rects = count_rect_shapes(&out.shapes);
        out.textures_delta.clear();
        rects
    };

    let rects_idle = run_momentary(1.0, Vec::new());
    assert_eq!(rects_idle, 1, "Momentary segmented control must paint only the track container at rest (no active pill)");

    let rects_down = run_momentary(
        1.1,
        vec![
            Event::PointerMoved(pos2(60.0, 12.0)),
            Event::PointerButton { pos: pos2(60.0, 12.0), button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
        ],
    );
    assert!(rects_down > rects_idle, "Momentary press must draw depressed highlight");
    assert!(actions.borrow().is_empty(), "Momentary action must not fire before release");

    let rects_up = run_momentary(
        1.15,
        vec![Event::PointerButton { pos: pos2(60.0, 12.0), button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }],
    );
    assert_eq!(*actions.borrow(), vec![1], "Momentary release inside segment 1 must fire action(1)");
    // Focus ring (1) + track (1) = 2 rects, strictly fewer than rects_down (focus ring + track + highlight = 3)
    assert_eq!(rects_up + 1, rects_down, "Momentary highlight must disappear immediately upon release");

    // Press-and-hold on momentary segment 0 (>= 0.25s) must open history menu without firing click action
    actions.borrow_mut().clear();
    let _ = run_momentary(
        2.0,
        vec![
            Event::PointerMoved(pos2(20.0, 12.0)),
            Event::PointerButton { pos: pos2(20.0, 12.0), button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
        ],
    );
    let _ = run_momentary(2.30, Vec::new());
    let _ = run_momentary(
        2.35,
        vec![Event::PointerButton { pos: pos2(20.0, 12.0), button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }],
    );
    assert_eq!(*menus.borrow(), vec![(0, SegmentMenuTrigger::PressAndHold)]);
    assert!(actions.borrow().is_empty(), "Press-and-hold menu must suppress momentary release click");
}
