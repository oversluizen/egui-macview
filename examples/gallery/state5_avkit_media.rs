//! State 5 (`v0.5`): AVKit Media & Audio Timeline Studio (`GalleryScreen::AvKitMediaStudio`).

use super::CenterState;
use egui::{FontId, Ui};
use egui_macview::{
    theme::MacTheme, MacButtonCell, MacLevelIndicator, MacLevelIndicatorStyle,
    MacLevelOrientation, MacSegmentedControl, MacSlider, MacTimelineZone, MacToastManager,
    MacToggle, MacTrimScrubber,
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

        // Advance playhead when playing (frame-paced at ~30fps to preserve battery)
        if self.media_playing {
            content_ui
                .ctx()
                .request_repaint_after(std::time::Duration::from_millis(33));
            let dt = self
                .media_last_time
                .map(|prev| (now - prev).clamp(0.0, 0.1))
                .unwrap_or(1.0 / 30.0);
            self.media_playhead += dt * 14.0;
            if self.media_playhead >= self.media_out_point {
                if self.media_loop {
                    self.media_playhead = self.media_in_point;
                } else {
                    self.media_playhead = self.media_out_point;
                    self.media_playing = false;
                }
            }
        }
        self.media_last_time = Some(now);

        // Live-couple stereo L/R VU levels to waveform amplitude at current playhead
        let norm_pos = (self.media_playhead / 120.0).clamp(0.0, 1.0);
        let wave_idx =
            ((norm_pos * (self.media_waveform.len().saturating_sub(1)) as f64).round() as usize)
                .min(self.media_waveform.len().saturating_sub(1));
        let wave_amp = self.media_waveform.get(wave_idx).copied().unwrap_or(0.5);
        let jitter = if self.media_playing {
            ((now * 19.0).sin() as f32) * 0.45
        } else {
            0.0
        };
        self.audio_level_l = ((wave_amp * 9.2 + jitter) * self.media_gain).clamp(0.4, 10.0);
        self.audio_level_r =
            (((wave_amp * 8.7 - jitter * 0.8) + 0.35) * self.media_gain).clamp(0.4, 10.0);
        if self.media_playing {
            self.audio_peak_l = (self.audio_peak_l - 0.03)
                .max(self.audio_level_l)
                .clamp(0.4, 10.0);
            self.audio_peak_r = (self.audio_peak_r - 0.03)
                .max(self.audio_level_r)
                .clamp(0.4, 10.0);
        } else {
            self.audio_peak_l = self.audio_peak_l.max(self.audio_level_l).clamp(0.4, 10.0);
            self.audio_peak_r = self.audio_peak_r.max(self.audio_level_r).clamp(0.4, 10.0);
        }

        content_ui.label(
            egui::RichText::new(
                "State 5 (v0.5) — AVKit Media & Audio Timeline Studio (MacTrimScrubber & MacLevelIndicator)",
            )
            .font(label_font.clone())
            .strong(),
        );
        content_ui.label(
            egui::RichText::new(
                "Drag the yellow In/Out trim handles or scrub the playhead; stereo L/R VU meters respond live to the waveform.",
            )
            .size(11.0),
        );
        content_ui.add_space(8.0);

        // 1. AVPlayerView MacTrimScrubber
        let zones = [
            MacTimelineZone::new(18.0, 46.0, "Scene 1: Dialogue"),
            MacTimelineZone::new(62.0, 88.0, "Scene 2: SFX Peak"),
        ];
        let scrubber_w = (content_ui.available_width() - 32.0).max(320.0);
        let trim_out = MacTrimScrubber::new(
            "studio_avkit_trim_scrubber",
            120.0,
            &mut self.media_in_point,
            &mut self.media_out_point,
            &mut self.media_playhead,
        )
        .theme(theme)
        .waveform(&self.media_waveform)
        .zones(&zones)
        .width(scrubber_w)
        .show(content_ui);

        if trim_out.in_point_dragged || trim_out.out_point_dragged {
            self.media_playhead = self
                .media_playhead
                .clamp(self.media_in_point, self.media_out_point);
        }

        // 2. Transport Controls Bar
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
                if self.media_playing && self.media_playhead >= self.media_out_point {
                    self.media_playhead = self.media_in_point;
                }
            }

            if ui.add(MacButtonCell::new("Jump to In").theme(theme)).clicked() {
                self.media_playhead = self.media_in_point;
                toasts.borrow_mut().info("Playhead snapped to In point");
            }
            if ui.add(MacButtonCell::new("Jump to Out").theme(theme)).clicked() {
                self.media_playhead = self.media_out_point;
                toasts.borrow_mut().info("Playhead snapped to Out point");
            }

            ui.add_space(6.0);
            ui.label(egui::RichText::new("Loop Trim:").size(11.0));
            ui.add(MacToggle::new(&mut self.media_loop).theme(theme));

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!(
                    "In: {:05.1}s | Playhead: {:05.1}s | Out: {:05.1}s (Dur: {:.1}s)",
                    self.media_in_point,
                    self.media_playhead,
                    self.media_out_point,
                    (self.media_out_point - self.media_in_point).max(0.0)
                ))
                .size(10.5)
                .monospace()
                .color(theme.secondary_label()),
            );
        });

        // 3. Stereo L/R MacLevelIndicator VU Meters (Discrete & Continuous)
        content_ui.add_space(16.0);
        content_ui.label(
            egui::RichText::new(
                "NSLevelIndicator — Live Stereo L/R Audio Meters (Green / Warning / Critical Thresholds)",
            )
            .font(label_font)
            .strong(),
        );
        content_ui.add_space(6.0);

        content_ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Meter Mode:").size(11.0).strong());
            let style_idx = usize::from(!self.media_discrete_vu);
            MacSegmentedControl::new(&["Discrete LED", "Continuous Bar"], style_idx)
                .theme(theme)
                .height(22.0)
                .show(ui, |idx| {
                    self.media_discrete_vu = idx == 0;
                });
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Threshold VU Colors:").size(11.0));
            ui.add(MacToggle::new(&mut self.media_threshold_vu).theme(theme));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Master Gain:").size(11.0));
            MacSlider::new(&mut self.media_gain, 0.2..=1.5)
                .theme(theme)
                .width(110.0)
                .show(ui);
        });
        content_ui.add_space(8.0);

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
        content_ui.add_space(6.0);

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
        content_ui.add_space(18.0);
    }
}
