//! ISO/IEC/IEEE 29119-3:2021 Integration & Property Test Suite for v0.5.0 Studio & AVKit Controls.
//!
//! Normative Reference: SPEC-011 (`REQ-011-01` through `REQ-011-06`) & Architecture Fitness Standards.

use egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect};
use egui_macview::{
    MacAppearance, MacPathComponent, MacPathControl, MacProgressControlSize, MacProgressIndicator,
    MacProgressStyle, MacSortDirection, MacTableColumn, MacTableView, MacTheme, MacTokenField,
    MacTrimScrubber,
};
use std::collections::BTreeSet;

fn click_input(pos: Pos2, modifiers: Modifiers) -> RawInput {
    let btn = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers,
    };
    RawInput {
        events: vec![Event::PointerMoved(pos), btn(true), btn(false)],
        ..Default::default()
    }
}

fn key_input(key: Key) -> RawInput {
    let ev = Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    RawInput {
        events: vec![ev],
        ..Default::default()
    }
}

fn run_pass(
    ctx: &Context,
    input: RawInput,
    f: impl FnMut(&mut egui::Ui),
) -> Vec<egui::epaint::ClippedShape> {
    let mut out = ctx.run_ui(input, f);
    out.textures_delta.clear();
    out.shapes
}

/// TC-011-01: Verifies `MacProgressIndicator` CASpring time animation, spinning capsule tips, and sizes.
#[test]
fn test_progress_indicator_styles_and_sizes() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));
    let (mut reg_bar, mut small_bar, mut mini_spin) = (Rect::NOTHING, Rect::NOTHING, Rect::NOTHING);
    for (val, t) in [(0.20, 0.0), (0.80, 0.01)] {
        run_pass(&ctx, RawInput::default(), |ui| {
            MacProgressIndicator::bar(val)
                .theme(&theme)
                .width(180.0)
                .time(t)
                .show_percentage(true)
                .show(ui);
        });
    }
    let shapes = run_pass(&ctx, RawInput::default(), |ui| {
        reg_bar = MacProgressIndicator::bar(0.80)
            .theme(&theme)
            .width(180.0)
            .time(0.06)
            .show_percentage(true)
            .show(ui)
            .rect;
        small_bar = MacProgressIndicator::new()
            .theme(&theme)
            .style(MacProgressStyle::Bar)
            .control_size(MacProgressControlSize::Small)
            .indeterminate(0.42)
            .label("Rendering audio...")
            .show(ui)
            .rect;
        mini_spin = MacProgressIndicator::spinning(0.25)
            .theme(&theme)
            .control_size(MacProgressControlSize::Mini)
            .show(ui)
            .rect;
    });
    let fill_w = shapes
        .iter()
        .find_map(|cs| match &cs.shape {
            egui::Shape::Rect(r)
                if r.fill == theme.control_accent() && reg_bar.contains(r.rect.center()) =>
            {
                Some(r.rect.width())
            }
            _ => None,
        })
        .unwrap_or(0.0);
    let tips = shapes
        .iter()
        .filter(|cs| matches!(&cs.shape, egui::Shape::Circle(c) if mini_spin.contains(c.center)))
        .count();
    assert!(fill_w > 36.0 && fill_w < 90.0 && tips >= 24);
    let safe_spin = mini_spin.shrink(theme.progress_indicator().mini.spoke_width.points() * 0.49);
    for cs in &shapes {
        if let egui::Shape::LineSegment { points, .. } = &cs.shape {
            if mini_spin.contains(points[0]) {
                assert!(safe_spin.contains(points[1]));
            }
        }
    }
    assert!(
        reg_bar.width() >= 180.0
            && reg_bar.height() >= theme.progress_indicator().regular.bar_height.points()
    );
    assert!(small_bar.height() >= theme.progress_indicator().small.bar_height.points());
    assert_eq!(
        mini_spin.size(),
        egui::Vec2::splat(theme.progress_indicator().mini.spinner_size.points())
    );
}

