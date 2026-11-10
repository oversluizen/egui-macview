//! Modular state and sidebar navigation for `macview_gallery` (v0.1 – v0.5 Studio Showcase).

pub mod audio_media;
pub mod center_state;
pub mod chrome;
pub mod finder_data;
pub mod inspector;
pub mod inspector_nib;
pub mod sidebar;
pub mod snippet;
pub mod state1_canvas;
pub mod state2_symbols_layout;
pub mod state3_command_state;
pub mod state4_finder_browser;
pub mod state4_publish;
pub mod state5_avkit_media;
pub mod state5_meters;
pub mod state6_advanced_controls;
pub mod types;
pub mod video_scene;

pub use center_state::CenterState;
pub use chrome::detect_frameless_resize_dir;
pub use finder_data::FinderCustomEntry;
pub use inspector::{convert_card_units, unit_scale_and_suffix};
pub use sidebar::SidebarState;
pub use types::{
    format_tracked_small_caps, setup_system_or_custom_fonts, ExpandedProTab, GalleryScreen,
    InspectorMode, SYMBOLS_META,
};

use egui::Color32;
use egui_macview::theme::MacTheme;

/// Dynamic properties of the interactive Canvas Vector Card.
#[derive(Debug, Clone, PartialEq)]
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
