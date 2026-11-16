//! Live two-way data store synchronization between `.xib`/`.nib` scenes and `GalleryApp` (`REQ-013-02Q`).

use super::{nib_showcase_builders::ShowcaseSceneDataStore, nib_window_studio::NibViewportSync, CenterState, InspectorMode};
use egui_macview::{
    controls::MacOriginAnchor,
    scene::{BindingId, MacSceneDataSource},
    theme::{MacAppearance, WindowControlsStyle},
    MacToastManager,
};
use std::cell::RefCell;

impl CenterState {
    pub(super) fn pull_store_live_from_gallery(&self, store: &mut ShowcaseSceneDataStore, sync: &NibViewportSync<'_>, sync_strings: bool) {
        let card = sync.card_cell.borrow();
        let (appearance, chrome, insp, sidebar, inspector) =
            (*sync.appearance, *sync.window_controls_style, *sync.inspector_mode, &*sync.sidebar, &*sync.inspector);
        let paths: Vec<(String, BindingId)> = store.path_to_id.iter().map(|(k, &v)| (k.clone(), v)).collect();
        let set_num = |st: &mut ShowcaseSceneDataStore, b: BindingId, v: f64| {
            st.floats.insert(b, v);
            st.strings.insert(b, format!("{v:.0}"));
        };
        for (path, bid) in paths {
            match path.strip_prefix("values.").unwrap_or(path.as_str()) {
                "layerName" if sync_strings => {
                    store.strings.insert(bid, card.layer_name.clone());
                }
                "blendMode" => {
                    store.ints.insert(bid, card.blend_mode_idx as i64);
                }
                "cardWidth" => set_num(store, bid, card.width.parse::<f64>().unwrap_or(260.0)),
                "cardHeight" | "canvasHeight" => set_num(store, bid, card.height.parse::<f64>().unwrap_or(150.0)),
                "canvasWidth" => set_num(store, bid, self.nib_width.parse::<f64>().unwrap_or(1920.0)),
                "nibWidthExpr" if sync_strings => {
                    store.strings.insert(bid, self.nib_width.clone());
                }
                "scalePercent" => set_num(store, bid, card.scale_pct as f64),
                "originAnchor" => {
                    store.ints.insert(bid, self.origin_anchor as i64);
                }
                "originAnchorLabel" => {
                    store.strings.insert(bid, format!("{:?}", self.origin_anchor));
                }
                "constrainProportions" => {
                    store.bools.insert(bid, card.constrain_proportions);
                }
                "shadowEnabled" => {
                    store.bools.insert(bid, card.shadow_enabled);
                }
                "shadowBlur" => {
                    store.floats.insert(bid, card.shadow_blur);
                }
                "rotationDeg" => set_num(store, bid, card.rotation_deg as f64),
                "opacityPct" => {
                    store.floats.insert(bid, card.opacity as f64);
                }
                "cornerRadius" => set_num(store, bid, card.radius.parse::<f64>().unwrap_or(10.0)),
                "rulerUnitIndex" | "nibUnitIdx" => {
                    store.ints.insert(bid, self.nib_unit_idx as i64);
                }
                "snapToPixelGrid" | "snapToGrid" => {
                    store.bools.insert(bid, inspector.snapping_enabled);
                }
                "gridSubdivision" => {
                    store.floats.insert(bid, self.nib_axis_count.max(1) as f64);
                }
                "nibAxesLabel" => {
                    store.strings.insert(bid, format!("Secondary Axes ({})", self.nib_axis_count));
                }
                "opticalAlignment" | "alignmentMode" => {
                    store.ints.insert(bid, card.alignment_idx as i64);
                }
                "themeVariant" => {
                    store.ints.insert(
                        bid,
                        match appearance {
                            MacAppearance::Aqua => 0,
                            MacAppearance::Anthracite => 1,
                            MacAppearance::DarkAqua => 2,
                        },
                    );
                }
                "windowChrome" | "windowStyle" => {
                    store.ints.insert(
                        bid,
                        match chrome {
                            WindowControlsStyle::AppleTrafficLights => 0,
                            WindowControlsStyle::WindowsCaptionButtons => 1,
                            WindowControlsStyle::NativeHost => 2,
                        },
                    );
                }
                "inspectorPreset" | "inspectorMode" => {
                    store.ints.insert(bid, insp as i64);
                }
                "expandedProTab" => {
                    store.ints.insert(bid, inspector.expanded_tab as i64);
                }
                "visCanvas" => {
                    store.bools.insert(bid, sidebar.vis_canvas);
                }
                "lockCanvas" => {
                    store.bools.insert(bid, sidebar.lock_canvas);
                }
                "glyphScaleIdx" => {
                    store.ints.insert(bid, self.symbol_preview_size_idx as i64);
                }
                "symbolTint" => {
                    store.colors.insert(bid, self.symbol_tint.to_srgba_unmultiplied());
                }
                "fixedClipMode" => {
                    store.ints.insert(bid, self.fixed_seg_idx as i64);
                }
                "rowWidthBudget" => {
                    store.floats.insert(bid, self.row_container_width as f64);
                }
                "matrixControlSize" => {
                    store.ints.insert(bid, self.showcase_control_size_idx as i64);
                }
                "matrixEnabled" => {
                    store.bools.insert(bid, self.showcase_controls_enabled);
                }
                "wcagFg" => {
                    store.colors.insert(bid, self.wcag_fg.to_srgba_unmultiplied());
                }
                "wcagBg" => {
                    store.colors.insert(bid, self.wcag_bg.to_srgba_unmultiplied());
                }
                "mediaGain" => set_num(store, bid, self.media_gain as f64),
                "mediaPan" => {
                    store.floats.insert(bid, self.media_pan as f64);
                }
                "mediaZoom" => {
                    store.floats.insert(bid, self.media_zoom as f64);
                }
                "mediaPlayhead" => {
                    store.floats.insert(bid, self.media_playhead);
                }
                "mediaExposure" => {
                    store.floats.insert(bid, self.media_exposure as f64);
                }
                "mediaLoop" | "loopPlayback" => {
                    store.bools.insert(bid, self.media_loop);
                }
                "discreteVu" => {
                    store.bools.insert(bid, self.media_discrete_vu);
                }
                "thresholdVu" => {
                    store.bools.insert(bid, self.media_threshold_vu);
                }
                "finderUrl" | "finderUrlInput" if sync_strings => {
                    store.strings.insert(bid, self.finder_url_input.clone());
                }
                "publishProgress" => {
                    store.floats.insert(bid, (self.publish_progress * 100.0) as f64);
                }
                "finderSearch" if sync_strings => {
                    store.strings.insert(bid, self.search_query.clone());
                }
                "publishIndeterminate" => {
                    store.bools.insert(bid, self.publish_indeterminate);
                }
                "surfaceContrast" => {
                    store.floats.insert(bid, (self.media_contrast * 50.0).clamp(0.0, 100.0) as f64);
                }
                "wcagHighContrast" | "nibCheckRegular" => {
                    store.bools.insert(bid, self.nib_check_regular);
                }
                "showAlignmentGuides" | "nibCheckSmall" => {
                    store.bools.insert(bid, self.nib_check_small);
                }
                "nibCheckMini" => {
                    store.bools.insert(bid, self.nib_check_mini_mixed);
                }
                "nibSmallExpr" if sync_strings => {
                    store.strings.insert(bid, self.nib_small_text.clone());
                }
                "comboText" if sync_strings => {
                    store.strings.insert(bid, self.advanced_combo_text.clone());
                }
                "showBaselineRulers" | "nibInspectorEnabled" => {
                    store.bools.insert(bid, self.nib_inspector_enabled);
                }
                "advancedGroupExpanded" => {
                    store.bools.insert(bid, self.advanced_disclosure_open);
                }
                "advancedButtonOpen" => {
                    store.bools.insert(bid, self.advanced_button_open);
                }
                "guideProfile" | "nibPresetIdx" => {
                    store.ints.insert(bid, self.nib_preset_idx as i64);
                }
                "finderTagColor" => {
                    store.ints.insert(bid, self.tag_color.map(|i| i as i64).unwrap_or(-1));
                }
                other => self.pull_extended_store_bindings(store, &card, other, bid, sync_strings),
            }
        }
    }

