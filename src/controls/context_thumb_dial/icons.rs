//! Official Apple SF Symbol Vector Iconography (`MacSFSymbol`, `MacThumbSectorIcon`) &
//! Taptic/Auditory Micro-Click Synthesizer (`MacTapticEvent`, `SPEC-014 §2.7`).

use super::render::polar_pos;
use super::{MacThumbDialLayer, MacThumbSector};
use crate::controls::SegmentIconFn;
use crate::symbols::MacSymbol;
use crate::tokens::ThumbDialMetrics;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, Vec2};

pub(crate) struct WedgeGeometry {
    pub(crate) center: Pos2,
    pub(crate) r_in: f32,
    pub(crate) r_out: f32,
    pub(crate) start_deg: f32,
    pub(crate) end_deg: f32,
}

pub(crate) fn paint_annular_wedge(painter: &Painter, geom: WedgeGeometry, fill: Color32, stroke: Stroke, m: &ThumbDialMetrics) {
    let steps = 12usize;
    let sweep = geom.end_deg - geom.start_deg;
    let r_mid = (geom.r_in + geom.r_out) * 0.5;
    let sheen = fill.gamma_multiply(m.hover_fill_opacity.points());
    for i in 0..steps {
        let t0 = i as f32 / steps as f32;
        let t1 = (i + 1) as f32 / steps as f32;
        let a0 = geom.start_deg + sweep * t0;
        let a1 = geom.start_deg + sweep * t1;
        let quad = vec![
            polar_pos(geom.center, geom.r_in, a0, m),
            polar_pos(geom.center, geom.r_out, a0, m),
            polar_pos(geom.center, geom.r_out, a1, m),
            polar_pos(geom.center, geom.r_in, a1, m),
        ];
        painter.add(Shape::convex_polygon(quad, fill, Stroke::NONE));
        let outer_sheen = vec![
            polar_pos(geom.center, r_mid, a0, m),
            polar_pos(geom.center, geom.r_out, a0, m),
            polar_pos(geom.center, r_out_clamp(geom.r_out), a1, m),
            polar_pos(geom.center, r_mid, a1, m),
        ];
        painter.add(Shape::convex_polygon(outer_sheen, sheen, Stroke::NONE));
    }
    if stroke.width > 0.0 {
        let mut ring_pts = Vec::with_capacity((steps + 1) * 2);
        let mut outer_arc = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let a = geom.start_deg + sweep * (i as f32 / steps as f32);
            let pt_out = polar_pos(geom.center, geom.r_out, a, m);
            ring_pts.push(pt_out);
            outer_arc.push(pt_out);
        }
        for i in (0..=steps).rev() {
            let a = geom.start_deg + sweep * (i as f32 / steps as f32);
            ring_pts.push(polar_pos(geom.center, geom.r_in, a, m));
        }
        painter.add(Shape::closed_line(ring_pts, stroke));
        painter.add(Shape::line(outer_arc, Stroke::new(m.active_stroke_width.points(), stroke.color)));
    }
}

#[inline]
const fn r_out_clamp(r: f32) -> f32 {
    r
}

/// Official Apple SF Symbols identifiers rendered as resolution-independent vector strokes (`REQ-014-07`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacSFSymbol {
    /// `"aspectratio"` — $1:1$ aspect ratio frame with corner ticks.
    AspectRatio,
    /// `"scope"` — Center origin target ($\odot$) with crosshair ticks.
    Scope,
    /// `"viewfinder"` — Combined $1:1$ aspect frame + center target ($\odot$).
    Viewfinder,
    /// `"plus.square.on.square"` — Overlapping duplicate/clone squares.
    PlusSquareOnSquare,
    /// `"skew"` — Slanted shear parallelogram with horizontal arrows.
    Skew,
    /// `"scribble"` — Smooth S-curve with a sliding on-curve anchor point.
    Scribble,
    /// `"sparkles"` — Four-point $G^2$ continuity Genius star ($\star$).
    Sparkles,
    /// `"plus.circle"` — Add node ($\oplus$) on a contour segment.
    PlusCircle,
    /// `"arrow.up.and.down.and.left.and.right"` — Four-way move/translate arrows ($\✥$).
    ArrowUpAndDownAndLeftAndRight,
    /// `"hand.raised"` — Pan canvas directional cross with center grip.
    HandRaised,
    /// `"angle"` — $45^\circ / 15^\circ$ angle constraint wedge with degree arc.
    Angle,
    /// `"point.topleft.down.curvedto.point.bottomright.up"` — Sharp cusp corner ($\wedge$) with broken handles.
    PointTopleftDownCurvedtoPointBottomrightUp,
    /// `"shift"` — Apple `⇧` Shift symbol.
    Shift,
    /// `"option"` — Apple `⌥` Option symbol.
    OptionKey,
    /// `"command"` — Apple `⌘` Command symbol.
    CommandKey,
    /// `"control"` — Apple `⌃` Control symbol.
    ControlKey,
}

