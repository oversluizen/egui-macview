//! Default NIB / Compact HUD Inspector panel implementation & shared geometry helpers.

use super::inspector::InspectorState;
use super::CenterState;
use crate::{CardModel, InspectorMode};
use egui::{pos2, vec2, CornerRadius, Id, Rect, Sense, Ui};
use egui_macview::{
    theme::MacTheme, MacButtonCell, MacColorWell, MacControlSize, MacFloatingInfoBox, MacForm,
    MacInputCell, MacPopUpButton, MacPopover, MacPopoverArrowEdge, MacSegmentedControl, MacSlider,
    MacStackOrientation, MacStackView, MacStepper, MacThumbnailCell, MacToastManager, MacToggle,
};
use std::cell::RefCell;

pub use super::types::{unit_row_labels, unit_scale_and_suffix};

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
        if let (Ok(old_w), Ok(new_w), Ok(old_h)) = (
            old_w_str.parse::<f32>(),
            card.width.parse::<f32>(),
            card.height.parse::<f32>(),
        ) {
            if old_w > 1.0 {
                card.height = format!(
                    "{:.1}",
                    (old_h * (new_w / old_w)).clamp(40.0 * h_scale, 300.0 * h_scale)
                );
            }
        }
    }
}

pub(super) fn sync_aspect_height_changed(card: &mut CardModel, old_h_str: &str, unit_idx: usize) {
    if card.constrain_proportions && card.height != old_h_str {
        let (w_scale, _, _, _) = unit_scale_and_suffix(unit_idx);
        if let (Ok(old_h), Ok(new_h), Ok(old_w)) = (
            old_h_str.parse::<f32>(),
            card.height.parse::<f32>(),
            card.width.parse::<f32>(),
        ) {
            if old_h > 1.0 {
                card.width = format!(
                    "{:.1}",
                    (old_w * (new_h / old_h)).clamp(60.0 * w_scale, 500.0 * w_scale)
                );
            }
        }
    }
}

