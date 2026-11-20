//! AppKit "Liquid Glass" Top-Down Cylindrical Edge Scroll Wheel with Free-Spin Flywheel Inertia (`MacEdgeThumbwheel`, `SPEC-014 §2.4`).
//!
//! Renders a recessed vertical frosted-glass capsule containing a cylindrical rotary scroll wheel
//! viewed from the top/edge:
//! - Horizontal tactile ridges follow true cylindrical foreshortening $y(\phi) = R \cdot \sin(\phi)$,
//!   appearing wider and brightly illuminated at the center crown and compressing toward the top and bottom edges.
//! - Supports **Ratchet Mode** on slow drags, **Free-Spin Flywheel Inertia** ($\omega(t) = \omega_0 e^{-\lambda t}$)
//!   with velocity boost on fast flicks, and instant **Touch-to-Brake** when touched while spinning.

use crate::controls::context_thumb_dial::icons::MacTapticEvent;
use crate::theme::MacTheme;
use crate::tokens::ThumbDialMetrics;
use egui::{Color32, ColorImage, CornerRadius, Id, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};
use std::hash::{Hash, Hasher};
use std::ops::RangeInclusive;

struct DrumMatrixSpec {
    tex_w: usize,
    tex_h: usize,
    r_cyl: f32,
    cap_w: f32,
    corner_r: f32,
    ptr_bias: f32,
    is_active: bool,
    accent_col: Color32,
}

fn color_to_rgb_norm(c: Color32) -> [f32; 3] {
    let [r, g, b, _] = c.to_array();
    let max_u8 = f32::from(u8::MAX);
    [f32::from(r) / max_u8, f32::from(g) / max_u8, f32::from(b) / max_u8]
}

fn eval_drum_backdrop_at(
    p: vitrea::GlamVec2,
    ds: &DrumMatrixSpec,
    (drum_rgb, accent_rgb): ([f32; 3], [f32; 3]),
    m: &ThumbDialMetrics,
) -> [f32; 3] {
    let (w, h) = (ds.tex_w as f32, ds.tex_h as f32);
    let y_frac = (p.y / h.max(1.0)).clamp(0.0, 1.0);
    let u_frac = (p.x / w.max(1.0)).clamp(0.0, 1.0);
    let phi = (y_frac * 2.0 - 1.0) * std::f32::consts::FRAC_PI_2;
    let c_val = phi.cos().clamp(0.0, 1.0);
    let (act_a, hov_a, spec_a) = (m.active_fill_opacity.points(), m.hover_fill_opacity.points(), m.glass_specular_alpha.points());
    let spec_dist = (y_frac - (0.5 - hov_a + ds.ptr_bias * act_a)).abs();
    let spec_band = (1.0 - spec_dist / act_a.max(0.5)).clamp(0.0, 1.0);
    let axial = (1.0 - (u_frac * 2.0 - 1.0).abs()).clamp(0.0, 1.0);
    let rim_dark = (1.0 - c_val * c_val) * spec_a;
    let hi_gain = (spec_band * spec_band * (0.5 + 0.5 * axial) * act_a).clamp(0.0, 1.0);
    let groove_d = (p.x - ds.cap_w).abs().min((w - ds.cap_w - p.x).abs());
    let groove_dark = (1.0 - (groove_d / (ds.cap_w * 0.5).max(1.0)).clamp(0.0, 1.0)).powi(2) * act_a;
    let tot_dark = (rim_dark + groove_dark).clamp(0.0, 1.0);
    let mut rgb = [0, 1, 2].map(|ch| (drum_rgb[ch] * (1.0 - tot_dark) + (1.0 - drum_rgb[ch]) * hi_gain).clamp(0.0, 1.0));
    let cap_bleed = ((1.0 - (p.x - ds.cap_w).max(0.0) / (ds.cap_w * 2.0).max(1.0)).clamp(0.0, 1.0))
        .max((1.0 - (w - ds.cap_w - p.x).max(0.0) / (ds.cap_w * 2.0).max(1.0)).clamp(0.0, 1.0));
    let bleed_w = cap_bleed * cap_bleed * (if ds.is_active { act_a } else { hov_a });
    for ch in 0..3 {
        rgb[ch] = (rgb[ch] * (1.0 - bleed_w) + accent_rgb[ch] * bleed_w).clamp(0.0, 1.0);
    }
    rgb
}