impl MacSFSymbol {
    /// Returns the official Apple SF Symbols catalog identifier string (`REQ-014-07`).
    #[inline]
    pub const fn sf_symbol_name(self) -> &'static str {
        match self {
            Self::AspectRatio => "aspectratio",
            Self::Scope => "scope",
            Self::Viewfinder => "viewfinder",
            Self::PlusSquareOnSquare => "plus.square.on.square",
            Self::Skew => "skew",
            Self::Scribble => "scribble",
            Self::Sparkles => "sparkles",
            Self::PlusCircle => "plus.circle",
            Self::ArrowUpAndDownAndLeftAndRight => "arrow.up.and.down.and.left.and.right",
            Self::HandRaised => "hand.raised",
            Self::Angle => "angle",
            Self::PointTopleftDownCurvedtoPointBottomrightUp => "point.topleft.down.curvedto.point.bottomright.up",
            Self::Shift => "shift",
            Self::OptionKey => "option",
            Self::CommandKey => "command",
            Self::ControlKey => "control",
        }
    }
}

/// Pluggable icon descriptor for a single radial cushion of [`super::MacContextThumbDial`] (`REQ-014-07`).
#[derive(Clone, Copy)]
pub enum MacThumbSectorIcon {
    /// Official Apple SF Symbol rendered via resolution-independent vector strokes.
    SFSymbol(MacSFSymbol),
    /// Embedded `CupertinoIcons` font glyph ([`MacSymbol`]).
    Cupertino(MacSymbol),
    /// Host-provided zero-allocation vector painter callback (`fn(&Painter, Rect, Color32)`).
    CustomFn(SegmentIconFn),
}

impl std::fmt::Debug for MacThumbSectorIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SFSymbol(s) => f.debug_tuple("SFSymbol").field(s).finish(),
            Self::Cupertino(c) => f.debug_tuple("Cupertino").field(c).finish(),
            Self::CustomFn(_) => f.write_str("CustomFn(fn)"),
        }
    }
}

impl PartialEq for MacThumbSectorIcon {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::SFSymbol(a), Self::SFSymbol(b)) => a == b,
            (Self::Cupertino(a), Self::Cupertino(b)) => a == b,
            (Self::CustomFn(a), Self::CustomFn(b)) => std::ptr::fn_addr_eq(*a, *b),
            _ => false,
        }
    }
}

impl Eq for MacThumbSectorIcon {}

pub(super) fn paint_sector_icon(painter: &Painter, icon: MacThumbSectorIcon, rect: Rect, color: Color32, m: &ThumbDialMetrics) {
    match icon {
        MacThumbSectorIcon::CustomFn(cb) => cb(painter, rect, color),
        MacThumbSectorIcon::Cupertino(sym) => {
            painter.text(rect.center(), Align2::CENTER_CENTER, sym.as_str(), FontId::proportional(rect.height()), color);
        }
        MacThumbSectorIcon::SFSymbol(sf) => paint_sf_symbol_vector(painter, sf, rect, color, m),
    }
}

