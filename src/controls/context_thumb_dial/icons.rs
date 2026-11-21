//! Official Apple SF Symbol Vector Iconography (`MacSFSymbol`, `MacThumbSectorIcon`) &
//! Taptic/Auditory Micro-Click Synthesizer (`MacTapticEvent`, `SPEC-014 §2.7`).

use super::{MacThumbDialLayer, MacThumbSector};
use crate::controls::SegmentIconFn;
use crate::symbols::MacSymbol;
use crate::theme::MacTheme;
use crate::tokens::ThumbDialMetrics;
use egui::{Align2, Color32, FontId, Painter, Rect, Shape, Stroke, Vec2};

/// Member descriptor for the 3-Matrix `GlassEffectContainer` rosette pipeline (`SPEC-016 §4.4.4`).
pub(crate) struct RosetteMemberSpec {
    pub(crate) center: vitrea::GlamVec2,
    pub(crate) radii: (f32, f32),
    pub(crate) angles_deg: (f32, f32),
    pub(crate) corner_radius: f32,
    pub(crate) active_weight: f32,
    pub(crate) lift_weight: f32,
    pub(crate) deform_vel: vitrea::GlamVec2,
}

/// Container descriptor for the 3-Matrix `GlassEffectContainer` rosette pipeline (`SPEC-016 §4.4.4`, `REQ-016-B1.9`).
pub(crate) struct RosetteContainerSpec {
    pub(crate) hub_center: vitrea::GlamVec2,
    pub(crate) r_outer: f32,
    pub(crate) r_dish: f32,
    pub(crate) r_hub: f32,
    pub(crate) r_skirt_in: f32,
    pub(crate) r_skirt_out: f32,
    pub(crate) undo_arc_deg: (f32, f32),
    pub(crate) redo_arc_deg: (f32, f32),
    pub(crate) undo_active_weight: f32,
    pub(crate) redo_active_weight: f32,
    pub(crate) scale: f32,
    pub(crate) cursor_pos: Option<vitrea::GlamVec2>,
    pub(crate) accent_col: Color32,
    pub(crate) members: Vec<RosetteMemberSpec>,
}

fn color_to_rgb_norm(c: Color32) -> [f32; 3] {
    let [r, g, b, _] = c.to_array();
    let max_u8 = f32::from(u8::MAX);
    [f32::from(r) / max_u8, f32::from(g) / max_u8, f32::from(b) / max_u8]
}

use super::paint::eval_shared_dish_matrix_at;

