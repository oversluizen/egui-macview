//! State 1 (`v0.1–v0.2`): Vector & Inspector Studio (`GalleryScreen::CanvasPhysics`).

use super::types::apply_blend_mode;
use super::CenterState;
use crate::CardModel;
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Ui};
use egui_macview::{theme::MacTheme, MacBackgroundStyle, MacContextMenu, MacToastManager, NSMenuItem};
use std::cell::RefCell;

impl CenterState {
    pub fn render_state1_canvas_slot(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        card: &mut CardModel,
        args: &super::center_state::CenterRenderArgs<'_>,
        toasts: &RefCell<MacToastManager>,
        slot_rect: Rect,
    ) {
        let super::center_state::CenterRenderArgs { inspector_mode, snapping_enabled, unit_segmented_idx, vis_canvas, lock_canvas, .. } =
            *args;
        let vis_max_x = slot_rect.max.x.min(content_ui.clip_rect().max.x - 4.0).max(slot_rect.min.x + 260.0);
        let canvas_rect = Rect::from_min_max(slot_rect.min, pos2(vis_max_x, slot_rect.max.y));
        let canvas_resp = content_ui.interact(canvas_rect, content_ui.scope_id().with("state1_vector_canvas_hit"), Sense::click());
        if canvas_resp.clicked() {
            self.bounce_start_time = Some(content_ui.input(|i| i.time));
        }

        let cp = content_ui.painter_at(canvas_rect);
        cp.rect(canvas_rect, CornerRadius::same(8), theme.window_background(), Stroke::new(1.0, theme.separator()), StrokeKind::Inside);

        if self.show_grid {
            let grid_stroke = Stroke::new(0.5, theme.separator().gamma_multiply(0.4));
            let mut gx = canvas_rect.min.x + 24.0;
            while gx < canvas_rect.max.x {
                cp.line_segment([pos2(gx, canvas_rect.min.y), pos2(gx, canvas_rect.max.y)], grid_stroke);
                gx += 24.0;
            }
            let mut gy = canvas_rect.min.y + 24.0;
            while gy < canvas_rect.max.y {
                cp.line_segment([pos2(canvas_rect.min.x, gy), pos2(canvas_rect.max.x, gy)], grid_stroke);
                gy += 24.0;
            }
        }

        if !vis_canvas {
            cp.text(
                canvas_rect.center(),
                Align2::CENTER_CENTER,
                "Canvas Layer Hidden — Click the visibility dot in Sidebar to show",
                FontId::proportional(12.0),
                theme.secondary_label(),
            );
            return;
        }

        let (w_scale, h_scale, r_scale, _) = super::inspector::unit_scale_and_suffix(unit_segmented_idx);
        let raw_card_w = (card.width.parse::<f32>().unwrap_or(240.0 * w_scale) / w_scale).clamp(60.0, 500.0);
        let card_h = (card.height.parse::<f32>().unwrap_or(140.0 * h_scale) / h_scale).clamp(40.0, 300.0);
        let card_radius = (card.radius.parse::<f32>().unwrap_or(10.0 * r_scale) / r_scale).clamp(0.0, 40.0);

        let is_compact = inspector_mode == crate::InspectorMode::CompactHud;
        let card_zone_min_x = if is_compact { canvas_rect.min.x + 196.0 } else { canvas_rect.min.x };
        let card_zone_max_x =
            if is_compact { canvas_rect.max.x - 16.0 } else { (canvas_rect.max.x - 196.0).max(canvas_rect.min.x + 180.0) };
        let card_zone_center_x = (card_zone_min_x + card_zone_max_x) * 0.5;
        let (card_w, base_center_x) = match card.alignment_idx {
            0 => (raw_card_w, card_zone_min_x + 24.0 + raw_card_w * 0.5),
            2 => (raw_card_w, card_zone_max_x - 16.0 - raw_card_w * 0.5),
            3 => ((card_zone_max_x - card_zone_min_x - 36.0).max(60.0), card_zone_center_x),
            _ => (raw_card_w, card_zone_center_x),
        };
        let cur_time = content_ui.input(|i| i.time);
        if self.bounce_start_time.is_some_and(|t| t < 0.0) {
            self.bounce_start_time = Some(cur_time);
        }
        if self.spring_start_time.is_some_and(|t| t < 0.0) {
            self.spring_start_time = Some(cur_time);
        }
        let bounce_scale = self.bounce_start_time.map_or(1.0, |start_t| {
            let elapsed = (cur_time - start_t).max(0.0) as f32;
            if elapsed < self.bounce_spring.settling_duration() {
                content_ui.ctx().request_repaint_after_secs(1.0 / 60.0);
                let p = self.bounce_spring.evaluate(elapsed);
                1.0 + 0.20 * (1.0 - p).abs() * (elapsed * 12.0).sin()
            } else {
                1.0
            }
        });

        let effective_rot = card.rotation_deg;
        let effective_scale = card.scale_pct;
        let scale_factor = (effective_scale / 100.0) * bounce_scale;
        let (a, b, c, d) = card.affine_matrix(effective_rot, scale_factor);

        let (ox, oy) = match self.origin_anchor as usize {
            0 => (-28.0, -18.0),
            1 => (0.0, -18.0),
            2 => (28.0, -18.0),
            3 => (-28.0, 0.0),
            5 => (28.0, 0.0),
            6 => (-28.0, 18.0),
            7 => (0.0, 18.0),
            8 => (28.0, 18.0),
            _ => (0.0, 0.0),
        };
        let drag_off = if snapping_enabled {
            vec2((self.card_drag_offset.x / 8.0).round() * 8.0, (self.card_drag_offset.y / 8.0).round() * 8.0)
        } else {
            self.card_drag_offset
        };
        let card_center = pos2(
            (base_center_x + drag_off.x + ox).clamp(canvas_rect.min.x + 40.0, canvas_rect.max.x - 40.0),
            (canvas_rect.center().y + drag_off.y + oy).clamp(canvas_rect.min.y + 30.0, canvas_rect.max.y - 30.0),
        );
        let (hw, hh) = (card_w * 0.5, card_h * 0.5);
        let corners_world: [Pos2; 4] = [pos2(-hw, -hh), pos2(hw, -hh), pos2(hw, hh), pos2(-hw, hh)]
            .map(|pt| pos2(card_center.x + a * pt.x + b * pt.y, card_center.y + c * pt.x + d * pt.y));

        let fill_alpha = (card.opacity / 100.0).clamp(0.0, 1.0);
        let card_fill = apply_blend_mode(card.alpha_color, card.fill_color, card.blend_mode_idx).gamma_multiply(fill_alpha);
        let border_stroke = Stroke::new(card.stroke_width.max(0.0), card.stroke_color);
        let unit_str = ["pt", "px", "%"][unit_segmented_idx.min(2)];
        let blend_label = ["Normal", "Multiply", "Screen", "Overlay"][card.blend_mode_idx.min(3)];
        let card_bg_style = MacBackgroundStyle::from_color(card_fill);
        let (text_primary, text_secondary) = (card_bg_style.primary_text_color(theme), card_bg_style.secondary_text_color(theme));

        if (effective_rot.abs() < 1e-4) && (card.skew_deg.abs() < 1e-4) && card_radius <= 0.5 {
            let card_rect = Rect::from_center_size(card_center, vec2(card_w * scale_factor, card_h * scale_factor));
            if card.shadow_enabled && card.shadow_blur > 0.1 {
                let blur = card.shadow_blur as f32;
                for ring in 1..=3 {
                    let spread = blur * (ring as f32 / 3.0);
                    let shadow_rect = card_rect.expand(spread * 0.4).translate(vec2(0.0, 2.0 + spread * 0.35));
                    cp.rect_filled(
                        shadow_rect,
                        CornerRadius::same((card_radius + spread) as u8),
                        Color32::from_black_alpha((48 / ring) as u8),
                    );
                }
            }
            cp.rect(card_rect, CornerRadius::same(card_radius as u8), card_fill, border_stroke, StrokeKind::Inside);
            let title_str: std::borrow::Cow<'_, str> = if lock_canvas {
                std::borrow::Cow::Owned(format!("{} [LOCKED]", card.layer_name))
            } else {
                std::borrow::Cow::Borrowed(&card.layer_name)
            };
            cp.text(
                pos2(card_rect.center().x, card_rect.center().y - 12.0),
                Align2::CENTER_CENTER,
                title_str.as_ref(),
                FontId::proportional(13.0),
                text_primary,
            );
            cp.text(
                pos2(card_rect.center().x, card_rect.center().y + 12.0),
                Align2::CENTER_CENTER,
                format!(
                    "{:.0}×{:.0} {unit_str} | R: {card_radius:.0}pt | {blend_label} | Blur {:.0}pt",
                    card_w * w_scale,
                    card_h * h_scale,
                    card.shadow_blur
                ),
                FontId::proportional(10.5),
                text_secondary,
            );
        } else {
            let poly_pts = build_transformed_rounded_rect_pts(card_center, hw, hh, card_radius, (a, b, c, d));
            if card.shadow_enabled && card.shadow_blur > 0.1 {
                let offset_y = 2.0 + card.shadow_blur as f32 * 0.4;
                let shadow_pts: Vec<Pos2> = poly_pts.iter().map(|pt| pos2(pt.x, pt.y + offset_y)).collect();
                cp.add(egui::epaint::Shape::convex_polygon(shadow_pts, Color32::from_black_alpha(45), Stroke::NONE));
            }
            cp.add(egui::epaint::Shape::convex_polygon(poly_pts, card_fill, border_stroke));
            cp.text(
                pos2(card_center.x, card_center.y - 10.0),
                Align2::CENTER_CENTER,
                &card.layer_name,
                FontId::proportional(13.0),
                text_primary,
            );
            cp.text(
                pos2(card_center.x, card_center.y + 10.0),
                Align2::CENTER_CENTER,
                format!("θ: {effective_rot:.1}° | S: {effective_scale:.0}% | R: {card_radius:.0}pt | {blend_label}"),
                FontId::proportional(10.5),
                text_secondary,
            );
        }

