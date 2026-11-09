//! Central interactive state and layout orchestrator across the 5 Gallery Studio screens.

use super::{
    audio_media,
    finder_data::FinderCustomEntry,
    types::{AvKitStudioSnapshot, GalleryScreen, InspectorMode},
    CardModel,
};
use egui::{pos2, vec2, Align, Color32, CornerRadius, Layout, Pos2, Rect, Ui};
use egui_macview::{
    theme::MacTheme, CASpring, MacCanvasFooter, MacCanvasFooterAction, MacScrollView,
    MacSearchField, MacSortDirection, MacSymbol, MacToastManager, NSUndoManager,
    CANVAS_FOOTER_HEIGHT,
};
use std::cell::RefCell;
use std::collections::BTreeSet;

/// Shared state across the 5 interactive Center Studio States (`v0.1` – `v0.5`).
pub struct CenterState {
    pub search_query: String,
    pub spring: CASpring,
    pub spring_start_time: Option<f64>,
    pub popover_open: bool,
    pub bounce_spring: CASpring,
    pub bounce_start_time: Option<f64>,
    pub hud_pos: Pos2,
    pub card_drag_offset: egui::Vec2,
    pub fixed_grid_snap: bool,
    pub fixed_seg_idx: usize,
    pub vector_action_count: usize,
    pub selected_symbol: MacSymbol,
    pub symbol_preview_size_idx: usize,
    pub symbol_tint: Color32,
    pub demo_scroll_offset: egui::Vec2,
    pub token_tab_idx: usize,
    pub wcag_fg: Color32,
    pub wcag_bg: Color32,
    pub row_container_width: f32,
    pub cascade_start_time: Option<f64>,
    pub card_menu_open: bool,
    pub card_menu_pos: Pos2,
    pub card_menu_selected: Option<usize>,
    pub showcase_control_size_idx: usize,
    pub showcase_controls_enabled: bool,
    pub showcase_text: String,
    pub showcase_num_text: String,
    pub showcase_step_val: f64,
    pub showcase_popup_idx: usize,
    pub showcase_color: Color32,
    // State 4 (v0.5 Finder & Publishing Data Browser)
    pub active_path_idx: usize,
    pub table_sort_col: usize,
    pub table_sort_dir: MacSortDirection,
    pub selected_table_rows: BTreeSet<usize>,
    pub asset_tokens: [Vec<String>; 6],
    pub asset_ratings: [u32; 6],
    pub token_draft: String,
    pub publish_progress: f32,
    pub publish_indeterminate: bool,
    pub is_publishing: bool,
    pub finder_current_dir: Option<std::path::PathBuf>,
    pub finder_custom_entries: Vec<FinderCustomEntry>,
    pub finder_url_input: String,
    // State 5 (v0.5 AVKit Media & Audio Timeline Studio)
    pub media_playing: bool,
    pub media_loop: bool,
    pub media_in_point: f64,
    pub media_out_point: f64,
    pub media_playhead: f64,
    pub media_last_time: Option<f64>,
    pub media_waveform: Vec<f32>,
    pub media_gain: f32,
    pub media_pan: f32,
    pub media_exposure: f32,
    pub media_contrast: f32,
    pub media_zoom: f32,
    pub media_rating: u32,
    pub media_sound_output: bool,
    pub media_discrete_vu: bool,
    pub media_threshold_vu: bool,
    pub audio_level_l: f32,
    pub audio_level_r: f32,
    pub audio_peak_l: f32,
    pub audio_peak_r: f32,
    pub audio_track: audio_media::AudioTrack,
    pub avkit_undo: NSUndoManager<AvKitStudioSnapshot>,
    pub last_stable_avkit: Option<AvKitStudioSnapshot>,
}

