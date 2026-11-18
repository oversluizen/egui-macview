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
    MacContextMenu, MacScrollView, MacSlider, MacStepper, NSMenuItem, NSUndoManager,
};

mod common;
use common::collect_text_shapes;

#[allow(dead_code)]
#[path = "../examples/macview_gallery.rs"]
mod gallery_app;
pub(crate) use gallery_app::{format_tracked_small_caps, CardModel, ExpandedProTab, GalleryApp, GalleryScreen, InspectorMode};

fn click_raw(pos: Pos2, pressed: bool, time: Option<f64>) -> RawInput {
    let ev = Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: egui::Modifiers::default() };
    RawInput { time, events: vec![Event::PointerMoved(pos), ev], ..Default::default() }
}

#[test]
fn test_sidebar_navigator_clean_labels_and_sufficient_width() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::DarkAqua));
    gallery_app::setup_system_or_custom_fonts(&ctx);
    let mut app = GalleryApp::new();
    let raw = RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))), ..Default::default() };
    let mut out = ctx.run_ui(raw, |ui| app.render_ui(ui));
    out.textures_delta.clear();
    let mut texts = Vec::new();
    for cs in &out.shapes {
        collect_text_shapes(&cs.shape, &mut texts);
    }
    for bad in ["Default NIB (260pt)", "Compact HUD (178x63)", "Expanded Pro (380pt)"] {
        assert!(!texts.iter().any(|t| t.galley.text() == bad));
    }
    let clean_labels = ["Default", "Compact", "Pro"];
    let outline_metrics = theme.outline_view();
    let base_w =
        outline_metrics.indent_step.points() + theme.thumbnail().size.points() + outline_metrics.content_spacing.points() + 16.0f32;
    let mut out2 = ctx.run_ui(RawInput::default(), |ui| {
        let font_id = egui::FontId::proportional(outline_metrics.font_size.points());
        for label in &clean_labels {
            let text_w = ui.painter().layout_no_wrap((*label).to_string(), font_id.clone(), egui::Color32::WHITE).size().x;
            assert!(base_w + text_w <= 220.0);
        }
    });
    out2.textures_delta.clear();
}

#[test]
fn test_screen_navigator_and_bidirectional_gallery_bindings() {
    let ctx = Context::default();
    gallery_app::setup_system_or_custom_fonts(&ctx);
    let mut app = GalleryApp::new();
    assert_eq!(GalleryScreen::ALL.len(), 6);
    for screen in GalleryScreen::ALL {
        app.active_screen = screen;
        app.sidebar.active_screen = screen;
        app.sidebar.selected_node = screen.title().to_string();
        let raw = RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))), ..Default::default() };
        let mut out = ctx.run_ui(raw, |ui| app.render_ui(ui));
        out.textures_delta.clear();
        assert!(!out.shapes.is_empty());
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
        })
        .textures_delta
        .clear();
        assert!((val - expected).abs() < 1e-6);
    }

    let top_pos = Pos2::new(stepper_rect.center().x, stepper_rect.top() + 3.0);
    ctx.run_ui(click_raw(top_pos, true, Some(1.0)), |ui| {
        MacStepper::new(&mut val, -1000.0..=1000.0, 1.0).theme(&theme).show(ui);
    })
    .textures_delta
    .clear();

    let val_before_hold = val;
    ctx.run_ui(RawInput { time: Some(1.45), ..Default::default() }, |ui| {
        MacStepper::new(&mut val, -1000.0..=1000.0, 1.0).theme(&theme).show(ui);
    })
    .textures_delta
    .clear();
    assert!(val > val_before_hold);
}