fn paint_sf_symbol_vector(painter: &Painter, sf: MacSFSymbol, rect: Rect, color: Color32, m: &ThumbDialMetrics) {
    let c = rect.center();
    let r = rect.width().min(rect.height()) * 0.5;
    let q = r * 0.5;
    let stroke = Stroke::new(m.active_stroke_width.points(), color);
    let thin = Stroke::new(m.border_stroke_width.points(), color);

    match sf {
        MacSFSymbol::AspectRatio => {
            let b = Rect::from_center_size(c, Vec2::splat(r * 2.0));
            painter.rect_stroke(b, 2, stroke, egui::StrokeKind::Inside);
            painter.line_segment([b.min + Vec2::new(q * 0.5, q * 0.5), b.max - Vec2::new(q * 0.5, q * 0.5)], thin);
        }
        MacSFSymbol::Scope => {
            painter.circle_stroke(c, r, stroke);
            painter.circle_filled(c, q * 0.5, color);
            painter.line_segment([c - Vec2::new(r + q * 0.5, 0.0), c - Vec2::new(q, 0.0)], thin);
            painter.line_segment([c + Vec2::new(q, 0.0), c + Vec2::new(r + q * 0.5, 0.0)], thin);
            painter.line_segment([c - Vec2::new(0.0, r + q * 0.5), c - Vec2::new(0.0, q)], thin);
            painter.line_segment([c + Vec2::new(0.0, q), c + Vec2::new(0.0, r + q * 0.5)], thin);
        }
        MacSFSymbol::Viewfinder => {
            let b = Rect::from_center_size(c, Vec2::splat(r * 2.0));
            painter.rect_stroke(b, 2, stroke, egui::StrokeKind::Inside);
            painter.circle_stroke(c, q, thin);
            painter.circle_filled(c, q * 0.5, color);
        }
        MacSFSymbol::PlusSquareOnSquare => {
            let off = Vec2::new(q * 0.5, q * 0.5);
            let b1 = Rect::from_center_size(c - off, Vec2::splat(r + q));
            let b2 = Rect::from_center_size(c + off, Vec2::splat(r + q));
            painter.rect_stroke(b1, 1, thin, egui::StrokeKind::Inside);
            painter.rect_stroke(b2, 1, stroke, egui::StrokeKind::Inside);
        }
        MacSFSymbol::Skew => {
            let pts = vec![
                c + Vec2::new(-r + q, -r + q * 0.5),
                c + Vec2::new(r, -r + q * 0.5),
                c + Vec2::new(r - q, r - q * 0.5),
                c + Vec2::new(-r, r - q * 0.5),
            ];
            painter.add(Shape::closed_line(pts, stroke));
        }
        MacSFSymbol::Scribble => {
            let p0 = c + Vec2::new(-r, r - q * 0.5);
            let p1 = c + Vec2::new(-q * 0.5, -r + q * 0.5);
            let p2 = c + Vec2::new(q * 0.5, r - q * 0.5);
            let p3 = c + Vec2::new(r, -r + q * 0.5);
            painter.add(Shape::line(vec![p0, p1, p2, p3], stroke));
            painter.circle_filled(c, q * 0.5 + m.border_stroke_width.points(), color);
        }
        MacSFSymbol::Sparkles => {
            let pts = vec![
                c + Vec2::new(0.0, -r),
                c + Vec2::new(q * 0.5, -q * 0.5),
                c + Vec2::new(r, 0.0),
                c + Vec2::new(q * 0.5, q * 0.5),
                c + Vec2::new(0.0, r),
                c + Vec2::new(-q * 0.5, q * 0.5),
                c + Vec2::new(-r, 0.0),
                c + Vec2::new(-q * 0.5, -q * 0.5),
            ];
            painter.add(Shape::convex_polygon(pts, color, thin));
        }
        MacSFSymbol::PlusCircle => {
            painter.circle_stroke(c, r, stroke);
            painter.line_segment([c - Vec2::new(q, 0.0), c + Vec2::new(q, 0.0)], stroke);
            painter.line_segment([c - Vec2::new(0.0, q), c + Vec2::new(0.0, q)], stroke);
        }
        MacSFSymbol::ArrowUpAndDownAndLeftAndRight | MacSFSymbol::HandRaised => {
            painter.line_segment([c - Vec2::new(r, 0.0), c + Vec2::new(r, 0.0)], stroke);
            painter.line_segment([c - Vec2::new(0.0, r), c + Vec2::new(0.0, r)], stroke);
            painter.circle_filled(c, q * 0.5, color);
        }
        MacSFSymbol::Angle => {
            let v = c + Vec2::new(-r + q * 0.5, r - q * 0.5);
            painter.line_segment([v, c + Vec2::new(r, r - q * 0.5)], stroke);
            painter.line_segment([v, c + Vec2::new(q, -r + q * 0.5)], stroke);
            painter.circle_stroke(v, q, thin);
        }
        MacSFSymbol::PointTopleftDownCurvedtoPointBottomrightUp => {
            let apex = c + Vec2::new(0.0, -r + q * 0.5);
            painter.line_segment([c + Vec2::new(-r, r - q * 0.5), apex], stroke);
            painter.line_segment([apex, c + Vec2::new(r, r - q * 0.5)], stroke);
            painter.circle_filled(apex, q * 0.5, color);
        }
        MacSFSymbol::Shift | MacSFSymbol::OptionKey | MacSFSymbol::CommandKey | MacSFSymbol::ControlKey => {
            painter.circle_stroke(c, r, stroke);
            painter.circle_filled(c, q * 0.5, color);
        }
    }
}

