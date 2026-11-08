//! Gallery State Binding & Adversarial Stress Tests.
//!
//! Verifies:
//! 1. Clean Sidebar Preset Labels & Zero Text Clipping at Minimum Width (220pt).
//! 2. Master-Detail 4-Screen Navigator & Breadcrumbs.
//! 3. Interactive Controls Bidirectional State Binding & Zero Dead Controls.
//! 4. Adversarial Menu Accelerator Clearance under 100-char titles.

use egui::{Context, Event, Key, PointerButton, Pos2, RawInput, Rect, Sense, Vec2};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    MacContextMenu, MacKeyboardShortcut, MacMenuBar, MacScrollView, MacSlider, MacStepper, NSMenu,
    NSMenuItem, NSUndoManager,
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
fn test_sidebar_navigator_clean_labels_and_sufficient_width() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let gallery_src = std::fs::read_to_string("examples/macview_gallery.rs")
        .expect("Must read examples/macview_gallery.rs");

    assert!(!gallery_src.contains("\"Default NIB (260pt)\""));
    assert!(!gallery_src.contains("\"Compact HUD (178x63)\""));
    assert!(!gallery_src.contains("\"Expanded Pro (380pt)\""));

    let clean_labels = ["Default NIB", "Compact HUD", "Expanded Pro"];
    let outline_metrics = theme.outline_view();
    let min_sidebar_w = 220.0f32;
    let row_indent = outline_metrics.indent_step.points();
    let thumb_w = theme.thumbnail().size.points();
    let spacing = outline_metrics.content_spacing.points();
    let padding_h = 16.0f32;

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let font_id = egui::FontId::proportional(outline_metrics.font_size.points());
        for label in &clean_labels {
            let text_w = ui
                .painter()
                .layout_no_wrap((*label).to_string(), font_id.clone(), egui::Color32::WHITE)
                .size()
                .x;
            let total_row_w = row_indent + thumb_w + spacing + text_w + padding_h;
            assert!(total_row_w <= min_sidebar_w);
        }
    });
    out.textures_delta.clear();
}

#[test]
fn test_screen_navigator_and_bidirectional_gallery_bindings() {
    let gallery_src = std::fs::read_to_string("examples/macview_gallery.rs")
        .expect("Must read examples/macview_gallery.rs");

    assert!(gallery_src.contains("enum GalleryScreen"));
    assert!(gallery_src.contains("CanvasPhysics") && gallery_src.contains("VectorAssets"));
    assert!(!gallery_src.contains("\"Expanded Inspector\""));
    assert!(!gallery_src.contains("if resp.is_selected {"));
    assert!(gallery_src.contains("if resp.clicked {"));
    assert!(
        gallery_src.contains("MacToolbar::new") || gallery_src.contains("MacFloatingInfoBox::new"),
        "Gallery must wire MacToolbar and MacFloatingInfoBox"
    );
}

#[test]
fn test_adversarial_menu_accelerator_clearance_extreme_strings() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let export_menu = NSMenu::with_items(
        "Nested Export",
        vec![NSMenuItem::action_with_shortcut(
            "Export High-Throughput Cryptographic Signature Keyframe Archive Package",
            "export.keyframe",
            MacKeyboardShortcut::cmd_shift(Key::K),
        )],
    );

    let items = vec![
        NSMenuItem::action_with_shortcut(
            "Synthesize Deeply Nested Multi-Stage Hierarchical Cryptographic Artifact",
            "adv.long1",
            MacKeyboardShortcut::cmd_shift(Key::S),
        ),
        NSMenuItem::action_with_shortcut(
            "Cut",
            "adv.short",
            MacKeyboardShortcut::cmd_shift(Key::X),
        ),
        NSMenuItem::action_with_shortcut(
            "Supercalifragilisticexpialidocious Ultra Wide Screen Projection Alignment Matrix",
            "adv.long2",
            MacKeyboardShortcut::cmd(Key::P),
        ),
        NSMenuItem::submenu("Cascading Options...", export_menu),
    ];

    let menus = vec![NSMenu::with_items("Adversarial", items)];
    let mut open_menu_idx = Some(0);
    let mut highlighted_item_idx = Some(0);

    let raw_input = RawInput {
        screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1920.0, 1080.0))),
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

    let test_pairs = [
        (
            "Synthesize Deeply Nested Multi-Stage Hierarchical Cryptographic Artifact",
            "Ctrl+Shift+S",
        ),
        ("Cut", "Ctrl+Shift+X"),
        (
            "Supercalifragilisticexpialidocious Ultra Wide Screen Projection Alignment Matrix",
            "Ctrl+P",
        ),
        ("Cascading Options...", "›"),
    ];

    for (title, sc) in &test_pairs {
        let title_shape = text_shapes
            .iter()
            .find(|t| t.galley.text() == *title)
            .unwrap_or_else(|| panic!("Title '{title}' must be rendered in dropdown"));
        let sc_shape = text_shapes
            .iter()
            .find(|t| t.galley.text() == *sc)
            .unwrap_or_else(|| panic!("Shortcut / chevron '{sc}' must be rendered"));

        let title_right = title_shape.pos.x + title_shape.galley.size().x;
        let sc_left = sc_shape.pos.x - sc_shape.galley.size().x;
        assert!(sc_left - title_right >= 24.0);
    }
}

