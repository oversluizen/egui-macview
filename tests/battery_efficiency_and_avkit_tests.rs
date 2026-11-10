//! Integration and unit tests for Battery Efficiency (7 leaks) and AVKit Audio Loop Sync.
//!
//! Grounded in GL-1015 (failing red COTS test assertions first, zero unused dependencies,
//! file size ceilings, zero idle repaint invariant, and robust edge-case verification).

#[allow(dead_code)]
#[path = "../examples/macview_gallery.rs"]
mod gallery_app;

use egui::{Context, Pos2, RawInput, Rect, Vec2, ViewportId, ViewportInfo};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    MacProgressControlSize, MacProgressIndicator, MacTableColumn, MacTableView,
};
pub(crate) use gallery_app::{
    format_tracked_small_caps, CardModel, ExpandedProTab, GalleryApp, GalleryScreen, InspectorMode,
};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn run_ui_with_input(
    ctx: &Context,
    mut raw: RawInput,
    mut f: impl FnMut(&mut egui::Ui),
) -> Duration {
    if raw.screen_rect.is_none() {
        raw.screen_rect = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0)));
    }
    let mut out = ctx.run_ui(raw, |ui| f(ui));
    out.textures_delta.clear();
    out.viewport_output
        .get(&ViewportId::ROOT)
        .map(|v| v.repaint_delay)
        .unwrap_or(Duration::MAX)
}

fn run_app_frame(
    ctx: &Context,
    app: &mut GalleryApp,
    time: f64,
    focused: bool,
    minimized: bool,
) -> Duration {
    let mut raw = RawInput {
        time: Some(time),
        focused,
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1160.0, 740.0))),
        ..Default::default()
    };
    let vp = ViewportInfo {
        focused: Some(focused),
        minimized: Some(minimized),
        ..Default::default()
    };
    raw.viewports.insert(ViewportId::ROOT, vp);
    let mut out = ctx.run_ui(raw, |ui| app.render_ui(ui));
    out.textures_delta.clear();
    out.viewport_output
        .get(&ViewportId::ROOT)
        .map(|v| v.repaint_delay)
        .unwrap_or(Duration::MAX)
}

#[test]
fn test_indeterminate_progress_unfocused_minimized_zero_repaint() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    // 1. Unfocused window with indeterminate bar: MUST NOT request 33ms repaint
    let mut raw_unfocused = RawInput {
        focused: false,
        ..Default::default()
    };
    let vp_unfocused = ViewportInfo {
        focused: Some(false),
        ..Default::default()
    };
    raw_unfocused
        .viewports
        .insert(ViewportId::ROOT, vp_unfocused);

    // Settle 3 frames so egui layout/font cache reaches rest
    let mut t = 1.0;
    for _ in 0..3 {
        t += 0.05;
        raw_unfocused.time = Some(t);
        let _ = run_ui_with_input(&ctx, raw_unfocused.clone(), |ui| {
            MacProgressIndicator::bar(0.5)
                .theme(&theme)
                .indeterminate(1.0)
                .show(ui);
        });
    }

    t += 0.05;
    raw_unfocused.time = Some(t);
    let delay_unfocused = run_ui_with_input(&ctx, raw_unfocused, |ui| {
        MacProgressIndicator::bar(0.5)
            .theme(&theme)
            .indeterminate(1.0)
            .show(ui);
    });
    assert_eq!(
        delay_unfocused,
        Duration::MAX,
        "GL-1015 Leak 3: Unfocused window must not schedule 33ms indeterminate repaints"
    );

    // 2. Minimized window with spinning indicator: MUST NOT request repaints
    let mut raw_minimized = RawInput {
        focused: true,
        ..Default::default()
    };
    let vp = ViewportInfo {
        minimized: Some(true),
        ..Default::default()
    };
    raw_minimized.viewports.insert(ViewportId::ROOT, vp);

    for _ in 0..3 {
        t += 0.05;
        raw_minimized.time = Some(t);
        let _ = run_ui_with_input(&ctx, raw_minimized.clone(), |ui| {
            MacProgressIndicator::spinning(0.5)
                .theme(&theme)
                .control_size(MacProgressControlSize::Regular)
                .show(ui);
        });
    }

    t += 0.05;
    raw_minimized.time = Some(t);
    let delay_minimized = run_ui_with_input(&ctx, raw_minimized, |ui| {
        MacProgressIndicator::spinning(0.5)
            .theme(&theme)
            .control_size(MacProgressControlSize::Regular)
            .show(ui);
    });
    assert_eq!(
        delay_minimized,
        Duration::MAX,
        "GL-1015 Leak 3: Minimized window must not schedule spinning indicator repaints"
    );

    // 3. Focused & visible window: MUST request animation repaints (< 100ms)
    let raw_active = RawInput {
        time: Some(2.0),
        focused: true,
        ..Default::default()
    };
    let delay_active = run_ui_with_input(&ctx, raw_active, |ui| {
        MacProgressIndicator::bar(0.5)
            .theme(&theme)
            .indeterminate(1.0)
            .show(ui);
    });
    assert!(
        delay_active <= Duration::from_millis(50),
        "Active focused window must schedule smooth indeterminate animation"
    );
}