/// Renders the shared Width, Height, and Radius numeric `MacInputCell` rows with tab-chain focus and aspect locking.
pub fn render_card_geometry_rows(
    form: &mut egui_macview::MacFormUi<'_>,
    theme: &MacTheme,
    card: &mut CardModel,
    unit_idx: usize,
    ids: (Id, Id, Id),
) {
    let (id_w, id_h, id_r) = ids;
    let (w_scale, h_scale, r_scale, _) = unit_scale_and_suffix(unit_idx);
    let (w_lbl, h_lbl, r_lbl) = unit_row_labels(unit_idx);
    form.row_label(egui::RichText::new(w_lbl).size(11.0), |ui| {
        let prev_w = card.width.clone();
        let w_range = ((60.0 * w_scale) as f64)..=((500.0 * w_scale) as f64);
        MacInputCell::with_id(id_w, &mut card.width)
            .theme(theme)
            .numeric(w_range, 1.0)
            .width(68.0)
            .next_focus(id_h)
            .prev_focus(id_r)
            .placeholder("Width")
            .show(ui);
        sync_aspect_width_changed(card, &prev_w, unit_idx);
    });
    form.row_label(egui::RichText::new(h_lbl).size(11.0), |ui| {
        let prev_h = card.height.clone();
        let h_range = ((40.0 * h_scale) as f64)..=((300.0 * h_scale) as f64);
        MacInputCell::with_id(id_h, &mut card.height)
            .theme(theme)
            .numeric(h_range, 1.0)
            .width(68.0)
            .next_focus(id_r)
            .prev_focus(id_w)
            .placeholder("Height")
            .show(ui);
        sync_aspect_height_changed(card, &prev_h, unit_idx);
    });
    form.row_label(egui::RichText::new(r_lbl).size(11.0), |ui| {
        let r_range = 0.0..=((40.0 * r_scale) as f64);
        MacInputCell::with_id(id_r, &mut card.radius)
            .theme(theme)
            .numeric(r_range, 1.0)
            .width(68.0)
            .next_focus(id_w)
            .prev_focus(id_h)
            .placeholder("Radius")
            .show(ui);
    });
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
            let mut hud_abs_pos = pos2(
                canvas_rect.min.x + self.hud_pos.x,
                canvas_rect.min.y + self.hud_pos.y,
            );
            let label_col = theme.hud_secondary_text();
            let font_pt = theme.text_field().compact_infobox.font_size.points();
            let cell_w = theme.text_field().compact_infobox.default_width.points();
            let (hud_w_id, hud_h_id, hud_r_id) = (
                Id::NULL.with("hud_w"),
                Id::NULL.with("hud_h"),
                Id::NULL.with("hud_r"),
            );
            let (w_range, h_range, r_range) = (
                ((60.0 * w_scale) as f64)..=((500.0 * w_scale) as f64),
                ((40.0 * h_scale) as f64)..=((300.0 * h_scale) as f64),
                0.0..=((40.0 * r_scale) as f64),
            );
            MacFloatingInfoBox::new("gallery_compact_hud", &mut hud_abs_pos)
                .theme(theme)
                .title("HUD Palette")
                .bounds(canvas_rect)
                .show(
                    content_ui.ctx(),
                    |row1_ui| {
                        row1_ui.label(egui::RichText::new("W:").size(font_pt).color(label_col));
                        MacInputCell::with_id(hud_w_id, &mut card.width)
                            .theme(theme)
                            .control_size(MacControlSize::CompactInfobox)
                            .numeric(w_range, 1.0)
                            .width(cell_w)
                            .prev_focus(hud_r_id)
                            .next_focus(hud_h_id)
                            .show(row1_ui);
                        row1_ui.add_space(4.0);
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
                        row2_ui.add_space(4.0);
                        row2_ui.label(egui::RichText::new("θ:").size(font_pt).color(label_col));
                        MacSlider::new(&mut card.rotation_deg, 0.0..=360.0)
                            .theme(theme)
                            .width(55.0)
                            .show(row2_ui);
                    },
                );
            self.hud_pos = pos2(
                hud_abs_pos.x - canvas_rect.min.x,
                hud_abs_pos.y - canvas_rect.min.y,
            );
        }

        content_ui.add_space(12.0);
        let btn_text = if self.popover_open {
            "Dismiss Spring Popover"
        } else {
            "Present Spring Popover"
        };
        if content_ui
            .add(MacButtonCell::new(btn_text).theme(theme))
            .clicked()
        {
            self.popover_open = !self.popover_open;
            self.spring_start_time = Some(content_ui.input(|i| i.time));
        }
        if self.popover_open {
            content_ui.add_space(6.0);
            let (pop_rect, _) = content_ui.allocate_exact_size(vec2(320.0, 96.0), Sense::hover());
            MacPopover::new()
                .theme(theme)
                .arrow_edge(MacPopoverArrowEdge::Top)
                .show(content_ui, pop_rect, |pop_ui| {
                    pop_ui.add_space(6.0);
                    pop_ui.label(
                        egui::RichText::new(format!(
                            "CASpring Harmonic Oscillator (T₀={:.2}s, ζ={:.2})",
                            self.spring.response, self.spring.damping_ratio
                        ))
                        .size(11.5)
                        .strong(),
                    );
                    pop_ui.label(
                        egui::RichText::new(format!(
                            "Settling Duration: {:.3}s | Angular Freq ω₀: {:.2} rad/s",
                            self.spring.settling_duration(),
                            self.spring.omega_0()
                        ))
                        .size(10.5),
                    );
                });
        }
        content_ui.add_space(18.0);
    }
}

impl InspectorState {
    pub(super) fn render_default_nib(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        card: &mut CardModel,
        mode: &mut InspectorMode,
        toasts: &RefCell<MacToastManager>,
    ) {
        let input_id_w = Id::NULL.with("nib_input_w");
        let input_id_h = Id::NULL.with("nib_input_h");
        let input_id_r = Id::NULL.with("nib_input_r");

        MacStackView::new(MacStackOrientation::Vertical)
            .theme(theme)
            .spacing(10.0)
            .padding(12.0)
            .show_separators(true)
            .show(ui, |stack| {
                stack.add_section(Some("Preview (MacThumbnailCell)"), |sec_ui| {
                    MacForm::new("nib_preview_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Vector Badge:").size(11.0), |ui| {
                                MacThumbnailCell::new()
                                    .theme(theme)
                                    .selected(true)
                                    .custom_draw(|painter, rect| {
                                        painter.circle_filled(
                                            rect.center(),
                                            7.0,
                                            theme.control_accent(),
                                        );
                                        painter.rect_filled(
                                            rect.shrink(5.0),
                                            CornerRadius::same(2),
                                            theme.window_background(),
                                        );
                                    })
                                    .show(ui);
                            });
                        });
                });

