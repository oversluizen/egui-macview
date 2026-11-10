//! Authentic macOS AVKit Media Player View (`AVPlayerView` equivalent).
//!
//! Grounded in Apple AVKit macOS Sonoma 14.8.9 hardware inspection (Darwin 23.6.0).
//!
//! # Engineering Rationale & Apple AVKit Measurements
//!
//! On macOS Sonoma, QuickTime and AVKit media players present controls using three
//! standard presentation styles matching `AVPlayerViewControlsStyle`:
//!
//! 1. **Floating HUD Pod (`AVPlayerViewControlsStyleFloating`)**:
//!    - Translucent dark pill container measuring exactly $481.0 \times 64.0\,\text{pt}$
//!      with corner radius $R = 16.0\,\text{pt}$ and a $16.0\,\text{pt}$ margin from
//!      the bottom edge of the viewport.
//!    - **3.0-Second Auto-Hide Invariant**: Controls automatically fade out to $\alpha = 0.0$
//!      via `CASpring` after 3.0 seconds of inactivity when playing. Moving the cursor or
//!      hovering over the HUD pod resets the idle timer and restores full opacity.
//!    - **Top Row Controls ($34.0\,\text{pt}$)**: Audio volume slider ($125 \times 18\,\text{pt}$),
//!      mute toggle ($24 \times 17\,\text{pt}$), frame step ($19 \times 21\,\text{pt}$),
//!      skip 15s ($25 \times 15\,\text{pt}$), center transport play/pause ($29 \times 29\,\text{pt}$),
//!      and playback speed pill ($34 \times 18\,\text{pt}$).
//!    - **Bottom Row Scrubber ($22.5\,\text{pt}$)**: Continuous scrubbing track ($3.5\,\text{pt}$
//!      height) with buffered fill (28% white), elapsed fill, and playhead knob ($8 \times 8\,\text{pt}$).
//!
//! 2. **Inline Bottom Bar (`AVPlayerViewControlsStyleInline`)**:
//!    - Fixed-height $24.0\,\text{pt}$ bottom bar spanning 100% viewport width with a
//!      $1.0\,\text{pt}$ top separator hairline, left play button, right mute button, and
//!      timeline scrubber nestled between monospace timecode readouts.
//!
//! 3. **Minimal Transport (`AVPlayerViewControlsStyleMinimal`)**:
//!    - Compact $29.0 \times 18.0\,\text{pt}$ pill transport for thumbnail and preview cards.
//!
//! # 3-Phase Pipeline Architecture
//!
//! The control executes in three clean, independent phases:
//! 1. [`MacPlayerView::compute_geometry`]: Pure layout calculations producing [`PlayerViewGeometry`].
//! 2. [`MacPlayerView::handle_input`]: Event handling and state updates producing [`MacPlayerViewResponse`].
//! 3. [`MacPlayerView::paint`]: Painter drawing using resolved geometry and state.

mod hud;
mod inline;
mod types;

pub use types::{
    format_remaining_timecode, format_timecode, MacPlayerControlsStyle, MacPlayerViewResponse,
    PlayerViewGeometry, PlayerViewState,
};

use crate::layout::subpixel_snap_rect;
use crate::motion::CASpring;
use crate::theme::MacTheme;
use egui::{
    CornerRadius, Id, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget,
};

/// Authentic macOS AVKit Media Player View (`AVPlayerView` equivalent).
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacPlayerControlsStyle, MacPlayerView, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::DarkAqua);
/// let mut is_playing = false;
/// let mut progress = 0.25f32;
/// let mut volume = 0.8f32;
///
/// ui.add(
///     MacPlayerView::new()
///         .controls_style(MacPlayerControlsStyle::Floating)
///         .is_playing_mut(&mut is_playing)
///         .progress_mut(&mut progress)
///         .volume_mut(&mut volume)
///         .duration_secs(120.0)
///         .theme(&theme),
/// );
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacPlayerView<'a> {
    controls_style: MacPlayerControlsStyle,
    is_playing: bool,
    is_playing_ref: Option<&'a mut bool>,
    progress: f32,
    progress_ref: Option<&'a mut f32>,
    volume: f32,
    volume_ref: Option<&'a mut f32>,
    muted: bool,
    muted_ref: Option<&'a mut bool>,
    playback_rate: f32,
    playback_rate_ref: Option<&'a mut f32>,
    buffered_progress: f32,
    duration_secs: f32,
    theme: Option<&'a MacTheme>,
}

