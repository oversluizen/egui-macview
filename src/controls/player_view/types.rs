//! Core types, geometry representations, and state models for `MacPlayerView`.
//!
//! Grounded in Apple AVKit macOS Sonoma 14.8.9 hardware inspection (Darwin 23.6.0).

use egui::{Pos2, Rect, Response};

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

/// Explicit interaction and animation state for [`crate::controls::MacPlayerView`].
///
/// Replaces scattered egui context queries with a consolidated, testable state container.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerViewState {
    /// Timestamp (in seconds) of the last user interaction or cursor movement within the player view.
    pub last_active_time: f64,
    /// Whether the user is actively dragging the timeline scrubber playhead knob.
    pub is_scrubbing: bool,
    /// Current animated opacity alpha of the floating HUD controls ($0.0 \to 1.0$).
    pub alpha: f32,
}

impl Default for PlayerViewState {
    fn default() -> Self {
        Self {
            last_active_time: 0.0,
            is_scrubbing: false,
            alpha: 1.0,
        }
    }
}

/// Resolved layout geometry for [`crate::controls::MacPlayerView`] controls.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerViewGeometry {
    /// Total allocated container rectangle for the player view.
    pub total_rect: Rect,
    /// Bounding rectangle for the controls (floating HUD pod, inline bottom bar, or minimal pill).
    pub controls_rect: Rect,
    /// Center transport play/pause hit rectangle.
    pub play_rect: Rect,
    /// Skip 15s backward hit rectangle.
    pub skip_back_rect: Rect,
    /// Skip 15s forward hit rectangle.
    pub skip_fwd_rect: Rect,
    /// Frame step backward hit rectangle.
    pub step_back_rect: Rect,
    /// Frame step forward hit rectangle.
    pub step_fwd_rect: Rect,
    /// Audio mute button hit rectangle.
    pub mute_rect: Rect,
    /// Audio volume slider container rectangle.
    pub volume_rect: Rect,
    /// Audio volume slider track rectangle.
    pub volume_track_rect: Rect,
    /// Playback speed selector hit rectangle.
    pub speed_rect: Rect,
    /// Auxiliary AirPlay / PiP button hit rectangle.
    pub aux_rect: Rect,
    /// Timeline scrubber track rectangle.
    pub scrubber_track_rect: Rect,
    /// Timeline scrubber interactive hit area.
    pub scrubber_hit_rect: Rect,
    /// Elapsed timecode label origin position.
    pub elapsed_pos: Pos2,
    /// Remaining timecode label origin position.
    pub remaining_pos: Pos2,
    /// Playhead knob center position.
    pub knob_pos: Pos2,
    /// Corner radius of the controls container.
    pub corner_radius: f32,
}

/// Detailed response returned from [`crate::controls::MacPlayerView::handle_input`].
pub struct MacPlayerViewResponse {
    /// Standard egui response.
    pub response: Response,
    /// Whether playback state was toggled.
    pub play_toggled: bool,
    /// Whether progress was modified by scrubbing or skipping.
    pub progress_changed: bool,
    /// Whether audio volume was adjusted.
    pub volume_changed: bool,
    /// Whether mute state was toggled.
    pub mute_toggled: bool,
    /// Whether playback speed was cycled.
    pub rate_changed: bool,
}

/// Formats an elapsed duration in seconds into an authentic AVKit timecode string.
///
/// Formats as `HH:MM:SS` when `total_dur_secs as u32 >= 3600` (or `secs >= 3600`);
/// otherwise formats as standard `MM:SS`.
pub fn format_timecode(secs: u32, total_dur_secs: f32) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    if total_dur_secs as u32 >= 3600 || hours > 0 {
        format!("{hours:02}:{mins:02}:{s:02}")
    } else {
        format!("{mins:02}:{s:02}")
    }
}

/// Formats a remaining duration in seconds into a negative AVKit timecode string (`-MM:SS` or `-HH:MM:SS`).
pub fn format_remaining_timecode(rem_secs: u32, total_dur_secs: f32) -> String {
    format!("-{}", format_timecode(rem_secs, total_dur_secs))
}