                let (_, _, r_scale, unit_str) = unit_scale_and_suffix(self.unit_segmented_idx);
                stack.add_section(Some("Geometry & Tab-Chain (↑/↓ Numeric)"), |sec_ui| {
                    MacForm::new("nib_geometry_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            render_card_geometry_rows(
                                form,
                                theme,
                                card,
                                self.unit_segmented_idx,
                                (input_id_w, input_id_h, input_id_r),
                            );
                        });
                });

                stack.add_section(Some("Continuous Controls (Slider & Stepper)"), |sec_ui| {
                    MacForm::new("nib_continuous_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Card Scale:").size(11.0), |ui| {
                                MacSlider::new(&mut card.scale_pct, 25.0..=200.0)
                                    .theme(theme)
                                    .ticks(7, true)
                                    .width(110.0)
                                    .show(ui);
                                ui.label(
                                    egui::RichText::new(format!("{:.0}%", card.scale_pct))
                                        .size(10.5),
                                );
                            });
                            form.row_label(
                                egui::RichText::new("Corner Radius:").size(11.0),
                                |ui| {
                                    let max_r = (40.0 * r_scale) as f64;
                                    let mut r =
                                        card.radius.parse::<f64>().unwrap_or(10.0 * r_scale as f64);
                                    let prev_r = r;
                                    MacStepper::new(&mut r, 0.0..=max_r, 1.0)
                                        .theme(theme)
                                        .wraps(false)
                                        .show(ui);
                                    if (r - prev_r).abs() > 0.01 {
                                        card.radius = format!("{:.1}", r);
                                    }
                                    ui.label(
                                        egui::RichText::new(format!("{:.1} {}", r, unit_str))
                                            .size(10.5),
                                    );
                                },
                            );
                        });
                });

                stack.add_section(Some("Selection (PopUp & Segmented)"), |sec_ui| {
                    let popup_items = ["Default NIB", "Compact HUD", "Expanded Pro"];
                    self.popup_selected = match *mode {
                        InspectorMode::DefaultNib => 0,
                        InspectorMode::CompactHud => 1,
                        InspectorMode::ExpandedPro => 2,
                    };
                    MacForm::new("nib_selection_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Preset:").size(11.0), |ui| {
                                let prev_sel = self.popup_selected;
                                MacPopUpButton::new(&mut self.popup_selected, &popup_items)
                                    .theme(theme)
                                    .width(135.0)
                                    .show(ui);
                                if self.popup_selected != prev_sel {
                                    *mode = match self.popup_selected {
                                        0 => InspectorMode::DefaultNib,
                                        1 => InspectorMode::CompactHud,
                                        _ => InspectorMode::ExpandedPro,
                                    };
                                    toasts.borrow_mut().info("Preset applied");
                                }
                            });
                            form.row_label(egui::RichText::new("Units:").size(11.0), |ui| {
                                MacSegmentedControl::new(
                                    &["Points", "Pixels", "Percent"],
                                    self.unit_segmented_idx,
                                )
                                .theme(theme)
                                .show(ui, |idx| {
                                    let prev_u = self.unit_segmented_idx;
                                    self.unit_segmented_idx = idx;
                                    convert_card_units(card, prev_u, idx);
                                });
                            });
                        });
                });

                stack.add_section(Some("State & Color (Toggle & ColorWell)"), |sec_ui| {
                    MacForm::new("nib_state_color_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            form.row_label(
                                egui::RichText::new("Subpixel Snap:").size(11.0),
                                |ui| {
                                    ui.add(MacToggle::new(&mut self.snapping_enabled).theme(theme));
                                },
                            );
                            form.row_label(
                                egui::RichText::new("Accent Swatch:").size(11.0),
                                |ui| {
                                    MacColorWell::new(&mut card.fill_color)
                                        .theme(theme)
                                        .show(ui);
                                },
                            );
                            form.row_label(egui::RichText::new("Alpha Well:").size(11.0), |ui| {
                                MacColorWell::new(&mut card.alpha_color)
                                    .theme(theme)
                                    .show_alpha(true)
                                    .show(ui);
                            });
                        });
                });
            });
    }
}
