//! Standalone AppKit "Liquid Glass" $360^\circ$ Free Joystick Bowl (`MacFreeJoystickBowl`, `SPEC-014 §2.6–§2.7`).
//!
//! Renders an unobstructed $360^\circ$ recessed 3D *Liquid Glass* bowl at the exact same compact
//! $1:1$ circular footprint as [`crate::controls::MacContextThumbDial`], featuring dotted
//! `Fine` and `Fast` speed rings, a **5-layer 3D translucent crystal glass marble**, curved
//! `↶ Undo` / `↷ Redo` crescent buttons, custom `.glass_accent(Color32)` adapted by the
//! Apple calculated vibrancy & contrast system, and [`MacTapticEvent`] micro-click telemetry.

use crate::controls::context_thumb_dial::icons::MacTapticEvent;
use crate::controls::context_thumb_dial::paint::{paint_undo_redo_crescent_arcs, UndoRedoPaintStyle};
use crate::controls::context_thumb_dial::render::{
    allocate_docked_thumb_rect, compute_undo_redo_hit_rects, deg_to_rad, hit_test_undo_redo_arcs, polar_pos, step_joystick_bowl_physics,
};
use crate::controls::{MacThumbArcPlacement, MacThumbDockZone};
use crate::theme::{MacAppearance, MacTheme};
use crate::tokens::ThumbDialMetrics;
use egui::{Align2, Color32, FontId, Id, Painter, Pos2, Rect, Response, Shape, Stroke, Ui, Vec2};
use std::hash::Hash;

pub(crate) struct MarblePaintSpec {
    pub(crate) hub_center: Pos2,
    pub(crate) marble_center: Pos2,
    pub(crate) puck_r: f32,
    pub(crate) scale: f32,
    pub(crate) accent_col: Color32,
}

pub(crate) fn paint_3d_recessed_bowl(painter: &Painter, spec: MarblePaintSpec, r_bowl: f32, theme: &MacTheme, m: &ThumbDialMetrics) {
    let s = spec.scale;
    let bw = m.border_stroke_width.points();
    let border_stroke = Stroke::new(bw, theme.separator());
    let specular_col = theme.specular_rim_top();
    let is_light = theme.appearance == MacAppearance::Aqua;
    let bowl_core = if is_light { theme.window_background() } else { theme.window_background().gamma_multiply(m.glass_fill_alpha.points()) };
    let bowl_rim = if is_light { theme.box_background() } else { theme.control_background().gamma_multiply(m.glass_fill_alpha.points()) };

    painter.circle_filled(spec.hub_center, r_bowl + bw, theme.drop_shadow().gamma_multiply(0.5));
    paint_radial_disc_mesh(painter, spec.hub_center, r_bowl, bowl_core, bowl_rim, m);
    painter.circle_stroke(spec.hub_center, r_bowl, border_stroke);
    painter.circle_stroke(
        spec.hub_center - Vec2::new(0.0, bw),
        r_bowl - bw,
        Stroke::new(m.active_stroke_width.points(), theme.drop_shadow().gamma_multiply(0.5)),
    );
    painter.circle_stroke(spec.hub_center + Vec2::new(0.0, bw), r_bowl - bw, Stroke::new(bw, specular_col));

    let fine_r = r_bowl * m.joystick_fine_ratio.points();
    let fast_r = r_bowl * m.joystick_fast_ratio.points();
    let slow_r = fine_r * 0.5 + fast_r * 0.5;
    let dot_col = theme.secondary_label_on(bowl_core).gamma_multiply(m.active_fill_opacity.points() * 2.0);
    let step_deg = (m.detent_step_deg.points() * 0.5).max(1.0);
    let mut deg = 0.0f32;
    while deg < m.full_circle_deg.points() {
        painter.circle_filled(polar_pos(spec.hub_center, fine_r, deg, m), bw, dot_col);
        painter.circle_filled(polar_pos(spec.hub_center, slow_r, deg, m), bw, dot_col);
        painter.circle_filled(polar_pos(spec.hub_center, fast_r, deg, m), bw, dot_col);
        deg += step_deg;
    }

    painter.text(
        spec.hub_center + Vec2::new(0.0, fine_r - m.hint_vertical_offset.points() * s),
        Align2::CENTER_CENTER,
        "Fine",
        FontId::proportional(m.hint_font_size.points() * s),
        theme.secondary_label_on(bowl_core),
    );
    painter.text(
        spec.hub_center + Vec2::new(0.0, fast_r - m.hint_vertical_offset.points() * s),
        Align2::CENTER_CENTER,
        "Fast",
        FontId::proportional(m.hint_font_size.points() * s),
        theme.secondary_label_on(bowl_core),
    );

    paint_crystal_glass_marble(painter, spec, theme, m);
}

