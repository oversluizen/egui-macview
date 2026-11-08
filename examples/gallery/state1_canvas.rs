//! State 1 (`v0.1–v0.2`): Vector & Inspector Studio (`GalleryScreen::CanvasPhysics`).

use super::CenterState;
use crate::{CardModel, InspectorMode};
use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, FontId, Id, Pos2, Rect, Sense, Stroke, StrokeKind,
    Ui,
};
use egui_macview::{
    theme::MacTheme, MacBackgroundStyle, MacButtonCell, MacContextMenu, MacControlSize,
    MacFloatingInfoBox, MacInputCell, MacPopover, MacPopoverArrowEdge, MacSlider, MacSymbol,
    MacToastManager, NSMenuItem, NSUndoManager,
};
use std::cell::RefCell;

fn apply_blend_mode(base: Color32, top: Color32, mode_idx: usize) -> Color32 {
    let (br, bg, bb) = (
        base.r() as f32 / 255.0,
        base.g() as f32 / 255.0,
        base.b() as f32 / 255.0,
    );
    let (tr, tg, tb) = (
        top.r() as f32 / 255.0,
        top.g() as f32 / 255.0,
        top.b() as f32 / 255.0,
    );
    let blend_ch = |b: f32, t: f32| -> f32 {
        match mode_idx {
            1 => b * t,
            2 => 1.0 - (1.0 - b) * (1.0 - t),
            3 if b < 0.5 => 2.0 * b * t,
            3 => 1.0 - 2.0 * (1.0 - b) * (1.0 - t),
            _ => t,
        }
    };
    Color32::from_rgba_unmultiplied(
        (blend_ch(br, tr).clamp(0.0, 1.0) * 255.0) as u8,
        (blend_ch(bg, tg).clamp(0.0, 1.0) * 255.0) as u8,
        (blend_ch(bb, tb).clamp(0.0, 1.0) * 255.0) as u8,
        top.a(),
    )
}

