//! Separate OS Viewport Window & Live Two-Way State Bridge for `.nib` / `.xib` Showcases (`REQ-013-02K`, `REQ-013-02P`).

use super::{
    nib_showcase_builders::{
        compile_embedded_showcase_by_name, ensure_showcase_nib_files, resolve_showcase_nib_dir, ShowcaseSceneDataStore, SHOWCASE_NIB_FILES,
    },
    CardModel, CenterState, GalleryScreen, InspectorMode, SidebarState,
};
use egui::{pos2, vec2, Align, CornerRadius, FontId, Layout, Rect, Stroke, StrokeKind, Ui, ViewportBuilder, ViewportId};
use egui_macview::{
    scene::{
        nib_varint::build_real_varint_nib_archive, CompiledConstraintSchedule, CustomClassMapper, LayoutPolicy, MacSceneIR,
        NibArchiveFrontend, NodeId, SceneExporter, SwiftUiFrontend, XibXmlFrontend,
    },
    theme::{MacAppearance, MacTheme, WindowControlsStyle},
    MacBox, MacButtonCell, MacScrollView, MacSearchField, MacSegmentedControl, MacToastManager,
};
use std::{cell::RefCell, path::Path};

/// Separate OS viewport state for rendering `.nib` / `.xib` files with live two-way `GalleryApp` bindings.
#[derive(Clone)]
pub struct NibShowcaseWindowState {
    pub open: bool,
    pub active_file_name: String,
    pub scene: MacSceneIR,
    pub swiftui_source: String,
    pub policy_idx: usize,
    pub scene_search_query: String,
    pub roundtrip_verified: bool,
    pub node_count: usize,
    pub section_count: usize,
    pub paired_label_count: usize,
    pub viewport_editing_text: bool,
    pub store: ShowcaseSceneDataStore,
}

impl Default for NibShowcaseWindowState {
    fn default() -> Self {
        static INITIAL_STUDIO: std::sync::OnceLock<NibShowcaseWindowState> = std::sync::OnceLock::new();
        let mut state = INITIAL_STUDIO
            .get_or_init(|| {
                let showcase_dir = resolve_showcase_nib_dir();
                ensure_showcase_nib_files(&showcase_dir);
                let mapper = CustomClassMapper::default_studio_mappings();
                let initial_bytes = build_real_varint_nib_archive();
                let scene = NibArchiveFrontend::from_bytes(&initial_bytes, &mapper).unwrap_or_else(|_| {
                    MacSceneIR::from_nodes(NodeId(1), [440.0, 310.0], Vec::new(), CompiledConstraintSchedule::default())
                });
                let swiftui_source = SceneExporter::to_swiftui_source(&scene);
                let roundtrip_verified = SwiftUiFrontend::parse(&swiftui_source).is_ok();
                let mut store = ShowcaseSceneDataStore::default();
                store.seed_from_scene(&scene);
                let mut s = Self {
                    open: false,
                    active_file_name: SHOWCASE_NIB_FILES[0].to_string(),
                    node_count: scene.nodes().len(),
                    section_count: scene.semantic_sections().len(),
                    paired_label_count: scene.labeled_pairs().len(),
                    scene,
                    swiftui_source,
                    policy_idx: 0,
                    scene_search_query: String::new(),
                    roundtrip_verified,
                    viewport_editing_text: false,
                    store,
                };
                let default_nib = showcase_dir.join(SHOWCASE_NIB_FILES[0]);
                if default_nib.exists() {
                    let _ = s.load_nib_or_xib_file(&default_nib, &RefCell::new(MacToastManager::new()));
                }
                s
            })
            .clone();
        state.open = std::env::var("MACVIEW_OPEN_NIB_SHOWCASE").as_deref() == Ok("1");
        state
    }
}

impl NibShowcaseWindowState {
    pub fn set_path_f64(&mut self, path: &str, val: f64) {
        self.store.set_path_f64(path, val);
    }
    pub fn set_path_i64(&mut self, path: &str, val: i64) {
        self.store.set_path_i64(path, val);
    }