        // Draw Vector Artboard Selection Bounding Box + 8 White Square Handles + Reference Origin Crosshair
        let bbox_stroke = Stroke::new(1.0, theme.control_accent());
        for i in 0..4 {
            cp.line_segment([corners_world[i], corners_world[(i + 1) % 4]], bbox_stroke);
        }
        let mid_pts = [
            pos2((corners_world[0].x + corners_world[1].x) * 0.5, (corners_world[0].y + corners_world[1].y) * 0.5),
            pos2((corners_world[1].x + corners_world[2].x) * 0.5, (corners_world[1].y + corners_world[2].y) * 0.5),
            pos2((corners_world[2].x + corners_world[3].x) * 0.5, (corners_world[2].y + corners_world[3].y) * 0.5),
            pos2((corners_world[3].x + corners_world[0].x) * 0.5, (corners_world[3].y + corners_world[0].y) * 0.5),
        ];
        for h_pt in corners_world.iter().chain(mid_pts.iter()) {
            let hr = Rect::from_center_size(*h_pt, vec2(7.0, 7.0));
            cp.rect(hr, CornerRadius::same(1), Color32::WHITE, Stroke::new(1.2, theme.control_accent()), StrokeKind::Inside);
        }
        let origin_local = match self.origin_anchor as usize {
            0 => pos2(-hw, -hh),
            1 => pos2(0.0, -hh),
            2 => pos2(hw, -hh),
            3 => pos2(-hw, 0.0),
            5 => pos2(hw, 0.0),
            6 => pos2(-hw, hh),
            7 => pos2(0.0, hh),
            8 => pos2(hw, hh),
            _ => pos2(0.0, 0.0),
        };
        let origin_world =
            pos2(card_center.x + a * origin_local.x + b * origin_local.y, card_center.y + c * origin_local.x + d * origin_local.y);
        cp.circle_stroke(origin_world, 6.0, Stroke::new(1.5, theme.system_orange()));
        cp.circle_filled(origin_world, 2.2, theme.system_orange());
        cp.line_segment([origin_world - vec2(9.0, 0.0), origin_world + vec2(9.0, 0.0)], Stroke::new(1.2, theme.system_orange()));
        cp.line_segment([origin_world - vec2(0.0, 9.0), origin_world + vec2(0.0, 9.0)], Stroke::new(1.2, theme.system_orange()));

