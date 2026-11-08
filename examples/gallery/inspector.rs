//! 3-Tier Right Inspector (`DefaultNib`, `CompactHud`, `ExpandedPro`) with `.numeric(...)` stepping.

use crate::{CardModel, ExpandedProTab, InspectorMode};
use egui::{
    pos2, vec2, Align, Align2, CornerRadius, FontId, Id, Layout, Rect, Sense, Stroke, StrokeKind,
    Ui,
};
use egui_macview::{
    theme::MacTheme, MacButtonCell, MacColorWell, MacForm, MacInputCell, MacPopUpButton,
    MacScrollView, MacSegmentedControl, MacSlider, MacStackOrientation, MacStackView, MacStepper,
    MacToastManager, MacToggle,
};
use std::cell::RefCell;

pub struct InspectorState {
    pub popup_selected: usize,
    pub unit_segmented_idx: usize,
    pub snapping_enabled: bool,
    pub expanded_tab: ExpandedProTab,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            popup_selected: 0,
            unit_segmented_idx: 0,
            snapping_enabled: true,
            expanded_tab: ExpandedProTab::Attributes,
        }
    }
}

/// Returns `(width_scale, height_scale, radius_scale, unit_suffix)` for unit index (`0=pt, 1=px(2x Retina), 2=%`).
pub fn unit_scale_and_suffix(unit_idx: usize) -> (f32, f32, f32, &'static str) {
    match unit_idx {
        1 => (2.0, 2.0, 2.0, "px"),
        2 => (100.0 / 260.0, 100.0 / 150.0, 100.0 / 40.0, "%"),
        _ => (1.0, 1.0, 1.0, "pt"),
    }
}

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

/// Returns `(width_label, height_label, radius_label)` static strings for unit index (`0=pt, 1=px, 2=%`).
pub const fn unit_row_labels(unit_idx: usize) -> (&'static str, &'static str, &'static str) {
    match unit_idx {
        1 => ("Width (px):", "Height (px):", "Radius (px):"),
        2 => ("Width (%):", "Height (%):", "Radius (%):"),
        _ => ("Width (pt):", "Height (pt):", "Radius (pt):"),
    }
}

/// Renders the shared Width, Height, and Radius numeric `MacInputCell` rows with tab-chain focus and aspect locking.
pub(crate) fn render_card_geometry_rows(
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
        MacInputCell::with_id(id_w, &mut card.width).theme(theme).numeric(w_range, 1.0).width(68.0).next_focus(id_h).prev_focus(id_r).placeholder("Width").show(ui);
        sync_aspect_width_changed(card, &prev_w, unit_idx);
    });
    form.row_label(egui::RichText::new(h_lbl).size(11.0), |ui| {
        let prev_h = card.height.clone();
        let h_range = ((40.0 * h_scale) as f64)..=((300.0 * h_scale) as f64);
        MacInputCell::with_id(id_h, &mut card.height).theme(theme).numeric(h_range, 1.0).width(68.0).next_focus(id_r).prev_focus(id_w).placeholder("Height").show(ui);
        sync_aspect_height_changed(card, &prev_h, unit_idx);
    });
    form.row_label(egui::RichText::new(r_lbl).size(11.0), |ui| {
        let r_range = 0.0..=((40.0 * r_scale) as f64);
        MacInputCell::with_id(id_r, &mut card.radius).theme(theme).numeric(r_range, 1.0).width(68.0).next_focus(id_w).prev_focus(id_h).placeholder("Radius").show(ui);
    });
}

impl InspectorState {
    pub fn render(&mut self, ui: &mut Ui, theme: &MacTheme, card_cell: &RefCell<CardModel>, mode: &mut InspectorMode, toasts: &RefCell<MacToastManager>) {
        let mut card = card_cell.borrow_mut();
        let avail_rect = ui.available_rect_before_wrap();
        ui.painter_at(avail_rect).rect_filled(avail_rect, CornerRadius::ZERO, theme.window_background());

        MacScrollView::new("gallery_inspector_scroll")
            .theme(theme)
            .vertical(true)
            .horizontal(false)
            .max_height(avail_rect.height())
            .max_width(avail_rect.width())
            .show(ui, |scroll_ui| {
                scroll_ui.with_layout(Layout::top_down(Align::Min), |s_ui| {
                    s_ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    match *mode {
                        InspectorMode::ExpandedPro => self.render_expanded_pro(s_ui, theme, &mut card, toasts),
                        _ => self.render_default_nib(s_ui, theme, &mut card, mode, toasts),
                    }
                });
            });
    }

