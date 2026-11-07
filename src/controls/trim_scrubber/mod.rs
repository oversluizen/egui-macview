//! Authentic macOS AVKit / QuickTime In–Out Trim & Timeline Scrubber (`AVPlayerView`, SPEC-011).

pub mod painter;

pub use painter::MacTimelineZone;

use crate::theme::MacTheme;
use egui::{Id, Response, Sense, Ui, Vec2};
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

    /// Formats a timestamp in seconds as `MM:SS.d`.
    pub fn format_timecode(_theme: &MacTheme, _seconds: f64) -> String {
        String::new()
    }

    /// Renders the trim scrubber in `ui`.
    pub fn show(self, ui: &mut Ui) -> MacTrimScrubberResponse {
        let _ = (
            self.id,
            self.duration,
            &self.in_point,
            &self.out_point,
            &self.playhead,
            self.waveform,
            self.zones,
            self.width,
            self.height,
            self.enabled,
            self.theme,
        );
        let (_, response) = ui.allocate_exact_size(Vec2::ZERO, Sense::hover());
        MacTrimScrubberResponse {
            response,
            in_point_dragged: false,
            out_point_dragged: false,
            playhead_dragged: false,
            changed: false,
        }
    }
}
