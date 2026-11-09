//! State 4 (`v0.5`): Finder & Publishing Data Browser (`GalleryScreen::FinderDataBrowser`).

use super::{
    audio_media::AudioTrack,
    finder_data::{build_parent_chain, collect_finder_rows, filter_and_sort_rows},
    CenterState, FinderCustomEntry,
};
use egui::{FontId, Ui};
use egui_macview::{
    theme::MacTheme, MacButtonCell, MacInputCell, MacPathComponent, MacPathControl,
    MacProgressControlSize, MacProgressIndicator, MacRatingIndicator, MacSlider, MacTableColumn,
    MacTableView, MacToastManager, MacToggle, MacTokenField,
};
use std::cell::RefCell;
use std::path::PathBuf;

impl CenterState {
    pub fn render_state4_finder_browser(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        toasts: &RefCell<MacToastManager>,
    ) {
        let label_font = FontId::proportional(12.0);
        let t = content_ui.input(|i| i.time);

        // Process OS Drag-and-Drop files
        let dropped_files = content_ui.input(|i| i.raw.dropped_files.clone());
        for df in dropped_files {
            let path: PathBuf = df.path().to_path_buf();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Dropped File".to_string());

            if path.is_dir() {
                self.finder_current_dir = Some(path.clone());
                toasts
                    .borrow_mut()
                    .success(format!("Opened local directory: {name}"));
            } else {
                let is_wav = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("wav"))
                    .unwrap_or(false);
                if is_wav {
                    match std::fs::read(&path) {
                        Ok(bytes) => match AudioTrack::from_wav_bytes(&name, &bytes) {
                            Ok(track) => {
                                self.media_waveform = track.waveform_peaks.clone();
                                self.media_out_point = track.duration_secs;
                                self.audio_track = track;
                                toasts
                                    .borrow_mut()
                                    .success(format!("Loaded WAV Audio into AVKit Studio: {name}"));
                            }
                            Err(err) => {
                                toasts
                                    .borrow_mut()
                                    .error(format!("Failed to parse WAV audio {name}: {err}"));
                            }
                        },
                        Err(err) => {
                            toasts
                                .borrow_mut()
                                .error(format!("Failed to read file {name}: {err}"));
                        }
                    }
                } else {
                    toasts.borrow_mut().info(format!("Loaded: {name}"));
                }
                let size_mb =
                    (std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) / 1048576) as u32;
                self.finder_custom_entries.push(FinderCustomEntry {
                    name: name.clone(),
                    path: path.clone(),
                    is_dir: false,
                    size_mb: size_mb.max(1),
                    codec: path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("File")
                        .to_uppercase(),
                    duration: "--:--:--:--".to_string(),
                    rating: 4,
                    tags: vec!["Dropped".to_string(), "Local".to_string()],
                });
            }
        }

        let window_active =
            content_ui.input(|i| i.focused && !i.viewport().minimized.unwrap_or(false));
        if self.is_publishing && window_active {
            content_ui
                .ctx()
                .request_repaint_after(std::time::Duration::from_millis(50));
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
                "Real local file/folder browsing, OS Drag & Drop (.wav audio / media), web links, and interactive star ratings.",
            )
            .size(11.0),
        );
        content_ui.add_space(8.0);

        // Web Link / URL Loader Bar
        content_ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Open Web / Docs:").size(11.0).strong());
            MacInputCell::with_id(
                egui::Id::NULL.with("finder_web_url"),
                &mut self.finder_url_input,
            )
            .theme(theme)
            .width(260.0)
            .show(ui);
            if ui
                .add(MacButtonCell::new("↗ Open URL").theme(theme))
                .clicked()
            {
                ui.ctx().open_url(egui::OpenUrl {
                    url: self.finder_url_input.clone(),
                    new_tab: true,
                });
                toasts
                    .borrow_mut()
                    .info(format!("Opened URL: {}", self.finder_url_input));
            }
            if self.finder_current_dir.is_some() {
                ui.add_space(8.0);
                if ui
                    .add(MacButtonCell::new("Reset to Studio Demo Assets").theme(theme))
                    .clicked()
                {
                    self.finder_current_dir = None;
                    self.active_path_idx = 3;
                    toasts.borrow_mut().info("Reset to built-in studio assets");
                }
            }
        });
        content_ui.add_space(6.0);

        // 1. MacPathControl (NSPathControl) with dynamic real directory or built-in assets
        let (dir_parents, path_titles) = build_parent_chain(&self.finder_current_dir);
        let path_items: Vec<MacPathComponent> = path_titles
            .iter()
            .map(|s| MacPathComponent::folder(s.as_str()))
            .collect();

        if !path_items.is_empty() {
            content_ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Path:").size(11.0).strong());
                let avail_w = (ui.available_width() - 24.0).max(260.0);
                let active_idx = self.active_path_idx.min(path_items.len() - 1);
                let out = MacPathControl::new("studio_path_v05", &path_items[..=active_idx])
                    .theme(theme)
                    .show_choose_button(true)
                    .width(avail_w)
                    .show(ui);

                if let Some(clicked) = out.clicked_segment {
                    if let Some(ref parents) = dir_parents {
                        if let Some(target) = parents.get(clicked) {
                            self.finder_current_dir = Some(target.clone());
                            self.active_path_idx = clicked;
                        }
                    } else {
                        self.active_path_idx = clicked;
                    }
                    if let Some(comp) = path_items.get(clicked) {
                        toasts
                            .borrow_mut()
                            .info(format!("Navigated to {}", comp.title));
                    }
                }
                if out.choose_clicked {
                    let cur = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    self.finder_current_dir = Some(cur);
                    self.active_path_idx = 100;
                    toasts.borrow_mut().info("Loaded current working directory");
                }
            });
            content_ui.add_space(8.0);
        }

        // Gather table rows from either real filesystem or built-in studio assets
        let all_rows = collect_finder_rows(
            &self.finder_current_dir,
            &self.finder_custom_entries,
            &self.asset_ratings,
            &self.asset_tokens,
        );
        let visible_rows = filter_and_sort_rows(
            all_rows,
            &self.search_query,
            self.table_sort_col,
            self.table_sort_dir,
        );

        // 2. MacTableView (NSTableView + NSTableHeaderView)
        let col_sort = |idx: usize| {
            if self.table_sort_col == idx {
                Some(self.table_sort_dir)
            } else {
                None
            }
        };
        let cols = [
            MacTableColumn::new("Asset Name", 180.0).sort(col_sort(0)),
            MacTableColumn::new("Codec / Format", 130.0).sort(col_sort(1)),
            MacTableColumn::new("Duration", 90.0).sort(col_sort(2)),
            MacTableColumn::new("Size (MB)", 80.0).sort(col_sort(3)),
            MacTableColumn::new("Rating", 90.0).sort(col_sort(4)),
            MacTableColumn::new("Tags", 130.0),
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
            let item = &visible_rows[row_idx];
            match col_idx {
                0 => {
                    let prefix = if item.is_dir { "📁 " } else { "📄 " };
                    row_ui.label(format!("{prefix}{}", item.name));
                }
                1 => {
                    row_ui.label(&item.codec);
                }
                2 => {
                    row_ui.label(&item.duration);
                }
                3 => {
                    row_ui.label(format!("{} MB", item.size_mb));
                }
                4 => {
                    let mut stars = item.rating;
                    MacRatingIndicator::new(&mut stars, 5)
                        .theme(theme)
                        .item_size(12.0)
                        .enabled(false)
                        .show(row_ui);
                }
                _ => {
                    let tag_summary = item.tags.join(", ");
                    row_ui.label(tag_summary);
                }
            }
        });
        if let Some((col_idx, dir)) = tbl_out.toggled_sort {
            self.table_sort_col = col_idx;
            self.table_sort_dir = dir;
        }

        // If a directory row was selected/opened, allow navigating into it
        let active_row_idx = self.selected_table_rows.iter().next().copied();
        if let Some(r_idx) = active_row_idx {
            if let Some(selected_item) = visible_rows.get(r_idx) {
                if let Some(ref target_dir) = selected_item.dir_path {
                    if selected_item.is_dir {
                        content_ui.add_space(4.0);
                        if content_ui
                            .add(MacButtonCell::new("📂 Enter Selected Folder").theme(theme))
                            .clicked()
                        {
                            self.finder_current_dir = Some(target_dir.clone());
                            toasts
                                .borrow_mut()
                                .info(format!("Entered folder: {}", selected_item.name));
                        }
                    }
                }
            }
        }

        // 3. Metadata Rating & MacTokenField bound to active asset row
        content_ui.add_space(8.0);
        let active_asset_idx = active_row_idx.unwrap_or(0).min(5);
        content_ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Asset Rating:")
                    .font(label_font.clone())
                    .strong(),
            );
            MacRatingIndicator::new(&mut self.asset_ratings[active_asset_idx], 5)
                .theme(theme)
                .item_size(16.0)
                .show(ui);
        });

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

        // 4. MacProgressIndicator Publish Queue
        content_ui.add_space(10.0);
        content_ui.horizontal(|ui| {
            let btn_label = if self.is_publishing {
                "Pause Publish"
            } else {
                "Publish Selected Asset"
            };
            if ui
                .add(
                    MacButtonCell::new(btn_label)
                        .theme(theme)
                        .selected(self.is_publishing),
                )
                .clicked()
            {
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
                .width(180.0);
            if self.publish_indeterminate {
                bar = bar.indeterminate((t * 0.65).fract());
            }
            bar.show(ui);

            let spin_t = if (self.is_publishing || self.publish_indeterminate) && window_active {
                t.fract().max(1e-4)
            } else {
                0.0
            };
            MacProgressIndicator::spinning(spin_t)
                .theme(theme)
                .control_size(MacProgressControlSize::Regular)
                .show(ui);
        });

        content_ui.add_space(6.0);
        content_ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Manual Scrub:").size(11.0));
            MacSlider::new(&mut self.publish_progress, 0.0..=1.0)
                .theme(theme)
                .width(140.0)
                .show(ui);
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Indeterminate:").size(11.0));
            ui.add(MacToggle::new(&mut self.publish_indeterminate).theme(theme));
        });
        content_ui.add_space(14.0);
    }
}