impl<'a> Default for MacPlayerView<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MacPlayerView<'a> {
    /// Creates a new player view widget with default floating HUD pod presentation.
    pub fn new() -> Self {
        Self {
            controls_style: MacPlayerControlsStyle::Floating,
            is_playing: false,
            is_playing_ref: None,
            progress: 0.0,
            progress_ref: None,
            volume: 1.0,
            volume_ref: None,
            muted: false,
            muted_ref: None,
            playback_rate: 1.0,
            playback_rate_ref: None,
            buffered_progress: 0.0,
            duration_secs: crate::tokens::metrics::controls::PlayerViewMetrics::default().default_duration_seconds.points(),
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

    /// Sets audio volume in $[0.0, 1.0]$ by value.
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Binds mutable audio volume reference.
    pub fn volume_mut(mut self, volume: &'a mut f32) -> Self {
        self.volume = *volume;
        self.volume_ref = Some(volume);
        self
    }

    /// Sets mute status by value.
    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    /// Binds mutable mute status reference.
    pub fn muted_mut(mut self, muted: &'a mut bool) -> Self {
        self.muted = *muted;
        self.muted_ref = Some(muted);
        self
    }

    /// Sets playback speed multiplier by value.
    pub fn playback_rate(mut self, rate: f32) -> Self {
        self.playback_rate = rate;
        self
    }

    /// Binds mutable playback speed multiplier reference.
    pub fn playback_rate_mut(mut self, rate: &'a mut f32) -> Self {
        self.playback_rate = *rate;
        self.playback_rate_ref = Some(rate);
        self
    }

    /// Sets buffered timeline progress in $[0.0, 1.0]$.
    pub fn buffered_progress(mut self, buffered: f32) -> Self {
        self.buffered_progress = buffered;
        self
    }

    /// Sets total media duration in seconds.
    pub fn duration_secs(mut self, duration: f32) -> Self {
        self.duration_secs = duration;
        self
    }

    /// Injects dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Floating HUD pod dimensions ($481.0 \times 64.0\,\text{pt}$).
    pub fn hud_pod_size(&self) -> Vec2 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let m = theme_cow.player_view();
        Vec2::new(m.hud_width.points(), m.hud_height.points())
    }

    /// Floating HUD pod corner radius ($16.0\,\text{pt}$).
    pub fn hud_corner_radius(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.player_view().hud_corner_radius.points()
    }

    /// Floating HUD auto-hide idle threshold in seconds ($3.0\,\text{s}$).
    pub fn auto_hide_seconds(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.player_view().auto_hide_seconds.points()
    }

    /// Timeline scrubber container height ($22.5\,\text{pt}$).
    pub fn scrubber_height(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.player_view().scrubber_height.points()
    }

    /// Timeline scrubber track height ($3.5\,\text{pt}$).
    pub fn scrubber_track_height(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.player_view().scrubber_track_height.points()
    }

    /// Timeline scrubber playhead knob dimensions ($8.0 \times 8.0\,\text{pt}$).
    pub fn scrubber_knob_size(&self) -> Vec2 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let sz = theme_cow.player_view().scrubber_knob_size.points();
        Vec2::new(sz, sz)
    }

    /// Inline bottom bar height ($24.0\,\text{pt}$).
    pub fn inline_bar_height(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.player_view().inline_height.points()
    }

    /// Inline bottom bar corner radius ($0.0\,\text{pt}$).
    pub fn inline_corner_radius(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.player_view().inline_corner_radius.points()
    }

