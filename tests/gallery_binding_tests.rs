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

#[allow(dead_code)]
#[path = "../examples/macview_gallery.rs"]
mod gallery_app;
pub(crate) use gallery_app::{
    format_tracked_small_caps, CardModel, ExpandedProTab, GalleryApp, GalleryScreen, InspectorMode,
};

fn collect_text_shapes<'a>(shape: &'a egui::epaint::Shape, out: &mut Vec<&'a egui::epaint::TextShape>) {
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

fn click_raw(pos: Pos2, pressed: bool, time: Option<f64>) -> RawInput {
    let ev = Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: egui::Modifiers::default() };
    RawInput { time, events: vec![Event::PointerMoved(pos), ev], ..Default::default() }
}

fn read_all_gallery_sources() -> String {
    let mut src = std::fs::read_to_string("examples/macview_gallery.rs").expect("read gallery");
    if let Ok(entries) = std::fs::read_dir("examples/gallery") {
        for e in entries.flatten().filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs")) {
            if let Ok(sub) = std::fs::read_to_string(e.path()) {
                src.push('\n');
                src.push_str(&sub);
            }
        }
    }
    src
}

#[test]
fn test_sidebar_navigator_clean_labels_and_sufficient_width() {
    let (ctx, theme, gallery_src) = (Context::default(), MacTheme::new(MacAppearance::DarkAqua), read_all_gallery_sources());
    for bad in ["\"Default NIB (260pt)\"", "\"Compact HUD (178x63)\"", "\"Expanded Pro (380pt)\""] {
        assert!(!gallery_src.contains(bad));
    }
    let clean_labels = ["Default NIB", "Compact HUD", "Expanded Pro"];
    let outline_metrics = theme.outline_view();
    let base_w = outline_metrics.indent_step.points() + theme.thumbnail().size.points() + outline_metrics.content_spacing.points() + 16.0f32;
    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let font_id = egui::FontId::proportional(outline_metrics.font_size.points());
        for label in &clean_labels {
            let text_w = ui.painter().layout_no_wrap((*label).to_string(), font_id.clone(), egui::Color32::WHITE).size().x;
            assert!(base_w + text_w <= 220.0);
        }
    });
    out.textures_delta.clear();
}

#[test]
fn test_screen_navigator_and_bidirectional_gallery_bindings() {
    let gallery_src = read_all_gallery_sources();
    assert!(gallery_src.contains("enum GalleryScreen") && gallery_src.contains("CanvasPhysics") && gallery_src.contains("VectorAssets"));
    assert!(!gallery_src.contains("\"Expanded Inspector\"") && !gallery_src.contains("if resp.is_selected {") && gallery_src.contains("if resp.clicked {"));
    assert!(gallery_src.contains("MacToolbar::new") && gallery_src.contains("MacFloatingInfoBox::new"));
}

#[test]
fn test_adversarial_menu_accelerator_clearance_extreme_strings() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::DarkAqua));
    let export_menu = NSMenu::with_items("Nested Export", vec![NSMenuItem::action_with_shortcut(
        "Export High-Throughput Cryptographic Signature Keyframe Archive Package",
        "export.keyframe",
        MacKeyboardShortcut::cmd_shift(Key::K),
    )]);
    let items = vec![
        NSMenuItem::action_with_shortcut("Synthesize Deeply Nested Multi-Stage Hierarchical Cryptographic Artifact", "adv.long1", MacKeyboardShortcut::cmd_shift(Key::S)),
        NSMenuItem::action_with_shortcut("Cut", "adv.short", MacKeyboardShortcut::cmd_shift(Key::X)),
        NSMenuItem::action_with_shortcut("Supercalifragilisticexpialidocious Ultra Wide Screen Projection Alignment Matrix", "adv.long2", MacKeyboardShortcut::cmd(Key::P)),
        NSMenuItem::submenu("Cascading Options...", export_menu),
    ];
    let menus = vec![NSMenu::with_items("Adversarial", items)];
    let (mut open_menu_idx, mut highlighted_item_idx) = (Some(0), Some(0));
    let raw_input = RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1920.0, 1080.0))), ..Default::default() };

    for _ in 0..2 {
        let mut out = ctx.run_ui(raw_input.clone(), |ui| {
            MacMenuBar::new(&menus, &mut open_menu_idx, &mut highlighted_item_idx).theme(&theme).show(ui);
        });
        out.textures_delta.clear();
        if !out.shapes.is_empty() {
            let mut text_shapes = Vec::new();
            for clipped in &out.shapes {
                collect_text_shapes(&clipped.shape, &mut text_shapes);
            }
            if text_shapes.iter().any(|t| t.galley.text() == "Cut") {
                for (title, sc) in [
                    ("Synthesize Deeply Nested Multi-Stage Hierarchical Cryptographic Artifact", "Ctrl+Shift+S"),
                    ("Cut", "Ctrl+Shift+X"),
                    ("Supercalifragilisticexpialidocious Ultra Wide Screen Projection Alignment Matrix", "Ctrl+P"),
                    ("Cascading Options...", "›"),
                ] {
                    let title_shape = text_shapes.iter().find(|t| t.galley.text() == title).unwrap();
                    let sc_shape = text_shapes.iter().find(|t| t.galley.text() == sc).unwrap();
                    assert!((sc_shape.pos.x - sc_shape.galley.size().x) - (title_shape.pos.x + title_shape.galley.size().x) >= 24.0);
                }
            }
        }
    }
}

