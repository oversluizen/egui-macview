//! Central interactive state and layout orchestrator across the 6 Gallery Studio screens.

use super::{
    audio_media,
    finder_data::{FinderCustomEntry, FinderDisplayRow},
    types::{AvKitStudioSnapshot, GalleryScreen, InspectorMode},
    CardModel,
};
use egui::{pos2, vec2, Align, Color32, CornerRadius, Layout, Pos2, Rect, Ui};
use egui_macview::{
    theme::MacTheme, CASpring, MacCanvasFooter, MacCanvasFooterAction, MacScrollView, MacSearchField, MacSortDirection, MacSymbol,
    MacToastManager, NSUndoManager, CANVAS_FOOTER_HEIGHT,
};
use std::{cell::RefCell, collections::BTreeSet};

/// Composite cache key to avoid per-frame disk scans and sorting in State 4 Finder.
#[derive(Clone, PartialEq, Debug)]
pub struct FinderCacheKey {
    pub current_dir: Option<std::path::PathBuf>,
    pub custom_len: usize,
    pub ratings: [u32; 6],
    pub tokens: Vec<Vec<String>>,
    pub search_query: String,
    pub sort_col: usize,
    pub sort_dir: MacSortDirection,
}

/// Shared state across the 6 interactive Center Studio States (`v0.1` – `v0.7`).
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
    pub showcase_popup_idx: usize,
    pub active_path_idx: usize,
    pub table_sort_col: usize,
    pub table_sort_dir: MacSortDirection,
    pub selected_table_rows: BTreeSet<usize>,
    pub asset_tokens: [Vec<String>; 6],
    pub asset_ratings: [u32; 6],
    pub publish_progress: f32,
    pub publish_indeterminate: bool,
    pub is_publishing: bool,
    pub finder_current_dir: Option<std::path::PathBuf>,
    pub finder_custom_entries: Vec<FinderCustomEntry>,
    pub finder_url_input: String,
    pub finder_cached_rows: Vec<FinderDisplayRow>,
    pub finder_cache_key: Option<FinderCacheKey>,
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
    pub audio_levels_cached: (f32, f32, f32, f32),
    pub audio_levels_key: (f64, f32, f32),
    pub audio_track: audio_media::AudioTrack,
    pub media_last_audio_restart: f64,
    pub media_was_window_active: bool,
    pub avkit_undo: NSUndoManager<AvKitStudioSnapshot>,
    pub last_stable_avkit: Option<AvKitStudioSnapshot>,
    pub advanced_combo_text: String,
    pub advanced_date: egui_macview::controls::MacDate,
    pub advanced_tab_idx: usize,
    pub advanced_disclosure_open: bool,
    pub advanced_button_open: bool,
    pub advanced_rules: Vec<egui_macview::controls::MacRuleRow>,
    pub nib_check_regular: bool,
    pub nib_check_small: bool,
    pub nib_check_mini_mixed: bool,
    pub nib_axis_count: usize,
    pub nib_inspector_enabled: bool,
    pub nib_width: String,
    pub nib_unit_idx: usize,
    pub nib_preset_idx: usize,
    pub nib_small_text: String,
    pub origin_anchor: egui_macview::controls::MacOriginAnchor,
    pub tag_color: Option<usize>,
    pub action_items: Vec<String>,
    pub show_grid: bool,
    pub typo_font_family: usize,
    pub typo_tracking_em: f32,
    pub typo_point_size: f32,
    pub typo_optical_kern: bool,
    pub typo_font_weight: usize,
    pub matrix_check: bool,
    pub matrix_switch: bool,
    pub matrix_slider: f32,
    pub accent_index: usize,
    pub contrast_target: f32,
    pub high_contrast: bool,
    pub reduce_transparency: bool,
    pub export_path: String,
    pub export_format: usize,
    pub export_bitrate: f32,
    pub worker_count: usize,
    pub open_after_export: bool,
    pub verify_checksums: bool,
    pub media_speed: f32,
    pub sat_drive: f32,
    pub brickwall_limiter: bool,
    pub bus_mode: usize,
    pub codec_preset: usize,
    pub last_video_viewport_rect: Rect,
    pub export_preset: String,
    pub thumb_dial_latched: egui_macview::MacThumbIntentMask,
    pub thumb_dial_layer: egui_macview::MacThumbDialLayer,
    pub thumb_dial_rotary: f32,
    pub thumb_dial_preset_idx: usize,
    pub thumb_config_mode: usize,
    pub thumb_glass_color: Color32,
    pub nib_studio: super::nib_window_studio::NibShowcaseWindowState,
    pub screen_nib_scenes: Vec<(egui_macview::scene::MacSceneIR, super::nib_showcase_builders::ShowcaseSceneDataStore)>,
}

