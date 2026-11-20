//! Shared geometry helpers and Compact HUD / Popover overlays (`REQ-013-02P`).

use super::CenterState;
use crate::{CardModel, InspectorMode};
use egui::{pos2, vec2, Id, Rect, Sense, Ui};
use egui_macview::{theme::MacTheme, MacControlSize, MacFloatingInfoBox, MacInputCell, MacPopover, MacPopoverArrowEdge, MacSlider};

pub use super::types::unit_scale_and_suffix;

/// Converts `CardModel` width, height, and corner radius strings in-place when switching units.
pub fn convert_card_units(card: &mut CardModel, old_unit: usize, new_unit: usize) {
    if old_unit == new_unit {
        return;
    }
    let (ow, oh, or, _) = unit_scale_and_suffix(old_unit);
    let (nw, nh, nr, _) = unit_scale_and_suffix(new_unit);
    if let Ok(v) = card.width.parse::<f32>() {
        card.width = format!("{:.1}", (v / ow) * nw);
    }
    if let Ok(v) = card.height.parse::<f32>() {
        card.height = format!("{:.1}", (v / oh) * nh);
    }
    if let Ok(v) = card.radius.parse::<f32>() {
        card.radius = format!("{:.1}", (v / or) * nr);
    }
}

pub(super) fn sync_aspect_width_changed(card: &mut CardModel, old_w_str: &str, unit_idx: usize) {
    if card.constrain_proportions && card.width != old_w_str {
        let (_, h_scale, _, _) = unit_scale_and_suffix(unit_idx);
        if let (Ok(old_w), Ok(new_w), Ok(old_h)) = (old_w_str.parse::<f32>(), card.width.parse::<f32>(), card.height.parse::<f32>()) {
            if old_w > 1.0 {
                card.height = format!("{:.1}", (old_h * (new_w / old_w)).clamp(40.0 * h_scale, 300.0 * h_scale));
            }
        }
    }
}

pub(super) fn sync_aspect_height_changed(card: &mut CardModel, old_h_str: &str, unit_idx: usize) {
    if card.constrain_proportions && card.height != old_h_str {
        let (w_scale, _, _, _) = unit_scale_and_suffix(unit_idx);
        if let (Ok(old_h), Ok(new_h), Ok(old_w)) = (old_h_str.parse::<f32>(), card.height.parse::<f32>(), card.width.parse::<f32>()) {
            if old_h > 1.0 {
                card.width = format!("{:.1}", (old_w * (new_h / old_h)).clamp(60.0 * w_scale, 500.0 * w_scale));
            }
        }
    }
}