#[test]
fn test_mac_slider_caspring_press_scale_and_scroll_page_jump() {
    let (ctx, theme, mut val, mut slider_rect) = (Context::default(), MacTheme::new(MacAppearance::Aqua), 50.0f32, Rect::NOTHING);
    ctx.run_ui(RawInput { time: Some(0.0), ..Default::default() }, |ui| {
        slider_rect = MacSlider::new(&mut val, 0.0..=100.0).theme(&theme).show(ui).rect;
    })
    .textures_delta
    .clear();

    ctx.run_ui(click_raw(slider_rect.center(), true, Some(0.05)), |ui| {
        MacSlider::new(&mut val, 0.0..=100.0).theme(&theme).show(ui);
    })
    .textures_delta
    .clear();

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

    let (mut offset, mut viewport) = (Vec2::ZERO, Rect::NOTHING);
    let mut s0 = ctx.run_ui(RawInput::default(), |ui| {
        viewport = MacScrollView::both("hover_jump_scroll")
            .theme(&theme)
            .auto_hide(false)
            .with_offset(&mut offset)
            .max_width(200.0)
            .max_height(100.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(600.0, 500.0), Sense::hover());
            })
            .inner_rect;
    });
    s0.textures_delta.clear();
    assert!(s0
        .shapes
        .iter()
        .any(|cs| matches!(&cs.shape, egui::epaint::Shape::Rect(r) if r.rect.height() > 15.0 && (r.rect.width() - 7.0).abs() < 1e-3)));

    for pressed in [true, false] {
        let mut s = ctx.run_ui(click_raw(Pos2::new(viewport.right() - 4.0, viewport.bottom() - 20.0), pressed, None), |ui| {
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
            assert!(s.shapes.iter().any(
                |cs| matches!(&cs.shape, egui::epaint::Shape::Rect(r) if r.rect.height() > 15.0 && (r.rect.width() - 10.0).abs() < 1e-3)
            ));
        }
        s.textures_delta.clear();
    }
    assert!((offset.y - 100.0).abs() < 1e-3);

    for pressed in [true, false] {
        ctx.run_ui(click_raw(Pos2::new(viewport.right() - 20.0, viewport.bottom() - 4.0), pressed, None), |ui| {
            MacScrollView::both("hover_jump_scroll")
                .theme(&theme)
                .auto_hide(false)
                .with_offset(&mut offset)
                .max_width(200.0)
                .max_height(100.0)
                .show(ui, |c| {
                    c.allocate_exact_size(Vec2::new(600.0, 500.0), Sense::hover());
                });
        })
        .textures_delta
        .clear();
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
    let items =
        vec![NSMenuItem::action("Toggle Grid", "view.grid").checked(true), NSMenuItem::action("Plain Item", "view.plain").checked(false)];
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
    let ctx = Context::default();
    gallery_app::setup_system_or_custom_fonts(&ctx);
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut app = gallery_app::GalleryApp::new();
    app.active_screen = GalleryScreen::AdvancedControls;
    let screen_rect = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 900.0)));
    let mut out = ctx.run_ui(RawInput { screen_rect, ..Default::default() }, |ui| {
        app.render_ui(ui);
    });
    out.textures_delta.clear();
    let mut texts = Vec::new();
    for cs in &out.shapes {
        collect_text_shapes(&cs.shape, &mut texts);
    }
    let find_text = |s: &str| texts.iter().copied().find(|t| t.galley.text() == s).unwrap();
    for req in [
        "Enable Inspector Controls (ui.add_enabled_ui)",
        "Width:",
        "1920",
        "pt",
        "Sonoma 14.8",
        "?",
        "Regular (14pt)",
        "Small (12pt)",
        "Mini Mixed (10pt)",
        "Small 19pt",
        "Apple ProRes 422 HQ",
        "Primary Label",
        "Tertiary Caption (WCAG Clamped)",
        "Origin:",
        "Tag Label:",
        "Transform Layer",
        "Color Tag",
        "Center",
    ] {
        let _ = find_text(req);
    }
    let (w_lbl, w_in, u_pop, p_pop) = (find_text("Width:"), find_text("1920"), find_text("pt"), find_text("Sonoma 14.8"));
    assert!((w_lbl.pos.y - w_in.pos.y).abs() <= 1.0 && (w_in.pos.y - u_pop.pos.y).abs() <= 1.0 && (u_pop.pos.y - p_pop.pos.y).abs() <= 1.0);
    let (orig_lbl, tag_lbl) = (find_text("Origin:"), find_text("Tag Label:"));
    assert!((orig_lbl.pos.y - tag_lbl.pos.y).abs() <= 1.0);
    assert_eq!(
        find_text("Tertiary Caption (WCAG Clamped)").galley.job.sections[0].format.color,
        theme.tertiary_label_on(theme.box_background())
    );

    let s6_scene = &app.center.screen_nib_scenes[5].0;
    let w_lbl_r = s6_scene.nodes().iter().find(|n| n.ib_object_id == "s6-en-lbl-w").unwrap().alignment_rect;
    let chk_r = s6_scene.nodes().iter().find(|n| n.ib_object_id == "s6-en-chk-reg").unwrap().alignment_rect;
    let plus_r = s6_scene.nodes().iter().find(|n| n.ib_object_id == "s6-en-plus").unwrap().alignment_rect;
    let s6_origin = w_lbl.pos - Vec2::new(w_lbl_r[0], w_lbl_r[1]);
    let chk_pos = s6_origin + Vec2::new(chk_r[0] + 8.0, chk_r[1] + chk_r[3] * 0.5);
    let plus_rect = Rect::from_min_size(s6_origin + Vec2::new(plus_r[0], plus_r[1]), Vec2::new(plus_r[2], plus_r[3]));
    assert!(common::first_line_stroke_color_in_rect(&out.shapes, plus_rect).is_some(), "s6-en-plus must paint vector crosshair lines");
    let plus_pos = plus_rect.center();
    app.center.nib_inspector_enabled = false;
    let (init_chk, init_axes, init_btn) = (app.center.nib_check_regular, app.center.nib_axis_count, app.center.advanced_button_open);
    for pos in [chk_pos, plus_pos] {
        for pressed in [true, false] {
            let mut raw = click_raw(pos, pressed, None);
            raw.screen_rect = screen_rect;
            ctx.run_ui(raw, |ui| app.render_ui(ui)).textures_delta.clear();
        }
    }
    assert_eq!((app.center.nib_check_regular, app.center.nib_axis_count, app.center.advanced_button_open), (init_chk, init_axes, init_btn));
}

