//! Target-action dispatcher for `.xib`/`.nib` scenes (`REQ-013-02Q`).

use super::CenterState;
use crate::{CardModel, GalleryScreen};
use egui_macview::{controls::MacOriginAnchor, theme::MacTheme, MacControlSize, MacToastManager};
use std::cell::RefCell;

impl CenterState {
    /// Resolves the dynamic `MacControlSize` of State 6 inspector controls from the active size checkboxes.
    pub fn resolved_state6_control_size(&self) -> MacControlSize {
        if self.nib_check_mini_mixed && !self.nib_check_regular {
            MacControlSize::Mini
        } else if self.nib_check_small && !self.nib_check_regular {
            MacControlSize::Small
        } else {
            MacControlSize::Regular
        }
    }

    /// Applies an OS preset selection (`nibPresetIdx`) from the `.xib` `PopUpButton` options across State 6.
    pub(super) fn apply_os_preset_profile(
        &mut self,
        store: &super::nib_showcase_builders::ShowcaseSceneDataStore,
        bid: egui_macview::scene::BindingId,
        idx: usize,
        card_cell: &RefCell<CardModel>,
        toasts: &RefCell<MacToastManager>,
    ) {
        use egui_macview::scene::MacSceneDataSource;
        self.nib_preset_idx = idx;
        if let Some(opt_lbl) = store.get_options(bid).get(idx).cloned() {
            self.nib_small_text = opt_lbl.clone();
            toasts.borrow_mut().info(format!("Applied Profile: {opt_lbl}"));
        }
        match idx {
            1 => {
                (self.nib_check_regular, self.nib_check_small, self.nib_check_mini_mixed) = (false, true, false);
                (self.showcase_control_size_idx, self.nib_unit_idx, self.advanced_tab_idx) = (1, 1, 1);
                self.nib_width = "2560".into();
            }
            2 => {
                (self.nib_check_regular, self.nib_check_small, self.nib_check_mini_mixed) = (false, false, true);
                (self.showcase_control_size_idx, self.nib_unit_idx, self.advanced_tab_idx) = (2, 2, 2);
                self.nib_width = "1440".into();
            }
            _ => {
                (self.nib_check_regular, self.nib_check_small, self.nib_check_mini_mixed) = (true, false, false);
                (self.showcase_control_size_idx, self.nib_unit_idx, self.advanced_tab_idx) = (0, 0, 0);
                self.nib_width = "1920".into();
            }
        }
        let palette = MacTheme::default().tag_label_palette();
        let col = palette[idx % palette.len()];
        self.accent_index = idx % palette.len();
        card_cell.borrow_mut().fill_color = col;
        self.wcag_bg = col;
        self.symbol_tint = col;
        self.thumb_glass_color = col;
    }

    /// Synchronizes a `MacCanvasFooter` zoom preset (`0.5`, `1.0`, `2.0`) across `card.scale_pct`, `.nib` stores, and the active screen.
    pub fn apply_footer_zoom_preset(&mut self, z: f32, card_cell: &RefCell<CardModel>, active_screen: GalleryScreen) {
        let pct = (z * 100.0).clamp(25.0, 250.0);
        card_cell.borrow_mut().scale_pct = pct;
        if let Some((_, st0)) = self.screen_nib_scenes.get_mut(0) {
            st0.set_path_f64("scalePercent", pct as f64);
        }
        self.nib_studio.set_path_f64("scalePercent", pct as f64);
        match active_screen {
            GalleryScreen::CanvasPhysics => {}
            GalleryScreen::VectorAssets => {
                let (g_idx, pt) = if z < 0.75 {
                    (0, 10.0)
                } else if z > 1.5 {
                    (2, 20.0)
                } else {
                    (1, 13.0)
                };
                (self.symbol_preview_size_idx, self.typo_point_size) = (g_idx, pt);
                if let Some((_, st1)) = self.screen_nib_scenes.get_mut(1) {
                    st1.set_path_i64("glyphScaleIdx", g_idx as i64);
                    st1.set_path_f64("pointSize", pt as f64);
                }
            }
            GalleryScreen::DesignTokensWcag => {
                let sz_idx = if z < 0.75 {
                    2
                } else if z > 1.5 {
                    0
                } else {
                    1
                };
                self.showcase_control_size_idx = sz_idx;
                if let Some((_, st2)) = self.screen_nib_scenes.get_mut(2) {
                    st2.set_path_i64("matrixControlSize", sz_idx as i64);
                }
            }
            GalleryScreen::FinderDataBrowser => {
                self.media_zoom = z.clamp(0.5, 2.0);
            }
            GalleryScreen::AvKitMediaStudio => {
                self.media_zoom = z.clamp(0.5, 2.5);
                if let Some((_, st4)) = self.screen_nib_scenes.get_mut(4) {
                    st4.set_path_f64("mediaZoom", self.media_zoom as f64);
                }
            }
            GalleryScreen::AdvancedControls => {
                (self.nib_check_regular, self.nib_check_small, self.nib_check_mini_mixed) = (z > 1.5, (0.75..=1.5).contains(&z), z < 0.75);
            }
        }
    }

