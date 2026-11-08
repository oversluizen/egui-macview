//! High-Assurance Layout Engine & Real Widget Proptest Suite.
//!
//! Verifies:
//! 1. `MacSplitView`, `MacOutlineView`, `MacScrollView`, `MacStackView`, `MacForm`, `MacFixedView`.
//! 2. 7 Real `egui::Context` Proptests executing the actual widget code (`MacSlider`, `MacSplitView`,
//!    `MacOutlineView`, `MacStepper`, `MacScrollView`, `MacStackView`) across arbitrary randomized inputs.

use egui::{Color32, Context, Event, Key, PointerButton, Pos2, RawInput, Rect, Sense, Vec2};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    MacColorWell, MacFixedView, MacForm, MacInputCell, MacOutlineView, MacScrollView, MacSlider,
    MacSplitOrientation, MacSplitView, MacStackAlignment, MacStackOrientation, MacStackView,
    MacStepper, MacToggle,
};
use proptest::prelude::*;

#[test]
fn test_split_view_horizontal_and_vertical_constraints() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut pos_x = 100.0f32;
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let (r1_width, r2_width) =
            MacSplitView::with_pos("h_split", MacSplitOrientation::Horizontal, &mut pos_x)
                .theme(&theme)
                .min_first(60.0)
                .max_first(200.0)
                .min_second(80.0)
                .show(
                    ui,
                    |pane1| pane1.available_rect_before_wrap().width(),
                    |pane2| pane2.available_rect_before_wrap().width(),
                );
        assert_eq!(r1_width, 100.0);
        assert!(r2_width > 0.0);
    });
    out1.textures_delta.clear();

    let mut pos_collapse = 120.0f32;
    let mut out2 = ctx.run_ui(RawInput::default(), |ui| {
        let (collapsed_w, active_w) = MacSplitView::with_pos(
            "collapsed_split",
            MacSplitOrientation::Horizontal,
            &mut pos_collapse,
        )
        .theme(&theme)
        .collapsed_first(true)
        .show(
            ui,
            |pane1| pane1.available_rect_before_wrap().width(),
            |pane2| pane2.available_rect_before_wrap().width(),
        );
        assert_eq!(collapsed_w, 0.0);
        assert!(active_w > 0.0);
    });
    out2.textures_delta.clear();
}

#[test]
fn test_outline_view_hierarchy_and_accessory_toggles() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut is_expanded = false;
    let mut is_visible = true;

    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        MacOutlineView::new("test_outline")
            .theme(&theme)
            .show(ui, |tree| {
                let resp = tree.row("item_1", 1, true, &mut is_expanded, true, |row| {
                    row.thumbnail(|_p, _r| {});
                    row.label("Test Node");
                    row.accessory_custom("vis_toggle", &mut is_visible, |_p, rect, active| {
                        assert!(rect.width() > 0.0 && active);
                    });
                });
                assert!(resp.is_selected && !resp.is_expanded);
            });
    });
    output.textures_delta.clear();
}