impl CenterState {
    pub(super) fn render_state1_hud_and_popover(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        card: &mut CardModel,
        inspector_mode: InspectorMode,
        canvas_rect: Rect,
        scales: (f32, f32, f32),
    ) {
        let (w_scale, h_scale, r_scale) = scales;
        if inspector_mode == InspectorMode::CompactHud {
            if self.hud_pos.x < 196.0 {
                self.hud_pos.x = 204.0;
            }
            let mut hud_abs_pos = pos2(canvas_rect.min.x + self.hud_pos.x, canvas_rect.min.y + self.hud_pos.y);
            let label_col = theme.hud_secondary_text();
            let font_pt = theme.text_field().compact_infobox.font_size.points();
            let cell_w = theme.text_field().compact_infobox.default_width.points();
            let (hud_w_id, hud_h_id, hud_r_id) = (Id::NULL.with("hud_w"), Id::NULL.with("hud_h"), Id::NULL.with("hud_r"));
            let (w_range, h_range, r_range) = (
                ((60.0 * w_scale) as f64)..=((500.0 * w_scale) as f64),
                ((40.0 * h_scale) as f64)..=((300.0 * h_scale) as f64),
                0.0..=((40.0 * r_scale) as f64),
            );
            MacFloatingInfoBox::new("gallery_compact_hud", &mut hud_abs_pos).theme(theme).title("HUD Palette").bounds(canvas_rect).show(
                content_ui.ctx(),
                |row1_ui| {
                    row1_ui.label(egui::RichText::new("W:").size(font_pt).color(label_col));
                    MacInputCell::with_id(hud_w_id, &mut card.width)
                        .theme(theme)
                        .control_size(MacControlSize::CompactInfobox)
                        .liquid_glass_pill(true)
                        .numeric(w_range, 1.0)
                        .width(cell_w)
                        .prev_focus(hud_r_id)
                        .next_focus(hud_h_id)
                        .show(row1_ui);
                    row1_ui.label(egui::RichText::new("H:").size(font_pt).color(label_col));
                    MacInputCell::with_id(hud_h_id, &mut card.height)
                        .theme(theme)
                        .control_size(MacControlSize::CompactInfobox)
                        .numeric(h_range, 1.0)
                        .width(cell_w)
                        .prev_focus(hud_w_id)
                        .next_focus(hud_r_id)
                        .show(row1_ui);
                },
                |row2_ui| {
                    row2_ui.label(egui::RichText::new("R:").size(font_pt).color(label_col));
                    MacInputCell::with_id(hud_r_id, &mut card.radius)
                        .theme(theme)
                        .control_size(MacControlSize::CompactInfobox)
                        .numeric(r_range, 1.0)
                        .width(cell_w)
                        .prev_focus(hud_h_id)
                        .next_focus(hud_w_id)
                        .show(row2_ui);
                    row2_ui.label(egui::RichText::new("θ:").size(font_pt).color(label_col));
                    MacSlider::new(&mut card.rotation_deg, 0.0..=360.0).theme(theme).width(55.0).show(row2_ui);
                },
            );
            self.hud_pos = pos2(hud_abs_pos.x - canvas_rect.min.x, hud_abs_pos.y - canvas_rect.min.y);
        }

        if self.popover_open {
            let pop_rect = Rect::from_min_size(pos2(canvas_rect.center().x - 160.0, canvas_rect.max.y - 104.0), vec2(320.0, 96.0));
            let _ = content_ui.interact(pop_rect, content_ui.scope_id().with("state1_spring_popover_anchor"), Sense::hover());
            MacPopover::new().theme(theme).arrow_edge(MacPopoverArrowEdge::Top).show(content_ui, pop_rect, |pop_ui| {
                pop_ui.label(
                    egui::RichText::new(format!(
                        "CASpring Harmonic Oscillator (T0={:.2}s, zeta={:.2})",
                        self.spring.response, self.spring.damping_ratio
                    ))
                    .size(11.5)
                    .strong(),
                );
                pop_ui.label(
                    egui::RichText::new(format!(
                        "Settling Duration: {:.3}s | Angular Freq w0: {:.2} rad/s",
                        self.spring.settling_duration(),
                        self.spring.omega_0()
                    ))
                    .size(10.5),
                );
            });
        }
    }

