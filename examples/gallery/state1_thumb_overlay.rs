//! Thumb Dial, Rotary Edge Wheel & Free Joystick Bowl overlay for State 1 (`GalleryScreen::CanvasPhysics`).

use super::CenterState;
use crate::CardModel;
use egui::{pos2, vec2, Align2, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Ui};
use egui_macview::{
    theme::MacTheme, MacBackgroundStyle, MacContextThumbDial, MacEdgeThumbwheel, MacFreeJoystickBowl, MacThumbArcPlacement,
    MacThumbContextPreset, MacThumbDockZone, MacThumbIntent, MacToastManager,
};
use std::cell::RefCell;

impl CenterState {
    pub(super) fn render_state1_thumb_dial_overlay(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        card: &mut CardModel,
        args: &super::center_state::CenterRenderArgs<'_>,
        toasts: &RefCell<MacToastManager>,
        canvas_rect: Rect,
    ) {
        let presets = MacThumbContextPreset::ALL;
        let active_preset = presets[self.thumb_dial_preset_idx.min(presets.len() - 1)];
        let is_compact = args.inspector_mode == crate::InspectorMode::CompactHud;
        let dock_zone = if is_compact { MacThumbDockZone::BottomLeft } else { MacThumbDockZone::BottomRight };
        let pill_labels = ["Pen (P)", "Select (V)", "Prim (F)", "Knife (E)"];
        let strip_w = 184.0f32;
        let pill_w = strip_w / 4.0;
        let strip_x = if is_compact { canvas_rect.max.x - strip_w - 8.0 } else { canvas_rect.min.x + 10.0 };
        let strip_rect = Rect::from_min_size(pos2(strip_x, canvas_rect.min.y + 6.0), vec2(strip_w, 18.0));
        let sp = content_ui.painter_at(strip_rect);
        for (idx, lbl) in pill_labels.iter().enumerate() {
            let p_rect = Rect::from_min_size(pos2(strip_rect.min.x + idx as f32 * pill_w, strip_rect.min.y), vec2(pill_w - 2.0, 18.0));
            let hit = content_ui.interact(p_rect, content_ui.scope_id().with(("thumb_preset_pill", idx)), Sense::click());
            if hit.clicked() {
                self.thumb_dial_preset_idx = idx;
            }
            let is_sel = self.thumb_dial_preset_idx == idx;
            let fill = if is_sel { theme.control_accent() } else { theme.box_background() };
            let fg = if is_sel { theme.on_accent_text() } else { theme.label() };
            sp.rect(p_rect, CornerRadius::same(4), fill, Stroke::new(1.0, theme.separator()), StrokeKind::Inside);
            sp.text(p_rect.center(), Align2::CENTER_CENTER, *lbl, FontId::proportional(9.0), fg);
        }

        // Row 2: 3 Standalone Interchangeable Configurations + Calculated Glass Accent ColorWell
        let cfg_labels = ["Paired", "Rotary", "Select", "Joystick"];
        let cfg_rect = Rect::from_min_size(pos2(strip_rect.min.x, strip_rect.max.y + 4.0), vec2(strip_w - 36.0, 18.0));
        let cfg_w = cfg_rect.width() / 4.0;
        let cp_cfg = content_ui.painter_at(cfg_rect);
        let eff_accent = theme.resolve_glass_accent_on(Some(self.thumb_glass_color), theme.popover_background());
        for (idx, lbl) in cfg_labels.iter().enumerate() {
            let p_rect = Rect::from_min_size(pos2(cfg_rect.min.x + idx as f32 * cfg_w, cfg_rect.min.y), vec2(cfg_w - 2.0, 18.0));
            let hit = content_ui.interact(p_rect, content_ui.scope_id().with(("thumb_cfg_mode_pill", idx)), Sense::click());
            if hit.clicked() {
                self.thumb_config_mode = idx;
            }
            let is_sel = self.thumb_config_mode == idx;
            let fill = if is_sel { eff_accent } else { theme.box_background() };
            let fg = if is_sel { MacBackgroundStyle::from_color(eff_accent).primary_text_color(theme) } else { theme.label() };
            cp_cfg.rect(p_rect, CornerRadius::same(4), fill, Stroke::new(1.0, theme.separator()), StrokeKind::Inside);
            cp_cfg.text(p_rect.center(), Align2::CENTER_CENTER, *lbl, FontId::proportional(8.5), fg);
        }
        let well_rect = Rect::from_min_size(pos2(cfg_rect.max.x + 2.0, cfg_rect.min.y), vec2(34.0, 18.0));
        let well_hit = content_ui.interact(well_rect, content_ui.scope_id().with("thumb_glass_color_swatch"), Sense::click());
        if well_hit.clicked() {
            let pal = theme.tag_label_palette();
            let cur_idx = pal.iter().position(|c| *c == self.thumb_glass_color).unwrap_or(0);
            self.thumb_glass_color = pal[(cur_idx + 1) % pal.len()];
        }
        cp_cfg.rect(well_rect, CornerRadius::same(4), eff_accent, Stroke::new(1.0, theme.separator()), StrokeKind::Inside);
        cp_cfg.text(
            well_rect.center(),
            Align2::CENTER_CENTER,
            "Hue",
            FontId::proportional(8.5),
            MacBackgroundStyle::from_color(eff_accent).primary_text_color(theme),
        );

        let can_undo = args.undo_manager.borrow().can_undo();
        let can_redo = args.undo_manager.borrow().can_redo();
        let rot_str = format!("θ {:.0}°", card.rotation_deg);
        let max_paired_scale = ((canvas_rect.height() - 20.0) / 262.0).clamp(0.72, 1.02);
        let base_scale = if is_compact {
            1.02f32
        } else if canvas_rect.width() < 460.0 {
            0.84f32
        } else {
            0.94f32
        };
        let dial_scale = if self.thumb_config_mode == 0 { base_scale.min(max_paired_scale) } else { base_scale };
        let mut undo_clicked = false;
        let mut redo_clicked = false;
        let mut nudge_delta = egui::Vec2::ZERO;
        let mut rotary_delta_ticks = 0.0f32;
        let mut effective_mask = egui_macview::MacThumbIntentMask::NONE;
        let prev_rot = card.rotation_deg;
        let mut wheel_deg = prev_rot;
        let taptic_ev;

        match self.thumb_config_mode {
            1 => {
                let drum_w = (theme.thumb_dial().outer_radius.points() * 2.0 * dial_scale) * 0.72;
                let w_resp = MacEdgeThumbwheel::new("state1_standalone_rotary", &mut wheel_deg, 0.0..=360.0)
                    .wide_drum_width(drum_w)
                    .scale(dial_scale)
                    .glass_accent(self.thumb_glass_color)
                    .dock_right_edge(canvas_rect)
                    .theme(theme)
                    .show(content_ui);
                taptic_ev = w_resp.taptic_event;
            }
            2 => {
                let mut fixed_layer = egui_macview::MacThumbDialLayer::CompassModifiers;
                let resp =
                    MacContextThumbDial::new("state1_standalone_select_dial", &mut self.thumb_dial_latched, &mut self.thumb_dial_rotary)
                        .layer(&mut fixed_layer)
                        .undo_arc_placement(MacThumbArcPlacement::TopCrown)
                        .preset(active_preset)
                        .rotary_parameter("Select", &rot_str)
                        .can_undo(can_undo)
                        .can_redo(can_redo)
                        .scale(dial_scale)
                        .glass_accent(self.thumb_glass_color)
                        .dock_in_rect(canvas_rect, dock_zone)
                        .theme(theme)
                        .show(content_ui);
                undo_clicked = resp.undo_clicked;
                redo_clicked = resp.redo_clicked;
                nudge_delta = resp.nudge_delta;
                rotary_delta_ticks = resp.rotary_delta_ticks;
                effective_mask = resp.effective_mask;
                taptic_ev = resp.taptic_event;
            }
            3 => {
                let j_resp = MacFreeJoystickBowl::new("state1_standalone_joystick_bowl")
                    .undo_arc_placement(MacThumbArcPlacement::TopCrown)
                    .can_undo(can_undo)
                    .can_redo(can_redo)
                    .scale(dial_scale)
                    .glass_accent(self.thumb_glass_color)
                    .dock_in_rect(canvas_rect, dock_zone)
                    .theme(theme)
                    .show(content_ui);
                undo_clicked = j_resp.undo_clicked;
                redo_clicked = j_resp.redo_clicked;
                nudge_delta = j_resp.nudge_delta;
                taptic_ev = j_resp.taptic_event;
            }
            _ => {
                let resp = MacContextThumbDial::new("state1_canvas_thumb_dial", &mut self.thumb_dial_latched, &mut self.thumb_dial_rotary)
                    .layer(&mut self.thumb_dial_layer)
                    .undo_arc_placement(MacThumbArcPlacement::TopCrown)
                    .preset(active_preset)
                    .rotary_parameter("Tap: Mode", &rot_str)
                    .can_undo(can_undo)
                    .can_redo(can_redo)
                    .scale(dial_scale)
                    .glass_accent(self.thumb_glass_color)
                    .dock_in_rect(canvas_rect, dock_zone)
                    .theme(theme)
                    .show(content_ui);
                let mut ev = resp.taptic_event;
                if self.thumb_dial_layer == egui_macview::MacThumbDialLayer::CompassModifiers {
                    let drum_w = resp.response.rect.width() * 0.72;
                    let wheel_anchor_rect = Rect::from_min_max(
                        pos2(resp.response.rect.min.x, canvas_rect.min.y + 4.0),
                        pos2(resp.response.rect.max.x, (resp.response.rect.min.y - 6.0).max(canvas_rect.min.y + 36.0)),
                    );
                    let w_resp = MacEdgeThumbwheel::new("state1_edge_thumbwheel", &mut wheel_deg, 0.0..=360.0)
                        .wide_drum_width(drum_w)
                        .scale(dial_scale)
                        .glass_accent(self.thumb_glass_color)
                        .dock_right_edge(wheel_anchor_rect)
                        .theme(theme)
                        .show(content_ui);
                    ev = ev.or(w_resp.taptic_event);
                }
                taptic_ev = ev;
                undo_clicked = resp.undo_clicked;
                redo_clicked = resp.redo_clicked;
                nudge_delta = resp.nudge_delta;
                rotary_delta_ticks = resp.rotary_delta_ticks;
                effective_mask = resp.effective_mask;
            }
        }
        if let Some(ev) = taptic_ev {
            super::audio_media::NativeAudioOutput::play_wav(&ev.synthesize_micro_click_wav());
        }

        if undo_clicked && args.undo_manager.borrow_mut().undo(card) {
            *args.last_stable_card.borrow_mut() = card.clone();
            toasts.borrow_mut().info("Thumb Dial: Undone");
        }
        if redo_clicked && args.undo_manager.borrow_mut().redo(card) {
            *args.last_stable_card.borrow_mut() = card.clone();
            toasts.borrow_mut().info("Thumb Dial: Redone");
        }
        if nudge_delta != egui::Vec2::ZERO && !args.lock_canvas {
            self.card_drag_offset += nudge_delta;
        }
        let mut wheel_delta = (wheel_deg - prev_rot) % 360.0;
        if wheel_delta > 180.0 {
            wheel_delta -= 360.0;
        } else if wheel_delta < -180.0 {
            wheel_delta += 360.0;
        }
        let total_deg_delta = wheel_delta + rotary_delta_ticks * 5.0;
        if total_deg_delta.abs() > 1e-4 && !args.lock_canvas {
            let has_c = effective_mask.contains(MacThumbIntent::Constrain);
            let has_a = effective_mask.contains(MacThumbIntent::Alter);
            let has_g = effective_mask.contains(MacThumbIntent::Glide);
            let has_m = effective_mask.contains(MacThumbIntent::MovePan);
            if has_c && has_a {
                card.alignment_idx = 1;
                card.constrain_proportions = true;
                card.scale_pct = (card.scale_pct + total_deg_delta * 0.8).clamp(25.0, 250.0);
            } else if has_c {
                card.constrain_proportions = true;
                card.scale_pct = (card.scale_pct + total_deg_delta * 0.8).clamp(25.0, 250.0);
            } else if has_g && has_a {
                card.skew_deg = (card.skew_deg + total_deg_delta * 0.35).clamp(-45.0, 45.0);
            } else if has_a {
                let step = if total_deg_delta > 0.0 { 1usize } else { 8usize };
                let next_idx = ((self.origin_anchor as usize) + step) % 9;
                self.origin_anchor = egui_macview::MacOriginAnchor::ALL[next_idx];
            } else if has_g {
                let cur_r = card.radius.parse::<f32>().unwrap_or(10.0);
                card.radius = format!("{:.1}", (cur_r + total_deg_delta * 0.35).clamp(0.0, 60.0));
            } else if has_m {
                self.card_drag_offset.x += total_deg_delta * 1.5;
            } else {
                card.rotation_deg = (prev_rot + total_deg_delta).rem_euclid(360.0);
            }
        }
        if effective_mask.contains(MacThumbIntent::Constrain) {
            card.constrain_proportions = true;
        }
        if effective_mask.contains(MacThumbIntent::Alter) && effective_mask.contains(MacThumbIntent::Constrain) {
            card.alignment_idx = 1;
        }
        if effective_mask.contains(MacThumbIntent::Glide)
            && effective_mask.contains(MacThumbIntent::Alter)
            && total_deg_delta.abs() <= 1e-4
            && card.skew_deg.abs() < 1e-4
        {
            card.skew_deg = 12.0;
        }
    }
}