impl Default for CenterState {
    fn default() -> Self {
        let track = audio_media::AudioTrack::default();
        let waveform = track.waveform_peaks.clone();
        Self {
            search_query: String::new(),
            spring: MacTheme::default()
                .motion()
                .hud_panel_transition
                .to_spring(),
            spring_start_time: None,
            popover_open: false,
            bounce_spring: CASpring::from_perceptual(0.35, 0.65),
            bounce_start_time: None,
            hud_pos: Pos2::new(24.0, 28.0),
            card_drag_offset: egui::Vec2::ZERO,
            fixed_grid_snap: true,
            fixed_seg_idx: 0,
            vector_action_count: 0,
            selected_symbol: MacSymbol::Search,
            symbol_preview_size_idx: 1,
            symbol_tint: Color32::from_rgb(0, 122, 255),
            demo_scroll_offset: egui::Vec2::ZERO,
            token_tab_idx: 0,
            wcag_fg: Color32::from_rgb(255, 255, 255),
            wcag_bg: Color32::from_rgb(30, 30, 32),
            row_container_width: 320.0,
            cascade_start_time: None,
            card_menu_open: false,
            card_menu_pos: Pos2::ZERO,
            card_menu_selected: None,
            showcase_control_size_idx: 1,
            showcase_controls_enabled: true,
            showcase_text: "MacView Text".to_string(),
            showcase_num_text: "42.0".to_string(),
            showcase_step_val: 1.0,
            showcase_popup_idx: 0,
            showcase_color: Color32::from_rgb(0, 122, 255),
            active_path_idx: 3,
            table_sort_col: 0,
            table_sort_dir: MacSortDirection::Ascending,
            selected_table_rows: BTreeSet::from([0]),
            asset_tokens: [
                vec!["ProRes".into(), "HDR".into(), "Master".into()],
                vec!["ProRes".into(), "4444".into(), "VFX".into()],
                vec!["Stem".into(), "24bit".into(), "48k".into()],
                vec!["Proxy".into(), "HEVC".into(), "Dailies".into()],
                vec!["LUT".into(), "Rec709".into(), "Color".into()],
                vec!["QuickTime".into(), "Master".into(), "Deliverable".into()],
            ],
            asset_ratings: [5, 4, 5, 3, 4, 5],
            token_draft: String::new(),
            publish_progress: 0.68,
            publish_indeterminate: false,
            is_publishing: false,
            finder_current_dir: None,
            finder_custom_entries: Vec::new(),
            finder_url_input: "https://github.com/oversluizen/egui-macview".to_string(),
            media_playing: false,
            media_loop: false,
            media_in_point: 15.0,
            media_out_point: 92.0,
            media_playhead: 36.0,
            media_last_time: None,
            media_waveform: waveform,
            media_gain: 1.0,
            media_pan: 0.0,
            media_exposure: 0.0,
            media_contrast: 1.0,
            media_zoom: 1.0,
            media_rating: 5,
            media_sound_output: false,
            media_discrete_vu: true,
            media_threshold_vu: true,
            audio_level_l: 7.4,
            audio_level_r: 8.6,
            audio_peak_l: 8.2,
            audio_peak_r: 9.1,
            audio_track: track,
            avkit_undo: NSUndoManager::new(),
            last_stable_avkit: None,
        }
    }
}