impl CenterState {
    #[allow(clippy::too_many_arguments)]
    pub fn render_state1_canvas(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        card: &mut CardModel,
        undo_manager: &RefCell<NSUndoManager<CardModel>>,
        last_stable_card: &RefCell<CardModel>,
        inspector_mode: InspectorMode,
        snapping_enabled: bool,
        unit_segmented_idx: usize,
        vis_canvas: bool,
        lock_canvas: bool,
        toasts: &RefCell<MacToastManager>,
    ) {
        let label_font = FontId::proportional(12.0);
        content_ui.label(
            egui::RichText::new("State 1 (v0.1–v0.2) — Vector & Inspector Studio (2D Affine Canvas & HUD)")
                .font(label_font)
                .strong(),
        );
        content_ui.label(
            egui::RichText::new("Drag card or its corner resize handle on canvas, step W/H/R with ArrowUp/Down (Shift=×10, Cmd/Ctrl=×100), or right-click for NSMenu.")
                .size(11.0),
        );
        content_ui.add_space(6.0);

        let canvas_w = (content_ui.available_width() - 32.0).max(340.0);
        let (canvas_rect, canvas_resp) =
            content_ui.allocate_exact_size(vec2(canvas_w, 240.0), Sense::click());
        if canvas_resp.clicked() {
            self.bounce_start_time = Some(content_ui.input(|i| i.time));
        }

        let cp = content_ui.painter_at(canvas_rect);
        cp.rect(
            canvas_rect,
            CornerRadius::same(8),
            theme.window_background(),
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );

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

        let raw_card_w = card.width.parse::<f32>().unwrap_or(240.0).clamp(60.0, 500.0);
        let card_h = card.height.parse::<f32>().unwrap_or(140.0).clamp(40.0, 300.0);
        let card_radius = card.radius.parse::<f32>().unwrap_or(10.0).clamp(0.0, 40.0);

        let (card_w, base_center_x) = match card.alignment_idx {
            0 => (raw_card_w, canvas_rect.min.x + 24.0 + raw_card_w * 0.5),
            2 => (raw_card_w, canvas_rect.max.x - 24.0 - raw_card_w * 0.5),
            3 => ((canvas_rect.width() - 48.0).max(60.0), canvas_rect.center().x),
            _ => (raw_card_w, canvas_rect.center().x),
        };
        let cur_time = content_ui.input(|i| i.time);
        let bounce_scale = self.bounce_start_time.map_or(1.0, |start_t| {
            let elapsed = (cur_time - start_t) as f32;
            if elapsed < self.bounce_spring.settling_duration() {
                content_ui.ctx().request_repaint();
                let p = self.bounce_spring.evaluate(elapsed);
                1.0 + 0.20 * (1.0 - p).abs() * (elapsed * 12.0).sin()
            } else {
                1.0
            }
        });

        let effective_rot = if snapping_enabled {
            (card.rotation_deg / 15.0).round() * 15.0
        } else {
            card.rotation_deg
        };
        let effective_scale = if snapping_enabled {
            (card.scale_pct / 25.0).round() * 25.0
        } else {
            card.scale_pct
        };
        let theta = effective_rot.to_radians();
        let scale_factor = (effective_scale / 100.0) * bounce_scale;
        let skew_val = card.skew_deg.to_radians().tan();
        let (a, b, c, d) = (
            scale_factor * theta.cos(),
            -scale_factor * theta.sin() + skew_val,
            scale_factor * theta.sin(),
            scale_factor * theta.cos(),
        );

        let card_center = pos2(
            (base_center_x + self.card_drag_offset.x).clamp(canvas_rect.min.x + 40.0, canvas_rect.max.x - 40.0),
            (canvas_rect.center().y + self.card_drag_offset.y).clamp(canvas_rect.min.y + 30.0, canvas_rect.max.y - 30.0),
        );
        let (hw, hh) = (card_w * 0.5, card_h * 0.5);
        let corners_world: Vec<Pos2> = [pos2(-hw, -hh), pos2(hw, -hh), pos2(hw, hh), pos2(-hw, hh)]
            .iter()
            .map(|pt| pos2(card_center.x + a * pt.x + b * pt.y, card_center.y + c * pt.x + d * pt.y))
            .collect();

        let fill_alpha = (card.opacity / 100.0).clamp(0.0, 1.0);
        let card_fill = apply_blend_mode(card.alpha_color, card.fill_color, card.blend_mode_idx).gamma_multiply(fill_alpha);
        let unit_str = ["pt", "px", "%"][unit_segmented_idx.min(2)];
        let blend_label = ["Normal", "Multiply", "Screen", "Overlay"][card.blend_mode_idx.min(3)];
        let card_bg_style = MacBackgroundStyle::from_color(card_fill);
        let (text_primary, text_secondary) = (
            card_bg_style.primary_text_color(theme),
            card_bg_style.secondary_text_color(theme),
        );

        if (theta.abs() < 1e-4) && (skew_val.abs() < 1e-4) {
            let card_rect = Rect::from_center_size(card_center, vec2(card_w * scale_factor, card_h * scale_factor));
            if card.shadow_enabled && card.shadow_blur > 0.1 {
                let blur = card.shadow_blur as f32;
                for ring in 1..=3 {
                    let spread = blur * (ring as f32 / 3.0);
                    let shadow_rect = card_rect.expand(spread * 0.4).translate(vec2(0.0, 2.0 + spread * 0.35));
                    cp.rect_filled(shadow_rect, CornerRadius::same((card_radius + spread) as u8), Color32::from_black_alpha((48 / ring) as u8));
                }
            }
            cp.rect(card_rect, CornerRadius::same(card_radius as u8), card_fill, Stroke::new(2.0, card.alpha_color), StrokeKind::Inside);
            let title_str = if lock_canvas { format!("{} [LOCKED]", card.layer_name) } else { card.layer_name.clone() };
            cp.text(pos2(card_rect.center().x, card_rect.center().y - 12.0), Align2::CENTER_CENTER, title_str, FontId::proportional(13.0), text_primary);
            cp.text(
                pos2(card_rect.center().x, card_rect.center().y + 12.0),
                Align2::CENTER_CENTER,
                format!("{card_w:.0}×{card_h:.0} {unit_str} | {blend_label} | Blur {:.0}pt", card.shadow_blur),
                FontId::proportional(10.5),
                text_secondary,
            );
        } else {
            if card.shadow_enabled && card.shadow_blur > 0.1 {
                let offset_y = 2.0 + card.shadow_blur as f32 * 0.4;
                let shadow_pts: Vec<Pos2> = corners_world.iter().map(|pt| pos2(pt.x, pt.y + offset_y)).collect();
                cp.add(egui::epaint::Shape::convex_polygon(shadow_pts, Color32::from_black_alpha(45), Stroke::NONE));
            }
            cp.add(egui::epaint::Shape::convex_polygon(corners_world.clone(), card_fill, Stroke::new(2.0, card.alpha_color)));
            cp.text(pos2(card_center.x, card_center.y - 10.0), Align2::CENTER_CENTER, &card.layer_name, FontId::proportional(13.0), text_primary);
            cp.text(
                pos2(card_center.x, card_center.y + 10.0),
                Align2::CENTER_CENTER,
                format!("θ: {effective_rot:.1}° | S: {effective_scale:.0}% | {blend_label}"),
                FontId::proportional(10.5),
                text_secondary,
            );
        }

        let handle_rect = Rect::from_center_size(corners_world[2], vec2(10.0, 10.0));
        cp.rect(handle_rect, CornerRadius::same(2), theme.window_background(), Stroke::new(1.5, theme.control_accent()), StrokeKind::Inside);
        let resize_resp = content_ui.interact(
            handle_rect.expand(4.0),
            content_ui.scope_id().with("interactive_canvas_card_resize"),
            Sense::click_and_drag(),
        );
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
            card.width = format!("{next_w:.1}");
            card.height = format!("{next_h:.1}");
        }

