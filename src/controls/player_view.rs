//! Authentic macOS AVKit Media Player View (`AVPlayerView` equivalent, REQ-012-01).
//!
//! Grounded in Apple AVKit Sonoma 14.8.9 hardware inspection (Darwin 23.6.0)
//! supporting QuickTime floating HUD pod (`AVPlayerViewControlsStyleFloating`),
//! inline bottom bar (`AVPlayerViewControlsStyleInline`), and minimal transport (`AVPlayerViewControlsStyleMinimal`).

use crate::layout::subpixel_snap_rect;
use crate::motion::CASpring;
use crate::theme::MacTheme;
use egui::{
    CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget,
};

/// AVKit media presentation controls styles matching `AVPlayerViewControlsStyle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacPlayerControlsStyle {
    /// QuickTime floating translucent HUD pod (`AVPlayerViewControlsStyleFloating` = 2).
    #[default]
    Floating,
    /// Fixed bottom control bar (`AVPlayerViewControlsStyleInline` = 1).
    Inline,
    /// Minimal circular transport overlay (`AVPlayerViewControlsStyleMinimal` = 3).
    Minimal,
}

/// Authentic macOS AVKit Media Player View (`AVPlayerView` equivalent).
pub struct MacPlayerView<'a> {
    controls_style: MacPlayerControlsStyle,
    is_playing: bool,
    is_playing_ref: Option<&'a mut bool>,
    progress: f32,
    progress_ref: Option<&'a mut f32>,
    theme: Option<&'a MacTheme>,
}

impl<'a> Default for MacPlayerView<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MacPlayerView<'a> {
    /// Creates a new player view widget.
    pub fn new() -> Self {
        Self {
            controls_style: MacPlayerControlsStyle::Floating,
            is_playing: false,
            is_playing_ref: None,
            progress: 0.0,
            progress_ref: None,
            theme: None,
        }
    }

    /// Sets the media controls style (`Floating`, `Inline`, or `Minimal`).
    pub fn controls_style(mut self, style: MacPlayerControlsStyle) -> Self {
        self.controls_style = style;
        self
    }

    /// Sets playback status by value.
    pub fn is_playing(mut self, playing: bool) -> Self {
        self.is_playing = playing;
        self
    }

    /// Binds mutable playback status reference for two-way state synchronization.
    pub fn is_playing_mut(mut self, playing: &'a mut bool) -> Self {
        self.is_playing = *playing;
        self.is_playing_ref = Some(playing);
        self
    }

    /// Sets timeline playback progress in $[0.0, 1.0]$ by value.
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress;
        self
    }

    /// Binds mutable playback progress reference for two-way timeline scrubbing.
    pub fn progress_mut(mut self, progress: &'a mut f32) -> Self {
        self.progress = *progress;
        self.progress_ref = Some(progress);
        self
    }

    /// Injects dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Floating HUD pod dimensions ($481.0 \times 64.0\,\text{pt}$).
    pub fn hud_pod_size(&self) -> Vec2 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        let m = theme_cow.player_view();
        Vec2::new(m.hud_width.points(), m.hud_height.points())
    }

    /// Floating HUD pod corner radius ($16.0\,\text{pt}$).
    pub fn hud_corner_radius(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        theme_cow.player_view().hud_corner_radius.points()
    }

    /// Floating HUD auto-hide idle threshold in seconds ($3.0\,\text{s}$).
    pub fn auto_hide_seconds(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        theme_cow.player_view().auto_hide_seconds.points()
    }

    /// Timeline scrubber container height ($22.5\,\text{pt}$).
    pub fn scrubber_height(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        theme_cow.player_view().scrubber_height.points()
    }

    /// Timeline scrubber track height ($3.5\,\text{pt}$).
    pub fn scrubber_track_height(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        theme_cow.player_view().scrubber_track_height.points()
    }

    /// Timeline scrubber playhead knob dimensions ($8.0 \times 8.0\,\text{pt}$).
    pub fn scrubber_knob_size(&self) -> Vec2 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        let sz = theme_cow.player_view().scrubber_knob_size.points();
        Vec2::new(sz, sz)
    }

    /// Inline bottom bar height ($24.0\,\text{pt}$).
    pub fn inline_bar_height(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        theme_cow.player_view().inline_height.points()
    }

    /// Inline bottom bar corner radius ($0.0\,\text{pt}$).
    pub fn inline_corner_radius(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        theme_cow.player_view().inline_corner_radius.points()
    }
}

