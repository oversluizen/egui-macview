//! Default NIB / Compact HUD Inspector panel implementation.

use super::inspector::{sync_aspect_height_changed, sync_aspect_width_changed, InspectorState};
use crate::{CardModel, InspectorMode};
use egui::{CornerRadius, Id, Ui};
use egui_macview::{
    theme::MacTheme, MacColorWell, MacForm, MacInputCell, MacPopUpButton, MacSegmentedControl,
    MacSlider, MacStackOrientation, MacStackView, MacStepper, MacThumbnailCell, MacToastManager,
    MacToggle,
};
use std::cell::RefCell;

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

                stack.add_section(Some("Geometry & Tab-Chain (↑/↓ Numeric)"), |sec_ui| {
                    MacForm::new("nib_geometry_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Width:").size(11.0), |ui| {
                                let prev_w = card.width.clone();
                                MacInputCell::with_id(input_id_w, &mut card.width)
                                    .theme(theme)
                                    .numeric(60.0..=500.0, 1.0)
                                    .width(68.0)
                                    .next_focus(input_id_h)
                                    .prev_focus(input_id_r)
                                    .placeholder("Width")
                                    .show(ui);
                                sync_aspect_width_changed(card, &prev_w);
                            });
                            form.row_label(egui::RichText::new("Height:").size(11.0), |ui| {
                                let prev_h = card.height.clone();
                                MacInputCell::with_id(input_id_h, &mut card.height)
                                    .theme(theme)
                                    .numeric(40.0..=300.0, 1.0)
                                    .width(68.0)
                                    .next_focus(input_id_r)
                                    .prev_focus(input_id_w)
                                    .placeholder("Height")
                                    .show(ui);
                                sync_aspect_height_changed(card, &prev_h);
                            });
                            form.row_label(egui::RichText::new("Corner Radius:").size(11.0), |ui| {
                                MacInputCell::with_id(input_id_r, &mut card.radius)
                                    .theme(theme)
                                    .numeric(0.0..=40.0, 1.0)
                                    .width(68.0)
                                    .next_focus(input_id_w)
                                    .prev_focus(input_id_h)
                                    .placeholder("Radius")
                                    .show(ui);
                            });
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
                            form.row_label(egui::RichText::new("Corner Radius:").size(11.0), |ui| {
                                let mut r = card.radius.parse::<f64>().unwrap_or(10.0);
                                let prev_r = r;
                                MacStepper::new(&mut r, 0.0..=40.0, 1.0)
                                    .theme(theme)
                                    .wraps(false)
                                    .show(ui);
                                if (r - prev_r).abs() > 0.01 {
                                    card.radius = format!("{:.0}", r);
                                }
                                let unit_str = match self.unit_segmented_idx {
                                    1 => "px",
                                    2 => "%",
                                    _ => "pt",
                                };
                                ui.label(
                                    egui::RichText::new(format!("{:.0} {}", r, unit_str))
                                        .size(10.5),
                                );
                            });
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
                                    self.unit_segmented_idx = idx;
                                });
                            });
                        });
                });

                stack.add_section(Some("State & Color (Toggle & ColorWell)"), |sec_ui| {
                    MacForm::new("nib_state_color_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Subpixel Snap:").size(11.0), |ui| {
                                ui.add(MacToggle::new(&mut self.snapping_enabled).theme(theme));
                            });
                            form.row_label(egui::RichText::new("Accent Swatch:").size(11.0), |ui| {
                                MacColorWell::new(&mut card.fill_color).theme(theme).show(ui);
                            });
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