    pub(super) fn dispatch_pending_store_actions(
        &mut self,
        store: &mut super::nib_showcase_builders::ShowcaseSceneDataStore,
        card_cell: &std::cell::RefCell<CardModel>,
        active_screen: &mut crate::GalleryScreen,
        sidebar: &mut super::SidebarState,
        inspector: &mut super::inspector::InspectorState,
        toasts: &std::cell::RefCell<egui_macview::MacToastManager>,
    ) {
        let actions: Vec<String> = std::mem::take(&mut store.pending_actions);
        for act in actions {
            match act.as_str() {
                "resetTransform:" => {
                    let mut c = card_cell.borrow_mut();
                    (c.rotation_deg, c.scale_pct, c.opacity, c.skew_deg) = (0.0, 100.0, 100.0, 0.0);
                    (c.width, c.height, c.radius, c.alignment_idx) = ("260.0".into(), "150.0".into(), "10.0".into(), 1);
                    (self.origin_anchor, self.card_drag_offset) = (MacOriginAnchor::Center, egui::Vec2::ZERO);
                    store.set_path_f64("scalePercent", 100.0);
                    store.set_path_i64("originAnchor", 4);
                    self.nib_studio.set_path_f64("scalePercent", 100.0);
                    self.nib_studio.set_path_i64("originAnchor", 4);
                    toasts.borrow_mut().info("NIB Action: Reset transform to identity");
                }
                "openVectorStudio:" | "applyTransform:" => {
                    (*active_screen, sidebar.active_screen) = (crate::GalleryScreen::CanvasPhysics, crate::GalleryScreen::CanvasPhysics);
                    toasts.borrow_mut().success("NIB Action: Applied & switched to Vector Studio");
                }
                "resetInspector:" | "resetTypography:" | "resetThemeStudio:" => {
                    let mut c = card_cell.borrow_mut();
                    (c.width, c.height, c.scale_pct, c.alignment_idx) = ("260.0".into(), "150.0".into(), 100.0, 1);
                    (self.nib_width, self.nib_axis_count, self.origin_anchor) = ("1920".into(), 2, MacOriginAnchor::Center);
                    (self.typo_font_family, self.typo_tracking_em, self.typo_point_size, self.typo_optical_kern, self.typo_font_weight) =
                        (0, 0.0, 13.0, true, 0);
                    (inspector.unit_segmented_idx, inspector.snapping_enabled) = (0, true);
                    toasts.borrow_mut().info("NIB Action: Reset metrics to defaults");
                }
                "applyInspector:" => toasts.borrow_mut().success(format!("NIB Action: Applied Grid ({}pt)", self.nib_width)),
                "pulseSpring:" => {
                    self.bounce_start_time = Some(-1.0);
                    toasts.borrow_mut().info("Triggered CASpringAnimation harmonic pulse");
                }
                "togglePopover:" => {
                    (self.popover_open, self.spring_start_time) = (!self.popover_open, Some(-1.0));
                }
                "triggerChoreo:" => {
                    self.cascade_start_time = Some(-1.0);
                    toasts.borrow_mut().info("Triggered cascading spring choreography");
                }
                "vectorActionA:" | "matrixAction:" => {
                    self.vector_action_count += 1;
                    toasts.borrow_mut().success(format!("NIB Action #{}: Executed", self.vector_action_count));
                }
                "toastInfo:" => toasts.borrow_mut().info("AppKit HUD Notification"),
                "toastSuccess:" => toasts.borrow_mut().success("State Snapshot Verified"),
                "toastWarning:" => toasts.borrow_mut().warning("Contrast Threshold Alert"),
                "toastError:" => toasts.borrow_mut().error("Validation Error Alert"),
                "wcagPresetAqua:" => {
                    let aqua = MacTheme::new(egui_macview::MacAppearance::Aqua);
                    (self.wcag_fg, self.wcag_bg) = (aqua.label(), aqua.control_background());
                }
                "wcagPresetDark:" => {
                    let dark = MacTheme::new(egui_macview::MacAppearance::DarkAqua);
                    (self.wcag_fg, self.wcag_bg) = (dark.on_accent_text(), dark.control_accent());
                }
                "wcagPresetLow:" => {
                    let aqua = MacTheme::new(egui_macview::MacAppearance::Aqua);
                    (self.wcag_fg, self.wcag_bg) = (aqua.tertiary_label(), aqua.separator());
                }
                "openFinderUrl:" => {
                    let candidate = std::path::PathBuf::from(self.finder_url_input.trim());
                    if candidate.is_dir() {
                        self.finder_current_dir = Some(candidate);
                        toasts.borrow_mut().success(format!("Opened directory: {}", self.finder_url_input));
                    } else if candidate.exists() {
                        let _ = self.nib_studio.load_nib_or_xib_file(&candidate, toasts);
                    } else {
                        self.search_query = self.finder_url_input.clone();
                        toasts.borrow_mut().info(format!("Filtered Finder by: {}", self.finder_url_input));
                    }
                }
                "toggleFinderLocal:" => {
                    if self.finder_current_dir.is_some() {
                        self.finder_current_dir = None;
                        toasts.borrow_mut().info("Switched Finder to Studio Asset Library");
                    } else {
                        let dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/Users/studio/Local_Disk"));
                        self.finder_current_dir = Some(dir);
                        toasts.borrow_mut().success("Local Disk Mode active (select local folder in browser)");
                    }
                }
                "toggleMediaPlayback:" | "togglePlay:" => {
                    self.media_playing = !self.media_playing;
                    if self.media_playing && self.media_sound_output {
                        let wav =
                            self.audio_track.to_wav_bytes_mixed(self.media_playhead, self.media_out_point, self.media_gain, self.media_pan);
                        super::audio_media::NativeAudioOutput::play_wav(&wav);
                    } else if !self.media_playing {
                        super::audio_media::NativeAudioOutput::stop();
                    }
                }
                "stepFramePrev:" => {
                    self.media_playhead = (self.media_playhead - 1.0 / 24.0).max(self.media_in_point);
                }
                "stepFrameNext:" => {
                    self.media_playhead = (self.media_playhead + 1.0 / 24.0).min(self.media_out_point);
                }
                "enqueueJob:" => {
                    let fmt = self.scene_node_option(3, "s4-pop-fmt", self.export_format.min(3)).to_string();
                    let idx = self.finder_custom_entries.len() + 1;
                    self.finder_custom_entries.push(super::FinderCustomEntry {
                        name: format!("Export_Job_{idx:02}.mov"),
                        path: std::path::PathBuf::from(&self.export_path),
                        is_dir: false,
                        size_mb: (self.export_bitrate as u32).max(12),
                        codec: fmt.clone(),
                        duration: "00:01:24:00".to_string(),
                        rating: 5,
                        tags: vec!["Queued".into(), format!("{}w", self.worker_count)],
                    });
                    toasts.borrow_mut().success(format!("Enqueued Export_Job_{idx:02} ({fmt})"));
                }
                "nibPlusAxis:" | "addTransformLayer:" | "addKeyframe:" => {
                    self.nib_axis_count = self.nib_axis_count.saturating_add(1);
                    toasts.borrow_mut().info(format!("Added keyframe / axis (#{})", self.nib_axis_count));
                }
                "openNibStudio:" => {
                    let base = self.finder_current_dir.clone().unwrap_or_else(super::nib_showcase_builders::resolve_showcase_nib_dir);
                    let _ = self.nib_studio.load_nib_or_xib_file(&base.join(super::nib_showcase_builders::SHOWCASE_NIB_FILES[0]), toasts);
                }
                "startPublishQueue:" | "publishNow:" | "bounceMaster:" | "exportTrimmedWav:" => {
                    (self.publish_progress, self.is_publishing) = (0.05, true);
                    toasts.borrow_mut().success(format!("Started job ({})", act.trim_end_matches(':')));
                }
                "clearCompleted:" => {
                    self.finder_custom_entries.clear();
                    (self.publish_progress, self.is_publishing) = (0.0, false);
                    toasts.borrow_mut().info("Cleared completed jobs");
                }
                "resetAudioStrip:" => {
                    (self.media_gain, self.media_pan, self.media_zoom, self.media_speed, self.sat_drive) = (1.0, 0.0, 1.0, 1.0, 0.35);
                    (self.media_playhead, self.media_playing, self.bus_mode) = (self.media_in_point, false, 0);
                    toasts.borrow_mut().info("Reset AVKit audio strip to unity");
                }
                "auditContrast:" => {
                    let ratio = egui_macview::contrast_ratio(self.wcag_fg, self.wcag_bg);
                    toasts.borrow_mut().success(format!("WCAG Contrast Ratio: {ratio:.2}:1 (Target {:.1}:1)", self.contrast_target));
                }
                "saveThemeSettings:" | "copySnippet:" => toasts.borrow_mut().success("NIB Action: Verified & Copied"),
                _ => toasts.borrow_mut().info(format!("NIB Action: {act}")),
            }
        }
    }