fn paint_radial_disc_mesh(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    core_col: Color32,
    rim_col: Color32,
    m: &ThumbDialMetrics,
) {
    let steps = 32usize;
    let full_deg = m.full_circle_deg.points();
    let mut mesh = egui::epaint::Mesh::default();
    mesh.vertices.reserve(steps + 2);
    mesh.indices.reserve(steps * 3);
    mesh.colored_vertex(center, core_col);
    for i in 0..=steps {
        let a = full_deg * (i as f32 / steps as f32);
        let pt = polar_pos(center, radius, a, m);
        mesh.colored_vertex(pt, rim_col);
    }
    for i in 0..steps {
        let idx = (i + 1) as u32;
        mesh.add_triangle(0, idx, idx + 1);
    }
    painter.add(Shape::mesh(mesh));
}

pub(crate) fn paint_crystal_glass_marble(painter: &Painter, spec: MarblePaintSpec, theme: &MacTheme, m: &ThumbDialMetrics) {
    let s = spec.scale;
    let active_stroke = Stroke::new(m.active_stroke_width.points(), spec.accent_col);
    let specular_col = theme.specular_rim_top();
    let dot_r = m.latch_dot_radius.points() * s;
    let bw = m.border_stroke_width.points();

    painter.line_segment(
        [spec.hub_center, spec.marble_center],
        Stroke::new(m.active_stroke_width.points() * 2.0, spec.accent_col.gamma_multiply(m.hover_fill_opacity.points())),
    );
    painter.line_segment([spec.hub_center, spec.marble_center], active_stroke);
    painter.circle_stroke(spec.hub_center, dot_r, Stroke::new(bw, theme.secondary_label()));
    painter.circle_filled(spec.hub_center, dot_r * 0.5, theme.window_background());

    // 1. Multi-layer soft ambient drop shadow + lower-right blue caustic floor transmission
    let sh_off = Vec2::new(dot_r * 2.0, dot_r * 2.0);
    paint_radial_disc_mesh(
        painter,
        spec.marble_center + sh_off,
        spec.puck_r + dot_r * 2.0,
        spec.accent_col.gamma_multiply(m.active_fill_opacity.points()),
        Color32::TRANSPARENT,
        m,
    );
    paint_radial_disc_mesh(
        painter,
        spec.marble_center + Vec2::new(0.0, dot_r),
        spec.puck_r + dot_r,
        theme.drop_shadow().gamma_multiply(m.glass_fill_alpha.points()),
        Color32::TRANSPARENT,
        m,
    );

    // 2. 3D Spherical Refractive Crystal Mesh (4 concentric radial rings x 24 angular segments)
    let is_light = theme.appearance == MacAppearance::Aqua;
    let rings = 4usize;
    let segs = 24usize;
    let full_deg = m.full_circle_deg.points();
    let mut sphere = egui::epaint::Mesh::default();
    sphere.vertices.reserve((rings + 1) * (segs + 1));
    sphere.indices.reserve(rings * segs * 6);
    for r_i in 0..=rings {
        let r_frac = r_i as f32 / rings as f32;
        let rad = spec.puck_r * r_frac;
        let nz = (1.0 - r_frac * r_frac).max(0.0).sqrt();
        let fresnel = (1.0 - nz) * (1.0 - nz);
        for s_i in 0..=segs {
            let a_deg = full_deg * (s_i as f32 / segs as f32);
            let a_rad = deg_to_rad(a_deg, m);
            let (nx, ny) = (a_rad.cos() * r_frac, a_rad.sin() * r_frac);
            let caustic = ((nx + ny) * 0.5).clamp(0.0, 1.0) * r_frac;
            let spec_glint = ((-nx - ny) * 0.5).clamp(0.0, 1.0) * (1.0 - r_frac * 0.5);
            let mut h = egui::epaint::Hsva::from(spec.accent_col);
            let base_s = if is_light { m.hover_fill_opacity.points() } else { m.active_fill_opacity.points() };
            h.s = (h.s * (base_s + caustic * m.glass_fill_alpha.points() + fresnel * 0.5)).clamp(0.0, 1.0);
            h.v = if is_light {
                (m.glass_fill_alpha.points() + (1.0 - m.glass_fill_alpha.points()) * (spec_glint + caustic * 0.5)).clamp(0.0, 1.0)
            } else {
                (h.v + (1.0 - h.v) * (spec_glint * m.glass_fill_alpha.points() + fresnel * m.active_fill_opacity.points())).clamp(0.0, 1.0)
            };
            h.a = (m.active_fill_opacity.points() + fresnel * m.glass_fill_alpha.points() + caustic * 0.5).clamp(0.0, 1.0);
            let pt = spec.marble_center + Vec2::new(a_rad.cos() * rad, a_rad.sin() * rad);
            sphere.colored_vertex(pt, Color32::from(h));
        }
    }
    let stride = (segs + 1) as u32;
    for r_i in 0..rings {
        for s_i in 0..segs {
            let b0 = (r_i as u32) * stride + (s_i as u32);
            let b1 = b0 + stride;
            sphere.add_triangle(b0, b1, b1 + 1);
            sphere.add_triangle(b0, b1 + 1, b0 + 1);
        }
    }
    painter.add(Shape::mesh(sphere));

    // 3. Lower-right internal cyan caustic crescent & outer Fresnel rim
    let caustic_c = spec.marble_center + Vec2::new(dot_r, dot_r);
    paint_radial_disc_mesh(
        painter,
        caustic_c,
        spec.puck_r * m.joystick_fine_ratio.points(),
        spec.accent_col.gamma_multiply(m.glass_fill_alpha.points()),
        Color32::TRANSPARENT,
        m,
    );
    painter.circle_stroke(spec.marble_center, spec.puck_r, Stroke::new(bw, spec.accent_col.gamma_multiply(m.glass_fill_alpha.points())));
    painter.circle_stroke(spec.marble_center, spec.puck_r - bw, Stroke::new(m.active_stroke_width.points(), specular_col));

    // 4. Upper-left Blinn-Phong primary specular lens reflection + secondary glint
    let hi_step = dot_r + dot_r * 0.5;
    let hi_c = spec.marble_center - Vec2::new(hi_step, hi_step);
    paint_radial_disc_mesh(painter, hi_c, dot_r * 2.0, theme.knob_fill(), Color32::TRANSPARENT, m);
    painter.circle_filled(hi_c, dot_r, theme.knob_fill());
}

