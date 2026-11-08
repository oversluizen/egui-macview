//! Runtime Hygiene, Frame Pacing, Cold-Start, and State Invariant Integration Tests (GL-1015).

#[allow(dead_code)]
#[path = "../examples/macview_gallery.rs"]
mod gallery_app;

#[allow(dead_code)]
#[path = "../examples/studio_controls_v05.rs"]
mod studio_app;

use egui::{Context, Event, Pos2, RawInput};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    MacTableColumn, MacTableView, MacTrimScrubber, NSUndoManager,
};
pub(crate) use gallery_app::{
    format_tracked_small_caps, CardModel, ExpandedProTab, GalleryApp, GalleryScreen, InspectorMode,
};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};
use studio_app::StudioApp;

#[test]
fn test_full_gallery_cpu_hygiene_all_states_and_frame_pacing() {
    let ctx = Context::default();
    let mut app = GalleryApp::new();

    let run_gallery_frame =
        |app: &mut GalleryApp, time: f64, hover_pos: Option<Pos2>| -> Duration {
            let screen_rect = Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1160.0, 740.0),
            ));
            let mut raw = RawInput {
                time: Some(time),
                screen_rect,
                ..Default::default()
            };
            if let Some(pos) = hover_pos {
                raw.events.push(Event::PointerMoved(pos));
            }
            let mut out = ctx.run_ui(raw, |ui| app.render_ui(ui));
            out.textures_delta.clear();
            out.viewport_output[&egui::ViewportId::ROOT].repaint_delay
        };

    // 1. Verify MacTheme::apply_to_egui idempotence (zero Arc<Style> clone thrashing when unchanged)
    let theme = MacTheme::new(MacAppearance::Aqua);
    theme.apply_to_egui(&ctx);
    let style_arc_1 = ctx.global_style();
    theme.apply_to_egui(&ctx);
    assert!(std::sync::Arc::ptr_eq(&style_arc_1, &ctx.global_style()));

    // 2. Verify all 5 GalleryScreen states settle to Duration::MAX (zero pending timers)
    let mut t = 10.0;
    for screen in GalleryScreen::ALL {
        app.active_screen = screen;
        app.sidebar.active_screen = screen;
        for step in [0.25, 0.25, 0.25, 0.25, 1.0, 0.5] {
            t += step;
            let _ = run_gallery_frame(&mut app, t, None);
        }
        t += 0.5;
        let delay = run_gallery_frame(&mut app, t, None);
        assert_eq!(
            delay,
            Duration::MAX,
            "GalleryScreen::{screen:?} at rest must settle to Duration::MAX"
        );
    }

    // 3. Verify stationary pointer hovering over an active overflowing scrollbar track/thumb
    //    does NOT cause a 0ms busy-loop, and mid-fade reactivation resets the fade timeout
    //    before settling to Duration::MAX.
    let scroll_ctx = Context::default();
    theme.apply_to_egui(&scroll_ctx);
    let run_scroll_frame = |time: f64, hover_pos: Option<Pos2>| -> (Duration, egui::Rect) {
        let screen_rect = Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(200.0, 120.0),
        ));
        let mut raw = RawInput {
            time: Some(time),
            screen_rect,
            ..Default::default()
        };
        if let Some(pos) = hover_pos {
            raw.events.push(Event::PointerMoved(pos));
        }
        let mut scroll_rect = egui::Rect::NOTHING;
        let mut out = scroll_ctx.run_ui(raw, |ui| {
            let s_out = egui_macview::MacScrollView::new("cpu_hygiene_scroll")
                .theme(&theme)
                .max_width(200.0)
                .max_height(100.0)
                .show(ui, |ui| {
                    for i in 0..20 {
                        ui.label(format!("Scrollable Row {i}"));
                    }
                });
            scroll_rect = s_out.inner_rect;
        });
        out.textures_delta.clear();
        (
            out.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
            scroll_rect,
        )
    };

    let _ = run_scroll_frame(1.0, Some(egui::pos2(10.0, 10.0)));
    let _ = run_scroll_frame(1.016, None);
    let (settled_initial, scroll_rect) = run_scroll_frame(2.5, None);
    assert_eq!(settled_initial, Duration::MAX);

    let sb_x = scroll_rect.right() - 4.0;
    let sb_y = scroll_rect.center().y;
    let _ = run_scroll_frame(3.0, Some(egui::pos2(sb_x, sb_y)));
    let _ = run_scroll_frame(3.016, None);
    let (hover_active_delay, _) = run_scroll_frame(3.032, None);
    assert!(
        hover_active_delay >= Duration::from_millis(700)
            && hover_active_delay <= Duration::from_millis(850),
        "got hover_active_delay={hover_active_delay:?}, scroll_rect={scroll_rect:?}"
    );

    assert_eq!(run_scroll_frame(3.85, None).0, Duration::from_millis(16));
    let _ = run_scroll_frame(3.90, Some(egui::pos2(sb_x, sb_y + 8.0)));
    let _ = run_scroll_frame(3.916, None);
    assert!(run_scroll_frame(3.932, None).0 >= Duration::from_millis(700));
    assert_eq!(run_scroll_frame(5.2, None).0, Duration::MAX);

    // 4. Verify active timeline playback on State 5 uses frame-paced delay (>= 16ms), never Duration::ZERO,
    //    and pauses automatically (Duration::MAX) when the window loses focus.
    app.active_screen = GalleryScreen::AvKitMediaStudio;
    app.center.media_playing = true;
    t += 0.050;
    let play_delay = run_gallery_frame(&mut app, t, None);
    assert!(!play_delay.is_zero() && play_delay <= Duration::from_millis(50));

    let unfocused_raw = RawInput {
        time: Some(t + 0.050),
        focused: false,
        screen_rect: Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            egui::vec2(1160.0, 740.0),
        )),
        ..Default::default()
    };
    let mut unfocused_out = ctx.run_ui(unfocused_raw, |ui| app.render_ui(ui));
    unfocused_out.textures_delta.clear();
    assert_eq!(
        unfocused_out.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
        Duration::MAX,
        "Unfocused window must pause timeline repaints (0.0% background CPU)"
    );
    app.center.media_playing = false;
}

