//! State 4 (`v0.5`): Finder & Publishing Data Browser (`GalleryScreen::FinderDataBrowser`).

use super::{
    audio_media::AudioTrack,
    finder_data::{build_parent_chain, collect_finder_rows, filter_and_sort_rows},
    CenterState, FinderCustomEntry,
};
use egui::{FontId, Ui};
use egui_macview::{
    theme::MacTheme, MacButtonCell, MacInputCell, MacPathComponent, MacPathControl,
    MacRatingIndicator, MacTableColumn, MacTableView, MacToastManager, MacTokenField,
};
use std::cell::RefCell;
use std::path::PathBuf;

impl CenterState {
    /// Loads a local WAV audio file into AVKit Studio and parses its waveform peaks.
    pub fn load_wav_file(
        &mut self,
        path: &std::path::Path,
        name: &str,
        toasts: &RefCell<MacToastManager>,
    ) -> bool {
        match std::fs::read(path) {
            Ok(bytes) => match AudioTrack::from_wav_bytes(name, &bytes) {
                Ok(track) => {
                    self.media_waveform = track.waveform_peaks.clone();
                    self.media_out_point = track.duration_secs;
                    self.audio_track = track;
                    toasts
                        .borrow_mut()
                        .success(format!("Loaded WAV Audio into AVKit Studio: {name}"));
                    true
                }
                Err(err) => {
                    toasts
                        .borrow_mut()
                        .error(format!("Failed to parse WAV audio {name}: {err}"));
                    false
                }
            },
            Err(err) => {
                toasts
                    .borrow_mut()
                    .error(format!("Failed to read file {name}: {err}"));
                false
            }
        }
    }

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
                    self.load_wav_file(&path, &name, toasts);
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

