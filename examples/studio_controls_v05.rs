//! `studio_controls_v05` — Interactive Showcase for macOS Sonoma 14 / Sequoia 15 Studio & AVKit Controls.
//!
//! Demonstrates all 6 domain-agnostic `v0.5.0` controls (`SPEC-011`):
//! 1. `MacProgressIndicator` (`NSProgressIndicator` determinate, indeterminate, and 12-fin spinning styles)
//! 2. `MacTableView` (`NSTableView` + `NSTableHeaderView` with sortable columns, alternating rows, and selection)
//! 3. `MacTokenField` (`NSTokenField` with rounded token capsules, selection, and backspace/delete removal)
//! 4. `MacLevelIndicator` (`NSLevelIndicator` continuous and discrete VU meters with warning/critical thresholds)
//! 5. `MacTrimScrubber` (`AVPlayerView` inline trim bar with yellow frame, double-slit handles, and playhead)
//! 6. `MacPathControl` (`NSPathControl` breadcrumb bar with clickable segments and Choose... action)

#![cfg_attr(not(test), windows_subsystem = "windows")]

use egui_macview::{
    setup_symbols_font,
    theme::{MacAppearance, MacTheme},
    MacLevelIndicator, MacLevelIndicatorStyle, MacLevelOrientation, MacPathComponent,
    MacPathControl, MacProgressControlSize, MacProgressIndicator, MacSegmentedControl, MacSlider,
    MacSortDirection, MacTableColumn, MacTableView, MacTimelineZone, MacTokenField,
    MacTrimScrubber,
};
use std::collections::BTreeSet;

struct StudioApp {
    appearance: MacAppearance,
    progress: f32,
    indeterminate: bool,
    sort_dir: Option<MacSortDirection>,
    selected_rows: BTreeSet<usize>,
    tokens: Vec<String>,
    token_draft: String,
    audio_level_l: f32,
    audio_level_r: f32,
    segmented_vu: bool,
    in_point: f64,
    out_point: f64,
    playhead: f64,
    waveform: Vec<f32>,
    active_path_idx: usize,
}

impl Default for StudioApp {
    fn default() -> Self {
        let waveform = (0..64)
            .map(|i| {
                let phase = i as f32 * 0.22;
                (phase.sin().abs() * 0.65 + (phase * 2.3).cos().abs() * 0.35).clamp(0.08, 1.0)
            })
            .collect();
        Self {
            appearance: MacAppearance::DarkAqua,
            progress: 0.68,
            indeterminate: false,
            sort_dir: Some(MacSortDirection::Ascending),
            selected_rows: BTreeSet::from([0]),
            tokens: vec![
                "ProRes 422 HQ".to_string(),
                "Rec.709".to_string(),
                "24 fps".to_string(),
            ],
            token_draft: String::new(),
            audio_level_l: 7.4,
            audio_level_r: 8.8,
            segmented_vu: true,
            in_point: 14.5,
            out_point: 82.0,
            playhead: 41.2,
            waveform,
            active_path_idx: 3,
        }
    }
}

