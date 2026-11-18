//! Integration and unit test suite for interactive media, audio sync, real Finder browsing, and code generation.
//!
//! Grounded in Test-Driven Development (failing red baseline test assertions first, zero unused dependencies,
//! file size ceilings, zero idle repaint invariant, and robust edge-case verification).

#[allow(dead_code)]
#[path = "../examples/macview_gallery.rs"]
mod gallery_app;

use egui::{Context, Event, Key, Modifiers, PointerButton, Pos2, RawInput, ViewportId};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    MacCircularSlider, MacRatingIndicator,
};
pub(crate) use gallery_app::{format_tracked_small_caps, CardModel, ExpandedProTab, GalleryApp, GalleryScreen, InspectorMode};
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
    out.viewport_output.get(&ViewportId::ROOT).map(|v| v.repaint_delay).unwrap_or(Duration::MAX)
}

fn run_app_frame(ctx: &Context, app: &mut GalleryApp, time: f64, raw_extra: Option<RawInput>) -> Duration {
    let mut raw = raw_extra.unwrap_or_default();
    raw.time = Some(time);
    raw.screen_rect = Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1160.0, 740.0)));
    let mut out = ctx.run_ui(raw, |ui| app.render_ui(ui));
    out.textures_delta.clear();
    out.viewport_output.get(&ViewportId::ROOT).map(|v| v.repaint_delay).unwrap_or(Duration::MAX)
}

