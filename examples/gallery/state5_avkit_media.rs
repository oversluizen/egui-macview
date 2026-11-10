//! State 5 (`v0.5`): AVKit Media & Audio Timeline Studio (`GalleryScreen::AvKitMediaStudio`).

use super::{
    audio_media::NativeAudioOutput, state5_meters::render_rotary_studio_bar,
    state5_meters::render_stereo_meters, video_scene::render_procedural_video_viewport,
    CenterState,
};
use egui::{FontId, Ui};
use egui_macview::{
    theme::MacTheme, MacButtonCell, MacStepper, MacTimelineZone, MacToastManager, MacToggle,
    MacTrimScrubber,
};
use std::cell::RefCell;

impl CenterState {
    pub fn render_state5_avkit_media(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        toasts: &RefCell<MacToastManager>,
    ) {
        let label_font = FontId::proportional(12.0);
        let now = content_ui.input(|i| i.time);
        let window_active = egui_macview::is_window_active(content_ui.ctx());

        // Advance playhead when playing (60fps animation pacing, pauses when unfocused for 0.0% background CPU)
        if self.media_playing {
            if window_active {
                // If window just regained focus while playing, restart audio in sync
                if !self.media_was_window_active && self.media_sound_output {
                    let wav = self.audio_track.to_wav_bytes_mixed(
                        self.media_playhead,
                        self.media_out_point,
                        self.media_gain,
                        self.media_pan,
                    );
                    NativeAudioOutput::play_wav(&wav);
                }

                content_ui
                    .ctx()
                    .request_repaint_after(std::time::Duration::from_millis(16));
                let dt = self
                    .media_last_time
                    .map(|prev| (now - prev).max(0.0))
                    .unwrap_or(1.0 / 60.0);
                self.media_playhead += dt;
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
                // Window unfocused or minimized: stop audio to prevent desync while repaints are paused
                if self.media_was_window_active && self.media_sound_output {
                    NativeAudioOutput::stop();
                }
                self.media_last_time = None;
            }
        } else {
            self.media_last_time = Some(now);
        }
        self.media_was_window_active = window_active;

        // Real-time audio metering (memoized when paused to preserve Zero-Idle-Repaint)
        let (lvl_l, lvl_r, pk_l, pk_r) = if self.media_playing {
            self.audio_track.compute_stereo_levels_at(
                self.media_playhead,
                self.media_gain,
                self.media_pan,
            )
        } else {
            let key = (self.media_playhead, self.media_gain, self.media_pan);
            if (key.0 - self.audio_levels_key.0).abs() < 1e-4
                && (key.1 - self.audio_levels_key.1).abs() < 1e-4
                && (key.2 - self.audio_levels_key.2).abs() < 1e-4
            {
                self.audio_levels_cached
            } else {
                let computed = self.audio_track.compute_stereo_levels_at(
                    self.media_playhead,
                    self.media_gain,
                    self.media_pan,
                );
                self.audio_levels_cached = computed;
                self.audio_levels_key = key;
                computed
            }
        };

        if self.media_playing {
            self.audio_level_l = lvl_l;
            self.audio_level_r = lvl_r;
            let decay = 0.05;
            self.audio_peak_l = (self.audio_peak_l - decay).max(pk_l);
            self.audio_peak_r = (self.audio_peak_r - decay).max(pk_r);
        } else {
            self.audio_level_l = lvl_l;
            self.audio_level_r = lvl_r;
            self.audio_peak_l = pk_l;
            self.audio_peak_r = pk_r;
        }

        content_ui.label(
            egui::RichText::new(
                "State 5 (v0.5) — AVKit Media & Audio Timeline Studio (MacTrimScrubber, MacCircularSlider & MacLevelIndicator)",
            )
            .font(label_font.clone())
            .strong(),
        );
        content_ui.label(
            egui::RichText::new(
                "Real audio playback & waveform sync, 60fps procedural video canvas, rotary controls (Gain, Pan, Exposure, Contrast, Zoom), and star rating.",
            )
            .size(11.0),
        );
        content_ui.add_space(8.0);

        // 1. Procedural 60fps Video Scene Viewport
        render_procedural_video_viewport(content_ui, theme, self);
        content_ui.add_space(8.0);

        // 2. AVPlayerView MacTrimScrubber
        let zones = [
            MacTimelineZone::new(4.0, 12.0, "Scene 1: Dialogue"),
            MacTimelineZone::new(15.0, 24.0, "Scene 2: SFX Peak"),
        ];
        let scrubber_w = (content_ui.available_width() - 32.0).max(320.0);
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
        let mut force_restart = false;
        if trim_out.in_point_dragged || trim_out.out_point_dragged {
            self.media_playhead = self
                .media_playhead
                .clamp(self.media_in_point, self.media_out_point);
            if self.media_playing {
                audio_restart = true;
            }
        }
        if trim_out.playhead_dragged && self.media_playing {
            audio_restart = true;
        }

        // 3. Transport Controls Bar
        content_ui.add_space(8.0);
        content_ui.horizontal(|ui| {
            let play_label = if self.media_playing {
                "Pause Timeline"
            } else {
                "Play Timeline"
            };
            if ui
                .add(
                    MacButtonCell::new(play_label)
                        .theme(theme)
                        .selected(self.media_playing),
                )
                .clicked()
            {
                self.media_playing = !self.media_playing;
                if self.media_playing {
                    self.media_last_time = Some(now);
                    if self.media_playhead >= self.media_out_point {
                        self.media_playhead = self.media_in_point;
                    }
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
                        self.media_last_audio_restart = now;
                    }
                } else {
                    NativeAudioOutput::stop();
                }
            }

            if ui.add(MacButtonCell::new("Jump In").theme(theme)).clicked() {
                self.media_playhead = self.media_in_point;
                toasts.borrow_mut().info("Playhead snapped to In point");
                if self.media_playing {
                    force_restart = true;
                }
            }
            if ui
                .add(MacButtonCell::new("Jump Out").theme(theme))
                .clicked()
            {
                self.media_playhead = self.media_out_point;
                self.media_playing = false;
                NativeAudioOutput::stop();
                toasts.borrow_mut().info("Playhead snapped to Out point");
            }

            ui.add_space(4.0);
            ui.label(egui::RichText::new("Step:").size(11.0));
            let mut step_val = self.media_playhead;
            if ui
                .add(
                    MacStepper::new(
                        &mut step_val,
                        self.media_in_point..=self.media_out_point,
                        0.1,
                    )
                    .theme(theme),
                )
                .changed()
            {
                self.media_playhead = step_val;
                if self.media_playing {
                    force_restart = true;
                }
            }

            ui.add_space(4.0);
            ui.label(egui::RichText::new("Loop:").size(11.0));
            ui.add(MacToggle::new(&mut self.media_loop).theme(theme));

            ui.add_space(4.0);
            let sound_btn_text = if self.media_sound_output {
                "🔊 Audio Output: ON"
            } else {
                "🔇 Audio Output: OFF"
            };
            if ui
                .add(
                    MacButtonCell::new(sound_btn_text)
                        .theme(theme)
                        .selected(self.media_sound_output),
                )
                .clicked()
            {
                self.media_sound_output = !self.media_sound_output;
                if !self.media_sound_output {
                    NativeAudioOutput::stop();
                } else if self.media_playing {
                    force_restart = true;
                }
            }

            ui.add_space(4.0);
            let (can_undo, can_redo) = (self.avkit_undo.can_undo(), self.avkit_undo.can_redo());
            let undo_lbl = format!("⤺ Undo ({})", self.avkit_undo.undo_count());
            let redo_lbl = format!("⤻ Redo ({})", self.avkit_undo.redo_count());
            if ui
                .add(MacButtonCell::new(&undo_lbl).theme(theme).enabled(can_undo))
                .clicked()
                && self.undo_avkit()
            {
                toasts.borrow_mut().info("AVKit: Undone adjustment");
            }
            if ui
                .add(MacButtonCell::new(&redo_lbl).theme(theme).enabled(can_redo))
                .clicked()
                && self.redo_avkit()
            {
                toasts.borrow_mut().info("AVKit: Redone adjustment");
            }
        });

        // 4. Rotary Circular Sliders Studio Bar (Gain, Pan, Exposure, Contrast, Zoom)
        content_ui.add_space(12.0);
        content_ui.label(
            egui::RichText::new(
                "NSSlider (Circular Style) & NSLevelIndicator — Rotary Dial Studio & Live VU Meters",
            )
            .font(label_font)
            .strong(),
        );
        content_ui.add_space(6.0);

        if render_rotary_studio_bar(content_ui, theme, self) {
            audio_restart = true;
        }

        let should_restart =
            force_restart || (audio_restart && (now - self.media_last_audio_restart >= 0.08));
        if should_restart && self.media_playing && self.media_sound_output {
            let wav = self.audio_track.to_wav_bytes_mixed(
                self.media_playhead,
                self.media_out_point,
                self.media_gain,
                self.media_pan,
            );
            NativeAudioOutput::play_wav(&wav);
            self.media_last_audio_restart = now;
        }

        // 5. Stereo L/R MacLevelIndicator VU Meters
        render_stereo_meters(content_ui, theme, self, scrubber_w);

        self.checkpoint_avkit_if_changed();
        content_ui.add_space(16.0);
    }
}