    fn render_expanded_pro(&mut self, ui: &mut Ui, theme: &MacTheme, card: &mut CardModel, toasts: &RefCell<MacToastManager>) {
        ui.add_space(8.0);
        let tab_idx = self.expanded_tab as usize;
        const TAB_ITEMS: [&str; 3] = ["\u{2630} Attributes", "\u{2699} Metrics", "\u{21BB} Matrix"];

        ui.vertical_centered(|c_ui| {
            MacSegmentedControl::new(&TAB_ITEMS, tab_idx).theme(theme).height(24.0).show(c_ui, |idx| {
                self.expanded_tab = match idx {
                    0 => ExpandedProTab::Attributes,
                    1 => ExpandedProTab::Metrics,
                    _ => ExpandedProTab::Matrix,
                };
            });
        });
        ui.add_space(6.0);

        MacStackView::new(MacStackOrientation::Vertical)
            .theme(theme)
            .spacing(10.0)
            .padding(12.0)
            .show_separators(true)
            .show(ui, |stack| match self.expanded_tab {
                ExpandedProTab::Attributes => {
                    stack.add_section(Some("Layer Identity"), |sec_ui| {
                        MacForm::new("exp_layer_identity_form").theme(theme).show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Name:").size(11.0), |ui| {
                                MacInputCell::with_id(Id::NULL.with("exp_layer_name"), &mut card.layer_name).theme(theme).width(180.0).show(ui);
                            });
                        });
                    });