impl<'a> Widget for MacPlayerView<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.player_view();

        let available_w = ui.available_width();
        let total_size = match self.controls_style {
            MacPlayerControlsStyle::Floating => {
                let pod = self.hud_pod_size();
                Vec2::new(available_w.max(pod.x), pod.y + metrics.hud_bottom_margin.points() * 2.0)
            }
            MacPlayerControlsStyle::Inline => {
                Vec2::new(available_w, self.inline_bar_height())
            }
            MacPlayerControlsStyle::Minimal => {
                let pod = self.hud_pod_size();
                Vec2::new(available_w, pod.y)
            }
        };

        let (rect, mut response) = ui.allocate_exact_size(total_size, Sense::click_and_drag());
        let rect = subpixel_snap_rect(rect, ui.ctx().pixels_per_point());

        let painter = ui.painter_at(rect);

        match self.controls_style {
            MacPlayerControlsStyle::Floating => {
                let pod_size = self.hud_pod_size();
                let corner_r = self.hud_corner_radius();
                let pod_rect = Rect::from_center_size(
                    Pos2::new(rect.center().x, rect.max.y - metrics.hud_bottom_margin.points() - pod_size.y * 0.5),
                    pod_size,
                );

                // CASpring auto-hide animation on pointer motion/hover and 3.0s idle threshold
                let now = ui.ctx().input(|i| i.time);
                let last_active_id = Id::NULL.with("hud_last_active");
                let mut last_active = ui.ctx().data(|d| d.get_temp::<f64>(last_active_id)).unwrap_or(now);

                let pointer_pos = ui.input(|i| i.pointer.hover_pos());
                let pointer_in_video = pointer_pos.is_some_and(|p| rect.contains(p));
                if pointer_in_video {
                    last_active = now;
                    ui.ctx().data_mut(|d| d.insert_temp(last_active_id, now));
                }

                let idle_time = (now - last_active) as f32;
                let auto_hide_thresh = metrics.auto_hide_seconds.points();
                let is_hovered = pointer_pos.is_some_and(|p| pod_rect.contains(p));
                let target_alpha = if is_hovered || !self.is_playing || idle_time < auto_hide_thresh {
                    1.0
                } else {
                    0.0
                };

                let spring = CASpring::new(1.0, 1.0);
                let alpha = spring.animate_f32(ui.ctx(), Id::NULL.with("hud_alpha"), target_alpha);

                if alpha > 0.0 {
                    let hud_bg = theme.control_background().gamma_multiply(alpha);
                    let hud_stroke = Stroke::new(1.0, theme.separator().gamma_multiply(alpha));

                    painter.rect(
                        pod_rect,
                        CornerRadius::same(corner_r as u8),
                        hud_bg,
                        hud_stroke,
                        StrokeKind::Inside,
                    );

                    // Scrubber bottom row
                    let scrub_h = self.scrubber_height();
                    let track_h = self.scrubber_track_height();
                    let knob_sz = self.scrubber_knob_size();

                    let scrub_center_y = pod_rect.max.y - scrub_h * 0.5;
                    let track_margin = corner_r * 2.0;
                    let track_rect = Rect::from_center_size(
                        Pos2::new(pod_rect.center().x, scrub_center_y),
                        Vec2::new(pod_rect.width() - track_margin * 2.0, track_h),
                    );

                    let scrub_hit_rect = Rect::from_center_size(
                        track_rect.center(),
                        Vec2::new(track_rect.width(), scrub_h),
                    );

                    let interact_pos = ui.input(|i| i.pointer.interact_pos());
                    if interact_pos.is_some_and(|p| scrub_hit_rect.contains(p)) && ui.input(|i| i.pointer.primary_down()) {
                        if let Some(pos) = interact_pos {
                            let prog = ((pos.x - track_rect.min.x) / track_rect.width()).clamp(0.0, 1.0);
                            self.progress = prog;
                            if let Some(ref mut p_ref) = self.progress_ref {
                                **p_ref = prog;
                            }
                            response.mark_changed();
                        }
                    }

                    painter.rect_filled(
                        track_rect,
                        CornerRadius::same(metrics.scrubber_track_corner_radius.points() as u8),
                        theme.separator().gamma_multiply(alpha),
                    );

                    let knob_x = track_rect.min.x + track_rect.width() * self.progress.clamp(0.0, 1.0);
                    let knob_pos = Pos2::new(knob_x, scrub_center_y);
                    painter.circle_filled(
                        knob_pos,
                        knob_sz.x * 0.5,
                        theme.control_accent().gamma_multiply(alpha),
                    );

                    // Timecode readouts
                    let total_secs = 100;
                    let elapsed_secs = (self.progress * (total_secs as f32)) as u32;
                    let remain_secs = ((1.0 - self.progress) * (total_secs as f32)) as u32;
                    let elapsed_str = format!("{:02}:{:02}", elapsed_secs / 60, elapsed_secs % 60);
                    let remain_str = format!("-{:02}:{:02}", remain_secs / 60, remain_secs % 60);

                    let text_style = ui.style().text_styles.get(&egui::TextStyle::Small).cloned().unwrap_or_default();
                    let el_galley = painter.layout_no_wrap(elapsed_str, text_style.clone(), theme.label().gamma_multiply(alpha));
                    let rem_galley = painter.layout_no_wrap(remain_str, text_style, theme.label().gamma_multiply(alpha));

                    painter.galley(
                        Pos2::new(pod_rect.min.x + corner_r, scrub_center_y - el_galley.size().y * 0.5),
                        el_galley,
                        theme.label().gamma_multiply(alpha),
                    );
                    painter.galley(
                        Pos2::new(pod_rect.max.x - corner_r - rem_galley.size().x, scrub_center_y - rem_galley.size().y * 0.5),
                        rem_galley,
                        theme.label().gamma_multiply(alpha),
                    );

                    // Center transport play/pause icon in top row
                    let play_center = Pos2::new(pod_rect.center().x, pod_rect.min.y + metrics.top_row_height.points() * 0.5);
                    let play_hit = Rect::from_center_size(play_center, Vec2::new(knob_sz.x * 2.0, knob_sz.x * 2.0));
                    if interact_pos.is_some_and(|p| play_hit.contains(p)) && ui.input(|i| i.pointer.primary_clicked()) {
                        self.is_playing = !self.is_playing;
                        if let Some(ref mut play_ref) = self.is_playing_ref {
                            **play_ref = self.is_playing;
                        }
                        response.mark_changed();
                    }

                    if self.is_playing {
                        // Pause icon: two vertical bars
                        let bar_w = 2.0;
                        let bar_h = knob_sz.x;
                        let bar1 = Rect::from_center_size(Pos2::new(play_center.x - 2.0, play_center.y), Vec2::new(bar_w, bar_h));
                        let bar2 = Rect::from_center_size(Pos2::new(play_center.x + 2.0, play_center.y), Vec2::new(bar_w, bar_h));
                        painter.rect_filled(bar1, CornerRadius::same(0), theme.label().gamma_multiply(alpha));
                        painter.rect_filled(bar2, CornerRadius::same(0), theme.label().gamma_multiply(alpha));
                    } else {
                        // Play icon: right-pointing triangle
                        let r = knob_sz.x * 0.5;
                        let p1 = Pos2::new(play_center.x - r * 0.5, play_center.y - r);
                        let p2 = Pos2::new(play_center.x - r * 0.5, play_center.y + r);
                        let p3 = Pos2::new(play_center.x + r, play_center.y);
                        painter.add(egui::Shape::convex_polygon(vec![p1, p2, p3], theme.label().gamma_multiply(alpha), Stroke::NONE));
                    }
                }
            }
            MacPlayerControlsStyle::Inline => {
                let bar_h = self.inline_bar_height();
                let bar_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x, rect.max.y - bar_h),
                    Vec2::new(rect.width(), bar_h),
                );

                painter.rect_filled(
                    bar_rect,
                    CornerRadius::same(0),
                    theme.control_background(),
                );
                painter.line_segment(
                    [bar_rect.left_top(), bar_rect.right_top()],
                    Stroke::new(1.0, theme.separator()),
                );

                // Play icon
                let play_center = bar_rect.center();
                let icon_radius = metrics.scrubber_knob_size.points() * 0.5;
                painter.circle_filled(play_center, icon_radius, theme.label());
            }
            MacPlayerControlsStyle::Minimal => {
                let center = rect.center();
                let min_radius = metrics.scrubber_knob_size.points();
                painter.circle_filled(center, min_radius, theme.control_background());
                painter.circle_stroke(center, min_radius, Stroke::new(1.0, theme.control_accent()));
            }
        }

        if response.clicked() {
            response.mark_changed();
        }

        response
    }
}
