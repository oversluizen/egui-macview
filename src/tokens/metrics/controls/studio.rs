//! Studio, Timeline, and Publishing AppKit/AVKit control geometry metrics tokens (SPEC-011).

use crate::tokens::dtcg::DimensionValue;
use crate::tokens::metrics::controls::basic::default_border_width;
use serde::{Deserialize, Serialize};

/// Per-size geometry metrics for `NSProgressIndicator` (`Regular`, `Small`, `Mini`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressSizeMetrics {
    /// Height of horizontal progress bar track.
    pub bar_height: DimensionValue,
    /// Corner radius of horizontal progress bar track and fill.
    pub bar_corner_radius: DimensionValue,
    /// Diameter of radial fin spinner.
    pub spinner_size: DimensionValue,
    /// Stroke thickness of each radial fin spoke.
    pub spoke_width: DimensionValue,
    /// Typography font size for optional status/percentage label.
    pub font_size: DimensionValue,
}

impl ProgressSizeMetrics {
    fn new(bar_h: f32, bar_r: f32, spin_sz: f32, spoke_w: f32, font_sz: f32) -> Self {
        Self {
            bar_height: DimensionValue::from_pt(bar_h),
            bar_corner_radius: DimensionValue::from_pt(bar_r),
            spinner_size: DimensionValue::from_pt(spin_sz),
            spoke_width: DimensionValue::from_pt(spoke_w),
            font_size: DimensionValue::from_pt(font_sz),
        }
    }
}

/// `NSProgressIndicator` metrics across `Regular`, `Small`, and `Mini` control sizes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressIndicatorMetrics {
    /// Regular control size metrics.
    pub regular: ProgressSizeMetrics,
    /// Small control size metrics.
    pub small: ProgressSizeMetrics,
    /// Mini control size metrics.
    pub mini: ProgressSizeMetrics,
    /// Default horizontal bar width when not explicitly overridden.
    pub default_bar_width: DimensionValue,
    /// Number of radial spokes in spinning indicator (12).
    pub spoke_count: DimensionValue,
    /// Inner hub radius ratio relative to outer spinner radius.
    pub spoke_inner_ratio: DimensionValue,
    /// Indeterminate sliding pill width ratio relative to track width.
    pub indeterminate_pill_ratio: DimensionValue,
    /// Rotational frequency in Hz for spinning mode.
    pub spin_speed_hz: DimensionValue,
    /// Oscillation frequency in Hz for indeterminate bar pulse.
    pub bar_indeterminate_speed: DimensionValue,
    /// Spacing between indicator and optional label.
    pub label_spacing: DimensionValue,
    /// Hairline track border stroke width.
    #[serde(default = "default_border_width")]
    pub border_width: DimensionValue,
    /// Full circle angle $2\pi$ in radians.
    pub tau_radians: DimensionValue,
    /// Minimum trailing spoke alpha ratio in spinning mode.
    pub min_spoke_alpha: DimensionValue,
    /// Percentage conversion multiplier ($100.0$).
    pub percentage_scale: DimensionValue,
}

impl Default for ProgressIndicatorMetrics {
    fn default() -> Self {
        Self {
            regular: ProgressSizeMetrics::new(6.0, 3.0, 32.0, 2.5, 12.0),
            small: ProgressSizeMetrics::new(4.0, 2.0, 16.0, 1.6, 11.0),
            mini: ProgressSizeMetrics::new(3.0, 1.5, 12.0, 1.2, 9.0),
            default_bar_width: DimensionValue::from_pt(160.0),
            spoke_count: DimensionValue::from_pt(12.0),
            spoke_inner_ratio: DimensionValue::from_pt(0.52),
            indeterminate_pill_ratio: DimensionValue::from_pt(0.35),
            spin_speed_hz: DimensionValue::from_pt(1.0),
            bar_indeterminate_speed: DimensionValue::from_pt(1.2),
            label_spacing: DimensionValue::from_pt(6.0),
            border_width: default_border_width(),
            tau_radians: DimensionValue::from_pt(6.283_185_5),
            min_spoke_alpha: DimensionValue::from_pt(0.18),
            percentage_scale: DimensionValue::from_pt(100.0),
        }
    }
}

