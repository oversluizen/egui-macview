//! ISO 29119-4 / macOS AppKit Specification Tests for State Integrity and Feedback.
//!
//! Normative Reference: SPEC-008 (State Integrity, Context Menus, and Animated Feedback).
//! Grounded in `NSUndoManager` bounded ringbuffer semantics, boundary-clamped context menus,
//! and `CASpring`-animated toast notifications.

use egui::{Context, Event, Key, Modifiers, Pos2, RawInput, Rect, Vec2};
use egui_macview::{
    clamp_menu_rect,
    motion::CASpring,
    theme::{MacAppearance, MacTheme},
    MacContextMenu, MacToast, MacToastManager, NSMenuItem, NSUndoManager, ToastLevel,
    DEFAULT_UNDO_CAPACITY,
};
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum UndoOp {
    Mutate(i32),
    Undo,
    Redo,
}

fn undo_op_strategy() -> impl Strategy<Value = UndoOp> {
    prop_oneof![
        any::<i32>().prop_map(UndoOp::Mutate),
        Just(UndoOp::Undo),
        Just(UndoOp::Redo),
    ]
}

proptest! {
    /// Invariant 1 (ISO 29119-4 / AppKit NSUndoManager):
    /// Bounded ringbuffer cyclic preservation and capacity limits.
    /// Under any random sequence of mutations, undos, and redos:
    /// - `undo_count() <= capacity` strictly holds at every step.
    /// - `redo_count() <= capacity` strictly holds at every step.
    /// - Push snapshot clears the redo stack.
    /// - Undo then Redo restores the exact mutated state.
    #[test]
    fn prop_undo_manager_cyclic_preservation(
        initial in any::<i32>(),
        capacity in 5usize..30usize,
        ops in prop::collection::vec(undo_op_strategy(), 1..60),
    ) {
        let mut manager = NSUndoManager::<i32>::with_capacity(capacity);
        let mut current_state = initial;

        for op in ops {
            match op {
                UndoOp::Mutate(val) => {
                    let prev_state = current_state;
                    manager.push_snapshot(current_state);
                    current_state = val;

                    prop_assert_eq!(current_state, val);
                    prop_assert_eq!(manager.redo_count(), 0, "Mutate must clear redo stack");
                    prop_assert!(manager.undo_count() <= capacity, "Undo count must not exceed capacity");

                    // Immediate undo must restore previous state
                    if manager.can_undo() {
                        manager.undo(&mut current_state);
                        prop_assert_eq!(current_state, prev_state);
                        manager.redo(&mut current_state);
                        prop_assert_eq!(current_state, val);
                    }
                }
                UndoOp::Undo => {
                    if manager.can_undo() {
                        let pre_undo = current_state;
                        let undo_count_before = manager.undo_count();
                        let redo_count_before = manager.redo_count();

                        manager.undo(&mut current_state);

                        prop_assert_eq!(manager.undo_count(), undo_count_before - 1);
                        prop_assert_eq!(manager.redo_count(), redo_count_before + 1);

                        // Redo should return strictly to pre_undo
                        manager.redo(&mut current_state);
                        prop_assert_eq!(current_state, pre_undo);
                        manager.undo(&mut current_state); // back to undone state
                    }
                }
                UndoOp::Redo => {
                    if manager.can_redo() {
                        let redo_count_before = manager.redo_count();
                        let undo_count_before = manager.undo_count();

                        manager.redo(&mut current_state);

                        prop_assert_eq!(manager.redo_count(), redo_count_before - 1);
                        prop_assert_eq!(manager.undo_count(), undo_count_before + 1);
                    }
                }
            }

            prop_assert!(manager.undo_count() <= capacity);
            prop_assert!(manager.redo_count() <= capacity);
        }
    }

    /// Invariant 2 (Deterministic Rollback to Initial State):
    /// Performing K mutations (K <= capacity) followed by K consecutive undos
    /// MUST return strictly to the original initial state.
    #[test]
    fn prop_undo_manager_full_rollback_exactness(
        initial in any::<i64>(),
        mutations in prop::collection::vec(any::<i64>(), 1..25),
    ) {
        let cap = 50;
        let mut manager = NSUndoManager::<i64>::with_capacity(cap);
        let mut current_state = initial;

        for &m in &mutations {
            manager.push_snapshot(current_state);
            current_state = m;
        }

        prop_assert_eq!(manager.undo_count(), mutations.len());

        for _ in 0..mutations.len() {
            prop_assert!(manager.can_undo());
            manager.undo(&mut current_state);
        }

        prop_assert_eq!(current_state, initial, "Full rollback must restore exact initial state");
        prop_assert!(!manager.can_undo());
        prop_assert_eq!(manager.redo_count(), mutations.len());
    }
}