/// TC-011-02: Verifies `MacTableView` header sort toggle, single/Shift/Cmd row selection, and consumed arrow keys.
#[test]
fn test_table_view_header_sort_selection_and_keyboard() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));
    let (mut selection, mut sort_dir) = (BTreeSet::new(), Some(MacSortDirection::Ascending));
    let (mut table_rect, mut cell0_clip_w) = (Rect::NOTHING, f32::MAX);

    let hdr_shapes = run_pass(&ctx, RawInput::default(), |ui| {
        let cols = [
            MacTableColumn::new("Track", 120.0).sort(sort_dir),
            MacTableColumn::new("LUFS", 80.0),
        ];
        table_rect = MacTableView::new("studio_table", &cols, 5, &mut selection)
            .theme(&theme)
            .uses_alternating_row_backgrounds(true)
            .show(ui, |cell_ui, r, c| {
                if r == 0 && c == 0 {
                    cell0_clip_w = cell_ui.clip_rect().width();
                }
                cell_ui.label(format!("R{r}C{c}"));
            })
            .response
            .rect;
    });
    let (hdr_h, chev_sz, row_h) = (
        theme.table_view().header_height.points(),
        theme.table_view().sort_chevron_size.points(),
        theme.table_view().row_height.points(),
    );
    let hdr_clip_r = hdr_shapes
        .iter()
        .find_map(|cs| match &cs.shape {
            egui::Shape::Text(t)
                if t.pos.y < table_rect.top() + hdr_h && t.pos.x < table_rect.left() + 60.0 =>
            {
                Some(cs.clip_rect.right())
            }
            _ => None,
        })
        .unwrap_or(f32::MAX);
    assert!(cell0_clip_w <= 120.0 && hdr_clip_r <= table_rect.left() + 120.0 - chev_sz);

    let hdr_pos = Pos2::new(table_rect.left() + 30.0, table_rect.top() + hdr_h * 0.5);
    let mut clicked_hdr = None;
    run_pass(&ctx, click_input(hdr_pos, Modifiers::NONE), |ui| {
        let cols = [
            MacTableColumn::new("Track", 120.0).sort(sort_dir),
            MacTableColumn::new("LUFS", 80.0),
        ];
        let resp = MacTableView::new("studio_table", &cols, 5, &mut selection)
            .theme(&theme)
            .show(ui, |u, r, c| {
                u.label(format!("R{r}C{c}"));
            });
        clicked_hdr = resp.clicked_header;
        sort_dir = resp.toggled_sort.map(|(_, d)| d).or(sort_dir);
    });
    assert_eq!(
        (clicked_hdr, sort_dir),
        (Some(0), Some(MacSortDirection::Descending))
    );

    let base_cols = [
        MacTableColumn::new("Track", 120.0),
        MacTableColumn::new("LUFS", 80.0),
    ];
    let row_pos = |idx: f32| {
        Pos2::new(
            table_rect.left() + 40.0,
            table_rect.top() + hdr_h + row_h * idx,
        )
    };
    run_pass(&ctx, click_input(row_pos(1.5), Modifiers::NONE), |ui| {
        MacTableView::new("studio_table", &base_cols, 5, &mut selection)
            .theme(&theme)
            .show(ui, |u, r, c| {
                u.label(format!("R{r}C{c}"));
            });
    });
    assert_eq!(selection, BTreeSet::from([1]));

    let shift = Modifiers {
        shift: true,
        ..Default::default()
    };
    let mut sel_fg = None;
    run_pass(&ctx, click_input(row_pos(3.5), shift), |ui| {
        MacTableView::new("studio_table", &base_cols, 5, &mut selection)
            .theme(&theme)
            .show(ui, |cell_ui, r, c| {
                if r == 1 && c == 0 {
                    sel_fg = cell_ui.visuals().override_text_color;
                }
                cell_ui.label(format!("R{r}C{c}"));
            });
    });
    assert_eq!(
        (&selection, sel_fg),
        (
            &BTreeSet::from([1, 2, 3]),
            Some(
                theme
                    .selected_control_background_style()
                    .primary_text_color(&theme)
            )
        )
    );

    let mut arrow_leaked = true;
    run_pass(&ctx, key_input(Key::ArrowDown), |ui| {
        MacTableView::new("studio_table", &base_cols, 5, &mut selection)
            .theme(&theme)
            .show(ui, |u, r, c| {
                u.label(format!("R{r}C{c}"));
            });
        arrow_leaked = ui.input(|i| i.key_pressed(Key::ArrowDown));
    });
    assert_eq!((selection, arrow_leaked), (BTreeSet::from([4]), false));
}

/// TC-011-03: Verifies `MacTokenField` comma/Enter commit, pill selection, Delete/Backspace removal, and flow wrap.
#[test]
fn test_token_field_commit_backspace_and_flow_wrap() {
    let (ctx, theme) = (Context::default(), MacTheme::new(MacAppearance::Aqua));
    let (mut tokens, mut buf, mut field_rect) = (
        vec!["Audio".to_string()],
        "Video, Timeline,".to_string(),
        Rect::NOTHING,
    );
    let shapes1 = run_pass(&ctx, RawInput::default(), |ui| {
        field_rect = MacTokenField::new("tags", &mut tokens, &mut buf)
            .theme(&theme)
            .width(260.0)
            .max_chars(100)
            .show(ui)
            .rect;
    });
    let pill_clip_w = shapes1
        .iter()
        .find_map(|cs| match &cs.shape {
            egui::Shape::Text(t) if t.pos.y < field_rect.top() + 20.0 => Some(cs.clip_rect.width()),
            _ => None,
        })
        .unwrap_or(f32::MAX);
    assert!(tokens.len() == 3 && buf.is_empty() && pill_clip_w < 120.0);

    let pill0_pos = Pos2::new(field_rect.left() + 14.0, field_rect.top() + 10.0);
    let mut sel_tok = None;
    run_pass(&ctx, click_input(pill0_pos, Modifiers::NONE), |ui| {
        MacTokenField::new("tags", &mut tokens, &mut buf)
            .theme(&theme)
            .width(260.0)
            .show(ui);
        sel_tok = MacTokenField::selected_token(ui, "tags");
    });
    assert_eq!(sel_tok, Some(0));

    let mut bs_leaked = true;
    for k in [Key::Delete, Key::Backspace] {
        run_pass(&ctx, key_input(k), |ui| {
            MacTokenField::new("tags", &mut tokens, &mut buf)
                .theme(&theme)
                .width(260.0)
                .show(ui);
            bs_leaked = ui.input(|i| i.key_pressed(Key::Backspace));
        });
    }
    assert_eq!(
        (tokens.as_slice(), bs_leaked),
        (&["Video".to_string()][..], false)
    );

    tokens.extend((0..6).map(|i| format!("StudioTag{i}")));
    let mut wrapped_h = 0.0;
    run_pass(&ctx, RawInput::default(), |ui| {
        wrapped_h = MacTokenField::new("tags", &mut tokens, &mut buf)
            .theme(&theme)
            .width(180.0)
            .max_chars(100)
            .show(ui)
            .rect
            .height();
    });
    assert!(wrapped_h > field_rect.height());
}