impl Default for CenterState {
    fn default() -> Self {
        let default_theme = MacTheme::default();
        let track = audio_media::AudioTrack::default();
        let waveform = track.waveform_peaks.clone();
        Self {
            search_query: String::new(),
            spring: default_theme.motion().hud_panel_transition.to_spring(),
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
            symbol_tint: default_theme.control_accent(),
            demo_scroll_offset: egui::Vec2::ZERO,
            wcag_fg: default_theme.label(),
            wcag_bg: default_theme.window_background(),
            row_container_width: 480.0,
            cascade_start_time: None,
            card_menu_open: false,
            card_menu_pos: Pos2::ZERO,
            card_menu_selected: None,
            showcase_control_size_idx: 0,
            showcase_controls_enabled: true,
            showcase_text: "1920 x 1080".to_string(),
            showcase_popup_idx: 0,
            active_path_idx: 3,
            table_sort_col: 0,
            table_sort_dir: MacSortDirection::Ascending,
            selected_table_rows: BTreeSet::from([0]),
            asset_tokens: [
                vec!["Vector".to_string(), "SVG".to_string(), "4K".to_string()],
                vec!["ProRes".to_string(), "HDR10".to_string()],
                vec!["Lossless".to_string(), "24-bit".to_string()],
                vec!["LUT".to_string(), "Color".to_string()],
                vec!["UI Kit".to_string(), "Sonoma".to_string()],
                vec!["Release".to_string(), "Signed".to_string()],
            ],
            asset_ratings: [5, 4, 5, 3, 5, 4],
            publish_progress: 0.68,
            publish_indeterminate: false,
            is_publishing: false,
            finder_current_dir: {
                let nib_dir = super::nib_showcase_builders::resolve_showcase_nib_dir();
                super::nib_showcase_builders::ensure_showcase_nib_files(&nib_dir);
                if nib_dir.exists() {
                    Some(nib_dir)
                } else {
                    std::env::current_dir().ok()
                }
            },
            finder_custom_entries: Vec::new(),
            finder_url_input: "assets/nib_showcase".to_string(),
            finder_cached_rows: Vec::new(),
            finder_cache_key: None,
            media_playing: false,
            media_loop: false,
            media_in_point: 4.0,
            media_out_point: 26.0,
            media_playhead: 4.0,
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
            audio_levels_cached: (7.4, 8.6, 8.2, 9.1),
            audio_levels_key: (-1.0, -1.0, -99.0),
            audio_track: track,
            media_last_audio_restart: -10.0,
            media_was_window_active: true,
            avkit_undo: NSUndoManager::new(),
            last_stable_avkit: None,
            advanced_combo_text: "Apple ProRes 422 HQ".to_string(),
            advanced_date: egui_macview::controls::MacDate::default(),
            advanced_tab_idx: 0,
            advanced_disclosure_open: true,
            advanced_button_open: true,
            advanced_rules: vec![egui_macview::controls::MacRuleRow::new(vec![0, 0]), egui_macview::controls::MacRuleRow::new(vec![1, 1])],
            nib_check_regular: true,
            nib_check_small: false,
            nib_check_mini_mixed: false,
            nib_axis_count: 2,
            nib_inspector_enabled: true,
            nib_width: "1920".to_string(),
            nib_unit_idx: 0,
            nib_preset_idx: 0,
            nib_small_text: "Small 19pt".to_string(),
            origin_anchor: egui_macview::controls::MacOriginAnchor::Center,
            tag_color: Some(7),
            action_items: vec!["Transform Layer".into(), "Color Tag".into()],
            show_grid: true,
            typo_font_family: 0,
            typo_tracking_em: 0.0,
            typo_point_size: 13.0,
            typo_optical_kern: true,
            typo_font_weight: 0,
            matrix_check: true,
            matrix_switch: true,
            matrix_slider: 64.0,
            accent_index: 0,
            contrast_target: 7.0,
            high_contrast: false,
            reduce_transparency: false,
            export_path: "/Volumes/StudioRAID/Exports".to_string(),
            export_format: 0,
            export_bitrate: 120.0,
            worker_count: 8,
            open_after_export: true,
            verify_checksums: true,
            media_speed: 1.0,
            sat_drive: 0.35,
            brickwall_limiter: true,
            bus_mode: 0,
            codec_preset: 0,
            last_video_viewport_rect: Rect::ZERO,
            export_preset: "ProRes 4444 XQ".to_string(),
            thumb_dial_latched: egui_macview::MacThumbIntentMask::NONE,
            thumb_dial_layer: egui_macview::MacThumbDialLayer::CompassModifiers,
            thumb_dial_rotary: 0.35,
            thumb_dial_preset_idx: 2,
            thumb_config_mode: 0,
            thumb_glass_color: default_theme.control_accent(),
            nib_studio: super::nib_window_studio::NibShowcaseWindowState::default(),
            screen_nib_scenes: super::nib_showcase_builders::compile_gallery_screen_scenes(),
        }
    }
}