#[test]
fn test_undo_manager_default_capacity_and_clear() {
    let mut manager = NSUndoManager::<&str>::new();
    let mut current_state = "initial_state";

    assert_eq!(manager.capacity(), DEFAULT_UNDO_CAPACITY);
    assert_eq!(current_state, "initial_state");
    assert!(!manager.can_undo());
    assert!(!manager.can_redo());

    manager.push_snapshot(current_state);
    current_state = "state_1";

    manager.push_snapshot(current_state);
    current_state = "state_2";

    assert_eq!(manager.undo_count(), 2);
    assert!(manager.can_undo());

    let restored = manager.undo(&mut current_state);
    assert!(restored);
    assert_eq!(current_state, "state_1");
    assert!(manager.can_redo());

    let redone = manager.redo(&mut current_state);
    assert!(redone);
    assert_eq!(current_state, "state_2");

    manager.clear();
    assert_eq!(manager.undo_count(), 0);
    assert_eq!(manager.redo_count(), 0);
    assert!(!manager.can_undo());
    assert!(!manager.can_redo());
}

#[test]
fn test_context_menu_boundary_clamping() {
    let viewport = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(1000.0, 800.0));
    let menu_size = Vec2::new(160.0, 200.0);

    // 1. Spawning inside comfortable bounds: min equals spawn pos
    let pos_in = Pos2::new(100.0, 100.0);
    let clamped_in = clamp_menu_rect(pos_in, menu_size, viewport);
    assert_eq!(clamped_in.min, pos_in);
    assert_eq!(clamped_in.size(), menu_size);

    // 2. Spawning too close to right edge: shifts left to fit inside viewport
    let pos_right = Pos2::new(950.0, 100.0);
    let clamped_right = clamp_menu_rect(pos_right, menu_size, viewport);
    assert!(clamped_right.max.x <= viewport.max.x);
    assert_eq!(clamped_right.max.x, viewport.max.x);
    assert_eq!(clamped_right.size(), menu_size);

    // 3. Spawning too close to bottom edge: shifts up to fit inside viewport
    let pos_bottom = Pos2::new(200.0, 750.0);
    let clamped_bottom = clamp_menu_rect(pos_bottom, menu_size, viewport);
    assert!(clamped_bottom.max.y <= viewport.max.y);
    assert_eq!(clamped_bottom.max.y, viewport.max.y);
    assert_eq!(clamped_bottom.size(), menu_size);

    // 4. Spawning at bottom-right corner: clamped on both axes
    let pos_corner = Pos2::new(980.0, 790.0);
    let clamped_corner = clamp_menu_rect(pos_corner, menu_size, viewport);
    assert_eq!(clamped_corner.max.x, viewport.max.x);
    assert_eq!(clamped_corner.max.y, viewport.max.y);
    assert_eq!(clamped_corner.size(), menu_size);
}

#[test]
fn test_context_menu_keyboard_and_action() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let items = vec![
        NSMenuItem::action("Inspect Element", "action.inspect"),
        NSMenuItem::separator(),
        NSMenuItem::action("Delete", "action.delete"),
    ];

    let mut is_open = true;
    let mut selected_idx = Some(0);
    let mut spawn_pos = Pos2::new(100.0, 100.0);

    // Arrow down skips separator from index 0 to 2
    let mut raw_down = RawInput::default();
    raw_down.events.push(Event::Key {
        key: Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });

    let mut out1 = ctx.run_ui(raw_down, |ui| {
        let menu = MacContextMenu::new(&items, &mut is_open, &mut spawn_pos, &mut selected_idx)
            .theme(&theme);

        menu.show(ui);
    });
    out1.textures_delta.clear();

    assert_eq!(selected_idx, Some(2));

    // Enter triggers highlighted action ("action.delete")
    let mut raw_enter = RawInput::default();
    raw_enter.events.push(Event::Key {
        key: Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });

    let mut triggered = None;
    let mut out2 = ctx.run_ui(raw_enter, |ui| {
        let menu = MacContextMenu::new(&items, &mut is_open, &mut spawn_pos, &mut selected_idx)
            .theme(&theme);

        triggered = menu.show(ui);
    });
    out2.textures_delta.clear();

    assert_eq!(triggered, Some("action.delete".to_string()));
}

#[test]
fn test_toast_timing_and_spring_settling() {
    let mut toast = MacToast::new("Item saved to cloud")
        .level(ToastLevel::Success)
        .duration(2.5);

    // Initial state: elapsed 0.0, active
    assert_eq!(toast.elapsed, 0.0);
    assert!(!toast.is_dismissed);

    // Update 1.0s: still active
    toast.update(1.0);
    assert_eq!(toast.elapsed, 1.0);
    assert!(!toast.is_dismissed);

    // Update 1.4s: elapsed 2.4s, still active (< 2.5s)
    toast.update(1.4);
    assert_eq!(toast.elapsed, 2.4);
    assert!(!toast.is_dismissed);

    // Update 0.2s: elapsed 2.6s, marked as dismissed
    toast.update(0.2);
    assert!(toast.elapsed >= 2.5);
    assert!(toast.is_dismissed);

    // Spring physics animation bounds
    let spring = CASpring::new(0.35, 0.85);
    assert_eq!(spring.evaluate(0.0), 0.0);
    let settling_t = spring.settling_duration();
    assert!(settling_t > 0.0 && settling_t < 2.0);

    // Manager tests
    let mut manager = MacToastManager::new();
    manager.success("Saved successfully");
    manager.error("Network error");
    assert_eq!(manager.active_count(), 2);

    // Advance by 3.0s: both toasts auto-dismissed and pruned
    manager.update(3.0);
    assert_eq!(manager.active_count(), 0);
}