/// `NSTableView` and `NSTableHeaderView` geometry metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableViewMetrics {
    /// Height of the `NSTableHeaderView` bar.
    pub header_height: DimensionValue,
    /// Default height of each data row.
    pub row_height: DimensionValue,
    /// Horizontal cell padding inside headers and rows.
    pub cell_padding_horizontal: DimensionValue,
    /// Vertical cell padding inside data rows.
    pub cell_padding_vertical: DimensionValue,
    /// Typography font size for header column titles.
    pub header_font_size: DimensionValue,
    /// Typography font size for standard cell labels.
    pub cell_font_size: DimensionValue,
    /// Icon size for ascending/descending sort chevrons.
    pub sort_chevron_size: DimensionValue,
    /// Spacing between header title and sort chevron.
    pub sort_chevron_spacing: DimensionValue,
    /// Hairline stroke width for grid and header separators.
    pub separator_width: DimensionValue,
    /// Vertical top/bottom inset for column dividers in the header bar.
    pub header_divider_inset: DimensionValue,
    /// Outer container squircle corner radius.
    pub corner_radius: DimensionValue,
    /// Corner radius for selected row highlight.
    pub row_corner_radius: DimensionValue,
}

impl Default for TableViewMetrics {
    fn default() -> Self {
        Self {
            header_height: DimensionValue::from_pt(28.0),
            row_height: DimensionValue::from_pt(24.0),
            cell_padding_horizontal: DimensionValue::from_pt(8.0),
            cell_padding_vertical: DimensionValue::from_pt(2.0),
            header_font_size: DimensionValue::from_pt(11.0),
            cell_font_size: DimensionValue::from_pt(12.0),
            sort_chevron_size: DimensionValue::from_pt(9.0),
            sort_chevron_spacing: DimensionValue::from_pt(4.0),
            separator_width: DimensionValue::from_pt(1.0),
            header_divider_inset: DimensionValue::from_pt(4.0),
            corner_radius: DimensionValue::from_pt(6.0),
            row_corner_radius: DimensionValue::from_pt(4.0),
        }
    }
}

/// `NSTokenField` pill and flow-wrapping geometry metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenFieldMetrics {
    /// Minimum single-row bezel height.
    pub min_height: DimensionValue,
    /// Outer bezel corner radius.
    pub corner_radius: DimensionValue,
    /// Outer bezel border stroke width.
    pub border_width: DimensionValue,
    /// Horizontal inset from bezel to token pills.
    pub padding_horizontal: DimensionValue,
    /// Vertical inset from bezel to token pills.
    pub padding_vertical: DimensionValue,
    /// Height of each rounded token pill.
    pub pill_height: DimensionValue,
    /// Capsule corner radius of each token pill.
    pub pill_corner_radius: DimensionValue,
    /// Horizontal text padding inside each token pill.
    pub pill_padding_horizontal: DimensionValue,
    /// Horizontal gap between adjacent token pills.
    pub pill_spacing: DimensionValue,
    /// Vertical gap between wrapped token rows.
    pub row_spacing: DimensionValue,
    /// Font size for pill text and inline input.
    pub font_size: DimensionValue,
    /// Font size for optional bottom-right character counter.
    pub counter_font_size: DimensionValue,
    /// Hit target size of trailing `×` remove button on a pill.
    pub remove_icon_size: DimensionValue,
    /// Stroke width of vector cross mark inside remove button.
    pub remove_icon_stroke: DimensionValue,
    /// Half-span of vector cross mark inside remove button.
    pub remove_cross_span: DimensionValue,
    /// Minimum width reserved for trailing inline text input.
    pub min_input_width: DimensionValue,
    /// Default control width when not explicitly overridden.
    pub default_width: DimensionValue,
    /// Additional bottom row height when character counter is shown.
    pub counter_row_height: DimensionValue,
}

