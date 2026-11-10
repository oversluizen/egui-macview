//! Fixed bottom inline media control bar for `MacPlayerView`.
//!
//! Grounded in Apple AVKit `AVPlayerViewControlsStyleInline` (24.0 pt height),
//! featuring interactive Play/Pause, Scrubber with buffered fill, Mute button, and Time labels.

use crate::theme::MacTheme;
use egui::{CornerRadius, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};

use super::types::{PlayerViewGeometry, PlayerViewState};
use super::MacPlayerView;

/// Computes inline bottom bar geometry inside `available_rect`.
pub(crate) fn compute_inline_geometry(view: &MacPlayerView<'_>, available_rect: Rect) -> PlayerViewGeometry {
    let theme_cow = MacTheme::resolve_or_default(view.theme);
    let m = theme_cow.player_view();

    let bar_h = m.inline_height.points();
    let bar_rect = Rect::from_min_size(
        Pos2::new(available_rect.min.x, available_rect.max.y - bar_h),
        Vec2::new(available_rect.width(), bar_h),
    );
    let play_inset = m.inline_play_button_inset.points();
    let play_w = m.inline_play_button_width.points();
    let play_rect = Rect::from_min_size(
        Pos2::new(bar_rect.min.x + play_inset, bar_rect.min.y + 2.0),
        Vec2::new(play_w, bar_rect.height() - play_inset * 0.5),
    );
    let mute_w = m.mute_button_width.points();
    let mute_rect = Rect::from_center_size(
        Pos2::new(bar_rect.max.x - play_inset - mute_w * 0.5, bar_rect.center().y),
        Vec2::new(mute_w, m.mute_button_height.points()),
    );
    let btn_spacing = m.button_spacing.points();
    let dur = view.duration_secs.max(1.0);
    let time_label_w = if dur as u32 >= 3600 {
        m.speed_button_width.points() + m.button_spacing.points() * 2.0
    } else {
        m.speed_button_width.points()
    };
    let el_pos = Pos2::new(play_rect.max.x + btn_spacing, bar_rect.center().y - m.scrubber_knob_size.points() * 0.5);
    let rem_pos = Pos2::new(mute_rect.min.x - btn_spacing - time_label_w, bar_rect.center().y - m.scrubber_knob_size.points() * 0.5);

    let track_start_x = el_pos.x + time_label_w + btn_spacing;
    let track_end_x = rem_pos.x - btn_spacing;
    let track_h = m.scrubber_track_height.points();
    let track_rect = Rect::from_min_max(
        Pos2::new(track_start_x, bar_rect.center().y - track_h * 0.5),
        Pos2::new(track_end_x, bar_rect.center().y + track_h * 0.5),
    );
    let scrub_hit = Rect::from_min_max(
        Pos2::new(track_start_x, bar_rect.min.y),
        Pos2::new(track_end_x, bar_rect.max.y),
    );
    let knob_x = track_rect.min.x + track_rect.width() * view.progress.clamp(0.0, 1.0);

    PlayerViewGeometry {
        total_rect: available_rect,
        controls_rect: bar_rect,
        play_rect,
        skip_back_rect: Rect::NOTHING,
        skip_fwd_rect: Rect::NOTHING,
        step_back_rect: Rect::NOTHING,
        step_fwd_rect: Rect::NOTHING,
        mute_rect,
        volume_rect: Rect::NOTHING,
        volume_track_rect: Rect::NOTHING,
        speed_rect: Rect::NOTHING,
        aux_rect: Rect::NOTHING,
        scrubber_track_rect: track_rect,
        scrubber_hit_rect: scrub_hit,
        elapsed_pos: el_pos,
        remaining_pos: rem_pos,
        knob_pos: Pos2::new(knob_x, bar_rect.center().y),
        corner_radius: m.inline_corner_radius.points(),
    }
}