                    stack.add_section(Some("Appearance & Blending"), |sec_ui| {
                        let blend_modes = ["Normal", "Multiply", "Screen", "Overlay"];
                        MacForm::new("exp_appearance_form").theme(theme).show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Opacity:").size(11.0), |ui| {
                                MacSlider::new(&mut card.opacity, 0.0..=100.0).theme(theme).ticks(5, true).width(120.0).show(ui);
                                ui.label(egui::RichText::new(format!("{:.0}%", card.opacity)).size(10.5));
                            });
                            form.row_label(egui::RichText::new("Blend:").size(11.0), |ui| {
                                MacPopUpButton::new(&mut card.blend_mode_idx, &blend_modes).theme(theme).width(140.0).show(ui);
                            });
                        });
                    });

                    stack.add_section(Some("Color Styles"), |sec_ui| {
                        MacForm::new("exp_color_styles_form").theme(theme).show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Fill Color:").size(11.0), |ui| {
                                MacColorWell::new(&mut card.fill_color).theme(theme).show(ui);
                            });
                            form.row_label(egui::RichText::new("Alpha Well:").size(11.0), |ui| {
                                MacColorWell::new(&mut card.alpha_color).theme(theme).show_alpha(true).show(ui);
                            });
                        });
                    });

                    stack.add_section(Some("Shadow & Effects"), |sec_ui| {
                        MacForm::new("exp_shadow_effects_form").theme(theme).show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Drop Shadow:").size(11.0), |ui| {
                                ui.add(MacToggle::new(&mut card.shadow_enabled).theme(theme));
                            });
                            if card.shadow_enabled {
                                form.row_label(egui::RichText::new("Blur Radius:").size(11.0), |ui| {
                                    MacStepper::new(&mut card.shadow_blur, 0.0..=24.0, 1.0).theme(theme).show(ui);
                                    ui.label(egui::RichText::new(format!("{:.0} pt", card.shadow_blur)).size(10.5));
                                });
                            }
                        });
                    });
                }
                ExpandedProTab::Metrics => {
                    let ids = (Id::NULL.with("exp_input_w"), Id::NULL.with("exp_input_h"), Id::NULL.with("exp_input_r"));

                    stack.add_section(Some("Dimensions & Corner Radius (↑/↓ Step)"), |sec_ui| {
                        MacForm::new("exp_dimensions_form").theme(theme).show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Units:").size(11.0), |ui| {
                                MacSegmentedControl::new(&["Points", "Pixels", "Percent"], self.unit_segmented_idx).theme(theme).show(ui, |idx| {
                                    let prev = self.unit_segmented_idx;
                                    self.unit_segmented_idx = idx;
                                    convert_card_units(card, prev, idx);
                                });
                            });
                            render_card_geometry_rows(form, theme, card, self.unit_segmented_idx, ids);
                            form.row_label(egui::RichText::new("Constrain Aspect:").size(11.0), |ui| {
                                ui.add(MacToggle::new(&mut card.constrain_proportions).theme(theme));
                            });
                        });
                    });

                    stack.add_section(Some("Layout & Alignment Mask"), |sec_ui| {
                        MacForm::new("exp_align_mask_form").theme(theme).show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Align:").size(11.0), |ui| {
                                MacSegmentedControl::new(&["Left", "Center", "Right", "Fill"], card.alignment_idx).theme(theme).show(ui, |idx| {
                                    card.alignment_idx = idx;
                                });
                            });
                        });
                    });
                }
                ExpandedProTab::Matrix => {
                    stack.add_section(Some("2x2 Affine Transform Matrix [a, b; c, d]"), |sec_ui| {
                        let (a, b, c, d) = card.affine_matrix(card.rotation_deg, card.scale_pct / 100.0);

                        let matrix_box = sec_ui.available_rect_before_wrap();
                        let m_rect = Rect::from_min_size(matrix_box.min, vec2(matrix_box.width(), 48.0));
                        let mp = sec_ui.painter_at(m_rect);
                        mp.rect_filled(m_rect, CornerRadius::same(4), theme.control());
                        mp.rect_stroke(m_rect, CornerRadius::same(4), Stroke::new(1.0, theme.separator()), StrokeKind::Inside);
                        let font = FontId::monospace(11.0);
                        mp.text(pos2(m_rect.center().x, m_rect.min.y + 14.0), Align2::CENTER_CENTER, format!("│ {:>+6.3}   {:>+6.3} │", a, b), font.clone(), theme.label());
                        mp.text(pos2(m_rect.center().x, m_rect.min.y + 34.0), Align2::CENTER_CENTER, format!("│ {:>+6.3}   {:>+6.3} │", c, d), font, theme.label());
                        sec_ui.allocate_exact_size(vec2(m_rect.width(), 50.0), Sense::hover());
                    });

                    stack.add_section(Some("Continuous Transformation Controls"), |sec_ui| {
                        let mut rot_f64 = card.rotation_deg as f64;
                        MacForm::new("exp_matrix_controls_form").theme(theme).show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Rotation:").size(11.0), |ui| {
                                MacSlider::new(&mut card.rotation_deg, 0.0..=360.0).theme(theme).ticks(8, true).width(120.0).show(ui);
                                ui.label(egui::RichText::new(format!("{:.0}°", card.rotation_deg)).size(10.5));
                            });
                            form.row_label(egui::RichText::new("Fine Step:").size(11.0), |ui| {
                                if ui.add(MacStepper::new(&mut rot_f64, 0.0..=360.0, 5.0).theme(theme).wraps(true)).changed() {
                                    card.rotation_deg = rot_f64 as f32;
                                }
                                ui.label(egui::RichText::new("±5.0°").size(10.5));
                            });
                            form.row_label(egui::RichText::new("Scale:").size(11.0), |ui| {
                                MacSlider::new(&mut card.scale_pct, 50.0..=200.0).theme(theme).ticks(6, true).width(120.0).show(ui);
                                ui.label(egui::RichText::new(format!("{:.0}%", card.scale_pct)).size(10.5));
                            });
                            form.row_label(egui::RichText::new("Skew / Shear:").size(11.0), |ui| {
                                MacSlider::new(&mut card.skew_deg, -45.0..=45.0).theme(theme).width(120.0).show(ui);
                                ui.label(egui::RichText::new(format!("{:.0}°", card.skew_deg)).size(10.5));
                            });
                        });
                        sec_ui.add_space(6.0);
                        if sec_ui.add(MacButtonCell::new("\u{21BA} Reset to Identity Matrix").theme(theme)).clicked() {
                            (card.rotation_deg, card.scale_pct, card.skew_deg) = (0.0, 100.0, 0.0);
                            toasts.borrow_mut().info("Transform reset");
                        }
                    });
                }
            });
    }
}