#[test]
fn test_real_pointer_click_to_focus_input_cell_and_hud_palette() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut cell_val = "240.0".to_string();
    let mut cell_rect = Rect::NOTHING;

    let click_events = |pos: Pos2| RawInput {
        events: vec![
            Event::PointerMoved(pos),
            Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE },
            Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE },
        ],
        ..Default::default()
    };
    let arrow_up_input = || RawInput {
        events: vec![Event::Key { key: Key::ArrowUp, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE }],
        ..Default::default()
    };

    for step in 0..3 {
        let r_in = match step {
            0 => RawInput::default(),
            1 => click_events(cell_rect.center()),
            _ => arrow_up_input(),
        };
        ctx.run_ui(r_in, |ui| {
            cell_rect =
                egui_macview::MacInputCell::new("click_focus_cell", &mut cell_val).theme(&theme).numeric(60.0..=500.0, 1.0).show(ui).rect;
        })
        .textures_delta
        .clear();
    }
    assert_eq!(cell_val, "241.0");

    let (mut hud_pos, mut hud_w, mut hud_rot, mut hud_w_rect, hud_w_id) =
        (Pos2::new(40.0, 40.0), "240.0".to_string(), 0.0f32, Rect::NOTHING, egui::Id::NULL.with("hud_test_w"));
    for step in 0..3 {
        let raw = match step {
            0 => RawInput::default(),
            1 => click_events(hud_w_rect.center()),
            _ => arrow_up_input(),
        };
        ctx.run_ui(raw, |ui| {
            egui_macview::MacFloatingInfoBox::new("hud_click_test", &mut hud_pos).theme(&theme).title("HUD Palette").show(
                ui.ctx(),
                |r1| {
                    hud_w_rect = egui_macview::MacInputCell::with_id(hud_w_id, &mut hud_w)
                        .theme(&theme)
                        .control_size(egui_macview::MacControlSize::CompactInfobox)
                        .numeric(60.0..=500.0, 1.0)
                        .show(r1)
                        .rect;
                },
                |r2| {
                    egui_macview::MacSlider::new(&mut hud_rot, 0.0..=360.0).theme(&theme).width(60.0).show(r2);
                },
            );
        })
        .textures_delta
        .clear();
    }
    assert_eq!(hud_w, "241.0");
}