/// Paints the inline bottom bar controls.
pub(crate) fn paint_inline(
    view: &MacPlayerView<'_>,
    painter: &Painter,
    geom: &PlayerViewGeometry,
    _state: &PlayerViewState,
    theme: &MacTheme,
) {
    let m = theme.player_view();
    let rect = geom.controls_rect;
    painter.rect_filled(rect, CornerRadius::same(0), theme.control_background());
    painter.line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.0, theme.separator()));

    // Play/Pause icon
    let icon_h = m.icon_size.points();
    let pc = geom.play_rect.center();
    if view.is_playing {
        let b1 = Rect::from_center_size(Pos2::new(pc.x - 2.0, pc.y), Vec2::new(2.0, icon_h));
        let b2 = Rect::from_center_size(Pos2::new(pc.x + 2.0, pc.y), Vec2::new(2.0, icon_h));
        painter.rect_filled(b1, CornerRadius::same(0), theme.label());
        painter.rect_filled(b2, CornerRadius::same(0), theme.label());
    } else {
        let r = icon_h * 0.5;
        let p1 = Pos2::new(pc.x - r * 0.5, pc.y - r);
        let p2 = Pos2::new(pc.x - r * 0.5, pc.y + r);
        let p3 = Pos2::new(pc.x + r, pc.y);
        painter.add(egui::Shape::convex_polygon(vec![p1, p2, p3], theme.label(), Stroke::NONE));
    }

    // Mute button
    painter.circle_filled(geom.mute_rect.center(), (geom.mute_rect.height() * 0.5) * 0.5, theme.label());
    if view.muted {
        painter.line_segment([geom.mute_rect.left_top(), geom.mute_rect.right_bottom()], Stroke::new(1.0, theme.label()));
    }

    // Scrubber
    let tr = geom.scrubber_track_rect;
    if tr.width() > m.icon_size.points() {
        painter.rect_filled(tr, CornerRadius::same(m.scrubber_track_corner_radius.points() as u8), theme.separator());
        if view.buffered_progress > 0.0 {
            let bw = tr.width() * view.buffered_progress.clamp(0.0, 1.0);
            painter.rect_filled(Rect::from_min_size(tr.min, Vec2::new(bw, tr.height())), CornerRadius::same(m.scrubber_track_corner_radius.points() as u8), theme.label().gamma_multiply(m.buffered_fill_alpha.points()));
        }
        if view.progress > 0.0 {
            let ew = tr.width() * view.progress.clamp(0.0, 1.0);
            painter.rect_filled(Rect::from_min_size(tr.min, Vec2::new(ew, tr.height())), CornerRadius::same(m.scrubber_track_corner_radius.points() as u8), theme.control_accent());
        }
        painter.circle_filled(geom.knob_pos, m.scrubber_knob_size.points() * 0.5, theme.scrubber_playhead());
    }

    // Time labels
    let dur = view.duration_secs.max(1.0);
    let el_secs = (view.progress * dur) as u32;
    let rem_secs = ((1.0 - view.progress) * dur) as u32;
    let el_str = super::types::format_timecode(el_secs, dur);
    let rem_str = super::types::format_remaining_timecode(rem_secs, dur);
    let sm_style = egui::TextStyle::Small.resolve(&egui::Style::default());
    let el_g = painter.layout_no_wrap(el_str, sm_style.clone(), theme.secondary_label());
    let rem_g = painter.layout_no_wrap(rem_str, sm_style, theme.secondary_label());
    painter.galley(geom.elapsed_pos, el_g, theme.secondary_label());
    painter.galley(geom.remaining_pos, rem_g, theme.secondary_label());
}