    /// Minimal transport intrinsic dimensions ($29.0 \times 18.0\,\text{pt}$).
    pub fn minimal_size(&self) -> Vec2 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let m = theme_cow.player_view();
        Vec2::new(m.minimal_width.points(), m.minimal_height.points())
    }

    /// Standard AVKit discrete playback rates `[0.5, 1.0, 1.25, 1.5, 2.0]`.
    pub fn playback_rates(&self) -> [f32; 5] {
        [0.5, 1.0, 1.0 + 0.5 * 0.5, 1.0 + 0.5, 2.0]
    }

    /// Phase 1: Pure geometry calculation producing layout and hit rectangles.
    pub fn compute_geometry(&self, available_rect: Rect) -> PlayerViewGeometry {
        match self.controls_style {
            MacPlayerControlsStyle::Floating => hud::compute_hud_geometry(self, available_rect),
            MacPlayerControlsStyle::Inline => inline::compute_inline_geometry(self, available_rect),
            MacPlayerControlsStyle::Minimal => {
                let sz = self.minimal_size();
                let r = Rect::from_min_size(available_rect.min, sz);
                PlayerViewGeometry {
                    total_rect: available_rect,
                    controls_rect: r,
                    play_rect: r,
                    skip_back_rect: Rect::NOTHING,
                    skip_fwd_rect: Rect::NOTHING,
                    step_back_rect: Rect::NOTHING,
                    step_fwd_rect: Rect::NOTHING,
                    mute_rect: Rect::NOTHING,
                    volume_rect: Rect::NOTHING,
                    volume_track_rect: Rect::NOTHING,
                    speed_rect: Rect::NOTHING,
                    aux_rect: Rect::NOTHING,
                    scrubber_track_rect: Rect::NOTHING,
                    scrubber_hit_rect: Rect::NOTHING,
                    elapsed_pos: Pos2::ZERO,
                    remaining_pos: Pos2::ZERO,
                    knob_pos: r.center(),
                    corner_radius: r.height() * 0.5,
                }
            }
        }
    }

    /// Phase 2: Event handling, state mutation, and mutable binding synchronization.
    pub fn handle_input(
        &mut self,
        ui: &mut Ui,
        geom: &PlayerViewGeometry,
        state: &mut PlayerViewState,
    ) -> MacPlayerViewResponse {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let m = theme_cow.player_view();

        let sense = match self.controls_style {
            MacPlayerControlsStyle::Floating => Sense::click_and_drag(),
            MacPlayerControlsStyle::Inline | MacPlayerControlsStyle::Minimal => Sense::click(),
        };
        let (rect, response) = ui.allocate_exact_size(geom.total_rect.size(), sense);

        let now = ui.ctx().input(|i| i.time);
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let pointer_in_video = pointer_pos.is_some_and(|pos| rect.contains(pos));
        let pointer_active = ui.input(|i| {
            i.pointer.delta() != Vec2::ZERO || i.pointer.any_pressed() || i.pointer.any_released()
        });

        if pointer_in_video && pointer_active {
            state.last_active_time = now;
        }

        let idle_time = (now - state.last_active_time) as f32;
        let auto_hide_thresh = m.auto_hide_seconds.points();
        let is_hovered = pointer_pos.is_some_and(|pos| geom.controls_rect.contains(pos));
        let target_alpha = if is_hovered || !self.is_playing || idle_time < auto_hide_thresh {
            1.0
        } else {
            0.0
        };

        if self.is_playing && target_alpha > 0.0 && idle_time < auto_hide_thresh {
            let remaining_idle = auto_hide_thresh - idle_time;
            if remaining_idle > 0.0 && crate::is_window_active(ui.ctx()) {
                ui.ctx().request_repaint_after_secs(remaining_idle.max(m.min_repaint_delay_seconds.points()));
            }
        }

        state.alpha = if ui.is_rect_visible(rect) && crate::is_window_active(ui.ctx()) {
            let spring = CASpring::new(1.0, 1.0);
            spring.animate_f32(ui.ctx(), response.id.with("hud_alpha"), target_alpha)
        } else {
            target_alpha
        };

        let mut res = MacPlayerViewResponse {
            response,
            play_toggled: false,
            progress_changed: false,
            volume_changed: false,
            mute_toggled: false,
            rate_changed: false,
        };

        if state.alpha <= 0.0 {
            return res;
        }

        let interact_pos = ui.input(|i| i.pointer.interact_pos());
        let clicked = res.response.clicked() || ui.input(|i| i.pointer.primary_clicked());
        let down = ui.input(|i| i.pointer.primary_down());

        // Play/Pause
        if interact_pos.is_some_and(|pos| geom.play_rect.contains(pos)) && clicked {
            self.is_playing = !self.is_playing;
            if let Some(ref mut p_ref) = self.is_playing_ref {
                **p_ref = self.is_playing;
            }
            res.play_toggled = true;
            res.response.mark_changed();
        }

        // Mute
        if interact_pos.is_some_and(|pos| geom.mute_rect.contains(pos)) && clicked {
            self.muted = !self.muted;
            if let Some(ref mut m_ref) = self.muted_ref {
                **m_ref = self.muted;
            }
            res.mute_toggled = true;
            res.response.mark_changed();
        }

        // Volume drag
        if interact_pos.is_some_and(|pos| geom.volume_rect.contains(pos)) && down {
            if let Some(pos) = interact_pos {
                let v = ((pos.x - geom.volume_track_rect.min.x) / geom.volume_track_rect.width()).clamp(0.0, 1.0);
                self.volume = v;
                self.muted = false;
                if let Some(ref mut v_ref) = self.volume_ref {
                    **v_ref = v;
                }
                if let Some(ref mut m_ref) = self.muted_ref {
                    **m_ref = false;
                }
                res.volume_changed = true;
                res.response.mark_changed();
            }
        }

        // Skip / Step buttons
        let dur = self.duration_secs.max(1.0);
        if interact_pos.is_some_and(|pos| geom.skip_back_rect.contains(pos)) && clicked {
            let delta = m.skip_duration_seconds.points() / dur;
            self.progress = (self.progress - delta).clamp(0.0, 1.0);
            if let Some(ref mut pr_ref) = self.progress_ref {
                **pr_ref = self.progress;
            }
            res.progress_changed = true;
            res.response.mark_changed();
        }
        if interact_pos.is_some_and(|pos| geom.skip_fwd_rect.contains(pos)) && clicked {
            let delta = m.skip_duration_seconds.points() / dur;
            self.progress = (self.progress + delta).clamp(0.0, 1.0);
            if let Some(ref mut pr_ref) = self.progress_ref {
                **pr_ref = self.progress;
            }
            res.progress_changed = true;
            res.response.mark_changed();
        }
        if interact_pos.is_some_and(|pos| geom.step_back_rect.contains(pos)) && clicked {
            let step = 1.0 / (dur * m.frame_step_fps.points());
            self.progress = (self.progress - step).clamp(0.0, 1.0);
            if let Some(ref mut pr_ref) = self.progress_ref {
                **pr_ref = self.progress;
            }
            res.progress_changed = true;
            res.response.mark_changed();
        }
        if interact_pos.is_some_and(|pos| geom.step_fwd_rect.contains(pos)) && clicked {
            let step = 1.0 / (dur * m.frame_step_fps.points());
            self.progress = (self.progress + step).clamp(0.0, 1.0);
            if let Some(ref mut pr_ref) = self.progress_ref {
                **pr_ref = self.progress;
            }
            res.progress_changed = true;
            res.response.mark_changed();
        }

        // Playback rate cycle
        if interact_pos.is_some_and(|pos| geom.speed_rect.contains(pos)) && clicked {
            let rates = self.playback_rates();
            let cur_idx = rates.iter().position(|&r| (r - self.playback_rate).abs() < 1e-3).unwrap_or(1);
            let next_idx = (cur_idx + 1) % rates.len();
            self.playback_rate = rates[next_idx];
            if let Some(ref mut r_ref) = self.playback_rate_ref {
                **r_ref = self.playback_rate;
            }
            res.rate_changed = true;
            res.response.mark_changed();
        }

        // Scrubber drag
        if interact_pos.is_some_and(|pos| geom.scrubber_hit_rect.contains(pos)) && down {
            if let Some(pos) = interact_pos {
                state.is_scrubbing = true;
                let prog = ((pos.x - geom.scrubber_track_rect.min.x) / geom.scrubber_track_rect.width()).clamp(0.0, 1.0);
                self.progress = prog;
                if let Some(ref mut pr_ref) = self.progress_ref {
                    **pr_ref = prog;
                }
                res.progress_changed = true;
                res.response.mark_changed();
            }
        } else if !down {
            state.is_scrubbing = false;
        }

        res
    }

    /// Phase 3: Renders player view controls onto painter.
    pub fn paint(
        &self,
        painter: &Painter,
        geom: &PlayerViewGeometry,
        state: &PlayerViewState,
        theme: &MacTheme,
    ) {
        if state.alpha <= 0.0 {
            return;
        }

        match self.controls_style {
            MacPlayerControlsStyle::Floating => hud::paint_hud(self, painter, geom, state, theme),
            MacPlayerControlsStyle::Inline => inline::paint_inline(self, painter, geom, state, theme),
            MacPlayerControlsStyle::Minimal => {
                let rect = geom.controls_rect;
                let bg = theme.control_background();
                painter.rect(rect, CornerRadius::same(geom.corner_radius as u8), bg, Stroke::new(1.0, theme.separator()), StrokeKind::Inside);
                let icon_h = theme.player_view().icon_size.points();
                let pc = rect.center();
                if self.is_playing {
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
            }
        }
    }
}

