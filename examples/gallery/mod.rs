//! Modular state and sidebar navigation for `macview_gallery` (v0.1 – v0.5 Studio Showcase).

pub mod chrome;
pub mod inspector;
pub mod inspector_nib;
pub mod sidebar;
pub mod state1_canvas;
pub mod state2_symbols_layout;
pub mod state3_command_state;
pub mod state4_finder_browser;
pub mod state5_avkit_media;

pub use chrome::detect_frameless_resize_dir;
pub use inspector::{convert_card_units, unit_scale_and_suffix};
pub use sidebar::SidebarState;

use crate::{GalleryScreen, InspectorMode};
use egui::{pos2, vec2, Align, Color32, CornerRadius, Layout, Pos2, Rect, Ui};
use egui_macview::{
    theme::MacTheme, CASpring, MacCanvasFooter, MacCanvasFooterAction, MacScrollView,
    MacSearchField, MacSortDirection, MacSymbol, MacToastManager, NSUndoManager,
    CANVAS_FOOTER_HEIGHT,
};
use std::cell::RefCell;
use std::collections::BTreeSet;

/// Dynamic properties of the interactive Canvas Vector Card.
#[derive(Clone, PartialEq)]
pub struct CardModel {
    pub width: String,
    pub height: String,
    pub radius: String,
    pub rotation_deg: f32,
    pub scale_pct: f32,
    pub skew_deg: f32,
    pub opacity: f32,
    pub layer_name: String,
    pub shadow_enabled: bool,
    pub shadow_blur: f64,
    pub fill_color: Color32,
    pub alpha_color: Color32,
    pub blend_mode_idx: usize,
    pub constrain_proportions: bool,
    pub alignment_idx: usize,
}

impl CardModel {
    /// Computes the 2x2 affine transform matrix `(a, b, c, d)` for `rot_deg` and `scale_factor`.
    pub fn affine_matrix(&self, rot_deg: f32, scale_factor: f32) -> (f32, f32, f32, f32) {
        let theta = rot_deg.to_radians();
        let skew_val = self.skew_deg.to_radians().tan();
        (
            scale_factor * theta.cos(),
            -scale_factor * theta.sin() + skew_val,
            scale_factor * theta.sin(),
            scale_factor * theta.cos(),
        )
    }
}

impl Default for CardModel {
    fn default() -> Self {
        let default_theme = MacTheme::default();
        Self {
            width: "260.0".to_string(),
            height: "150.0".to_string(),
            radius: "10.0".to_string(),
            rotation_deg: 12.0,
            scale_pct: 100.0,
            skew_deg: 0.0,
            opacity: 100.0,
            layer_name: "Hero Vector Card".to_string(),
            shadow_enabled: true,
            shadow_blur: 8.0,
            fill_color: default_theme.control_accent(),
            alpha_color: default_theme.control_accent().gamma_multiply(0.7),
            blend_mode_idx: 0,
            constrain_proportions: true,
            alignment_idx: 1,
        }
    }
}

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
    pub token_draft: String,
    pub publish_progress: f32,
    pub publish_indeterminate: bool,
    pub is_publishing: bool,
    // State 5 (v0.5 AVKit Media & Audio Timeline Studio)
    pub media_playing: bool,
    pub media_loop: bool,
    pub media_in_point: f64,
    pub media_out_point: f64,
    pub media_playhead: f64,
    pub media_last_time: Option<f64>,
    pub media_waveform: Vec<f32>,
    pub media_gain: f32,
    pub media_discrete_vu: bool,
    pub media_threshold_vu: bool,
    pub audio_level_l: f32,
    pub audio_level_r: f32,
    pub audio_peak_l: f32,
    pub audio_peak_r: f32,
}

impl Default for CenterState {
    fn default() -> Self {
        let waveform = (0..64)
            .map(|i| {
                let phase = i as f32 * 0.22;
                (phase.sin().abs() * 0.65 + (phase * 2.3).cos().abs() * 0.35).clamp(0.12, 0.98)
            })
            .collect();
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
            wcag_bg: Color32::from_rgb(0, 122, 255),
            row_container_width: 420.0,
            cascade_start_time: None,
            card_menu_open: false,
            card_menu_pos: Pos2::ZERO,
            card_menu_selected: None,
            showcase_control_size_idx: 3,
            showcase_controls_enabled: true,
            showcase_text: "AppKit v0.5.0".to_string(),
            showcase_num_text: "128.0".to_string(),
            showcase_step_val: 42.0,
            showcase_popup_idx: 0,
            showcase_color: Color32::from_rgb(0, 122, 255),
            active_path_idx: 3,
            table_sort_col: 0,
            table_sort_dir: MacSortDirection::Ascending,
            selected_table_rows: BTreeSet::from([0]),
            asset_tokens: [
                vec!["ProRes 422".into(), "4K UHD".into(), "Approved".into()],
                vec!["ProRes 4444".into(), "Alpha".into(), "VFX".into()],
                vec!["PCM 24-bit".into(), "48kHz".into(), "Stereo".into()],
                vec!["HEVC".into(), "HDR10".into(), "Proxy".into()],
                vec!["Metal Shader".into(), "LUT".into(), "Rec.709".into()],
                vec!["QuickTime".into(), "Master".into(), "Deliverable".into()],
            ],
            token_draft: String::new(),
            publish_progress: 0.68,
            publish_indeterminate: false,
            is_publishing: false,
            media_playing: false,
            media_loop: false,
            media_in_point: 15.0,
            media_out_point: 92.0,
            media_playhead: 36.0,
            media_last_time: None,
            media_waveform: waveform,
            media_gain: 1.0,
            media_discrete_vu: true,
            media_threshold_vu: true,
            audio_level_l: 7.4,
            audio_level_r: 8.6,
            audio_peak_l: 8.2,
            audio_peak_r: 9.1,
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
}
