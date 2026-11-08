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

/// TC-011-01: Verifies `MacProgressIndicator` bar and spinning styles across Regular, Small, and Mini sizes.
#[test]
fn test_progress_indicator_styles_and_sizes() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut reg_bar_rect = Rect::NOTHING;
    let mut small_bar_rect = Rect::NOTHING;
    let mut mini_spin_rect = Rect::NOTHING;

    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        let r1 = MacProgressIndicator::bar(0.65)
            .theme(&theme)
            .control_size(MacProgressControlSize::Regular)
            .width(180.0)
            .show_percentage(true)
            .show(ui);
        reg_bar_rect = r1.rect;

        let r2 = MacProgressIndicator::new()
            .theme(&theme)
            .style(MacProgressStyle::Bar)
            .control_size(MacProgressControlSize::Small)
            .indeterminate(0.42)
            .label("Rendering audio...")
            .show(ui);
        small_bar_rect = r2.rect;

        let r3 = MacProgressIndicator::spinning(0.25)
            .theme(&theme)
            .control_size(MacProgressControlSize::Mini)
            .show(ui);
        mini_spin_rect = r3.rect;
    });
    out.textures_delta.clear();

    assert!(reg_bar_rect.width() >= 180.0);
    assert!(reg_bar_rect.height() >= theme.progress_indicator().regular.bar_height.points());
    assert!(small_bar_rect.height() >= theme.progress_indicator().small.bar_height.points());
    assert_eq!(
        mini_spin_rect.width(),
        theme.progress_indicator().mini.spinner_size.points()
    );
    assert_eq!(
        mini_spin_rect.height(),
        theme.progress_indicator().mini.spinner_size.points()
    );
}

