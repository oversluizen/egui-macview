//! Runtime Hygiene, Frame Pacing, and State Invariant Integration Tests.

#[allow(dead_code)]
#[path = "../examples/macview_gallery.rs"]
mod gallery_app;

use egui::{Context, Event, Pos2, RawInput};
use egui_macview::theme::{MacAppearance, MacTheme};
use gallery_app::{GalleryApp, GalleryScreen};

#[test]
fn test_full_gallery_cpu_hygiene_all_states_and_frame_pacing() {
    let ctx = Context::default();
    let mut app = GalleryApp::new();

    let run_gallery_frame =
        |app: &mut GalleryApp, time: f64, hover_pos: Option<Pos2>| -> std::time::Duration {
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
            std::time::Duration::MAX,
            "GalleryScreen::{screen:?} at rest must settle to Duration::MAX"
        );
    }

    // 3. Verify stationary pointer hovering over an active overflowing scrollbar track/thumb
    //    does NOT cause a 0ms busy-loop, and mid-fade reactivation resets the fade timeout
    //    before settling to Duration::MAX.
    let scroll_ctx = Context::default();
    theme.apply_to_egui(&scroll_ctx);
    let run_scroll_frame =
        |time: f64, hover_pos: Option<Pos2>| -> (std::time::Duration, egui::Rect) {
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
    assert_eq!(settled_initial, std::time::Duration::MAX);

    let sb_x = scroll_rect.right() - 4.0;
    let sb_y = scroll_rect.center().y;
    let _ = run_scroll_frame(3.0, Some(egui::pos2(sb_x, sb_y)));
    let _ = run_scroll_frame(3.016, None);
    let (hover_active_delay, _) = run_scroll_frame(3.032, None);
    assert!(
        hover_active_delay >= std::time::Duration::from_millis(700)
            && hover_active_delay <= std::time::Duration::from_millis(850),
        "got hover_active_delay={hover_active_delay:?}, scroll_rect={scroll_rect:?}"
    );

    assert_eq!(
        run_scroll_frame(3.85, None).0,
        std::time::Duration::from_millis(16)
    );
    let _ = run_scroll_frame(3.90, Some(egui::pos2(sb_x, sb_y + 8.0)));
    let _ = run_scroll_frame(3.916, None);
    assert!(run_scroll_frame(3.932, None).0 >= std::time::Duration::from_millis(700));
    assert_eq!(run_scroll_frame(5.2, None).0, std::time::Duration::MAX);

    // 4. Verify active timeline playback on State 5 uses frame-paced delay (>= 16ms), never Duration::ZERO,
    //    and pauses automatically (Duration::MAX) when the window loses focus.
    app.active_screen = GalleryScreen::AvKitMediaStudio;
    app.center.media_playing = true;
    t += 0.050;
    let play_delay = run_gallery_frame(&mut app, t, None);
    assert!(!play_delay.is_zero() && play_delay <= std::time::Duration::from_millis(50));

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
        std::time::Duration::MAX,
        "Unfocused window must pause timeline repaints (0.0% background CPU)"
    );
    app.center.media_playing = false;
}
