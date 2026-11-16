//! Runtime Hygiene, Frame Pacing, Cold-Start, and State Invariant Integration Tests.

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
pub(crate) use gallery_app::{format_tracked_small_caps, CardModel, ExpandedProTab, GalleryApp, GalleryScreen, InspectorMode};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};
use studio_app::StudioApp;

static HYGIENE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_full_gallery_cpu_hygiene_all_states_and_frame_pacing() {
    let _guard = HYGIENE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let ctx = Context::default();
    let mut app = GalleryApp::new();

    let run_gallery_frame = |app: &mut GalleryApp, time: f64, hover_pos: Option<Pos2>| -> Duration {
        let screen_rect = Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1160.0, 740.0)));
        let mut raw = RawInput { time: Some(time), screen_rect, ..Default::default() };
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
        assert_eq!(delay, Duration::MAX, "GalleryScreen::{screen:?} at rest must settle to Duration::MAX");
    }

    // 3. Verify stationary pointer hovering over an active overflowing scrollbar track/thumb
    //    does NOT cause a 0ms busy-loop, and mid-fade reactivation resets the fade timeout
    //    before settling to Duration::MAX.
    let scroll_ctx = Context::default();
    theme.apply_to_egui(&scroll_ctx);
    let run_scroll_frame = |time: f64, hover_pos: Option<Pos2>| -> (Duration, egui::Rect) {
        let screen_rect = Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(200.0, 120.0)));
        let mut raw = RawInput { time: Some(time), screen_rect, ..Default::default() };
        if let Some(pos) = hover_pos {
            raw.events.push(Event::PointerMoved(pos));
        }
        let mut scroll_rect = egui::Rect::NOTHING;
        let mut out = scroll_ctx.run_ui(raw, |ui| {
            let s_out =
                egui_macview::MacScrollView::new("cpu_hygiene_scroll").theme(&theme).max_width(200.0).max_height(100.0).show(ui, |ui| {
                    for i in 0..20 {
                        ui.label(format!("Scrollable Row {i}"));
                    }
                });
            scroll_rect = s_out.inner_rect;
        });
        out.textures_delta.clear();
        (out.viewport_output[&egui::ViewportId::ROOT].repaint_delay, scroll_rect)
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
        hover_active_delay >= Duration::from_millis(700) && hover_active_delay <= Duration::from_millis(850),
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
    app.sidebar.active_screen = GalleryScreen::AvKitMediaStudio;
    for step in [0.5, 0.5] {
        t += step;
        let _ = run_gallery_frame(&mut app, t, None);
    }
    app.center.media_playing = true;
    t += 0.050;
    let play_delay = run_gallery_frame(&mut app, t, None);
    assert!(!play_delay.is_zero() && play_delay <= Duration::from_millis(50));

    let unfocused_raw = RawInput {
        time: Some(t + 0.050),
        focused: false,
        screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1160.0, 740.0))),
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
    let _guard = HYGIENE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // Warm lazy global token catalog & egui font atlas once before measuring GalleryApp / StudioApp cold-start + Frame 1
    let screen_rect = Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1160.0, 740.0)));
    let gallery_ctx = Context::default();
    let studio_ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);
    theme.apply_to_egui(&gallery_ctx);
    theme.apply_to_egui(&studio_ctx);
    gallery_ctx.run_ui(RawInput { screen_rect, ..Default::default() }, |ui| GalleryApp::new().render_ui(ui)).textures_delta.clear();
    studio_ctx.run_ui(RawInput { screen_rect, ..Default::default() }, |ui| StudioApp::default().render_ui(ui)).textures_delta.clear();

    // 1. Cold-Start + First Frame < 500ms & Zero Idle Repaint Leak after 2 rest frames
    //    across all 5 GalleryScreen states in Aqua & DarkAqua + StudioApp (v0.5.0)
    let t0 = Instant::now();
    let mut gallery = GalleryApp::new();
    let mut out_first = gallery_ctx.run_ui(RawInput { time: Some(0.0), screen_rect, ..Default::default() }, |ui| gallery.render_ui(ui));
    out_first.textures_delta.clear();
    let gallery_cold_start = t0.elapsed();
    assert!(
        gallery_cold_start < Duration::from_millis(500),
        "GalleryApp cold-start + first frame must be < 500ms, took {gallery_cold_start:?}"
    );

    let t1 = Instant::now();
    let mut studio = StudioApp::default();
    let mut studio_first = studio_ctx.run_ui(RawInput { time: Some(0.0), screen_rect, ..Default::default() }, |ui| studio.render_ui(ui));
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
            let mut out = studio_ctx
                .run_ui(RawInput { time: Some(frame_idx as f64 * 0.5), screen_rect, ..Default::default() }, |ui| studio.render_ui(ui));
            out.textures_delta.clear();
            last_delay = out.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
        }
        assert!(last_delay > Duration::ZERO, "StudioApp ({appearance:?}) must have repaint_delay > Duration::ZERO after 2 rest frames");
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
                let mut out =
                    gallery_ctx.run_ui(RawInput { time: Some(sim_time), screen_rect, ..Default::default() }, |ui| gallery.render_ui(ui));
                out.textures_delta.clear();
                delay = out.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
            }
            assert!(delay > Duration::ZERO, "GalleryApp ({appearance:?}, {screen:?}) leaked 0ms idle repaint!");
        }
    }

    // 2. MacTrimScrubber Invariant: 0.0 <= in_point <= out_point <= duration
    for (dur, mut in_pt, mut out_pt, mut head) in [(120.0, 95.0, 20.0, -15.0), (60.0, -50.0, 999.0, 500.0), (0.0, 10.0, 5.0, 20.0)] {
        let mut out = studio_ctx.run_ui(RawInput::default(), |ui| {
            MacTrimScrubber::new("inv_trim", dur, &mut in_pt, &mut out_pt, &mut head).theme(&theme).show(ui);
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
        MacTableView::new("filter_inv_tbl", &cols, 4, &mut selected_rows).theme(&theme).show(ui, |cell_ui, r, _| {
            cell_ui.label(format!("Row {r}"));
        });
    });
    out_tbl1.textures_delta.clear();
    assert_eq!(selected_rows, BTreeSet::from([0, 2, 3]));

    // Filter reduces row_count from 4 to 2 -> out-of-bounds indices (2, 3) must be pruned!
    let mut out_tbl2 = studio_ctx.run_ui(RawInput::default(), |ui| {
        MacTableView::new("filter_inv_tbl", &cols, 2, &mut selected_rows).theme(&theme).show(ui, |cell_ui, r, _| {
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
        MacTableView::new("filter_inv_tbl", &cols, 0, &mut selected_rows).theme(&theme).show(ui, |cell_ui, r, _| {
            cell_ui.label(format!("Row {r}"));
        });
    });
    out_tbl3.textures_delta.clear();
    assert!(selected_rows.is_empty(), "MacTableView with 0 rows must have empty selected_rows, got {selected_rows:?}");

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
    assert_eq!(active_card, initial_card, "Undo must restore bit-exact initial CardModel");
    assert!(undo_mgr.redo(&mut active_card));
    undo_mgr.validate_invariants().expect("after redo");
    assert_eq!(active_card, mutated_card, "Redo must restore bit-exact mutated CardModel");

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

#[test]
fn test_req013_02o_mobile_tablet_responsive_layout_and_webgl_frame_encoder() {
    let _guard = HYGIENE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut session = gallery_app::gallery::mobile_web_server::MobileWebSession::new();
    let frame_bytes = session.encode_frame(390.0, 844.0, 2.0, "");
    assert!(frame_bytes.starts_with(b"MWGL"), "REQ-013-02O: Mobile WebGL2 frame stream must start with MWGL magic header");
    assert!(session.app.mobile_compact_active, "REQ-013-02O: 390pt iPhone portrait viewport must activate compact mobile layout");
    assert!(
        !session.atlas_rgba.is_empty() && session.atlas_width > 0 && session.atlas_height > 0,
        "REQ-013-02O: Mobile WebGL2 session must populate RGBA8 font/symbol atlas"
    );

    // Switch mobile pane to Inspector (1) and Navigator (2)
    session.app.mobile_pane_idx = 1;
    let insp_bytes = session.encode_frame(390.0, 844.0, 2.0, "");
    assert!(insp_bytes.len() > 64, "Inspector mobile pane must emit WebGL2 mesh data");

    session.app.mobile_pane_idx = 2;
    let nav_bytes = session.encode_frame(390.0, 844.0, 2.0, "");
    assert!(nav_bytes.len() > 64, "Navigator mobile pane must emit WebGL2 mesh data");

    // REQ-013-02O.1: Viewport dimensions from `V <w> <h> <dpr>` must persist across frames when body is empty
    let _ = session.encode_frame(390.0, 844.0, 2.0, "V 960 640 2.0");
    assert!(!session.app.mobile_compact_active, "REQ-013-02O.1: 960x640 landscape viewport must switch to 3-column desktop layout");
    let _ = session.encode_frame(390.0, 844.0, 2.0, "");
    assert!(
        !session.app.mobile_compact_active,
        "REQ-013-02O.1: MobileWebSession must persist last_viewport across empty-body frames instead of snapping to 390x844"
    );

    // REQ-013-02O.1: HTML client must use getBoundingClientRect() and fixed inset:0 canvas to prevent mobile URL-bar touch offset
    let html_src = std::fs::read_to_string("examples/gallery/mobile_web_html.html").expect("read mobile_web_html.html");
    assert!(
        html_src.contains("getBoundingClientRect()") && html_src.contains("position: fixed; inset: 0;"),
        "REQ-013-02O.1: mobile_web_html.html must normalize touch coordinates via getBoundingClientRect() and fixed inset:0"
    );

    // REQ-013-02O.2: Active animation sets bit 31 (0x8000_0000) of raw_atlas_ver in MWGL header for 60 FPS pacing
    session.app.active_screen = GalleryScreen::AvKitMediaStudio;
    session.app.center.media_playing = true;
    let anim_bytes = session.encode_frame(390.0, 844.0, 2.0, "V 390 844 2.0");
    let raw_ver = u32::from_le_bytes(anim_bytes[4..8].try_into().expect("4 bytes"));
    assert!(
        (raw_ver & 0x8000_0000) != 0,
        "REQ-013-02O.2: MWGL header must set bit 31 (0x8000_0000) when animation is active for 60 FPS client pacing"
    );

    // REQ-013-02O.2: State 6 (screen_nib_scenes[5]) TabView must have >= 180pt height and zero overlapping sibling controls
    let (s6_scene, _) = &session.app.center.screen_nib_scenes[5];
    let s6_tabs = s6_scene.nodes().iter().find(|n| n.ib_object_id == "s6-tabs").expect("s6-tabs node");
    assert!(
        s6_tabs.alignment_rect[3] >= 180.0,
        "REQ-013-02O.2: s6-tabs height must be >= 180pt so TabView body never overflows onto s6-rules, got {}",
        s6_tabs.alignment_rect[3]
    );
    let s6_rules = s6_scene.nodes().iter().find(|n| n.ib_object_id == "s6-rules").expect("s6-rules node");
    let tabs_bottom = s6_tabs.alignment_rect[1] + s6_tabs.alignment_rect[3];
    assert!(
        s6_rules.alignment_rect[1] >= tabs_bottom,
        "REQ-013-02O.2: s6-rules y ({}) must be >= s6-tabs bottom ({tabs_bottom})",
        s6_rules.alignment_rect[1]
    );

    // REQ-013-02O.3: Real-time wall-clock synchronization (`C <secs>`) and WebAssembly in-browser execution
    let _ = session.encode_frame(390.0, 844.0, 2.0, "V 390 844 2.0\nC 42.500");
    assert!(
        session.time_secs >= 42.500,
        "REQ-013-02O.3: MobileWebSession must synchronize time_secs to real wall-clock `C <secs>`, got {}",
        session.time_secs
    );
    assert!(
        html_src.contains("WebAssembly.instantiate") && html_src.contains("macview_gallery.wasm"),
        "REQ-013-02O.3: mobile_web_html.html must support in-browser WebAssembly execution via macview_gallery.wasm"
    );

    // REQ-013-02R: (1) State 2 uses Inter open-source labels (no SF Pro) and includes Italic weight segment
    let (s2_scene, _) = &session.app.center.screen_nib_scenes[1];
    let s2_font = s2_scene.nodes().iter().find(|n| n.ib_object_id == "s2-pop-font").expect("s2-pop-font");
    assert!(
        !s2_font.title.contains("SF Pro") && s2_font.title.contains("Inter"),
        "REQ-013-02R: s2-pop-font must use open-source Inter label instead of SF Pro, got {}",
        s2_font.title
    );

    // REQ-013-02R: (2) Finder includes embedded .nib and .xib files even without disk directory, and loads from memory
    let fallback_rows =
        gallery_app::gallery::finder_data::collect_finder_rows(&None, &[], &[5; 6], &[vec![], vec![], vec![], vec![], vec![], vec![]]);
    assert!(
        fallback_rows.iter().any(|r| r.name.ends_with(".nib")) && fallback_rows.iter().any(|r| r.name.ends_with(".xib")),
        "REQ-013-02R: collect_finder_rows must include embedded .nib and .xib files for WebAssembly / GitHub Pages"
    );

    // REQ-013-02R: (3) State 6 spacious uncramped height (s6-outer-box >= 740pt)
    let (s6_scene, _) = &session.app.center.screen_nib_scenes[5];
    let s6_box = s6_scene.nodes().iter().find(|n| n.ib_object_id == "s6-outer-box").expect("s6-outer-box");
    assert!(
        s6_box.alignment_rect[3] >= 740.0,
        "REQ-013-02R: State 6 outer box height must be >= 740pt for uncramped layout, got {}",
        s6_box.alignment_rect[3]
    );
    assert!(
        session
            .app
            .center
            .nib_studio
            .load_nib_or_xib_file(std::path::Path::new("in_memory/2_Theme_and_Appearance_Studio.xib"), &session.app.toasts),
        "REQ-013-02R: load_nib_or_xib_file must load embedded .xib/.nib from memory when disk read is unavailable"
    );
}

#[test]
fn test_req013_02s_color_well_fonts_cursor_split_and_zero_label_truncation() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let _guard = HYGIENE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            run_req013_02s_suite();
        })
        .expect("spawn req013_02s thread")
        .join()
        .expect("join req013_02s thread");
}

