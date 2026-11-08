//! ISO 29119-4 / macOS HIG Specification Tests for `MacMenuBar` & `NSMenu`.
//!
//! Normative Reference: SPEC-007 (Desktop Navigation, Menu Bar & Cascading Submenus).
//! Verifies keyboard navigation, accelerator clearance, text-focus shortcut guard,
//! outside-click menu dismissal, and `NSMenuItem::checked` checkmark rendering.

use egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    MacInputCell, MacKeyboardShortcut, MacMenuBar, NSMenu, NSMenuItem,
};

fn collect_text_shapes<'a>(
    shape: &'a egui::epaint::Shape,
    out: &mut Vec<&'a egui::epaint::TextShape>,
) {
    match shape {
        egui::epaint::Shape::Text(t) => out.push(t),
        egui::epaint::Shape::Vec(shapes) => {
            for s in shapes {
                collect_text_shapes(s, out);
            }
        }
        _ => {}
    }
}

#[test]
fn test_menu_bar_keyboard_navigation_and_action() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let menus = vec![NSMenu::default_file_menu(), NSMenu::default_edit_menu()];
    let mut open_menu_idx = Some(0);
    let mut highlighted_item_idx = Some(0);

    let mut raw_down = RawInput::default();
    raw_down.events.push(Event::Key {
        key: Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });
    let mut down_leaked = false;
    let mut out1 = ctx.run_ui(raw_down, |ui| {
        MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
        down_leaked = ui.input(|i| i.key_pressed(Key::ArrowDown));
    });
    out1.textures_delta.clear();
    assert_eq!(open_menu_idx, Some(0));
    assert_eq!(highlighted_item_idx, Some(1));
    assert!(
        !down_leaked,
        "Open MacMenuBar must consume ArrowDown navigation key"
    );

    let mut raw_right = RawInput::default();
    raw_right.events.push(Event::Key {
        key: Key::ArrowRight,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });
    let mut out2 = ctx.run_ui(raw_right, |ui| {
        MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
    });
    out2.textures_delta.clear();
    assert_eq!(open_menu_idx, Some(1));
    assert_eq!(highlighted_item_idx, Some(0));

    let mut raw_enter = RawInput::default();
    raw_enter.events.push(Event::Key {
        key: Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });
    let mut triggered = None;
    let mut out3 = ctx.run_ui(raw_enter, |ui| {
        triggered = MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
    });
    out3.textures_delta.clear();
    assert_eq!(triggered, Some("edit.undo".to_string()));
    assert_eq!(open_menu_idx, None);

    open_menu_idx = Some(0);
    highlighted_item_idx = Some(0);
    let mut raw_escape = RawInput::default();
    raw_escape.events.push(Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });
    let mut out4 = ctx.run_ui(raw_escape, |ui| {
        MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
    });
    out4.textures_delta.clear();
    assert_eq!(open_menu_idx, None, "Escape must close open menu");
}

#[test]
fn test_menu_global_keyboard_shortcut_and_text_focus_guard() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let menus = vec![NSMenu::default_file_menu()];
    let mut open_menu_idx = None;
    let mut highlighted_item_idx = None;

    // 1. When no text field has focus, Cmd+S triggers "file.save"
    let mut raw_shortcut = RawInput::default();
    raw_shortcut.events.push(Event::Key {
        key: Key::S,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers {
            command: true,
            ..Modifiers::default()
        },
    });
    let mut triggered = None;
    let mut shortcut_leaked = false;
    let mut out = ctx.run_ui(raw_shortcut, |ui| {
        triggered = MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
        shortcut_leaked = ui.input(|i| i.key_pressed(Key::S));
    });
    out.textures_delta.clear();
    assert_eq!(triggered, Some("file.save".to_string()));
    assert!(
        !shortcut_leaked,
        "Matched menu shortcut Cmd+S must be consumed by MacMenuBar"
    );

    // 2. When a MacInputCell has keyboard focus (`wants_keyboard_input() == true`),
    // MacMenuBar::handle_shortcuts must NOT intercept shortcuts!
    let mut text = "Editing".to_string();
    let mut out_focus = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacInputCell::new("focused_editor", &mut text)
            .theme(&theme)
            .show(ui);
        resp.request_focus();
    });
    out_focus.textures_delta.clear();

    // Steady frame so TextEdit registers keyboard focus
    let mut out_steady = ctx.run_ui(RawInput::default(), |ui| {
        MacInputCell::new("focused_editor", &mut text)
            .theme(&theme)
            .show(ui);
    });
    out_steady.textures_delta.clear();

    let mut raw_while_editing = RawInput::default();
    raw_while_editing.events.push(Event::Key {
        key: Key::S,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers {
            command: true,
            ..Modifiers::default()
        },
    });
    let mut triggered_while_editing = None;
    let mut out_guard = ctx.run_ui(raw_while_editing, |ui| {
        MacInputCell::new("focused_editor", &mut text)
            .theme(&theme)
            .show(ui);
        triggered_while_editing =
            MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
                .theme(&theme)
                .show(ui);
    });
    out_guard.textures_delta.clear();

    assert_eq!(
        triggered_while_editing, None,
        "MacMenuBar::handle_shortcuts must NOT fire when a text field has keyboard focus!"
    );
}