pub(crate) fn synthesize_rosette_matrix_rgba(tex_dim: usize, sc: &RosetteContainerSpec, theme: &MacTheme, m: &ThumbDialMetrics) -> Vec<u8> {
    let mut flat = vec![0u8; tex_dim * tex_dim * 4];
    let is_light = theme.appearance == crate::theme::MacAppearance::Aqua;
    let dish_core = color_to_rgb_norm(theme.window_background());
    let dish_rim = color_to_rgb_norm(if is_light { theme.box_background() } else { theme.control_background() });
    let passive_tint = color_to_rgb_norm(theme.specular_rim_top());
    let (accent_rgb, grid_rgb, shadow_rgb, warm_rgb) = (
        color_to_rgb_norm(sc.accent_col),
        color_to_rgb_norm(theme.secondary_label()),
        color_to_rgb_norm(theme.drop_shadow()),
        color_to_rgb_norm(theme.system_orange()),
    );
    let params = vitrea::LiquidGlassOpticalParams {
        ior_glass: m.ior_glass.points(), ior_dispersion_rgb: [m.ior_red.points(), m.ior_green.points(), m.ior_blue.points()],
        fresnel_f0: m.fresnel_f0.points(), bevel_width_pt: m.bevel_width_pt.points() * sc.scale,
        dome_thickness_pt: m.dome_thickness_pt.points() * sc.scale, vibrancy_chroma_boost: m.vibrancy_chroma_boost.points(),
        meniscus_merge_k_pt: m.meniscus_merge_k_pt.points() * sc.scale, shadow_decay_length_pt: m.bevel_width_pt.points() * sc.scale,
        glare_angle_rad: std::f32::consts::FRAC_PI_4, ..Default::default()
    };
    let stretch_gains = (m.viscous_stretch_gain.points(), m.viscous_press_gain.points());
    let sh_off = vitrea::GlamVec2::new(-params.bevel_width_pt * m.hover_fill_opacity.points(), params.bevel_width_pt * m.active_fill_opacity.points());
    let (fill_a, act_a, hov_a, spec_a) = (m.glass_fill_alpha.points(), m.active_fill_opacity.points(), m.hover_fill_opacity.points(), m.glass_specular_alpha.points());
    let (max_u8, dish_cols) = (f32::from(u8::MAX), (dish_core, dish_rim, accent_rgb, grid_rgb, warm_rgb));
    let skirt_radii = (sc.r_skirt_in, sc.r_skirt_out);

    for y in 0..tex_dim {
        for x in 0..tex_dim {
            let p = vitrea::GlamVec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let d_center = p - sc.hub_center;
            let r = d_center.length();
            if r > sc.r_skirt_out + params.bevel_width_pt * 2.0 {
                continue;
            }
            let outer_grad = if r > 0.0 { d_center / r } else { vitrea::GlamVec2::X };
            let mut best_s = vitrea::SdfSample2d { distance: sc.r_skirt_out, gradient: vitrea::GlamVec2::X };
            let (mut sh_dist, mut best_act, mut best_lift) = (sc.r_skirt_out, 0.0f32, 0.0f32);
            for (idx, mem) in sc.members.iter().enumerate() {
                let r_mid = (mem.radii.0 + mem.radii.1) * 0.5;
                let rad = super::render::deg_to_rad(mem.angles_deg.0, m);
                let c_cent = mem.center + vitrea::GlamVec2::new(rad.cos() * r_mid, rad.sin() * r_mid);
                let p_def = vitrea::deform_viscous_droplet_point(p, c_cent, -mem.deform_vel, (mem.radii.1 - mem.radii.0).max(1.0), mem.lift_weight * 0.5, stretch_gains);
                let s_i = vitrea::sdf_annular_rounded_sector_sample(p_def, mem.center, mem.radii, mem.angles_deg, mem.corner_radius);
                let s_sh = vitrea::sdf_annular_rounded_sector_sample(p_def - sh_off, mem.center, mem.radii, mem.angles_deg, mem.corner_radius);
                sh_dist = sh_dist.min(s_sh.distance);
                if idx == 0 {
                    (best_s, best_act, best_lift) = (s_i, mem.active_weight, mem.lift_weight);
                } else {
                    let pair_w = (best_lift * mem.lift_weight + best_act * mem.active_weight).clamp(0.0, 1.0);
                    if pair_w > 0.0 {
                        let k_m = (params.meniscus_merge_k_pt * pair_w * 0.5).max(params.fresnel_f0);
                        let h = (0.5 + 0.5 * (s_i.distance - best_s.distance) / k_m).clamp(0.0, 1.0);
                        best_s = vitrea::sdf_smooth_union_with_grad(best_s, s_i, k_m);
                        best_act = best_act * h + mem.active_weight * (1.0 - h);
                        best_lift = best_lift * h + mem.lift_weight * (1.0 - h);
                    } else if s_i.distance < best_s.distance {
                        (best_s, best_act, best_lift) = (s_i, mem.active_weight, mem.lift_weight);
                    }
                }
            }
            for (arc_deg, arc_act) in [(sc.undo_arc_deg, sc.undo_active_weight), (sc.redo_arc_deg, sc.redo_active_weight)] {
                let s_arc = vitrea::sdf_curved_stadium_arc_sample(p, sc.hub_center, skirt_radii, arc_deg);
                let s_arc_sh = vitrea::sdf_curved_stadium_arc_sample(p - sh_off, sc.hub_center, skirt_radii, arc_deg);
                sh_dist = sh_dist.min(s_arc_sh.distance);
                if s_arc.distance < best_s.distance {
                    (best_s, best_act, best_lift) = (s_arc, arc_act, 0.0);
                }
            }
            sh_dist = sh_dist.min((p - sh_off - sc.hub_center).length() - sc.r_hub);
            let hub_s = vitrea::SdfSample2d { distance: r - sc.r_hub, gradient: outer_grad };
            let (glass_s, act_w, lift_w) = if hub_s.distance < best_s.distance { (hub_s, 0.0, 0.0) } else { (best_s, best_act.clamp(0.0, 1.0), best_lift.clamp(0.0, 1.0)) };
            let ao_w = (m.active_stroke_width.points() * sc.scale).max(1.0);
            let ao_crease = (1.0 - (glass_s.distance.max(0.0) / ao_w).clamp(0.0, 1.0)).powi(2) * act_a;
            let sh_win = (1.0 - (sh_dist.max(0.0) / params.bevel_width_pt.max(1.0)).clamp(0.0, 1.0)).powi(2);
            let exp_sh = (vitrea::exponential_sdf_attenuation(sh_dist.max(0.0), &params, if is_light { act_a } else { fill_a }) * sh_win + ao_crease).clamp(0.0, fill_a);
            let dish_here = eval_shared_dish_matrix_at(p, sc, dish_cols, 1.0, m);
            let mut floor_rgb = [0, 1, 2].map(|ch| dish_here[ch] * (1.0 - exp_sh) + shadow_rgb[ch] * exp_sh);
            let blue_excess = (dish_here[2] - dish_core[2]).max(0.0);
            if blue_excess > params.fresnel_f0 {
                let spill_w = (blue_excess * 0.5).clamp(0.0, act_a);
                floor_rgb = [0, 1, 2].map(|ch| floor_rgb[ch] * (1.0 - spill_w) + accent_rgb[ch] * spill_w);
            }
            let outer_cov = ((1.0 - (r - sc.r_outer)) * 0.5).clamp(0.0, 1.0);
            let floor_a = if r <= sc.r_outer + 1.0 {
                let outer_rim_s = vitrea::SdfSample2d { distance: r - sc.r_outer, gradient: outer_grad };
                let (f_rim, g_rim) = vitrea::anisotropic_bipolar_rim_glare(outer_rim_s, &params, None);
                let rim_glint = ((f_rim * hov_a + g_rim * spec_a) * outer_cov).clamp(0.0, 1.0);
                floor_rgb = [0, 1, 2].map(|ch| (floor_rgb[ch] + (passive_tint[ch] - floor_rgb[ch]) * rim_glint).clamp(0.0, 1.0));
                let rim_t = ((r - sc.r_dish).max(0.0) / (sc.r_outer - sc.r_dish).max(1.0)).clamp(0.0, 1.0);
                ((act_a + hov_a + rim_t * act_a + exp_sh + rim_glint) * outer_cov).clamp(0.0, fill_a)
            } else {
                let d_ext_bowl = ((p - sh_off - sc.hub_center).length() - sc.r_outer).max(0.0);
                let ext_win = (1.0 - (d_ext_bowl / params.bevel_width_pt.max(1.0)).clamp(0.0, 1.0)).powi(2);
                let bowl_sh = (vitrea::exponential_sdf_attenuation(d_ext_bowl, &params, hov_a) * ext_win).clamp(0.0, hov_a);
                floor_rgb = shadow_rgb;
                bowl_sh.max(exp_sh.clamp(0.0, act_a))
            };
            let surf = vitrea::evaluate_glass_surface_sample(p, glass_s, &params, sc.cursor_pos, sc.r_outer);
            let idx_px = (y * tex_dim + x) * 4;
            if surf.coverage <= 0.0 {
                if floor_a > 0.0 {
                    flat[idx_px..idx_px + 4].copy_from_slice(&[(floor_rgb[0] * max_u8).round() as u8, (floor_rgb[1] * max_u8).round() as u8, (floor_rgb[2] * max_u8).round() as u8, (floor_a * max_u8).round() as u8]);
                }
                continue;
            }
            let ([p_r, p_g, p_b], _, _) = vitrea::refract_bevel_chromatic_rgb(p, glass_s, &params);
            let sharp_rgb = [eval_shared_dish_matrix_at(p_r, sc, dish_cols, 1.0, m)[0], eval_shared_dish_matrix_at(p_g, sc, dish_cols, 1.0, m)[1], eval_shared_dish_matrix_at(p_b, sc, dish_cols, 1.0, m)[2]];
            let blur_rgb = [eval_shared_dish_matrix_at(p_r, sc, dish_cols, 2.0, m)[0], eval_shared_dish_matrix_at(p_g, sc, dish_cols, 2.0, m)[1], eval_shared_dish_matrix_at(p_b, sc, dish_cols, 2.0, m)[2]];
            let active_mix = (act_w + lift_w * 0.5).clamp(0.0, 1.0);
            let tint_rgb = [0, 1, 2].map(|ch| passive_tint[ch] * (1.0 - active_mix) + accent_rgb[ch] * active_mix);
            let tint_a = (hov_a * 0.5) * (1.0 - active_mix) + fill_a * active_mix;
            let lch_rgba = vitrea::compose_dielectric_lch_pixel(sharp_rgb, blur_rgb, [tint_rgb[0], tint_rgb[1], tint_rgb[2], tint_a], &surf, &params, 1.0, spec_a + act_a + act_a * active_mix);
            let s_bev = ((-glass_s.distance) / params.bevel_width_pt.max(1.0)).clamp(0.0, 1.0);
            let rim_crest = (1.0 - (s_bev / (act_a + hov_a)).clamp(0.0, 1.0)).powi(2);
            let pin_w = ((surf.fresnel_specular * spec_a + surf.bipolar_glare_factor * hov_a) * rim_crest).clamp(0.0, 1.0);
            let g_rgb = [0, 1, 2].map(|ch| ((lch_rgba[ch] + (1.0 - lch_rgba[ch]) * pin_w).clamp(0.0, 1.0) * surf.coverage + floor_rgb[ch] * (1.0 - surf.coverage)).clamp(0.0, 1.0));
            let out_a = (surf.coverage + (1.0 - surf.coverage) * floor_a).clamp(0.0, 1.0);
            flat[idx_px..idx_px + 4].copy_from_slice(&[(g_rgb[0] * max_u8).round() as u8, (g_rgb[1] * max_u8).round() as u8, (g_rgb[2] * max_u8).round() as u8, (out_a * max_u8).round() as u8]);
        }
    }
    flat
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