fn synthesize_drum_matrix_rgba(ds: &DrumMatrixSpec, theme: &MacTheme, m: &ThumbDialMetrics) -> Vec<u8> {
    let mut flat = vec![0u8; ds.tex_w * ds.tex_h * 4];
    let is_light = theme.appearance == crate::theme::MacAppearance::Aqua;
    let drum_rgb = color_to_rgb_norm(if is_light { theme.window_background() } else { theme.control_background() });
    let accent_rgb = color_to_rgb_norm(ds.accent_col);
    let spec_rgb = color_to_rgb_norm(theme.specular_rim_top());
    let (w, h) = (ds.tex_w as f32, ds.tex_h as f32);
    let center = vitrea::GlamVec2::new(w * 0.5, h * 0.5);
    let half_ext = vitrea::GlamVec2::new(w * 0.5, h * 0.5);
    let cap_half = vitrea::GlamVec2::new(ds.cap_w * 0.5, h * 0.5);
    let left_c = vitrea::GlamVec2::new(ds.cap_w * 0.5, h * 0.5);
    let right_c = vitrea::GlamVec2::new(w - ds.cap_w * 0.5, h * 0.5);
    let params = vitrea::LiquidGlassOpticalParams {
        ior_glass: m.ior_glass.points(),
        ior_dispersion_rgb: [m.ior_red.points(), m.ior_green.points(), m.ior_blue.points()],
        fresnel_f0: m.fresnel_f0.points(),
        bevel_width_pt: (ds.cap_w * 0.5).max(2.0),
        dome_thickness_pt: m.dome_thickness_pt.points(),
        vibrancy_chroma_boost: m.vibrancy_chroma_boost.points(),
        meniscus_merge_k_pt: m.meniscus_merge_k_pt.points(),
    };
    let (fill_a, act_a, hov_a, spec_a) = (m.glass_fill_alpha.points(), m.active_fill_opacity.points(), m.hover_fill_opacity.points(), m.glass_specular_alpha.points());
    let max_u8 = f32::from(u8::MAX);
    let cursor_pt = Some(vitrea::GlamVec2::new(w * 0.5, h * (0.5 + ds.ptr_bias)));

    for y in 0..ds.tex_h {
        for x in 0..ds.tex_w {
            let p = vitrea::GlamVec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let s_drum = vitrea::sdf_rounded_capsule_sample(p, center, half_ext, ds.corner_r);
            let surf_drum = vitrea::evaluate_glass_surface_sample(p, s_drum, &params, cursor_pt, ds.r_cyl * 2.0);
            if surf_drum.coverage <= 0.0 {
                continue;
            }
            let s_l = vitrea::sdf_rounded_capsule_sample(p, left_c, cap_half, ds.corner_r.min(ds.cap_w * 0.5));
            let s_r = vitrea::sdf_rounded_capsule_sample(p, right_c, cap_half, ds.corner_r.min(ds.cap_w * 0.5));
            let s_cap = if s_l.distance < s_r.distance { s_l } else { s_r };
            let surf_cap = vitrea::evaluate_glass_surface_sample(p, s_cap, &params, cursor_pt, ds.r_cyl * 2.0);
            let [p_r, p_g, p_b] = if surf_cap.coverage > 0.0 { surf_cap.refracted_rgb_pts } else { surf_drum.refracted_rgb_pts };
            let mut rgb = [
                eval_drum_backdrop_at(p_r, ds, (drum_rgb, accent_rgb), m)[0],
                eval_drum_backdrop_at(p_g, ds, (drum_rgb, accent_rgb), m)[1],
                eval_drum_backdrop_at(p_b, ds, (drum_rgb, accent_rgb), m)[2],
            ];
            let drum_rim = surf_drum.fresnel_specular * (if is_light { hov_a } else { act_a });
            for ch in 0..3 {
                rgb[ch] = (rgb[ch] + drum_rim * spec_rgb[ch]).clamp(0.0, 1.0);
            }
            if surf_cap.coverage > 0.0 {
                let cap_w_mix = (if ds.is_active { fill_a } else { act_a }) * surf_cap.coverage;
                let cap_rim = surf_cap.fresnel_specular * spec_a;
                let cap_dome = surf_cap.key_illumination * hov_a;
                for ch in 0..3 {
                    let c_body = (accent_rgb[ch] * (fill_a + surf_cap.dome_elevation * act_a) + cap_dome + cap_rim * spec_rgb[ch]).clamp(0.0, 1.0);
                    rgb[ch] = (rgb[ch] * (1.0 - cap_w_mix) + c_body * cap_w_mix).clamp(0.0, 1.0);
                }
            }
            let idx = (y * ds.tex_w + x) * 4;
            flat[idx] = (rgb[0] * max_u8).round() as u8;
            flat[idx + 1] = (rgb[1] * max_u8).round() as u8;
            flat[idx + 2] = (rgb[2] * max_u8).round() as u8;
            flat[idx + 3] = (surf_drum.coverage * max_u8).round() as u8;
        }
    }
    flat
}