#[test]
fn test_cold_start_zero_idle_repaint_and_state_invariants_smoke() {
    // Warm lazy global token catalog & egui font atlas once before measuring GalleryApp / StudioApp cold-start + Frame 1
    let screen_rect = Some(egui::Rect::from_min_size(
        Pos2::ZERO,
        egui::vec2(1160.0, 740.0),
    ));
    let gallery_ctx = Context::default();
    let studio_ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    theme.apply_to_egui(&gallery_ctx);
    theme.apply_to_egui(&studio_ctx);
    gallery_ctx
        .run_ui(
            RawInput {
                screen_rect,
                ..Default::default()
            },
            |ui| GalleryApp::new().render_ui(ui),
        )
        .textures_delta
        .clear();
    studio_ctx
        .run_ui(
            RawInput {
                screen_rect,
                ..Default::default()
            },
            |ui| StudioApp::default().render_ui(ui),
        )
        .textures_delta
        .clear();

    // 1. Cold-Start + First Frame < 500ms & Zero Idle Repaint Leak after 2 rest frames
    //    across all 5 GalleryScreen states in Aqua & DarkAqua + StudioApp (v0.5.0)
    let t0 = Instant::now();
    let mut gallery = GalleryApp::new();
    let mut out_first = gallery_ctx.run_ui(
        RawInput {
            time: Some(0.0),
            screen_rect,
            ..Default::default()
        },
        |ui| gallery.render_ui(ui),
    );
    out_first.textures_delta.clear();
    let gallery_cold_start = t0.elapsed();
    assert!(
        gallery_cold_start < Duration::from_millis(500),
        "GalleryApp cold-start + first frame must be < 500ms, took {gallery_cold_start:?}"
    );

    let t1 = Instant::now();
    let mut studio = StudioApp::default();
    let mut studio_first = studio_ctx.run_ui(
        RawInput {
            time: Some(0.0),
            screen_rect,
            ..Default::default()
        },
        |ui| studio.render_ui(ui),
    );
    studio_first.textures_delta.clear();
    let studio_cold_start = t1.elapsed();
    assert!(
        studio_cold_start < Duration::from_millis(500),
        "StudioApp cold-start + first frame must be < 500ms, took {studio_cold_start:?}"
    );

    // Check StudioApp zero idle repaint leak after 2 rest frames (when indeterminate == false)
    for appearance in [MacAppearance::Aqua, MacAppearance::DarkAqua] {
        studio.appearance = appearance;
        studio.indeterminate = false;
        let mut last_delay = Duration::ZERO;
        for frame_idx in 1..=3 {
            let mut out = studio_ctx.run_ui(
                RawInput {
                    time: Some(frame_idx as f64 * 0.5),
                    screen_rect,
                    ..Default::default()
                },
                |ui| studio.render_ui(ui),
            );
            out.textures_delta.clear();
            last_delay = out.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
        }
        assert!(
            last_delay > Duration::ZERO,
            "StudioApp ({appearance:?}) must have repaint_delay > Duration::ZERO after 2 rest frames"
        );
    }

    // Check GalleryApp all 5 screens in both Aqua and DarkAqua after 2 rest frames
    let mut sim_time = 1.0;
    for appearance in [MacAppearance::Aqua, MacAppearance::DarkAqua] {
        gallery.appearance = appearance;
        gallery.center.publish_indeterminate = false;
        gallery.center.media_playing = false;
        for screen in GalleryScreen::ALL {
            gallery.active_screen = screen;
            gallery.sidebar.active_screen = screen;
            let mut delay = Duration::ZERO;
            for _ in 0..3 {
                sim_time += 0.5;
                let mut out = gallery_ctx.run_ui(
                    RawInput {
                        time: Some(sim_time),
                        screen_rect,
                        ..Default::default()
                    },
                    |ui| gallery.render_ui(ui),
                );
                out.textures_delta.clear();
                delay = out.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
            }
            assert!(
                delay > Duration::ZERO,
                "GalleryApp ({appearance:?}, {screen:?}) leaked 0ms idle repaint!"
            );
        }
    }

    // 2. MacTrimScrubber Invariant: 0.0 <= in_point <= out_point <= duration
    for (dur, mut in_pt, mut out_pt, mut head) in [
        (120.0, 95.0, 20.0, -15.0),
        (60.0, -50.0, 999.0, 500.0),
        (0.0, 10.0, 5.0, 20.0),
    ] {
        let mut out = studio_ctx.run_ui(RawInput::default(), |ui| {
            MacTrimScrubber::new("inv_trim", dur, &mut in_pt, &mut out_pt, &mut head)
                .theme(&theme)
                .show(ui);
        });
        out.textures_delta.clear();
        let eff_dur = dur.max(1e-3);
        assert!(
            0.0 <= in_pt && in_pt <= out_pt && out_pt <= eff_dur,
            "MacTrimScrubber violated 0.0 <= in_point ({in_pt}) <= out_point ({out_pt}) <= duration ({eff_dur})"
        );
        assert!(
            in_pt <= head && head <= out_pt,
            "MacTrimScrubber violated in_point ({in_pt}) <= playhead ({head}) <= out_point ({out_pt})"
        );
    }

    // 3. MacTableView Invariant: valid selected_rows after table filtering (row_count reduction)
    let cols = [MacTableColumn::new("Asset", 160.0)];
    let mut selected_rows = BTreeSet::from([0usize, 2, 3]);
    let mut out_tbl1 = studio_ctx.run_ui(RawInput::default(), |ui| {
        MacTableView::new("filter_inv_tbl", &cols, 4, &mut selected_rows)
            .theme(&theme)
            .show(ui, |cell_ui, r, _| {
                cell_ui.label(format!("Row {r}"));
            });
    });
    out_tbl1.textures_delta.clear();
    assert_eq!(selected_rows, BTreeSet::from([0, 2, 3]));

    // Filter reduces row_count from 4 to 2 -> out-of-bounds indices (2, 3) must be pruned!
    let mut out_tbl2 = studio_ctx.run_ui(RawInput::default(), |ui| {
        MacTableView::new("filter_inv_tbl", &cols, 2, &mut selected_rows)
            .theme(&theme)
            .show(ui, |cell_ui, r, _| {
                cell_ui.label(format!("Row {r}"));
            });
    });
    out_tbl2.textures_delta.clear();
    assert!(
        selected_rows.iter().all(|&idx| idx < 2),
        "MacTableView must prune out-of-bounds selected_rows after table filtering! Got: {selected_rows:?}"
    );

    // Filter reduces row_count to 0 -> selected_rows must be empty
    let mut out_tbl3 = studio_ctx.run_ui(RawInput::default(), |ui| {
        MacTableView::new("filter_inv_tbl", &cols, 0, &mut selected_rows)
            .theme(&theme)
            .show(ui, |cell_ui, r, _| {
                cell_ui.label(format!("Row {r}"));
            });
    });
    out_tbl3.textures_delta.clear();
    assert!(
        selected_rows.is_empty(),
        "MacTableView with 0 rows must have empty selected_rows, got {selected_rows:?}"
    );

    // 4. Bit-exact Action -> Undo -> Redo roundtrips on NSUndoManager<CardModel> + validate_invariants()
    let mut undo_mgr = NSUndoManager::<CardModel>::new();
    undo_mgr.validate_invariants().expect("initial invariants");
    let initial_card = CardModel::default();
    let mut active_card = initial_card.clone();

    // Discrete action
    undo_mgr.push_snapshot(active_card.clone());
    undo_mgr.validate_invariants().expect("after push_snapshot");
    active_card.layer_name = "Hero Vector Layer".to_string();
    active_card.rotation_deg = 33.5;
    active_card.opacity = 72.0;
    let mutated_card = active_card.clone();

    assert!(undo_mgr.undo(&mut active_card));
    undo_mgr.validate_invariants().expect("after undo");
    assert_eq!(
        active_card, initial_card,
        "Undo must restore bit-exact initial CardModel"
    );
    assert!(undo_mgr.redo(&mut active_card));
    undo_mgr.validate_invariants().expect("after redo");
    assert_eq!(
        active_card, mutated_card,
        "Redo must restore bit-exact mutated CardModel"
    );

    // Coalesced group action
    undo_mgr.begin_grouping(active_card.clone());
    active_card.scale_pct = 125.0;
    undo_mgr.push_snapshot(active_card.clone());
    active_card.scale_pct = 175.0;
    assert!(undo_mgr.end_grouping(&active_card));
    undo_mgr.validate_invariants().expect("after end_grouping");
    let grouped_card = active_card.clone();

    assert!(undo_mgr.undo(&mut active_card));
    undo_mgr.validate_invariants().expect("after group undo");
    assert_eq!(active_card, mutated_card);
    assert!(undo_mgr.redo(&mut active_card));
    undo_mgr.validate_invariants().expect("after group redo");
    assert_eq!(active_card, grouped_card);
}