#[test]
fn test_undo_manager_peek_inspection() {
    let mut manager = NSUndoManager::<String>::new();
    assert_eq!(manager.peek_undo(), None);
    assert_eq!(manager.peek_redo(), None);

    let mut state = "State 0".to_string();
    manager.push_snapshot(state.clone());
    state = "State 1".to_string();

    assert_eq!(manager.peek_undo(), Some(&"State 0".to_string()));
    assert_eq!(manager.peek_redo(), None);

    manager.undo(&mut state);
    assert_eq!(state, "State 0");
    assert_eq!(manager.peek_undo(), None);
    assert_eq!(manager.peek_redo(), Some(&"State 1".to_string()));
}

#[test]
fn test_context_menu_attach_to_and_oversized_clamping() {
    let ctx = Context::default();

    // 1. Oversized menu clamping: menu larger than viewport
    let tiny_viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0));
    let huge_menu = Vec2::new(200.0, 300.0);
    let clamped = clamp_menu_rect(Pos2::new(50.0, 50.0), huge_menu, tiny_viewport);
    assert!(clamped.max.x <= tiny_viewport.max.x);
    assert!(clamped.max.y <= tiny_viewport.max.y);
    assert!(clamped.min.x >= tiny_viewport.min.x);
    assert!(clamped.min.y >= tiny_viewport.min.y);

    // 2. Attach to widget response
    let items = vec![NSMenuItem::action("Delete", "action.delete")];
    let mut is_open = false;
    let mut spawn_pos = Pos2::ZERO;
    let mut selected_idx = None;

    let mut menu = MacContextMenu::new(&items, &mut is_open, &mut spawn_pos, &mut selected_idx);

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        // Allocate interactive element (e.g. card or outline view row)
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(100.0, 40.0), egui::Sense::click());
        menu.attach_to(&resp);
        let _ = rect;
    });
    out.textures_delta.clear();

    // Without secondary click, remains closed
    assert!(!is_open);
}

#[test]
fn test_context_menu_arrow_down_when_unselected() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let items = vec![
        NSMenuItem::action("First", "action.first"),
        NSMenuItem::action("Second", "action.second"),
        NSMenuItem::action("Third", "action.third"),
    ];

    let mut is_open = true;
    let mut selected_idx = None; // Unselected initial state
    let mut spawn_pos = Pos2::new(100.0, 100.0);

    let mut raw_down = RawInput::default();
    raw_down.events.push(Event::Key {
        key: Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });

    let mut out = ctx.run_ui(raw_down, |ui| {
        let menu = MacContextMenu::new(&items, &mut is_open, &mut spawn_pos, &mut selected_idx)
            .theme(&theme);
        menu.show(ui);
    });
    out.textures_delta.clear();

    assert_eq!(
        selected_idx,
        Some(0),
        "ArrowDown when unselected must highlight item 0, not skip it"
    );
}

#[test]
fn test_context_menu_secondary_click_trigger() {
    let ctx = Context::default();

    let items = vec![NSMenuItem::action("Inspect", "action.inspect")];
    let mut is_open = false;
    let mut spawn_pos = Pos2::ZERO;
    let mut selected_idx = None;

    let click_pos = Pos2::new(60.0, 30.0);

    // Frame 1: Register widget in layout & move pointer over it
    let mut raw_init = RawInput::default();
    raw_init.events.push(Event::PointerMoved(click_pos));
    let mut out1 = ctx.run_ui(raw_init, |ui| {
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(120.0, 60.0), egui::Sense::click());
        let _ = rect;
    });
    out1.textures_delta.clear();

    // Frame 2: Press secondary button down on registered widget
    let mut raw_down = RawInput::default();
    raw_down.events.push(Event::PointerButton {
        pos: click_pos,
        button: egui::PointerButton::Secondary,
        pressed: true,
        modifiers: Modifiers::default(),
    });
    let mut out2 = ctx.run_ui(raw_down, |ui| {
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(120.0, 60.0), egui::Sense::click());
        let _ = rect;
    });
    out2.textures_delta.clear();

    // Frame 3: Release secondary button -> fires secondary_clicked()
    let mut raw_up = RawInput::default();
    raw_up.events.push(Event::PointerButton {
        pos: click_pos,
        button: egui::PointerButton::Secondary,
        pressed: false,
        modifiers: Modifiers::default(),
    });

    let mut out3 = ctx.run_ui(raw_up, |ui| {
        let mut menu = MacContextMenu::new(&items, &mut is_open, &mut spawn_pos, &mut selected_idx);
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(120.0, 60.0), egui::Sense::click());
        menu.attach_to(&resp);
        let _ = rect;
    });
    out3.textures_delta.clear();

    assert!(
        is_open,
        "Secondary click (right-click) on interactive widget must trigger context menu"
    );
    assert_eq!(
        spawn_pos, click_pos,
        "Context menu spawn position must match pointer position"
    );
}