        let handle_rect = Rect::from_center_size(corners_world[2], vec2(10.0, 10.0));
        cp.rect(
            handle_rect,
            CornerRadius::same(2),
            theme.window_background(),
            Stroke::new(1.5, theme.control_accent()),
            StrokeKind::Inside,
        );
        let handle_hit = handle_rect.expand(4.0).intersect(canvas_rect);
        let resize_resp =
            content_ui.interact(handle_hit, content_ui.scope_id().with("interactive_canvas_card_resize"), Sense::click_and_drag());
        if resize_resp.hovered() || resize_resp.dragged() {
            content_ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeSouthEast);
        }
        if resize_resp.dragged() && !lock_canvas {
            let delta = resize_resp.drag_delta();
            let next_w = (raw_card_w + delta.x * 2.0).clamp(60.0, 500.0);
            let next_h = if card.constrain_proportions && raw_card_w > 1.0 {
                (card_h * (next_w / raw_card_w)).clamp(40.0, 300.0)
            } else {
                (card_h + delta.y * 2.0).clamp(40.0, 300.0)
            };
            card.width = format!("{:.1}", next_w * w_scale);
            card.height = format!("{:.1}", next_h * h_scale);
        }

        let card_bounds = Rect::from_points(&corners_world).intersect(canvas_rect);
        let card_resp =
            content_ui.interact(card_bounds, content_ui.scope_id().with("interactive_canvas_card_hit"), Sense::click_and_drag());
        if card_resp.dragged() && !resize_resp.dragged() && !lock_canvas {
            self.card_drag_offset += card_resp.drag_delta();
        }

        let context_items = [
            NSMenuItem::action("Reset Transform", "card.reset_transform"),
            NSMenuItem::action("Center Card", "card.center"),
            NSMenuItem::action("Fill Width", "card.fill_width"),
            NSMenuItem::separator(),
            NSMenuItem::action("Toggle Proportions", "card.toggle_proportions").checked(card.constrain_proportions),
            NSMenuItem::action("Toggle Drop Shadow", "card.toggle_shadow").checked(card.shadow_enabled),
            NSMenuItem::action("Copy Style", "card.copy_style"),
        ];
        let mut context_menu =
            MacContextMenu::new(&context_items, &mut self.card_menu_open, &mut self.card_menu_pos, &mut self.card_menu_selected)
                .theme(theme);
        context_menu.attach_to(&card_resp);
        if !card_resp.secondary_clicked() {
            context_menu.attach_to(&canvas_resp);
        }
        if let Some(action) = context_menu.show(content_ui) {
            match action.as_str() {
                "card.reset_transform" => {
                    (card.rotation_deg, card.scale_pct, card.skew_deg, self.card_drag_offset) = (0.0, 100.0, 0.0, egui::Vec2::ZERO);
                    toasts.borrow_mut().info("Transform reset");
                }
                "card.center" => {
                    (card.alignment_idx, self.card_drag_offset) = (1, egui::Vec2::ZERO);
                    toasts.borrow_mut().info("Preset applied");
                }
                "card.fill_width" => {
                    card.alignment_idx = 3;
                    toasts.borrow_mut().info("Preset applied");
                }
                "card.toggle_proportions" => {
                    card.constrain_proportions = !card.constrain_proportions;
                    toasts.borrow_mut().info("Aspect lock toggled");
                }
                "card.toggle_shadow" => {
                    card.shadow_enabled = !card.shadow_enabled;
                    toasts.borrow_mut().info("Drop shadow toggled");
                }
                "card.copy_style" => {
                    content_ui.ctx().copy_text(format!("width: {}pt; height: {}pt; radius: {}pt;", card.width, card.height, card.radius));
                    toasts.borrow_mut().success("Style copied");
                }
                _ => {}
            }
        }

        self.render_state1_thumb_dial_overlay(content_ui, theme, card, args, toasts, canvas_rect);
        self.render_state1_hud_and_popover(content_ui, theme, card, inspector_mode, canvas_rect, (w_scale, h_scale, r_scale));
    }
}

