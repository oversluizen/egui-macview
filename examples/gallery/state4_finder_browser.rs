//! State 4 (`v0.5`): Finder & Publishing Data Browser (`GalleryScreen::FinderDataBrowser`).

use super::CenterState;
use egui::{FontId, Ui};
use egui_macview::{
    theme::MacTheme, MacButtonCell, MacPathComponent, MacPathControl, MacProgressControlSize,
    MacProgressIndicator, MacSlider, MacSortDirection, MacTableColumn, MacTableView, MacToastManager,
    MacToggle, MacTokenField,
};
use std::cell::RefCell;

struct StudioAssetRow {
    idx: usize,
    name: &'static str,
    folder_depth: usize,
    codec: &'static str,
    duration: &'static str,
    size_mb: u32,
}

const STUDIO_ASSETS: [StudioAssetRow; 6] = [
    StudioAssetRow {
        idx: 0,
        name: "A001_C004_MainTitle.mov",
        folder_depth: 3,
        codec: "Apple ProRes 422 HQ",
        duration: "00:01:24:12",
        size_mb: 1420,
    },
    StudioAssetRow {
        idx: 1,
        name: "A001_C009_VFX_Plate.mov",
        folder_depth: 3,
        codec: "Apple ProRes 4444",
        duration: "00:00:42:08",
        size_mb: 2180,
    },
    StudioAssetRow {
        idx: 2,
        name: "Master_Mix_Stems_48k.wav",
        folder_depth: 2,
        codec: "PCM 24-bit 48kHz",
        duration: "00:03:18:00",
        size_mb: 340,
    },
    StudioAssetRow {
        idx: 3,
        name: "Dailies_Proxy_Reel01.mp4",
        folder_depth: 2,
        codec: "HEVC 10-bit HDR10",
        duration: "00:04:10:00",
        size_mb: 480,
    },
    StudioAssetRow {
        idx: 4,
        name: "Studio_ColorGrade_Rec709.cube",
        folder_depth: 1,
        codec: "3D LUT Metal Shader",
        duration: "00:00:01:00",
        size_mb: 12,
    },
    StudioAssetRow {
        idx: 5,
        name: "Deliverable_Theatrical_4K.mov",
        folder_depth: 3,
        codec: "Apple ProRes 422 HQ",
        duration: "00:12:45:00",
        size_mb: 8940,
    },
];