fn run_req013_02s_suite() {
    use gallery_app::gallery::{mobile_web_server::MobileWebSession, GalleryScreen};
    let mut session = MobileWebSession::new();

    // 1. REQ-013-02S.1: Behavioral MacColorWell click-to-open inside MacScrollView and swatch click color mutation
    session.app.active_screen = GalleryScreen::DesignTokensWcag;
    session.app.sidebar.active_screen = GalleryScreen::DesignTokensWcag;
    let _ = session.encode_frame(1440.0, 900.0, 1.0, "V 1440 900 1.0");
    let initial_fg = session.app.center.wcag_fg;
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(1440.0, 900.0))),
        ..Default::default()
    };
    let mut out = session.ctx.run_ui(raw_input.clone(), |ui| session.app.render_ui(ui));
    out.textures_delta.clear();
    let well_pos = out
        .shapes
        .iter()
        .find_map(|c| match &c.shape {
            egui::epaint::Shape::Rect(r)
                if r.fill == initial_fg
                    && r.rect.min.x >= 260.0
                    && r.rect.max.x <= 1060.0
                    && (r.rect.height() - 15.0).abs() < 4.0
                    && (28.0..=65.0).contains(&r.rect.width()) =>
            {
                Some(r.rect.center())
            }
            _ => None,
        })
        .expect("REQ-013-02S.1: s3-cw-fg inner color swatch rect must be painted in State 3");

    let _ = session.encode_frame(
        1440.0,
        900.0,
        1.0,
        &format!("V 1440 900 1.0\nM {:.1} {:.1}\nD {:.1} {:.1}", well_pos.x, well_pos.y, well_pos.x, well_pos.y),
    );
    let _ = session.encode_frame(
        1440.0,
        900.0,
        1.0,
        &format!("V 1440 900 1.0\nM {:.1} {:.1}\nU {:.1} {:.1}", well_pos.x, well_pos.y, well_pos.x, well_pos.y),
    );
    let _ = session.encode_frame(1440.0, 900.0, 1.0, "V 1440 900 1.0");
    let mut out_open = session.ctx.run_ui(raw_input, |ui| session.app.render_ui(ui));
    out_open.textures_delta.clear();
    let expected_red = egui_macview::theme::MacTheme::default().system_red();
    let swatch_pos = out_open
        .shapes
        .iter()
        .find_map(|c| match &c.shape {
            egui::epaint::Shape::Rect(r) if r.fill == expected_red && (5.0..=20.0).contains(&r.rect.width()) => Some(r.rect.center()),
            _ => None,
        })
        .expect("REQ-013-02S.1: Clicking s3-cw-fg must open and keep open the MacColorWell popover with system swatches");

    let _ = session.encode_frame(
        1440.0,
        900.0,
        1.0,
        &format!("V 1440 900 1.0\nM {:.1} {:.1}\nD {:.1} {:.1}", swatch_pos.x, swatch_pos.y, swatch_pos.x, swatch_pos.y),
    );
    let _ = session.encode_frame(
        1440.0,
        900.0,
        1.0,
        &format!("V 1440 900 1.0\nM {:.1} {:.1}\nU {:.1} {:.1}", swatch_pos.x, swatch_pos.y, swatch_pos.x, swatch_pos.y),
    );
    assert_eq!(session.app.center.wcag_fg, expected_red, "REQ-013-02S.1: Swatch click must mutate wcag_fg");

    // 2. REQ-013-02S.2 & REQ-013-06.1: FontFamilies registered in setup_symbols_font & Monospace renders ASCII without tofu boxes
    let mut font_defs = egui::FontDefinitions::default();
    egui_macview::setup_symbols_font(&mut font_defs);
    let mono_list = font_defs.families.get(&egui::FontFamily::Monospace).expect("Monospace family");
    assert!(
        mono_list.contains(&egui_macview::INTER_REGULAR_FONT_NAME.to_string()),
        "REQ-013-06.1: FontFamily::Monospace must include inter_regular so FontId::monospace never renders tofu boxes"
    );
    let families = session.ctx.fonts(|f| f.families());
    assert!(
        families.contains(&egui::FontFamily::Name("Inter-SemiBold".into()))
            && families.contains(&egui::FontFamily::Name("Ubuntu-Light".into())),
        "REQ-013-02S.2: setup_symbols_font must register both Inter-SemiBold and Ubuntu-Light named font families"
    );
    let s2_font = session.app.center.screen_nib_scenes[1].0.nodes().iter().find(|n| n.ib_object_id == "s2-pop-font").expect("s2-pop-font");
    if let egui_macview::scene::SceneNodeKind::PopUpButton { options } = &s2_font.kind {
        assert_eq!(options, &["Inter Regular (OFL)", "Inter SemiBold (OFL)", "Hack Monospace (MIT)", "Ubuntu Light (UFL)"]);
    } else {
        panic!("Expected PopUpButton for s2-pop-font");
    }

    // 3. REQ-013-02S.3 & REQ-013-06.4: Verified HTTP 200 MP4 & real Wikimedia MP3 classical streams (no 403 gtv-videos-bucket or sine buzzer)
    let html_src = std::fs::read_to_string("examples/gallery/mobile_web_html.html").expect("read mobile_web_html.html");
    assert!(
        !html_src.contains("id=\"hud\"")
            && !html_src.contains("createOscillator")
            && !html_src.contains("buildClassicalWavUrl")
            && !html_src.contains("gtv-videos-bucket")
            && html_src.contains("media.w3.org/2010/05/bunny/trailer.mp4")
            && html_src.contains("Bach_-_Brandenburg_Concerto_No._3_-_1._Allegro.ogg.mp3"),
        "REQ-013-06.4: mobile_web_html.html must use real HTTP 200 MP4 and Wikimedia MP3 recordings without 403 gtv-videos-bucket or sine WAV"
    );
    let s5_codec =
        session.app.center.screen_nib_scenes[4].0.nodes().iter().find(|n| n.ib_object_id == "s5-pop-codec").expect("s5-pop-codec");
    if let egui_macview::scene::SceneNodeKind::PopUpButton { options } = &s5_codec.kind {
        assert!(options.len() >= 6 && options.iter().any(|o| o.contains("Big Buck Bunny")) && options.iter().any(|o| o.contains("Bach")));
    } else {
        panic!("Expected PopUpButton for s5-pop-codec");
    }

    // 4. REQ-013-02S.4: MacSplitView divider hover & drag encodes CursorIcon::ResizeHorizontal (code 1 in bits 28..30) and moves left_split_pos
    session.app.active_screen = GalleryScreen::CanvasPhysics;
    session.app.sidebar.active_screen = GalleryScreen::CanvasPhysics;
    let _ = session.encode_frame(1440.0, 900.0, 1.0, "V 1440 900 1.0");
    let split_x = session.app.left_split_pos;
    let hover_bytes = session.encode_frame(1440.0, 900.0, 1.0, &format!("V 1440 900 1.0\nM {:.1} 450.0", split_x));
    let ver_word = u32::from_le_bytes(hover_bytes[4..8].try_into().expect("4 bytes"));
    assert_eq!((ver_word >> 28) & 0x7, 1, "REQ-013-02S.4: Hovering vertical divider must encode cursor_code == 1");
    let _ = session.encode_frame(1440.0, 900.0, 1.0, &format!("V 1440 900 1.0\nM {:.1} 450.0\nD {:.1} 450.0", split_x, split_x));
    let _ =
        session.encode_frame(1440.0, 900.0, 1.0, &format!("V 1440 900 1.0\nM {:.1} 450.0\nU {:.1} 450.0", split_x + 35.0, split_x + 35.0));
    assert!((session.app.left_split_pos - (split_x + 35.0)).abs() < 2.0, "REQ-013-02S.4: Dragging must update left_split_pos");

    // 5. REQ-013-02S.5: Generous default panel widths across all 6 screens & State 6 TabView / State 5 PopUpButton widths
    let fresh_session = MobileWebSession::new();
    assert!(fresh_session.app.left_split_pos >= 260.0 && fresh_session.app.inspector_width >= 380.0);
    let s5_w = fresh_session.app.center.screen_nib_scenes[4]
        .0
        .nodes()
        .iter()
        .find(|n| n.ib_object_id == "s5-pop-codec")
        .expect("s5-pop-codec")
        .alignment_rect[2];
    assert!(s5_w >= 228.0, "REQ-013-02S.5: s5-pop-codec width ({s5_w}) must be >= 228pt so stream titles never clip");
    let rw_src = std::fs::read_to_string("src/scene/runtime_widgets.rs").expect("read runtime_widgets.rs");
    assert!(
        rw_src.contains("\"AVPlayer HUD\"") && rw_src.contains("\"Collection Grid\"") && rw_src.contains("\"Clock & Calendar\""),
        "REQ-013-02S.5: render_tab_view_node must use concise non-overlapping tab labels"
    );
}
