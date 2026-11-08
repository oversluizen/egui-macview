//! Waveform, timeline zone, and QuickTime trim handle painter for [`super::MacTrimScrubber`].

use crate::theme::MacTheme;
use egui::{Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

/// Named background timeline region within [`super::MacTrimScrubber`].
#[derive(Debug, Clone)]
pub struct MacTimelineZone<'a> {
    /// Start timestamp in seconds.
    pub start: f64,
    /// End timestamp in seconds.
    pub end: f64,
    /// Zone display label.
    pub label: &'a str,
    /// Optional subtle background tint color.
    pub tint: Option<Color32>,
}

impl<'a> MacTimelineZone<'a> {
    /// Creates a new timeline zone spanning `start..=end` in seconds.
    pub fn new(start: f64, end: f64, label: &'a str) -> Self {
        Self {
            start,
            end,
            label,
            tint: None,
        }
    }

    /// Sets an optional custom tint color for this timeline zone.
    pub fn tint(mut self, tint: Color32) -> Self {
        self.tint = Some(tint);
        self
    }
}

/// Paints the track background, named zones, normalized waveform, outside dim scrims,
/// yellow In/Out trim frame with grip slots, and playhead needle with `MM:SS.d` badge.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_trim_scrubber(
    painter: &Painter,
    theme: &MacTheme,
    full_rect: Rect,
    track_rect: Rect,
    duration: f64,
    in_point: f64,
    out_point: f64,
    playhead: f64,
    waveform: &[f32],
    zones: &[MacTimelineZone<'_>],
    timecode_text: &str,
) {
    let sm = theme.trim_scrubber();
    let corner_r = CornerRadius::same(sm.corner_radius.points() as u8);
    let handle_w = sm.handle_width.points();
    let border_w = sm.selection_border_width.points();
    let dur = duration.max(f64::EPSILON);

    let time_to_x = |t: f64| -> f32 {
        let frac = (t / dur).clamp(0.0, 1.0) as f32;
        track_rect.left() + frac * track_rect.width()
    };

    // 1. Track base background
    painter.rect(
        track_rect,
        corner_r,
        theme.control_background(),
        Stroke::new(border_w * 0.5, theme.separator()),
        StrokeKind::Inside,
    );

    // 2. Named background timeline zones
    let zone_pad = sm.zone_label_padding.points();
    let zone_font = FontId::proportional(sm.zone_font_size.points());
    for zone in zones {
        let zx0 = time_to_x(zone.start);
        let zx1 = time_to_x(zone.end);
        if zx1 > zx0 {
            let z_rect = Rect::from_min_max(
                Pos2::new(zx0, track_rect.top()),
                Pos2::new(zx1, track_rect.bottom()),
            );
            let tint = zone.tint.unwrap_or_else(|| theme.control());
            painter.rect_filled(z_rect, 0, tint);
            painter.line_segment(
                [
                    Pos2::new(zx1, track_rect.top()),
                    Pos2::new(zx1, track_rect.bottom()),
                ],
                Stroke::new(border_w * 0.5, theme.separator()),
            );
            if !zone.label.is_empty() {
                painter.text(
                    Pos2::new(zx0 + zone_pad, track_rect.top() + zone_pad * 0.5),
                    Align2::LEFT_TOP,
                    zone.label,
                    zone_font.clone(),
                    theme.secondary_label(),
                );
            }
        }
    }

    // 3. Normalized waveform profile
    if !waveform.is_empty() {
        let count = waveform.len();
        let bar_w = sm.waveform_bar_width.points();
        let min_amp = sm.waveform_min_amplitude.points();
        let max_half_h = track_rect.height() * 0.5;
        let cy = track_rect.center().y;
        let wave_stroke = Stroke::new(bar_w, theme.scrubber_waveform());

        for (i, &sample) in waveform.iter().enumerate() {
            let frac = (i as f32 + 0.5) / (count as f32);
            let x = track_rect.left() + frac * track_rect.width();
            let amp = sample.abs().clamp(min_amp, 1.0);
            let half_h = max_half_h * amp;
            painter.line_segment(
                [Pos2::new(x, cy - half_h), Pos2::new(x, cy + half_h)],
                wave_stroke,
            );
        }
    }

    let in_x = time_to_x(in_point);
    let out_x = time_to_x(out_point);

    // 4. Dimmed outside regions
    let dim_color = theme.scrubber_outside_dim();
    if in_x > track_rect.left() {
        let left_dim = Rect::from_min_max(track_rect.min, Pos2::new(in_x, track_rect.bottom()));
        painter.rect_filled(left_dim, corner_r, dim_color);
    }
    if out_x < track_rect.right() {
        let right_dim = Rect::from_min_max(Pos2::new(out_x, track_rect.top()), track_rect.max);
        painter.rect_filled(right_dim, corner_r, dim_color);
    }

    // 5. Yellow/accent QuickTime selection frame & In/Out grab handles
    let trim_color = theme.scrubber_selection_border();
    let sel_rect = Rect::from_min_max(
        Pos2::new(in_x, track_rect.top()),
        Pos2::new(out_x.max(in_x), track_rect.bottom()),
    );
    painter.rect_stroke(
        sel_rect,
        corner_r,
        Stroke::new(border_w, trim_color),
        StrokeKind::Inside,
    );

    let left_handle = Rect::from_min_size(
        Pos2::new(in_x, track_rect.top()),
        Vec2::new(handle_w, track_rect.height()),
    );
    let right_handle = Rect::from_min_size(
        Pos2::new((out_x - handle_w).max(in_x), track_rect.top()),
        Vec2::new(handle_w, track_rect.height()),
    );
    painter.rect_filled(left_handle, corner_r, trim_color);
    painter.rect_filled(right_handle, corner_r, trim_color);

    let slot_half = sm.handle_slot_height.points() * 0.5;
    let slot_stroke = Stroke::new(sm.handle_slot_width.points(), theme.control_background());
    for hc in [left_handle.center(), right_handle.center()] {
        painter.line_segment(
            [
                Pos2::new(hc.x, hc.y - slot_half),
                Pos2::new(hc.x, hc.y + slot_half),
            ],
            slot_stroke,
        );
    }

    // 6. Playhead needle & MM:SS.d timecode pill badge
    let play_x = time_to_x(playhead);
    let badge_h = sm.badge_height.points();
    let badge_pad_h = sm.badge_padding_horizontal.points();
    let badge_r = CornerRadius::same(sm.badge_corner_radius.points() as u8);
    let badge_font = FontId::monospace(sm.badge_font_size.points());

    painter.line_segment(
        [
            Pos2::new(play_x, track_rect.top() - border_w),
            Pos2::new(play_x, track_rect.bottom() + border_w),
        ],
        Stroke::new(sm.playhead_width.points(), theme.scrubber_playhead()),
    );

    let badge_w = (timecode_text.len() as f32) * (sm.badge_font_size.points() * 0.5)
        + badge_pad_h * 2.0
        + handle_w;
    let clamped_bx = play_x.clamp(
        full_rect.left() + badge_w * 0.5,
        full_rect.right() - badge_w * 0.5,
    );
    let badge_rect = Rect::from_center_size(
        Pos2::new(clamped_bx, full_rect.top() + badge_h * 0.5),
        Vec2::new(badge_w, badge_h),
    );
    painter.rect_filled(badge_rect, badge_r, theme.scrubber_outside_dim());
    painter.text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        timecode_text,
        badge_font,
        theme.scrubber_playhead(),
    );
}
