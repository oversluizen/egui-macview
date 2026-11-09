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

fn key_raw_input(key: Key) -> RawInput {
    RawInput {
        events: vec![Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::default(),
        }],
        ..Default::default()
    }
}

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
        assert_eq!(footer.show(ui, container_width), None);
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

    let mut out1 = ctx.run_ui(key_raw_input(Key::ArrowDown), |ui| {
        palette.show(ui.ctx(), &theme);
    });
    out1.textures_delta.clear();
    assert_eq!(palette.selected_index, 1);

    let mut triggered = None;
    let mut out2 = ctx.run_ui(key_raw_input(Key::Enter), |ui| {
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

    let mut esc_leaked = false;
    let mut out1 = ctx.run_ui(key_raw_input(Key::Escape), |ui| {
        palette.show(ui.ctx(), &theme);
        esc_leaked = ui.input(|i| i.key_pressed(Key::Escape));
    });
    out1.textures_delta.clear();
    assert!(palette.is_open() && palette.query().is_empty() && !esc_leaked);

    let mut out2 = ctx.run_ui(key_raw_input(Key::Escape), |ui| {
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
    let reqs = [
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
    ];
    for req in reqs {
        assert!(combined_src.contains(req), "Gallery must integrate {req}");
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
            |first_ui| first_id = first_ui.next_auto_id(),
            |second_ui| second_id = second_ui.next_auto_id(),
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
                            row.label("Idle Row")
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
    let eval_at = |time: f64, target: f32| {
        let mut val = 0.0;
        let raw = RawInput {
            time: Some(time),
            ..Default::default()
        };
        let mut out = spring_ctx.run_ui(raw, |ui| {
            val = spring.animate_f32(ui.ctx(), anim_id, target)
        });
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
    let mut c = gallery_app::CardModel::default();
    fn d(card: &gallery_app::CardModel) -> (&str, &str, &str) {
        (&card.width, &card.height, &card.radius)
    }
    for (from, to, exp) in [
        (0, 0, ("260.0", "150.0", "10.0")),
        (0, 1, ("520.0", "300.0", "20.0")),
        (1, 2, ("100.0", "100.0", "25.0")),
        (2, 0, ("260.0", "150.0", "10.0")),
    ] {
        if from != to {
            gallery_app::convert_card_units(&mut c, from, to);
        }
        assert_eq!(d(&c), exp);
    }
}

#[test]
fn test_gallery_native_to_custom_chrome_resize_and_hitbox_resync() {
    use egui::ViewportCommand as VC;
    use egui_macview::theme::WindowControlsStyle as WCS;
    let (ctx, mut app) = (Context::default(), gallery_app::GalleryApp::new());
    let run_frame = |app: &mut gallery_app::GalleryApp,
                     in_sz: egui::Vec2,
                     out_sz: egui::Vec2,
                     max: Option<bool>| {
        let (rin, rout) = (
            egui::Rect::from_min_size(Pos2::ZERO, in_sz),
            egui::Rect::from_min_size(Pos2::ZERO, out_sz),
        );
        let mut raw = RawInput {
            screen_rect: Some(rin),
            ..Default::default()
        };
        let vp = raw.viewports.entry(egui::ViewportId::ROOT).or_default();
        (vp.inner_rect, vp.outer_rect, vp.maximized) = (Some(rin), Some(rout), max);
        let mut out = ctx.run_ui(raw, |ui| app.render_ui(ui));
        out.textures_delta.clear();
        let vp_out = out.viewport_output.remove(&egui::ViewportId::ROOT).unwrap();
        (vp_out.commands, vp_out.repaint_delay)
    };
    let rf = |app: &mut gallery_app::GalleryApp, sz: egui::Vec2, max: Option<bool>| {
        run_frame(app, sz, sz, max)
    };

    let init_sz = egui::vec2(1160.0, 740.0);
    let _ = rf(&mut app, init_sz, Some(false));
    let (cmds_idle, _) = rf(&mut app, init_sz, Some(false));
    assert!(!cmds_idle
        .iter()
        .any(|c| matches!(c, VC::Decorations(_) | VC::InnerSize(_))));

    // 1. AppleTrafficLights -> NativeHost: Decorations(true) + 1-frame InnerSize nudge/restore + repaint
    app.window_controls_style = WCS::NativeHost;
    let (c0, d0) = rf(&mut app, init_sz, Some(false));
    assert!(c0.contains(&VC::Decorations(true)) && d0.is_zero());
    assert!(c0.contains(&VC::InnerSize(egui::vec2(1161.0, 741.0))));
    let (c1, d1) = rf(&mut app, init_sz, Some(false));
    assert!(c1.contains(&VC::InnerSize(init_sz)) && d1.is_zero());
    let (c2, _) = rf(&mut app, init_sz, Some(false));
    assert!(!c2.iter().any(|c| matches!(c, VC::InnerSize(_))));

    // 2. Resize in NativeHost to 1320x840, then switch back to WindowsCaptionButtons
    let new_sz = egui::vec2(1320.0, 840.0);
    let _ = rf(&mut app, new_sz, Some(false));
    app.window_controls_style = WCS::WindowsCaptionButtons;
    let (cw0, _) = rf(&mut app, new_sz, Some(false));
    assert!(cw0.contains(&VC::Decorations(false)));
    assert!(cw0.contains(&VC::InnerSize(egui::vec2(1321.0, 841.0))));
    let (cw1, dw1) = rf(&mut app, new_sz, Some(false));
    assert!(cw1.contains(&VC::InnerSize(new_sz)) && dw1.is_zero());

    // 3. Maximize in NativeHost, then switch to AppleTrafficLights: retains maximization without unmaximize glitch
    app.window_controls_style = WCS::NativeHost;
    let _ = rf(&mut app, new_sz, Some(true));
    assert!(app.is_maximized);
    app.window_controls_style = WCS::AppleTrafficLights;
    let (cmds_max, _) = rf(&mut app, new_sz, Some(true));
    assert!(!cmds_max.contains(&VC::Maximized(false)) && cmds_max.contains(&VC::Maximized(true)));
    let (cmds_max_idle, _) = rf(&mut app, new_sz, Some(true));
    assert!(!cmds_max_idle.iter().any(|c| matches!(c, VC::InnerSize(_))));
    assert!(app.is_maximized, "App must remain maximized");

    // 4. Rapid consecutive toggle mid-resync preserves target_size without +1pt drift
    app.window_controls_style = WCS::NativeHost;
    let _ = rf(&mut app, new_sz, Some(false));
    app.window_controls_style = WCS::WindowsCaptionButtons;
    let _ = rf(&mut app, new_sz, Some(false));
    let (cmds_rapid, _) = rf(&mut app, new_sz, Some(false));
    assert!(cmds_rapid.contains(&VC::InnerSize(new_sz)));

    // 5. Switching from NativeHost (where outer_rect > inner_rect) preserves outer_rect bounds
    let (nat_in, nat_out) = (egui::vec2(1144.0, 692.0), egui::vec2(1160.0, 740.0));
    app.window_controls_style = WCS::NativeHost;
    let _ = run_frame(&mut app, nat_in, nat_out, Some(false));
    let _ = run_frame(&mut app, nat_in, nat_out, Some(false));
    app.window_controls_style = WCS::AppleTrafficLights;
    let (c_pres, _) = run_frame(&mut app, nat_in, nat_out, Some(false));
    assert!(c_pres.contains(&VC::Decorations(false)));
    assert!(c_pres.contains(&VC::InnerSize(egui::vec2(nat_out.x + 1.0, nat_out.y + 1.0))));
    let (c_rest, _) = run_frame(&mut app, nat_in, nat_out, Some(false));
    assert!(c_rest.contains(&VC::InnerSize(nat_out)));
}
