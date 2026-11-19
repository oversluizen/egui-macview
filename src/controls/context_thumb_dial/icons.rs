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

fn filleted_radii(i: usize, steps: usize, r_in: f32, r_out: f32, fillet_pt: f32) -> (f32, f32) {
    let corner_steps = 8usize;
    let u = if i < corner_steps {
        (i as f32) / (corner_steps as f32)
    } else if i > steps.saturating_sub(corner_steps) {
        ((steps - i) as f32) / (corner_steps as f32)
    } else {
        1.0
    };
    if u >= 1.0 {
        return (r_in, r_out);
    }
    let du = 1.0 - u;
    let circle_prof = 1.0 - (1.0 - du * du).max(0.0).sqrt();
    let inset_in = fillet_pt * circle_prof;
    let inset_out = (fillet_pt + fillet_pt * 0.5) * circle_prof;
    let r_i = r_in + inset_in;
    let r_o = (r_out - inset_out).max(r_i + 1.0);
    (r_i, r_o)
}

fn fill_annular_strip(
    painter: &Painter,
    center: Pos2,
    (r_in, r_out): (f32, f32),
    (start_deg, end_deg): (f32, f32),
    (col_in, col_out): (Color32, Color32),
    light_vec: Vec2,
    m: &ThumbDialMetrics,
) {
    let steps = 32usize;
    let sweep = end_deg - start_deg;
    let fillet_pt = m.ring_gap.points() * 2.0;
    let mid_deg = (start_deg + end_deg) * 0.5;
    let mid_rad = super::render::deg_to_rad(mid_deg, m);
    let sec_dir = Vec2::new(mid_rad.cos(), mid_rad.sin());
    let dir_boost = (sec_dir.x * light_vec.x + sec_dir.y * light_vec.y).max(0.0);
    let bevel_w = (r_out - r_in).max(1.0) * 0.5;
    let r_mid = (r_in + r_out) * 0.5;
    let c_center = polar_pos(center, r_mid, mid_deg, m);
    let cursor_est = (light_vec.length_sq() > 0.0).then(|| center + light_vec * r_out);
    let mut mesh = egui::epaint::Mesh::default();
    mesh.vertices.reserve((steps + 1) * 2);
    mesh.indices.reserve(steps * 6);
    for i in 0..=steps {
        let a = start_deg + sweep * (i as f32 / steps as f32);
        let a_rad = super::render::deg_to_rad(a, m);
        let n_dir = Vec2::new(a_rad.cos(), a_rad.sin());
        let spec = (n_dir.x * light_vec.x + n_dir.y * light_vec.y).max(0.0);
        let top_bias = (-n_dir.y).clamp(0.0, 1.0) * m.hover_fill_opacity.points();
        let (ri, ro) = filleted_radii(i, steps, r_in, r_out, fillet_pt);
        let p_in = polar_pos(center, ri, a, m);
        let p_out = polar_pos(center, ro, a, m);
        let n_in = super::super::liquid_glass::dome_surface_normal_3d(p_in, c_center, bevel_w, fillet_pt);
        let n_out = super::super::liquid_glass::dome_surface_normal_3d(p_out, c_center, bevel_w, fillet_pt);
        let prox_in = super::super::liquid_glass::evaluate_vertex_cursor_fresnel_lighting(p_in, n_in, cursor_est, r_out);
        let prox_out = super::super::liquid_glass::evaluate_vertex_cursor_fresnel_lighting(p_out, n_out, cursor_est, r_out);
        let boost_in = (dir_boost + spec) * 0.5 + top_bias + prox_in * 0.5;
        let boost_out = (dir_boost + spec) * 0.5 + top_bias + prox_out;
        mesh.colored_vertex(p_in, brighten_color(col_in, boost_in * m.active_fill_opacity.points()));
        mesh.colored_vertex(p_out, brighten_color(col_out, boost_out * m.glass_specular_alpha.points()));
    }
    for i in 0..steps {
        let base = (i * 2) as u32;
        mesh.add_triangle(base, base + 1, base + 3);
        mesh.add_triangle(base, base + 3, base + 2);
    }
    painter.add(Shape::mesh(mesh));
}

fn brighten_color(c: Color32, gain: f32) -> Color32 {
    let mut hsva = egui::epaint::Hsva::from(c);
    let g_clamped = gain.clamp(0.0, 1.0);
    hsva.v = (hsva.v + (1.0 - hsva.v) * g_clamped).clamp(0.0, 1.0);
    hsva.s = (hsva.s * (1.0 - g_clamped * 0.5)).clamp(0.0, 1.0);
    Color32::from(hsva)
}

fn shade_color(c: Color32, drop: f32) -> Color32 {
    let mut hsva = egui::epaint::Hsva::from(c);
    let d_clamped = drop.clamp(0.0, 1.0);
    hsva.v = (hsva.v * (1.0 - d_clamped)).clamp(0.0, 1.0);
    Color32::from(hsva)
}

