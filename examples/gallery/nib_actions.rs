//! Target-action dispatcher for `.xib`/`.nib` scenes (`REQ-013-02Q`).

use super::CenterState;
use crate::CardModel;
use egui_macview::controls::MacOriginAnchor;

impl CenterState {
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
                    (self.wcag_fg, self.wcag_bg) = (egui::Color32::from_rgb(20, 20, 22), egui::Color32::WHITE);
                }
                "wcagPresetDark:" => {
                    (self.wcag_fg, self.wcag_bg) = (egui::Color32::WHITE, egui::Color32::from_rgb(10, 132, 255));
                }
                "wcagPresetLow:" => {
                    (self.wcag_fg, self.wcag_bg) = (egui::Color32::from_rgb(150, 150, 155), egui::Color32::from_rgb(210, 210, 215));
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
                    self.finder_current_dir = if self.finder_current_dir.is_some() { None } else { std::env::current_dir().ok() };
                }
                "toggleMediaPlayback:" | "togglePlay:" => {
                    self.media_playing = !self.media_playing;
                }
                "stepFramePrev:" => {
                    self.media_playhead = (self.media_playhead - 1.0 / 24.0).max(self.media_in_point);
                }
                "stepFrameNext:" => {
                    self.media_playhead = (self.media_playhead + 1.0 / 24.0).min(self.media_out_point);
                }
                "enqueueJob:" => {
                    let fmt = ["ProRes 4444 XQ", "ProRes 422 HQ", "HEVC 10-bit", "WAV 96kHz"][self.export_format.min(3)];
                    let idx = self.finder_custom_entries.len() + 1;
                    self.finder_custom_entries.push(super::FinderCustomEntry {
                        name: format!("Export_Job_{idx:02}.mov"),
                        path: std::path::PathBuf::from(&self.export_path),
                        is_dir: false,
                        size_mb: (self.export_bitrate as u32).max(12),
                        codec: fmt.to_string(),
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
}