impl CenterState {
    pub fn render_state4_finder_browser(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        toasts: &RefCell<MacToastManager>,
    ) {
        let label_font = FontId::proportional(12.0);
        let t = content_ui.input(|i| i.time);

        if self.is_publishing {
            content_ui
                .ctx()
                .request_repaint_after(std::time::Duration::from_millis(33));
            self.publish_progress = (self.publish_progress + 0.015).min(1.0);
            if self.publish_progress >= 1.0 {
                self.is_publishing = false;
                toasts
                    .borrow_mut()
                    .success("Publish Queue Completed (100%)");
            }
        }

        content_ui.label(
            egui::RichText::new(
                "State 4 (v0.5) — Finder & Publishing Data Browser (NSPathControl, NSTableView, NSTokenField, NSProgressIndicator)",
            )
            .font(label_font.clone())
            .strong(),
        );
        content_ui.label(
            egui::RichText::new(
                "Click breadcrumb folders to filter assets, click table headers to sort, edit NSTokenField tags per row, and run the Publish queue.",
            )
            .size(11.0),
        );
        content_ui.add_space(8.0);

        // 1. MacPathControl (NSPathControl)
        let path_items = [
            MacPathComponent::folder("Macintosh HD"),
            MacPathComponent::folder("Projects"),
            MacPathComponent::folder("AppKit_Studio_v05"),
            MacPathComponent::folder("Master_Deliverables"),
        ];
        content_ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Path:").size(11.0).strong());
            let avail_w = (ui.available_width() - 24.0).max(260.0);
            let out = MacPathControl::new("studio_path_v05", &path_items[..=self.active_path_idx])
                .theme(theme)
                .show_choose_button(true)
                .width(avail_w)
                .show(ui);
            if let Some(clicked) = out.clicked_segment {
                self.active_path_idx = clicked;
                toasts.borrow_mut().info(format!(
                    "Navigated to {}",
                    path_items[clicked].title
                ));
            }
            if out.choose_clicked {
                self.active_path_idx = path_items.len() - 1;
                toasts
                    .borrow_mut()
                    .info("Restored full Master_Deliverables path");
            }
        });
        content_ui.add_space(8.0);

        // Filter & Sort rows
        let q = self.search_query.trim().to_lowercase();
        let max_depth = self.active_path_idx;
        let mut visible_rows: Vec<&StudioAssetRow> = STUDIO_ASSETS
            .iter()
            .filter(|r| {
                let depth_ok = match max_depth {
                    0 => true,
                    1 => r.folder_depth >= 1,
                    2 => r.folder_depth >= 2,
                    _ => r.folder_depth >= 3,
                };
                if !depth_ok {
                    return false;
                }
                if q.is_empty() {
                    return true;
                }
                let tags_match = self.asset_tokens[r.idx]
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&q));
                r.name.to_lowercase().contains(&q)
                    || r.codec.to_lowercase().contains(&q)
                    || tags_match
            })
            .collect();

        visible_rows.sort_by(|a, b| {
            let ord = match self.table_sort_col {
                0 => a.name.cmp(b.name),
                1 => a.codec.cmp(b.codec),
                2 => a.duration.cmp(b.duration),
                _ => a.size_mb.cmp(&b.size_mb),
            };
            match self.table_sort_dir {
                MacSortDirection::Ascending => ord,
                MacSortDirection::Descending => ord.reverse(),
            }
        });

        // 2. MacTableView (NSTableView + NSTableHeaderView)
        let col_sort = |idx: usize| {
            if self.table_sort_col == idx {
                Some(self.table_sort_dir)
            } else {
                None
            }
        };
        let cols = [
            MacTableColumn::new("Asset Name", 190.0).sort(col_sort(0)),
            MacTableColumn::new("Codec / Format", 150.0).sort(col_sort(1)),
            MacTableColumn::new("Duration", 95.0).sort(col_sort(2)),
            MacTableColumn::new("Size (MB)", 85.0).sort(col_sort(3)),
            MacTableColumn::new("Tags", 150.0),
        ];

        let tbl_out = MacTableView::new(
            "studio_finder_table_v05",
            &cols,
            visible_rows.len(),
            &mut self.selected_table_rows,
        )
        .theme(theme)
        .uses_alternating_row_backgrounds(true)
        .show(content_ui, |row_ui, row_idx, col_idx| {
            let item = visible_rows[row_idx];
            match col_idx {
                0 => {
                    row_ui.label(item.name);
                }
                1 => {
                    row_ui.label(item.codec);
                }
                2 => {
                    row_ui.label(item.duration);
                }
                3 => {
                    row_ui.label(format!("{} MB", item.size_mb));
                }
                _ => {
                    let tag_summary = self.asset_tokens[item.idx].join(", ");
                    row_ui.label(tag_summary);
                }
            }
        });
        if let Some((col_idx, dir)) = tbl_out.toggled_sort {
            self.table_sort_col = col_idx;
            self.table_sort_dir = dir;
        }

        // 3. MacTokenField (NSTokenField) bound to selected asset row
        content_ui.add_space(10.0);
        let active_asset_idx = self
            .selected_table_rows
            .iter()
            .next()
            .copied()
            .and_then(|vis_i| visible_rows.get(vis_i).map(|r| r.idx))
            .unwrap_or(0);

        content_ui.label(
            egui::RichText::new(format!(
                "NSTokenField — Metadata Tags for '{}'",
                STUDIO_ASSETS[active_asset_idx].name
            ))
            .font(label_font.clone())
            .strong(),
        );
        content_ui.add_space(4.0);
        let avail_token_w = (content_ui.available_width() - 32.0).max(280.0);
        MacTokenField::new(
            "studio_asset_tokens_v05",
            &mut self.asset_tokens[active_asset_idx],
            &mut self.token_draft,
        )
        .theme(theme)
        .placeholder("Type tag and press comma (,) or Enter...")
        .max_chars(120)
        .width(avail_token_w)
        .show(content_ui);

        // 4. MacProgressIndicator (NSProgressIndicator) Publish / Export Queue
        content_ui.add_space(12.0);
        content_ui.label(
            egui::RichText::new(
                "NSProgressIndicator — Determinate, Indeterminate & 12-Fin Spinning Styles",
            )
            .font(label_font)
            .strong(),
        );
        content_ui.add_space(6.0);

        content_ui.horizontal(|ui| {
            let btn_label = if self.is_publishing {
                "Pause Publish"
            } else {
                "Publish Selected Asset"
            };
            if ui.add(MacButtonCell::new(btn_label).theme(theme).selected(self.is_publishing)).clicked() {
                if !self.is_publishing && self.publish_progress >= 1.0 {
                    self.publish_progress = 0.0;
                }
                self.is_publishing = !self.is_publishing;
                toasts.borrow_mut().info(if self.is_publishing {
                    "Publishing started..."
                } else {
                    "Publishing paused"
                });
            }

            let mut bar = MacProgressIndicator::bar(self.publish_progress)
                .theme(theme)
                .control_size(MacProgressControlSize::Regular)
                .show_percentage(true)
                .width(190.0);
            if self.publish_indeterminate {
                bar = bar.indeterminate((t * 0.65).fract());
            }
            bar.show(ui);

            let spin_t = if self.is_publishing || self.publish_indeterminate {
                t.fract().max(1e-4)
            } else {
                0.0
            };
            MacProgressIndicator::spinning(spin_t)
                .theme(theme)
                .control_size(MacProgressControlSize::Regular)
                .show(ui);
            MacProgressIndicator::spinning(spin_t)
                .theme(theme)
                .control_size(MacProgressControlSize::Small)
                .show(ui);
            MacProgressIndicator::spinning(spin_t)
                .theme(theme)
                .control_size(MacProgressControlSize::Mini)
                .show(ui);
        });

        content_ui.add_space(6.0);
        content_ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Manual Scrub:").size(11.0));
            MacSlider::new(&mut self.publish_progress, 0.0..=1.0)
                .theme(theme)
                .width(150.0)
                .show(ui);
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Indeterminate:").size(11.0));
            ui.add(MacToggle::new(&mut self.publish_indeterminate).theme(theme));
        });
        content_ui.add_space(18.0);
    }
}
