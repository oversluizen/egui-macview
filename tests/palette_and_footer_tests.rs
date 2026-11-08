//! ISO 29119-4 / macOS HIG Specification Tests for `MacCommandPalette` and `MacCanvasFooter`.
//!
//! Normative Reference: SPEC-007 §3 & §4.
//! Verifies Spotlight Command Palette filtering, keyboard navigation, key consumption,
//! and Canvas Footer 24pt layout budget + interactive breadcrumb segment clicks.

use egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    CommandPaletteItem, MacCanvasFooter, MacCanvasFooterAction, MacCommandPalette,
    MacKeyboardShortcut, CANVAS_FOOTER_HEIGHT, COMMAND_PALETTE_HEIGHT, COMMAND_PALETTE_WIDTH,
};

#[test]
fn test_canvas_footer_24pt_layout_budget_and_interactive_breadcrumbs() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    assert_eq!(CANVAS_FOOTER_HEIGHT, 24.0);

    let breadcrumbs = ["macview_gallery", "Screen 1", "Interactive Canvas"];
    let cursor_pos = Pos2::new(128.5, 256.0);
    let zoom_factor = 1.0f32;
    let container_width = 600.0f32;

    // Frame 1: Render footer
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let footer =
            MacCanvasFooter::new(&breadcrumbs, Some(cursor_pos), zoom_factor).theme(&theme);
        assert_eq!(footer.get_height(), 24.0);
        let action = footer.show(ui, container_width);
        assert_eq!(action, None);
    });
    out1.textures_delta.clear();

    // Frame 2: Click on the first breadcrumb segment ("macview_gallery" near x=25.0, y=12.0)
    let click_pos = Pos2::new(25.0, 12.0);
    let mut press_input = RawInput::default();
    press_input.events.push(Event::PointerMoved(click_pos));
    press_input.events.push(Event::PointerButton {
        pos: click_pos,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    let mut out2 = ctx.run_ui(press_input, |ui| {
        let footer =
            MacCanvasFooter::new(&breadcrumbs, Some(cursor_pos), zoom_factor).theme(&theme);
        let _ = footer.show(ui, container_width);
    });
    out2.textures_delta.clear();

    let mut release_input = RawInput::default();
    release_input.events.push(Event::PointerMoved(click_pos));
    release_input.events.push(Event::PointerButton {
        pos: click_pos,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    let mut clicked_action = None;
    let mut out3 = ctx.run_ui(release_input, |ui| {
        let footer =
            MacCanvasFooter::new(&breadcrumbs, Some(cursor_pos), zoom_factor).theme(&theme);
        clicked_action = footer.show(ui, container_width);
    });
    out3.textures_delta.clear();

    assert_eq!(
        clicked_action,
        Some(MacCanvasFooterAction::BreadcrumbClicked(0)),
        "Clicking first breadcrumb segment in MacCanvasFooter must return BreadcrumbClicked(0)"
    );
}

#[test]
fn test_command_palette_query_filter_and_navigation() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

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
    assert_eq!(palette.filtered_items().len(), 4);
    assert_eq!(palette.selected_index, 0);

    palette.set_query("export");
    let filtered = palette.filtered_items();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "cmd.export");

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
}

#[test]
fn test_command_palette_fuzzy_subsequence_scoring_and_escape() {
    assert!(MacCommandPalette::fuzzy_match_score("expdf", "Export as PDF").is_some());
    assert!(MacCommandPalette::fuzzy_match_score("sv", "Save Document").is_some());
    assert!(MacCommandPalette::fuzzy_match_score("zout", "Zoom Out").is_some());
    assert!(MacCommandPalette::fuzzy_match_score("xyz", "Export as PDF").is_none());

    let score_prefix = MacCommandPalette::fuzzy_match_score("exp", "Export as PDF").unwrap();
    let score_scatter = MacCommandPalette::fuzzy_match_score("eaf", "Export as PDF").unwrap();
    assert!(score_prefix > score_scatter);

    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let items = vec![
        CommandPaletteItem::new("cmd.save", "Save Document"),
        CommandPaletteItem::new("cmd.export", "Export Document"),
    ];

    let mut palette = MacCommandPalette::new(items).open(true);
    palette.set_query("export");

    // First Escape clears query and consumes Escape
    let mut raw_escape1 = RawInput::default();
    raw_escape1.events.push(Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });
    let mut esc_leaked = false;
    let mut out1 = ctx.run_ui(raw_escape1, |ui| {
        palette.show(ui.ctx(), &theme);
        esc_leaked = ui.input(|i| i.key_pressed(Key::Escape));
    });
    out1.textures_delta.clear();
    assert!(palette.is_open());
    assert_eq!(palette.query(), "");
    assert!(!esc_leaked, "MacCommandPalette must consume Key::Escape");

    // Second Escape closes palette
    let mut raw_escape2 = RawInput::default();
    raw_escape2.events.push(Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::default(),
    });
    let mut out2 = ctx.run_ui(raw_escape2, |ui| {
        palette.show(ui.ctx(), &theme);
    });
    out2.textures_delta.clear();
    assert!(!palette.is_open());
}
