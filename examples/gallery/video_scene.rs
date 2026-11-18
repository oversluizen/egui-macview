//! Procedural 60fps Cinema & Broadcast Video Monitor with Interactive QuickTime Floating HUD (`SPEC-011`, `REQ-015-07`).

use super::{audio_media::NativeAudioOutput, CenterState};
use egui::{pos2, vec2, Align2, CornerRadius, FontId, Id, Pos2, Rect, Stroke, StrokeKind, Ui, UiBuilder};
use egui_macview::{controls::player_view::PlayerViewState, theme::MacTheme, MacPlayerControlsStyle, MacPlayerView};

/// Renders the 60fps multi-scene studio video viewport with an embedded interactive [`MacPlayerView`] (`Floating` style).
pub fn render_procedural_video_viewport(content_ui: &mut Ui, theme: &MacTheme, state: &mut CenterState) -> Rect {
    let avail_w = (content_ui.available_width() - 24.0).clamp(320.0, 640.0);
    let max_video_h = (content_ui.available_height() - 62.0).clamp(136.0, 212.0);
    let video_h = (avail_w * (9.0 / 16.0)).min(max_video_h);
    let start_pos = content_ui.cursor().min;
    let video_rect = Rect::from_min_size(start_pos, vec2(avail_w, video_h));

    let vp = content_ui.painter_at(video_rect);
    let exp_gain = (1.0 + state.media_exposure * 0.35).clamp(0.4, 1.8) * state.media_contrast.clamp(0.5, 2.0);
    let base_bg = theme.hud_background().gamma_multiply(exp_gain);
    let r_corner = CornerRadius::same(theme.box_view().corner_radius.points() as u8);
    let b_width = theme.box_view().border_width.points();
    vp.rect_filled(video_rect, r_corner, base_bg);

    paint_preset_scene(
        &vp,
        theme,
        video_rect,
        state.codec_preset,
        (state.media_playhead as f32, state.media_zoom.clamp(0.5, 2.5), state.media_pan.clamp(-1.0, 1.0)),
        &state.audio_track.waveform_peaks,
        state.media_gain,
    );

    // SMPTE Timecode & Active AudioTrack Name (100% data-driven from state.audio_track.name & CupertinoIcons)
    let fps = theme.player_view().frame_step_fps.points().max(1.0) as u64;
    let total_frames = (state.media_playhead * (fps as f64)).floor() as u64;
    let (frames, secs, mins) = (total_frames % fps, (total_frames / fps) % 60, (total_frames / (fps * 60)) % 60);
    let pad = theme.box_view().inset_left.points();
    let font_sz = theme.box_view().title_font_size.points();
    let status_word = if state.media_playing { "PLAYING" } else { "PAUSED" };
    let ic = pos2(video_rect.left() + pad + 4.0, video_rect.top() + pad + font_sz * 0.5);
    if state.media_playing {
        vp.add(egui::Shape::convex_polygon(
            vec![pos2(ic.x - 3.0, ic.y - 4.0), pos2(ic.x - 3.0, ic.y + 4.0), pos2(ic.x + 4.0, ic.y)],
            theme.hud_text(),
            Stroke::NONE,
        ));
    } else {
        vp.rect_filled(Rect::from_center_size(pos2(ic.x - 2.0, ic.y), vec2(2.0, 8.0)), CornerRadius::same(0), theme.hud_text());
        vp.rect_filled(Rect::from_center_size(pos2(ic.x + 2.0, ic.y), vec2(2.0, 8.0)), CornerRadius::same(0), theme.hud_text());
    }
    vp.text(
        pos2(video_rect.left() + pad + 13.0, video_rect.top() + pad),
        Align2::LEFT_TOP,
        format!("{status_word}  •  TC 00:{mins:02}:{secs:02}:{frames:02}"),
        FontId::monospace(font_sz),
        theme.hud_text(),
    );
    vp.text(
        pos2(video_rect.right() - pad, video_rect.top() + pad),
        Align2::RIGHT_TOP,
        format!("{}  •  {}★", state.audio_track.name, state.media_rating),
        FontId::proportional(font_sz),
        theme.hud_secondary_text(),
    );

    // Interactive QuickTime Floating HUD Pod (`MacPlayerView` 3-phase execution pipeline)
    let dur = (state.media_out_point - state.media_in_point).max(0.5) as f32;
    let mut is_playing = state.media_playing;
    let mut progress = (((state.media_playhead - state.media_in_point) as f32) / dur).clamp(0.0, 1.0);
    let buffered = (progress + 0.22).min(1.0);
    let mut volume = (state.media_gain * 0.5).clamp(0.0, 1.0);
    let mut muted = !state.media_sound_output;
    let mut playback_rate = state.media_speed;

    let pv_state_id = Id::NULL.with("state5_quicktime_floating_hud");
    let mut pv_state = content_ui.data_mut(|d| d.get_temp::<PlayerViewState>(pv_state_id).unwrap_or_default());
    if !state.media_playing || pv_state.last_active_time <= 0.0 {
        pv_state.last_active_time = content_ui.input(|i| i.time);
    }

    let mut player_ui = content_ui.new_child(UiBuilder::new().id_salt("state5_player_view_child").max_rect(video_rect));
    let (geom, mut pv_resp) = {
        let mut player = MacPlayerView::new()
            .controls_style(MacPlayerControlsStyle::Floating)
            .is_playing_mut(&mut is_playing)
            .progress_mut(&mut progress)
            .volume_mut(&mut volume)
            .muted_mut(&mut muted)
            .playback_rate_mut(&mut playback_rate)
            .buffered_progress(buffered)
            .duration_secs(dur)
            .theme(theme);
        let geom = player.compute_geometry(video_rect);
        let resp = player.handle_input(&mut player_ui, &geom, &mut pv_state);
        player.paint(&vp, &geom, &pv_state, theme);
        (geom, resp)
    };

    let space_toggled =
        !content_ui.ctx().egui_wants_keyboard_input() && content_ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Space));
    let clicked_backdrop = pv_resp.response.clicked()
        && content_ui.input(|i| i.pointer.interact_pos()).is_some_and(|p| video_rect.contains(p) && !geom.controls_rect.contains(p));
    if space_toggled || clicked_backdrop {
        is_playing = !is_playing;
        pv_resp.play_toggled = true;
    }

    content_ui.data_mut(|d| d.insert_temp(pv_state_id, pv_state));
    content_ui.advance_cursor_after_rect(video_rect);

    if pv_resp.play_toggled || pv_resp.mute_toggled || pv_resp.volume_changed || pv_resp.progress_changed || pv_resp.rate_changed {
        state.media_playing = is_playing;
        state.media_sound_output = !muted;
        if pv_resp.volume_changed {
            state.media_gain = (volume * 2.0).clamp(0.0, 2.0);
        }
        if pv_resp.rate_changed {
            state.media_speed = playback_rate;
        }
        if pv_resp.progress_changed {
            state.media_playhead =
                (state.media_in_point + (progress as f64) * (dur as f64)).clamp(state.media_in_point, state.media_out_point);
        }
        if state.media_playing && state.media_sound_output {
            let wav = state.audio_track.to_wav_bytes_mixed(state.media_playhead, state.media_out_point, state.media_gain, state.media_pan);
            NativeAudioOutput::play_wav(&wav);
            state.media_last_audio_restart = content_ui.input(|i| i.time);
        } else {
            NativeAudioOutput::stop();
        }
    }

    vp.rect_stroke(video_rect, r_corner, Stroke::new(b_width, theme.separator()), StrokeKind::Inside);
    video_rect
}