impl CenterState {
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        card_cell: &RefCell<CardModel>,
        undo_manager: &RefCell<NSUndoManager<CardModel>>,
        last_stable_card: &RefCell<CardModel>,
        inspector_mode: InspectorMode,
        active_screen: &mut GalleryScreen,
        snapping_enabled: bool,
        unit_segmented_idx: usize,
        vis_canvas: bool,
        lock_canvas: bool,
        toasts: &RefCell<MacToastManager>,
    ) {
        let mut card = card_cell.borrow_mut();
        let avail_rect = ui.available_rect_before_wrap();
        ui.painter_at(avail_rect).rect_filled(
            avail_rect,
            CornerRadius::ZERO,
            theme.control_background(),
        );

        let padding = 16.0;
        let search_h = 24.0;
        let search_rect = Rect::from_min_size(
            pos2(avail_rect.min.x + padding, avail_rect.min.y + padding),
            vec2(avail_rect.width() - padding * 2.0, search_h),
        );
        let mut search_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("gallery_center_search_ui")
                .max_rect(search_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        MacSearchField::new(&mut self.search_query)
            .theme(theme)
            .placeholder(active_screen.search_placeholder())
            .width(search_rect.width())
            .corner_radius(6.0)
            .show(&mut search_ui);

        let footer_h = CANVAS_FOOTER_HEIGHT;
        let footer_rect = Rect::from_min_max(
            pos2(avail_rect.min.x, avail_rect.max.y - footer_h),
            avail_rect.max,
        );
        let mut footer_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("gallery_center_footer_ui")
                .max_rect(footer_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        let breadcrumbs = active_screen.breadcrumb_path();
        let cursor_pos = ui
            .input(|i| i.pointer.hover_pos())
            .map(|p| pos2(p.x - avail_rect.min.x, p.y - avail_rect.min.y));
        let footer_action = MacCanvasFooter::new(&breadcrumbs, cursor_pos, card.scale_pct / 100.0)
            .theme(theme)
            .height(footer_h)
            .show(&mut footer_ui, footer_rect.width());

        match footer_action {
            Some(MacCanvasFooterAction::SelectZoom(z)) => {
                card.scale_pct = z * 100.0;
                toasts.borrow_mut().info("Preset applied");
            }
            Some(MacCanvasFooterAction::BreadcrumbClicked(crumb_idx)) => {
                if crumb_idx == 0 {
                    *active_screen = GalleryScreen::CanvasPhysics;
                } else {
                    let cur_idx = *active_screen as usize;
                    *active_screen = GalleryScreen::ALL[(cur_idx + 1) % GalleryScreen::ALL.len()];
                }
                toasts.borrow_mut().info(active_screen.title());
            }
            None => {}
        }

        let scroll_rect = Rect::from_min_max(
            pos2(avail_rect.min.x, search_rect.max.y + 12.0),
            pos2(avail_rect.max.x, avail_rect.max.y - footer_h),
        );
        let mut scroll_container_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("gallery_center_scroll_container_ui")
                .max_rect(scroll_rect)
                .layout(Layout::top_down(Align::Min)),
        );

        MacScrollView::new("center_canvas_scroll")
            .theme(theme)
            .vertical(true)
            .horizontal(false)
            .max_height(scroll_rect.height())
            .max_width(scroll_rect.width())
            .show(
                &mut scroll_container_ui,
                |content_ui| match *active_screen {
                    GalleryScreen::CanvasPhysics => {
                        self.render_state1_canvas(
                            content_ui,
                            theme,
                            &mut card,
                            undo_manager,
                            last_stable_card,
                            inspector_mode,
                            snapping_enabled,
                            unit_segmented_idx,
                            vis_canvas,
                            lock_canvas,
                            toasts,
                        );
                    }
                    GalleryScreen::VectorAssets => {
                        self.render_state2_symbols_layout(content_ui, theme, &mut card, toasts);
                    }
                    GalleryScreen::DesignTokensWcag => {
                        self.render_state3_command_state(
                            content_ui,
                            theme,
                            &mut card,
                            undo_manager,
                            last_stable_card,
                            toasts,
                        );
                    }
                    GalleryScreen::FinderDataBrowser => {
                        self.render_state4_finder_browser(content_ui, theme, toasts);
                    }
                    GalleryScreen::AvKitMediaStudio => {
                        self.render_state5_avkit_media(content_ui, theme, toasts);
                    }
                },
            );
    }

    pub fn current_avkit_snapshot(&self) -> AvKitStudioSnapshot {
        AvKitStudioSnapshot {
            in_point: self.media_in_point,
            out_point: self.media_out_point,
            playhead: self.media_playhead,
            gain: self.media_gain,
            pan: self.media_pan,
            exposure: self.media_exposure,
            contrast: self.media_contrast,
            zoom: self.media_zoom,
            rating: self.media_rating,
        }
    }

    pub fn apply_avkit_snapshot(&mut self, snap: &AvKitStudioSnapshot) {
        self.media_in_point = snap.in_point;
        self.media_out_point = snap.out_point;
        self.media_playhead = snap.playhead.clamp(snap.in_point, snap.out_point);
        self.media_gain = snap.gain;
        self.media_pan = snap.pan;
        self.media_exposure = snap.exposure;
        self.media_contrast = snap.contrast;
        self.media_zoom = snap.zoom;
        self.media_rating = snap.rating;
    }

    pub fn undo_avkit(&mut self) -> bool {
        let mut cur = self.current_avkit_snapshot();
        if self.avkit_undo.undo(&mut cur) {
            self.apply_avkit_snapshot(&cur);
            self.last_stable_avkit = Some(cur);
            true
        } else {
            false
        }
    }

    pub fn redo_avkit(&mut self) -> bool {
        let mut cur = self.current_avkit_snapshot();
        if self.avkit_undo.redo(&mut cur) {
            self.apply_avkit_snapshot(&cur);
            self.last_stable_avkit = Some(cur);
            true
        } else {
            false
        }
    }

    pub fn checkpoint_avkit_if_changed(&mut self) {
        let cur = self.current_avkit_snapshot();
        if let Some(ref prev) = self.last_stable_avkit {
            if prev != &cur {
                self.avkit_undo.push_snapshot(*prev);
                self.last_stable_avkit = Some(cur);
            }
        } else {
            self.last_stable_avkit = Some(cur);
        }
    }
}
