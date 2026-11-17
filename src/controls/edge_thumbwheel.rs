//! AppKit "Liquid Glass" Top-Down Cylindrical Edge Scroll Wheel with Free-Spin Flywheel Inertia (`MacEdgeThumbwheel`, `SPEC-014 §2.4`).
//!
//! Renders a recessed vertical frosted-glass capsule containing a cylindrical rotary scroll wheel
//! viewed from the top/edge:
//! - Horizontal tactile ridges follow true cylindrical foreshortening $y(\phi) = R \cdot \sin(\phi)$,
//!   appearing wider and brightly illuminated at the center crown and compressing toward the top and bottom edges.
//! - Supports **Ratchet Mode** on slow drags, **Free-Spin Flywheel Inertia** ($\omega(t) = \omega_0 e^{-\lambda t}$)
//!   with velocity boost on fast flicks, and instant **Touch-to-Brake** when touched while spinning.

use crate::theme::MacTheme;
use egui::{Color32, CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};
use std::hash::Hash;
use std::ops::RangeInclusive;

#[derive(Clone, Copy, Default)]
struct EdgeWheelMemory {
    phase_deg: f32,
    velocity_pt_s: f32,
    was_down: bool,
    last_drag_y: Option<f32>,
    last_time: Option<f64>,
}

/// Interaction output returned by [`MacEdgeThumbwheel::show`].
#[derive(Debug, Clone)]
pub struct MacEdgeThumbwheelResponse {
    /// Underlying `egui::Response` of the wheel slot.
    pub response: Response,
    /// Value delta applied on this frame.
    pub value_delta: f32,
    /// Detent tick delta applied on this frame.
    pub delta_ticks: f32,
    /// `true` when the wheel is actively coasting under Free-Spin Flywheel inertia.
    pub is_free_spinning: bool,
    /// Effective Liquid Glass accent color used to render the wheel glow and end-caps (`REQ-014-06`).
    pub effective_glass_accent: Color32,
}