    /// Compiles a `.nib` or `.xib` file from disk into `MacSceneIR`, generates SwiftUI code, and opens the separate OS window.
    pub fn load_nib_or_xib_file(&mut self, path: &Path, toasts: &RefCell<MacToastManager>) -> bool {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Scene.nib".to_string());
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        let mapper = CustomClassMapper::default_studio_mappings();
        let compiled = if ext == "xib" {
            match std::fs::read_to_string(path) {
                Ok(xml) => XibXmlFrontend::from_xml_str(&xml),
                Err(err) => {
                    if let Some(scene) = compile_embedded_showcase_by_name(&name) {
                        Ok(scene)
                    } else {
                        toasts.borrow_mut().error(format!("Failed to read {name}: {err}"));
                        return false;
                    }
                }
            }
        } else {
            match std::fs::read(path) {
                Ok(bytes) => NibArchiveFrontend::from_bytes(&bytes, &mapper),
                Err(err) => {
                    if let Some(scene) = compile_embedded_showcase_by_name(&name) {
                        Ok(scene)
                    } else {
                        toasts.borrow_mut().error(format!("Failed to read {name}: {err}"));
                        return false;
                    }
                }
            }
        };
        match compiled {
            Ok(scene) => {
                let swiftui = SceneExporter::to_swiftui_source(&scene);
                self.roundtrip_verified = SwiftUiFrontend::parse(&swiftui).is_ok();
                (self.node_count, self.section_count, self.paired_label_count) =
                    (scene.nodes().len(), scene.semantic_sections().len(), scene.labeled_pairs().len());
                self.store.seed_from_scene(&scene);
                self.store.last_action = format!("Loaded {name} ({} nodes)", self.node_count);
                (self.scene, self.swiftui_source, self.active_file_name, self.open) = (scene, swiftui, name.clone(), true);
                toasts.borrow_mut().success(format!("Opened NIB Window: {name}"));
                true
            }
            Err(err) => {
                toasts.borrow_mut().error(format!("NIB compile error in {name}: {}", err.0));
                false
            }
        }
    }

    fn render_window_contents(&mut self, ui: &mut Ui, theme: &MacTheme, toasts: &RefCell<MacToastManager>) {
        ui.horizontal(|bar| {
            let mut next_pol = self.policy_idx;
            MacSegmentedControl::new(&["Mode A: ExactAppleNib", "Mode B: AdaptiveSemanticReflow"], self.policy_idx)
                .theme(theme)
                .height(22.0)
                .show(bar, |idx| next_pol = idx);
            self.policy_idx = next_pol;
            bar.add_space(6.0);
            MacSearchField::new(&mut self.scene_search_query).theme(theme).placeholder("Filter NIB controls...").width(210.0).show(bar);
            if bar.add(MacButtonCell::new("Copy SwiftUI").theme(theme).selected(true)).clicked() {
                bar.ctx().copy_text(self.swiftui_source.clone());
                toasts.borrow_mut().success("Copied transpiled SwiftUI code to clipboard");
            }
            if bar.add(MacButtonCell::new("Close Window").theme(theme)).clicked() {
                self.open = false;
            }
        });
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "Active NIB: {}  |  Nodes: {}  |  Paired Labels: {}  |  SwiftUI Round-Trip: {}  |  Live Sync: ACTIVE ({})",
                self.active_file_name,
                self.node_count,
                self.paired_label_count,
                if self.roundtrip_verified { "PASS" } else { "ERR" },
                self.store.last_action
            ))
            .size(11.0),
        );
        ui.add_space(6.0);

        let (top_y, max_r) = (ui.cursor().min.y, ui.max_rect());
        let (total_w, pane_h) = (max_r.width().max(680.0), (max_r.bottom() - top_y).max(320.0));
        let left_w = (total_w * 0.54).max(380.0);
        let right_w = (total_w - left_w - 10.0).max(260.0);
        let left_rect = Rect::from_min_size(pos2(max_r.left(), top_y), vec2(left_w, pane_h));
        let right_rect = Rect::from_min_size(pos2(max_r.left() + left_w + 10.0, top_y), vec2(right_w, pane_h));
        ui.painter_at(left_rect).rect(
            left_rect,
            CornerRadius::same(6),
            theme.window_background(),
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );
        let mut scene_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("nib_studio_live_scene_pane")
                .max_rect(left_rect.shrink(6.0))
                .layout(Layout::top_down(Align::Min)),
        );
        let policy = if self.policy_idx == 0 { LayoutPolicy::ExactAppleNib } else { LayoutPolicy::AdaptiveSemanticReflow };
        self.scene.show(&mut scene_ui, theme, &mut self.store, policy, 0, &self.scene_search_query);
        let mut code_ui = ui.new_child(
            egui::UiBuilder::new().id_salt("nib_studio_swiftui_code_pane").max_rect(right_rect).layout(Layout::top_down(Align::Min)),
        );
        MacBox::new("Generated SwiftUI Source (SceneExporter::to_swiftui_source)").theme(theme).fixed_rect(right_rect).show(
            &mut code_ui,
            |box_ui| {
                MacScrollView::new("nib_studio_swiftui_scroll")
                    .theme(theme)
                    .vertical(true)
                    .horizontal(true)
                    .max_height((pane_h - 34.0).max(180.0))
                    .max_width((right_w - 20.0).max(220.0))
                    .show(box_ui, |scroll_ui| {
                        scroll_ui.label(egui::RichText::new(&self.swiftui_source).font(FontId::monospace(11.5)));
                    });
            },
        );
    }
}

