//! ISO 29119-4 / macOS HIG Specification Tests for Desktop Navigation and Palette.
//!
//! Normative Reference: SPEC-007 (Desktop Navigation, Canvas Footer & Palette).
//! Grounded in AppKit `NSMenu`, `NSMenuItem`, Canvas Footer $24\,\text{pt}$ layout budget,
//! and Spotlight Command Palette (`Cmd+K` / `Ctrl+K`).

use egui::{Context, Event, Key, Modifiers, Pos2, RawInput};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    CommandPaletteItem, MacCanvasFooter, MacCommandPalette, MacKeyboardShortcut, MacMenuBar,
    NSMenu, CANVAS_FOOTER_HEIGHT, COMMAND_PALETTE_HEIGHT, COMMAND_PALETTE_WIDTH,
};

#[test]
fn test_menu_bar_keyboard_navigation_and_action() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let menus = vec![NSMenu::default_file_menu(), NSMenu::default_edit_menu()];

    let mut open_menu_idx = Some(0);
    let mut highlighted_item_idx = Some(0);

    // 1. Initial State: Menu 0 (File) is open, item 0 is highlighted
    let mut raw_down = RawInput::default();
    raw_down.events.push(Event::Key {
        key: Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });

    let mut out1 = ctx.run_ui(raw_down, |ui| {
        let bar =
            MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx).theme(&theme);

        bar.show(ui);
    });
    out1.textures_delta.clear();

    // Arrow down should advance highlight from 0 to 1
    assert_eq!(open_menu_idx, Some(0));
    assert_eq!(highlighted_item_idx, Some(1));

    // 2. Arrow Right: switches to next top menu (Edit)
    let mut raw_right = RawInput::default();
    raw_right.events.push(Event::Key {
        key: Key::ArrowRight,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });

    let mut out2 = ctx.run_ui(raw_right, |ui| {
        let bar =
            MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx).theme(&theme);

        bar.show(ui);
    });
    out2.textures_delta.clear();

    assert_eq!(open_menu_idx, Some(1));
    assert_eq!(highlighted_item_idx, Some(0));

    // 3. Enter key triggers action of currently highlighted item in Edit menu (Undo: "edit.undo")
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
        let bar =
            MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx).theme(&theme);

        triggered = bar.show(ui);
    });
    out3.textures_delta.clear();

    assert_eq!(triggered, Some("edit.undo".to_string()));
    assert_eq!(
        open_menu_idx, None,
        "Menu should close after action trigger"
    );

    // 4. Escape closes open menu
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
        let bar =
            MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx).theme(&theme);

        bar.show(ui);
    });
    out4.textures_delta.clear();

    assert_eq!(open_menu_idx, None, "Escape must close open menu");
}

#[test]
fn test_menu_global_keyboard_shortcut_trigger() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let menus = vec![NSMenu::default_file_menu()];
    let mut open_menu_idx = None;
    let mut highlighted_item_idx = None;

    // Press Cmd+S (or Ctrl+S) without opening any menu
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
    let mut out = ctx.run_ui(raw_shortcut, |ui| {
        let bar =
            MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx).theme(&theme);

        triggered = bar.show(ui);
    });
    out.textures_delta.clear();

    assert_eq!(triggered, Some("file.save".to_string()));
}

#[test]
fn test_canvas_footer_24pt_layout_budget() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    // Specification constraint: exact 24pt deterministic layout
    assert_eq!(CANVAS_FOOTER_HEIGHT, 24.0);

    let breadcrumbs = ["Project", "Flowchart", "Node 42"];
    let cursor_pos = Pos2::new(128.5, 256.0);
    let zoom_factor = 1.0f32;
    let container_width = 600.0f32;

    let mut action_result = None;
    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let footer =
            MacCanvasFooter::new(&breadcrumbs, Some(cursor_pos), zoom_factor).theme(&theme);

        assert_eq!(
            footer.get_height(),
            24.0,
            "Canvas footer must strictly occupy 24.0pt vertical budget"
        );

        let action = footer.show(ui, container_width);
        action_result = action;
    });
    out.textures_delta.clear();
    let _ = action_result;
}

#[test]
fn test_command_palette_query_filter_and_navigation() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    // Modal HUD size verification
    assert_eq!(COMMAND_PALETTE_WIDTH, 480.0);
    assert_eq!(COMMAND_PALETTE_HEIGHT, 280.0);

    let items = vec![
        CommandPaletteItem::new("cmd.save", "Save Document")
            .shortcut(MacKeyboardShortcut::cmd(Key::S).format_text())
            .category("File"),
        CommandPaletteItem::new("cmd.export", "Export as PDF").category("File"),
        CommandPaletteItem::new("cmd.zoom_in", "Zoom In")
            .shortcut(MacKeyboardShortcut::cmd(Key::Plus).format_text())
            .category("View"),
        CommandPaletteItem::new("cmd.zoom_out", "Zoom Out")
            .shortcut(MacKeyboardShortcut::cmd(Key::Minus).format_text())
            .category("View"),
    ];

    let mut palette = MacCommandPalette::new(items).open(true);

    // 1. Initial State: 4 items, selected_index 0
    assert_eq!(palette.filtered_items().len(), 4);
    assert_eq!(palette.selected_index, 0);

    // 2. Query filtering
    palette.set_query("export");
    let filtered = palette.filtered_items();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "cmd.export");

    // 3. Arrow Down and Up on filtered results
    palette.set_query("zoom");
    assert_eq!(palette.filtered_items().len(), 2);

    let mut raw_down = RawInput::default();
    raw_down.events.push(Event::Key {
        key: Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });

    let mut out1 = ctx.run_ui(raw_down, |ui| {
        palette.show(ui.ctx(), &theme);
    });
    out1.textures_delta.clear();

    assert_eq!(palette.selected_index, 1);

    // 4. Enter executes highlighted command
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
        triggered = palette.show(ui.ctx(), &theme);
    });
    out2.textures_delta.clear();

    assert_eq!(triggered, Some("cmd.zoom_out".to_string()));
    assert!(!palette.is_open(), "Palette must close on Enter execution");

    // 5. Escape closes palette
    palette.open = true;
    let mut raw_escape = RawInput::default();
    raw_escape.events.push(Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });

    let mut out3 = ctx.run_ui(raw_escape, |ui| {
        palette.show(ui.ctx(), &theme);
    });
    out3.textures_delta.clear();

    assert!(!palette.is_open(), "Escape must close command palette");
}