/// Top-Down Cylindrical Edge Scroll Wheel with Free-Spin Flywheel Inertia (`SPEC-014 §2.4`).
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacEdgeThumbwheel, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::DarkAqua);
/// let mut zoom_val = 100.0f32;
///
/// let resp = MacEdgeThumbwheel::new("edge_zoom_wheel", &mut zoom_val, 25.0..=400.0)
///     .theme(&theme)
///     .show(ui);
/// # let _ = resp.is_free_spinning;
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacEdgeThumbwheel<'a> {
    id_source: Id,
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    dock_rect: Option<Rect>,
    wide_drum_width: Option<f32>,
    scale: f32,
    glass_accent: Option<Color32>,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacEdgeThumbwheel<'a> {
    /// Creates a new top-down cylindrical edge thumbwheel bound to `value` over `range`.
    pub fn new(id_source: impl Hash + std::fmt::Debug, value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        Self {
            id_source: Id::NULL.with(id_source),
            value,
            range,
            dock_rect: None,
            wide_drum_width: None,
            scale: 1.0,
            glass_accent: None,
            theme: None,
        }
    }

    /// Overrides the Liquid Glass accent color used for the cylindrical end-cap rings and active flywheel glow (`REQ-014-06`).
    pub fn glass_accent(mut self, color: Color32) -> Self {
        self.glass_accent = Some(color);
        self
    }

    /// Configures a wide cylindrical drum roller whose capsule width matches the `MacContextThumbDial` below it.
    pub fn wide_drum_width(mut self, width_pt: f32) -> Self {
        self.wide_drum_width = Some(width_pt);
        self
    }

    /// Docks the thumbwheel at `target_rect`.
    pub fn dock_right_edge(mut self, canvas_rect: Rect) -> Self {
        self.dock_rect = Some(canvas_rect);
        self
    }

    /// Scales the overall wheel dimensions (`1.0` default).
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.5);
        self
    }

    /// Injects a [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the cylindrical edge thumbwheel and evaluates Free-Spin Flywheel physics.
    pub fn show(self, ui: &mut Ui) -> MacEdgeThumbwheelResponse {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let m = theme.thumb_dial();
        let s = self.scale;

        let w = self.wide_drum_width.unwrap_or_else(|| m.edge_wheel_width.points() * s);
        let h = if self.wide_drum_width.is_some() {
            m.edge_wheel_width.points() * 2.0 * s
        } else {
            m.edge_wheel_height.points() * s
        };
        let desired = Vec2::new(w, h);

        let (rect, mut response) = if let Some(c_rect) = self.dock_rect {
            let margin = m.dock_margin.points() * s;
            let pos = Pos2::new(c_rect.max.x - w - margin, c_rect.min.y + margin * 2.0 + m.undo_bar_height.points());
            let r = Rect::from_min_size(pos, desired);
            let resp = ui.interact(r, self.id_source, Sense::click_and_drag());
            (r, resp)
        } else {
            ui.allocate_exact_size(desired, Sense::click_and_drag())
        };

        let (min_val, max_val) = (*self.range.start(), *self.range.end());
        let span = (max_val - min_val).max(1.0);
        *self.value = self.value.clamp(min_val, max_val);

        let mem_id = self.id_source.with("edge_thumbwheel_state");
        let mut mem = ui.ctx().data_mut(|d| d.get_temp::<EdgeWheelMemory>(mem_id).unwrap_or_default());
        let now = ui.input(|i| i.time);
        let dt = mem
            .last_time
            .map(|prev| (now - prev).clamp(0.0, 0.5) as f32)
            .unwrap_or(0.0);
        mem.last_time = Some(now);

        let is_down = response.is_pointer_button_down_on();
        let pointer_pos = response.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.hover_pos()));
        let mut step_pt = 0.0f32;

        if is_down {
            if !mem.was_down {
                // Touch-to-Brake: immediately zero any active flywheel coasting on finger contact!
                mem.velocity_pt_s = 0.0;
                mem.was_down = true;
                mem.last_drag_y = pointer_pos.map(|p| p.y);
            } else if let (Some(pos), Some(prev_y)) = (pointer_pos, mem.last_drag_y) {
                let dy = prev_y - pos.y;
                mem.last_drag_y = Some(pos.y);
                if dy.abs() > 0.0 {
                    step_pt = dy;
                    if dt > 0.0 {
                        let inst_vel = dy / dt;
                        mem.velocity_pt_s = mem.velocity_pt_s * 0.5 + inst_vel * 0.5;
                    }
                }
            }
        } else if mem.was_down {
            mem.was_down = false;
            mem.last_drag_y = None;
            let flick_thresh = m.edge_wheel_flick_threshold.points();
            if mem.velocity_pt_s.abs() >= flick_thresh {
                let boost = 1.0 + mem.velocity_pt_s.abs() * m.edge_wheel_boost_gain.points();
                mem.velocity_pt_s *= boost;
            } else {
                mem.velocity_pt_s = 0.0;
            }
        } else if mem.velocity_pt_s.abs() > m.edge_wheel_stop_threshold.points() {
            if dt > 0.0 {
                step_pt = mem.velocity_pt_s * dt;
                let decay = (-m.edge_wheel_friction_decay.points() * dt).exp();
                mem.velocity_pt_s *= decay;
                if mem.velocity_pt_s.abs() <= m.edge_wheel_stop_threshold.points() {
                    mem.velocity_pt_s = 0.0;
                }
            }
            if mem.velocity_pt_s.abs() > m.edge_wheel_stop_threshold.points() {
                ui.ctx().request_repaint();
            }
        } else {
            mem.velocity_pt_s = 0.0;
        }

        let mut value_delta = 0.0f32;
        let mut delta_ticks = 0.0f32;
        if step_pt.abs() > 0.0 {
            let phase_step_deg = step_pt * (m.half_circle_deg.points() / h.max(1.0));
            mem.phase_deg = (mem.phase_deg + phase_step_deg) % m.full_circle_deg.points();
            delta_ticks = phase_step_deg / m.detent_step_deg.points().max(1.0);
            let prev = *self.value;
            let next = (prev + (step_pt / h.max(1.0)) * span).clamp(min_val, max_val);
            *self.value = next;
            value_delta = next - prev;
            if value_delta.abs() > 0.0 {
                response.mark_changed();
            }
        }

        let is_free_spinning = !is_down && mem.velocity_pt_s.abs() > m.edge_wheel_stop_threshold.points();
        ui.ctx().data_mut(|d| d.insert_temp(mem_id, mem));

        // Paint Top-Down Cylindrical Edge Thumbwheel in Liquid Glass
        let accent_col = theme.resolve_glass_accent_on(self.glass_accent, theme.popover_background());
        let painter = ui.painter_at(rect.intersect(ui.clip_rect()));
        let outer_cr = CornerRadius::same((m.edge_wheel_corner_radius.points() * s) as u8);
        let pad = m.edge_wheel_slot_padding.points() * s;
        let inner_rect = rect.shrink(pad);
        let inner_cr = CornerRadius::same(((m.edge_wheel_corner_radius.points() * s) - pad).max(1.0) as u8);

        let bw = m.border_stroke_width.points();
        let glass_housing = theme.popover_background().gamma_multiply(m.glass_fill_alpha.points());
        let border_stroke = Stroke::new(bw, theme.separator());
        let specular_col = theme.knob_fill().gamma_multiply(m.glass_specular_alpha.points());
        painter.rect(rect, outer_cr, glass_housing, border_stroke, StrokeKind::Inside);
        painter.line_segment(
            [Pos2::new(rect.min.x + pad, rect.min.y + bw), Pos2::new(rect.max.x - pad, rect.min.y + bw)],
            Stroke::new(bw, specular_col),
        );

        if is_free_spinning || is_down {
            let glow_stroke = Stroke::new(
                m.active_stroke_width.points(),
                accent_col.gamma_multiply(m.active_fill_opacity.points()),
            );
            painter.rect_stroke(inner_rect.expand(1.0), inner_cr, glow_stroke, StrokeKind::Outside);
        }

        painter.rect(inner_rect, inner_cr, theme.window_background(), border_stroke, StrokeKind::Inside);

        // Illuminated Liquid Glass End-Cap Rings on Left and Right Drum Edges
        let cap_w = (m.ring_gap.points() * 2.0 * s).min(inner_rect.width() * m.hover_fill_opacity.points());
        let cap_fill = accent_col.gamma_multiply(m.active_fill_opacity.points());
        let cap_stroke = Stroke::new(m.border_stroke_width.points(), accent_col);
        let left_cap = Rect::from_min_max(inner_rect.min, Pos2::new(inner_rect.min.x + cap_w, inner_rect.max.y));
        let right_cap = Rect::from_min_max(Pos2::new(inner_rect.max.x - cap_w, inner_rect.min.y), inner_rect.max);
        painter.rect(left_cap, inner_cr, cap_fill, cap_stroke, StrokeKind::Inside);
        painter.rect(right_cap, inner_cr, cap_fill, cap_stroke, StrokeKind::Inside);

        let r_cyl = inner_rect.height() * 0.5;
        let y_c = inner_rect.center().y;
        let max_ang = m.edge_wheel_max_angle_deg.points();
        let step_deg = m.detent_step_deg.points().max(1.0);
        let phase_mod = mem.phase_deg.rem_euclid(step_deg);

        let mut base_ang = -max_ang - step_deg;
        while base_ang <= max_ang + step_deg {
            let phi_deg = base_ang + phase_mod;
            if phi_deg.abs() <= max_ang {
                let phi_rad = phi_deg * std::f32::consts::PI / m.half_circle_deg.points();
                let y = y_c - r_cyl * phi_rad.sin();
                let crown_cos = phi_rad.cos().clamp(0.0, 1.0);
                let thickness = m.border_stroke_width.points() + crown_cos * m.active_stroke_width.points();
                let ridge_col = if crown_cos > m.joystick_fast_ratio.points() {
                    theme.knob_fill()
                } else {
                    theme.label().gamma_multiply(crown_cos)
                };
                painter.line_segment(
                    [Pos2::new(inner_rect.min.x + cap_w, y), Pos2::new(inner_rect.max.x - cap_w, y)],
                    Stroke::new(thickness, ridge_col),
                );
            }
            base_ang += step_deg;
        }

        MacEdgeThumbwheelResponse {
            response,
            value_delta,
            delta_ticks,
            is_free_spinning,
            effective_glass_accent: accent_col,
        }
    }
}