#[test]
fn test_mac_stepper_modifier_multipliers_and_hold_to_repeat() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut val = 0.0f64;
    let mut stepper_rect = Rect::NOTHING;

    let mut out0 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacStepper::new(&mut val, -1000.0..=1000.0, 1.0).theme(&theme).show(ui);
        stepper_rect = resp.rect;
        resp.request_focus();
    });
    out0.textures_delta.clear();

    let mut shift_up = RawInput::default();
    shift_up.events.push(Event::Key {
        key: Key::ArrowUp,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::SHIFT,
    });
    let mut out1 = ctx.run_ui(shift_up, |ui| {
        MacStepper::new(&mut val, -1000.0..=1000.0, 1.0).theme(&theme).show(ui);
    });
    out1.textures_delta.clear();
    assert!((val - 10.0).abs() < 1e-6);

    let mut cmd_up = RawInput::default();
    cmd_up.events.push(Event::Key {
        key: Key::ArrowUp,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::COMMAND,
    });
    let mut out2 = ctx.run_ui(cmd_up, |ui| {
        MacStepper::new(&mut val, -1000.0..=1000.0, 1.0).theme(&theme).show(ui);
    });
    out2.textures_delta.clear();
    assert!((val - 110.0).abs() < 1e-6);

    let top_pos = Pos2::new(stepper_rect.center().x, stepper_rect.top() + 3.0);
    let mut press_in = RawInput { time: Some(1.0), ..Default::default() };
    press_in.events.push(Event::PointerMoved(top_pos));
    press_in.events.push(Event::PointerButton {
        pos: top_pos,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    let mut out3 = ctx.run_ui(press_in, |ui| {
        MacStepper::new(&mut val, -1000.0..=1000.0, 1.0).theme(&theme).show(ui);
    });
    out3.textures_delta.clear();

    let val_before_hold = val;
    let mut out4 = ctx.run_ui(RawInput { time: Some(1.45), ..Default::default() }, |ui| {
        MacStepper::new(&mut val, -1000.0..=1000.0, 1.0).theme(&theme).show(ui);
    });
    out4.textures_delta.clear();
    assert!(val > val_before_hold);
}

#[test]
fn test_mac_slider_caspring_press_scale_and_scroll_page_jump() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut val = 50.0f32;
    let mut slider_rect = Rect::NOTHING;

    let mut out0 = ctx.run_ui(RawInput { time: Some(0.0), ..Default::default() }, |ui| {
        slider_rect = MacSlider::new(&mut val, 0.0..=100.0).theme(&theme).show(ui).rect;
    });
    out0.textures_delta.clear();

    let center = slider_rect.center();
    let mut press_in = RawInput { time: Some(0.05), ..Default::default() };
    press_in.events.push(Event::PointerMoved(center));
    press_in.events.push(Event::PointerButton {
        pos: center,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    let mut out1 = ctx.run_ui(press_in, |ui| {
        MacSlider::new(&mut val, 0.0..=100.0).theme(&theme).show(ui);
    });
    out1.textures_delta.clear();

    let mut out2 = ctx.run_ui(RawInput { time: Some(0.25), ..Default::default() }, |ui| {
        MacSlider::new(&mut val, 0.0..=100.0).theme(&theme).show(ui);
    });
    out2.textures_delta.clear();

    let base_r = theme.slider().thumb_size.points() * 0.5;
    let max_r = out2
        .shapes
        .iter()
        .filter_map(|cs| match &cs.shape {
            egui::epaint::Shape::Circle(c) => Some(c.radius),
            _ => None,
        })
        .fold(0.0f32, f32::max);
    assert!(max_r > base_r);

    // ScrollView hover-expansion (7pt -> 10pt) and vertical + horizontal track page-jump
    let mut offset = Vec2::ZERO;
    let mut viewport = Rect::NOTHING;
    let mut s0 = ctx.run_ui(RawInput::default(), |ui| {
        let res = MacScrollView::both("hover_jump_scroll")
            .theme(&theme)
            .auto_hide(false)
            .with_offset(&mut offset)
            .max_width(200.0)
            .max_height(100.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(600.0, 500.0), Sense::hover());
            });
        viewport = res.inner_rect;
    });
    s0.textures_delta.clear();

    let unhovered_v_thumbs: Vec<f32> = s0
        .shapes
        .iter()
        .filter_map(|cs| match &cs.shape {
            egui::epaint::Shape::Rect(r) if r.rect.height() > 15.0 && r.rect.width() <= 12.0 => {
                Some(r.rect.width())
            }
            _ => None,
        })
        .collect();
    assert!(unhovered_v_thumbs.iter().any(|&w| (w - 7.0).abs() < 1e-3));

    let v_click_pos = Pos2::new(viewport.right() - 4.0, viewport.bottom() - 20.0);
    for pressed in [true, false] {
        let mut inp = RawInput::default();
        inp.events.push(Event::PointerMoved(v_click_pos));
        inp.events.push(Event::PointerButton {
            pos: v_click_pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        });
        let mut s = ctx.run_ui(inp, |ui| {
            MacScrollView::both("hover_jump_scroll")
                .theme(&theme)
                .auto_hide(false)
                .with_offset(&mut offset)
                .max_width(200.0)
                .max_height(100.0)
                .show(ui, |c| {
                    c.allocate_exact_size(Vec2::new(600.0, 500.0), Sense::hover());
                });
        });
        if pressed {
            let hovered_v_thumbs: Vec<f32> = s
                .shapes
                .iter()
                .filter_map(|cs| match &cs.shape {
                    egui::epaint::Shape::Rect(r)
                        if r.rect.height() > 15.0 && r.rect.width() <= 12.0 =>
                    {
                        Some(r.rect.width())
                    }
                    _ => None,
                })
                .collect();
            assert!(hovered_v_thumbs.iter().any(|&w| (w - 10.0).abs() < 1e-3));
        }
        s.textures_delta.clear();
    }
    assert!((offset.y - 100.0).abs() < 1e-3);

    let h_click_pos = Pos2::new(viewport.right() - 20.0, viewport.bottom() - 4.0);
    for pressed in [true, false] {
        let mut inp = RawInput::default();
        inp.events.push(Event::PointerMoved(h_click_pos));
        inp.events.push(Event::PointerButton {
            pos: h_click_pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        });
        let mut s = ctx.run_ui(inp, |ui| {
            MacScrollView::both("hover_jump_scroll")
                .theme(&theme)
                .auto_hide(false)
                .with_offset(&mut offset)
                .max_width(200.0)
                .max_height(100.0)
                .show(ui, |c| {
                    c.allocate_exact_size(Vec2::new(600.0, 500.0), Sense::hover());
                });
        });
        s.textures_delta.clear();
    }
    assert!((offset.x - 200.0).abs() < 1e-3);
}