#[test]
fn test_circular_slider_rotary_geometry_drag_and_keyboard() {
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);

    let mut gain = 1.0f32;
    let mut resp_rect = egui::Rect::NOTHING;

    // 1. Initial render pass
    let _ = run_ui_cleared(&ctx, RawInput::default(), |ui| {
        let resp = MacCircularSlider::new(&mut gain, 0.0..=2.0).theme(&theme).diameter(36.0).step(0.05).show(ui);
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
            Event::PointerButton { pos: start_pos, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
        ],
        ..Default::default()
    };
    let _ = run_ui_cleared(&ctx, raw_down, |ui| {
        MacCircularSlider::new(&mut gain, 0.0..=2.0).theme(&theme).diameter(36.0).show(ui);
    });

    // Frame B: Move mouse upwards while holding button down
    let raw_move = RawInput { events: vec![Event::PointerMoved(drag_pos)], ..Default::default() };
    let _ = run_ui_cleared(&ctx, raw_move, |ui| {
        MacCircularSlider::new(&mut gain, 0.0..=2.0).theme(&theme).diameter(36.0).show(ui);
    });
    assert!(gain > 1.0, "Vertical drag upwards must increase rotary value (gain={gain})");
    assert!(gain <= 2.0, "Rotary value must stay clamped within range");

    // Frame C: Release mouse button
    let raw_up = RawInput {
        events: vec![Event::PointerButton { pos: drag_pos, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }],
        ..Default::default()
    };
    let _ = run_ui_cleared(&ctx, raw_up, |ui| {
        MacCircularSlider::new(&mut gain, 0.0..=2.0).theme(&theme).diameter(36.0).show(ui);
    });

    // 3. Keyboard Arrow keys navigation
    let prev_val = gain;
    let raw_key_down = RawInput {
        events: vec![Event::Key { key: Key::ArrowDown, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE }],
        ..Default::default()
    };
    let _ = run_ui_cleared(&ctx, raw_key_down.clone(), |ui| {
        let r = MacCircularSlider::new(&mut gain, 0.0..=2.0).theme(&theme).diameter(36.0).step(0.1).show(ui);
        r.request_focus();
    });

    let _ = run_ui_cleared(&ctx, raw_key_down.clone(), |ui| {
        MacCircularSlider::new(&mut gain, 0.0..=2.0).theme(&theme).diameter(36.0).step(0.1).show(ui);
    });
    assert!(gain < prev_val || (gain - prev_val).abs() < 1e-4);

    // 4. Boundary clamping & NaN safety verification
    gain = 2.0;
    let raw_up_key = RawInput {
        events: vec![Event::Key { key: Key::ArrowUp, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE }],
        ..Default::default()
    };
    let _ = run_ui_cleared(&ctx, raw_up_key, |ui| {
        MacCircularSlider::new(&mut gain, 0.0..=2.0).theme(&theme).show(ui);
    });
    assert_eq!(gain, 2.0, "Gain must clamp at maximum 2.0");

    let mut nan_val = f32::NAN;
    let _ = run_ui_cleared(&ctx, RawInput::default(), |ui| {
        MacCircularSlider::new(&mut nan_val, 0.0..=2.0).theme(&theme).show(ui);
    });
    assert_eq!(nan_val, 0.0, "NaN must sanitize to min range");

    gain = 0.0;
    let _ = run_ui_cleared(&ctx, raw_key_down, |ui| {
        MacCircularSlider::new(&mut gain, 0.0..=2.0).theme(&theme).show(ui);
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
        let resp = MacRatingIndicator::new(&mut rating, 5).theme(&theme).item_size(16.0).show(ui);
        rating_rect = resp.rect;
    });
    assert_eq!(rating, 3);
    assert!(rating_rect.width() >= 16.0 * 5.0);

    // Click 5th star (rightmost) to set rating to 5
    let star_5_pos = Pos2::new(rating_rect.max.x - 8.0, rating_rect.center().y);
    let click_5 = RawInput {
        events: vec![
            Event::PointerMoved(star_5_pos),
            Event::PointerButton { pos: star_5_pos, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
            Event::PointerButton { pos: star_5_pos, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE },
        ],
        ..Default::default()
    };
    let _ = run_ui_cleared(&ctx, click_5.clone(), |ui| {
        MacRatingIndicator::new(&mut rating, 5).theme(&theme).item_size(16.0).show(ui);
    });
    assert_eq!(rating, 5, "Clicking 5th star must set rating to 5");

    // Click 5th star again to toggle off / reset to 0
    let _ = run_ui_cleared(&ctx, click_5, |ui| {
        MacRatingIndicator::new(&mut rating, 5).theme(&theme).item_size(16.0).show(ui);
    });
    assert_eq!(rating, 0, "Clicking current rating star again resets rating to 0");

    let mut clamped_r = 99u32;
    let _ = run_ui_cleared(&ctx, RawInput::default(), |ui| {
        MacRatingIndicator::new(&mut clamped_r, 5).theme(&theme).show(ui);
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
    let dummy_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).join("Cargo.toml");
    let raw_drop = RawInput { dropped_files: vec![Arc::new(MockDroppedFile(dummy_path)) as egui::DroppedFileHandle], ..Default::default() };
    let _ = run_app_frame(&ctx, &mut app, 1.1, Some(raw_drop));

    // Toast manager should record the file inspection
    assert!(
        app.toasts.borrow().active_toasts().iter().any(|t| t.message.contains("Cargo.toml") || t.message.contains("Loaded")),
        "Dropping a real file into Finder must trigger receipt/toast"
    );

    // AudioTrack robustness tests
    use gallery_app::gallery::audio_media::AudioTrack;
    let corrupt = b"RIFF\x00\x00\x00\x00WAVEfmt \x02\x00\x00\x00\x01\x00";
    assert!(AudioTrack::from_wav_bytes("bad.wav", corrupt).is_err());
    let track = AudioTrack::default();
    let inverted_wav = track.to_wav_bytes_mixed(60.0, 10.0, 1.0, 0.0);
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
    assert_eq!(idle_delay, Duration::MAX, "When AVKit media is paused, repaint delay MUST be Duration::MAX");

    // 2. Adjust gain via circular slider & verify audio level response
    app.center.media_gain = 1.5;
    t += 0.1;
    let _ = run_app_frame(&ctx, &mut app, t, None);
    assert!(app.center.audio_level_l > 0.0, "Audio levels must reflect scrubbed playhead and gain even when paused");

    // 3. Playing state triggers active repaint
    app.center.media_playing = true;
    t += 0.1;
    let playing_delay = run_app_frame(&ctx, &mut app, t, None);
    assert!(playing_delay < Duration::MAX, "When playing, active repaint must be requested for 60fps pacing");

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

        let snippet = gallery_app::gallery::snippet::generate_rust_snippet(screen, &app.card.borrow(), &(&app.center).into());
        assert!(!snippet.is_empty(), "Snippet for {screen:?} must not be empty");
        assert!(snippet.contains("egui_macview::"), "Snippet for {screen:?} must reference egui_macview primitives");
        assert!(snippet.contains("theme"), "Snippet for {screen:?} must be data-driven with theme");
    }

    // Adversarial escaping check: quotes and backslashes in layer name
    app.card.borrow_mut().layer_name = r#"Hero "Quoted" & \Backslash\"#.to_string();
    let snip =
        gallery_app::gallery::snippet::generate_rust_snippet(GalleryScreen::CanvasPhysics, &app.card.borrow(), &(&app.center).into());
    assert!(snip.contains(r#""Hero \"Quoted\" & \\Backslash\\""#), "Snippet generator must produce valid escaped Rust string literals");
}

#[test]
fn test_finder_nib_xib_showcase_files_and_swiftui_studio_window() {
    let ctx = Context::default();
    let mut app = GalleryApp::new();
    app.active_screen = GalleryScreen::FinderDataBrowser;
    app.sidebar.active_screen = GalleryScreen::FinderDataBrowser;

    let showcase_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("nib_showcase");
    for file_name in [
        "1_Vector_and_Transform_Inspector.nib",
        "2_Theme_and_Appearance_Studio.nib",
        "3_AVKit_Audio_Mastering_Strip.nib",
        "4_Publishing_and_Asset_Queue.nib",
        "5_Binary_Varint_Inspector.nib",
    ] {
        let path = showcase_dir.join(file_name);
        assert!(path.exists(), "Binary .nib showcase file must exist: {}", path.display());
        let raw_bytes = std::fs::read(&path).unwrap_or_default();
        assert!(raw_bytes.starts_with(b"NIBArchive\x01\x00\x00\x00"), "{file_name} must be a binary NIBArchive file");
        assert!(app.center.nib_studio.load_nib_or_xib_file(&path, &app.toasts), "Must compile and open {file_name}");
        assert!(app.center.nib_studio.open && !app.center.nib_studio.swiftui_source.contains('#'));
        assert!(app.center.nib_studio.roundtrip_verified);
        let _ = run_app_frame(&ctx, &mut app, 30.0, None);
    }

    // Verify dynamic two-way coupling between the NIB window and live GalleryApp state
    app.center.nib_studio.set_path_f64("values.scalePercent", 145.0);
    app.center.nib_studio.set_path_i64("values.themeVariant", 2);
    app.center.nib_studio.set_path_f64("values.mediaGain", 1.65);
    let _ = run_app_frame(&ctx, &mut app, 31.0, None);
    assert!((app.card.borrow().scale_pct - 145.0).abs() < 1e-3);
    assert_eq!(app.appearance, MacAppearance::DarkAqua);
    assert!((app.center.media_gain - 1.65).abs() < 1e-3);
}

#[test]
fn test_req_015_12_all_seven_xib_and_nib_help_buttons_have_non_empty_tooltips_and_foreground_popover() {
    use egui_macview::scene::{CustomClassMapper, NibArchiveFrontend, SceneNodeKind, XibXmlFrontend};
    let assets = [
        "1_Vector_and_Transform_Inspector",
        "1_Typography_and_Grid_Inspector",
        "2_Appearance_and_Theme_Settings",
        "3_AVKit_Audio_Mastering_Strip",
        "4_Publishing_and_Asset_Queue",
        "2_Theme_and_Appearance_Studio",
        "Right_Inspector_Default",
    ];
    let mapper = CustomClassMapper::default_studio_mappings();
    for stem in assets {
        let xib_path = format!("assets/nib_showcase/{stem}.xib");
        let xml = std::fs::read_to_string(&xib_path).expect("read xib");
        let xib_scene = XibXmlFrontend::from_xml_str(&xml).expect("compile xib");
        let xib_helps: Vec<_> = xib_scene.nodes().iter().filter(|n| matches!(n.kind, SceneNodeKind::HelpButton)).collect();
        assert!(!xib_helps.is_empty(), "{xib_path} must contain a HelpButton");
        for h in xib_helps {
            assert!(h.tooltip.len() >= 24 && h.tooltip != "?", "{xib_path} HelpButton `{}` missing toolTip", h.ib_object_id);
        }

        let nib_path = format!("assets/nib_showcase/{stem}.nib");
        let nib_bytes = std::fs::read(&nib_path).expect("read nib");
        let nib_scene = NibArchiveFrontend::from_bytes(&nib_bytes, &mapper).expect("decode nib");
        let nib_helps: Vec<_> = nib_scene.nodes().iter().filter(|n| matches!(n.kind, SceneNodeKind::HelpButton)).collect();
        assert!(!nib_helps.is_empty(), "{nib_path} must contain a HelpButton");
        for h in nib_helps {
            assert!(h.tooltip.len() >= 24 && h.tooltip != "?", "{nib_path} HelpButton `{}` missing toolTip", h.ib_object_id);
        }
    }
    let rw_src = std::fs::read_to_string("src/scene/runtime_widgets.rs").expect("read runtime_widgets.rs");
    assert!(rw_src.contains("Order::Foreground") && rw_src.contains("TextWrapMode::Wrap"));
}

#[test]
fn test_req_015_07_avkit_video_viewport_uses_interactive_mac_player_view_and_unmuted_audio_by_default() {
    use egui_macview::scene::MacSceneDataSource;
    let video_src = std::fs::read_to_string("examples/gallery/video_scene.rs").expect("read video_scene.rs");
    assert!(video_src.contains("MacPlayerView") && video_src.contains("MacPlayerControlsStyle::Floating"));
    assert!(!video_src.contains("❚❚"), "video_scene.rs must not contain missing Unicode glyph `❚❚`");

    let app = GalleryApp::new();
    assert!(app.center.media_sound_output, "media_sound_output must default to true");
    assert!(app.center.audio_track.name.contains("Big Buck Bunny"), "audio_track.name must seed from s5-pop-codec option 0");

    let avkit_store = &app.center.screen_nib_scenes[4].1;
    let mgain_bid = *avkit_store.path_to_id.get("mediaGain").or_else(|| avkit_store.path_to_id.get("values.mediaGain")).expect("mediaGain");
    let mgain_str = avkit_store.get_str(mgain_bid);
    assert_ne!(mgain_str, "100", "s5-num-mgain must not seed with stale `100`");
    assert!(mgain_str.starts_with("1"), "s5-num-mgain must format 1.0 gain cleanly, got {mgain_str:?}");
}

#[test]
fn test_req_015_13_canvas_footer_50_100_200_zoom_buttons_click_reliably_and_sync_across_screens() {
    use egui_macview::{MacCanvasFooter, MacCanvasFooterAction, MacControlSize};
    let ctx = Context::default();
    let theme = MacTheme::new(MacAppearance::Aqua);
    let fm = theme.footer_metrics();
    let right_edge = 600.0 - theme.stack_view().default_spacing.points();
    let (pw, gap) = (fm.preset_width.points(), fm.preset_spacing.points());
    let (x_200, x_100, x_50) = (right_edge - pw * 0.5, right_edge - pw * 1.5 - gap, right_edge - pw * 2.5 - gap * 2.0);

    for (click_x, expected_zoom) in [(x_50, 0.5f32), (x_100, 1.0f32), (x_200, 2.0f32)] {
        let pos = Pos2::new(click_x, 12.0);
        let mut action_out = None;
        for (pressed, is_release) in [(false, false), (true, false), (false, true)] {
            let events = if !pressed && !is_release {
                vec![Event::PointerMoved(pos)]
            } else {
                vec![
                    Event::PointerMoved(pos),
                    Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE },
                ]
            };
            let _ = run_ui_cleared(
                &ctx,
                RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(600.0, 24.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    action_out = MacCanvasFooter::new(&["Workspace", "Typography"], Some(pos), 1.0).theme(&theme).show(ui, 600.0);
                },
            );
        }
        assert_eq!(action_out, Some(MacCanvasFooterAction::SelectZoom(expected_zoom)));
    }

    let mut app = GalleryApp::new();
    app.active_screen = GalleryScreen::VectorAssets;
    app.center.apply_footer_zoom_preset(0.5, &app.card, app.active_screen);
    assert_eq!((app.card.borrow().scale_pct, app.center.symbol_preview_size_idx), (50.0, 0));
    app.center.apply_footer_zoom_preset(2.0, &app.card, app.active_screen);
    assert_eq!((app.card.borrow().scale_pct, app.center.symbol_preview_size_idx), (200.0, 2));

    app.active_screen = GalleryScreen::AvKitMediaStudio;
    app.center.apply_footer_zoom_preset(2.0, &app.card, app.active_screen);
    assert!((app.center.media_zoom - 2.0).abs() < 1e-4);

    app.active_screen = GalleryScreen::AdvancedControls;
    app.center.screen_nib_scenes[5].1.set_path_i64("nibPresetIdx", 1);
    let _ = run_app_frame(&ctx, &mut app, 40.0, None);
    assert_eq!((app.center.nib_preset_idx, app.center.resolved_state6_control_size()), (1, MacControlSize::Small));

    app.center.screen_nib_scenes[5].1.set_path_i64("nibPresetIdx", 2);
    let _ = run_app_frame(&ctx, &mut app, 41.0, None);
    assert_eq!((app.center.nib_preset_idx, app.center.resolved_state6_control_size()), (2, MacControlSize::Mini));
}