/// Paints transport skip/step glyphs, speed pill background, and AirPlay auxiliary icon for floating HUD.
pub(crate) fn paint_hud_aux_and_step_glyphs(
    painter: &Painter,
    geom: &PlayerViewGeometry,
    theme: &MacTheme,
    alpha: f32,
) {
    let color = theme.label().gamma_multiply(alpha);

    // 1. Skip Backward (15s): circular arc + left chevron
    let sb_c = geom.skip_back_rect.center();
    let sb_r = (geom.skip_back_rect.height() * 0.5) * 0.5;
    painter.circle_stroke(sb_c, sb_r, Stroke::new(1.0, color));
    let ch_w = sb_r * 0.5;
    painter.line_segment([Pos2::new(sb_c.x, sb_c.y - ch_w), Pos2::new(sb_c.x - ch_w, sb_c.y)], Stroke::new(1.0, color));
    painter.line_segment([Pos2::new(sb_c.x - ch_w, sb_c.y), Pos2::new(sb_c.x, sb_c.y + ch_w)], Stroke::new(1.0, color));

    // 2. Step Backward (1 frame): vertical bar on left + left-pointing triangle
    let stp_b_c = geom.step_back_rect.center();
    let stp_b_h = (geom.step_back_rect.height() * 0.5) * 0.5;
    let bar_b_x = stp_b_c.x - stp_b_h;
    painter.line_segment([Pos2::new(bar_b_x, stp_b_c.y - stp_b_h), Pos2::new(bar_b_x, stp_b_c.y + stp_b_h)], Stroke::new(1.0, color));
    let tri_b1 = Pos2::new(stp_b_c.x + stp_b_h, stp_b_c.y - stp_b_h);
    let tri_b2 = Pos2::new(stp_b_c.x - stp_b_h * 0.5, stp_b_c.y);
    let tri_b3 = Pos2::new(stp_b_c.x + stp_b_h, stp_b_c.y + stp_b_h);
    painter.add(Shape::convex_polygon(vec![tri_b1, tri_b2, tri_b3], color, Stroke::NONE));

    // 3. Skip Forward (15s): circular arc + right chevron
    let sf_c = geom.skip_fwd_rect.center();
    let sf_r = (geom.skip_fwd_rect.height() * 0.5) * 0.5;
    painter.circle_stroke(sf_c, sf_r, Stroke::new(1.0, color));
    let ch_f_w = sf_r * 0.5;
    painter.line_segment([Pos2::new(sf_c.x, sf_c.y - ch_f_w), Pos2::new(sf_c.x + ch_f_w, sf_c.y)], Stroke::new(1.0, color));
    painter.line_segment([Pos2::new(sf_c.x + ch_f_w, sf_c.y), Pos2::new(sf_c.x, sf_c.y + ch_f_w)], Stroke::new(1.0, color));

    // 4. Step Forward (1 frame): right-pointing triangle + vertical bar on right
    let stp_f_c = geom.step_fwd_rect.center();
    let stp_f_h = (geom.step_fwd_rect.height() * 0.5) * 0.5;
    let bar_f_x = stp_f_c.x + stp_f_h;
    painter.line_segment([Pos2::new(bar_f_x, stp_f_c.y - stp_f_h), Pos2::new(bar_f_x, stp_f_c.y + stp_f_h)], Stroke::new(1.0, color));
    let tri_f1 = Pos2::new(stp_f_c.x - stp_f_h, stp_f_c.y - stp_f_h);
    let tri_f2 = Pos2::new(stp_f_c.x + stp_f_h * 0.5, stp_f_c.y);
    let tri_f3 = Pos2::new(stp_f_c.x - stp_f_h, stp_f_c.y + stp_f_h);
    painter.add(Shape::convex_polygon(vec![tri_f1, tri_f2, tri_f3], color, Stroke::NONE));

    // 5. Speed pill background / border
    let speed_h = geom.speed_rect.height();
    painter.rect(
        geom.speed_rect,
        CornerRadius::same((speed_h * 0.5) as u8),
        theme.control_background().gamma_multiply(alpha),
        Stroke::new(1.0, theme.separator().gamma_multiply(alpha)),
        StrokeKind::Inside,
    );

    // 6. Auxiliary AirPlay / PiP button: screen rect + bottom broadcast triangle
    let aux = geom.aux_rect;
    let screen_w = aux.width() * 0.5 + (aux.width() * 0.5) * 0.5;
    let screen_h = aux.height() * 0.5;
    let screen_rect = Rect::from_center_size(
        Pos2::new(aux.center().x, aux.min.y + screen_h * 0.5),
        Vec2::new(screen_w, screen_h),
    );
    painter.rect_stroke(
        screen_rect,
        CornerRadius::same(1),
        Stroke::new(1.0, color),
        StrokeKind::Inside,
    );
    let tri_w = (aux.width() * 0.5) * 0.5;
    let tri_top = Pos2::new(aux.center().x, screen_rect.max.y - 1.0);
    let tri_left = Pos2::new(aux.center().x - tri_w, aux.max.y);
    let tri_right = Pos2::new(aux.center().x + tri_w, aux.max.y);
    painter.add(Shape::convex_polygon(vec![tri_top, tri_right, tri_left], color, Stroke::NONE));
}