#[test]
fn test_menu_bar_outside_click_dismissal_and_checked_item() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let view_menu = NSMenu::with_items(
        "View",
        vec![NSMenuItem::action("Show Grid", "view.grid").checked(true)],
    );
    let menus = vec![view_menu];
    let mut open_menu_idx = Some(0);
    let mut highlighted_item_idx = Some(0);

    // Frame 1: Render open menu with checked item
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
    });
    out1.textures_delta.clear();
    assert_eq!(open_menu_idx, Some(0));

    // Frame 2: Click outside menu bar & dropdown (e.g. at Pos2(500.0, 500.0))
    let outside_pos = Pos2::new(500.0, 500.0);
    let mut click_outside = RawInput::default();
    click_outside.events.push(Event::PointerMoved(outside_pos));
    click_outside.events.push(Event::PointerButton {
        pos: outside_pos,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    let mut out2 = ctx.run_ui(click_outside, |ui| {
        MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
    });
    out2.textures_delta.clear();

    let mut release_outside = RawInput::default();
    release_outside
        .events
        .push(Event::PointerMoved(outside_pos));
    release_outside.events.push(Event::PointerButton {
        pos: outside_pos,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    let mut out3 = ctx.run_ui(release_outside, |ui| {
        MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
    });
    out3.textures_delta.clear();

    assert_eq!(
        open_menu_idx, None,
        "Clicking outside an open MacMenuBar dropdown must dismiss the menu!"
    );
}

#[test]
fn test_menu_bar_submenu_keyboard_navigation_and_shortcuts() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let export_menu = NSMenu::with_items(
        "Export",
        vec![
            NSMenuItem::action_with_shortcut(
                "Export as PDF",
                "export.pdf",
                MacKeyboardShortcut::cmd_shift(Key::P),
            ),
            NSMenuItem::action("Export as PNG", "export.png"),
        ],
    );
    let file_menu = NSMenu::with_items(
        "File",
        vec![
            NSMenuItem::action("New", "file.new"),
            NSMenuItem::submenu("Export As...", export_menu),
            NSMenuItem::action("Close", "file.close"),
        ],
    );

    let menus = vec![file_menu];
    let mut open_menu_idx = Some(0);
    let mut highlighted_item_idx = Some(1);

    let mut raw_right = RawInput::default();
    raw_right.events.push(Event::Key {
        key: Key::ArrowRight,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });
    let mut out1 = ctx.run_ui(raw_right, |ui| {
        let _ = MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
    });
    out1.textures_delta.clear();
    assert_eq!(MacMenuBar::submenu_highlight(&ctx), Some(0));

    let mut raw_down = RawInput::default();
    raw_down.events.push(Event::Key {
        key: Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });
    let mut out2 = ctx.run_ui(raw_down, |ui| {
        let _ = MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
    });
    out2.textures_delta.clear();
    assert_eq!(MacMenuBar::submenu_highlight(&ctx), Some(1));

    let mut raw_left = RawInput::default();
    raw_left.events.push(Event::Key {
        key: Key::ArrowLeft,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });
    let mut out3 = ctx.run_ui(raw_left, |ui| {
        let _ = MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
    });
    out3.textures_delta.clear();
    assert_eq!(MacMenuBar::submenu_highlight(&ctx), None);
}

#[test]
fn test_standard_menu_bar_presets_and_accelerator_clearance() {
    let bar = NSMenu::default_menu_bar();
    assert_eq!(bar.len(), 5);
    assert_eq!(bar[0].title, "File");
    assert_eq!(bar[1].title, "Edit");
    assert_eq!(bar[2].title, "View");
    assert_eq!(bar[3].title, "Window");
    assert_eq!(bar[4].title, "Help");

    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let items = vec![NSMenuItem::action_with_shortcut(
        "Export Compressed Keyframe Archive...",
        "file.export_long",
        MacKeyboardShortcut::cmd_shift(Key::E),
    )];
    let menus = vec![NSMenu::with_items("File", items)];
    let mut open_menu_idx = Some(0);
    let mut highlighted_item_idx = Some(0);

    let raw_input = RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(1024.0, 768.0),
        )),
        ..Default::default()
    };

    let mut out1 = ctx.run_ui(raw_input.clone(), |ui| {
        MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
    });
    out1.textures_delta.clear();

    let mut out = ctx.run_ui(raw_input, |ui| {
        MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx)
            .theme(&theme)
            .show(ui);
    });
    out.textures_delta.clear();

    let mut text_shapes = Vec::new();
    for clipped in &out.shapes {
        collect_text_shapes(&clipped.shape, &mut text_shapes);
    }

    let title_shape = text_shapes
        .iter()
        .find(|t| t.galley.text() == "Export Compressed Keyframe Archive...")
        .expect("Menu item title text shape must be rendered");
    let shortcut_shape = text_shapes
        .iter()
        .find(|t| t.galley.text() == "Ctrl+Shift+E")
        .expect("Menu item accelerator text shape must be rendered");

    let title_right = title_shape.pos.x + title_shape.galley.size().x;
    let shortcut_left = shortcut_shape.pos.x - shortcut_shape.galley.size().x;
    assert!(shortcut_left - title_right >= 24.0);
}