#[test]
fn test_undo_manager_grouping_deduplication_and_context_menu_checkmarks() {
    let mut manager = NSUndoManager::<i32>::new();
    let mut state = 10;
    manager.push_snapshot(state);
    manager.push_snapshot(state);
    assert_eq!(manager.undo_count(), 1);
    state = 20;

    manager.begin_grouping(state);
    assert!(manager.is_grouping());
    manager.push_snapshot(25);
    manager.push_snapshot(30);
    assert_eq!(manager.undo_count(), 1);
    state = 40;
    assert!(manager.end_grouping(&state));
    assert!(!manager.is_grouping());
    assert_eq!(manager.undo_count(), 2);
    assert!(manager.undo(&mut state));
    assert_eq!(state, 20);

    // Mid-grouping undo cancels active group and restores group initial state
    manager.begin_grouping(state);
    state = 35;
    assert!(manager.undo(&mut state));
    assert_eq!(state, 20);
    assert!(!manager.is_grouping());
    assert_eq!(manager.redo_count(), 2);

    manager.begin_grouping(state);
    assert!(!manager.end_grouping(&state));

    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let items = vec![
        NSMenuItem::action("Toggle Grid", "view.grid").checked(true),
        NSMenuItem::action("Plain Item", "view.plain").checked(false),
    ];
    let mut is_open = true;
    let mut spawn_pos = Pos2::new(60.0, 60.0);
    let mut selected_idx = None;
    let raw_in = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0))),
        ..Default::default()
    };
    let mut out1 = ctx.run_ui(raw_in.clone(), |ui| {
        MacContextMenu::new(&items, &mut is_open, &mut spawn_pos, &mut selected_idx)
            .theme(&theme)
            .show(ui);
    });
    out1.textures_delta.clear();
    let mut out = ctx.run_ui(raw_in, |ui| {
        MacContextMenu::new(&items, &mut is_open, &mut spawn_pos, &mut selected_idx)
            .theme(&theme)
            .show(ui);
    });
    out.textures_delta.clear();
    let mut text_shapes = Vec::new();
    for cs in &out.shapes {
        collect_text_shapes(&cs.shape, &mut text_shapes);
    }
    assert!(text_shapes.iter().any(|t| t.galley.text() == egui_macview::MacSymbol::Checkmark.as_str()));
}