#[derive(Clone, Copy, Default)]
struct FreeJoystickMemory {
    was_pressed: bool,
    is_dragging_bowl: bool,
    joystick_offset: Vec2,
}

/// Interaction output returned by [`MacFreeJoystickBowl::show`].
#[derive(Debug, Clone)]
pub struct MacFreeJoystickResponse {
    /// Underlying `egui::Response` of the joystick bowl widget.
    pub response: Response,
    /// Continuous $360^\circ$ point-nudge vector (`(dx, dy)`) scaled quadratically from center (`Fine`) to rim (`Fast`).
    pub nudge_delta: Vec2,
    /// Current screen-space center of the 3D crystal glass marble.
    pub marble_center: Pos2,
    /// Screen-space center of the bowl hub.
    pub hub_center: Pos2,
    /// `true` if the curved `↶ Undo` crescent button was clicked.
    pub undo_clicked: bool,
    /// `true` if the curved `↷ Redo` crescent button was clicked.
    pub redo_clicked: bool,
    /// Screen-space hit rect of the curved `↶ Undo` button.
    pub undo_rect: Rect,
    /// Screen-space hit rect of the curved `↷ Redo` button.
    pub redo_rect: Rect,
    /// Effective Liquid Glass accent color resolved by the Apple calculated color system (`REQ-014-06`).
    pub effective_glass_accent: Color32,
    /// Semantic Haptic / Auditory Micro-Click event emitted on this frame (`REQ-014-07`).
    pub taptic_event: Option<MacTapticEvent>,
}

