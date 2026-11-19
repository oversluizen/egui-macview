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
    let radial_rings = 6usize;
    let ang_steps = 24usize;
    let sweep = geom.end_deg - geom.start_deg;
    let mid_deg = (geom.start_deg + geom.end_deg) * 0.5;
    let half_sweep = sweep.abs() * 0.5;
    let fillet_pt = m.ring_gap.points() * 2.0;
    let bevel_w = m.bevel_width_pt.points().max(fillet_pt);
    let bw = m.border_stroke_width.points();
    let cursor_est = (light_vec.length_sq() > 0.0)
        .then(|| geom.center + light_vec * geom.r_out)
        .or_else(|| Some(geom.center - Vec2::new(geom.r_in * 0.5, geom.r_out)));
    let r_mid = (geom.r_in + geom.r_out) * 0.5;
    let c_centroid = polar_pos(geom.center, r_mid, mid_deg, m);
    let r_span = (geom.r_out - geom.r_in).max(1.0);
    let vel_off = light_vec * r_span * 0.5;
    let press_amt = light_vec.length().clamp(0.0, 1.0);
    let deform_pt = |p: Pos2| -> Pos2 {
        super::super::liquid_glass::deform_viscous_droplet_point(p, c_centroid, vel_off, r_span, press_amt)
    };

    let mut sh_pts = Vec::with_capacity((ang_steps + 1) * 2);
    for j in 0..=ang_steps {
        let a = geom.start_deg + sweep * (j as f32 / ang_steps as f32);
        let (_, ro) = filleted_radii(j, ang_steps, geom.r_in, geom.r_out, fillet_pt);
        sh_pts.push(deform_pt(polar_pos(geom.center, ro, a, m)) + Vec2::new(0.0, bw * 2.0));
    }
    for j in (0..=ang_steps).rev() {
        let a = geom.start_deg + sweep * (j as f32 / ang_steps as f32);
        let (ri, _) = filleted_radii(j, ang_steps, geom.r_in, geom.r_out, fillet_pt);
        sh_pts.push(deform_pt(polar_pos(geom.center, ri, a, m)) + Vec2::new(0.0, bw * 2.0));
    }
    painter.add(Shape::convex_polygon(
        sh_pts,
        shade_color(fill, m.active_fill_opacity.points()).gamma_multiply(m.hover_fill_opacity.points()),
        Stroke::NONE,
    ));

    let mut mesh = egui::epaint::Mesh::default();
    mesh.vertices.reserve((radial_rings + 1) * (ang_steps + 1));
    mesh.indices.reserve(radial_rings * ang_steps * 6);
    for r_i in 0..=radial_rings {
        let u_r = r_i as f32 / radial_rings as f32;
        for j in 0..=ang_steps {
            let a = geom.start_deg + sweep * (j as f32 / ang_steps as f32);
            let (ri, ro) = filleted_radii(j, ang_steps, geom.r_in, geom.r_out, fillet_pt);
            let r = ri + (ro - ri) * u_r;
            let raw_pt = polar_pos(geom.center, r, a, m);
            let pt = deform_pt(raw_pt);
            let sdf_sample = vitrea::sdf_annular_rounded_sector_sample(
                vitrea::GlamVec2::new(raw_pt.x, raw_pt.y),
                vitrea::GlamVec2::new(geom.center.x, geom.center.y),
                (geom.r_in, geom.r_out),
                (mid_deg, half_sweep),
                fillet_pt,
            );
            let n3d = super::super::liquid_glass::dome_surface_normal_3d_from_sample(sdf_sample, bevel_w);
            let fresnel_glint = super::super::liquid_glass::evaluate_vertex_cursor_fresnel_lighting(pt, n3d, cursor_est, geom.r_out);
            let top_dome = (-n3d[1]).clamp(0.0, 1.0) * m.hover_fill_opacity.points();
            let caustic_core = (n3d[2] * n3d[2]) * m.hover_fill_opacity.points();
            let bevel_rim = (1.0 - n3d[2].clamp(0.0, 1.0)) * m.hover_fill_opacity.points();
            let base_c = shade_color(fill, bevel_rim * 0.5);
            let lit_c = brighten_color(base_c, top_dome + caustic_core + fresnel_glint * m.glass_specular_alpha.points());
            mesh.colored_vertex(pt, lit_c);
        }
    }
    let stride = (ang_steps + 1) as u32;
    for r_i in 0..radial_rings {
        for j in 0..ang_steps {
            let b0 = (r_i as u32) * stride + (j as u32);
            let b1 = b0 + stride;
            mesh.add_triangle(b0, b1, b1 + 1);
            mesh.add_triangle(b0, b1 + 1, b0 + 1);
        }
    }
    painter.add(Shape::mesh(mesh));

    // Iter P2 (`REQ-016-02.7`): Live Snellius 3D Lens Refraction (`vitrea::refract_ray_snell_3d`) of sub-cushion dish rings
    let lens_steps = 12usize;
    let ior = m.ior_glass.points();
    let dome_thick = m.dome_thickness_pt.points();
    let lens_col = stroke.color.gamma_multiply(m.hover_fill_opacity.points());
    for r_ratio in [m.joystick_fine_ratio.points(), m.joystick_fast_ratio.points()] {
        let r_ring = geom.r_in + r_span * r_ratio;
        let mut refr_arc = Vec::with_capacity(lens_steps + 1);
        for k in 0..=lens_steps {
            let a = geom.start_deg + sweep * (k as f32 / lens_steps as f32);
            let pt = polar_pos(geom.center, r_ring, a, m);
            let s_k = vitrea::sdf_annular_rounded_sector_sample(
                vitrea::GlamVec2::new(pt.x, pt.y),
                vitrea::GlamVec2::new(geom.center.x, geom.center.y),
                (geom.r_in, geom.r_out),
                (mid_deg, half_sweep),
                fillet_pt,
            );
            let n_k = vitrea::dome_normal_3d_from_sdf(s_k, bevel_w);
            let refr = vitrea::refract_ray_snell_3d(vitrea::GlamVec2::new(pt.x, pt.y), n_k, dome_thick * 0.5, ior);
            refr_arc.push(deform_pt(Pos2::new(refr.x, refr.y)));
        }
        painter.add(Shape::line(refr_arc, Stroke::new(bw, lens_col)));
    }

    if stroke.width > 0.0 {
        let mut ring_pts = Vec::with_capacity((ang_steps + 1) * 2);
        let mut outer_arc = Vec::with_capacity(ang_steps + 1);
        for j in 0..=ang_steps {
            let a = geom.start_deg + sweep * (j as f32 / ang_steps as f32);
            let (_, ro) = filleted_radii(j, ang_steps, geom.r_in, geom.r_out, fillet_pt);
            let pt_out = deform_pt(polar_pos(geom.center, ro, a, m));
            ring_pts.push(pt_out);
            outer_arc.push(pt_out);
        }
        for j in (0..=ang_steps).rev() {
            let a = geom.start_deg + sweep * (j as f32 / ang_steps as f32);
            let (ri, _) = filleted_radii(j, ang_steps, geom.r_in, geom.r_out, fillet_pt);
            ring_pts.push(deform_pt(polar_pos(geom.center, ri, a, m)));
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