fn build_transformed_rounded_rect_pts(center: Pos2, hw: f32, hh: f32, radius: f32, (a, b, c, d): (f32, f32, f32, f32)) -> Vec<Pos2> {
    let r = radius.clamp(0.0, hw.min(hh) - 1.0);
    if r <= 0.5 {
        return [pos2(-hw, -hh), pos2(hw, -hh), pos2(hw, hh), pos2(-hw, hh)]
            .into_iter()
            .map(|pt| pos2(center.x + a * pt.x + b * pt.y, center.y + c * pt.x + d * pt.y))
            .collect();
    }
    let arcs = [
        (-hw + r, -hh + r, std::f32::consts::PI, 1.5 * std::f32::consts::PI),
        (hw - r, -hh + r, 1.5 * std::f32::consts::PI, 2.0 * std::f32::consts::PI),
        (hw - r, hh - r, 0.0, 0.5 * std::f32::consts::PI),
        (-hw + r, hh - r, 0.5 * std::f32::consts::PI, std::f32::consts::PI),
    ];
    let mut pts = Vec::with_capacity(100);
    for (cx, cy, a0, a1) in arcs {
        for step in 0..=24 {
            let t = a0 + (a1 - a0) * (step as f32 / 24.0);
            let (lx, ly) = (cx + r * t.cos(), cy + r * t.sin());
            pts.push(pos2(center.x + a * lx + b * ly, center.y + c * lx + d * ly));
        }
    }
    pts
}