        let window_active = egui_macview::is_window_active(content_ui.ctx());
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
                    self.selected_table_rows.clear();
                    self.selected_table_rows.insert(0);
                    toasts.borrow_mut().info("Reset to built-in studio assets");
                }
            } else {
                ui.add_space(8.0);
                if ui
                    .add(MacButtonCell::new("Switch to Local Disk Browser").theme(theme))
                    .clicked()
                {
                    self.finder_current_dir = std::env::current_dir().ok();
                    self.selected_table_rows.clear();
                    self.selected_table_rows.insert(0);
                    toasts.borrow_mut().info("Switched to local disk browser");
                }
            }
        });
        content_ui.add_space(6.0);

        // Gather table rows from filesystem/assets (memoized via finder_cached_rows to eliminate per-frame I/O)
        let key_matches = if let Some(ref k) = self.finder_cache_key {
            k.current_dir == self.finder_current_dir
                && k.custom_len == self.finder_custom_entries.len()
                && k.ratings == self.asset_ratings
                && k.tokens.as_slice() == self.asset_tokens.as_slice()
                && k.search_query == self.search_query
                && k.sort_col == self.table_sort_col
                && k.sort_dir == self.table_sort_dir
        } else {
            false
        };

        if !key_matches {
            let all_rows = collect_finder_rows(
                &self.finder_current_dir,
                &self.finder_custom_entries,
                &self.asset_ratings,
                &self.asset_tokens,
            );
            self.finder_cached_rows = filter_and_sort_rows(
                all_rows,
                &self.search_query,
                self.table_sort_col,
                self.table_sort_dir,
            );
            self.finder_cache_key = Some(super::center_state::FinderCacheKey {
                current_dir: self.finder_current_dir.clone(),
                custom_len: self.finder_custom_entries.len(),
                ratings: self.asset_ratings,
                tokens: self.asset_tokens.to_vec(),
                search_query: self.search_query.clone(),
                sort_col: self.table_sort_col,
                sort_dir: self.table_sort_dir,
            });
        }
        let visible_rows = &self.finder_cached_rows;

        // Keyboard ArrowUp / Backspace navigation to parent directory (Backspace only when not editing text)
        let wants_kb = content_ui.ctx().egui_wants_keyboard_input();
        let nav_up = content_ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::COMMAND, egui::Key::ArrowUp)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::ArrowUp)
                || (!wants_kb && i.consume_key(egui::Modifiers::NONE, egui::Key::Backspace))
        });
        if nav_up {
            if let Some(ref cur) = self.finder_current_dir {
                if let Some(parent) = cur.parent() {
                    self.finder_current_dir = Some(parent.to_path_buf());
                    self.selected_table_rows.clear();
                    self.selected_table_rows.insert(0);
                    toasts.borrow_mut().info("Navigated to parent directory");
                }
            }
        }

        // 1. MacPathControl (NSPathControl) with dynamic real directory + trailing selection
        let (dir_parents, path_titles) = build_parent_chain(&self.finder_current_dir);
        let mut path_items: Vec<MacPathComponent> = path_titles
            .iter()
            .map(|s| MacPathComponent::folder(s.as_str()))
            .collect();

        // Authentic macOS Finder Path Bar: append selected item as final trailing component
        let active_row_idx = self.selected_table_rows.iter().next().copied();
        let selected_item = active_row_idx.and_then(|idx| visible_rows.get(idx));
        let trailing_item = selected_item.filter(|item| item.name != "..");

        if let Some(item) = trailing_item {
            if item.is_dir {
                path_items.push(MacPathComponent::folder(&item.name));
            } else {
                path_items.push(MacPathComponent::file(&item.name));
            }
        }

        if !path_items.is_empty() {
            content_ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Path:").size(11.0).strong());
                let avail_w = (ui.available_width() - 24.0).max(260.0);
                let out = MacPathControl::new("studio_path_v05", &path_items)
                    .theme(theme)
                    .show_choose_button(true)
                    .width(avail_w)
                    .show(ui);

                if let Some(clicked) = out.clicked_segment {
                    let dir_chain_len = path_titles.len();
                    if clicked < dir_chain_len {
                        if let Some(ref parents) = dir_parents {
                            if let Some(target) = parents.get(clicked) {
                                self.finder_current_dir = Some(target.clone());
                                self.selected_table_rows.clear();
                                self.selected_table_rows.insert(0);
                            }
                        } else {
                            self.active_path_idx = clicked;
                        }
                        if let Some(comp) = path_items.get(clicked) {
                            toasts
                                .borrow_mut()
                                .info(format!("Navigated to {}", comp.title));
                        }
                    } else if clicked == dir_chain_len {
                        if let Some(item) = trailing_item {
                            if item.is_dir {
                                if let Some(ref target_dir) = item.dir_path {
                                    self.finder_current_dir = Some(target_dir.clone());
                                    self.selected_table_rows.clear();
                                    self.selected_table_rows.insert(0);
                                    toasts
                                        .borrow_mut()
                                        .info(format!("Opened folder: {}", item.name));
                                }
                            }
                        }
                    }
                }
                if out.choose_clicked {
                    let cur = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    self.finder_current_dir = Some(cur);
                    self.selected_table_rows.clear();
                    self.selected_table_rows.insert(0);
                    toasts.borrow_mut().info("Loaded current working directory");
                }
            });
            content_ui.add_space(8.0);
        }

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

        // Authentic macOS AppKit doubleAction / Key::Enter on table rows
        let double_clicked_action = tbl_out.double_clicked_row.and_then(|dc_idx| {
            visible_rows
                .get(dc_idx)
                .map(|item| (item.is_dir, item.name.clone(), item.dir_path.clone()))
        });

        if let Some((is_dir, name, dir_path)) = double_clicked_action {
            if is_dir {
                if let Some(target_dir) = dir_path {
                    self.finder_current_dir = Some(target_dir);
                    self.selected_table_rows.clear();
                    self.selected_table_rows.insert(0);
                    toasts.borrow_mut().info(format!("Opened folder: {name}"));
                }
            } else if name.to_lowercase().ends_with(".wav") {
                if let Some(path) = dir_path {
                    self.load_wav_file(&path, &name, toasts);
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
        self.render_state4_publish_queue(content_ui, theme, toasts, window_active, t);
    }
}
