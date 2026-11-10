//! Procedural 60fps video canvas scene and radar/oscilloscope monitor (`SPEC-011`).

use super::CenterState;
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontId, Rect, Stroke, StrokeKind, Ui};
use egui_macview::theme::MacTheme;

/// Renders the 60fps procedural studio video viewport with color grading, radar, and oscilloscope.
pub fn render_procedural_video_viewport(
    content_ui: &mut Ui,
    theme: &MacTheme,
    state: &CenterState,
) -> Rect {
    let avail_w = (content_ui.available_width() - 24.0).clamp(320.0, 640.0);
    let video_h = avail_w * (9.0 / 16.0);
    let (video_rect, _) =
        content_ui.allocate_exact_size(vec2(avail_w, video_h), egui::Sense::hover());
    let vp = content_ui.painter_at(video_rect);

    // 1. Color-graded background
    let base_exposure = (state.media_exposure * 24.0) as i32;
    let c_factor = state.media_contrast.clamp(0.5, 2.0);
    let bg_val = (((22.0 + base_exposure as f32) * c_factor).round() as u8).clamp(8, 240);
    let video_bg = Color32::from_rgb(bg_val, bg_val + 2, bg_val + 6);

    vp.rect_filled(video_rect, CornerRadius::same(6), video_bg);
    vp.rect_stroke(
        video_rect,
        CornerRadius::same(6),
        Stroke::new(1.0, theme.separator()),
        StrokeKind::Inside,
    );

    // 2. Rotating radar / test pattern
    let v_center = video_rect.center();
    let zoom = state.media_zoom.clamp(0.5, 2.0);
    let pattern_r = 44.0 * zoom;
    let rot_angle = (state.media_playhead * 1.5) as f32;
    vp.circle_stroke(
        v_center,
        pattern_r,
        Stroke::new(1.5, theme.control_accent().gamma_multiply(0.65)),
    );
    vp.line_segment(
        [
            v_center + vec2(rot_angle.cos() * pattern_r, rot_angle.sin() * pattern_r),
            v_center - vec2(rot_angle.cos() * pattern_r, rot_angle.sin() * pattern_r),
        ],
        Stroke::new(1.5, theme.control_accent()),
    );

    // 3. Oscilloscope waveform
    let wave = &state.audio_track.waveform_peaks;
    if !wave.is_empty() {
        let num_pts = wave.len().min(48);
        let step_x = (video_rect.width() - 32.0) / (num_pts as f32);
        let base_y = video_rect.bottom() - 28.0;
        let pts: Vec<egui::Pos2> = (0..num_pts)
            .map(|i| {
                let y = base_y - wave[i] * 22.0 * state.media_gain;
                let x = video_rect.left() + 16.0 + (i as f32) * step_x;
                pos2(x, y)
            })
            .collect();
        if pts.len() >= 2 {
            vp.add(egui::Shape::line(
                pts,
                Stroke::new(1.0, theme.system_green().gamma_multiply(0.75)),
            ));
        }
    }

    // 4. SMPTE Timecode overlay
    let total_frames = (state.media_playhead * 24.0).floor() as u64;
    let frames = total_frames % 24;
    let secs = (total_frames / 24) % 60;
    let mins = (total_frames / 1440) % 60;
    let timecode_str = format!("TC 00:{:02}:{:02}:{:02} (24 fps)", mins, secs, frames);
    vp.text(
        pos2(video_rect.left() + 14.0, video_rect.top() + 14.0),
        Align2::LEFT_TOP,
        &timecode_str,
        FontId::monospace(11.0),
        theme.label(),
    );

    // 5. Header title and rating
    vp.text(
        pos2(video_rect.right() - 14.0, video_rect.top() + 14.0),
        Align2::RIGHT_TOP,
        format!("{} | {} ★", state.audio_track.name, state.media_rating),
        FontId::proportional(11.0),
        theme.secondary_label(),
    );

    video_rect
}