    pub(super) fn pull_extended_store_bindings(
        &self,
        store: &mut super::nib_showcase_builders::ShowcaseSceneDataStore,
        card: &CardModel,
        norm: &str,
        bid: egui_macview::scene::BindingId,
        sync_strings: bool,
    ) {
        let set_num = |st: &mut super::nib_showcase_builders::ShowcaseSceneDataStore, b: egui_macview::scene::BindingId, v: f64| {
            st.floats.insert(b, v);
            if sync_strings || !st.strings.contains_key(&b) {
                st.strings.insert(b, super::nib_showcase_builders::format_nib_num(v));
            }
        };
        match norm {
            "showGrid" => {
                store.bools.insert(bid, self.show_grid);
            }
            "strokeWidth" => {
                store.floats.insert(bid, card.stroke_width as f64);
            }
            "strokeColor" => {
                store.colors.insert(bid, card.stroke_color.to_srgba_unmultiplied());
            }
            "exportPreset" if sync_strings => {
                store.strings.insert(bid, self.export_preset.clone());
            }
            "fontFamily" => {
                store.ints.insert(bid, self.typo_font_family as i64);
            }
            "trackingEm" => {
                store.floats.insert(bid, self.typo_tracking_em as f64);
            }
            "pointSize" => set_num(store, bid, self.typo_point_size as f64),
            "opticalKern" => {
                store.bools.insert(bid, self.typo_optical_kern);
            }
            "fontWeight" => {
                store.ints.insert(bid, self.typo_font_weight as i64);
            }
            "matrixInput" | "matrixText" if sync_strings => {
                store.strings.insert(bid, self.showcase_text.clone());
            }
            "matrixPopup" => {
                store.ints.insert(bid, self.showcase_popup_idx as i64);
            }
            "matrixCheck" => {
                store.bools.insert(bid, self.matrix_check);
            }
            "matrixSwitch" => {
                store.bools.insert(bid, self.matrix_switch);
            }
            "matrixSlider" => {
                store.floats.insert(bid, self.matrix_slider as f64);
            }
            "accentIndex" => {
                store.ints.insert(bid, self.accent_index as i64);
            }
            "contrastTarget" => {
                store.floats.insert(bid, self.contrast_target as f64);
            }
            "highContrast" => {
                store.bools.insert(bid, self.high_contrast);
            }
            "reduceTransparency" => {
                store.bools.insert(bid, self.reduce_transparency);
            }
            "exportPath" if sync_strings => {
                store.strings.insert(bid, self.export_path.clone());
            }
            "exportFormat" => {
                store.ints.insert(bid, self.export_format as i64);
            }
            "exportBitrate" => {
                store.floats.insert(bid, self.export_bitrate as f64);
            }
            "workerCount" => set_num(store, bid, self.worker_count as f64),
            "openAfterExport" => {
                store.bools.insert(bid, self.open_after_export);
            }
            "verifyChecksums" => {
                store.bools.insert(bid, self.verify_checksums);
            }
            "mediaSpeed" => {
                store.floats.insert(bid, self.media_speed as f64);
            }
            "satDrive" => {
                store.floats.insert(bid, self.sat_drive as f64);
            }
            "brickwallLimiter" => {
                store.bools.insert(bid, self.brickwall_limiter);
            }
            "busMode" => {
                store.ints.insert(bid, self.bus_mode as i64);
            }
            "codecPreset" => {
                store.ints.insert(bid, self.codec_preset as i64);
            }
            "pickerDay" => {
                store.ints.insert(bid, self.advanced_date.day as i64);
            }
            "proTabIndex" => {
                store.ints.insert(bid, self.advanced_tab_idx as i64);
            }
            "ruleCount" => {
                store.ints.insert(bid, self.advanced_rules.len() as i64);
            }
            "actionBarSel" | "tableActionItems" => {
                store.options.insert(bid, self.action_items.clone());
            }
            "vuLevelL" => {
                store.floats.insert(bid, (self.audio_level_l as f64 - 10.0) * 4.8);
            }
            "vuLevelR" => {
                store.floats.insert(bid, (self.audio_level_r as f64 - 10.0) * 4.8);
            }
            "vuTextL" => {
                let disp_l = if self.media_playing {
                    let bucket_t = (self.media_playhead * 5.0).floor() / 5.0;
                    let (bl, _, _, _) = self.audio_track.compute_stereo_levels_at(bucket_t, self.media_gain, self.media_pan);
                    bl.clamp(0.0, 10.0)
                } else {
                    self.audio_level_l
                };
                store.strings.insert(bid, format!("{:4.1} dB", disp_l - 10.0));
            }
            "vuTextR" => {
                let disp_r = if self.media_playing {
                    let bucket_t = (self.media_playhead * 5.0).floor() / 5.0;
                    let (_, br, _, _) = self.audio_track.compute_stereo_levels_at(bucket_t, self.media_gain, self.media_pan);
                    br.clamp(0.0, 10.0)
                } else {
                    self.audio_level_r
                };
                store.strings.insert(bid, format!("{:4.1} dB", disp_r - 10.0));
            }
            "takeRating" => {
                store.floats.insert(bid, self.media_rating as f64);
            }
            "vuDiscrete" => {
                store.bools.insert(bid, self.media_discrete_vu);
            }
            "vuThresholds" => {
                store.bools.insert(bid, self.media_threshold_vu);
            }
            "assetRating" => {
                let idx = self.selected_table_rows.iter().next().copied().unwrap_or(0).min(5);
                store.floats.insert(bid, self.asset_ratings[idx] as f64);
            }
            "assetTokens" if sync_strings => {
                let idx = self.selected_table_rows.iter().next().copied().unwrap_or(0).min(5);
                store.strings.insert(bid, self.asset_tokens[idx].join(", "));
            }
            "publishStatusText" => {
                let ver_tag = if self.verify_checksums { "SHA✓" } else { "Raw" };
                let open_tag = if self.open_after_export { " • Reveal" } else { "" };
                store.strings.insert(bid, format!("{:.0}% {ver_tag}{open_tag}", self.publish_progress * 100.0));
            }
            _ => {}
        }
    }