    /// Renders the body of the selected `<tabViewItem>` in State 6 (`s6-tabs`).
    pub fn render_state6_tab_item(&mut self, tab_ui: &mut egui::Ui, theme: &MacTheme, tab_idx: usize, body_rect: egui::Rect) {
        use egui_macview::controls::{
            MacCollectionView, MacCollectionViewFlowLayout, MacDate, MacDatePicker, MacDatePickerElementFlags, MacDatePickerStyle,
            MacInlineActionButton, MacLabel, MacLabelColor, MacPlayerControlsStyle, MacPlayerView, MacThumbnailCell,
        };
        let tv_id = tab_ui.scope_id().with("s6_tab_body_state");
        let (mut playing, mut progress, mut grid_sel, mut grid_count, mut date) = tab_ui
            .data(|d| d.get_temp::<(bool, f32, std::collections::BTreeSet<usize>, usize, MacDate)>(tv_id))
            .unwrap_or_else(|| (false, 0.35, std::collections::BTreeSet::new(), 24usize, self.advanced_date));
        if playing {
            progress = (progress + 0.0025).fract();
            tab_ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
        }
        match tab_idx {
            0 => {
                let (vid_rect, vid_resp) = tab_ui.allocate_exact_size(body_rect.size(), egui::Sense::click());
                if vid_resp.clicked() {
                    playing = !playing;
                }
                let vp = tab_ui.painter_at(vid_rect);
                vp.rect_filled(vid_rect, egui::CornerRadius::same(theme.tab_view().corner_radius.points() as u8), theme.hud_background());
                let wave_y = vid_rect.center().y;
                let phase = progress * std::f32::consts::TAU * 2.0;
                let inset_x = theme.tab_view().regular_inset_horizontal.points();
                let pts: Vec<egui::Pos2> = (0..=24)
                    .map(|i| {
                        let t = i as f32 / 24.0;
                        let x = vid_rect.min.x + inset_x + t * (vid_rect.width() - inset_x * 2.0);
                        let y = wave_y + (t * std::f32::consts::TAU * 2.0 + phase).sin() * theme.player_view().scrubber_knob_size.points();
                        egui::pos2(x, y)
                    })
                    .collect();
                vp.add(egui::Shape::line(pts, egui::Stroke::new(theme.box_view().border_width.points(), theme.control_accent())));
                let hud_h = theme.player_view().hud_height.points();
                let hud_rect =
                    egui::Rect::from_min_max(egui::pos2(vid_rect.min.x, (vid_rect.max.y - hud_h).max(vid_rect.min.y)), vid_rect.max);
                let mut hud_ui = tab_ui.new_child(egui::UiBuilder::new().id_salt("s6_av_hud_child").max_rect(hud_rect));
                hud_ui.add(
                    MacPlayerView::new()
                        .controls_style(MacPlayerControlsStyle::Floating)
                        .is_playing_mut(&mut playing)
                        .progress_mut(&mut progress)
                        .theme(theme),
                );
            }
            1 => {
                tab_ui.horizontal(|hdr_ui| {
                    MacLabel::new(format!("#{grid_count}"))
                        .control_size(MacControlSize::Small)
                        .color(MacLabelColor::Secondary)
                        .theme(theme)
                        .show(hdr_ui);
                    if MacInlineActionButton::new("+").theme(theme).show(hdr_ui).clicked() {
                        grid_count = grid_count.saturating_add(1);
                    }
                    if MacInlineActionButton::new("−").theme(theme).enabled(grid_count > 1).show(hdr_ui).clicked() {
                        grid_count = grid_count.saturating_sub(1).max(1);
                    }
                });
                let items: Vec<usize> = (0..grid_count).collect();
                let mut clicked_item: Option<usize> = None;
                egui_macview::MacScrollView::new("s6_tv_coll_scroll")
                    .theme(theme)
                    .vertical(true)
                    .auto_hide(false)
                    .max_width(body_rect.width())
                    .max_height(body_rect.height())
                    .show(tab_ui, |scroll_ui| {
                        let sel_copy = grid_sel.clone();
                        MacCollectionView::new(&items, &mut grid_sel)
                            .layout(MacCollectionViewFlowLayout::default())
                            .theme(theme)
                            .show_with_item_tracker(scroll_ui, |item_ui, item| {
                                let frac = ((*item % 12 + 1) as f32) / 12.0;
                                let (accent, sep) = (theme.control_accent(), theme.separator());
                                item_ui.vertical_centered(|c| {
                                    if MacThumbnailCell::new()
                                        .theme(theme)
                                        .selected(sel_copy.contains(item))
                                        .square(26.0)
                                        .custom_draw(move |p, r| {
                                            let (ctr, rad) = (r.center(), r.width() * 0.38);
                                            p.circle_stroke(ctr, rad, egui::Stroke::new(1.0, sep));
                                            let end_a = -std::f32::consts::FRAC_PI_2 + frac * std::f32::consts::TAU;
                                            p.line_segment(
                                                [ctr, ctr + egui::vec2(end_a.cos() * rad, end_a.sin() * rad)],
                                                egui::Stroke::new(1.8, accent),
                                            );
                                            p.circle_filled(ctr + egui::vec2(end_a.cos() * rad, end_a.sin() * rad), 2.2, accent);
                                        })
                                        .show(c)
                                        .clicked()
                                    {
                                        clicked_item = Some(*item);
                                    }
                                    MacLabel::new(format!("#{item}")).control_size(MacControlSize::Mini).theme(theme).show(c);
                                });
                            });
                    });
                if let Some(chosen) = clicked_item {
                    grid_sel = std::collections::BTreeSet::from([chosen]);
                }
            }
            _ => {
                tab_ui.add(
                    MacDatePicker::new()
                        .date(&mut date)
                        .style(MacDatePickerStyle::ClockAndCalendar)
                        .elements(MacDatePickerElementFlags::DateAndTime)
                        .theme(theme),
                );
                self.advanced_date = date;
            }
        }
        tab_ui.data_mut(|d| d.insert_temp(tv_id, (playing, progress, grid_sel, grid_count, date)));
    }
}
