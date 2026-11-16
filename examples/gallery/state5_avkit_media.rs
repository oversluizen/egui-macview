//! State 5 (`v0.5`): AVKit Media & Audio Timeline Studio (`GalleryScreen::AvKitMediaStudio`).

use super::{audio_media::NativeAudioOutput, types::AvKitStudioSnapshot, video_scene::render_procedural_video_viewport, CenterState};
use egui::{Align, Layout, Rect, Ui, UiBuilder};
use egui_macview::{theme::MacTheme, MacTimelineZone, MacTrimScrubber};

impl CenterState {
    pub fn render_avkit_video_scrubber_slot(&mut self, parent_ui: &mut Ui, theme: &MacTheme, slot_rect: Rect) {
        let mut content_ui = parent_ui
            .new_child(UiBuilder::new().id_salt("state5_video_scrubber_slot").max_rect(slot_rect).layout(Layout::top_down(Align::Min)));
        let content_ui = &mut content_ui;
        let now = content_ui.input(|i| i.time);
        let window_active = egui_macview::is_window_active(content_ui.ctx());

        if self.media_playing {
            if window_active {
                if !self.media_was_window_active && self.media_sound_output {
                    let wav =
                        self.audio_track.to_wav_bytes_mixed(self.media_playhead, self.media_out_point, self.media_gain, self.media_pan);
                    NativeAudioOutput::play_wav(&wav);
                }

                content_ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
                let dt = self.media_last_time.map(|prev| (now - prev).max(0.0)).unwrap_or(1.0 / 60.0);
                self.media_playhead += dt * (self.media_speed.clamp(0.25, 2.0) as f64);
                if self.media_playhead >= self.media_out_point {
                    if self.media_loop {
                        let loop_len = (self.media_out_point - self.media_in_point).max(0.1);
                        while self.media_playhead >= self.media_out_point {
                            self.media_playhead -= loop_len;
                        }
                        self.media_playhead = self.media_playhead.max(self.media_in_point);
                        self.audio_peak_l = 0.0;
                        self.audio_peak_r = 0.0;
                        if self.media_sound_output {
                            let wav = self.audio_track.to_wav_bytes_mixed(
                                self.media_playhead,
                                self.media_out_point,
                                self.media_gain,
                                self.media_pan,
                            );
                            NativeAudioOutput::play_wav(&wav);
                        }
                    } else {
                        self.media_playhead = self.media_out_point;
                        self.media_playing = false;
                        NativeAudioOutput::stop();
                    }
                }
                self.media_last_time = Some(now);
            } else {
                if self.media_was_window_active && self.media_sound_output {
                    NativeAudioOutput::stop();
                }
                self.media_last_time = None;
            }
        } else {
            self.media_last_time = Some(now);
        }
        self.media_was_window_active = window_active;

        let (lvl_l, lvl_r, pk_l, pk_r) = if self.media_playing {
            self.audio_track.compute_stereo_levels_at(self.media_playhead, self.media_gain, self.media_pan)
        } else {
            let key = (self.media_playhead, self.media_gain, self.media_pan);
            if (key.0 - self.audio_levels_key.0).abs() < 1e-4
                && (key.1 - self.audio_levels_key.1).abs() < 1e-4
                && (key.2 - self.audio_levels_key.2).abs() < 1e-4
            {
                self.audio_levels_cached
            } else {
                let computed = self.audio_track.compute_stereo_levels_at(self.media_playhead, self.media_gain, self.media_pan);
                self.audio_levels_cached = computed;
                self.audio_levels_key = key;
                computed
            }
        };

        let drive_boost = 1.0 + self.sat_drive.clamp(0.0, 1.0) * 0.45;
        let max_ceil = if self.brickwall_limiter { 8.4 } else { 10.0 };
        let (mut l_out, mut r_out) = ((lvl_l * drive_boost).clamp(0.0, max_ceil), (lvl_r * drive_boost).clamp(0.0, max_ceil));
        let (mut pk_l_out, mut pk_r_out) = ((pk_l * drive_boost).clamp(0.0, max_ceil), (pk_r * drive_boost).clamp(0.0, max_ceil));
        if self.bus_mode == 1 {
            let mid = (l_out + r_out) * 0.5;
            let pk_m = (pk_l_out + pk_r_out) * 0.5;
            (l_out, r_out, pk_l_out, pk_r_out) = (mid, mid, pk_m, pk_m);
        } else if self.bus_mode == 2 {
            let (mid, side) = ((l_out + r_out) * 0.5, (l_out - r_out).abs());
            (l_out, r_out) = (mid, side.clamp(0.0, max_ceil));
        }

        if self.media_playing {
            self.audio_level_l = l_out;
            self.audio_level_r = r_out;
            let decay = 0.05;
            self.audio_peak_l = (self.audio_peak_l - decay).max(pk_l_out).min(max_ceil);
            self.audio_peak_r = (self.audio_peak_r - decay).max(pk_r_out).min(max_ceil);
        } else {
            self.audio_level_l = l_out;
            self.audio_level_r = r_out;
            self.audio_peak_l = pk_l_out;
            self.audio_peak_r = pk_r_out;
        }

        let v_rect = render_procedural_video_viewport(content_ui, theme, self);
        self.last_video_viewport_rect = v_rect.intersect(content_ui.clip_rect());
        content_ui.add_space(6.0);

        let zones = [MacTimelineZone::new(4.0, 12.0, "Scene 1: Dialogue"), MacTimelineZone::new(15.0, 24.0, "Scene 2: SFX Peak")];
        let scrubber_w = slot_rect.width().max(320.0);
        let trim_out = MacTrimScrubber::new(
            "studio_avkit_trim_scrubber",
            self.audio_track.duration_secs.max(24.0),
            &mut self.media_in_point,
            &mut self.media_out_point,
            &mut self.media_playhead,
        )
        .theme(theme)
        .waveform(&self.audio_track.waveform_peaks)
        .zones(&zones)
        .width(scrubber_w)
        .show(content_ui);

        let mut audio_restart = false;
        if trim_out.in_point_dragged || trim_out.out_point_dragged {
            self.media_playhead = self.media_playhead.clamp(self.media_in_point, self.media_out_point);
            if self.media_playing {
                audio_restart = true;
            }
        }
        if trim_out.playhead_dragged && self.media_playing {
            audio_restart = true;
        }
        if audio_restart && (now - self.media_last_audio_restart >= 0.08) && self.media_playing && self.media_sound_output {
            let wav = self.audio_track.to_wav_bytes_mixed(self.media_playhead, self.media_out_point, self.media_gain, self.media_pan);
            NativeAudioOutput::play_wav(&wav);
            self.media_last_audio_restart = now;
        }

        self.checkpoint_avkit_if_changed();
    }