/// Standalone $360^\circ$ Liquid Glass Free Joystick Bowl with a 3D Crystal Glass Marble (`SPEC-014 §2.6–§2.7`).
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacFreeJoystickBowl, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::DarkAqua);
/// let resp = MacFreeJoystickBowl::new("standalone_joystick")
///     .glass_accent(theme.system_green())
///     .theme(&theme)
///     .show(ui);
/// # let _ = resp.nudge_delta;
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacFreeJoystickBowl<'a> {
    id_source: Id,
    arc_placement: MacThumbArcPlacement,
    can_undo: bool,
    can_redo: bool,
    dock_target: Option<(Rect, MacThumbDockZone)>,
    scale: f32,
    glass_accent: Option<Color32>,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacFreeJoystickBowl<'a> {
    /// Creates a new standalone $360^\circ$ Liquid Glass Free Joystick Bowl.
    pub fn new(id_source: impl Hash + std::fmt::Debug) -> Self {
        Self {
            id_source: Id::NULL.with(id_source),
            arc_placement: MacThumbArcPlacement::TopCrown,
            can_undo: true,
            can_redo: true,
            dock_target: None,
            scale: 1.0,
            glass_accent: None,
            theme: None,
        }
    }

    /// Overrides the Liquid Glass accent color (adapted through `MacTheme::resolve_glass_accent_on` in the Apple calculated color system, `REQ-014-06`).
    pub fn glass_accent(mut self, color: Color32) -> Self {
        self.glass_accent = Some(color);
        self
    }

    /// Configures the placement of the curved `↶ Undo` and `↷ Redo` crescent buttons (defaults to [`MacThumbArcPlacement::TopCrown`]).
    pub fn undo_arc_placement(mut self, placement: MacThumbArcPlacement) -> Self {
        self.arc_placement = placement;
        self
    }

    /// Enables or disables the curved `↶ Undo` button.
    pub fn can_undo(mut self, can_undo: bool) -> Self {
        self.can_undo = can_undo;
        self
    }

    /// Enables or disables the curved `↷ Redo` button.
    pub fn can_redo(mut self, can_redo: bool) -> Self {
        self.can_redo = can_redo;
        self
    }

    /// Docks the joystick bowl in the `BottomLeft` or `BottomRight` thumb zone of `canvas_rect`.
    pub fn dock_in_rect(mut self, canvas_rect: Rect, zone: MacThumbDockZone) -> Self {
        self.dock_target = Some((canvas_rect, zone));
        self
    }

    /// Scales the overall radial dimensions (`1.0` default).
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.5);
        self
    }

    /// Injects a [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the standalone $360^\circ$ Liquid Glass Free Joystick Bowl and processes pointer/touch gestures.
    pub fn show(self, ui: &mut Ui) -> MacFreeJoystickResponse {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let m = theme.thumb_dial();
        let s = self.scale;
        let accent_col = theme.resolve_glass_accent_on(self.glass_accent, theme.popover_background());

        let gap = m.ring_gap.points() * s;
        let r_outer = m.outer_radius.points() * s;
        let r_clover_out = r_outer - gap * 2.0;
        let r_skirt_in = m.skirt_inner_radius.points() * s;
        let r_skirt_out = m.skirt_outer_radius.points() * s;
        let bw = m.border_stroke_width.points();

        let desired_size = Vec2::new((r_outer + gap) * 2.0, (r_skirt_out + gap) * 2.0);
        let (rect, response) = allocate_docked_thumb_rect(ui, self.id_source, self.dock_target, desired_size, m, s);

        let hub_center = rect.center();
        let (arcs, undo_rect, redo_rect) = compute_undo_redo_hit_rects(hub_center, self.arc_placement, r_skirt_in, r_skirt_out, m);

        let mem_id = self.id_source.with("free_joystick_bowl_state");
        let mut mem = ui.ctx().data_mut(|d| d.get_temp::<FreeJoystickMemory>(mem_id).unwrap_or_default());

        let now = ui.input(|i| i.time);
        let pointer_pos = response
            .interact_pointer_pos()
            .or_else(|| ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.latest_pos())));
        let (hovered_undo, hovered_redo) = pointer_pos
            .map(|pos| hit_test_undo_redo_arcs(pos, hub_center, (r_outer, r_skirt_out), arcs, (undo_rect, redo_rect), m))
            .unwrap_or((false, false));

        let is_down = response.is_pointer_button_down_on();
        let mut nudge_delta = Vec2::ZERO;
        let mut undo_clicked = false;
        let mut redo_clicked = false;
        let mut taptic_event = None;

        if is_down {
            mem.was_pressed = true;
            if let Some(pos) = pointer_pos {
                if !hovered_undo && !hovered_redo {
                    mem.is_dragging_bowl = true;
                }
                if mem.is_dragging_bowl {
                    let delta = pos - hub_center;
                    let (nudge, tap) = step_joystick_bowl_physics(&mut mem.joystick_offset, delta, delta.length(), r_clover_out, m);
                    nudge_delta = nudge;
                    taptic_event = tap;
                }
            }
        } else if mem.was_pressed {
            if !mem.is_dragging_bowl {
                if hovered_undo && self.can_undo {
                    undo_clicked = true;
                    taptic_event = Some(MacTapticEvent::HistoryStep { is_redo: false });
                } else if hovered_redo && self.can_redo {
                    redo_clicked = true;
                    taptic_event = Some(MacTapticEvent::HistoryStep { is_redo: true });
                }
            }
            mem = FreeJoystickMemory::default();
        }
        ui.ctx().data_mut(|d| d.insert_temp(mem_id, mem));

        let base_spring = theme.tokens().macview.motion.swiftui_snappy.to_spring();
        let return_spring = crate::motion::CASpring::new(base_spring.response * 0.5, base_spring.damping_ratio);
        let sx_id = self.id_source.with("jb_ret_x");
        let sy_id = self.id_source.with("jb_ret_y");
        let animated_offset = if is_down && mem.is_dragging_bowl {
            ui.ctx().data_mut(|d| {
                d.insert_temp(sx_id, (mem.joystick_offset.x, mem.joystick_offset.x, now));
                d.insert_temp(sy_id, (mem.joystick_offset.y, mem.joystick_offset.y, now));
            });
            mem.joystick_offset
        } else {
            Vec2::new(
                return_spring.animate_f32_at(ui.ctx(), sx_id, 0.0, now, false),
                return_spring.animate_f32_at(ui.ctx(), sy_id, 0.0, now, false),
            )
        };

        let painter = ui.painter_at(rect.intersect(ui.clip_rect()));
        let border_stroke = Stroke::new(bw, theme.separator());
        let specular_col = theme.specular_rim_top();
        let is_light = theme.appearance == MacAppearance::Aqua;
        let glass_bg = if is_light {
            theme.box_background()
        } else {
            theme.box_background().gamma_multiply(m.glass_fill_alpha.points())
        };

        painter.circle_filled(hub_center + Vec2::new(0.0, gap * 2.0), r_outer + bw, theme.drop_shadow());
        painter.circle_filled(hub_center + Vec2::new(0.0, gap), r_outer, theme.drop_shadow());
        painter.circle(hub_center, r_outer, glass_bg, border_stroke);
        painter.circle_stroke(hub_center, r_outer - bw, Stroke::new(m.active_stroke_width.points(), specular_col));

        let marble_center = hub_center + animated_offset;
        let puck_r = m.joystick_puck_radius.points() * (s + s * 0.5);
        paint_3d_recessed_bowl(
            &painter,
            MarblePaintSpec { hub_center, marble_center, puck_r, scale: s, accent_col },
            r_clover_out,
            theme,
            m,
        );

        paint_undo_redo_crescent_arcs(
            &painter,
            theme,
            m,
            (hub_center, r_skirt_in, r_skirt_out, s),
            arcs,
            UndoRedoPaintStyle { can_undo: self.can_undo, can_redo: self.can_redo, hovered_undo, hovered_redo, accent_col },
        );

        MacFreeJoystickResponse {
            response,
            nudge_delta,
            marble_center,
            hub_center,
            undo_clicked,
            redo_clicked,
            undo_rect,
            redo_rect,
            effective_glass_accent: accent_col,
            taptic_event,
        }
    }
}