pub(crate) fn paint_annular_wedge_lit(
    painter: &Painter,
    geom: WedgeGeometry,
    fill: Color32,
    stroke: Stroke,
    light_vec: Vec2,
    m: &ThumbDialMetrics,
) {
    let steps = 32usize;
    let sweep = geom.end_deg - geom.start_deg;
    let r_mid = (geom.r_in + geom.r_out) * 0.5;
    let r_top = (r_mid + geom.r_out) * 0.5;
    let dome_hi = brighten_color(fill, m.hover_fill_opacity.points());
    let sheen = brighten_color(fill, m.active_fill_opacity.points());
    let deep_fill = shade_color(fill, m.hover_fill_opacity.points());
    let fillet_pt = m.ring_gap.points() * 2.0;
    let bw = m.border_stroke_width.points();
    let mut sh_pts = Vec::with_capacity((steps + 1) * 2);
    for i in 0..=steps {
        let a = geom.start_deg + sweep * (i as f32 / steps as f32);
        let (_, ro) = filleted_radii(i, steps, geom.r_in, geom.r_out, fillet_pt);
        sh_pts.push(polar_pos(geom.center + Vec2::new(0.0, bw * 2.0), ro, a, m));
    }
    for i in (0..=steps).rev() {
        let a = geom.start_deg + sweep * (i as f32 / steps as f32);
        let (ri, _) = filleted_radii(i, steps, geom.r_in, geom.r_out, fillet_pt);
        sh_pts.push(polar_pos(geom.center + Vec2::new(0.0, bw * 2.0), ri, a, m));
    }
    painter.add(Shape::convex_polygon(sh_pts, shade_color(fill, m.active_fill_opacity.points()).gamma_multiply(m.hover_fill_opacity.points()), Stroke::NONE));
    fill_annular_strip(painter, geom.center, (geom.r_in, geom.r_out), (geom.start_deg, geom.end_deg), (deep_fill, dome_hi), light_vec, m);
    fill_annular_strip(painter, geom.center, (r_mid, geom.r_out), (geom.start_deg, geom.end_deg), (fill, sheen), light_vec, m);
    fill_annular_strip(painter, geom.center, (r_top, geom.r_out), (geom.start_deg, geom.end_deg), (dome_hi, stroke.color), light_vec, m);
    if stroke.width > 0.0 {
        let mut ring_pts = Vec::with_capacity((steps + 1) * 2);
        let mut outer_arc = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let a = geom.start_deg + sweep * (i as f32 / steps as f32);
            let (_, ro) = filleted_radii(i, steps, geom.r_in, geom.r_out, fillet_pt);
            let pt_out = polar_pos(geom.center, ro, a, m);
            ring_pts.push(pt_out);
            outer_arc.push(pt_out);
        }
        for i in (0..=steps).rev() {
            let a = geom.start_deg + sweep * (i as f32 / steps as f32);
            let (ri, _) = filleted_radii(i, steps, geom.r_in, geom.r_out, fillet_pt);
            ring_pts.push(polar_pos(geom.center, ri, a, m));
        }
        painter.add(Shape::closed_line(ring_pts, stroke));
        painter.add(Shape::line(outer_arc, Stroke::new(m.active_stroke_width.points(), stroke.color)));
    }
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
    /// `"angle.and.cusp"` — $45^\circ$ angle constraint combined with cusp corner apex.
    AngleAndCusp,
    /// `"sparkles.and.cusp"` — $G^2$ continuity star combined with cusp corner chevron.
    SparklesAndCusp,
    /// `"axis.and.square.on.square"` — Orthogonal axis lock combined with duplicate square.
    AxisAndClone,
    /// `"point.single.handle"` — Anchor point with single-side independent handle lever.
    SingleSideHandle,
    /// `"angle.and.plus"` — $15^\circ$ slice angle combined with add-node badge.
    AngleAndPlus,
    /// `"scope.and.plus"` — Grid snap target combined with add-node badge.
    ScopeAndPlus,
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
            Self::AngleAndCusp => "angle.and.cusp",
            Self::SparklesAndCusp => "sparkles.and.cusp",
            Self::AxisAndClone => "axis.and.square.on.square",
            Self::SingleSideHandle => "point.single.handle",
            Self::AngleAndPlus => "angle.and.plus",
            Self::ScopeAndPlus => "scope.and.plus",
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
            let p_tl = b.min + Vec2::new(q * 0.5, q * 0.5);
            let p_br = b.max - Vec2::new(q * 0.5, q * 0.5);
            painter.line_segment([p_tl, p_tl + Vec2::new(q, 0.0)], stroke);
            painter.line_segment([p_tl, p_tl + Vec2::new(0.0, q)], stroke);
            painter.line_segment([p_br, p_br - Vec2::new(q, 0.0)], stroke);
            painter.line_segment([p_br, p_br - Vec2::new(0.0, q)], stroke);
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
            let ah = q * 0.5;
            painter.line_segment([c - Vec2::new(r, 0.0), c - Vec2::new(r - ah, ah)], thin);
            painter.line_segment([c - Vec2::new(r, 0.0), c - Vec2::new(r - ah, -ah)], thin);
            painter.line_segment([c + Vec2::new(r, 0.0), c + Vec2::new(r - ah, ah)], thin);
            painter.line_segment([c + Vec2::new(r, 0.0), c + Vec2::new(r - ah, -ah)], thin);
            painter.line_segment([c - Vec2::new(0.0, r), c - Vec2::new(ah, r - ah)], thin);
            painter.line_segment([c - Vec2::new(0.0, r), c - Vec2::new(-ah, r - ah)], thin);
            painter.line_segment([c + Vec2::new(0.0, r), c + Vec2::new(ah, r - ah)], thin);
            painter.line_segment([c + Vec2::new(0.0, r), c + Vec2::new(-ah, r - ah)], thin);
            painter.circle_filled(c, ah * 0.5, color);
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
        MacSFSymbol::AngleAndCusp => {
            let v = c + Vec2::new(-r + q * 0.5, r - q * 0.5);
            painter.line_segment([v, c + Vec2::new(r, r - q * 0.5)], stroke);
            painter.line_segment([v, c + Vec2::new(0.0, -r + q * 0.5)], stroke);
            painter.circle_filled(c + Vec2::new(0.0, -r + q * 0.5), q * 0.5, color);
            painter.rect_stroke(Rect::from_center_size(c + Vec2::new(q, 0.0), Vec2::splat(q)), 1, thin, egui::StrokeKind::Inside);
        }
        MacSFSymbol::SparklesAndCusp => {
            let pts = vec![
                c + Vec2::new(-q * 0.5, -r),
                c + Vec2::new(0.0, -q * 0.5),
                c + Vec2::new(q, 0.0),
                c + Vec2::new(0.0, q * 0.5),
                c + Vec2::new(-q * 0.5, r),
                c + Vec2::new(-q, q * 0.5),
                c + Vec2::new(-r, 0.0),
                c + Vec2::new(-q, -q * 0.5),
            ];
            painter.add(Shape::convex_polygon(pts, color, thin));
            painter.line_segment([c + Vec2::new(q * 0.5, q), c + Vec2::new(r, -q)], stroke);
            painter.circle_filled(c + Vec2::new(r, -q), q * 0.5, color);
        }
        MacSFSymbol::AxisAndClone => {
            painter.line_segment([c - Vec2::new(r, 0.0), c + Vec2::new(r, 0.0)], thin);
            painter.line_segment([c - Vec2::new(0.0, r), c + Vec2::new(0.0, r)], thin);
            let b = Rect::from_center_size(c + Vec2::new(q * 0.5, -q * 0.5), Vec2::splat(r));
            painter.rect_stroke(b, 1, stroke, egui::StrokeKind::Inside);
        }
        MacSFSymbol::SingleSideHandle => {
            let anchor = c + Vec2::new(-q, q * 0.5);
            let handle = c + Vec2::new(r - q * 0.5, -r + q * 0.5);
            painter.line_segment([anchor, handle], stroke);
            painter.rect_filled(Rect::from_center_size(anchor, Vec2::splat(q)), 1, color);
            painter.circle_stroke(handle, q * 0.5, stroke);
        }
        MacSFSymbol::AngleAndPlus => {
            let v = c + Vec2::new(-r + q * 0.5, r - q * 0.5);
            painter.line_segment([v, c + Vec2::new(r, r - q * 0.5)], stroke);
            painter.line_segment([v, c + Vec2::new(0.0, -r + q * 0.5)], stroke);
            let pc = c + Vec2::new(q, -q * 0.5);
            painter.line_segment([pc - Vec2::new(q * 0.5, 0.0), pc + Vec2::new(q * 0.5, 0.0)], stroke);
            painter.line_segment([pc - Vec2::new(0.0, q * 0.5), pc + Vec2::new(0.0, q * 0.5)], stroke);
        }
        MacSFSymbol::ScopeAndPlus => {
            painter.circle_stroke(c - Vec2::new(q * 0.5, 0.0), r - q * 0.5, stroke);
            let pc = c + Vec2::new(q, -q);
            painter.line_segment([pc - Vec2::new(q * 0.5, 0.0), pc + Vec2::new(q * 0.5, 0.0)], stroke);
            painter.line_segment([pc - Vec2::new(0.0, q * 0.5), pc + Vec2::new(0.0, q * 0.5)], stroke);
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