impl eframe::App for StudioApp {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {}

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let theme = MacTheme::new(self.appearance);
        theme.apply_to_egui(ui.ctx());
        let t = ui.input(|i| i.time);
        let spin_t = if self.indeterminate {
            t.fract().max(1e-4)
        } else {
            0.0
        };

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("egui-macview v0.5.0 — AppKit & AVKit Studio Controls");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mode_idx = usize::from(self.appearance == MacAppearance::DarkAqua);
                    MacSegmentedControl::new(&["Aqua", "DarkAqua"], mode_idx)
                        .theme(&theme)
                        .show(ui, |idx| {
                            self.appearance = if idx == 0 {
                                MacAppearance::Aqua
                            } else {
                                MacAppearance::DarkAqua
                            };
                        });
                });
            });
            ui.separator();

            // 1. NSPathControl
            let path_items = [
                MacPathComponent::folder("Macintosh HD"),
                MacPathComponent::folder("Projects"),
                MacPathComponent::folder("Feature_Film_2026"),
                MacPathComponent::file("Master_Timeline.mov"),
            ];
            ui.horizontal(|ui| {
                ui.label("NSPathControl:");
                let out =
                    MacPathControl::new("studio_path_std", &path_items[..=self.active_path_idx])
                        .theme(&theme)
                        .show_choose_button(true)
                        .width(480.0)
                        .show(ui);
                if let Some(clicked) = out.clicked_segment {
                    self.active_path_idx = clicked;
                }
                if out.choose_clicked {
                    self.active_path_idx = path_items.len() - 1;
                }
            });
            ui.add_space(6.0);

            // 2. NSProgressIndicator
            ui.horizontal(|ui| {
                ui.label("NSProgressIndicator:");
                let mut bar = MacProgressIndicator::bar(self.progress)
                    .theme(&theme)
                    .control_size(MacProgressControlSize::Regular)
                    .show_percentage(true)
                    .width(200.0);
                if self.indeterminate {
                    bar = bar.indeterminate((t * 0.6).fract());
                }
                bar.show(ui);
                MacProgressIndicator::spinning(spin_t)
                    .theme(&theme)
                    .control_size(MacProgressControlSize::Regular)
                    .show(ui);
                MacProgressIndicator::spinning(spin_t)
                    .theme(&theme)
                    .control_size(MacProgressControlSize::Small)
                    .show(ui);
                MacProgressIndicator::spinning(spin_t)
                    .theme(&theme)
                    .control_size(MacProgressControlSize::Mini)
                    .show(ui);
                MacSlider::new(&mut self.progress, 0.0..=1.0)
                    .theme(&theme)
                    .show(ui);
                ui.checkbox(&mut self.indeterminate, "Indeterminate");
            });
            ui.add_space(6.0);

            // 3. NSTokenField
            ui.horizontal(|ui| {
                ui.label("NSTokenField:");
                MacTokenField::new("studio_tokens", &mut self.tokens, &mut self.token_draft)
                    .theme(&theme)
                    .placeholder("Add tag (comma or Enter)...")
                    .width(460.0)
                    .show(ui);
            });
            ui.add_space(6.0);

            // 4. NSLevelIndicator
            ui.horizontal(|ui| {
                ui.label("NSLevelIndicator (L/R):");
                MacLevelIndicator::new(self.audio_level_l, 0.0..=10.0)
                    .theme(&theme)
                    .style(MacLevelIndicatorStyle::Discrete)
                    .orientation(MacLevelOrientation::Horizontal)
                    .warning_value(6.5)
                    .critical_value(8.5)
                    .segments(12)
                    .segmented_threshold_colors(self.segmented_vu)
                    .tick_marks(6)
                    .length(180.0)
                    .show(ui);
                MacLevelIndicator::new(self.audio_level_r, 0.0..=10.0)
                    .theme(&theme)
                    .style(MacLevelIndicatorStyle::Continuous)
                    .orientation(MacLevelOrientation::Horizontal)
                    .warning_value(6.5)
                    .critical_value(8.5)
                    .tick_marks(6)
                    .length(180.0)
                    .show(ui);
                ui.checkbox(&mut self.segmented_vu, "Threshold VU segments");
            });
            ui.add_space(6.0);

            // 5. NSTableView + NSTableHeaderView
            let cols = [
                MacTableColumn::new("Clip Name", 180.0).sort(self.sort_dir),
                MacTableColumn::new("Duration", 110.0),
                MacTableColumn::new("Codec", 140.0),
            ];
            let rows = [
                ["A001_C004_ProRes.mov", "00:01:24:12", "Apple ProRes 422 HQ"],
                ["A001_C009_Broll.mov", "00:00:42:08", "Apple ProRes 4444"],
                ["Audio_Mix_Stems.wav", "00:03:18:00", "PCM 24-bit 48kHz"],
                ["Title_Card_Main.mov", "00:00:12:00", "Apple ProRes 4444"],
            ];
            let tbl_out = MacTableView::new(
                "studio_clips_table",
                &cols,
                rows.len(),
                &mut self.selected_rows,
            )
            .theme(&theme)
            .uses_alternating_row_backgrounds(true)
            .show(ui, |row_ui, row_idx, col_idx| {
                row_ui.label(rows[row_idx][col_idx]);
            });
            if let Some((_, dir)) = tbl_out.toggled_sort {
                self.sort_dir = Some(dir);
            }
            ui.add_space(6.0);

            // 6. AVPlayerView MacTrimScrubber
            ui.label("AVPlayerView Trim Scrubber:");
            let zones = [MacTimelineZone::new(20.0, 45.0, "Scene 1")];
            MacTrimScrubber::new(
                "studio_trim_scrubber",
                120.0,
                &mut self.in_point,
                &mut self.out_point,
                &mut self.playhead,
            )
            .theme(&theme)
            .waveform(&self.waveform)
            .zones(&zones)
            .width(640.0)
            .show(ui);
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "egui-macview v0.5.0 Studio Controls",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            setup_symbols_font(&mut fonts);
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(StudioApp::default()))
        }),
    )
}