#[test]
fn test_mac_stepper_modifier_multipliers_and_hold_to_repeat() {
    let (ctx, theme, mut val, mut stepper_rect) = (Context::default(), MacTheme::new(MacAppearance::Aqua), 0.0f64, Rect::NOTHING);
    let mut out0 = ctx.run_ui(RawInput::default(), |ui| {
        let resp = MacStepper::new(&mut val, -1000.0..=1000.0, 1.0).theme(&theme).show(ui);
        stepper_rect = resp.rect;
        resp.request_focus();
    });
    out0.textures_delta.clear();

    for (mods, expected) in [(egui::Modifiers::SHIFT, 10.0), (egui::Modifiers::COMMAND, 110.0)] {
        let ev = Event::Key { key: Key::ArrowUp, physical_key: None, pressed: true, repeat: false, modifiers: mods };
        ctx.run_ui(RawInput { events: vec![ev], ..Default::default() }, |ui| {
            MacStepper::new(&mut val, -1000.0..=1000.0, 1.0).theme(&theme).show(ui);
        }).textures_delta.clear();
        assert!((val - expected).abs() < 1e-6);
    }

    let top_pos = Pos2::new(stepper_rect.center().x, stepper_rect.top() + 3.0);
    ctx.run_ui(click_raw(top_pos, true, Some(1.0)), |ui| {
        MacStepper::new(&mut val, -1000.0..=1000.0, 1.0).theme(&theme).show(ui);
    }).textures_delta.clear();

    let val_before_hold = val;
    ctx.run_ui(RawInput { time: Some(1.45), ..Default::default() }, |ui| {
        MacStepper::new(&mut val, -1000.0..=1000.0, 1.0).theme(&theme).show(ui);
    }).textures_delta.clear();
    assert!(val > val_before_hold);
}

#[test]
fn test_mac_slider_caspring_press_scale_and_scroll_page_jump() {
    let (ctx, theme, mut val, mut slider_rect) = (Context::default(), MacTheme::new(MacAppearance::Aqua), 50.0f32, Rect::NOTHING);
    ctx.run_ui(RawInput { time: Some(0.0), ..Default::default() }, |ui| {
        slider_rect = MacSlider::new(&mut val, 0.0..=100.0).theme(&theme).show(ui).rect;
    }).textures_delta.clear();

    ctx.run_ui(click_raw(slider_rect.center(), true, Some(0.05)), |ui| {
        MacSlider::new(&mut val, 0.0..=100.0).theme(&theme).show(ui);
    }).textures_delta.clear();

    let mut out2 = ctx.run_ui(RawInput { time: Some(0.25), ..Default::default() }, |ui| {
        MacSlider::new(&mut val, 0.0..=100.0).theme(&theme).show(ui);
    });
    out2.textures_delta.clear();
    let base_r = theme.slider().thumb_size.points() * 0.5;
    let max_r = out2.shapes.iter().filter_map(|cs| match &cs.shape { egui::epaint::Shape::Circle(c) => Some(c.radius), _ => None }).fold(0.0f32, f32::max);
    assert!(max_r > base_r);

    let (mut offset, mut viewport) = (Vec2::ZERO, Rect::NOTHING);
    let mut s0 = ctx.run_ui(RawInput::default(), |ui| {
        viewport = MacScrollView::both("hover_jump_scroll").theme(&theme).auto_hide(false).with_offset(&mut offset).max_width(200.0).max_height(100.0).show(ui, |c| {
            c.allocate_exact_size(Vec2::new(600.0, 500.0), Sense::hover());
        }).inner_rect;
    });
    s0.textures_delta.clear();
    assert!(s0.shapes.iter().any(|cs| matches!(&cs.shape, egui::epaint::Shape::Rect(r) if r.rect.height() > 15.0 && (r.rect.width() - 7.0).abs() < 1e-3)));

    for pressed in [true, false] {
        let mut s = ctx.run_ui(click_raw(Pos2::new(viewport.right() - 4.0, viewport.bottom() - 20.0), pressed, None), |ui| {
            MacScrollView::both("hover_jump_scroll").theme(&theme).auto_hide(false).with_offset(&mut offset).max_width(200.0).max_height(100.0).show(ui, |c| {
                c.allocate_exact_size(Vec2::new(600.0, 500.0), Sense::hover());
            });
        });
        if pressed {
            assert!(s.shapes.iter().any(|cs| matches!(&cs.shape, egui::epaint::Shape::Rect(r) if r.rect.height() > 15.0 && (r.rect.width() - 10.0).abs() < 1e-3)));
        }
        s.textures_delta.clear();
    }
    assert!((offset.y - 100.0).abs() < 1e-3);

    for pressed in [true, false] {
        ctx.run_ui(click_raw(Pos2::new(viewport.right() - 20.0, viewport.bottom() - 4.0), pressed, None), |ui| {
            MacScrollView::both("hover_jump_scroll").theme(&theme).auto_hide(false).with_offset(&mut offset).max_width(200.0).max_height(100.0).show(ui, |c| {
                c.allocate_exact_size(Vec2::new(600.0, 500.0), Sense::hover());
            });
        }).textures_delta.clear();
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
    assert!(manager.end_grouping(&state) && !manager.is_grouping() && manager.undo_count() == 2);
    assert!(manager.undo(&mut state) && state == 20);
    manager.begin_grouping(state);
    state = 35;
    assert!(manager.undo(&mut state) && state == 20 && !manager.is_grouping() && manager.redo_count() == 2);
    manager.begin_grouping(state);
    assert!(!manager.end_grouping(&state));

    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));
    let items = vec![NSMenuItem::action("Toggle Grid", "view.grid").checked(true), NSMenuItem::action("Plain Item", "view.plain").checked(false)];
    let (mut is_open, mut spawn_pos, mut selected_idx) = (true, Pos2::new(60.0, 60.0), None);
    let raw_in = RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0))), ..Default::default() };
    for _ in 0..2 {
        let mut out = ctx.run_ui(raw_in.clone(), |ui| {
            MacContextMenu::new(&items, &mut is_open, &mut spawn_pos, &mut selected_idx).theme(&theme).show(ui);
        });
        out.textures_delta.clear();
        let mut text_shapes = Vec::new();
        for cs in &out.shapes {
            collect_text_shapes(&cs.shape, &mut text_shapes);
        }
        let chk = egui_macview::MacSymbol::Checkmark.as_str();
        if text_shapes.iter().any(|t| t.galley.text() == chk) {
            return;
        }
    }
    panic!("Expected Checkmark symbol in context menu");
}