fn paint_raytraced_edge_thumbwheel_quad(
    painter: &Painter,
    id_source: Id,
    inner_rect: Rect,
    (cap_w, corner_r, ptr_bias, is_active): (f32, f32, f32, bool),
    accent_col: Color32,
    theme: &MacTheme,
    m: &ThumbDialMetrics,
) {
    let px_scale = 2.0f32;
    let tex_w = ((inner_rect.width() * px_scale).round() as usize).max(2);
    let tex_h = ((inner_rect.height() * px_scale).round() as usize).max(2);
    let max_u8 = f32::from(u8::MAX);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    (tex_w, tex_h, is_active).hash(&mut hasher);
    (theme.appearance == crate::theme::MacAppearance::Aqua).hash(&mut hasher);
    accent_col.to_array().hash(&mut hasher);
    ((ptr_bias * max_u8).round() as i32).hash(&mut hasher);
    let state_hash = hasher.finish();
    let cache_id = id_source.with("lg_drum_ray_tex");
    let ctx = painter.ctx();
    let cached = ctx.data(|d| d.get_temp::<(u64, egui::TextureHandle)>(cache_id));
    let tex_handle = match cached {
        Some((prev_h, handle)) if prev_h == state_hash => handle,
        _ => {
            let ds = DrumMatrixSpec {
                tex_w,
                tex_h,
                r_cyl: inner_rect.height() * 0.5 * px_scale,
                cap_w: cap_w * px_scale,
                corner_r: corner_r * px_scale,
                ptr_bias,
                is_active,
                accent_col,
            };
            let flat = synthesize_drum_matrix_rgba(&ds, theme, m);
            let img = ColorImage::from_rgba_unmultiplied([tex_w, tex_h], &flat);
            let handle = ctx.load_texture("vitrea_drum_lg", img, egui::TextureOptions::LINEAR);
            ctx.data_mut(|d| d.insert_temp(cache_id, (state_hash, handle.clone())));
            handle
        }
    };
    let mut mesh = egui::epaint::Mesh::with_texture(tex_handle.id());
    let tint = Color32::from(egui::epaint::Hsva::new(0.0, 0.0, 1.0, 1.0));
    mesh.add_rect_with_uv(inner_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), tint);
    painter.add(Shape::mesh(mesh));
}

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
    /// Semantic Haptic / Auditory Micro-Click event emitted on this frame (`REQ-014-07`).
    pub taptic_event: Option<MacTapticEvent>,
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

    /// Immediately zeroes any active free-spin flywheel momentum for the thumbwheel identified by `id_source` (`REQ-015-10`).
    pub fn brake_flywheel(ctx: &egui::Context, id_source: impl Hash + std::fmt::Debug) {
        let mem_id = Id::NULL.with(id_source).with("edge_thumbwheel_state");
        ctx.data_mut(|d| {
            if let Some(mut mem) = d.get_temp::<EdgeWheelMemory>(mem_id) {
                mem.velocity_pt_s = 0.0;
                d.insert_temp(mem_id, mem);
            }
        });
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
        let h = if self.wide_drum_width.is_some() { m.edge_wheel_width.points() * 2.0 * s } else { m.edge_wheel_height.points() * s };
        let desired = Vec2::new(w, h);

        let (rect, mut response) = if let Some(c_rect) = self.dock_rect {
            let margin = m.dock_margin.points() * s;
            let pos = if self.wide_drum_width.is_some() {
                Pos2::new(c_rect.center().x - w * 0.5, (c_rect.max.y - h).max(c_rect.min.y))
            } else {
                Pos2::new(c_rect.max.x - w - margin, c_rect.min.y + margin * 2.0 + m.undo_bar_height.points())
            };
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
        let dt = mem.last_time.map(|prev| (now - prev).clamp(0.0, 0.5) as f32).unwrap_or(0.0);
        mem.last_time = Some(now);

        let is_down = response.is_pointer_button_down_on();
        let pointer_pos = response.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.hover_pos()));
        let mut step_pt = 0.0f32;
        let mut taptic_event = None;

        if is_down {
            if !mem.was_down {
                // Touch-to-Brake: immediately zero any active flywheel coasting on finger contact!
                if mem.velocity_pt_s.abs() > m.edge_wheel_stop_threshold.points() {
                    taptic_event = Some(MacTapticEvent::FlywheelBraked);
                }
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
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
            }
        } else {
            mem.velocity_pt_s = 0.0;
        }

        if response.has_focus() {
            let key_dir = ui.input(|i| {
                if i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::ArrowRight) {
                    1.0f32
                } else if i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::ArrowLeft) {
                    -1.0f32
                } else {
                    0.0f32
                }
            });
            if key_dir.abs() > 0.0 {
                let detent_deg = m.detent_step_deg.points().max(1.0);
                step_pt = key_dir * (h * detent_deg / m.half_circle_deg.points().max(1.0));
            }
        }
        response.widget_info(|| egui::WidgetInfo::slider(true, f64::from(*self.value), "Rotary Edge Thumbwheel"));

        let mut value_delta = 0.0f32;
        let mut delta_ticks = 0.0f32;
        if step_pt.abs() > 0.0 {
            let step_deg = m.detent_step_deg.points().max(1.0);
            let prev_detent = (mem.phase_deg / step_deg).floor() as i32;
            let phase_step_deg = step_pt * (m.half_circle_deg.points() / h.max(1.0));
            mem.phase_deg = (mem.phase_deg + phase_step_deg) % m.full_circle_deg.points();
            let next_detent = (mem.phase_deg / step_deg).floor() as i32;
            if next_detent != prev_detent && taptic_event.is_none() {
                taptic_event = Some(MacTapticEvent::RotaryDetentTick);
            }
            delta_ticks = phase_step_deg / step_deg;
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

        // Paint Top-Down Cylindrical Edge Thumbwheel in Liquid Glass (Iter 6: W2-V)
        let accent_col = theme.resolve_glass_accent_on(self.glass_accent, theme.popover_background());
        let painter = ui.painter_at(rect.intersect(ui.clip_rect()));
        let outer_cr = CornerRadius::same((m.edge_wheel_corner_radius.points() * s) as u8);
        let pad = m.edge_wheel_slot_padding.points() * s;
        let inner_rect = rect.shrink(pad);
        let inner_cr = CornerRadius::same(((m.edge_wheel_corner_radius.points() * s) - pad).max(1.0) as u8);

        let bw = m.border_stroke_width.points();
        let is_light = theme.appearance == crate::theme::MacAppearance::Aqua;
        let glass_housing = if is_light {
            theme.box_background()
        } else {
            theme.popover_background().gamma_multiply(m.glass_fill_alpha.points())
        };
        let border_stroke = Stroke::new(bw, theme.separator());
        let specular_col = theme.specular_rim_top();
        painter.rect_filled(rect.translate(Vec2::new(0.0, bw * 2.0)), outer_cr, theme.drop_shadow());
        painter.rect_stroke(
            rect.expand(bw),
            outer_cr,
            Stroke::new(m.active_stroke_width.points(), accent_col.gamma_multiply(m.hover_fill_opacity.points())),
            StrokeKind::Outside,
        );
        painter.rect(rect, outer_cr, glass_housing, border_stroke, StrokeKind::Inside);
        painter.line_segment(
            [Pos2::new(rect.min.x + pad, rect.min.y + bw), Pos2::new(rect.max.x - pad, rect.min.y + bw)],
            Stroke::new(bw, specular_col),
        );

        if is_free_spinning || is_down {
            let glow_stroke = Stroke::new(m.active_stroke_width.points(), accent_col.gamma_multiply(m.active_fill_opacity.points()));
            painter.rect_stroke(inner_rect.expand(1.0), inner_cr, glow_stroke, StrokeKind::Outside);
        }

        let r_cyl = inner_rect.height() * 0.5;
        let y_c = inner_rect.center().y;
        let ptr_bias = pointer_pos
            .filter(|p| rect.contains(*p))
            .map(|p| ((p.y - y_c) / r_cyl.max(1.0)).clamp(-0.5, 0.5))
            .unwrap_or(0.0);
        let cap_w = (m.ring_gap.points() * 2.0 * s).min(inner_rect.width() * m.hover_fill_opacity.points());
        let inner_cr_pt = ((m.edge_wheel_corner_radius.points() * s) - pad).max(1.0);
        paint_raytraced_edge_thumbwheel_quad(
            &painter,
            self.id_source,
            inner_rect,
            (cap_w, inner_cr_pt, ptr_bias, is_free_spinning || is_down),
            accent_col,
            theme,
            m,
        );
        painter.rect_stroke(inner_rect, inner_cr, border_stroke, StrokeKind::Inside);

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
                let thickness = bw + crown_cos * bw;
                let trough_col = theme.drop_shadow().gamma_multiply(crown_cos * 0.5);
                let crest_col = if is_light {
                    specular_col.gamma_multiply(crown_cos)
                } else if crown_cos > m.joystick_fast_ratio.points() {
                    specular_col
                } else {
                    theme.secondary_label().gamma_multiply(crown_cos)
                };
                let ridge_inset = cap_w + pad * 0.5;
                painter.line_segment(
                    [Pos2::new(inner_rect.min.x + ridge_inset, y + bw), Pos2::new(inner_rect.max.x - ridge_inset, y + bw)],
                    Stroke::new(thickness, trough_col),
                );
                painter.line_segment(
                    [Pos2::new(inner_rect.min.x + ridge_inset, y), Pos2::new(inner_rect.max.x - ridge_inset, y)],
                    Stroke::new(thickness, crest_col),
                );
            }
            base_ang += step_deg;
        }

        MacEdgeThumbwheelResponse { response, value_delta, delta_ticks, is_free_spinning, effective_glass_accent: accent_col, taptic_event }
    }
}