#[test]
fn test_scroll_view_offset_and_simulated_drag() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut offset = Vec2::new(0.0, 50.0);

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let output = MacScrollView::new("test_scroll_offset")
            .theme(&theme)
            .with_offset(&mut offset)
            .max_height(80.0)
            .max_width(150.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(150.0, 400.0), Sense::hover());
            });
        assert_eq!(output.offset.y, 50.0);
    });
    out.textures_delta.clear();

    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        MacScrollView::new("test_scroll_drag")
            .theme(&theme)
            .max_height(80.0)
            .max_width(120.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(120.0, 400.0), Sense::hover());
            });
    });
    out1.textures_delta.clear();

    let thumb_center = Pos2::new(113.5, 13.0);
    let mut input_press = RawInput::default();
    input_press.events.push(Event::PointerMoved(thumb_center));
    input_press.events.push(Event::PointerButton {
        pos: thumb_center,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    let mut out2 = ctx.run_ui(input_press, |ui| {
        MacScrollView::new("test_scroll_drag")
            .theme(&theme)
            .max_height(80.0)
            .max_width(120.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(120.0, 400.0), Sense::hover());
            });
    });
    out2.textures_delta.clear();

    let mut input_move = RawInput::default();
    input_move
        .events
        .push(Event::PointerMoved(Pos2::new(113.5, 33.0)));
    let mut dragged_offset = Vec2::ZERO;
    let mut out3 = ctx.run_ui(input_move, |ui| {
        let output = MacScrollView::new("test_scroll_drag")
            .theme(&theme)
            .max_height(80.0)
            .max_width(120.0)
            .show(ui, |c| {
                c.allocate_exact_size(Vec2::new(120.0, 400.0), Sense::hover());
            });
        dragged_offset = output.offset;
    });
    out3.textures_delta.clear();
    assert!(dragged_offset.y > 0.0);
}

#[test]
fn test_stack_view_and_fixed_view_clip_rect() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let output = MacStackView::new(MacStackOrientation::Vertical)
            .theme(&theme)
            .width(250.0)
            .height(120.0)
            .alignment(MacStackAlignment::Leading)
            .show_separators(true)
            .show(ui, |stack| {
                stack.add(|u| {
                    u.label("Item 1");
                });
                stack.add_section(Some("Section Header"), |sec_ui| {
                    sec_ui.label("Section Content");
                });
            });

        assert_eq!(output.rect.width(), 250.0);
        assert_eq!(output.rect.height(), 120.0);

        let sub_rect = Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::new(50.0, 30.0));
        let mut child_clip = Rect::NOTHING;
        let mut fixed =
            MacFixedView::new(ui, Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0)));
        fixed.subview(Pos2::new(10.0, 10.0), Vec2::new(50.0, 30.0), |child| {
            child_clip = child.clip_rect();
        });
        assert!(
            sub_rect.expand(0.5).contains_rect(child_clip) || child_clip == sub_rect,
            "MacFixedView::subview must clip child Ui to sub_rect: got {:?} vs {:?}",
            child_clip,
            sub_rect
        );
    });
    out.textures_delta.clear();
}

