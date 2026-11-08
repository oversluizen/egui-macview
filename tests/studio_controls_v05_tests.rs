//! ISO/IEC/IEEE 29119-3:2021 Integration & Property Test Suite for v0.5.0 Studio & AVKit Controls.
//!
//! Normative Reference: SPEC-011 (`REQ-011-01` through `REQ-011-06`) & GL-1015.
//! Every test and property test executes the actual widgets inside a headless `egui::Context`.

use egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect};
use egui_macview::{
    MacAppearance, MacLevelIndicator, MacLevelIndicatorStyle, MacLevelOrientation,
    MacPathComponent, MacPathControl, MacProgressControlSize, MacProgressIndicator,
    MacProgressStyle, MacSortDirection, MacTableColumn, MacTableView, MacTheme, MacTimelineZone,
    MacTokenField, MacTrimScrubber,
};
use proptest::prelude::*;
use std::collections::BTreeSet;

fn click_input(pos: Pos2, modifiers: Modifiers) -> RawInput {
    RawInput {
        events: vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers,
            },
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers,
            },
        ],
        ..Default::default()
    }
}

fn key_input(key: Key) -> RawInput {
    RawInput {
        events: vec![Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
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
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let (mut reg_bar, mut small_bar, mut mini_spin) = (Rect::NOTHING, Rect::NOTHING, Rect::NOTHING);

    run_pass(&ctx, RawInput::default(), |ui| {
        reg_bar = MacProgressIndicator::bar(0.20)
            .theme(&theme)
            .control_size(MacProgressControlSize::Regular)
            .width(180.0)
            .time(0.0)
            .show_percentage(true)
            .show(ui)
            .rect;
    });

    // Defect 1 & 2: Jump from 0.20 -> 0.80 at t=0.01 must animate via CASpring (not jump in 1 frame),
    // and spinning spokes must be inset within rect with rounded Circle capsule tips.
    let shapes = run_pass(&ctx, RawInput::default(), |ui| {
        reg_bar = MacProgressIndicator::bar(0.80)
            .theme(&theme)
            .control_size(MacProgressControlSize::Regular)
            .width(180.0)
            .time(0.01)
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
        .filter_map(|cs| match &cs.shape {
            egui::Shape::Rect(r) if r.fill == theme.control_accent() && reg_bar.contains(r.rect.center()) => {
                Some(r.rect.width())
            }
            _ => None,
        })
        .next()
        .unwrap_or(0.0);
    assert!(
        fill_w > 36.0 && fill_w < 90.0,
        "CASpring determinate progress at t=0.01s must interpolate smoothly (got fill_w={fill_w})"
    );

    let circle_tips = shapes
        .iter()
        .filter(|cs| matches!(&cs.shape, egui::Shape::Circle(c) if mini_spin.contains(c.center)))
        .count();
    assert!(circle_tips >= 24, "Spinning fins must have 24 rounded capsule tip circles (got {circle_tips})");

    let spoke_half_w = theme.progress_indicator().mini.spoke_width.points() * 0.49;
    let safe_spin = mini_spin.shrink(spoke_half_w);
    for cs in &shapes {
        if let egui::Shape::LineSegment { points, .. } = &cs.shape {
            if mini_spin.contains(points[0]) {
                assert!(
                    safe_spin.contains(points[1]),
                    "Spoke outer tip {:?} must be inset inside {:?}",
                    points[1],
                    safe_spin
                );
            }
        }
    }

    assert!(reg_bar.width() >= 180.0);
    assert!(reg_bar.height() >= theme.progress_indicator().regular.bar_height.points());
    assert!(small_bar.height() >= theme.progress_indicator().small.bar_height.points());
    let mini_sz = theme.progress_indicator().mini.spinner_size.points();
    assert_eq!(mini_spin.size(), egui::Vec2::splat(mini_sz));
}

/// TC-011-02: Verifies `MacTableView` header sort toggle, single/Shift/Cmd row selection, and consumed arrow keys.
#[test]
fn test_table_view_header_sort_selection_and_keyboard() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut selection = BTreeSet::new();
    let mut sort_dir = Some(MacSortDirection::Ascending);
    let mut table_rect = Rect::NOTHING;
    let mut cell0_clip_w = f32::MAX;

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
    // Defect 4: Cell clip rect must be constrained to column width, and header title clipped before chevron
    assert!(cell0_clip_w <= 120.0, "Cell clip width ({cell0_clip_w}) must be <= column width 120.0");
    let hdr_h = theme.table_view().header_height.points();
    let chev_sz = theme.table_view().sort_chevron_size.points();
    let hdr_text_clip_right = hdr_shapes
        .iter()
        .filter_map(|cs| match &cs.shape {
            egui::Shape::Text(t) if t.pos.y < table_rect.top() + hdr_h && t.pos.x < table_rect.left() + 60.0 => {
                Some(cs.clip_rect.right())
            }
            _ => None,
        })
        .next()
        .unwrap_or(f32::MAX);
    assert!(
        hdr_text_clip_right <= table_rect.left() + 120.0 - chev_sz,
        "Sorted header title clip right ({hdr_text_clip_right}) must stop before sort chevron"
    );

    let row_h = theme.table_view().row_height.points();
    let hdr_click_pos = Pos2::new(table_rect.left() + 30.0, table_rect.top() + hdr_h * 0.5);
    let mut clicked_hdr = None;
    run_pass(&ctx, click_input(hdr_click_pos, Modifiers::NONE), |ui| {
        let cols = [
            MacTableColumn::new("Track", 120.0).sort(sort_dir),
            MacTableColumn::new("LUFS", 80.0),
        ];
        let resp = MacTableView::new("studio_table", &cols, 5, &mut selection)
            .theme(&theme)
            .show(ui, |cell_ui, r, c| {
                cell_ui.label(format!("R{r}C{c}"));
            });
        clicked_hdr = resp.clicked_header;
        if let Some((_, new_dir)) = resp.toggled_sort {
            sort_dir = Some(new_dir);
        }
    });
    assert_eq!(clicked_hdr, Some(0));
    assert_eq!(sort_dir, Some(MacSortDirection::Descending));

    let base_cols = [
        MacTableColumn::new("Track", 120.0),
        MacTableColumn::new("LUFS", 80.0),
    ];

    let row1_pos = Pos2::new(table_rect.left() + 40.0, table_rect.top() + hdr_h + row_h * 1.5);
    run_pass(&ctx, click_input(row1_pos, Modifiers::NONE), |ui| {
        MacTableView::new("studio_table", &base_cols, 5, &mut selection)
            .theme(&theme)
            .show(ui, |cell_ui, r, c| {
                cell_ui.label(format!("R{r}C{c}"));
            });
    });
    assert_eq!(selection, BTreeSet::from([1]));

    // Defect 3: Selected row must override child_ui text color with selected_control_background_style primary text color
    let row3_pos = Pos2::new(table_rect.left() + 40.0, table_rect.top() + hdr_h + row_h * 3.5);
    let shift_mods = Modifiers { shift: true, ..Default::default() };
    let mut sel_fg = None;
    run_pass(&ctx, click_input(row3_pos, shift_mods), |ui| {
        MacTableView::new("studio_table", &base_cols, 5, &mut selection)
            .theme(&theme)
            .show(ui, |cell_ui, r, c| {
                if r == 1 && c == 0 {
                    sel_fg = cell_ui.visuals().override_text_color;
                }
                cell_ui.label(format!("R{r}C{c}"));
            });
    });
    assert_eq!(selection, BTreeSet::from([1, 2, 3]));
    assert_eq!(
        sel_fg,
        Some(theme.selected_control_background_style().primary_text_color(&theme)),
        "Selected row cell must override text color for WCAG contrast on selected_control()"
    );

    let mut arrow_leaked = true;
    run_pass(&ctx, key_input(Key::ArrowDown), |ui| {
        MacTableView::new("studio_table", &base_cols, 5, &mut selection)
            .theme(&theme)
            .show(ui, |cell_ui, r, c| {
                cell_ui.label(format!("R{r}C{c}"));
            });
        arrow_leaked = ui.input(|i| i.key_pressed(Key::ArrowDown));
    });
    assert_eq!(selection, BTreeSet::from([4]));
    assert!(!arrow_leaked, "ArrowDown key must be consumed by MacTableView");
}

/// TC-011-03: Verifies `MacTokenField` pill selection, Delete key removal, long-pill clipping, and flow wrap.
#[test]
fn test_token_field_commit_backspace_and_flow_wrap() {
    let ctx = Context::default();
    let theme = MacTheme::default();
    let mut tokens = vec!["Audio".to_string(), "Video".to_string()];
    let mut buf = "Timeline,".to_string();
    let mut field_rect = Rect::NOTHING;

    let shapes1 = run_pass(&ctx, RawInput::default(), |ui| {
        field_rect = MacTokenField::new("tags", &mut tokens, &mut buf)
            .theme(&theme)
            .width(260.0)
            .max_chars(100)
            .show(ui)
            .rect;
    });
    assert_eq!(tokens, vec!["Audio", "Video", "Timeline"]);
    assert!(buf.is_empty());

    // Defect 6: Token pill text clip rect must be constrained inside the pill before the 'x' button
    let pill_text_clip_w = shapes1
        .iter()
        .filter_map(|cs| match &cs.shape {
            egui::Shape::Text(t) if t.pos.y < field_rect.top() + 20.0 => Some(cs.clip_rect.width()),
            _ => None,
        })
        .next()
        .unwrap_or(f32::MAX);
    assert!(
        pill_text_clip_w < 120.0,
        "Token pill text must be clipped to pill interior (got clip width {pill_text_clip_w})"
    );

    // Defect 5: Clicking pill 0 body selects token 0, and Delete key removes token 0
    let pill0_body_pos = Pos2::new(field_rect.left() + 14.0, field_rect.top() + 10.0);
    let mut sel_tok = None;
    run_pass(&ctx, click_input(pill0_body_pos, Modifiers::NONE), |ui| {
        MacTokenField::new("tags", &mut tokens, &mut buf)
            .theme(&theme)
            .width(260.0)
            .show(ui);
        sel_tok = MacTokenField::selected_token(ui, "tags");
    });
    assert_eq!(sel_tok, Some(0), "Clicking token pill 0 body must select token 0");

    run_pass(&ctx, key_input(Key::Delete), |ui| {
        MacTokenField::new("tags", &mut tokens, &mut buf)
            .theme(&theme)
            .width(260.0)
            .show(ui);
    });
    assert_eq!(tokens, vec!["Video", "Timeline"], "Delete key must remove selected token 0");

    let mut bs_leaked = true;
    run_pass(&ctx, key_input(Key::Backspace), |ui| {
        MacTokenField::new("tags", &mut tokens, &mut buf)
            .theme(&theme)
            .width(260.0)
            .show(ui);
        bs_leaked = ui.input(|i| i.key_pressed(Key::Backspace));
    });
    assert_eq!(tokens, vec!["Video"]);
    assert!(!bs_leaked, "Backspace must be consumed by MacTokenField");

    for i in 0..6 {
        tokens.push(format!("StudioTag{i}"));
    }
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

/// TC-011-04: Verifies `MacLevelIndicator` uniform `.discreteCapacity` color, tick marks, and orientations.
#[test]
fn test_level_indicator_thresholds_and_orientations() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let meter = MacLevelIndicator::new(-14.0, -60.0..=0.0)
        .theme(&theme)
        .warning_value(-18.0)
        .critical_value(-6.0)
        .peak_hold(-3.0);

    assert_eq!(meter.color_for_value(&theme, -30.0), theme.system_green());
    assert_eq!(meter.color_for_value(&theme, -12.0), theme.system_yellow());
    assert_eq!(meter.color_for_value(&theme, -2.0), theme.system_red());

    let (mut horiz_rect, mut vert_rect) = (Rect::NOTHING, Rect::NOTHING);
    let shapes = run_pass(&ctx, RawInput::default(), |ui| {
        horiz_rect = MacLevelIndicator::new(-2.0, -60.0..=0.0)
            .theme(&theme)
            .warning_value(-18.0)
            .critical_value(-6.0)
            .style(MacLevelIndicatorStyle::Discrete)
            .tick_marks(5)
            .orientation(MacLevelOrientation::Horizontal)
            .length(160.0)
            .thickness(16.0)
            .show(ui)
            .rect;
        vert_rect = MacLevelIndicator::new(-14.0, -60.0..=0.0)
            .theme(&theme)
            .style(MacLevelIndicatorStyle::Continuous)
            .orientation(MacLevelOrientation::Vertical)
            .length(120.0)
            .thickness(16.0)
            .show(ui)
            .rect;
    });

    // Defect 7: In default Discrete mode at critical value (-2.0), ALL active segments must be system_red()
    let non_red_active_segs = shapes
        .iter()
        .filter(|cs| match &cs.shape {
            egui::Shape::Rect(r) if horiz_rect.contains(r.rect.center()) => {
                r.fill == theme.system_green() || r.fill == theme.system_yellow()
            }
            _ => false,
        })
        .count();
    assert_eq!(
        non_red_active_segs, 0,
        "AppKit .discreteCapacity at critical level must tint all active segments system_red()"
    );
    let ticks = shapes
        .iter()
        .filter(|cs| matches!(&cs.shape, egui::Shape::LineSegment { .. }))
        .count();
    assert!(ticks >= 5, "Expected >= 5 tick mark LineSegments, found {ticks}");

    assert_eq!(horiz_rect.size(), egui::Vec2::new(160.0, 16.0));
    assert_eq!(vert_rect.size(), egui::Vec2::new(16.0, 120.0));
}

/// TC-011-05 & TC-011-06: Verifies `MacTrimScrubber` non-overlapping handles, double slits, and `MacPathControl` clipping.
#[test]
fn test_trim_scrubber_and_path_control_interactions() {
    let ctx = Context::default();
    let theme = MacTheme::default();
    assert_eq!(MacTrimScrubber::format_timecode(&theme, 65.4), "01:05.4");

    // Defect 8 & 9: When in_pt == out_pt == 50.0, dragging right handle ear must move out_pt independently,
    // and trim handles must render 4 vertical grip slits (2 per handle).
    let (mut in_pt, mut out_pt, mut playhead) = (50.0, 50.0, 50.0);
    let mut scrub_rect = Rect::NOTHING;
    let scrub_shapes = run_pass(&ctx, RawInput::default(), |ui| {
        scrub_rect = MacTrimScrubber::new("scrubber", 100.0, &mut in_pt, &mut out_pt, &mut playhead)
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
    assert_eq!(grip_slits, 4, "AVKit trim handles must render 4 grip slits (2 per handle)");

    let out_ear_start = Pos2::new(scrub_rect.center().x + 4.0, scrub_rect.center().y);
    let out_ear_drag = Pos2::new(scrub_rect.center().x + 60.0, scrub_rect.center().y);
    let drag_start = RawInput {
        events: vec![
            Event::PointerMoved(out_ear_start),
            Event::PointerButton {
                pos: out_ear_start,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
        ..Default::default()
    };
    run_pass(&ctx, drag_start, |ui| {
        MacTrimScrubber::new("scrubber", 100.0, &mut in_pt, &mut out_pt, &mut playhead)
            .theme(&theme)
            .width(300.0)
            .show(ui);
    });
    let drag_move = RawInput {
        events: vec![
            Event::PointerMoved(out_ear_drag),
            Event::PointerButton {
                pos: out_ear_drag,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
        ..Default::default()
    };
    run_pass(&ctx, drag_move, |ui| {
        MacTrimScrubber::new("scrubber", 100.0, &mut in_pt, &mut out_pt, &mut playhead)
            .theme(&theme)
            .width(300.0)
            .show(ui);
    });
    assert!(
        out_pt > 60.0 && (in_pt - 50.0).abs() < 1e-4,
        "Dragging right handle when in==out must increase out_pt ({out_pt}) while keeping in_pt ({in_pt}) at 50.0"
    );

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
    assert_eq!(path_rect.height(), theme.path_control().height.points());

    // Defect 10: Segment text clip_rect must stop before Choose... button
    let seg_text_clip_right = path_shapes
        .iter()
        .filter_map(|cs| match &cs.shape {
            egui::Shape::Text(t) if path_rect.contains(t.pos) && t.pos.x < path_rect.right() - 70.0 => {
                Some(cs.clip_rect.right())
            }
            _ => None,
        })
        .next()
        .unwrap_or(f32::MAX);
    assert!(
        seg_text_clip_right <= path_rect.right() - 40.0,
        "Segment text clip_rect right ({seg_text_clip_right}) must stop before Choose... button"
    );

    let seg0_pos = Pos2::new(path_rect.left() + 24.0, path_rect.center().y);
    let mut clicked_seg = None;
    run_pass(&ctx, click_input(seg0_pos, Modifiers::NONE), |ui| {
        clicked_seg = MacPathControl::new("path_bar", &comps)
            .theme(&theme)
            .show_choose_button(true)
            .width(320.0)
            .show(ui)
            .clicked_segment;
    });
    assert_eq!(clicked_seg, Some(0));

    let choose_pos = Pos2::new(path_rect.right() - 24.0, path_rect.center().y);
    let mut choose_clicked = false;
    run_pass(&ctx, click_input(choose_pos, Modifiers::NONE), |ui| {
        choose_clicked = MacPathControl::new("path_bar", &comps)
            .theme(&theme)
            .show_choose_button(true)
            .width(320.0)
            .show(ui)
            .choose_clicked;
    });
    assert!(choose_clicked);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Property Oracle (REQ-011-01 & REQ-011-05):
    /// Across arbitrary progress values and trim pointer drags in a headless `egui::Context`,
    /// `MacProgressIndicator` clamps to `[0.0, 1.0]` and `MacTrimScrubber` preserves
    /// `0.0 <= in_point <= playhead <= out_point <= duration`.
    #[test]
    fn prop_studio_scrubber_and_progress_invariants(
        raw_progress in -5.0f32..5.0f32,
        duration in 10.0f64..600.0f64,
        in_frac in 0.0f64..1.0f64,
        out_frac in 0.0f64..1.0f64,
        play_frac in 0.0f64..1.0f64,
    ) {
        let ctx = Context::default();
        let theme = MacTheme::default();
        let mut in_pt = in_frac * duration;
        let mut out_pt = out_frac * duration;
        let mut playhead = play_frac * duration;
        let samples = [0.2f32, 0.7, 0.95, 0.4, 0.6];
        let zones = [MacTimelineZone::new(0.0, duration * 0.5, "Intro")];

        let mut clamped_p = None;
        run_pass(&ctx, RawInput::default(), |ui| {
            let pi = MacProgressIndicator::bar(raw_progress).theme(&theme);
            clamped_p = pi.clamped_progress();
            pi.show(ui);

            MacTrimScrubber::new("prop_scrubber", duration, &mut in_pt, &mut out_pt, &mut playhead)
                .theme(&theme)
                .waveform(&samples)
                .zones(&zones)
                .show(ui);
        });

        let p = clamped_p.unwrap();
        prop_assert!((0.0..=1.0).contains(&p));
        prop_assert!(in_pt >= 0.0 && in_pt <= out_pt && out_pt <= duration);
        prop_assert!(playhead >= in_pt && playhead <= out_pt);
    }
}
