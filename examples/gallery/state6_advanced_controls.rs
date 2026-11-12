//! State 6 (`v0.6` – `v0.6.8`): Advanced Media, Scheduling, Grid & AppKit NIB Primitives (`SPEC-012`).
//!
//! Grounded in Apple AppKit/AVKit Sonoma 14.8.9 hardware inspection (Darwin 23.6.0).

use super::CenterState;
use egui::Ui;
use egui_macview::{
    controls::{
        MacBox, MacCheckbox, MacCollectionView, MacCollectionViewFlowLayout, MacComboBox,
        MacControlSize, MacDatePicker, MacDatePickerElementFlags, MacDatePickerStyle,
        MacDisclosureButton, MacDisclosureGroup, MacFontWeight, MacHelpButton,
        MacInlineActionButton, MacInputCell, MacLabel, MacLabelColor, MacPlayerControlsStyle,
        MacPlayerView, MacPopUpButton, MacRuleEditor, MacTabStyle, MacTabView,
    },
    theme::MacTheme,
    MacToastManager,
};
use std::cell::RefCell;

impl CenterState {
    /// Renders State 6 showcasing Milestone 0.6.0–0.6.8 advanced media, scheduling, grid, and NIB primitives.
    pub fn render_state6_advanced_controls(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        toasts: &RefCell<MacToastManager>,
    ) {
        content_ui.vertical(|ui| {
            MacBox::new("Advanced Media, Scheduling, Grid & AppKit NIB Primitives (SPEC-012 / v0.6.8)")
                .theme(theme)
                .show(ui, |box_ui| {
                    let combo_text = &mut self.advanced_combo_text;
                    let button_open = &mut self.advanced_button_open;

                    box_ui.vertical(|ui| {
                        // 1. v0.6.8 Cohesive AppKit NIB Inspector Matrix & Disabled Group Inheritance
                        let disc_resp = MacDisclosureGroup::new(
                            "gallery_disc_group",
                            "AppKit NIB Inspector Matrix, Baseline & Surface Vibrancy (v0.6.8)",
                            &mut self.advanced_disclosure_open,
                        )
                        .theme(theme)
                        .with_trailing_add_button(true)
                        .show(ui, |disc_ui| {
                            MacCheckbox::new(
                                &mut self.v068_inspector_enabled,
                                "Enable Inspector Controls (ui.add_enabled_ui)",
                            )
                            .control_size(MacControlSize::Regular)
                            .theme(theme)
                            .show(disc_ui);
                            disc_ui.add_space(4.0);

                            disc_ui.add_enabled_ui(self.v068_inspector_enabled, |en_ui| {
                                // Row A — Baseline & Corner-Radius Parity (2.0pt Square vs 5.0pt PopUp on #E5E5E5 MacBox)
                                en_ui.horizontal(|h_ui| {
                                    MacLabel::new("Width:")
                                        .weight(MacFontWeight::Semibold)
                                        .color(MacLabelColor::Primary)
                                        .theme(theme)
                                        .show(h_ui);
                                    MacInputCell::new("gallery_nib_width", &mut self.v068_nib_width)
                                        .width(64.0)
                                        .theme(theme)
                                        .show(h_ui);
                                    MacPopUpButton::new(
                                        &mut self.v068_nib_unit_idx,
                                        &["pt", "px", "em", "%"],
                                    )
                                    .width(64.0)
                                    .theme(theme)
                                    .show(h_ui);
                                    MacPopUpButton::new(
                                        &mut self.v068_nib_preset_idx,
                                        &["Sonoma 14.8", "Sequoia 15.0", "Big Sur 11.0"],
                                    )
                                    .width(128.0)
                                    .accent_badge(true)
                                    .theme(theme)
                                    .show(h_ui);
                                    if MacHelpButton::new().theme(theme).show(h_ui).clicked() {
                                        toasts
                                            .borrow_mut()
                                            .info("AppKit NSBezelStyleHelpButton (20x20pt)");
                                    }
                                    if MacInlineActionButton::new("+").theme(theme).show(h_ui).clicked() {
                                        self.v065_axis_count = self.v065_axis_count.saturating_add(1);
                                        toasts.borrow_mut().info("Added font axis");
                                    }
                                });

                                en_ui.add_space(4.0);

                                // Row B — Control Size Variants (Regular 22pt, Small 19pt, Mini 16pt) & Checkboxes
                                en_ui.horizontal(|h_ui| {
                                    MacCheckbox::new(&mut self.v065_check_regular, "Regular (14pt)")
                                        .control_size(MacControlSize::Regular)
                                        .theme(theme)
                                        .show(h_ui);
                                    MacCheckbox::new(&mut self.v065_check_small, "Small (12pt)")
                                        .control_size(MacControlSize::Small)
                                        .theme(theme)
                                        .show(h_ui);
                                    let is_mixed = !self.v065_check_mini_mixed;
                                    MacCheckbox::new(
                                        &mut self.v065_check_mini_mixed,
                                        "Mini Mixed (10pt)",
                                    )
                                    .control_size(MacControlSize::Mini)
                                    .mixed(is_mixed)
                                    .theme(theme)
                                    .show(h_ui);
                                    MacInputCell::new(
                                        "gallery_nib_small_in",
                                        &mut self.v068_nib_small_text,
                                    )
                                    .control_size(MacControlSize::Small)
                                    .width(78.0)
                                    .theme(theme)
                                    .show(h_ui);
                                    let items = [
                                        "Apple ProRes 422 HQ",
                                        "Apple ProRes 4444 XQ",
                                        "H.265 / HEVC 10-bit",
                                        "AV1 Main Profile",
                                        "Uncompressed 12-bit RAW",
                                    ];
                                    MacComboBox::new("gallery_codec_combo", combo_text, &items)
                                        .theme(theme)
                                        .control_size(MacControlSize::Regular)
                                        .width(152.0)
                                        .show(h_ui);
                                    h_ui.add(
                                        MacDisclosureButton::new(button_open)
                                            .control_size(MacControlSize::Regular)
                                            .theme(theme),
                                    );
                                });

                                en_ui.add_space(4.0);

                                // Row C — Surface Vibrancy (Primary / Secondary / Tertiary MacLabel on MacBox)
                                en_ui.horizontal(|h_ui| {
                                    MacLabel::new("Vibrancy:")
                                        .weight(MacFontWeight::Semibold)
                                        .color(MacLabelColor::Primary)
                                        .theme(theme)
                                        .show(h_ui);
                                    MacLabel::new("Primary Label")
                                        .color(MacLabelColor::Primary)
                                        .theme(theme)
                                        .show(h_ui);
                                    MacLabel::new(format!("Secondary Axes ({})", self.v065_axis_count))
                                        .control_size(MacControlSize::Small)
                                        .color(MacLabelColor::Secondary)
                                        .theme(theme)
                                        .show(h_ui);
                                    MacLabel::new("Tertiary Caption (WCAG Clamped)")
                                        .control_size(MacControlSize::Mini)
                                        .color(MacLabelColor::Tertiary)
                                        .theme(theme)
                                        .show(h_ui);
                                });
                            });
                        });
                        if disc_resp.trailing_clicked {
                            self.v065_axis_count = self.v065_axis_count.saturating_add(1);
                            toasts.borrow_mut().success("Disclosure header trailing '+' clicked");
                        }

                        MacBox::separator(ui);

                        // 2. Date Picker
                        ui.horizontal(|h_ui| {
                            MacLabel::new("Schedule Date & Time (NSDatePicker):")
                                .weight(MacFontWeight::Semibold)
                                .theme(theme)
                                .show(h_ui);
                            h_ui.add(
                                MacDatePicker::new()
                                    .date(&mut self.advanced_date)
                                    .style(MacDatePickerStyle::TextFieldAndStepper)
                                    .elements(MacDatePickerElementFlags::DateAndTime)
                                    .control_size(MacControlSize::Regular)
                                    .theme(theme),
                            );
                        });

                        MacBox::separator(ui);

                        // 3. Tab View hosting Media Player & Collection View
                        MacTabView::new(&mut self.advanced_tab_idx)
                            .style(MacTabStyle::TopTabsBezelBorder)
                            .control_size(MacControlSize::Regular)
                            .theme(theme)
                            .tab("AVPlayerView Floating HUD", |tab_ui| {
                                tab_ui.vertical(|ui| {
                                    MacLabel::new("QuickTime Floating HUD Pod & Inline Transport:")
                                        .color(MacLabelColor::Secondary)
                                        .theme(theme)
                                        .show(ui);
                                    ui.add(
                                        MacPlayerView::new()
                                            .controls_style(MacPlayerControlsStyle::Floating)
                                            .is_playing_mut(&mut self.advanced_player_playing)
                                            .progress_mut(&mut self.advanced_player_progress)
                                            .theme(theme),
                                    );
                                });
                            })
                            .tab("NSCollectionView Flow Layout Grid", |tab_ui| {
                                tab_ui.vertical(|ui| {
                                    MacLabel::new("2D Viewport-Culled Grid with Rubber-Band Marquee Selection:")
                                        .color(MacLabelColor::Secondary)
                                        .theme(theme)
                                        .show(ui);
                                    const GRID_ITEMS: [usize; 60] = {
                                        let mut arr = [0; 60];
                                        let mut i = 0;
                                        while i < 60 {
                                            arr[i] = i;
                                            i += 1;
                                        }
                                        arr
                                    };
                                    MacCollectionView::new(&GRID_ITEMS, &mut self.advanced_grid_selection)
                                        .layout(MacCollectionViewFlowLayout::default())
                                        .theme(theme)
                                        .show_with_item_tracker(ui, |item_ui, item| {
                                            item_ui.centered_and_justified(|c_ui| {
                                                MacLabel::new(format!("#{item}"))
                                                    .control_size(MacControlSize::Small)
                                                    .theme(theme)
                                                    .show(c_ui);
                                            });
                                        });
                                });
                            })
                            .tab("NSDatePicker Clock & Calendar", |tab_ui| {
                                tab_ui.vertical(|ui| {
                                    MacLabel::new("Visual Month Calendar Matrix & Analog Clock Face:")
                                        .color(MacLabelColor::Secondary)
                                        .theme(theme)
                                        .show(ui);
                                    ui.add(
                                        MacDatePicker::new()
                                            .date(&mut self.advanced_date)
                                            .style(MacDatePickerStyle::ClockAndCalendar)
                                            .elements(MacDatePickerElementFlags::DateAndTime)
                                            .theme(theme),
                                    );
                                });
                            })
                            .show(ui);

                        MacBox::separator(ui);

                        // 4. Rule Editor (NSRuleEditor)
                        ui.vertical(|r_ui| {
                            MacLabel::new("Filter Criteria Predicate Editor (NSRuleEditor):")
                                .weight(MacFontWeight::Semibold)
                                .theme(theme)
                                .show(r_ui);
                            const RULE_CRITERIA: &[&[&str]] = &[
                                &["Name", "Kind", "Created Date", "File Size"],
                                &["contains", "is exactly", "begins with", "ends with"],
                            ];
                            MacRuleEditor::new("gallery_rule_editor", &mut self.advanced_rules, RULE_CRITERIA)
                                .theme(theme)
                                .show(r_ui);
                        });

                        MacBox::separator(ui);
                        super::state6_pro_controls::render_state6_pro_controls(
                            ui,
                            theme,
                            &mut self.v069_origin_anchor,
                            &mut self.v069_tag_color,
                            &mut self.v069_action_items,
                            toasts,
                        );
                    });
                });
        });
    }
}