        let card_bounds = Rect::from_points(&corners_world);
        let card_resp = content_ui.interact(card_bounds, content_ui.scope_id().with("interactive_canvas_card_hit"), Sense::click_and_drag());
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
        let mut context_menu = MacContextMenu::new(&context_items, &mut self.card_menu_open, &mut self.card_menu_pos, &mut self.card_menu_selected).theme(theme);
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

        content_ui.add_space(8.0);
        content_ui.horizontal(|h| {
            let pulse_label = format!("{} Trigger CASpring Pulse", MacSymbol::ArrowClockwise.as_str());
            if h.add(MacButtonCell::new(&pulse_label).theme(theme).selected(self.bounce_start_time.is_some())).clicked() {
                self.bounce_start_time = Some(h.input(|i| i.time));
            }
            let reset_label = format!("{} Reset Identity", MacSymbol::ArrowCounterClockwise.as_str());
            if h.add(MacButtonCell::new(&reset_label).theme(theme)).clicked() {
                (card.rotation_deg, card.scale_pct, card.skew_deg, self.card_drag_offset) = (0.0, 100.0, 0.0, egui::Vec2::ZERO);
                toasts.borrow_mut().info("Transform reset");
            }
            let (can_undo, can_redo, undo_n, redo_m) = {
                let um = undo_manager.borrow();
                (um.can_undo(), um.can_redo(), um.undo_count(), um.redo_count())
            };
            let undo_label = format!("{} Undo ({undo_n})", MacSymbol::ArrowCounterClockwise.as_str());
            if h.add(MacButtonCell::new(&undo_label).theme(theme).enabled(can_undo)).clicked() && undo_manager.borrow_mut().undo(card) {
                *last_stable_card.borrow_mut() = card.clone();
                toasts.borrow_mut().info("Undo applied");
            }
            let redo_label = format!("{} Redo ({redo_m})", MacSymbol::ArrowClockwise.as_str());
            if h.add(MacButtonCell::new(&redo_label).theme(theme).enabled(can_redo)).clicked() && undo_manager.borrow_mut().redo(card) {
                *last_stable_card.borrow_mut() = card.clone();
                toasts.borrow_mut().info("Redo applied");
            }
        });

