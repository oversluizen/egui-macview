//! Authentic macOS AVKit / QuickTime In–Out Trim & Timeline Scrubber (`AVPlayerView`, SPEC-011).
//!
//! Grounded in Apple AVKit `AVPlayerView` / QuickTime Player and W3C DTCG tokens via [`MacTheme`].
//! Renders normalized waveform profiles, named timeline zones, draggable In/Out yellow trim handles
//! with dimmed outer regions, and a draggable playhead with `MM:SS.d` timecode badge.

pub mod painter;

pub use painter::MacTimelineZone;

use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{Id, Pos2, Rect, Response, Sense, Ui, Vec2};
use std::fmt::Debug;
use std::hash::Hash;

/// Interaction output returned by [`MacTrimScrubber::show`].
#[derive(Debug, Clone)]
pub struct MacTrimScrubberResponse {
    /// Underlying widget interaction response.
    pub response: Response,
    /// True if the left In-point trim handle was dragged during this frame.
    pub in_point_dragged: bool,
    /// True if the right Out-point trim handle was dragged during this frame.
    pub out_point_dragged: bool,
    /// True if the playhead was dragged or clicked during this frame.
    pub playhead_dragged: bool,
    /// True if any of `in_point`, `out_point`, or `playhead` changed during this frame.
    pub changed: bool,
}