impl Default for TokenFieldMetrics {
    fn default() -> Self {
        Self {
            min_height: DimensionValue::from_pt(26.0),
            corner_radius: DimensionValue::from_pt(6.0),
            border_width: DimensionValue::from_pt(1.0),
            padding_horizontal: DimensionValue::from_pt(6.0),
            padding_vertical: DimensionValue::from_pt(4.0),
            pill_height: DimensionValue::from_pt(18.0),
            pill_corner_radius: DimensionValue::from_pt(9.0),
            pill_padding_horizontal: DimensionValue::from_pt(7.0),
            pill_spacing: DimensionValue::from_pt(4.0),
            row_spacing: DimensionValue::from_pt(4.0),
            font_size: DimensionValue::from_pt(12.0),
            counter_font_size: DimensionValue::from_pt(10.0),
            remove_icon_size: DimensionValue::from_pt(12.0),
            remove_icon_stroke: DimensionValue::from_pt(1.2),
            remove_cross_span: DimensionValue::from_pt(2.5),
            min_input_width: DimensionValue::from_pt(64.0),
            default_width: DimensionValue::from_pt(240.0),
            counter_row_height: DimensionValue::from_pt(14.0),
        }
    }
}

/// `NSLevelIndicator` continuous and discrete meter geometry metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelIndicatorMetrics {
    /// Default primary-axis length of the meter.
    pub default_length: DimensionValue,
    /// Default cross-axis thickness of the meter.
    pub thickness: DimensionValue,
    /// Outer bezel corner radius.
    pub corner_radius: DimensionValue,
    /// Corner radius of individual discrete segments.
    pub segment_corner_radius: DimensionValue,
    /// Gap between adjacent discrete segments.
    pub segment_spacing: DimensionValue,
    /// Default number of discrete segments (12).
    pub default_segment_count: DimensionValue,
    /// Perimeter border stroke width.
    pub border_width: DimensionValue,
    /// Stroke width of peak-hold marker line.
    pub peak_line_width: DimensionValue,
    /// Inner inset from outer bezel to active bar/segments.
    pub track_inset: DimensionValue,
}

impl Default for LevelIndicatorMetrics {
    fn default() -> Self {
        Self {
            default_length: DimensionValue::from_pt(160.0),
            thickness: DimensionValue::from_pt(12.0),
            corner_radius: DimensionValue::from_pt(3.0),
            segment_corner_radius: DimensionValue::from_pt(1.5),
            segment_spacing: DimensionValue::from_pt(2.0),
            default_segment_count: DimensionValue::from_pt(12.0),
            border_width: DimensionValue::from_pt(1.0),
            peak_line_width: DimensionValue::from_pt(2.0),
            track_inset: DimensionValue::from_pt(1.5),
        }
    }
}

/// `AVPlayerView` / QuickTime In–Out trim scrubber geometry metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrimScrubberMetrics {
    /// Total height of the trim scrubber widget.
    pub height: DimensionValue,
    /// Default width when not explicitly overridden.
    pub default_width: DimensionValue,
    /// Top and bottom vertical margin around the waveform track.
    pub track_inset_vertical: DimensionValue,
    /// Corner radius of the track and trim selection frame.
    pub corner_radius: DimensionValue,
    /// Horizontal width of left (In) and right (Out) grab handles.
    pub handle_width: DimensionValue,
    /// Stroke thickness of vertical grip slot inside trim handles.
    pub handle_slot_width: DimensionValue,
    /// Vertical length of grip slot inside trim handles.
    pub handle_slot_height: DimensionValue,
    /// Top and bottom border stroke thickness of active trim selection frame.
    pub selection_border_width: DimensionValue,
    /// Stroke thickness of vertical playhead line.
    pub playhead_width: DimensionValue,
    /// Interactive pointer hit width around the playhead.
    pub playhead_hit_width: DimensionValue,
    /// Height of floating `MM:SS.d` timecode badge.
    pub badge_height: DimensionValue,
    /// Horizontal padding inside timecode badge.
    pub badge_padding_horizontal: DimensionValue,
    /// Corner radius of timecode badge.
    pub badge_corner_radius: DimensionValue,
    /// Font size of timecode readout text.
    pub badge_font_size: DimensionValue,
    /// Font size of named timeline zone labels.
    pub zone_font_size: DimensionValue,
    /// Inset padding for timeline zone labels.
    pub zone_label_padding: DimensionValue,
    /// Stroke width of individual waveform amplitude bars.
    pub waveform_bar_width: DimensionValue,
    /// Minimum amplitude ratio for silent waveform samples.
    pub waveform_min_amplitude: DimensionValue,
    /// Seconds per minute conversion constant ($60.0$).
    pub seconds_per_minute: DimensionValue,
    /// Deciseconds per second multiplier ($10.0$).
    pub tenths_per_second: DimensionValue,
}

