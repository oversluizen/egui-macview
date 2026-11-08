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

#[allow(dead_code)]
#[path = "../examples/macview_gallery.rs"]
mod gallery_app;
pub(crate) use gallery_app::{
    format_tracked_small_caps, CardModel, ExpandedProTab, GalleryApp, GalleryScreen, InspectorMode,
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
    for (pressed, expect_click) in [(true, false), (false, true)] {
        let inp = RawInput {
            events: vec![
                Event::PointerMoved(click_pos),
                Event::PointerButton {
                    pos: click_pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..Default::default()
        };
        let mut clicked_action = None;
        let mut out = ctx.run_ui(inp, |ui| {
            let footer =
                MacCanvasFooter::new(&breadcrumbs, Some(cursor_pos), zoom_factor).theme(&theme);
            clicked_action = footer.show(ui, container_width);
        });
        out.textures_delta.clear();
        if expect_click {
            assert_eq!(
                clicked_action,
                Some(MacCanvasFooterAction::BreadcrumbClicked(0))
            );
        }
    }
}

#[test]
fn test_command_palette_query_filter_and_navigation() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    assert_eq!(
        (COMMAND_PALETTE_WIDTH, COMMAND_PALETTE_HEIGHT),
        (480.0, 280.0)
    );

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
    assert_eq!(
        (palette.filtered_items().len(), palette.selected_index),
        (4, 0)
    );

    palette.set_query("export");
    let filtered = palette.filtered_items();
    assert_eq!((filtered.len(), filtered[0].id.as_str()), (1, "cmd.export"));

    palette.set_query("zoom");
    assert_eq!(palette.filtered_items().len(), 2);

    let key_inp = |key: Key| RawInput {
        events: vec![Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::default(),
        }],
        ..Default::default()
    };
    let mut out1 = ctx.run_ui(key_inp(Key::ArrowDown), |ui| {
        palette.show(ui.ctx(), &theme);
    });
    out1.textures_delta.clear();
    assert_eq!(palette.selected_index, 1);

    let mut triggered = None;
    let mut out2 = ctx.run_ui(key_inp(Key::Enter), |ui| {
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

    let esc_inp = || RawInput {
        events: vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::default(),
        }],
        ..Default::default()
    };
    let mut esc_leaked = false;
    let mut out1 = ctx.run_ui(esc_inp(), |ui| {
        palette.show(ui.ctx(), &theme);
        esc_leaked = ui.input(|i| i.key_pressed(Key::Escape));
    });
    out1.textures_delta.clear();
    assert!(palette.is_open() && palette.query().is_empty() && !esc_leaked);

    let mut out2 = ctx.run_ui(esc_inp(), |ui| {
        palette.show(ui.ctx(), &theme);
    });
    out2.textures_delta.clear();
    assert!(!palette.is_open());
}

#[test]
fn test_gallery_v05_five_state_studio_and_frameless_decorations() {
    let mut combined_src = std::fs::read_to_string("examples/macview_gallery.rs")
        .expect("Must read examples/macview_gallery.rs");
    if let Ok(entries) = std::fs::read_dir("examples/gallery") {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(sub) = std::fs::read_to_string(entry.path()) {
                    combined_src.push('\n');
                    combined_src.push_str(&sub);
                }
            }
        }
    }

    assert!(
        combined_src.contains("ViewportCommand::Decorations("),
        "Gallery must dynamically toggle ViewportCommand::Decorations based on WindowControlsStyle"
    );
    assert!(
        combined_src.contains("ViewportCommand::BeginResize("),
        "Frameless gallery must support edge/corner resizing via ViewportCommand::BeginResize"
    );
    assert!(
        combined_src.contains(".numeric("),
        "Gallery numeric inspector fields must wire .numeric(range, step) for arrow-key stepping"
    );
    for required_v05 in [
        "MacPathControl::new",
        "MacTableView::new",
        "MacTokenField::new",
        ".max_chars(",
        "MacProgressIndicator::",
        "MacTrimScrubber::new",
        "MacLevelIndicator::new",
        ".peak_hold(",
        "interactive_canvas_card_resize",
        "!on_resize_border",
    ] {
        assert!(
            combined_src.contains(required_v05),
            "5-State v0.5 Gallery must integrate {required_v05}"
        );
    }
}

#[test]
fn test_split_view_sibling_panes_have_distinct_auto_ids() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut split_pos = 180.0;
    let mut first_id = egui::Id::NULL;
    let mut second_id = egui::Id::NULL;

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        egui_macview::MacSplitView::with_pos(
            "split_id_test",
            egui_macview::MacSplitOrientation::Vertical,
            &mut split_pos,
        )
        .theme(&theme)
        .show(
            ui,
            |first_ui| {
                first_id = first_ui.next_auto_id();
            },
            |second_ui| {
                second_id = second_ui.next_auto_id();
            },
        );
    });
    out.textures_delta.clear();

    assert_ne!(
        first_id, second_id,
        "MacSplitView first and second child UIs must have distinct id_salt scopes so widgets never collide or flicker"
    );
}

