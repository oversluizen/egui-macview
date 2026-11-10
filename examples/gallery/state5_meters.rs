//! Auxiliary rotary controls and stereo level meters for State 5 AVKit Studio.

use super::CenterState;
use egui::Ui;
use egui_macview::{
    theme::MacTheme, MacCircularSlider, MacLevelIndicator, MacLevelIndicatorStyle,
    MacLevelOrientation, MacRatingIndicator, MacSegmentedControl, MacToggle,
};

/// Renders the 5 rotary dials and rating indicator in the AVKit Studio bar.
pub fn render_rotary_studio_bar(ui: &mut Ui, theme: &MacTheme, state: &mut CenterState) -> bool {
    let mut audio_restart = false;
    ui.horizontal(|h_ui| {
        // Rotary Dial 1: Master Gain
        let gain_txt = format!("{:.2}x", state.media_gain);
        if render_rotary(
            h_ui,
            theme,
            "Gain",
            &mut state.media_gain,
            0.0..=2.0,
            0.05,
            gain_txt,
        ) && state.media_playing
        {
            audio_restart = true;
        }
        h_ui.add_space(10.0);

        // Rotary Dial 2: Stereo Pan
        let pan_txt = if state.media_pan.abs() < 0.05 {
            "C".to_string()
        } else if state.media_pan < 0.0 {
            format!("L {:.0}%", -state.media_pan * 100.0)
        } else {
            format!("R {:.0}%", state.media_pan * 100.0)
        };
        if render_rotary(
            h_ui,
            theme,
            "Pan",
            &mut state.media_pan,
            -1.0..=1.0,
            0.05,
            pan_txt,
        ) && state.media_playing
        {
            audio_restart = true;
        }
        h_ui.add_space(10.0);

        // Rotary Dial 3: Video Exposure
        let exp_txt = format!("{:+.1} EV", state.media_exposure);
        render_rotary(
            h_ui,
            theme,
            "Exposure",
            &mut state.media_exposure,
            -2.0..=2.0,
            0.1,
            exp_txt,
        );
        h_ui.add_space(10.0);

        // Rotary Dial 4: Video Contrast
        let con_txt = format!("{:.2}", state.media_contrast);
        render_rotary(
            h_ui,
            theme,
            "Contrast",
            &mut state.media_contrast,
            0.5..=2.0,
            0.05,
            con_txt,
        );
        h_ui.add_space(10.0);

        // Rotary Dial 5: Viewport Zoom
        let zoom_txt = format!("{:.0}%", state.media_zoom * 100.0);
        render_rotary(
            h_ui,
            theme,
            "Zoom",
            &mut state.media_zoom,
            0.5..=2.0,
            0.05,
            zoom_txt,
        );
        h_ui.add_space(12.0);

        // Rating Stars
        h_ui.vertical(|v_ui| {
            v_ui.label(egui::RichText::new("Take Rating").size(10.5).strong());
            MacRatingIndicator::new(&mut state.media_rating, 5)
                .theme(theme)
                .item_size(15.0)
                .show(v_ui);
            v_ui.label(
                egui::RichText::new(format!("{} of 5 stars", state.media_rating)).size(10.0),
            );
        });
    });
    audio_restart
}

fn render_rotary(
    ui: &mut Ui,
    theme: &MacTheme,
    title: &str,
    val: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    step: f32,
    fmt_val: String,
) -> bool {
    let mut changed = false;
    ui.vertical(|v_ui| {
        v_ui.label(egui::RichText::new(title).size(10.5).strong());
        let resp = MacCircularSlider::new(val, range)
            .theme(theme)
            .diameter(34.0)
            .step(step)
            .show(v_ui);
        if resp.changed() {
            changed = true;
        }
        v_ui.label(egui::RichText::new(fmt_val).size(10.0));
    });
    changed
}

/// Renders the stereo VU meters and discrete/continuous mode toggle.
pub fn render_stereo_meters(
    content_ui: &mut Ui,
    theme: &MacTheme,
    state: &mut CenterState,
    scrubber_w: f32,
) {
    content_ui.add_space(12.0);
    content_ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Meter Mode:").size(11.0).strong());
        let style_idx = usize::from(!state.media_discrete_vu);
        MacSegmentedControl::new(&["Discrete LED", "Continuous Bar"], style_idx)
            .theme(theme)
            .height(22.0)
            .show(ui, |idx| {
                state.media_discrete_vu = idx == 0;
            });
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Threshold Colors:").size(11.0));
        ui.add(MacToggle::new(&mut state.media_threshold_vu).theme(theme));
    });
    content_ui.add_space(6.0);

    let vu_style = if state.media_discrete_vu {
        MacLevelIndicatorStyle::Discrete
    } else {
        MacLevelIndicatorStyle::Continuous
    };
    let meter_len = (scrubber_w - 110.0).clamp(180.0, 420.0);

    render_meter_channel(
        content_ui,
        theme,
        "Ch 1 (Left): ",
        state.audio_level_l,
        state.audio_peak_l,
        vu_style,
        state.media_threshold_vu,
        meter_len,
    );
    content_ui.add_space(4.0);
    render_meter_channel(
        content_ui,
        theme,
        "Ch 2 (Right):",
        state.audio_level_r,
        state.audio_peak_r,
        vu_style,
        state.media_threshold_vu,
        meter_len,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_meter_channel(
    ui: &mut Ui,
    theme: &MacTheme,
    label: &str,
    level: f32,
    peak: f32,
    style: MacLevelIndicatorStyle,
    threshold_colors: bool,
    length: f32,
) {
    ui.horizontal(|h_ui| {
        h_ui.label(egui::RichText::new(label).size(11.0).monospace());
        MacLevelIndicator::new(level, 0.0..=10.0)
            .theme(theme)
            .style(style)
            .orientation(MacLevelOrientation::Horizontal)
            .warning_value(6.5)
            .critical_value(8.5)
            .peak_hold(peak)
            .segments(16)
            .segmented_threshold_colors(threshold_colors)
            .tick_marks(6)
            .length(length)
            .show(h_ui);
        h_ui.label(
            egui::RichText::new(format!("{:4.1} dB", level - 10.0))
                .size(10.5)
                .monospace(),
        );
    });
}