    pub(super) fn flush_extended_store_binding(
        &mut self,
        store: &mut super::nib_showcase_builders::ShowcaseSceneDataStore,
        card_cell: &std::cell::RefCell<CardModel>,
        norm: &str,
        bid: egui_macview::scene::BindingId,
    ) {
        use egui_macview::scene::MacSceneDataSource;
        match norm {
            "showGrid" => {
                self.show_grid = store.get_bool(bid);
            }
            "strokeWidth" => {
                card_cell.borrow_mut().stroke_width = store.get_path_f64(norm).clamp(0.0, 16.0) as f32;
            }
            "strokeColor" => {
                if let Some(col) = store.get_color(bid) {
                    card_cell.borrow_mut().stroke_color = col;
                }
            }
            "exportPreset" => {
                self.export_preset = store.get_str(bid).to_string();
            }
            "fontFamily" => {
                self.typo_font_family = store.get_i64(bid).clamp(0, 3) as usize;
            }
            "trackingEm" => {
                self.typo_tracking_em = store.get_f64(bid).clamp(-0.1, 0.25) as f32;
            }
            "pointSize" => {
                self.typo_point_size = store.get_f64(bid).clamp(8.0, 72.0) as f32;
            }
            "opticalKern" => {
                self.typo_optical_kern = store.get_bool(bid);
            }
            "fontWeight" => {
                self.typo_font_weight = store.get_i64(bid).clamp(0, 4) as usize;
            }
            "matrixInput" | "matrixText" => {
                self.showcase_text = store.get_str(bid).to_string();
            }
            "matrixPopup" => {
                self.showcase_popup_idx = store.get_i64(bid).clamp(0, 2) as usize;
            }
            "matrixCheck" => {
                self.matrix_check = store.get_bool(bid);
            }
            "matrixSwitch" => {
                self.matrix_switch = store.get_bool(bid);
            }
            "matrixSlider" => {
                self.matrix_slider = store.get_f64(bid).clamp(0.0, 100.0) as f32;
            }
            "accentIndex" => {
                let idx = store.get_i64(bid).max(0) as usize;
                self.accent_index = idx;
                let palette = MacTheme::default().tag_label_palette();
                let col = palette[idx % palette.len()];
                let mut c = card_cell.borrow_mut();
                c.fill_color = col;
                c.alpha_color = col.gamma_multiply(0.75);
                self.wcag_bg = col;
                self.symbol_tint = col;
                self.thumb_glass_color = col;
            }
            "contrastTarget" => {
                self.contrast_target = store.get_f64(bid).clamp(3.0, 15.0) as f32;
            }
            "highContrast" => {
                self.high_contrast = store.get_bool(bid);
                if self.high_contrast {
                    self.wcag_fg = egui_macview::ensure_wcag_contrast(self.wcag_fg, self.wcag_bg, 7.0);
                }
            }
            "reduceTransparency" => {
                self.reduce_transparency = store.get_bool(bid);
            }
            "exportPath" => {
                self.export_path = store.get_str(bid).to_string();
            }
            "exportFormat" => {
                self.export_format = store.get_i64(bid).clamp(0, 3) as usize;
            }
            "exportBitrate" => {
                self.export_bitrate = store.get_f64(bid).clamp(10.0, 250.0) as f32;
            }
            "workerCount" => {
                self.worker_count = store.get_f64(bid).round().clamp(1.0, 32.0) as usize;
            }
            "openAfterExport" => {
                self.open_after_export = store.get_bool(bid);
            }
            "verifyChecksums" => {
                self.verify_checksums = store.get_bool(bid);
            }
            "mediaSpeed" => {
                self.media_speed = store.get_f64(bid).clamp(0.25, 2.0) as f32;
            }
            "satDrive" => {
                self.sat_drive = store.get_f64(bid).clamp(0.0, 1.0) as f32;
            }
            "brickwallLimiter" => {
                self.brickwall_limiter = store.get_bool(bid);
            }
            "busMode" => {
                self.bus_mode = store.get_i64(bid).clamp(0, 2) as usize;
            }
            "codecPreset" => {
                let next = store.get_i64(bid).max(0) as usize;
                if next != self.codec_preset {
                    self.codec_preset = next;
                    self.media_playing = true;
                    let title = store.get_options(bid).get(next).cloned().unwrap_or_else(|| format!("Stream #{}", next + 1));
                    self.audio_track = super::audio_media::AudioTrack::from_stream_descriptor(&title, next);
                    self.media_waveform = self.audio_track.waveform_peaks.clone();
                    if self.media_sound_output {
                        let wav =
                            self.audio_track.to_wav_bytes_mixed(self.media_playhead, self.media_out_point, self.media_gain, self.media_pan);
                        super::audio_media::NativeAudioOutput::play_wav(&wav);
                    }
                }
            }
            "pickerDay" => {
                self.advanced_date.day = store.get_i64(bid).clamp(1, 28) as u32;
            }
            "proTabIndex" => {
                self.advanced_tab_idx = store.get_i64(bid).clamp(0, 2) as usize;
            }
            "ruleCount" => {
                let target_len = store.get_i64(bid).clamp(1, 12) as usize;
                while self.advanced_rules.len() < target_len {
                    self.advanced_rules.push(egui_macview::controls::MacRuleRow::new(vec![0, 0]));
                }
                self.advanced_rules.truncate(target_len);
            }
            "actionBarSel" | "tableActionItems" => {
                let delta = store.get_i64(bid);
                if delta > 0 {
                    self.action_items.push(format!("Layer #{}", self.action_items.len() + 1));
                } else if delta < 0 {
                    let _ = self.action_items.pop();
                }
                store.ints.insert(bid, 0);
            }
            "takeRating" => {
                self.media_rating = store.get_f64(bid).round().clamp(0.0, 5.0) as u32;
            }
            "vuDiscrete" => {
                self.media_discrete_vu = store.get_bool(bid);
            }
            "vuThresholds" => {
                self.media_threshold_vu = store.get_bool(bid);
            }
            "assetRating" => {
                let idx = self.selected_table_rows.iter().next().copied().unwrap_or(0).min(5);
                self.asset_ratings[idx] = store.get_f64(bid).round().clamp(0.0, 5.0) as u32;
            }
            "assetTokens" => {
                let idx = self.selected_table_rows.iter().next().copied().unwrap_or(0).min(5);
                self.asset_tokens[idx] =
                    store.get_str(bid).split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
            }
            _ => {}
        }
    }
}
