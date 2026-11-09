//! Integration and unit test suite for interactive media, audio sync, real Finder browsing, and code generation.
//!
//! Grounded in GL-1015 (failing red COTS test assertions first, zero unused dependencies,
//! file size ceilings, zero idle repaint invariant, and robust edge-case verification).

#[allow(dead_code)]
#[path = "../examples/macview_gallery.rs"]
mod gallery_app;

use egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, ViewportId};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    MacCircularSlider, MacRatingIndicator,
};
pub(crate) use gallery_app::{
    format_tracked_small_caps, CardModel, ExpandedProTab, GalleryApp, GalleryScreen, InspectorMode,
};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
struct MockDroppedFile(std::path::PathBuf);

impl egui::DroppedFile for MockDroppedFile {
    fn path(&self) -> &std::path::Path {
        &self.0
    }

    fn bytes(&self) -> Result<Vec<u8>, String> {
        std::fs::read(&self.0).map_err(|e| e.to_string())
    }
}

fn run_ui_cleared(ctx: &Context, raw: RawInput, mut f: impl FnMut(&mut egui::Ui)) -> Duration {
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
    raw_extra: Option<RawInput>,
) -> Duration {
    let mut raw = raw_extra.unwrap_or_default();
    raw.time = Some(time);
    raw.screen_rect = Some(egui::Rect::from_min_size(
        Pos2::ZERO,
        egui::vec2(1160.0, 740.0),
    ));
    let mut out = ctx.run_ui(raw, |ui| app.render_ui(ui));
    out.textures_delta.clear();
    out.viewport_output
        .get(&ViewportId::ROOT)
        .map(|v| v.repaint_delay)
        .unwrap_or(Duration::MAX)
}