        if inspector_mode == InspectorMode::CompactHud {
            let mut hud_abs_pos = pos2(canvas_rect.min.x + self.hud_pos.x, canvas_rect.min.y + self.hud_pos.y);
            let label_col = theme.hud_secondary_text();
            let font_pt = theme.text_field().compact_infobox.font_size.points();
            let cell_w = theme.text_field().compact_infobox.default_width.points();
            let (hud_w_id, hud_h_id, hud_r_id) = (Id::NULL.with("hud_w"), Id::NULL.with("hud_h"), Id::NULL.with("hud_r"));

            MacFloatingInfoBox::new("gallery_compact_hud", &mut hud_abs_pos)
                .theme(theme)
                .title("HUD Palette")
                .bounds(canvas_rect)
                .show(
                    content_ui.ctx(),
                    |row1_ui| {
                        row1_ui.label(egui::RichText::new("W:").size(font_pt).color(label_col));
                        MacInputCell::with_id(hud_w_id, &mut card.width).theme(theme).control_size(MacControlSize::CompactInfobox).numeric(60.0..=500.0, 1.0).width(cell_w).prev_focus(hud_r_id).next_focus(hud_h_id).show(row1_ui);
                        row1_ui.add_space(4.0);
                        row1_ui.label(egui::RichText::new("H:").size(font_pt).color(label_col));
                        MacInputCell::with_id(hud_h_id, &mut card.height).theme(theme).control_size(MacControlSize::CompactInfobox).numeric(40.0..=300.0, 1.0).width(cell_w).prev_focus(hud_w_id).next_focus(hud_r_id).show(row1_ui);
                    },
                    |row2_ui| {
                        row2_ui.label(egui::RichText::new("R:").size(font_pt).color(label_col));
                        MacInputCell::with_id(hud_r_id, &mut card.radius).theme(theme).control_size(MacControlSize::CompactInfobox).numeric(0.0..=40.0, 1.0).width(cell_w).prev_focus(hud_h_id).next_focus(hud_w_id).show(row2_ui);
                        row2_ui.add_space(4.0);
                        row2_ui.label(egui::RichText::new("θ:").size(font_pt).color(label_col));
                        MacSlider::new(&mut card.rotation_deg, 0.0..=360.0).theme(theme).width(55.0).show(row2_ui);
                    },
                );
            self.hud_pos = pos2(hud_abs_pos.x - canvas_rect.min.x, hud_abs_pos.y - canvas_rect.min.y);
        }

        content_ui.add_space(12.0);
        let btn_text = if self.popover_open { "Dismiss Spring Popover" } else { "Present Spring Popover" };
        if content_ui.add(MacButtonCell::new(btn_text).theme(theme)).clicked() {
            self.popover_open = !self.popover_open;
            self.spring_start_time = Some(content_ui.input(|i| i.time));
        }
        if self.popover_open {
            content_ui.add_space(6.0);
            let (pop_rect, _) = content_ui.allocate_exact_size(vec2(320.0, 96.0), Sense::hover());
            MacPopover::new().theme(theme).arrow_edge(MacPopoverArrowEdge::Top).show(content_ui, pop_rect, |pop_ui| {
                pop_ui.add_space(6.0);
                pop_ui.label(egui::RichText::new(format!("CASpring Harmonic Oscillator (T₀={:.2}s, ζ={:.2})", self.spring.response, self.spring.damping_ratio)).size(11.5).strong());
                pop_ui.label(egui::RichText::new(format!("Settling Duration: {:.3}s | Angular Freq ω₀: {:.2} rad/s", self.spring.settling_duration(), self.spring.omega_0())).size(10.5));
            });
        }
        content_ui.add_space(18.0);
    }
}