#[test]
fn test_caspring_controls_do_not_request_repaint_at_rest() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut toggle_val = true;
    let mut slider_val = 50.0;
    let mut expanded = true;

    let run_frame =
        |time: f64, toggle_val: &mut bool, slider_val: &mut f32, expanded: &mut bool| {
            let raw = RawInput {
                time: Some(time),
                ..Default::default()
            };
            let mut out = ctx.run_ui(raw, |ui| {
                ui.add(egui_macview::MacToggle::new(toggle_val).theme(&theme));
                egui_macview::MacSegmentedControl::new(&["Points", "Pixels", "Percent"], 0)
                    .theme(&theme)
                    .show(ui, |_| {});
                egui_macview::MacSlider::new(slider_val, 0.0..=100.0)
                    .theme(&theme)
                    .show(ui);
                egui_macview::MacOutlineView::new("idle_outline")
                    .theme(&theme)
                    .show(ui, |tree| {
                        tree.row("row_a", 0, true, expanded, false, |row| {
                            row.label("Idle Row");
                        });
                    });
            });
            out.textures_delta.clear();
            out.viewport_output[&egui::ViewportId::ROOT].repaint_delay
        };

    // Frame 1 & 2: initial layout/font warmup
    let _ = run_frame(1.0, &mut toggle_val, &mut slider_val, &mut expanded);
    let _ = run_frame(1.016, &mut toggle_val, &mut slider_val, &mut expanded);
    // Frame 3: steady state with zero state transitions must NOT request immediate repaint
    let delay = run_frame(1.032, &mut toggle_val, &mut slider_val, &mut expanded);
    assert!(
        !delay.is_zero(),
        "Steady-state CASpring controls must not request continuous repaints (got {delay:?})"
    );

    // Frame 4: state transition (toggle_val: true -> false) MUST request immediate repaint and interpolate
    toggle_val = false;
    let delay_on_change = run_frame(2.0, &mut toggle_val, &mut slider_val, &mut expanded);
    assert!(
        delay_on_change.is_zero(),
        "Changing MacToggle state must trigger an immediate CASpring animation repaint"
    );

    // Direct CASpring::animate_f32 mid-flight interpolation, reversal, and settling verification
    let spring_ctx = Context::default();
    let spring = theme.motion().swiftui_bouncy.to_spring();
    let anim_id = egui::Id::NULL.with("direct_spring_test");
    let eval_at = |time: f64, target: f32| -> (f32, std::time::Duration) {
        let mut val = 0.0;
        let mut out = spring_ctx.run_ui(
            RawInput {
                time: Some(time),
                ..Default::default()
            },
            |ui| {
                val = spring.animate_f32(ui.ctx(), anim_id, target);
            },
        );
        out.textures_delta.clear();
        (
            val,
            out.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
        )
    };
    let _ = eval_at(1.0, 0.0);
    let _ = eval_at(1.016, 0.0);
    let (v0, d0) = eval_at(1.032, 0.0);
    assert_eq!(v0, 0.0);
    assert!(!d0.is_zero());
    let (v_start, d_start) = eval_at(2.0, 1.0);
    assert!((v_start - 0.0).abs() < 1e-4 && d_start.is_zero());
    let (v_mid, d_mid) = eval_at(2.1, 1.0);
    assert!(
        v_mid > 0.1 && v_mid < 0.99 && d_mid.is_zero(),
        "Mid-flight spring must interpolate (got {v_mid})"
    );
    // Mid-flight reversal back to 0.0 must start smoothly from v_mid
    let (v_rev, d_rev) = eval_at(2.15, 0.0);
    assert!(
        v_rev > 0.1 && v_rev < 1.0 && d_rev.is_zero(),
        "Mid-flight reversal must preserve displacement (got {v_rev})"
    );
    // After settling_duration(), spring must settle at 0.0 and stop requesting repaints
    let _ = eval_at(2.15 + spring.settling_duration() as f64 + 0.1, 0.0);
    let (v_settled, d_settled) = eval_at(2.15 + spring.settling_duration() as f64 + 0.2, 0.0);
    assert_eq!(v_settled, 0.0);
    assert!(!d_settled.is_zero());
}

#[test]
fn test_inspector_convert_card_units_and_scale() {
    let mut card = gallery_app::CardModel::default();
    assert_eq!(
        (
            card.width.as_str(),
            card.height.as_str(),
            card.radius.as_str()
        ),
        ("260.0", "150.0", "10.0")
    );

    // Points (0) -> Pixels (1) (2x Retina scale)
    gallery_app::convert_card_units(&mut card, 0, 1);
    assert_eq!(
        (
            card.width.as_str(),
            card.height.as_str(),
            card.radius.as_str()
        ),
        ("520.0", "300.0", "20.0")
    );

    // Pixels (1) -> Percent (2) (260pt = 100%, 150pt = 100%, 10pt / 40pt = 25%)
    gallery_app::convert_card_units(&mut card, 1, 2);
    assert_eq!(
        (
            card.width.as_str(),
            card.height.as_str(),
            card.radius.as_str()
        ),
        ("100.0", "100.0", "25.0")
    );

    // Percent (2) -> Points (0) round-trip
    gallery_app::convert_card_units(&mut card, 2, 0);
    assert_eq!(
        (
            card.width.as_str(),
            card.height.as_str(),
            card.radius.as_str()
        ),
        ("260.0", "150.0", "10.0")
    );
}