#[test]
fn test_avkit_playback_does_not_flood_undo_stack() {
    let ctx = Context::default();
    let mut app = GalleryApp::new();
    app.active_screen = GalleryScreen::AvKitMediaStudio;
    app.sidebar.active_screen = GalleryScreen::AvKitMediaStudio;
    app.sidebar.selected_node = GalleryScreen::AvKitMediaStudio.title().to_string();

    // Settle transitions
    let mut t = 10.0;
    for _ in 0..5 {
        t += 0.2;
        let _ = run_app_frame(&ctx, &mut app, t, true, false);
    }

    // Clear any startup undo snapshots
    app.center.avkit_undo.clear();
    assert_eq!(app.center.avkit_undo.undo_count(), 0);

    // Start playback: running 12 frames of active playback
    app.center.media_playing = true;
    for _ in 0..12 {
        t += 0.016;
        let _ = run_app_frame(&ctx, &mut app, t, true, false);
    }

    // Playback playhead advancement MUST NOT push undo snapshots
    assert_eq!(
        app.center.avkit_undo.undo_count(),
        0,
        "GL-1015 Leak 2: 60fps playhead motion must not flood NSUndoManager (found {} snapshots)",
        app.center.avkit_undo.undo_count()
    );

    // Explicit studio parameter change (e.g. gain) DOES push undo
    app.center.media_playing = false;
    app.center.media_gain = 1.75;
    t += 0.1;
    let _ = run_app_frame(&ctx, &mut app, t, true, false);
    assert_eq!(
        app.center.avkit_undo.undo_count(),
        1,
        "Manual parameter adjustment must push exactly one undo snapshot"
    );
}

#[test]
fn test_table_view_viewport_row_culling() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let mut sel = BTreeSet::new();
    let cols = [
        MacTableColumn::new("Col A", 120.0),
        MacTableColumn::new("Col B", 120.0),
    ];
    let cell_invocation_count = Arc::new(AtomicUsize::new(0));

    let raw = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
        ..Default::default()
    };

    let invocations = cell_invocation_count.clone();
    let _ = run_ui_with_input(&ctx, raw, |ui| {
        // Restrict clip_rect to a 100pt high viewport
        ui.set_clip_rect(Rect::from_min_size(
            Pos2::new(0.0, 0.0),
            Vec2::new(300.0, 100.0),
        ));
        let count_ref = invocations.clone();
        MacTableView::new("culling_table_test", &cols, 1000, &mut sel)
            .theme(&theme)
            .row_height(24.0)
            .show(ui, |_cell_ui, _row, _col| {
                count_ref.fetch_add(1, Ordering::SeqCst);
            });
    });

    let total_invocations = cell_invocation_count.load(Ordering::SeqCst);
    assert!(
        total_invocations <= 30,
        "GL-1015 Leak 4: Table with 1000 rows in 100pt clip_rect must cull offscreen rows (got {total_invocations} cell calls)"
    );
}

#[test]
fn test_avkit_audio_track_duration_1_to_1_and_loop_restart() {
    use gallery_app::gallery::audio_media::AudioTrack;

    // Default procedural studio track MUST have 1:1 matching sample buffer and duration
    let track = AudioTrack::default();
    let sample_duration = track.samples_mono.len() as f64 / track.sample_rate as f64;
    assert!(
        (sample_duration - track.duration_secs).abs() < 0.25,
        "GL-1015 AVKit Bug: Procedural audio buffer ({sample_duration:.1}s) must match track duration ({:.1}s) 1-to-1",
        track.duration_secs
    );

    // Audio mixing with gain and pan
    let wav_mixed = track.to_wav_bytes_mixed(0.0, 2.0, 1.5, -0.5);
    assert!(
        wav_mixed.len() > 44,
        "WAV mixed output must contain standard 44-byte RIFF header"
    );
    assert_eq!(&wav_mixed[0..4], b"RIFF");
    assert_eq!(&wav_mixed[8..12], b"WAVE");

    // Loop boundary resets playhead to in_point in AVKit studio and clears peak meters
    let ctx = Context::default();
    let mut app = GalleryApp::new();
    app.active_screen = GalleryScreen::AvKitMediaStudio;
    app.center.media_playing = true;
    app.center.media_loop = true;
    app.center.media_in_point = 4.0;
    app.center.media_out_point = 26.0;
    app.center.media_playhead = 25.99;
    app.center.audio_peak_l = 9.5;
    app.center.audio_peak_r = 9.5;

    let mut t = 30.0;
    let _ = run_app_frame(&ctx, &mut app, t, true, false);
    t += 0.1;
    let _ = run_app_frame(&ctx, &mut app, t, true, false);

    assert!(
        app.center.media_playhead < 25.0,
        "GL-1015 AVKit Bug: Playhead must loop back to in_point (got playhead={})",
        app.center.media_playhead
    );
    assert!(
        app.center.media_playing,
        "Media must continue playing when media_loop is true"
    );

    // Procedural track duration clamping ensures 1:1 duration vs sample buffer match
    let clamped_track = AudioTrack::procedural_studio_track("Clamped", 200.0);
    let clamped_sample_dur =
        clamped_track.samples_mono.len() as f64 / clamped_track.sample_rate as f64;
    assert!(
        (clamped_track.duration_secs - clamped_sample_dur).abs() < 0.25,
        "Clamped track duration ({}) must match sample buffer duration ({})",
        clamped_track.duration_secs,
        clamped_sample_dur
    );
}

