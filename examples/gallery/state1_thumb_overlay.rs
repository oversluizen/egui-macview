//! Thumb Dial, Rotary Edge Wheel & Free Joystick Bowl overlay for State 1 (`GalleryScreen::CanvasPhysics`).

use super::CenterState;
use crate::CardModel;
use egui::{pos2, vec2, Rect, Ui};
use egui_macview::{
    controls::MacControlSize, theme::MacTheme, MacColorWell, MacContextThumbDial, MacEdgeThumbwheel, MacFreeJoystickBowl,
    MacSegmentedControl, MacThumbArcPlacement, MacThumbContextPreset, MacThumbDockZone, MacThumbIntent, MacThumbIntentMask,
    MacToastManager,
};
use std::cell::RefCell;

struct ThumbOverlayTelemetry {
    undo_clicked: bool,
    redo_clicked: bool,
    nudge_delta: egui::Vec2,
    rotary_delta_ticks: f32,
    wheel_value_delta: f32,
    effective_mask: MacThumbIntentMask,
    taptic_event: Option<egui_macview::MacTapticEvent>,
}

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
        let is_compact = args.inspector_mode == crate::InspectorMode::CompactHud;
        self.render_thumb_mode_strips(content_ui, theme, canvas_rect, is_compact);

        let telemetry = self.render_active_thumb_controller(content_ui, theme, card, args, canvas_rect, is_compact);
        if let Some(ev) = telemetry.taptic_event {
            super::audio_media::NativeAudioOutput::play_wav(&ev.synthesize_micro_click_wav());
        }
        if telemetry.undo_clicked && args.undo_manager.borrow_mut().undo(card) {
            *args.last_stable_card.borrow_mut() = card.clone();
            toasts.borrow_mut().info("Thumb Dial: Undone");
        }
        if telemetry.redo_clicked && args.undo_manager.borrow_mut().redo(card) {
            *args.last_stable_card.borrow_mut() = card.clone();
            toasts.borrow_mut().info("Thumb Dial: Redone");
        }
        self.apply_artboard_thumb_telemetry(content_ui.ctx(), theme, card, args.lock_canvas, telemetry);
    }

    fn render_thumb_mode_strips(&mut self, content_ui: &mut Ui, theme: &MacTheme, canvas_rect: Rect, is_compact: bool) {
        let pill_labels = ["Pen (P)", "Select (V)", "Prim (F)", "Knife (E)"];
        let strip_w = 184.0f32;
        let strip_x = if is_compact { canvas_rect.max.x - strip_w - 8.0 } else { canvas_rect.min.x + 10.0 };
        let strip_rect = Rect::from_min_size(pos2(strip_x, canvas_rect.min.y + 6.0), vec2(strip_w, 18.0));
        let mut preset_idx = self.thumb_dial_preset_idx;
        MacSegmentedControl::new(&pill_labels, preset_idx)
            .control_size(MacControlSize::Mini)
            .fixed_rect(strip_rect)
            .id_salt("thumb_preset_seg")
            .theme(theme)
            .show(content_ui, |i| preset_idx = i);
        self.thumb_dial_preset_idx = preset_idx;

        let cfg_labels = ["Paired", "Rotary", "Select", "Joystick"];
        let cfg_rect = Rect::from_min_size(pos2(strip_rect.min.x, strip_rect.max.y + 4.0), vec2(strip_w - 36.0, 18.0));
        let mut cfg_idx = self.thumb_config_mode;
        MacSegmentedControl::new(&cfg_labels, cfg_idx)
            .control_size(MacControlSize::Mini)
            .fixed_rect(cfg_rect)
            .id_salt("thumb_cfg_seg")
            .theme(theme)
            .show(content_ui, |i| cfg_idx = i);
        self.thumb_config_mode = cfg_idx;

        let well_rect = Rect::from_min_size(pos2(cfg_rect.max.x + 2.0, cfg_rect.min.y), vec2(34.0, 18.0));
        MacColorWell::new(&mut self.thumb_glass_color)
            .id_salt("thumb_glass_color_swatch")
            .fixed_rect(well_rect)
            .theme(theme)
            .show(content_ui);
    }

    fn render_active_thumb_controller(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        card: &CardModel,
        args: &super::center_state::CenterRenderArgs<'_>,
        canvas_rect: Rect,
        is_compact: bool,
    ) -> ThumbOverlayTelemetry {
        let presets = MacThumbContextPreset::ALL;
        let active_preset = presets[self.thumb_dial_preset_idx.min(presets.len() - 1)];
        let dock_zone = if is_compact { MacThumbDockZone::BottomLeft } else { MacThumbDockZone::BottomRight };
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

        match self.thumb_config_mode {
            1 => {
                let drum_w = (theme.thumb_dial().outer_radius.points() * 2.0 * dial_scale) * 0.72;
                let w_resp = MacEdgeThumbwheel::new("state1_standalone_rotary", &mut self.thumbwheel_angle_deg, -10_000.0..=10_000.0)
                    .wide_drum_width(drum_w)
                    .scale(dial_scale)
                    .glass_accent(self.thumb_glass_color)
                    .dock_right_edge(canvas_rect)
                    .theme(theme)
                    .show(content_ui);
                ThumbOverlayTelemetry {
                    undo_clicked: false,
                    redo_clicked: false,
                    nudge_delta: egui::Vec2::ZERO,
                    rotary_delta_ticks: 0.0,
                    wheel_value_delta: w_resp.delta_ticks * 5.0,
                    effective_mask: self.thumb_dial_latched,
                    taptic_event: w_resp.taptic_event,
                }
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
                ThumbOverlayTelemetry {
                    undo_clicked: resp.undo_clicked,
                    redo_clicked: resp.redo_clicked,
                    nudge_delta: resp.nudge_delta,
                    rotary_delta_ticks: resp.rotary_delta_ticks,
                    wheel_value_delta: 0.0,
                    effective_mask: resp.effective_mask,
                    taptic_event: resp.taptic_event,
                }
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
                ThumbOverlayTelemetry {
                    undo_clicked: j_resp.undo_clicked,
                    redo_clicked: j_resp.redo_clicked,
                    nudge_delta: j_resp.nudge_delta,
                    rotary_delta_ticks: 0.0,
                    wheel_value_delta: 0.0,
                    effective_mask: MacThumbIntentMask::NONE,
                    taptic_event: j_resp.taptic_event,
                }
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
                let mut wheel_deg_delta = 0.0f32;
                if self.thumb_dial_layer == egui_macview::MacThumbDialLayer::CompassModifiers {
                    let drum_w = resp.response.rect.width() * 0.72;
                    let wheel_anchor_rect = Rect::from_min_max(
                        pos2(resp.response.rect.min.x, canvas_rect.min.y + 4.0),
                        pos2(resp.response.rect.max.x, (resp.response.rect.min.y - 6.0).max(canvas_rect.min.y + 36.0)),
                    );
                    let w_resp = MacEdgeThumbwheel::new("state1_edge_thumbwheel", &mut self.thumbwheel_angle_deg, -10_000.0..=10_000.0)
                        .wide_drum_width(drum_w)
                        .scale(dial_scale)
                        .glass_accent(self.thumb_glass_color)
                        .dock_right_edge(wheel_anchor_rect)
                        .theme(theme)
                        .show(content_ui);
                    wheel_deg_delta = w_resp.delta_ticks * 5.0;
                    ev = ev.or(w_resp.taptic_event);
                }
                ThumbOverlayTelemetry {
                    undo_clicked: resp.undo_clicked,
                    redo_clicked: resp.redo_clicked,
                    nudge_delta: resp.nudge_delta,
                    rotary_delta_ticks: resp.rotary_delta_ticks,
                    wheel_value_delta: wheel_deg_delta,
                    effective_mask: resp.effective_mask,
                    taptic_event: ev,
                }
            }
        }
    }

    fn apply_artboard_thumb_telemetry(
        &mut self,
        ctx: &egui::Context,
        theme: &MacTheme,
        card: &mut CardModel,
        lock_canvas: bool,
        t: ThumbOverlayTelemetry,
    ) {
        if t.effective_mask != self.prev_thumb_effective_mask {
            self.origin_detent_accum_deg = 0.0;
            MacEdgeThumbwheel::brake_flywheel(ctx, "state1_edge_thumbwheel");
            MacEdgeThumbwheel::brake_flywheel(ctx, "state1_standalone_rotary");
            self.prev_thumb_effective_mask = t.effective_mask;
        }
        if lock_canvas {
            return;
        }
        if t.nudge_delta != egui::Vec2::ZERO {
            self.card_drag_offset += t.nudge_delta;
        }
        let total_deg_delta = t.wheel_value_delta + t.rotary_delta_ticks * 5.0;
        let has_c = t.effective_mask.contains(MacThumbIntent::Constrain);
        let has_a = t.effective_mask.contains(MacThumbIntent::Alter);
        let has_g = t.effective_mask.contains(MacThumbIntent::Glide);
        let has_m = t.effective_mask.contains(MacThumbIntent::MovePan);

        if total_deg_delta.abs() > 1e-4 {
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
                let detent_deg = theme.thumb_dial().detent_step_deg.points();
                self.origin_anchor = self.origin_anchor.step_by_detent(&mut self.origin_detent_accum_deg, total_deg_delta, detent_deg);
            } else if has_g {
                let cur_r = card.radius.parse::<f32>().unwrap_or(10.0);
                card.radius = format!("{:.1}", (cur_r + total_deg_delta * 0.35).clamp(0.0, 60.0));
            } else if has_m {
                self.card_drag_offset.x += total_deg_delta * 1.5;
            } else {
                card.rotation_deg = (card.rotation_deg + total_deg_delta).rem_euclid(360.0);
            }
        }
        if has_c {
            card.constrain_proportions = true;
        }
        if has_a && has_c {
            card.alignment_idx = 1;
        }
        if has_g && has_a && total_deg_delta.abs() <= 1e-4 && card.skew_deg.abs() < 1e-4 {
            card.skew_deg = 12.0;
        }
    }
}
