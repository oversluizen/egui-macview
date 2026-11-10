//! State 6 (`v0.6`): Advanced Media, Scheduling, and Grid Controls (`SPEC-012`).
//!
//! Grounded in Apple AppKit/AVKit Sonoma 14.8.9 hardware inspection (Darwin 23.6.0).

use super::CenterState;
use egui::Ui;
use egui_macview::{
    controls::{
        MacBox, MacCollectionView, MacCollectionViewFlowLayout, MacComboBox, MacControlSize,
        MacDatePicker, MacDatePickerElementFlags, MacDatePickerStyle, MacDisclosureButton,
        MacDisclosureGroup, MacPlayerControlsStyle, MacPlayerView, MacRuleEditor, MacTabStyle,
        MacTabView,
    },
    theme::MacTheme,
    MacToastManager,
};
use std::cell::RefCell;

impl CenterState {
    /// Renders State 6 showcasing Milestone 0.6.0 advanced media, scheduling, and grid controls.
    pub fn render_state6_advanced_controls(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        _toasts: &RefCell<MacToastManager>,
    ) {
        content_ui.vertical(|ui| {
            MacBox::new("Advanced Media, Scheduling & Grid Controls Suite (SPEC-012)")
                .theme(theme)
                .show(ui, |box_ui| {
                    let combo_text = &mut self.advanced_combo_text;
                    let button_open = &mut self.advanced_button_open;

                    box_ui.vertical(|ui| {
                        // 1. Disclosure Group & Button
                        MacDisclosureGroup::new(
                            "gallery_disc_group",
                            "Encoding & Codec Settings (NSBezelStyleDisclosure)",
                            &mut self.advanced_disclosure_open,
                        )
                        .theme(theme)
                        .show(ui, |disc_ui| {
                            disc_ui.horizontal(|h_ui| {
                                h_ui.label("Pill Disclosure Button:");
                                h_ui.add(
                                    MacDisclosureButton::new(button_open)
                                        .control_size(MacControlSize::Regular)
                                        .theme(theme),
                                );

                                h_ui.add_space(16.0);
                                h_ui.label("Video Codec:");
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
                                    .show(h_ui);
                            });
                        });

                        MacBox::separator(ui);

                        // 2. Date Picker
                        ui.horizontal(|h_ui| {
                            h_ui.label("Schedule Date & Time (NSDatePicker):");
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
                                    ui.label("QuickTime Floating HUD Pod & Inline Transport:");
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
                                    ui.label("2D Viewport-Culled Grid with Rubber-Band Marquee Selection:");
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
                                                c_ui.label(format!("#{item}"));
                                            });
                                        });
                                });
                            })
                            .tab("NSDatePicker Clock & Calendar", |tab_ui| {
                                tab_ui.vertical(|ui| {
                                    ui.label("Visual Month Calendar Matrix & Analog Clock Face:");
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
                            r_ui.label("Filter Criteria Predicate Editor (NSRuleEditor):");
                            const RULE_CRITERIA: &[&[&str]] = &[
                                &["Name", "Kind", "Created Date", "File Size"],
                                &["contains", "is exactly", "begins with", "ends with"],
                            ];
                            MacRuleEditor::new("gallery_rule_editor", &mut self.advanced_rules, RULE_CRITERIA)
                                .theme(theme)
                                .show(r_ui);
                        });
                    });
                });
        });
    }
}