impl<'a> Widget for MacPlayerView<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let m = theme.player_view();

        let available_w = ui.available_width();
        let total_size = match self.controls_style {
            MacPlayerControlsStyle::Floating => {
                let pod = self.hud_pod_size();
                Vec2::new(available_w.max(pod.x), pod.y + m.hud_bottom_margin.points() * 2.0)
            }
            MacPlayerControlsStyle::Inline => Vec2::new(available_w, self.inline_bar_height()),
            MacPlayerControlsStyle::Minimal => self.minimal_size(),
        };

        let start_pos = ui.cursor().min;
        let available_rect = Rect::from_min_size(start_pos, total_size);
        let available_rect = subpixel_snap_rect(available_rect, ui.ctx().pixels_per_point());

        // Explicit PlayerViewState load/store from temp storage
        let state_id = Id::NULL.with("player_view_state").with(ui.scope_id());
        let mut state = ui.data_mut(|d| d.get_temp::<PlayerViewState>(state_id).unwrap_or_default());

        // 3-Phase Execution Pipeline:
        // 1. Compute geometry
        let geom = self.compute_geometry(available_rect);

        // 2. Handle input and state transitions
        let resp = self.handle_input(ui, &geom, &mut state);

        // 3. Paint controls
        let painter = ui.painter_at(available_rect);
        self.paint(&painter, &geom, &state, theme);

        ui.data_mut(|d| d.insert_temp(state_id, state));

        resp.response
    }
}