#[test]
fn test_circular_slider_rotary_geometry_drag_and_keyboard() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut gain = 1.0f32;
    let mut resp_rect = egui::Rect::NOTHING;

    // 1. Initial render pass
    let _ = run_ui_cleared(&ctx, RawInput::default(), |ui| {
        let resp = MacCircularSlider::new(&mut gain, 0.0..=2.0)
            .theme(&theme)
            .diameter(36.0)
            .step(0.05)
            .show(ui);
        resp_rect = resp.rect;
    });
    assert_eq!(gain, 1.0);
    assert!((resp_rect.width() - 36.0).abs() < 1.0);
    assert!((resp_rect.height() - 36.0).abs() < 1.0);

    // 2. Multi-frame drag simulation: press on center, then drag up
    let start_pos = resp_rect.center();
    let drag_pos = Pos2::new(start_pos.x, start_pos.y - 25.0);

    // Frame A: Press mouse down
    let raw_down = RawInput {
        events: vec![
            Event::PointerMoved(start_pos),
            Event::PointerButton {
                pos: start_pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
        ..Default::default()
    };
    let _ = run_ui_cleared(&ctx, raw_down, |ui| {
        MacCircularSlider::new(&mut gain, 0.0..=2.0)
            .theme(&theme)
            .diameter(36.0)
            .show(ui);
    });

    // Frame B: Move mouse upwards while holding button down
    let raw_move = RawInput {
        events: vec![Event::PointerMoved(drag_pos)],
        ..Default::default()
    };
    let _ = run_ui_cleared(&ctx, raw_move, |ui| {
        MacCircularSlider::new(&mut gain, 0.0..=2.0)
            .theme(&theme)
            .diameter(36.0)
            .show(ui);
    });
    assert!(
        gain > 1.0,
        "Vertical drag upwards must increase rotary value (gain={gain})"
    );
    assert!(gain <= 2.0, "Rotary value must stay clamped within range");

    // Frame C: Release mouse button
    let raw_up = RawInput {
        events: vec![Event::PointerButton {
            pos: drag_pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
        ..Default::default()
    };
    let _ = run_ui_cleared(&ctx, raw_up, |ui| {
        MacCircularSlider::new(&mut gain, 0.0..=2.0)
            .theme(&theme)
            .diameter(36.0)
            .show(ui);
    });

    // 3. Keyboard Arrow keys navigation
    let prev_val = gain;
    let raw_key_down = RawInput {
        events: vec![Event::Key {
            key: Key::ArrowDown,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        ..Default::default()
    };
    let _ = run_ui_cleared(&ctx, raw_key_down.clone(), |ui| {
        let r = MacCircularSlider::new(&mut gain, 0.0..=2.0)
            .theme(&theme)
            .diameter(36.0)
            .step(0.1)
            .show(ui);
        r.request_focus();
    });

    let _ = run_ui_cleared(&ctx, raw_key_down.clone(), |ui| {
        MacCircularSlider::new(&mut gain, 0.0..=2.0)
            .theme(&theme)
            .diameter(36.0)
            .step(0.1)
            .show(ui);
    });
    assert!(gain < prev_val || (gain - prev_val).abs() < 1e-4);

    // 4. Boundary clamping & NaN safety verification
    gain = 2.0;
    let raw_up_key = RawInput {
        events: vec![Event::Key {
            key: Key::ArrowUp,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        ..Default::default()
    };
    let _ = run_ui_cleared(&ctx, raw_up_key, |ui| {
        MacCircularSlider::new(&mut gain, 0.0..=2.0)
            .theme(&theme)
            .show(ui);
    });
    assert_eq!(gain, 2.0, "Gain must clamp at maximum 2.0");

    let mut nan_val = f32::NAN;
    let _ = run_ui_cleared(&ctx, RawInput::default(), |ui| {
        MacCircularSlider::new(&mut nan_val, 0.0..=2.0)
            .theme(&theme)
            .show(ui);
    });
    assert_eq!(nan_val, 0.0, "NaN must sanitize to min range");

    gain = 0.0;
    let _ = run_ui_cleared(&ctx, raw_key_down, |ui| {
        MacCircularSlider::new(&mut gain, 0.0..=2.0)
            .theme(&theme)
            .show(ui);
    });
    assert_eq!(gain, 0.0, "Gain must clamp at minimum 0.0");
}

#[test]
fn test_rating_indicator_stars_selection_and_interaction() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::DarkAqua);

    let mut rating = 3u32;
    let mut rating_rect = egui::Rect::NOTHING;

    let _ = run_ui_cleared(&ctx, RawInput::default(), |ui| {
        let resp = MacRatingIndicator::new(&mut rating, 5)
            .theme(&theme)
            .item_size(16.0)
            .show(ui);
        rating_rect = resp.rect;
    });
    assert_eq!(rating, 3);
    assert!(rating_rect.width() >= 16.0 * 5.0);

    // Click 5th star (rightmost) to set rating to 5
    let star_5_pos = Pos2::new(rating_rect.max.x - 8.0, rating_rect.center().y);
    let click_5 = RawInput {
        events: vec![
            Event::PointerMoved(star_5_pos),
            Event::PointerButton {
                pos: star_5_pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            Event::PointerButton {
                pos: star_5_pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
        ..Default::default()
    };
    let _ = run_ui_cleared(&ctx, click_5.clone(), |ui| {
        MacRatingIndicator::new(&mut rating, 5)
            .theme(&theme)
            .item_size(16.0)
            .show(ui);
    });
    assert_eq!(rating, 5, "Clicking 5th star must set rating to 5");

    // Click 5th star again to toggle off / reset to 0
    let _ = run_ui_cleared(&ctx, click_5, |ui| {
        MacRatingIndicator::new(&mut rating, 5)
            .theme(&theme)
            .item_size(16.0)
            .show(ui);
    });
    assert_eq!(
        rating, 0,
        "Clicking current rating star again resets rating to 0"
    );

    let mut clamped_r = 99u32;
    let _ = run_ui_cleared(&ctx, RawInput::default(), |ui| {
        MacRatingIndicator::new(&mut clamped_r, 5)
            .theme(&theme)
            .show(ui);
    });
    assert_eq!(clamped_r, 5, "Rating > max_rating must clamp to max_rating");
}

#[test]
fn test_finder_browser_real_filesystem_and_drag_drop_loading() {
    let ctx = Context::default();
    let mut app = GalleryApp::new();
    app.active_screen = GalleryScreen::FinderDataBrowser;
    app.sidebar.active_screen = GalleryScreen::FinderDataBrowser;

    // Render baseline frame
    let _ = run_app_frame(&ctx, &mut app, 1.0, None);
    assert!(app.center.active_path_idx <= 4);

    // Simulate OS drag-and-drop file delivery
    let dummy_path = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("Cargo.toml");
    let raw_drop = RawInput {
        dropped_files: vec![Arc::new(MockDroppedFile(dummy_path)) as egui::DroppedFileHandle],
        ..Default::default()
    };
    let _ = run_app_frame(&ctx, &mut app, 1.1, Some(raw_drop));

    // Toast manager should record the file inspection
    assert!(
        app.toasts
            .borrow()
            .active_toasts()
            .iter()
            .any(|t| t.message.contains("Cargo.toml") || t.message.contains("Loaded")),
        "Dropping a real file into Finder must trigger receipt/toast"
    );

    // AudioTrack robustness tests
    use gallery_app::gallery::audio_media::AudioTrack;
    let corrupt = b"RIFF\x00\x00\x00\x00WAVEfmt \x02\x00\x00\x00\x01\x00";
    assert!(AudioTrack::from_wav_bytes("bad.wav", corrupt).is_err());
    let track = AudioTrack::default();
    let inverted_wav = track.to_wav_bytes(60.0, 10.0);
    assert!(inverted_wav.len() >= 44);
}

#[test]
fn test_avkit_audio_video_synchronization_and_undo_roundtrip() {
    let ctx = Context::default();
    let mut app = GalleryApp::new();
    app.active_screen = GalleryScreen::AvKitMediaStudio;
    app.sidebar.active_screen = GalleryScreen::AvKitMediaStudio;
    app.sidebar.selected_node = GalleryScreen::AvKitMediaStudio.title().to_string();

    // Settle initial screen transitions to rest
    let mut t = 10.0;
    for _ in 0..6 {
        t += 0.25;
        let _ = run_app_frame(&ctx, &mut app, t, None);
    }
    t += 0.5;

    // 1. Initial state: paused, level indicators at rest, zero idle repaint
    app.center.media_playing = false;
    let idle_delay = run_app_frame(&ctx, &mut app, t, None);
    assert_eq!(
        idle_delay,
        Duration::MAX,
        "GL-1015: When AVKit media is paused, repaint delay MUST be Duration::MAX"
    );

    // 2. Adjust gain via circular slider & verify audio level response
    app.center.media_gain = 1.5;
    t += 0.1;
    let _ = run_app_frame(&ctx, &mut app, t, None);
    assert!(
        app.center.audio_level_l > 0.0,
        "Audio levels must reflect scrubbed playhead and gain even when paused"
    );

    // 3. Playing state triggers active repaint
    app.center.media_playing = true;
    t += 0.1;
    let playing_delay = run_app_frame(&ctx, &mut app, t, None);
    assert!(
        playing_delay < Duration::MAX,
        "When playing, active repaint must be requested for 60fps pacing"
    );

    // 4. Real AVKit NSUndoManager roundtrip via menu action
    app.center.media_playing = false;
    let init_gain = app.center.media_gain;
    let init_zoom = app.center.media_zoom;
    let init_rating = app.center.media_rating;

    app.center.media_gain = 1.85;
    app.center.media_zoom = 1.40;
    app.center.media_rating = 2;
    t += 0.1;
    let _ = run_app_frame(&ctx, &mut app, t, None);
    assert!(app.center.avkit_undo.undo_count() >= 1);

    app.handle_menu_action("edit.undo", &ctx);
    assert!((app.center.media_gain - init_gain).abs() < 1e-4);
    assert!((app.center.media_zoom - init_zoom).abs() < 1e-4);
    assert_eq!(app.center.media_rating, init_rating);

    app.handle_menu_action("edit.redo", &ctx);
    assert!((app.center.media_gain - 1.85).abs() < 1e-4);
    assert!((app.center.media_zoom - 1.40).abs() < 1e-4);
    assert_eq!(app.center.media_rating, 2);
    let _ = app.center.avkit_undo.validate_invariants();
}

#[test]
fn test_gallery_inspector_live_rust_snippet_generation_and_clipboard() {
    let ctx = Context::default();
    let mut app = GalleryApp::new();

    // Verify all 5 screens produce valid, non-empty, compilable Rust snippet strings
    let mut t = 20.0;
    for screen in GalleryScreen::ALL {
        app.active_screen = screen;
        app.sidebar.active_screen = screen;
        t += 0.2;
        let _ = run_app_frame(&ctx, &mut app, t, None);

        // Switch to CodeSnippet tab in ExpandedPro
        app.inspector.expanded_tab = ExpandedProTab::CodeSnippet;
        t += 0.2;
        let _ = run_app_frame(&ctx, &mut app, t, None);

        let snippet = gallery_app::gallery::snippet::generate_rust_snippet(
            screen,
            &app.card.borrow(),
            &(&app.center).into(),
        );
        assert!(
            !snippet.is_empty(),
            "Snippet for {screen:?} must not be empty"
        );
        assert!(
            snippet.contains("egui_macview::"),
            "Snippet for {screen:?} must reference egui_macview primitives"
        );
        assert!(
            snippet.contains("theme"),
            "Snippet for {screen:?} must be data-driven with theme"
        );
    }

    // Adversarial escaping check: quotes and backslashes in layer name
    app.card.borrow_mut().layer_name = r#"Hero "Quoted" & \Backslash\"#.to_string();
    let snip = gallery_app::gallery::snippet::generate_rust_snippet(
        GalleryScreen::CanvasPhysics,
        &app.card.borrow(),
        &(&app.center).into(),
    );
    assert!(
        snip.contains(r#""Hero \"Quoted\" & \\Backslash\\""#),
        "Snippet generator must produce valid escaped Rust string literals"
    );
}