    pub fn current_avkit_snapshot(&self) -> AvKitStudioSnapshot {
        AvKitStudioSnapshot {
            in_point: self.media_in_point,
            out_point: self.media_out_point,
            playhead: self.media_playhead,
            gain: self.media_gain,
            pan: self.media_pan,
            exposure: self.media_exposure,
            contrast: self.media_contrast,
            zoom: self.media_zoom,
            rating: self.media_rating,
        }
    }

    pub fn apply_avkit_snapshot(&mut self, snap: &AvKitStudioSnapshot) {
        self.media_in_point = snap.in_point;
        self.media_out_point = snap.out_point;
        self.media_playhead = snap.playhead.clamp(snap.in_point, snap.out_point);
        self.media_gain = snap.gain;
        self.media_pan = snap.pan;
        self.media_exposure = snap.exposure;
        self.media_contrast = snap.contrast;
        self.media_zoom = snap.zoom;
        self.media_rating = snap.rating;
    }

    pub fn undo_avkit(&mut self) -> bool {
        let mut cur = self.current_avkit_snapshot();
        if self.avkit_undo.undo(&mut cur) {
            self.apply_avkit_snapshot(&cur);
            self.last_stable_avkit = Some(cur);
            true
        } else {
            false
        }
    }

    pub fn redo_avkit(&mut self) -> bool {
        let mut cur = self.current_avkit_snapshot();
        if self.avkit_undo.redo(&mut cur) {
            self.apply_avkit_snapshot(&cur);
            self.last_stable_avkit = Some(cur);
            true
        } else {
            false
        }
    }

    pub fn checkpoint_avkit_if_changed(&mut self) {
        let cur = self.current_avkit_snapshot();
        if let Some(ref prev) = self.last_stable_avkit {
            let changed = if self.media_playing {
                prev.params_differ(&cur)
            } else {
                prev.params_differ(&cur) || (prev.playhead - cur.playhead).abs() > 0.05
            };
            if changed {
                self.avkit_undo.push_snapshot(*prev);
                self.last_stable_avkit = Some(cur);
            }
        } else {
            self.last_stable_avkit = Some(cur);
        }
    }
}