/// TC-011-05 & TC-011-06: Verifies `MacTrimScrubber` non-overlapping handles, double slits, and `MacPathControl` clipping.
#[test]
fn test_trim_scrubber_and_path_control_interactions() {
    let (ctx, theme) = (Context::default(), MacTheme::default());
    assert_eq!(MacTrimScrubber::format_timecode(&theme, 65.4), "01:05.4");

    let (mut in_pt, mut out_pt, mut playhead, mut scrub_rect) = (50.0, 50.0, 50.0, Rect::NOTHING);
    let scrub_shapes = run_pass(&ctx, RawInput::default(), |ui| {
        scrub_rect =
            MacTrimScrubber::new("scrubber", 100.0, &mut in_pt, &mut out_pt, &mut playhead)
                .theme(&theme)
                .width(300.0)
                .show(ui)
                .response
                .rect;
    });
    let slot_h = theme.trim_scrubber().handle_slot_height.points();
    let grip_slits = scrub_shapes
        .iter()
        .filter(|cs| match &cs.shape {
            egui::Shape::LineSegment { points, .. } => {
                (points[0].x - points[1].x).abs() < 0.1
                    && ((points[1].y - points[0].y).abs() - slot_h).abs() < 0.5
            }
            _ => false,
        })
        .count();
    assert_eq!(grip_slits, 4);

    let (start_p, drag_p) = (
        Pos2::new(scrub_rect.center().x + 4.0, scrub_rect.center().y),
        Pos2::new(scrub_rect.center().x + 60.0, scrub_rect.center().y),
    );
    for (pos, pressed) in [(start_p, true), (drag_p, false)] {
        let btn = Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        let raw = RawInput {
            events: vec![Event::PointerMoved(pos), btn],
            ..Default::default()
        };
        run_pass(&ctx, raw, |ui| {
            MacTrimScrubber::new("scrubber", 100.0, &mut in_pt, &mut out_pt, &mut playhead)
                .theme(&theme)
                .width(300.0)
                .show(ui);
        });
    }
    assert!(out_pt > 60.0 && (in_pt - 50.0).abs() < 1e-4);

    let comps = [
        MacPathComponent::folder("Projects"),
        MacPathComponent::folder("Timeline"),
        MacPathComponent::file("Master.mov"),
    ];
    let mut path_rect = Rect::NOTHING;
    let path_shapes = run_pass(&ctx, RawInput::default(), |ui| {
        path_rect = MacPathControl::new("path_bar", &comps)
            .theme(&theme)
            .show_choose_button(true)
            .width(320.0)
            .show(ui)
            .response
            .rect;
    });
    let seg_clip_r = path_shapes
        .iter()
        .find_map(|cs| match &cs.shape {
            egui::Shape::Text(t)
                if path_rect.contains(t.pos) && t.pos.x < path_rect.right() - 70.0 =>
            {
                Some(cs.clip_rect.right())
            }
            _ => None,
        })
        .unwrap_or(f32::MAX);
    assert!(
        path_rect.height() == theme.path_control().height.points()
            && seg_clip_r <= path_rect.right() - 40.0
    );

    let (seg0_pos, choose_pos) = (
        Pos2::new(path_rect.left() + 24.0, path_rect.center().y),
        Pos2::new(path_rect.right() - 24.0, path_rect.center().y),
    );
    let (mut clicked_seg, mut choose_clicked) = (None, false);
    run_pass(&ctx, click_input(seg0_pos, Modifiers::NONE), |ui| {
        clicked_seg = MacPathControl::new("path_bar", &comps)
            .theme(&theme)
            .show_choose_button(true)
            .width(320.0)
            .show(ui)
            .clicked_segment;
    });
    run_pass(&ctx, click_input(choose_pos, Modifiers::NONE), |ui| {
        choose_clicked = MacPathControl::new("path_bar", &comps)
            .theme(&theme)
            .show_choose_button(true)
            .width(320.0)
            .show(ui)
            .choose_clicked;
    });
    assert_eq!((clicked_seg, choose_clicked), (Some(0), true));
}