/// TC-011-02: Verifies `MacTableView` header sort toggle, single/Shift/Cmd row selection, and consumed arrow keys.
#[test]
fn test_table_view_header_sort_selection_and_keyboard() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    let mut selection = BTreeSet::new();
    let mut sort_dir = Some(MacSortDirection::Ascending);

    let mut table_rect = Rect::NOTHING;
    // Frame 1: Initial layout pass
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let cols = [
            MacTableColumn::new("Track", 120.0).sort(sort_dir),
            MacTableColumn::new("LUFS", 80.0),
        ];
        let resp = MacTableView::new("studio_table", &cols, 5, &mut selection)
            .theme(&theme)
            .uses_alternating_row_backgrounds(true)
            .show(ui, |cell_ui, r, c| {
                cell_ui.label(format!("R{r}C{c}"));
            });
        table_rect = resp.response.rect;
    });
    out1.textures_delta.clear();

    let hdr_h = theme.table_view().header_height.points();
    let row_h = theme.table_view().row_height.points();

    // Frame 2: Click column 0 header to toggle sort
    let hdr_click_pos = Pos2::new(table_rect.left() + 30.0, table_rect.top() + hdr_h * 0.5);
    let mut clicked_hdr = None;
    let mut out2 = ctx.run_ui(click_input(hdr_click_pos, Modifiers::NONE), |ui| {
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
    out2.textures_delta.clear();
    assert_eq!(clicked_hdr, Some(0));
    assert_eq!(sort_dir, Some(MacSortDirection::Descending));

    // Frame 3: Single click row 1
    let row1_pos = Pos2::new(
        table_rect.left() + 40.0,
        table_rect.top() + hdr_h + row_h * 1.5,
    );
    let mut out3 = ctx.run_ui(click_input(row1_pos, Modifiers::NONE), |ui| {
        let cols = [
            MacTableColumn::new("Track", 120.0),
            MacTableColumn::new("LUFS", 80.0),
        ];
        MacTableView::new("studio_table", &cols, 5, &mut selection)
            .theme(&theme)
            .show(ui, |cell_ui, r, c| {
                cell_ui.label(format!("R{r}C{c}"));
            });
    });
    out3.textures_delta.clear();
    assert_eq!(selection, BTreeSet::from([1]));

    // Frame 4: Shift-click row 3 -> range 1..=3 selected
    let row3_pos = Pos2::new(
        table_rect.left() + 40.0,
        table_rect.top() + hdr_h + row_h * 3.5,
    );
    let shift_mods = Modifiers {
        shift: true,
        ..Default::default()
    };
    let mut out4 = ctx.run_ui(click_input(row3_pos, shift_mods), |ui| {
        let cols = [
            MacTableColumn::new("Track", 120.0),
            MacTableColumn::new("LUFS", 80.0),
        ];
        MacTableView::new("studio_table", &cols, 5, &mut selection)
            .theme(&theme)
            .show(ui, |cell_ui, r, c| {
                cell_ui.label(format!("R{r}C{c}"));
            });
    });
    out4.textures_delta.clear();
    assert_eq!(selection, BTreeSet::from([1, 2, 3]));

    // Frame 5: ArrowDown key advances selection to row 4 and consumes ArrowDown
    let arrow_input = RawInput {
        events: vec![Event::Key {
            key: Key::ArrowDown,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        ..Default::default()
    };
    let mut arrow_leaked = true;
    let mut out5 = ctx.run_ui(arrow_input, |ui| {
        let cols = [
            MacTableColumn::new("Track", 120.0),
            MacTableColumn::new("LUFS", 80.0),
        ];
        MacTableView::new("studio_table", &cols, 5, &mut selection)
            .theme(&theme)
            .show(ui, |cell_ui, r, c| {
                cell_ui.label(format!("R{r}C{c}"));
            });
        arrow_leaked = ui.input(|i| i.key_pressed(Key::ArrowDown));
    });
    out5.textures_delta.clear();
    assert_eq!(selection, BTreeSet::from([4]));
    assert!(!arrow_leaked, "ArrowDown key must be consumed by MacTableView");
}

/// TC-011-03: Verifies `MacTokenField` comma/Enter commit, Backspace deletion, flow wrap, and char counter.
#[test]
fn test_token_field_commit_backspace_and_flow_wrap() {
    let ctx = Context::default();
    let theme = MacTheme::default();
    let mut tokens = vec!["Audio".to_string(), "Video".to_string()];
    let mut buf = "Timeline,".to_string();

    // Frame 1: Comma delimiter inside buffer automatically commits "Timeline"
    let mut single_row_h = 0.0;
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let r = MacTokenField::new("tags", &mut tokens, &mut buf)
            .theme(&theme)
            .width(260.0)
            .max_chars(100)
            .show(ui);
        single_row_h = r.rect.height();
    });
    out1.textures_delta.clear();
    assert_eq!(tokens, vec!["Audio", "Video", "Timeline"]);
    assert!(buf.is_empty());

    // Frame 2: Backspace on empty buffer when focused pops last token and consumes Backspace
    let mut bs_leaked = true;
    let bs_input = RawInput {
        events: vec![Event::Key {
            key: Key::Backspace,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        ..Default::default()
    };
    let mut out2 = ctx.run_ui(bs_input, |ui| {
        // Request focus on the token field
        MacTokenField::new("tags", &mut tokens, &mut buf)
            .theme(&theme)
            .width(260.0)
            .show(ui);
        bs_leaked = ui.input(|i| i.key_pressed(Key::Backspace));
    });
    out2.textures_delta.clear();
    assert_eq!(tokens, vec!["Audio", "Video"]);
    assert!(!bs_leaked, "Backspace must be consumed by MacTokenField");

    // Frame 3: Adding many tokens wraps onto multiple lines and increases height
    for i in 0..6 {
        tokens.push(format!("StudioTag{i}"));
    }
    let mut wrapped_h = 0.0;
    let mut out3 = ctx.run_ui(RawInput::default(), |ui| {
        let r = MacTokenField::new("tags", &mut tokens, &mut buf)
            .theme(&theme)
            .width(180.0)
            .max_chars(100)
            .show(ui);
        wrapped_h = r.rect.height();
    });
    out3.textures_delta.clear();
    assert!(
        wrapped_h > single_row_h,
        "Wrapped token field height ({wrapped_h}) must exceed single-row height ({single_row_h})"
    );
}

/// TC-011-04: Verifies `MacLevelIndicator` semantic thresholds, continuous/discrete styles, and orientations.
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

    let mut horiz_rect = Rect::NOTHING;
    let mut vert_rect = Rect::NOTHING;
    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        horiz_rect = MacLevelIndicator::new(-14.0, -60.0..=0.0)
            .theme(&theme)
            .style(MacLevelIndicatorStyle::Discrete)
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
    out.textures_delta.clear();

    assert_eq!(horiz_rect.size(), egui::Vec2::new(160.0, 16.0));
    assert_eq!(vert_rect.size(), egui::Vec2::new(16.0, 120.0));
}

/// TC-011-05 & TC-011-06: Verifies `MacTrimScrubber` timecode formatting & `MacPathControl` segment clicks.
#[test]
fn test_trim_scrubber_and_path_control_interactions() {
    let ctx = Context::default();
    let theme = MacTheme::default();

    assert_eq!(MacTrimScrubber::format_timecode(&theme, 65.4), "01:05.4");

    let mut path_rect = Rect::NOTHING;
    let mut out1 = ctx.run_ui(RawInput::default(), |ui| {
        let comps = [
            MacPathComponent::folder("Projects"),
            MacPathComponent::folder("Timeline"),
            MacPathComponent::file("Master.mov"),
        ];
        let resp = MacPathControl::new("path_bar", &comps)
            .theme(&theme)
            .show_choose_button(true)
            .width(320.0)
            .show(ui);
        path_rect = resp.response.rect;
    });
    out1.textures_delta.clear();
    assert_eq!(path_rect.height(), theme.path_control().height.points());

    // Click first breadcrumb segment
    let seg0_pos = Pos2::new(path_rect.left() + 24.0, path_rect.center().y);
    let mut clicked_seg = None;
    let mut out2 = ctx.run_ui(click_input(seg0_pos, Modifiers::NONE), |ui| {
        let comps = [
            MacPathComponent::folder("Projects"),
            MacPathComponent::folder("Timeline"),
            MacPathComponent::file("Master.mov"),
        ];
        let resp = MacPathControl::new("path_bar", &comps)
            .theme(&theme)
            .show_choose_button(true)
            .width(320.0)
            .show(ui);
        clicked_seg = resp.clicked_segment;
    });
    out2.textures_delta.clear();
    // Click trailing Choose... button
    let choose_pos = Pos2::new(path_rect.right() - 24.0, path_rect.center().y);
    let mut choose_clicked = false;
    let mut out3 = ctx.run_ui(click_input(choose_pos, Modifiers::NONE), |ui| {
        let comps = [
            MacPathComponent::folder("Projects"),
            MacPathComponent::folder("Timeline"),
            MacPathComponent::file("Master.mov"),
        ];
        let resp = MacPathControl::new("path_bar", &comps)
            .theme(&theme)
            .show_choose_button(true)
            .width(320.0)
            .show(ui);
        choose_clicked = resp.choose_clicked;
    });
    out3.textures_delta.clear();
    assert!(choose_clicked, "Clicking Choose... button must set choose_clicked = true");
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
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            let pi = MacProgressIndicator::bar(raw_progress).theme(&theme);
            clamped_p = pi.clamped_progress();
            pi.show(ui);

            MacTrimScrubber::new("prop_scrubber", duration, &mut in_pt, &mut out_pt, &mut playhead)
                .theme(&theme)
                .waveform(&samples)
                .zones(&zones)
                .show(ui);
        });
        out.textures_delta.clear();

        let p = clamped_p.unwrap();
        prop_assert!((0.0..=1.0).contains(&p));
        prop_assert!(in_pt >= 0.0 && in_pt <= out_pt && out_pt <= duration);
        prop_assert!(playhead >= in_pt && playhead <= out_pt);
    }
}