/// Semantic Haptic & Auditory Micro-Click Event emitted by `SPEC-014` controls (`REQ-014-07`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacTapticEvent {
    /// Emitted on each $15^\circ$ ridge detent step of `MacEdgeThumbwheel` or rotary ring.
    RotaryDetentTick,
    /// Emitted when touching a free-spinning `MacEdgeThumbwheel` to brake it (`Touch-to-Brake`).
    FlywheelBraked,
    /// Emitted when a compass cushion is exclusively latched on (`Tap-to-Latch`).
    SectorLatched(MacThumbSector),
    /// Emitted when a latched cushion is toggled off or cleared via the Center Hub.
    SectorUnlatched,
    /// Emitted when the Center Hub toggles between `CompassModifiers` and `FreeJoystickBowl`.
    LayerSwitched(MacThumbDialLayer),
    /// Emitted when the 3D crystal marble crosses the `Fine` or `Fast` speed ring.
    JoystickRingCrossed,
    /// Emitted when clicking `↶ Undo` (`is_redo = false`) or `↷ Redo` (`is_redo = true`).
    HistoryStep {
        /// `true` for `↷ Redo`, `false` for `↶ Undo`.
        is_redo: bool,
    },
}

impl MacTapticEvent {
    /// Synthesizes a short, zero-dependency 16-bit PCM RIFF/WAVE Taptic micro-click buffer (`REQ-014-07`).
    pub fn synthesize_micro_click_wav(self) -> Vec<u8> {
        let sample_rate: u32 = 22050;
        let (freq_hz, decay, num_samples): (u32, u32, usize) = match self {
            Self::RotaryDetentTick => (1650, 260, 180),
            Self::FlywheelBraked => (240, 140, 320),
            Self::SectorLatched(_) => (1280, 180, 260),
            Self::SectorUnlatched => (780, 180, 260),
            Self::LayerSwitched(_) => (1040, 150, 300),
            Self::JoystickRingCrossed => (1420, 280, 160),
            Self::HistoryStep { is_redo: false } => (880, 200, 220),
            Self::HistoryStep { is_redo: true } => (1180, 200, 220),
        };
        let data_len = (num_samples * 2) as u32;
        let file_len = 36 + data_len;
        let mut out = Vec::with_capacity(44 + data_len as usize);
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&file_len.to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&sample_rate.to_le_bytes());
        out.extend_from_slice(&(sample_rate * 2).to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());
        let amp = i16::MAX as f32 * 0.5;
        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            let env = (-t * (decay as f32)).exp();
            let s = (t * (freq_hz as f32) * std::f32::consts::TAU).sin() * env;
            let pcm = (s * amp) as i16;
            out.extend_from_slice(&pcm.to_le_bytes());
        }
        out
    }
}