pub struct CenterRenderArgs<'a> {
    pub card_cell: &'a RefCell<CardModel>,
    pub undo_manager: &'a RefCell<NSUndoManager<CardModel>>,
    pub last_stable_card: &'a RefCell<CardModel>,
    pub inspector_mode: InspectorMode,
    pub snapping_enabled: bool,
    pub unit_segmented_idx: usize,
    pub vis_canvas: bool,
    pub lock_canvas: bool,
}

impl CenterState {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        args: CenterRenderArgs<'_>,
        active_screen: &mut GalleryScreen,
        toasts: &RefCell<MacToastManager>,
    ) {
        let scale_pct = args.card_cell.borrow().scale_pct;
        let avail_rect = ui.available_rect_before_wrap();
        ui.painter_at(avail_rect).rect_filled(avail_rect, CornerRadius::ZERO, theme.control_background());

        let (padding, search_h, footer_h) = (16.0, 24.0, CANVAS_FOOTER_HEIGHT);
        let search_rect = Rect::from_min_size(
            pos2(avail_rect.min.x + padding, avail_rect.min.y + padding),
            vec2(avail_rect.width() - padding * 2.0, search_h),
        );
        let mut search_ui = ui.new_child(
            egui::UiBuilder::new().id_salt("gallery_center_search_ui").max_rect(search_rect).layout(Layout::left_to_right(Align::Center)),
        );
        MacSearchField::new(&mut self.search_query)
            .theme(theme)
            .placeholder(active_screen.search_placeholder())
            .width(search_rect.width())
            .corner_radius(6.0)
            .show(&mut search_ui);

        let footer_rect = Rect::from_min_max(pos2(avail_rect.min.x, avail_rect.max.y - footer_h), avail_rect.max);
        let mut footer_ui = ui.new_child(
            egui::UiBuilder::new().id_salt("gallery_center_footer_ui").max_rect(footer_rect).layout(Layout::left_to_right(Align::Center)),
        );
        let breadcrumbs = active_screen.breadcrumb_path();
        let cursor_pos = ui.input(|i| i.pointer.hover_pos()).map(|p| pos2(p.x - avail_rect.min.x, p.y - avail_rect.min.y));
        match MacCanvasFooter::new(&breadcrumbs, cursor_pos, scale_pct / 100.0)
            .theme(theme)
            .height(footer_h)
            .show(&mut footer_ui, footer_rect.width())
        {
            Some(MacCanvasFooterAction::SelectZoom(z)) => {
                args.card_cell.borrow_mut().scale_pct = z * 100.0;
                toasts.borrow_mut().info("Preset applied");
            }
            Some(MacCanvasFooterAction::BreadcrumbClicked(crumb_idx)) => {
                *active_screen = if crumb_idx == 0 {
                    GalleryScreen::CanvasPhysics
                } else {
                    GalleryScreen::ALL[(*active_screen as usize + 1) % GalleryScreen::ALL.len()]
                };
                toasts.borrow_mut().info(active_screen.title());
            }
            None => {}
        }

        let scroll_rect =
            Rect::from_min_max(pos2(avail_rect.min.x, search_rect.max.y + 12.0), pos2(avail_rect.max.x, avail_rect.max.y - footer_h));
        let mut scroll_container_ui = ui.new_child(
            egui::UiBuilder::new().id_salt("gallery_center_scroll_container_ui").max_rect(scroll_rect).layout(Layout::top_down(Align::Min)),
        );

        MacScrollView::new(("center_canvas_scroll", *active_screen as usize))
            .theme(theme)
            .vertical(true)
            .horizontal(true)
            .max_height(scroll_rect.height())
            .max_width(scroll_rect.width())
            .show(&mut scroll_container_ui, |content_ui| {
                self.render_active_screen_nib(content_ui, theme, args, *active_screen, toasts);
            });
    }
}