/// Authentic macOS QuickTime Player / AVKit waveform and In–Out trim scrubber.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacTheme, MacTimelineZone, MacTrimScrubber};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut in_pt = 2.0;
/// let mut out_pt = 18.0;
/// let mut playhead = 6.5;
/// let waveform = [0.2f32, 0.6, 0.9, 0.5, 0.8, 0.3];
/// let zones = [MacTimelineZone::new(0.0, 10.0, "Intro")];
///
/// MacTrimScrubber::new("clip_trim", 20.0, &mut in_pt, &mut out_pt, &mut playhead)
///     .theme(&theme)
///     .waveform(&waveform)
///     .zones(&zones)
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacTrimScrubber<'a> {
    id: Id,
    duration: f64,
    in_point: &'a mut f64,
    out_point: &'a mut f64,
    playhead: &'a mut f64,
    waveform: &'a [f32],
    zones: &'a [MacTimelineZone<'a>],
    width: Option<f32>,
    height: Option<f32>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacTrimScrubber<'a> {
    /// Creates a new trim scrubber over `0.0..=duration` seconds.
    pub fn new(
        id_source: impl Hash + Debug,
        duration: f64,
        in_point: &'a mut f64,
        out_point: &'a mut f64,
        playhead: &'a mut f64,
    ) -> Self {
        #[allow(deprecated)]
        let id = Id::new(id_source);
        Self {
            id,
            duration,
            in_point,
            out_point,
            playhead,
            waveform: &[],
            zones: &[],
            width: None,
            height: None,
            enabled: true,
            theme: None,
        }
    }

    /// Attaches normalized waveform amplitude samples (`0.0..=1.0`).
    pub fn waveform(mut self, waveform: &'a [f32]) -> Self {
        self.waveform = waveform;
        self
    }

    /// Attaches named background timeline zones.
    pub fn zones(mut self, zones: &'a [MacTimelineZone<'a>]) -> Self {
        self.zones = zones;
        self
    }

    /// Overrides scrubber width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides scrubber height in points.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Enables or disables interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Formats a timestamp in seconds as `MM:SS.d` using token conversion constants.
    pub fn format_timecode(theme: &MacTheme, seconds: f64) -> String {
        let sm = theme.trim_scrubber();
        let sec_per_min = sm.seconds_per_minute.points() as f64;
        let tenths_mul = sm.tenths_per_second.points() as f64;
        let clamped = seconds.max(0.0);
        let total_tenths = (clamped * tenths_mul).round() as u64;
        let tenths_u64 = (tenths_mul.round() as u64).max(1);
        let sec_min_u64 = (sec_per_min.round() as u64).max(1);
        let total_secs = total_tenths / tenths_u64;
        let tenths = total_tenths % tenths_u64;
        let mins = total_secs / sec_min_u64;
        let secs = total_secs % sec_min_u64;
        format!("{mins:02}:{secs:02}.{tenths}")
    }

    /// Renders the trim scrubber in `ui`.
    pub fn show(self, ui: &mut Ui) -> MacTrimScrubberResponse {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let sm = theme.trim_scrubber();
        let width = self.width.unwrap_or_else(|| sm.default_width.points());
        let height = self.height.unwrap_or_else(|| sm.height.points());
        let dur = self.duration.max(f64::EPSILON);

        let mut changed = false;

        // Enforce ordering invariants: 0.0 <= in_point <= out_point <= duration, in_point <= playhead <= out_point
        let orig_in = *self.in_point;
        let orig_out = *self.out_point;
        let orig_play = *self.playhead;

        let clamped_in = orig_in.clamp(0.0, dur);
        let clamped_out = orig_out.clamp(clamped_in, dur);
        let clamped_play = orig_play.clamp(clamped_in, clamped_out);

        if (clamped_in - orig_in).abs() > f64::EPSILON
            || (clamped_out - orig_out).abs() > f64::EPSILON
            || (clamped_play - orig_play).abs() > f64::EPSILON
        {
            *self.in_point = clamped_in;
            *self.out_point = clamped_out;
            *self.playhead = clamped_play;
            changed = true;
        }

        let (raw_rect, mut response) =
            ui.allocate_exact_size(Vec2::new(width, height), Sense::click_and_drag());
        let rect = subpixel_snap_rect(raw_rect, ui.ctx().pixels_per_point());

        let v_inset = sm.track_inset_vertical.points();
        let badge_h = sm.badge_height.points();
        let track_rect = Rect::from_min_max(
            Pos2::new(rect.left(), rect.top() + badge_h),
            Pos2::new(
                rect.right(),
                (rect.bottom() - v_inset * 0.5).max(rect.top() + badge_h),
            ),
        );

        let handle_w = sm.handle_width.points();
        let track_w = track_rect.width().max(1.0);
        let x_to_time = |x: f32| -> f64 {
            let frac = ((x - track_rect.left()) / track_w).clamp(0.0, 1.0) as f64;
            frac * dur
        };
        let time_to_x = |t: f64| -> f32 {
            let frac = (t / dur).clamp(0.0, 1.0) as f32;
            track_rect.left() + frac * track_w
        };

        let in_x = time_to_x(*self.in_point);
        let out_x = time_to_x(*self.out_point);

        let (in_rect, out_rect) = painter::compute_handle_rects(track_rect, in_x, out_x, handle_w);

        let sense = if self.enabled {
            Sense::click_and_drag()
        } else {
            Sense::hover()
        };
        let in_resp = ui.interact(in_rect, self.id.with("in_handle"), sense);
        let out_resp = ui.interact(out_rect, self.id.with("out_handle"), sense);

        let mut in_point_dragged = false;
        let mut out_point_dragged = false;
        let mut playhead_dragged = false;

        if self.enabled {
            let drag_id = self.id.with("active_drag_target");
            let (any_pressed, primary_down, any_released, ptr_pos) = ui.input(|i| {
                (
                    i.pointer.any_pressed(),
                    i.pointer.primary_down(),
                    i.pointer.any_released(),
                    i.pointer.interact_pos().or_else(|| i.pointer.latest_pos()),
                )
            });
            let mut target = ui
                .data_mut(|d| d.get_temp::<(bool, bool, bool)>(drag_id))
                .unwrap_or((false, false, false));

            if any_pressed {
                if let Some(pos) = ptr_pos {
                    if in_rect.contains(pos) {
                        target = (true, false, false);
                    } else if out_rect.contains(pos) {
                        target = (false, true, false);
                    } else if rect.contains(pos) {
                        target = (false, false, true);
                    } else {
                        target = (false, false, false);
                    }
                }
            }

            let active_in = target.0 || in_resp.dragged() || in_resp.drag_stopped();
            let active_out = target.1 || out_resp.dragged() || out_resp.drag_stopped();
            let active_play = target.2 || response.clicked() || response.dragged();

            if (primary_down || any_released || any_pressed)
                && (active_in || active_out || active_play)
            {
                if let Some(pos) = ptr_pos {
                    if active_in {
                        let next_in = x_to_time(pos.x).clamp(0.0, *self.out_point);
                        *self.in_point = next_in;
                        if *self.playhead < next_in {
                            *self.playhead = next_in;
                        }
                        in_point_dragged = true;
                        changed = true;
                    } else if active_out {
                        let next_out = x_to_time(pos.x).clamp(*self.in_point, dur);
                        *self.out_point = next_out;
                        if *self.playhead > next_out {
                            *self.playhead = next_out;
                        }
                        out_point_dragged = true;
                        changed = true;
                    } else if active_play {
                        let next_play = x_to_time(pos.x).clamp(*self.in_point, *self.out_point);
                        *self.playhead = next_play;
                        playhead_dragged = true;
                        changed = true;
                    }
                }
            }

            if !primary_down || any_released {
                target = (false, false, false);
            }
            ui.data_mut(|d| d.insert_temp(drag_id, target));
        }

        let tc = Self::format_timecode(theme, *self.playhead);
        let painter = ui.painter_at(rect);
        painter::paint_trim_scrubber(
            &painter,
            theme,
            rect,
            track_rect,
            dur,
            *self.in_point,
            *self.out_point,
            *self.playhead,
            self.waveform,
            self.zones,
            &tc,
        );

        if changed {
            response.mark_changed();
        }

        MacTrimScrubberResponse {
            response,
            in_point_dragged,
            out_point_dragged,
            playhead_dragged,
            changed,
        }
    }
}