pub struct NibViewportSync<'a> {
    pub card_cell: &'a RefCell<CardModel>,
    pub appearance: &'a mut MacAppearance,
    pub window_controls_style: &'a mut WindowControlsStyle,
    pub inspector_mode: &'a mut InspectorMode,
    pub active_screen: &'a mut GalleryScreen,
    pub sidebar: &'a mut SidebarState,
    pub inspector: &'a mut super::inspector::InspectorState,
}

impl CenterState {
    pub fn sync_all_nib_stores(&mut self, sync: &mut NibViewportSync<'_>, toasts: &RefCell<MacToastManager>, sync_strings: bool) {
        let mut screen_stores: Vec<ShowcaseSceneDataStore> = self.screen_nib_scenes.iter_mut().map(|(_, s)| std::mem::take(s)).collect();
        for store in &mut screen_stores {
            self.flush_store_dirty_to_gallery(store, sync, toasts);
        }
        let mut insp_store = std::mem::take(&mut sync.inspector.nib_store);
        self.flush_store_dirty_to_gallery(&mut insp_store, sync, toasts);
        let mut studio_store = std::mem::take(&mut self.nib_studio.store);
        self.flush_store_dirty_to_gallery(&mut studio_store, sync, toasts);
        for store in &mut screen_stores {
            self.pull_store_live_from_gallery(store, sync, sync_strings);
        }
        self.pull_store_live_from_gallery(&mut insp_store, sync, sync_strings);
        self.pull_store_live_from_gallery(&mut studio_store, sync, sync_strings && !self.nib_studio.viewport_editing_text);
        for ((_, dst), src) in self.screen_nib_scenes.iter_mut().zip(screen_stores) {
            *dst = src;
        }
        (sync.inspector.nib_store, self.nib_studio.store) = (insp_store, studio_store);
    }

    pub fn sync_and_render_nib_viewport(
        &mut self,
        ctx: &egui::Context,
        theme: &MacTheme,
        mut sync: NibViewportSync<'_>,
        toasts: &RefCell<MacToastManager>,
    ) {
        let sync_strings = !ctx.egui_wants_keyboard_input();
        self.sync_all_nib_stores(&mut sync, toasts, sync_strings);
        if !self.nib_studio.open {
            self.nib_studio.viewport_editing_text = false;
            return;
        }
        let win_title = format!("macOS NIB Window & SwiftUI Studio — {}", self.nib_studio.active_file_name);
        if cfg!(target_arch = "wasm32") {
            let mut open_flag = self.nib_studio.open;
            egui::Window::new(&win_title)
                .id(egui::Id::unique("wasm_nib_studio_floating_sheet"))
                .open(&mut open_flag)
                .default_size([760.0, 460.0])
                .resizable(true)
                .show(ctx, |win_ui| {
                    self.nib_studio.render_window_contents(win_ui, theme, toasts);
                });
            if !open_flag {
                self.nib_studio.open = false;
            }
        } else {
            let vp_id = ViewportId::from_hash_of("nib_xib_swiftui_showcase_viewport");
            let builder =
                ViewportBuilder::default().with_title(win_title).with_inner_size([920.0, 520.0]).with_min_inner_size([740.0, 420.0]);
            let (mut close_req, mut editing_kb) = (false, false);
            ctx.show_viewport_immediate(vp_id, builder, |vp_ui, _| {
                if vp_ui.input(|i| i.viewport().close_requested()) {
                    close_req = true;
                }
                let full = vp_ui.max_rect();
                vp_ui.painter_at(full).rect_filled(full, CornerRadius::ZERO, theme.window_background());
                let mut body_ui = vp_ui.new_child(
                    egui::UiBuilder::new().id_salt("nib_viewport_root_ui").max_rect(full.shrink(12.0)).layout(Layout::top_down(Align::Min)),
                );
                self.nib_studio.render_window_contents(&mut body_ui, theme, toasts);
                editing_kb = vp_ui.ctx().egui_wants_keyboard_input();
            });
            self.nib_studio.viewport_editing_text = editing_kb;
            if close_req {
                (self.nib_studio.open, self.nib_studio.viewport_editing_text) = (false, false);
            }
        }
        self.sync_all_nib_stores(&mut sync, toasts, sync_strings);
    }
}