    pub(super) fn flush_store_dirty_to_gallery(
        &mut self,
        store: &mut ShowcaseSceneDataStore,
        sync: &mut NibViewportSync<'_>,
        toasts: &RefCell<MacToastManager>,
    ) {
        let NibViewportSync {
            card_cell,
            appearance,
            window_controls_style: chrome,
            inspector_mode: insp,
            active_screen,
            sidebar,
            inspector,
        } = sync;
        let dirty: Vec<String> = std::mem::take(&mut store.dirty_paths);
        for path in dirty {
            let norm = path.strip_prefix("values.").unwrap_or(path.as_str());
            let Some(&bid) = store.path_to_id.get(&path).or_else(|| store.path_to_id.get(norm)) else {
                continue;
            };
            match norm {
                "layerName" => {
                    card_cell.borrow_mut().layer_name = store.get_str(bid).to_string();
                }
                "blendMode" => {
                    card_cell.borrow_mut().blend_mode_idx = store.get_i64(bid).clamp(0, 3) as usize;
                }
                "cardWidth" => {
                    let mut c = card_cell.borrow_mut();
                    let prev_w = c.width.clone();
                    c.width = format!("{:.1}", store.get_f64(bid));
                    super::inspector_nib::sync_aspect_width_changed(&mut c, &prev_w, inspector.unit_segmented_idx);
                }
                "cardHeight" | "canvasHeight" => {
                    let mut c = card_cell.borrow_mut();
                    let prev_h = c.height.clone();
                    c.height = format!("{:.1}", store.get_f64(bid));
                    super::inspector_nib::sync_aspect_height_changed(&mut c, &prev_h, inspector.unit_segmented_idx);
                }
                "canvasWidth" => {
                    self.nib_width = format!("{:.0}", store.get_f64(bid));
                }
                "nibWidthExpr" => {
                    self.nib_width = store.get_str(bid).to_string();
                }
                "scalePercent" => {
                    card_cell.borrow_mut().scale_pct = store.get_f64(bid) as f32;
                }
                "originAnchor" => {
                    self.origin_anchor = MacOriginAnchor::ALL[(store.get_i64(bid).clamp(0, 8)) as usize];
                }
                "constrainProportions" => {
                    card_cell.borrow_mut().constrain_proportions = store.get_bool(bid);
                }
                "shadowEnabled" => {
                    card_cell.borrow_mut().shadow_enabled = store.get_bool(bid);
                }
                "shadowBlur" => {
                    card_cell.borrow_mut().shadow_blur = store.get_f64(bid);
                }
                "rotationDeg" => {
                    card_cell.borrow_mut().rotation_deg = store.get_f64(bid) as f32;
                }
                "opacityPct" => {
                    card_cell.borrow_mut().opacity = store.get_f64(bid) as f32;
                }
                "cornerRadius" => {
                    card_cell.borrow_mut().radius = format!("{:.1}", store.get_f64(bid));
                }
                "rulerUnitIndex" | "nibUnitIdx" => {
                    let (new_u, prev_u) = (store.get_i64(bid).clamp(0, 3) as usize, inspector.unit_segmented_idx);
                    (inspector.unit_segmented_idx, self.nib_unit_idx) = (new_u.min(2), new_u);
                    super::inspector_nib::convert_card_units(&mut card_cell.borrow_mut(), prev_u, new_u.min(2));
                }
                "snapToPixelGrid" | "snapToGrid" => {
                    inspector.snapping_enabled = store.get_bool(bid);
                }
                "gridSubdivision" => {
                    self.nib_axis_count = store.get_f64(bid).round().max(1.0) as usize;
                }
                "opticalAlignment" | "alignmentMode" => {
                    card_cell.borrow_mut().alignment_idx = store.get_i64(bid).clamp(0, 3) as usize;
                }
                "themeVariant" => {
                    **appearance = [MacAppearance::Aqua, MacAppearance::Anthracite, MacAppearance::DarkAqua, MacAppearance::DarkAqua]
                        [store.get_i64(bid).clamp(0, 3) as usize];
                }
                "windowChrome" | "windowStyle" => {
                    **chrome = [
                        WindowControlsStyle::AppleTrafficLights,
                        WindowControlsStyle::WindowsCaptionButtons,
                        WindowControlsStyle::NativeHost,
                    ][store.get_i64(bid).clamp(0, 2) as usize];
                }
                "inspectorPreset" | "inspectorMode" => {
                    **insp = [InspectorMode::DefaultNib, InspectorMode::CompactHud, InspectorMode::ExpandedPro]
                        [store.get_i64(bid).clamp(0, 2) as usize];
                }
                "expandedProTab" => {
                    inspector.expanded_tab = [
                        crate::ExpandedProTab::Attributes,
                        crate::ExpandedProTab::Metrics,
                        crate::ExpandedProTab::Matrix,
                        crate::ExpandedProTab::CodeSnippet,
                    ][store.get_i64(bid).clamp(0, 3) as usize];
                }
                "visCanvas" => {
                    sidebar.vis_canvas = store.get_bool(bid);
                }
                "lockCanvas" => {
                    sidebar.lock_canvas = store.get_bool(bid);
                }
                "glyphScaleIdx" => {
                    self.symbol_preview_size_idx = store.get_i64(bid).clamp(0, 2) as usize;
                }
                "symbolTint" => {
                    if let Some(col) = store.get_color(bid) {
                        self.symbol_tint = col;
                    }
                }
                "fixedClipMode" => {
                    self.fixed_seg_idx = store.get_i64(bid).clamp(0, 2) as usize;
                }
                "rowWidthBudget" => {
                    self.row_container_width = store.get_f64(bid).clamp(240.0, 640.0) as f32;
                }
                "matrixControlSize" => {
                    self.showcase_control_size_idx = store.get_i64(bid).clamp(0, 2) as usize;
                }
                "matrixEnabled" => {
                    self.showcase_controls_enabled = store.get_bool(bid);
                }
                "wcagFg" => {
                    if let Some(col) = store.get_color(bid) {
                        self.wcag_fg = col;
                    }
                }
                "wcagBg" => {
                    if let Some(col) = store.get_color(bid) {
                        self.wcag_bg = col;
                    }
                }
                "mediaGain" => {
                    self.media_gain = store.get_f64(bid) as f32;
                }
                "mediaPan" => {
                    self.media_pan = store.get_f64(bid) as f32;
                }
                "mediaZoom" => {
                    self.media_zoom = store.get_f64(bid) as f32;
                }
                "mediaPlayhead" => {
                    self.media_playhead = store.get_f64(bid).clamp(self.media_in_point, self.media_out_point);
                }
                "mediaExposure" => {
                    self.media_exposure = store.get_f64(bid) as f32;
                }
                "mediaLoop" | "loopPlayback" => {
                    self.media_loop = store.get_bool(bid);
                }
                "discreteVu" => {
                    self.media_discrete_vu = store.get_bool(bid);
                }
                "thresholdVu" => {
                    self.media_threshold_vu = store.get_bool(bid);
                }
                "finderUrl" | "finderUrlInput" => {
                    self.finder_url_input = store.get_str(bid).to_string();
                }
                "publishProgress" => {
                    self.publish_progress = (store.get_f64(bid) as f32 / 100.0).clamp(0.0, 1.0);
                }
                "finderSearch" => {
                    self.search_query = store.get_str(bid).to_string();
                }
                "publishIndeterminate" => {
                    self.publish_indeterminate = store.get_bool(bid);
                }
                "surfaceContrast" => {
                    self.media_contrast = (store.get_f64(bid) as f32 / 50.0).clamp(0.1, 2.0);
                }
                "wcagHighContrast" | "nibCheckRegular" => {
                    self.nib_check_regular = store.get_bool(bid);
                }
                "showAlignmentGuides" | "nibCheckSmall" => {
                    self.nib_check_small = store.get_bool(bid);
                }
                "nibCheckMini" => {
                    self.nib_check_mini_mixed = store.get_bool(bid);
                }
                "nibSmallExpr" => {
                    self.nib_small_text = store.get_str(bid).to_string();
                }
                "comboText" => {
                    self.advanced_combo_text = store.get_str(bid).to_string();
                }
                "showBaselineRulers" | "nibInspectorEnabled" => {
                    self.nib_inspector_enabled = store.get_bool(bid);
                }
                "advancedGroupExpanded" => {
                    self.advanced_disclosure_open = store.get_bool(bid);
                }
                "advancedButtonOpen" => {
                    self.advanced_button_open = store.get_bool(bid);
                }
                "guideProfile" | "nibPresetIdx" => {
                    self.nib_preset_idx = store.get_i64(bid).clamp(0, 2) as usize;
                }
                "finderTagColor" => {
                    let v = store.get_i64(bid);
                    self.tag_color = (v >= 0).then_some(v as usize);
                }
                other => self.flush_extended_store_binding(store, card_cell, other, bid),
            }
        }
        self.dispatch_pending_store_actions(store, card_cell, active_screen, sidebar, inspector, toasts);
    }
}
