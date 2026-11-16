//! Right Inspector (`DefaultNib`, `CompactHud`, `ExpandedPro`) rendered 100% from `Right_Inspector_Default.xib` (`REQ-013-02P`).

use crate::{CardModel, ExpandedProTab, GalleryScreen, InspectorMode};
use egui::{Align, CornerRadius, Layout, Ui};
use egui_macview::{theme::MacTheme, MacScrollView, MacToastManager};
use std::cell::RefCell;

pub struct InspectorState {
    pub popup_selected: usize,
    pub unit_segmented_idx: usize,
    pub snapping_enabled: bool,
    pub expanded_tab: ExpandedProTab,
    pub nib_scene: egui_macview::scene::MacSceneIR,
    pub nib_store: super::nib_showcase_builders::ShowcaseSceneDataStore,
}

impl Default for InspectorState {
    fn default() -> Self {
        let nib_scene = super::nib_showcase_builders::compile_right_inspector_scene();
        let mut nib_store = super::nib_showcase_builders::ShowcaseSceneDataStore::default();
        nib_store.seed_from_scene(&nib_scene);
        Self {
            popup_selected: 0,
            unit_segmented_idx: 0,
            snapping_enabled: true,
            expanded_tab: ExpandedProTab::Attributes,
            nib_scene,
            nib_store,
        }
    }
}

pub use super::inspector_nib::{convert_card_units, unit_scale_and_suffix};
use super::snippet::{generate_rust_snippet, SnippetContext};

impl InspectorState {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        card_cell: &RefCell<CardModel>,
        (_mode, active_screen): (&mut InspectorMode, GalleryScreen),
        snippet_ctx: SnippetContext,
        _toasts: &RefCell<MacToastManager>,
    ) {
        let card = card_cell.borrow();
        let (w_lbl, h_lbl, r_lbl) = super::types::unit_row_labels(self.unit_segmented_idx);
        let (a, b, c, d) = card.affine_matrix(card.rotation_deg, card.scale_pct / 100.0);
        let tab_body = match self.expanded_tab {
            ExpandedProTab::Attributes => format!(
                "Transform Matrix ({w_lbl}/{h_lbl}/{r_lbl}, blend #{}):\n[{a:+.3}, {b:+.3}; {c:+.3}, {d:+.3}]\nSize: {} x {} | Radius: {} | Align: #{}",
                card.blend_mode_idx, card.width, card.height, card.radius, card.alignment_idx
            ),
            ExpandedProTab::Metrics => format!(
                "Visual Effects & Stroke Metrics:\nStroke: {:.1}pt ({:?})\nShadow Blur: {:.1}pt (Enabled: {})\nOpacity: {:.0}% | Constrain: {}",
                card.stroke_width, card.stroke_color, card.shadow_blur, card.shadow_enabled, card.opacity, card.constrain_proportions
            ),
            ExpandedProTab::Matrix => format!(
                "CGAffineTransform 3x3 Homogeneous (Preset #{}):\n| {a:+.4}  {b:+.4}  0.0000 |\n| {c:+.4}  {d:+.4}  0.0000 |\n|  0.0000   0.0000  1.0000 |\ndet(A) = {:.4}",
                self.popup_selected,
                a * d - b * c
            ),
            ExpandedProTab::CodeSnippet => generate_rust_snippet(active_screen, &card, &snippet_ctx),
        };
        let hdr_b = self.nib_store.resolve_or_create_binding("inspectorMatrixHeader");
        self.nib_store.strings.insert(hdr_b, format!("Matrix [{a:+.2}, {b:+.2}; {c:+.2}, {d:+.2}] ({:?})", self.expanded_tab));
        let body_b = self.nib_store.resolve_or_create_binding("inspectorSnippetBody");
        self.nib_store.strings.insert(body_b, tab_body);
        drop(card);

        let avail_rect = ui.available_rect_before_wrap();
        ui.painter_at(avail_rect).rect_filled(avail_rect, CornerRadius::ZERO, theme.window_background());

        MacScrollView::new("gallery_inspector_scroll")
            .theme(theme)
            .vertical(true)
            .horizontal(true)
            .max_height(avail_rect.height())
            .max_width(avail_rect.width())
            .show(ui, |scroll_ui| {
                scroll_ui.with_layout(Layout::top_down(Align::Min), |s_ui| {
                    s_ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    self.nib_scene.show(s_ui, theme, &mut self.nib_store, egui_macview::scene::LayoutPolicy::ExactAppleNib, 0, "");
                });
            });
    }
}
