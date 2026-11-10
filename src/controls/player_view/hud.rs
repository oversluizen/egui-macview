//! QuickTime floating HUD pod rendering and geometry calculation.
//!
//! Grounded in Apple AVKit `AVPlayerViewControlsStyleFloating` ($481.0 \times 64.0\,\text{pt}$, $R = 16.0\,\text{pt}$).

use crate::theme::MacTheme;
use egui::{CornerRadius, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

use super::types::{PlayerViewGeometry, PlayerViewState};
use super::MacPlayerView;

/// Computes floating HUD pod geometry inside `available_rect`.
pub(crate) fn compute_hud_geometry(view: &MacPlayerView<'_>, available_rect: Rect) -> PlayerViewGeometry {
    let theme_cow = MacTheme::resolve_or_default(view.theme);
    let m = theme_cow.player_view();

    let pod_w = m.hud_width.points().min(available_rect.width() - m.hud_bottom_margin.points());
    let pod_size = Vec2::new(pod_w, m.hud_height.points());
    let corner_r = m.hud_corner_radius.points();
    let pod_rect = Rect::from_center_size(
        Pos2::new(
            available_rect.center().x,
            available_rect.max.y - m.hud_bottom_margin.points() - pod_size.y * 0.5,
        ),
        pod_size,
    );

    let top_center_y = pod_rect.min.y + m.top_row_height.points() * 0.5;
    let btn_spacing = m.button_spacing.points();

    let mute_rect = Rect::from_center_size(
        Pos2::new(pod_rect.min.x + corner_r + m.mute_button_width.points() * 0.5, top_center_y),
        Vec2::new(m.mute_button_width.points(), m.mute_button_height.points()),
    );
    let vol_rect = Rect::from_min_size(
        Pos2::new(mute_rect.max.x + btn_spacing, top_center_y - m.volume_slider_height.points() * 0.5),
        Vec2::new(m.volume_slider_width.points(), m.volume_slider_height.points()),
    );
    let vol_track_rect = Rect::from_center_size(
        vol_rect.center(),
        Vec2::new(m.volume_slider_width.points(), m.scrubber_track_height.points()),
    );

    let play_rect = Rect::from_center_size(
        Pos2::new(pod_rect.center().x, top_center_y),
        Vec2::new(m.play_button_width.points(), m.play_button_height.points()),
    );
    let skip_back_rect = Rect::from_center_size(
        Pos2::new(play_rect.min.x - m.skip_button_width.points() * 0.5 - btn_spacing, top_center_y),
        Vec2::new(m.skip_button_width.points(), m.skip_button_height.points()),
    );
    let step_back_rect = Rect::from_center_size(
        Pos2::new(skip_back_rect.min.x - m.step_button_width.points() * 0.5 - btn_spacing, top_center_y),
        Vec2::new(m.step_button_width.points(), m.step_button_height.points()),
    );
    let skip_fwd_rect = Rect::from_center_size(
        Pos2::new(play_rect.max.x + m.skip_button_width.points() * 0.5 + btn_spacing, top_center_y),
        Vec2::new(m.skip_button_width.points(), m.skip_button_height.points()),
    );
    let step_fwd_rect = Rect::from_center_size(
        Pos2::new(skip_fwd_rect.max.x + m.step_button_width.points() * 0.5 + btn_spacing, top_center_y),
        Vec2::new(m.step_button_width.points(), m.step_button_height.points()),
    );
    let aux_sz = m.aux_button_size.points();
    let aux_rect = Rect::from_center_size(
        Pos2::new(pod_rect.max.x - corner_r - aux_sz * 0.5, top_center_y),
        Vec2::splat(aux_sz),
    );
    let speed_rect = Rect::from_center_size(
        Pos2::new(pod_rect.max.x - corner_r - aux_sz - m.speed_button_width.points() * 0.5 - btn_spacing, top_center_y),
        Vec2::new(m.speed_button_width.points(), m.speed_button_height.points()),
    );

    let scrub_h = m.scrubber_height.points();
    let track_h = m.scrubber_track_height.points();
    let scrub_center_y = pod_rect.max.y - scrub_h * 0.5;

    let dur = view.duration_secs.max(1.0);
    let time_label_w = if dur as u32 >= 3600 {
        m.speed_button_width.points() + m.button_spacing.points() * 2.0
    } else {
        m.speed_button_width.points()
    };
    let el_pos = Pos2::new(pod_rect.min.x + corner_r, scrub_center_y - m.scrubber_knob_size.points() * 0.5);
    let rem_pos = Pos2::new(pod_rect.max.x - corner_r - time_label_w, scrub_center_y - m.scrubber_knob_size.points() * 0.5);

    let track_start_x = el_pos.x + time_label_w + btn_spacing;
    let track_end_x = rem_pos.x - btn_spacing;
    let track_rect = Rect::from_min_max(
        Pos2::new(track_start_x, scrub_center_y - track_h * 0.5),
        Pos2::new(track_end_x, scrub_center_y + track_h * 0.5),
    );
    let scrub_hit = Rect::from_min_max(
        Pos2::new(track_start_x, pod_rect.max.y - scrub_h),
        Pos2::new(track_end_x, pod_rect.max.y),
    );
    let knob_x = track_rect.min.x + track_rect.width() * view.progress.clamp(0.0, 1.0);

    PlayerViewGeometry {
        total_rect: available_rect,
        controls_rect: pod_rect,
        play_rect,
        skip_back_rect,
        skip_fwd_rect,
        step_back_rect,
        step_fwd_rect,
        mute_rect,
        volume_rect: vol_rect,
        volume_track_rect: vol_track_rect,
        speed_rect,
        aux_rect,
        scrubber_track_rect: track_rect,
        scrubber_hit_rect: scrub_hit,
        elapsed_pos: el_pos,
        remaining_pos: rem_pos,
        knob_pos: Pos2::new(knob_x, scrub_center_y),
        corner_radius: corner_r,
    }
}

/// Paints the floating HUD pod controls.
pub(crate) fn paint_hud(
    view: &MacPlayerView<'_>,
    painter: &Painter,
    geom: &PlayerViewGeometry,
    state: &PlayerViewState,
    theme: &MacTheme,
) {
    let alpha = state.alpha;
    let m = theme.player_view();

    let pod_rect = geom.controls_rect;
    let hud_bg = theme.control_background().gamma_multiply(alpha);
    let hud_stroke = Stroke::new(1.0, theme.separator().gamma_multiply(alpha));
    painter.rect(
        pod_rect,
        CornerRadius::same(geom.corner_radius as u8),
        hud_bg,
        hud_stroke,
        StrokeKind::Inside,
    );

    // Audio Mute button
    painter.circle_filled(geom.mute_rect.center(), (geom.mute_rect.height() * 0.5) * 0.5, theme.label().gamma_multiply(alpha));
    if view.muted {
        painter.line_segment([geom.mute_rect.left_top(), geom.mute_rect.right_bottom()], Stroke::new(1.0, theme.label().gamma_multiply(alpha)));
    }

    // Audio Volume slider
    painter.rect_filled(geom.volume_track_rect, CornerRadius::same(1), theme.separator().gamma_multiply(alpha));
    let vol_fill_w = geom.volume_track_rect.width() * if view.muted { 0.0 } else { view.volume.clamp(0.0, 1.0) };
    painter.rect_filled(
        Rect::from_min_size(geom.volume_track_rect.min, Vec2::new(vol_fill_w, geom.volume_track_rect.height())),
        CornerRadius::same(1),
        theme.control_accent().gamma_multiply(alpha),
    );

    // Play / Pause Icon
    let icon_h = m.icon_size.points();
    let pc = geom.play_rect.center();
    if view.is_playing {
        let b1 = Rect::from_center_size(Pos2::new(pc.x - 2.0, pc.y), Vec2::new(2.0, icon_h));
        let b2 = Rect::from_center_size(Pos2::new(pc.x + 2.0, pc.y), Vec2::new(2.0, icon_h));
        painter.rect_filled(b1, CornerRadius::same(0), theme.label().gamma_multiply(alpha));
        painter.rect_filled(b2, CornerRadius::same(0), theme.label().gamma_multiply(alpha));
    } else {
        let r = icon_h * 0.5;
        let p1 = Pos2::new(pc.x - r * 0.5, pc.y - r);
        let p2 = Pos2::new(pc.x - r * 0.5, pc.y + r);
        let p3 = Pos2::new(pc.x + r, pc.y);
        painter.add(egui::Shape::convex_polygon(vec![p1, p2, p3], theme.label().gamma_multiply(alpha), Stroke::NONE));
    }

    // Transport skip/step glyphs, speed pill background, and auxiliary AirPlay button
    super::inline::paint_hud_aux_and_step_glyphs(painter, geom, theme, alpha);

    // Speed rate pill text
    let speed_text = if (view.playback_rate * 2.0).fract() == 0.0 {
        format!("{:.1}x", view.playback_rate)
    } else {
        format!("{:.2}x", view.playback_rate)
    };
    let sp_galley = painter.layout_no_wrap(speed_text, egui::TextStyle::Small.resolve(&egui::Style::default()), theme.label().gamma_multiply(alpha));
    painter.galley(Pos2::new(geom.speed_rect.center().x - sp_galley.size().x * 0.5, geom.speed_rect.center().y - sp_galley.size().y * 0.5), sp_galley, theme.label().gamma_multiply(alpha));

    // Scrubber base track
    let tr = geom.scrubber_track_rect;
    painter.rect_filled(tr, CornerRadius::same(m.scrubber_track_corner_radius.points() as u8), theme.separator().gamma_multiply(alpha));

    // Buffered fill
    if view.buffered_progress > 0.0 {
        let bw = tr.width() * view.buffered_progress.clamp(0.0, 1.0);
        painter.rect_filled(Rect::from_min_size(tr.min, Vec2::new(bw, tr.height())), CornerRadius::same(m.scrubber_track_corner_radius.points() as u8), theme.label().gamma_multiply(alpha * m.buffered_fill_alpha.points()));
    }

    // Elapsed fill
    if view.progress > 0.0 {
        let ew = tr.width() * view.progress.clamp(0.0, 1.0);
        painter.rect_filled(Rect::from_min_size(tr.min, Vec2::new(ew, tr.height())), CornerRadius::same(m.scrubber_track_corner_radius.points() as u8), theme.control_accent().gamma_multiply(alpha));
    }

    // Playhead knob
    let knob_sz = m.scrubber_knob_size.points();
    painter.circle_filled(geom.knob_pos, knob_sz * 0.5, theme.scrubber_playhead().gamma_multiply(alpha));

    // Timecode readouts
    let dur = view.duration_secs.max(1.0);
    let el_secs = (view.progress * dur) as u32;
    let rem_secs = ((1.0 - view.progress) * dur) as u32;
    let el_str = super::types::format_timecode(el_secs, dur);
    let rem_str = super::types::format_remaining_timecode(rem_secs, dur);
    let sm_style = egui::TextStyle::Small.resolve(&egui::Style::default());
    let el_g = painter.layout_no_wrap(el_str, sm_style.clone(), theme.label().gamma_multiply(alpha));
    let rem_g = painter.layout_no_wrap(rem_str, sm_style, theme.label().gamma_multiply(alpha));
    painter.galley(geom.elapsed_pos, el_g, theme.label().gamma_multiply(alpha));
    painter.galley(geom.remaining_pos, rem_g, theme.label().gamma_multiply(alpha));
}