fn paint_preset_scene(
    vp: &egui::Painter,
    theme: &MacTheme,
    r: Rect,
    preset: usize,
    (t, zoom, pan): (f32, f32, f32),
    wave: &[f32],
    gain: f32,
) {
    let pan_x = pan * 36.0;
    let c = pos2(r.center().x + pan_x, r.center().y - 10.0);
    match preset {
        0 | 1 => {
            // Cinema Horizon, Parallax Ridge Silhouette & Animated Subject
            let horizon_y = r.center().y + 6.0;
            let sun_pos = pos2(r.left() + r.width() * (0.30 + 0.08 * (t * 0.4).sin()) + pan_x * 0.5, r.top() + 36.0);
            let sun_col = if preset == 0 { theme.system_yellow() } else { theme.system_orange() };
            vp.circle_filled(sun_pos, 22.0 * zoom, sun_col.gamma_multiply(0.28));
            vp.circle_filled(sun_pos, 11.0 * zoom, sun_col);
            let ridge_pts: Vec<Pos2> = (0..=28)
                .map(|i| {
                    let fx = i as f32 / 28.0;
                    let x = r.left() + fx * r.width();
                    let h = ((fx * 6.0 + t * 0.65 + pan).sin() * 14.0 + (fx * 13.0 - t * 0.35).cos() * 7.0) * zoom;
                    pos2(x, (horizon_y - h).clamp(r.top() + 18.0, r.bottom() - 42.0))
                })
                .collect();
            vp.add(egui::Shape::line(ridge_pts, Stroke::new(2.2, theme.system_green())));
            let orb_x = (r.left() + 44.0 + ((t * 0.35).fract()) * (r.width() - 88.0) + pan_x).clamp(r.left() + 24.0, r.right() - 24.0);
            let orb_y = (horizon_y - 12.0 - ((t * 4.5).sin().abs() * 20.0 * zoom)).clamp(r.top() + 22.0, r.bottom() - 56.0);
            vp.circle_filled(pos2(orb_x, orb_y), 9.0 * zoom, theme.control_accent());
            vp.circle_stroke(pos2(orb_x, orb_y), 12.5 * zoom, Stroke::new(1.2, theme.hud_text()));
        }
        2 | 3 => {
            // VFX Cyber Reticle, Perspective Grid & 60fps Telemetry Cadence
            let rad = (28.0 * zoom).min(r.height() * 0.28);
            let ang = t * 1.8;
            vp.circle_stroke(c, rad, Stroke::new(1.6, theme.control_accent()));
            vp.circle_stroke(c, rad * 0.62, Stroke::new(1.2, theme.system_orange()));
            vp.line_segment(
                [c + vec2(ang.cos() * rad, ang.sin() * rad), c - vec2(ang.cos() * rad, ang.sin() * rad)],
                Stroke::new(1.5, theme.control_accent()),
            );
            let ang2 = -t * 1.1;
            vp.line_segment(
                [c + vec2(ang2.cos() * rad * 0.8, ang2.sin() * rad * 0.8), c - vec2(ang2.cos() * rad * 0.8, ang2.sin() * rad * 0.8)],
                Stroke::new(1.0, theme.system_green()),
            );
        }
        _ => {
            // Mastering Studio Harmonic Lissajous Phase-Scope
            let rad = (26.0 * zoom).min(r.height() * 0.26);
            let lissajous: Vec<Pos2> = (0..=40)
                .map(|i| {
                    let p = (i as f32 / 40.0) * std::f32::consts::TAU;
                    let lx = (p * 3.0 + t * 2.2).sin() * rad * 1.3;
                    let ly = (p * 2.0 + t * 1.5).cos() * rad * 0.8;
                    pos2(c.x + lx, c.y + ly)
                })
                .collect();
            vp.add(egui::Shape::line(lissajous, Stroke::new(1.6, theme.control_accent())));
        }
    }
    // 32-Band Live Audio Spectrum above the Floating HUD pod
    if !wave.is_empty() {
        let bars = 32usize.min(wave.len());
        let span_w = r.width() - 48.0;
        let bar_w = (span_w / bars as f32).max(3.0);
        let base_y = r.bottom() - 78.0;
        let phase_idx = ((t * 12.0).floor() as usize) % wave.len().max(1);
        for b in 0..bars {
            let w_val = wave[(b + phase_idx) % wave.len()];
            let bh = (w_val * 28.0 * gain.clamp(0.2, 2.0) * zoom).clamp(2.0, 38.0);
            let bx = r.left() + 24.0 + b as f32 * (span_w / bars as f32);
            let br = Rect::from_min_max(pos2(bx, base_y - bh), pos2(bx + bar_w - 2.0, base_y));
            let col = if bh > 28.0 {
                theme.system_orange()
            } else if preset >= 4 {
                theme.control_accent()
            } else {
                theme.system_green().gamma_multiply(0.75)
            };
            vp.rect_filled(br, CornerRadius::same(1), col);
        }
    }
}
