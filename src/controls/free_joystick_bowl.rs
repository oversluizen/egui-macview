//! Standalone AppKit "Liquid Glass" $360^\circ$ Free Joystick Bowl (`MacFreeJoystickBowl`, `SPEC-014 §2.6–§2.7`).
//!
//! Renders an unobstructed $360^\circ$ recessed 3D *Liquid Glass* bowl at the exact same compact
//! $1:1$ circular footprint as [`crate::controls::MacContextThumbDial`], featuring dotted
//! `Fine` and `Fast` speed rings, a **5-layer 3D translucent crystal glass marble**, curved
//! `↶ Undo` / `↷ Redo` crescent buttons, custom `.glass_accent(Color32)` adapted by the
//! Apple calculated vibrancy & contrast system, and [`MacTapticEvent`] micro-click telemetry.

use crate::controls::context_thumb_dial::icons::{paint_annular_wedge, MacTapticEvent, WedgeGeometry};
use crate::controls::context_thumb_dial::render::{angle_in_arc, polar_pos, rad_to_deg, resolve_undo_redo_arcs};
use crate::controls::{MacThumbArcPlacement, MacThumbDockZone};
use crate::theme::MacTheme;
use crate::tokens::ThumbDialMetrics;
use egui::{Align2, Color32, FontId, Id, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use std::hash::Hash;

pub(crate) struct MarblePaintSpec {
    pub(crate) hub_center: Pos2,
    pub(crate) marble_center: Pos2,
    pub(crate) puck_r: f32,
    pub(crate) scale: f32,
    pub(crate) accent_col: Color32,
}

pub(crate) fn paint_3d_recessed_bowl(
    painter: &Painter,
    spec: MarblePaintSpec,
    r_bowl: f32,
    theme: &MacTheme,
    m: &ThumbDialMetrics,
) {
    let s = spec.scale;
    let bw = m.border_stroke_width.points();
    let border_stroke = Stroke::new(bw, theme.separator());
    let specular_col = theme.knob_fill().gamma_multiply(m.glass_specular_alpha.points());
    let bowl_fill = theme.window_background().gamma_multiply(m.glass_fill_alpha.points());

    // 3D Recessed Bowl: inner top shadow ring + base bowl + inner bottom reflection ring
    painter.circle_filled(spec.hub_center, r_bowl + bw, theme.drop_shadow());
    painter.circle(spec.hub_center, r_bowl, bowl_fill, border_stroke);
    painter.circle_stroke(spec.hub_center - Vec2::new(0.0, bw), r_bowl - bw, Stroke::new(m.active_stroke_width.points(), theme.drop_shadow()));
    painter.circle_stroke(spec.hub_center + Vec2::new(0.0, bw), r_bowl - bw, Stroke::new(bw, specular_col));

    // Dotted concentric Fine and Fast speed rings (24 discrete circular dots around 360°)
    let fine_r = r_bowl * m.joystick_fine_ratio.points();
    let fast_r = r_bowl * m.joystick_fast_ratio.points();
    let dot_col = theme.secondary_label_on(theme.window_background()).gamma_multiply(m.active_fill_opacity.points() * 2.0);
    let step_deg = m.detent_step_deg.points().max(1.0);
    let mut deg = 0.0f32;
    while deg < m.full_circle_deg.points() {
        painter.circle_filled(polar_pos(spec.hub_center, fine_r, deg, m), bw * 0.5, dot_col);
        painter.circle_filled(polar_pos(spec.hub_center, fast_r, deg, m), bw * 0.5, dot_col);
        deg += step_deg;
    }

    painter.text(
        spec.hub_center + Vec2::new(0.0, fine_r - m.hint_vertical_offset.points() * s),
        Align2::CENTER_CENTER,
        "Fine",
        FontId::proportional(m.hint_font_size.points() * s),
        theme.secondary_label_on(theme.window_background()),
    );
    painter.text(
        spec.hub_center + Vec2::new(0.0, fast_r - m.hint_vertical_offset.points() * s),
        Align2::CENTER_CENTER,
        "Fast",
        FontId::proportional(m.hint_font_size.points() * s),
        theme.secondary_label_on(theme.window_background()),
    );

    paint_crystal_glass_marble(painter, spec, theme, m);
}

pub(crate) fn paint_crystal_glass_marble(painter: &Painter, spec: MarblePaintSpec, theme: &MacTheme, m: &ThumbDialMetrics) {
    let s = spec.scale;
    let active_stroke = Stroke::new(m.active_stroke_width.points(), spec.accent_col);
    let specular_col = theme.knob_fill().gamma_multiply(m.glass_specular_alpha.points());
    let dot_r = m.latch_dot_radius.points() * s;
    let bw = m.border_stroke_width.points();

    // Luminous vector ray + center socket ring
    painter.line_segment([spec.hub_center, spec.marble_center], Stroke::new(m.active_stroke_width.points() * 2.0, spec.accent_col.gamma_multiply(m.active_fill_opacity.points())));
    painter.line_segment([spec.hub_center, spec.marble_center], active_stroke);
    painter.circle_stroke(spec.hub_center, dot_r * 2.0, Stroke::new(bw, spec.accent_col.gamma_multiply(m.active_fill_opacity.points())));
    painter.circle_filled(spec.hub_center, dot_r, spec.accent_col);

    // 5-Layer 3D Crystal Glass Marble (glazen balletje):
    // 1. Soft multi-ring ambient drop shadow below-right
    painter.circle_filled(spec.marble_center + Vec2::new(dot_r, dot_r * 2.0), spec.puck_r, theme.drop_shadow());
    painter.circle_filled(spec.marble_center + Vec2::new(dot_r * 0.5, dot_r), spec.puck_r, theme.drop_shadow());
    // 2. Refractive translucent glass sphere body
    painter.circle_filled(spec.marble_center, spec.puck_r, spec.accent_col.gamma_multiply(m.hover_fill_opacity.points()));
    // 3. Lower-right internal caustic refraction glow
    let caustic_r = spec.puck_r * m.joystick_fine_ratio.points();
    painter.circle_filled(
        spec.marble_center + Vec2::new(dot_r, dot_r),
        caustic_r,
        spec.accent_col.gamma_multiply(m.active_fill_opacity.points()),
    );
    // 4. Crisp Fresnel outer rim + inner specular refraction ring
    painter.circle_stroke(spec.marble_center, spec.puck_r, active_stroke);
    painter.circle_stroke(spec.marble_center, spec.puck_r - bw, Stroke::new(m.active_stroke_width.points(), specular_col));
    // 5. Upper-left 3D lens specular highlight + secondary micro-glint
    painter.circle_filled(spec.marble_center - Vec2::new(dot_r, dot_r), dot_r, theme.knob_fill());
    painter.circle_filled(spec.marble_center - Vec2::new(dot_r * 2.0, dot_r * 0.5), dot_r * 0.5, specular_col);
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
        let r_clover_out = m.clover_outer_radius.points() * s;
        let r_outer = m.outer_radius.points() * s;
        let r_skirt_in = m.skirt_inner_radius.points() * s;
        let r_skirt_out = m.skirt_outer_radius.points() * s;
        let r_wheel_in = r_clover_out + gap;
        let bw = m.border_stroke_width.points();

        let widget_w = (r_outer + gap) * 2.0;
        let widget_h = (r_skirt_out + gap) * 2.0;
        let desired_size = Vec2::new(widget_w, widget_h);

        let (rect, response) = if let Some((canvas_rect, zone)) = self.dock_target {
            let margin = m.dock_margin.points() * s;
            let min_pos = match zone {
                MacThumbDockZone::BottomLeft => Pos2::new(canvas_rect.min.x + margin, canvas_rect.max.y - widget_h - margin),
                MacThumbDockZone::BottomRight | MacThumbDockZone::Inline => {
                    Pos2::new(canvas_rect.max.x - widget_w - margin, canvas_rect.max.y - widget_h - margin)
                }
            };
            let docked_rect = Rect::from_min_size(min_pos, desired_size);
            let resp = ui.interact(docked_rect, self.id_source, Sense::click_and_drag());
            (docked_rect, resp)
        } else {
            ui.allocate_exact_size(desired_size, Sense::click_and_drag())
        };

        let hub_center = rect.center();
        let ((undo_start, undo_end), (redo_start, redo_end)) = resolve_undo_redo_arcs(self.arc_placement, m);
        let r_skirt_mid = (r_skirt_in + r_skirt_out) * 0.5;
        let undo_center = polar_pos(hub_center, r_skirt_mid, (undo_start + undo_end) * 0.5, m);
        let redo_center = polar_pos(hub_center, r_skirt_mid, (redo_start + redo_end) * 0.5, m);
        let btn_hit_size = Vec2::new((r_skirt_out - r_skirt_in) * 2.0, r_skirt_out - r_skirt_in);
        let undo_rect = Rect::from_center_size(undo_center, btn_hit_size);
        let redo_rect = Rect::from_center_size(redo_center, btn_hit_size);

        let mem_id = self.id_source.with("free_joystick_bowl_state");
        let mut mem = ui.ctx().data_mut(|d| d.get_temp::<FreeJoystickMemory>(mem_id).unwrap_or_default());

        let pointer_pos = response.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.hover_pos()));
        let mut hovered_undo = false;
        let mut hovered_redo = false;
        if let Some(pos) = pointer_pos {
            let delta = pos - hub_center;
            let dist = delta.length();
            let angle_deg = rad_to_deg(delta.y.atan2(delta.x), m);
            if undo_rect.contains(pos) || (dist >= r_outer && dist <= r_skirt_out && angle_in_arc(angle_deg, undo_start, undo_end, m)) {
                hovered_undo = true;
            } else if redo_rect.contains(pos) || (dist >= r_outer && dist <= r_skirt_out && angle_in_arc(angle_deg, redo_start, redo_end, m)) {
                hovered_redo = true;
            }
        }

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
                    let prev_r = mem.joystick_offset.length();
                    let delta = pos - hub_center;
                    let dist = delta.length();
                    let clamped_r = dist.min(r_clover_out);
                    let dir = if dist > 0.0 { delta / dist } else { Vec2::ZERO };
                    mem.joystick_offset = dir * clamped_r;
                    let fine_r = r_clover_out * m.joystick_fine_ratio.points();
                    if (prev_r < fine_r && clamped_r >= fine_r) || (prev_r < r_clover_out && clamped_r >= r_clover_out) {
                        taptic_event = Some(MacTapticEvent::JoystickRingCrossed);
                    }
                    let ratio = (clamped_r / r_clover_out.max(1.0)).clamp(0.0, 1.0);
                    nudge_delta = dir * (ratio * ratio * m.joystick_max_speed.points());
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

        let painter = ui.painter_at(rect.intersect(ui.clip_rect()));
        let border_stroke = Stroke::new(bw, theme.separator());
        let active_stroke = Stroke::new(m.active_stroke_width.points(), accent_col);
        let specular_col = theme.knob_fill().gamma_multiply(m.glass_specular_alpha.points());
        let glass_bg = theme.popover_background().gamma_multiply(m.glass_fill_alpha.points());

        painter.circle_filled(hub_center + Vec2::new(0.0, gap * 2.0), r_outer + bw, theme.drop_shadow());
        painter.circle_filled(hub_center + Vec2::new(0.0, gap), r_outer, theme.drop_shadow());
        painter.circle(hub_center, r_outer, glass_bg, border_stroke);
        painter.circle_stroke(hub_center, r_outer - bw, Stroke::new(m.active_stroke_width.points(), specular_col));

        let wheel_bg = theme.box_background().gamma_multiply(m.glass_fill_alpha.points());
        paint_annular_wedge(
            &painter,
            WedgeGeometry { center: hub_center, r_in: r_wheel_in, r_out: r_outer, start_deg: 0.0, end_deg: m.full_circle_deg.points() },
            wheel_bg,
            border_stroke,
            m,
        );

        let marble_center = hub_center + mem.joystick_offset;
        let puck_r = m.joystick_puck_radius.points() * s;
        paint_3d_recessed_bowl(
            &painter,
            MarblePaintSpec { hub_center, marble_center, puck_r, scale: s, accent_col },
            r_clover_out,
            theme,
            m,
        );

        for (start_d, end_d, lbl, enabled, hovered) in [
            (undo_start, undo_end, "↶ Undo", self.can_undo, hovered_undo),
            (redo_start, redo_end, "↷ Redo", self.can_redo, hovered_redo),
        ] {
            let fill = if hovered && enabled {
                accent_col.gamma_multiply(m.active_fill_opacity.points())
            } else {
                theme.control_background().gamma_multiply(m.glass_fill_alpha.points())
            };
            let stroke = if hovered && enabled { active_stroke } else { Stroke::new(bw, specular_col) };
            paint_annular_wedge(
                &painter,
                WedgeGeometry { center: hub_center, r_in: r_skirt_in, r_out: r_skirt_out, start_deg: start_d, end_deg: end_d },
                fill,
                stroke,
                m,
            );
            let mid_pt = polar_pos(hub_center, r_skirt_mid, (start_d + end_d) * 0.5, m);
            let fg = if enabled { theme.label() } else { theme.tertiary_label() };
            painter.text(mid_pt, Align2::CENTER_CENTER, lbl, FontId::proportional(m.label_font_size.points() * s), fg);
        }

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
