//! Waveform, timeline zone, and QuickTime trim handle painter for [`super::MacTrimScrubber`].

use egui::Color32;

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