impl Default for TrimScrubberMetrics {
    fn default() -> Self {
        Self {
            height: DimensionValue::from_pt(44.0),
            default_width: DimensionValue::from_pt(320.0),
            track_inset_vertical: DimensionValue::from_pt(8.0),
            corner_radius: DimensionValue::from_pt(6.0),
            handle_width: DimensionValue::from_pt(10.0),
            handle_slot_width: DimensionValue::from_pt(1.5),
            handle_slot_height: DimensionValue::from_pt(10.0),
            selection_border_width: DimensionValue::from_pt(2.0),
            playhead_width: DimensionValue::from_pt(2.0),
            playhead_hit_width: DimensionValue::from_pt(12.0),
            badge_height: DimensionValue::from_pt(14.0),
            badge_padding_horizontal: DimensionValue::from_pt(5.0),
            badge_corner_radius: DimensionValue::from_pt(4.0),
            badge_font_size: DimensionValue::from_pt(9.0),
            zone_font_size: DimensionValue::from_pt(9.0),
            zone_label_padding: DimensionValue::from_pt(4.0),
            waveform_bar_width: DimensionValue::from_pt(2.0),
            waveform_min_amplitude: DimensionValue::from_pt(0.08),
            seconds_per_minute: DimensionValue::from_pt(60.0),
            tenths_per_second: DimensionValue::from_pt(10.0),
        }
    }
}

/// `NSPathControl` breadcrumb bar geometry metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathControlMetrics {
    /// Intrinsic height of the path control bar.
    pub height: DimensionValue,
    /// Outer bezel corner radius.
    pub corner_radius: DimensionValue,
    /// Corner radius for hovered/clicked segment highlight.
    pub segment_corner_radius: DimensionValue,
    /// Leading and trailing horizontal padding inside path bar.
    pub padding_horizontal: DimensionValue,
    /// Horizontal padding inside each clickable path segment.
    pub segment_padding_horizontal: DimensionValue,
    /// Spacing around chevron separators between segments.
    pub segment_spacing: DimensionValue,
    /// Bounding size of leading folder/file icon per segment.
    pub icon_size: DimensionValue,
    /// Gap between segment icon and segment label.
    pub icon_spacing: DimensionValue,
    /// Font size of `ChevronRight` separator symbol.
    pub chevron_size: DimensionValue,
    /// Font size of segment title labels.
    pub font_size: DimensionValue,
    /// Hairline border stroke width.
    pub border_width: DimensionValue,
    /// Horizontal label padding inside trailing `Choose...` button.
    pub choose_button_padding: DimensionValue,
    /// Height of trailing `Choose...` button.
    pub choose_button_height: DimensionValue,
    /// Width of top-left tab on vector folder icon.
    pub folder_tab_width: DimensionValue,
    /// Height of top-left tab on vector folder icon.
    pub folder_tab_height: DimensionValue,
}

impl Default for PathControlMetrics {
    fn default() -> Self {
        Self {
            height: DimensionValue::from_pt(22.0),
            corner_radius: DimensionValue::from_pt(5.0),
            segment_corner_radius: DimensionValue::from_pt(4.0),
            padding_horizontal: DimensionValue::from_pt(6.0),
            segment_padding_horizontal: DimensionValue::from_pt(5.0),
            segment_spacing: DimensionValue::from_pt(2.0),
            icon_size: DimensionValue::from_pt(12.0),
            icon_spacing: DimensionValue::from_pt(4.0),
            chevron_size: DimensionValue::from_pt(9.0),
            font_size: DimensionValue::from_pt(11.0),
            border_width: DimensionValue::from_pt(1.0),
            choose_button_padding: DimensionValue::from_pt(8.0),
            choose_button_height: DimensionValue::from_pt(18.0),
            folder_tab_width: DimensionValue::from_pt(5.0),
            folder_tab_height: DimensionValue::from_pt(2.0),
        }
    }
}