#[test]
fn test_avkit_pauses_when_unfocused_and_resumes_cleanly_on_refocus() {
    let ctx = Context::default();
    let mut app = GalleryApp::new();
    app.active_screen = GalleryScreen::AvKitMediaStudio;

    // Settle startup scrollbar fade timers to rest
    let mut t = 50.0;
    let _ = run_app_frame(&ctx, &mut app, t, true, false);
    t += 1.5;
    let _ = run_app_frame(&ctx, &mut app, t, true, false);

    app.center.media_playing = true;
    app.center.media_playhead = 10.0;

    // Frame 1 focused: active 60fps pacing
    t += 0.05;
    let delay_active = run_app_frame(&ctx, &mut app, t, true, false);
    assert!(
        delay_active <= Duration::from_millis(50),
        "Active focused window must schedule smooth timeline repaints"
    );

    // Frame 2 unfocused: timeline and audio must pause (Duration::MAX) for 0.0% background CPU
    t += 0.05;
    let delay_unfocused = run_app_frame(&ctx, &mut app, t, false, false);
    assert_eq!(
        delay_unfocused,
        Duration::MAX,
        "Unfocused playing window must pause timeline repaints for 0.0% CPU"
    );
    let paused_pos = app.center.media_playhead;

    // Advance wall clock 5.0 seconds while unfocused: playhead must remain at paused_pos
    t += 5.0;
    let delay_unfocused_later = run_app_frame(&ctx, &mut app, t, false, false);
    assert_eq!(delay_unfocused_later, Duration::MAX);
    assert_eq!(app.center.media_playhead, paused_pos);

    // Refocus window: timeline repaints resume immediately without large dt jump
    t += 0.05;
    let delay_refocused = run_app_frame(&ctx, &mut app, t, true, false);
    assert!(
        delay_refocused <= Duration::from_millis(50),
        "Refocused window must resume timeline repaints"
    );
    // Next frame advances smoothly
    t += 0.016;
    let _ = run_app_frame(&ctx, &mut app, t, true, false);
    assert!(
        app.center.media_playhead > paused_pos && app.center.media_playhead < paused_pos + 0.1,
        "Playhead must advance smoothly on refocus without large dt jump (got playhead={})",
        app.center.media_playhead
    );
}

#[test]
fn test_finder_display_rows_are_cached_at_rest() {
    let ctx = Context::default();
    let mut app = GalleryApp::new();
    app.active_screen = GalleryScreen::FinderDataBrowser;

    let t = 40.0;
    let _ = run_app_frame(&ctx, &mut app, t, true, false);
    assert!(
        !app.center.finder_cached_rows.is_empty(),
        "GL-1015 Leak 1: Finder display rows must be populated in finder_cached_rows"
    );
    let initial_len = app.center.finder_cached_rows.len();

    // Second frame without state changes must reuse cached rows
    let _ = run_app_frame(&ctx, &mut app, t + 0.1, true, false);
    assert_eq!(
        app.center.finder_cached_rows.len(),
        initial_len,
        "Cached rows must be preserved on idle frames"
    );
}

#[test]
fn test_rating_indicator_precomputed_unit_star_geometry() {
    use egui_macview::UNIT_STAR;
    assert_eq!(UNIT_STAR.len(), 10);
    // Tip 0: (0.0, -1.0)
    assert_eq!(UNIT_STAR[0], (0.0, -1.0));
    // Inner indentation 5: (0.0, 0.4)
    assert!((UNIT_STAR[5].0 - 0.0).abs() < 1e-4);
    assert!((UNIT_STAR[5].1 - 0.4).abs() < 1e-4);
    // Check symmetry: x of tip 4 is opposite of tip 6
    assert!((UNIT_STAR[4].0 + UNIT_STAR[6].0).abs() < 1e-4);
}

#[test]
fn test_audio_meter_memoization_at_rest() {
    let ctx = Context::default();
    let mut app = GalleryApp::new();
    app.active_screen = GalleryScreen::AvKitMediaStudio;
    app.center.media_playing = false;

    // Run frame A
    let _ = run_app_frame(&ctx, &mut app, 10.0, true, false);
    let cached_levels = app.center.audio_levels_cached;
    assert!(cached_levels.0 >= 0.0);

    // Frame B with unchanged playhead/gain/pan must keep identical cached levels
    let _ = run_app_frame(&ctx, &mut app, 10.1, true, false);
    assert_eq!(app.center.audio_levels_cached, cached_levels);
}
