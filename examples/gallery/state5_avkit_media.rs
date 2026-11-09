//! State 5 (`v0.5`): AVKit Media & Audio Timeline Studio (`GalleryScreen::AvKitMediaStudio`).

use super::{
    audio_media::NativeAudioOutput, video_scene::render_procedural_video_viewport, CenterState,
};
use egui::{FontId, Ui};
use egui_macview::{
    theme::MacTheme, MacButtonCell, MacCircularSlider, MacLevelIndicator, MacLevelIndicatorStyle,
    MacLevelOrientation, MacRatingIndicator, MacSegmentedControl, MacStepper, MacTimelineZone,
    MacToastManager, MacToggle, MacTrimScrubber,
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

        // Advance playhead when playing (60fps animation pacing, paused when unfocused/minimized)
        let window_active =
            content_ui.input(|i| i.focused && !i.viewport().minimized.unwrap_or(false));
        if self.media_playing && window_active {
            content_ui
                .ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
            let dt = self
                .media_last_time
                .map(|prev| (now - prev).clamp(0.0, 0.05))
                .unwrap_or(1.0 / 60.0);
            self.media_playhead += dt * 4.0;
            if self.media_playhead >= self.media_out_point {
                if self.media_loop {
                    self.media_playhead = self.media_in_point;
                } else {
                    self.media_playhead = self.media_out_point;
                    self.media_playing = false;
                    NativeAudioOutput::stop();
                }
            }
        }
        self.media_last_time = Some(now);

        // Real-time audio metering calculated from track PCM samples
        let (lvl_l, lvl_r, pk_l, pk_r) = self.audio_track.compute_stereo_levels_at(
            self.media_playhead,
            self.media_gain,
            self.media_pan,
        );
        if self.media_playing {
            self.audio_level_l = lvl_l;
            self.audio_level_r = lvl_r;
            self.audio_peak_l = self.audio_peak_l.max(pk_l);
            self.audio_peak_r = self.audio_peak_r.max(pk_r);
        } else {
            // When paused, show static levels at current scrubbed playhead position (preserving Zero-Idle-Repaint)
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
            MacTimelineZone::new(18.0, 46.0, "Scene 1: Dialogue"),
            MacTimelineZone::new(62.0, 88.0, "Scene 2: SFX Peak"),
        ];
        let scrubber_w = (content_ui.available_width() - 32.0).max(320.0);
        let trim_out = MacTrimScrubber::new(
            "studio_avkit_trim_scrubber",
            self.audio_track.duration_secs.max(60.0),
            &mut self.media_in_point,
            &mut self.media_out_point,
            &mut self.media_playhead,
        )
        .theme(theme)
        .waveform(&self.audio_track.waveform_peaks)
        .zones(&zones)
        .width(scrubber_w)
        .show(content_ui);

        if trim_out.in_point_dragged || trim_out.out_point_dragged {
            self.media_playhead = self
                .media_playhead
                .clamp(self.media_in_point, self.media_out_point);
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
                    if self.media_playhead >= self.media_out_point {
                        self.media_playhead = self.media_in_point;
                    }
                    if self.media_sound_output {
                        let wav = self
                            .audio_track
                            .to_wav_bytes(self.media_in_point, self.media_out_point);
                        NativeAudioOutput::play_wav(&wav);
                    }
                } else {
                    NativeAudioOutput::stop();
                }
            }

            if ui.add(MacButtonCell::new("Jump In").theme(theme)).clicked() {
                self.media_playhead = self.media_in_point;
                toasts.borrow_mut().info("Playhead snapped to In point");
            }
            if ui
                .add(MacButtonCell::new("Jump Out").theme(theme))
                .clicked()
            {
                self.media_playhead = self.media_out_point;
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
                    let wav = self
                        .audio_track
                        .to_wav_bytes(self.media_in_point, self.media_out_point);
                    NativeAudioOutput::play_wav(&wav);
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

        content_ui.horizontal(|ui| {
            // Rotary Dial 1: Master Gain
            ui.vertical(|v_ui| {
                v_ui.label(egui::RichText::new("Gain").size(10.5).strong());
                MacCircularSlider::new(&mut self.media_gain, 0.0..=2.0)
                    .theme(theme)
                    .diameter(34.0)
                    .step(0.05)
                    .show(v_ui);
                v_ui.label(egui::RichText::new(format!("{:.2}x", self.media_gain)).size(10.0));
            });
            ui.add_space(10.0);

            // Rotary Dial 2: Stereo Pan
            ui.vertical(|v_ui| {
                v_ui.label(egui::RichText::new("Pan").size(10.5).strong());
                MacCircularSlider::new(&mut self.media_pan, -1.0..=1.0)
                    .theme(theme)
                    .diameter(34.0)
                    .step(0.05)
                    .show(v_ui);
                let pan_txt = if self.media_pan.abs() < 0.05 {
                    "C".to_string()
                } else if self.media_pan < 0.0 {
                    format!("L {:.0}%", -self.media_pan * 100.0)
                } else {
                    format!("R {:.0}%", self.media_pan * 100.0)
                };
                v_ui.label(egui::RichText::new(pan_txt).size(10.0));
            });
            ui.add_space(10.0);

            // Rotary Dial 3: Video Exposure
            ui.vertical(|v_ui| {
                v_ui.label(egui::RichText::new("Exposure").size(10.5).strong());
                MacCircularSlider::new(&mut self.media_exposure, -2.0..=2.0)
                    .theme(theme)
                    .diameter(34.0)
                    .step(0.1)
                    .show(v_ui);
                v_ui.label(
                    egui::RichText::new(format!("{:+.1} EV", self.media_exposure)).size(10.0),
                );
            });
            ui.add_space(10.0);

            // Rotary Dial 4: Video Contrast
            ui.vertical(|v_ui| {
                v_ui.label(egui::RichText::new("Contrast").size(10.5).strong());
                MacCircularSlider::new(&mut self.media_contrast, 0.5..=2.0)
                    .theme(theme)
                    .diameter(34.0)
                    .step(0.05)
                    .show(v_ui);
                v_ui.label(egui::RichText::new(format!("{:.2}", self.media_contrast)).size(10.0));
            });
            ui.add_space(10.0);

            // Rotary Dial 5: Viewport Zoom
            ui.vertical(|v_ui| {
                v_ui.label(egui::RichText::new("Zoom").size(10.5).strong());
                MacCircularSlider::new(&mut self.media_zoom, 0.5..=2.0)
                    .theme(theme)
                    .diameter(34.0)
                    .step(0.05)
                    .show(v_ui);
                v_ui.label(
                    egui::RichText::new(format!("{:.0}%", self.media_zoom * 100.0)).size(10.0),
                );
            });
            ui.add_space(12.0);

            // Rating Stars
            ui.vertical(|v_ui| {
                v_ui.label(egui::RichText::new("Take Rating").size(10.5).strong());
                MacRatingIndicator::new(&mut self.media_rating, 5)
                    .theme(theme)
                    .item_size(15.0)
                    .show(v_ui);
                v_ui.label(
                    egui::RichText::new(format!("{} of 5 stars", self.media_rating)).size(10.0),
                );
            });
        });

        // 5. Stereo L/R MacLevelIndicator VU Meters
        content_ui.add_space(12.0);
        content_ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Meter Mode:").size(11.0).strong());
            let style_idx = usize::from(!self.media_discrete_vu);
            MacSegmentedControl::new(&["Discrete LED", "Continuous Bar"], style_idx)
                .theme(theme)
                .height(22.0)
                .show(ui, |idx| {
                    self.media_discrete_vu = idx == 0;
                });
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Threshold Colors:").size(11.0));
            ui.add(MacToggle::new(&mut self.media_threshold_vu).theme(theme));
        });
        content_ui.add_space(6.0);

        let vu_style = if self.media_discrete_vu {
            MacLevelIndicatorStyle::Discrete
        } else {
            MacLevelIndicatorStyle::Continuous
        };
        let meter_len = (scrubber_w - 110.0).clamp(180.0, 420.0);

        content_ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Ch 1 (Left): ").size(11.0).monospace());
            MacLevelIndicator::new(self.audio_level_l, 0.0..=10.0)
                .theme(theme)
                .style(vu_style)
                .orientation(MacLevelOrientation::Horizontal)
                .warning_value(6.5)
                .critical_value(8.5)
                .peak_hold(self.audio_peak_l)
                .segments(16)
                .segmented_threshold_colors(self.media_threshold_vu)
                .tick_marks(6)
                .length(meter_len)
                .show(ui);
            ui.label(
                egui::RichText::new(format!("{:4.1} dB", self.audio_level_l - 10.0))
                    .size(10.5)
                    .monospace(),
            );
        });
        content_ui.add_space(4.0);

        content_ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Ch 2 (Right):").size(11.0).monospace());
            MacLevelIndicator::new(self.audio_level_r, 0.0..=10.0)
                .theme(theme)
                .style(vu_style)
                .orientation(MacLevelOrientation::Horizontal)
                .warning_value(6.5)
                .critical_value(8.5)
                .peak_hold(self.audio_peak_r)
                .segments(16)
                .segmented_threshold_colors(self.media_threshold_vu)
                .tick_marks(6)
                .length(meter_len)
                .show(ui);
            ui.label(
                egui::RichText::new(format!("{:4.1} dB", self.audio_level_r - 10.0))
                    .size(10.5)
                    .monospace(),
            );
        });

        self.checkpoint_avkit_if_changed();
        content_ui.add_space(16.0);
    }
}