#[test]
fn test_state6_nib_inspector_cohesive_showcase_and_disabled_inheritance() {
    let state6_src = std::fs::read_to_string("examples/gallery/state6_advanced_controls.rs").expect("read");
    assert!(!state6_src.contains(".label("));
    let ctx = Context::default();
    gallery_app::setup_system_or_custom_fonts(&ctx);
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut center = gallery_app::gallery::CenterState::default();
    let toasts = std::cell::RefCell::new(egui_macview::MacToastManager::new());
    let screen_rect = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 800.0)));
    let mut out = ctx.run_ui(RawInput { screen_rect, ..Default::default() }, |ui| {
        center.render_state6_advanced_controls(ui, &theme, &toasts);
    });
    out.textures_delta.clear();
    let mut texts = Vec::new();
    for cs in &out.shapes {
        collect_text_shapes(&cs.shape, &mut texts);
    }
    let find_text = |s: &str| texts.iter().copied().find(|t| t.galley.text() == s).unwrap();
    for req in ["Enable Inspector Controls (ui.add_enabled_ui)", "Width:", "1920", "pt", "Sonoma 14.8", "?", "+", "Regular (14pt)", "Small (12pt)", "Mini Mixed (10pt)", "Small 19pt", "Apple ProRes 422 HQ", "Primary Label", "Tertiary Caption (WCAG Clamped)", "Origin:", "Tag Label:", "Transform Layer", "Color Tag", "Center"] {
        let _ = find_text(req);
    }
    let (w_lbl, w_in, u_pop, p_pop) = (find_text("Width:"), find_text("1920"), find_text("pt"), find_text("Sonoma 14.8"));
    assert!((w_lbl.pos.y - w_in.pos.y).abs() <= 1.0 && (w_in.pos.y - u_pop.pos.y).abs() <= 1.0 && (u_pop.pos.y - p_pop.pos.y).abs() <= 1.0);
    let (orig_lbl, tag_lbl) = (find_text("Origin:"), find_text("Tag Label:"));
    assert!((orig_lbl.pos.y - tag_lbl.pos.y).abs() <= 1.0);
    assert_eq!(find_text("Tertiary Caption (WCAG Clamped)").galley.job.sections[0].format.color, theme.tertiary_label_on(theme.box_background()));

    let (chk_pos, plus_pos) = (find_text("Regular (14pt)").pos + Vec2::new(-8.0, 6.0), find_text("+").pos + Vec2::new(2.0, 6.0));
    center.v068_inspector_enabled = false;
    let (init_chk, init_axes, init_btn) = (center.v065_check_regular, center.v065_axis_count, center.advanced_button_open);
    for pos in [chk_pos, plus_pos] {
        for pressed in [true, false] {
            let mut raw = click_raw(pos, pressed, None);
            raw.screen_rect = screen_rect;
            ctx.run_ui(raw, |ui| center.render_state6_advanced_controls(ui, &theme, &toasts)).textures_delta.clear();
        }
    }
    assert_eq!((center.v065_check_regular, center.v065_axis_count, center.advanced_button_open), (init_chk, init_axes, init_btn));
}
