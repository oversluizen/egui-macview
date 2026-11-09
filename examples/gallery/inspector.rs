//! 3-Tier Right Inspector (`DefaultNib`, `CompactHud`, `ExpandedPro`) with `.numeric(...)` stepping.

use crate::{CardModel, ExpandedProTab, GalleryScreen, InspectorMode};
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

pub use super::inspector_nib::{
    convert_card_units, render_card_geometry_rows, unit_scale_and_suffix,
};

use super::snippet::SnippetContext;

impl InspectorState {
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        card_cell: &RefCell<CardModel>,
        mode: &mut InspectorMode,
        active_screen: GalleryScreen,
        snippet_ctx: SnippetContext,
        toasts: &RefCell<MacToastManager>,
    ) {
        let mut card = card_cell.borrow_mut();
        let avail_rect = ui.available_rect_before_wrap();
        ui.painter_at(avail_rect).rect_filled(
            avail_rect,
            CornerRadius::ZERO,
            theme.window_background(),
        );

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
                        InspectorMode::ExpandedPro => self.render_expanded_pro(
                            s_ui,
                            theme,
                            &mut card,
                            active_screen,
                            snippet_ctx,
                            toasts,
                        ),
                        _ => self.render_default_nib(s_ui, theme, &mut card, mode, toasts),
                    }
                });
            });
    }

    fn render_expanded_pro(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        card: &mut CardModel,
        active_screen: GalleryScreen,
        snippet_ctx: SnippetContext,
        toasts: &RefCell<MacToastManager>,
    ) {
        ui.add_space(8.0);
        let tab_idx = self.expanded_tab as usize;
        const TAB_ITEMS: [&str; 4] = [
            "\u{2630} Attributes",
            "\u{2699} Metrics",
            "\u{21BB} Matrix",
            "</> Snippet",
        ];

        ui.vertical_centered(|c_ui| {
            MacSegmentedControl::new(&TAB_ITEMS, tab_idx)
                .theme(theme)
                .height(24.0)
                .show(c_ui, |idx| {
                    self.expanded_tab = match idx {
                        0 => ExpandedProTab::Attributes,
                        1 => ExpandedProTab::Metrics,
                        2 => ExpandedProTab::Matrix,
                        _ => ExpandedProTab::CodeSnippet,
                    };
                });
        });
        ui.add_space(6.0);

        if self.expanded_tab == ExpandedProTab::CodeSnippet {
            super::snippet::render_code_snippet_tab(
                ui,
                theme,
                active_screen,
                card,
                snippet_ctx,
                toasts,
            );
            return;
        }

        MacStackView::new(MacStackOrientation::Vertical)
            .theme(theme)
            .spacing(10.0)
            .padding(12.0)
            .show_separators(true)
            .show(ui, |stack| match self.expanded_tab {
                ExpandedProTab::Attributes => {
                    stack.add_section(Some("Layer Identity"), |sec_ui| {
                        MacForm::new("exp_layer_identity_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(egui::RichText::new("Name:").size(11.0), |ui| {
                                    MacInputCell::with_id(
                                        Id::NULL.with("exp_layer_name"),
                                        &mut card.layer_name,
                                    )
                                    .theme(theme)
                                    .width(180.0)
                                    .show(ui);
                                });
                            });
                    });

                    stack.add_section(Some("Appearance & Blending"), |sec_ui| {
                        let blend_modes = ["Normal", "Multiply", "Screen", "Overlay"];
                        MacForm::new("exp_appearance_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(egui::RichText::new("Opacity:").size(11.0), |ui| {
                                    MacSlider::new(&mut card.opacity, 0.0..=100.0)
                                        .theme(theme)
                                        .ticks(5, true)
                                        .width(120.0)
                                        .show(ui);
                                    ui.label(
                                        egui::RichText::new(format!("{:.0}%", card.opacity))
                                            .size(10.5),
                                    );
                                });
                                form.row_label(egui::RichText::new("Blend:").size(11.0), |ui| {
                                    MacPopUpButton::new(&mut card.blend_mode_idx, &blend_modes)
                                        .theme(theme)
                                        .width(140.0)
                                        .show(ui);
                                });
                            });
                    });

                    stack.add_section(Some("Color Styles"), |sec_ui| {
                        MacForm::new("exp_color_styles_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(
                                    egui::RichText::new("Fill Color:").size(11.0),
                                    |ui| {
                                        MacColorWell::new(&mut card.fill_color)
                                            .theme(theme)
                                            .show(ui);
                                    },
                                );
                                form.row_label(
                                    egui::RichText::new("Alpha Well:").size(11.0),
                                    |ui| {
                                        MacColorWell::new(&mut card.alpha_color)
                                            .theme(theme)
                                            .show_alpha(true)
                                            .show(ui);
                                    },
                                );
                            });
                    });

                    stack.add_section(Some("Shadow & Effects"), |sec_ui| {
                        MacForm::new("exp_shadow_effects_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(
                                    egui::RichText::new("Drop Shadow:").size(11.0),
                                    |ui| {
                                        ui.add(
                                            MacToggle::new(&mut card.shadow_enabled).theme(theme),
                                        );
                                    },
                                );
                                if card.shadow_enabled {
                                    form.row_label(
                                        egui::RichText::new("Blur Radius:").size(11.0),
                                        |ui| {
                                            MacStepper::new(&mut card.shadow_blur, 0.0..=24.0, 1.0)
                                                .theme(theme)
                                                .show(ui);
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{:.0} pt",
                                                    card.shadow_blur
                                                ))
                                                .size(10.5),
                                            );
                                        },
                                    );
                                }
                            });
                    });
                }
                ExpandedProTab::Metrics => {
                    let ids = (
                        Id::NULL.with("exp_input_w"),
                        Id::NULL.with("exp_input_h"),
                        Id::NULL.with("exp_input_r"),
                    );

                    stack.add_section(Some("Dimensions & Corner Radius (↑/↓ Step)"), |sec_ui| {
                        MacForm::new("exp_dimensions_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(egui::RichText::new("Units:").size(11.0), |ui| {
                                    MacSegmentedControl::new(
                                        &["Points", "Pixels", "Percent"],
                                        self.unit_segmented_idx,
                                    )
                                    .theme(theme)
                                    .show(ui, |idx| {
                                        let prev = self.unit_segmented_idx;
                                        self.unit_segmented_idx = idx;
                                        convert_card_units(card, prev, idx);
                                    });
                                });
                                render_card_geometry_rows(
                                    form,
                                    theme,
                                    card,
                                    self.unit_segmented_idx,
                                    ids,
                                );
                                form.row_label(
                                    egui::RichText::new("Constrain Aspect:").size(11.0),
                                    |ui| {
                                        ui.add(
                                            MacToggle::new(&mut card.constrain_proportions)
                                                .theme(theme),
                                        );
                                    },
                                );
                            });
                    });

                    stack.add_section(Some("Layout & Alignment Mask"), |sec_ui| {
                        MacForm::new("exp_align_mask_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(egui::RichText::new("Align:").size(11.0), |ui| {
                                    MacSegmentedControl::new(
                                        &["Left", "Center", "Right", "Fill"],
                                        card.alignment_idx,
                                    )
                                    .theme(theme)
                                    .show(ui, |idx| {
                                        card.alignment_idx = idx;
                                    });
                                });
                            });
                    });
                }
                ExpandedProTab::Matrix => {
                    stack.add_section(Some("2x2 Affine Transform Matrix [a, b; c, d]"), |sec_ui| {
                        let (a, b, c, d) =
                            card.affine_matrix(card.rotation_deg, card.scale_pct / 100.0);

                        let matrix_box = sec_ui.available_rect_before_wrap();
                        let m_rect =
                            Rect::from_min_size(matrix_box.min, vec2(matrix_box.width(), 48.0));
                        let mp = sec_ui.painter_at(m_rect);
                        mp.rect_filled(m_rect, CornerRadius::same(4), theme.control());
                        mp.rect_stroke(
                            m_rect,
                            CornerRadius::same(4),
                            Stroke::new(1.0, theme.separator()),
                            StrokeKind::Inside,
                        );
                        let font = FontId::monospace(11.0);
                        mp.text(
                            pos2(m_rect.center().x, m_rect.min.y + 14.0),
                            Align2::CENTER_CENTER,
                            format!("│ {:>+6.3}   {:>+6.3} │", a, b),
                            font.clone(),
                            theme.label(),
                        );
                        mp.text(
                            pos2(m_rect.center().x, m_rect.min.y + 34.0),
                            Align2::CENTER_CENTER,
                            format!("│ {:>+6.3}   {:>+6.3} │", c, d),
                            font,
                            theme.label(),
                        );
                        sec_ui.allocate_exact_size(vec2(m_rect.width(), 50.0), Sense::hover());
                    });

                    stack.add_section(Some("Continuous Transformation Controls"), |sec_ui| {
                        let mut rot_f64 = card.rotation_deg as f64;
                        MacForm::new("exp_matrix_controls_form").theme(theme).show(
                            sec_ui,
                            |form| {
                                form.row_label(egui::RichText::new("Rotation:").size(11.0), |ui| {
                                    MacSlider::new(&mut card.rotation_deg, 0.0..=360.0)
                                        .theme(theme)
                                        .ticks(8, true)
                                        .width(120.0)
                                        .show(ui);
                                    ui.label(
                                        egui::RichText::new(format!("{:.0}°", card.rotation_deg))
                                            .size(10.5),
                                    );
                                });
                                form.row_label(
                                    egui::RichText::new("Fine Step:").size(11.0),
                                    |ui| {
                                        if ui
                                            .add(
                                                MacStepper::new(&mut rot_f64, 0.0..=360.0, 5.0)
                                                    .theme(theme)
                                                    .wraps(true),
                                            )
                                            .changed()
                                        {
                                            card.rotation_deg = rot_f64 as f32;
                                        }
                                        ui.label(egui::RichText::new("±5.0°").size(10.5));
                                    },
                                );
                                form.row_label(egui::RichText::new("Scale:").size(11.0), |ui| {
                                    MacSlider::new(&mut card.scale_pct, 50.0..=200.0)
                                        .theme(theme)
                                        .ticks(6, true)
                                        .width(120.0)
                                        .show(ui);
                                    ui.label(
                                        egui::RichText::new(format!("{:.0}%", card.scale_pct))
                                            .size(10.5),
                                    );
                                });
                                form.row_label(
                                    egui::RichText::new("Skew / Shear:").size(11.0),
                                    |ui| {
                                        MacSlider::new(&mut card.skew_deg, -45.0..=45.0)
                                            .theme(theme)
                                            .width(120.0)
                                            .show(ui);
                                        ui.label(
                                            egui::RichText::new(format!("{:.0}°", card.skew_deg))
                                                .size(10.5),
                                        );
                                    },
                                );
                            },
                        );
                        sec_ui.add_space(6.0);
                        if sec_ui
                            .add(
                                MacButtonCell::new("\u{21BA} Reset to Identity Matrix")
                                    .theme(theme),
                            )
                            .clicked()
                        {
                            (card.rotation_deg, card.scale_pct, card.skew_deg) = (0.0, 100.0, 0.0);
                            toasts.borrow_mut().info("Transform reset");
                        }
                    });
                }
                ExpandedProTab::CodeSnippet => {}
            });
    }
}