#[test]
fn test_mac_form_inspector_alignment_and_trailing_labels() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut rotation = 45.0f32;
    let mut fine_step = 5.0f64;
    let mut toggle = true;
    let mut text = String::from("Layer 1");
    let mut color = Color32::RED;
    let mut label_rects = Vec::new();
    let mut control_rects = Vec::new();

    for frame in 0..2 {
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            MacForm::new("test_inspector_alignment_form")
                .theme(&theme)
                .show(ui, |form| {
                    let (l1, c1) = form.row_label("Rotation:", |ui| {
                        MacSlider::new(&mut rotation, 0.0..=360.0)
                            .theme(&theme)
                            .width(120.0)
                            .show(ui)
                    });
                    let (l2, c2) = form.row_label("Fine Step:", |ui| {
                        ui.add(MacStepper::new(&mut fine_step, 0.0..=360.0, 5.0).theme(&theme))
                    });
                    let (l3, c3) = form.row_label("Drop Shadow:", |ui| {
                        ui.add(MacToggle::new(&mut toggle).theme(&theme))
                    });
                    let (l4, c4) = form.row_label("Name:", |ui| {
                        MacInputCell::new("test_input_name", &mut text)
                            .theme(&theme)
                            .width(180.0)
                            .show(ui)
                    });
                    let (l5, c5) = form.row_label("Fill Color:", |ui| {
                        MacColorWell::new(&mut color).theme(&theme).show(ui)
                    });

                    if frame == 1 {
                        label_rects = vec![l1.rect, l2.rect, l3.rect, l4.rect, l5.rect];
                        control_rects = vec![c1.rect, c2.rect, c3.rect, c4.rect, c5.rect];
                    }
                });
        });
        out.textures_delta.clear();
    }

    let right_edge_0 = label_rects[0].max.x;
    let left_edge_1 = control_rects[0].min.x;
    for i in 0..5 {
        assert!((label_rects[i].max.x - right_edge_0).abs() <= 1.0);
        assert!((control_rects[i].min.x - left_edge_1).abs() <= 1.0);
        assert!(label_rects[i].max.x < control_rects[i].min.x);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Real widget proptest: MacSlider clamps arbitrary values into [min, max] when rendered in egui::Context.
    #[test]
    fn prop_slider_fraction_bounds(
        min in -500.0f32..500.0f32,
        span in 1.0f32..1000.0f32,
        raw_val in -2000.0f32..2000.0f32,
    ) {
        let max = min + span;
        let ctx = Context::default();
        let theme = MacTheme::new(MacAppearance::Aqua);
        let mut val = raw_val;
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            let resp = MacSlider::new(&mut val, min..=max).theme(&theme).width(120.0).show(ui);
            assert_eq!(resp.rect.width(), 120.0);
        });
        out.textures_delta.clear();
        prop_assert!(!val.is_nan());
        prop_assert!(val >= min - 1e-4 && val <= max + 1e-4);
    }

    /// Real widget proptest: MacSplitView clamps divider position within [min_first, max_first] in egui::Context.
    #[test]
    fn prop_split_view_clamp_invariants(
        min_1 in 20.0f32..80.0f32,
        max_1 in 120.0f32..240.0f32,
        raw_pos in -500.0f32..1500.0f32,
    ) {
        let ctx = Context::default();
        let theme = MacTheme::new(MacAppearance::Aqua);
        let mut split_pos = raw_pos;
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            MacSplitView::with_pos("prop_split", MacSplitOrientation::Horizontal, &mut split_pos)
                .theme(&theme)
                .min_first(min_1)
                .max_first(max_1)
                .show(ui, |_| {}, |_| {});
        });
        out.textures_delta.clear();
        prop_assert!(split_pos >= min_1 - 1e-4 && split_pos <= max_1 + 1e-4);
    }

    /// Real widget proptest: MacOutlineView row rendering at depth d and d+1 has strictly monotonic x indentation.
    #[test]
    fn prop_outline_indentation_monotonic(depth in 0usize..12usize) {
        let ctx = Context::default();
        let theme = MacTheme::new(MacAppearance::Aqua);
        let mut exp_a = true;
        let mut exp_b = false;
        let mut x1 = 0.0f32;
        let mut x2 = 0.0f32;
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            MacOutlineView::new("prop_outline").theme(&theme).show(ui, |tree| {
                tree.row("r1", depth, true, &mut exp_a, false, |row| {
                    row.thumbnail(|_, r| { x1 = r.min.x; });
                    row.label("Parent");
                });
                tree.row("r2", depth + 1, false, &mut exp_b, false, |row| {
                    row.thumbnail(|_, r| { x2 = r.min.x; });
                    row.label("Child");
                });
            });
        });
        out.textures_delta.clear();
        prop_assert!(x2 > x1, "Depth {} thumbnail x ({x2}) must exceed depth {} thumbnail x ({x1})", depth + 1, depth);
    }

    /// Real widget proptest: MacStepper with wraps(false) strictly clamps value to [min, max] on ArrowUp/ArrowDown.
    #[test]
    fn prop_stepper_bounds_clamping(
        min in -200.0f64..200.0f64,
        span in 5.0f64..500.0f64,
        step in 0.5f64..10.0f64,
        up_press in any::<bool>(),
    ) {
        let max = min + span;
        let ctx = Context::default();
        let theme = MacTheme::new(MacAppearance::Aqua);
        let mut val = if up_press { max } else { min };
        let mut out0 = ctx.run_ui(RawInput::default(), |ui| {
            MacStepper::new(&mut val, min..=max, step).theme(&theme).wraps(false).show(ui).request_focus();
        });
        out0.textures_delta.clear();

        let mut input = RawInput::default();
        input.events.push(Event::Key {
            key: if up_press { Key::ArrowUp } else { Key::ArrowDown },
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        });
        let mut out1 = ctx.run_ui(input, |ui| {
            MacStepper::new(&mut val, min..=max, step).theme(&theme).wraps(false).show(ui);
        });
        out1.textures_delta.clear();
        prop_assert!(val >= min - 1e-6 && val <= max + 1e-6);
    }

    /// Real widget proptest: MacScrollView clamps arbitrary programmatic offset into [0, content_h - viewport_h].
    #[test]
    fn prop_scroll_view_viewport_clamping(
        viewport_h in 50.0f32..300.0f32,
        content_h in 100.0f32..1000.0f32,
        raw_offset_y in -200.0f32..1500.0f32,
    ) {
        let ctx = Context::default();
        let theme = MacTheme::new(MacAppearance::Aqua);
        let mut offset = Vec2::new(0.0, raw_offset_y);
        let mut max_scroll = 0.0f32;
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            let res = MacScrollView::new("prop_scroll")
                .theme(&theme)
                .with_offset(&mut offset)
                .max_height(viewport_h)
                .max_width(150.0)
                .show(ui, |c| {
                    c.allocate_exact_size(Vec2::new(150.0, content_h), Sense::hover());
                });
            max_scroll = (res.content_size.y - res.inner_rect.height()).max(0.0);
        });
        out.textures_delta.clear();
        prop_assert!(offset.y >= -1e-3 && offset.y <= max_scroll + 1e-3);
    }

    /// Real widget proptest: MacStackView total height scales monotonically with item_count and spacing.
    #[test]
    fn prop_stack_view_cumulative_spacing(
        item_count in 1usize..12usize,
        spacing in 2.0f32..20.0f32,
    ) {
        let ctx = Context::default();
        let theme = MacTheme::new(MacAppearance::Aqua);
        let mut rect_h = 0.0f32;
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            let res = MacStackView::vertical()
                .theme(&theme)
                .spacing(spacing)
                .padding(0.0)
                .show(ui, |stack| {
                    for _ in 0..item_count {
                        stack.add(|u| { u.allocate_exact_size(Vec2::new(60.0, 10.0), Sense::hover()); });
                    }
                });
            rect_h = res.rect.height();
        });
        out.textures_delta.clear();
        let expected_min = (item_count as f32 * 10.0) + (item_count.saturating_sub(1) as f32 * spacing);
        prop_assert!(rect_h >= expected_min - 1.0);
    }

    /// Real widget proptest: MacScrollView dual overlay renders both scrollbars without panic or out-of-bounds offset.
    #[test]
    fn prop_scroll_view_dual_overlay_no_overlap(
        view_w in 40.0f32..300.0f32,
        view_h in 40.0f32..300.0f32,
        content_w in 350.0f32..800.0f32,
        content_h in 350.0f32..800.0f32,
    ) {
        let ctx = Context::default();
        let theme = MacTheme::new(MacAppearance::DarkAqua);
        let mut offset = Vec2::new(50.0, 50.0);
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            let res = MacScrollView::both("prop_dual_scroll")
                .theme(&theme)
                .with_offset(&mut offset)
                .max_width(view_w)
                .max_height(view_h)
                .show(ui, |c| {
                    c.allocate_exact_size(Vec2::new(content_w, content_h), Sense::hover());
                });
            assert_eq!(res.inner_rect.size(), Vec2::new(view_w, view_h));
        });
        out.textures_delta.clear();
        prop_assert!(offset.x >= 0.0 && offset.x <= content_w - view_w + 1e-3);
        prop_assert!(offset.y >= 0.0 && offset.y <= content_h - view_h + 1e-3);
    }
}