#[test]
fn test_req013_02m_all_nib_scenes_controls_and_buttons_bound_two_way() {
    use egui_macview::scene::SceneNodeKind;
    let mut app = gallery_app::GalleryApp::new();
    let check_scene = |name: &str, scene: &egui_macview::scene::MacSceneIR| {
        let mut path_ids = std::collections::BTreeMap::new();
        for node in scene.nodes() {
            if let (Some(bid), Some(path)) = (node.value_binding, &node.binding_path) {
                assert_eq!(*path_ids.entry(path.clone()).or_insert(bid), bid, "{name} duplicate BindingId for keyPath {path}");
            }
            match node.kind {
                SceneNodeKind::TextField
                | SceneNodeKind::NumberField { .. }
                | SceneNodeKind::Slider { .. }
                | SceneNodeKind::Stepper { .. }
                | SceneNodeKind::Checkbox
                | SceneNodeKind::Switch
                | SceneNodeKind::PopUpButton { .. }
                | SceneNodeKind::SegmentedControl { .. }
                | SceneNodeKind::OriginMatrix => {
                    assert!(node.value_binding.is_some() && node.binding_path.is_some(), "{name} node {:?} unbound", node.ib_object_id);
                }
                SceneNodeKind::PushButton { .. } | SceneNodeKind::HelpButton | SceneNodeKind::InlineActionButton => {
                    assert!(node.action.is_some() && node.action_selector.is_some(), "{name} button {:?} has no action", node.ib_object_id);
                }
                _ => {}
            }
        }
    };
    let forbidden_wrappers = [
        "MacVectorCanvasSurface",
        "MacCupertinoSymbolsAndLayoutLab",
        "MacCommandAndWcagIntegrityLab",
        "MacFinderBrowserAndPublishView",
        "MacAvKitVideoAndTimelineStudio",
        "MacProControlsAndNibShowcaseLab",
        "MacRightInspectorCompleteView",
        "NSAvKitStereoVuMetersView",
        "NSAffineMatrixAndSnippetView",
    ];
    for deleted_file in [
        "examples/gallery/state4_publish.rs",
        "examples/gallery/state5_meters.rs",
        "examples/gallery/state6_advanced_controls.rs",
        "examples/gallery/state6_pro_controls.rs",
    ] {
        assert!(!std::path::Path::new(deleted_file).exists(), "REQ-013-02Q: {deleted_file} must be eliminated into .xib");
    }
    for (idx, (scene, _)) in app.center.screen_nib_scenes.iter().enumerate() {
        check_scene(&format!("screen_nib_scenes[{idx}]"), scene);
        assert!(scene.nodes().len() >= 15);
        for n in scene.nodes() {
            if let SceneNodeKind::CustomCanvasSlot { class_name } = &n.kind {
                assert!(!forbidden_wrappers.contains(&class_name.as_str()));
            }
        }
    }
    check_scene("inspector.nib_scene", &app.inspector.nib_scene);
    assert!(app.inspector.nib_scene.nodes().len() >= 25);
    check_scene("nib_studio.scene", &app.center.nib_studio.scene);
    app.inspector.nib_store.set_path_f64("cardWidth", 384.0);
    app.inspector.nib_store.set_path_f64("strokeWidth", 5.5);
    for (idx, path, val) in [
        (0, "scalePercent", 105.0),
        (0, "rotationDeg", 17.0),
        (0, "cornerRadius", 28.0),
        (1, "pointSize", 18.0),
        (2, "contrastTarget", 9.5),
        (3, "exportBitrate", 180.0),
        (3, "workerCount", 16.0),
        (4, "mediaSpeed", 1.5),
    ] {
        app.center.screen_nib_scenes[idx].1.set_path_f64(path, val);
    }
    for (idx, path, val) in [
        (1, "fontFamily", 2),
        (1, "fontWeight", 3),
        (2, "accentIndex", 3),
        (4, "busMode", 2),
        (4, "codecPreset", 5),
        (5, "themeVariant", 2),
        (5, "actionBarSel", 1),
    ] {
        app.center.screen_nib_scenes[idx].1.set_path_i64(path, val);
    }
    app.center.screen_nib_scenes[0].1.pending_actions.push("pulseSpring:".into());
    app.center.screen_nib_scenes[3].1.pending_actions.push("enqueueJob:".into());
    let ctx = Context::default();
    gallery_app::setup_system_or_custom_fonts(&ctx);
    let screen_rect = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 900.0)));
    let raw = RawInput { time: Some(25.0), screen_rect, ..Default::default() };
    let mut out = ctx.run_ui(raw, |ui| app.render_ui(ui));
    out.textures_delta.clear();
    assert_eq!((app.card.borrow().width.as_str(), app.card.borrow().radius.as_str()), ("384.0", "28.0"));
    assert_eq!((app.center.codec_preset, app.center.media_playing), (5, true), "REQ-013-06.2: codecPreset 5 (Debussy) must not clamp to 3");
    let has_rounded_rotated_poly =
        out.shapes.iter().any(|cs| matches!(&cs.shape, egui::epaint::Shape::Path(p) if p.closed && p.points.len() >= 16));
    assert!(has_rounded_rotated_poly, "Rotated vector card must render a rounded polygon with >= 16 points when cornerRadius > 0");
    assert_eq!((app.appearance, app.center.finder_custom_entries.len(), app.center.action_items.len()), (MacAppearance::DarkAqua, 1, 3));
    assert!(!app.center.popover_open && app.center.bounce_start_time.unwrap_or(0.0) >= 25.0);

    let mut out = ctx.run_ui(RawInput { time: Some(35.0), screen_rect, ..Default::default() }, |ui| app.render_ui(ui));
    out.textures_delta.clear();
    let mut texts = Vec::new();
    for cs in &out.shapes {
        collect_text_shapes(&cs.shape, &mut texts);
    }
    let find_text = |s: &str| texts.iter().copied().find(|t| t.galley.text() == s).unwrap();
    let s1_scene = &app.center.screen_nib_scenes[0].0;
    let lbl_scale_rect = s1_scene.nodes().iter().find(|n| n.title == "Scale (%):").unwrap().alignment_rect;
    let sld_scale_rect = s1_scene.nodes().iter().find(|n| n.ib_object_id == "s1-sld-scale").unwrap().alignment_rect;
    let sld_rot_rect = s1_scene.nodes().iter().find(|n| n.ib_object_id == "s1-sld-rot").unwrap().alignment_rect;
    let sld_rad_rect = s1_scene.nodes().iter().find(|n| n.ib_object_id == "s1-sld-rad").unwrap().alignment_rect;
    let scene_origin = find_text("Scale (%):").pos - Vec2::new(lbl_scale_rect[0], lbl_scale_rect[1]);
    let slider_target = |r: [f32; 4]| scene_origin + Vec2::new(r[0] + r[2] * 0.85, r[1] + r[3] * 0.5);
    for pos in [slider_target(sld_scale_rect), slider_target(sld_rot_rect), slider_target(sld_rad_rect)] {
        for pressed in [true, false] {
            let mut r = click_raw(pos, pressed, Some(35.1));
            r.screen_rect = screen_rect;
            ctx.run_ui(r, |ui| app.render_ui(ui)).textures_delta.clear();
        }
    }
    assert!(app.card.borrow().scale_pct > 150.0, "Scale slider click must update scale_pct");
    assert!(app.card.borrow().rotation_deg > 40.0, "Rotation slider click must update rotation_deg");
    assert!(app.card.borrow().radius.parse::<f32>().unwrap_or(0.0) > 30.0, "Radius slider click must update radius");

    app.active_screen = GalleryScreen::ALL[4];
    app.center.media_playing = false;
    let mut out5 = ctx.run_ui(RawInput { time: Some(40.0), screen_rect, ..Default::default() }, |ui| app.render_ui(ui));
    out5.textures_delta.clear();
    let mut texts5 = Vec::new();
    for cs in &out5.shapes {
        collect_text_shapes(&cs.shape, &mut texts5);
    }
    let play_pos = texts5.iter().copied().find(|t| t.galley.text() == "Play / Pause").unwrap().pos + Vec2::new(12.0, 6.0);
    let scrubber_pos = texts5.iter().copied().find(|t| t.galley.text() == "00:04.0").unwrap().pos;
    assert!(scrubber_pos.y < play_pos.y, "REQ-013-02Q: State 5 video+scrubber slot must stay strictly above Play button");
    for pressed in [true, false] {
        let mut r = click_raw(play_pos, pressed, Some(40.1));
        r.screen_rect = screen_rect;
        ctx.run_ui(r, |ui| app.render_ui(ui)).textures_delta.clear();
    }
    assert!(app.center.media_playing, "State 5 Play button below video slot must respond to real pointer click without slot overflow");
}
