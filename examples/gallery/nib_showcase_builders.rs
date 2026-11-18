//! Declarative `.xib` $\rightarrow$ binary `.nib` (`NIBArchive\x01\x00\x00\x00`) compiler pipeline
//! and [`ShowcaseSceneDataStore`] (`REQ-013-02K`, `REQ-013-02L`, `REQ-013-02P`).

use egui::{Rect, Ui};
use egui_macview::{
    controls::MacControlSize,
    scene::{
        nib_varint::build_real_varint_nib_archive, ActionId, BindingId, CompiledConstraintSchedule, MacSceneDataSource, MacSceneIR,
        NibArchiveFrontend, NodeId, SceneNodeKind, XibXmlFrontend,
    },
    theme::MacTheme,
};
use std::{collections::BTreeMap, path::Path};

pub const SHOWCASE_NIB_FILES: [&str; 5] = [
    "1_Vector_and_Transform_Inspector.nib",
    "2_Theme_and_Appearance_Studio.nib",
    "3_AVKit_Audio_Mastering_Strip.nib",
    "4_Publishing_and_Asset_Queue.nib",
    "5_Binary_Varint_Inspector.nib",
];

const XIB_VECTOR_INSPECTOR: &str = include_str!("../../assets/nib_showcase/1_Vector_and_Transform_Inspector.xib");
const XIB_THEME_STUDIO: &str = include_str!("../../assets/nib_showcase/2_Theme_and_Appearance_Studio.xib");
const XIB_AVKIT_MASTERING: &str = include_str!("../../assets/nib_showcase/3_AVKit_Audio_Mastering_Strip.xib");
const XIB_PUBLISHING_QUEUE: &str = include_str!("../../assets/nib_showcase/4_Publishing_and_Asset_Queue.xib");
const XIB_TYPOGRAPHY_GRID: &str = include_str!("../../assets/nib_showcase/1_Typography_and_Grid_Inspector.xib");
const XIB_APPEARANCE_PREFS: &str = include_str!("../../assets/nib_showcase/2_Appearance_and_Theme_Settings.xib");
const XIB_RIGHT_INSPECTOR: &str = include_str!("../../assets/nib_showcase/Right_Inspector_Default.xib");

/// Live interactive backing store implementing [`MacSceneDataSource`] with two-way path mapping.
#[derive(Default, Clone)]
pub struct ShowcaseSceneDataStore {
    pub bools: BTreeMap<BindingId, bool>,
    pub floats: BTreeMap<BindingId, f64>,
    pub ints: BTreeMap<BindingId, i64>,
    pub strings: BTreeMap<BindingId, String>,
    pub colors: BTreeMap<BindingId, [u8; 4]>,
    pub options: BTreeMap<BindingId, Vec<String>>,
    pub path_to_id: BTreeMap<String, BindingId>,
    pub action_names: BTreeMap<ActionId, String>,
    pub dirty_paths: Vec<String>,
    pub pending_actions: Vec<String>,
    pub last_action: String,
}

impl ShowcaseSceneDataStore {
    pub fn seed_from_scene(&mut self, scene: &MacSceneIR) {
        self.path_to_id.clear();
        self.action_names.clear();
        self.dirty_paths.clear();
        self.pending_actions.clear();
        for node in scene.nodes() {
            if let (Some(act_id), Some(sel)) = (node.action, node.action_selector.as_ref()) {
                self.action_names.insert(act_id, sel.clone());
            }
            let Some(bid) = node.value_binding else {
                continue;
            };
            if let Some(ref path) = node.binding_path {
                self.path_to_id.insert(path.clone(), bid);
            }
            match &node.kind {
                SceneNodeKind::Checkbox | SceneNodeKind::Switch => {
                    self.bools.entry(bid).or_insert(true);
                }
                SceneNodeKind::NumberField { min, max, .. } => {
                    let default_v = min.zip(*max).map(|(a, b)| (a + b) * 0.5).unwrap_or(100.0);
                    let parsed = node.title.trim().parse::<f64>().ok().or_else(|| self.floats.get(&bid).copied()).unwrap_or(default_v);
                    let v = *self.floats.entry(bid).or_insert(parsed);
                    self.strings.insert(bid, format_nib_num(v));
                }
                SceneNodeKind::Slider { min, max, .. } => {
                    let parsed = node.title.trim().parse::<f64>().unwrap_or((*min + *max) * 0.5);
                    self.floats.entry(bid).or_insert(parsed);
                }
                SceneNodeKind::PopUpButton { options } => {
                    self.ints.entry(bid).or_insert(0);
                    if !options.is_empty() {
                        self.options.insert(bid, options.clone());
                    }
                }
                SceneNodeKind::SegmentedControl { segments } | SceneNodeKind::TabView { tabs: segments } => {
                    self.ints.entry(bid).or_insert(0);
                    if !segments.is_empty() {
                        self.options.insert(bid, segments.clone());
                    }
                }
                SceneNodeKind::OriginMatrix => {
                    self.ints.entry(bid).or_insert(4);
                }
                SceneNodeKind::ColorWell => {
                    self.colors.entry(bid).or_insert([0, 122, 255, 255]);
                }
                _ => {
                    self.strings.entry(bid).or_insert_with(|| node.title.clone());
                }
            }
        }
    }

    pub fn mark_dirty_id(&mut self, id: BindingId) {
        for (p, &bid) in &self.path_to_id {
            if bid == id && !self.dirty_paths.contains(p) {
                self.dirty_paths.push(p.clone());
            }
        }
    }

    pub fn resolve_or_create_binding(&mut self, path: &str) -> BindingId {
        let norm = path.strip_prefix("values.").unwrap_or(path);
        if let Some(&bid) = self.path_to_id.get(norm).or_else(|| self.path_to_id.get(path)) {
            return bid;
        }
        let next_id = BindingId(900 + self.path_to_id.len() as u32);
        self.path_to_id.insert(norm.to_string(), next_id);
        next_id
    }

    pub fn set_path_f64(&mut self, path: &str, val: f64) {
        let norm = path.strip_prefix("values.").unwrap_or(path).to_string();
        let bid = self.resolve_or_create_binding(&norm);
        self.floats.insert(bid, val);
        self.strings.insert(bid, format_nib_num(val));
        if !self.dirty_paths.contains(&norm) {
            self.dirty_paths.push(norm);
        }
    }

    pub fn get_path_f64(&self, path: &str) -> f64 {
        let norm = path.strip_prefix("values.").unwrap_or(path);
        self.path_to_id.get(norm).or_else(|| self.path_to_id.get(path)).and_then(|bid| self.floats.get(bid).copied()).unwrap_or(0.0)
    }

    pub fn set_path_i64(&mut self, path: &str, val: i64) {
        let norm = path.strip_prefix("values.").unwrap_or(path).to_string();
        let bid = self.resolve_or_create_binding(&norm);
        self.ints.insert(bid, val);
        if !self.dirty_paths.contains(&norm) {
            self.dirty_paths.push(norm);
        }
    }
}

impl MacSceneDataSource for ShowcaseSceneDataStore {
    fn get_bool(&self, id: BindingId) -> bool {
        self.bools.get(&id).copied().unwrap_or(false)
    }
    fn set_bool(&mut self, id: BindingId, val: bool) {
        if self.bools.insert(id, val) != Some(val) {
            self.mark_dirty_id(id);
        }
    }
    fn get_f64(&self, id: BindingId) -> f64 {
        self.floats.get(&id).copied().unwrap_or(0.0)
    }
    fn set_f64(&mut self, id: BindingId, val: f64) {
        let prev = self.floats.insert(id, val);
        self.strings.insert(id, format_nib_num(val));
        if prev.map(|p| (p - val).abs() > 1e-4).unwrap_or(true) {
            self.mark_dirty_id(id);
        }
    }
    fn get_i64(&self, id: BindingId) -> i64 {
        self.ints.get(&id).copied().unwrap_or(0)
    }
    fn set_i64(&mut self, id: BindingId, val: i64) {
        if self.ints.insert(id, val) != Some(val) {
            self.mark_dirty_id(id);
        }
    }
    fn get_str(&self, id: BindingId) -> &str {
        self.strings.get(&id).map(String::as_str).unwrap_or("")
    }
    fn set_str(&mut self, id: BindingId, val: &str) {
        let prev = self.strings.insert(id, val.to_string());
        if let Ok(v) = val.trim().parse::<f64>() {
            self.floats.insert(id, v);
        }
        if prev.as_deref() != Some(val) {
            self.mark_dirty_id(id);
        }
    }
    fn get_options(&self, id: BindingId) -> &[String] {
        self.options.get(&id).map(Vec::as_slice).unwrap_or(&[])
    }
    fn get_color(&self, id: BindingId) -> Option<egui::Color32> {
        let [r, g, b, a] = self.colors.get(&id).copied().unwrap_or([0, 122, 255, 255]);
        Some(egui::Rgba::from_srgba_unmultiplied(r, g, b, a).into())
    }
    fn set_color(&mut self, id: BindingId, col: egui::Color32) {
        let rgba = col.to_srgba_unmultiplied();
        if self.colors.insert(id, rgba) != Some(rgba) {
            self.mark_dirty_id(id);
        }
    }
    fn on_action(&mut self, action: ActionId, sender: NodeId) {
        let name = self.action_names.get(&action).cloned().unwrap_or_else(|| format!("action#{}", action.0));
        self.last_action = format!("Triggered {name} (Node #{})", sender.0);
        self.pending_actions.push(name);
    }
    fn render_custom_slot(&mut self, _ui: &mut Ui, _theme: &MacTheme, _cls: &str, _rect: Rect) {}
}

/// Formats a numeric `.xib` binding value cleanly (preserving decimals for small ranges like `0.0..=2.0`).
pub fn format_nib_num(val: f64) -> String {
    if val.abs() <= 10.0 && (val.fract()).abs() > 1e-3 {
        format!("{val:.2}")
    } else {
        format!("{val:.0}")
    }
}

/// Parses an Xcode `.xib` XML string into [`MacSceneIR`], normalizing AppKit coordinates via `isFlipped`.
pub fn compile_xib_scene(xib_xml: &str) -> MacSceneIR {
    XibXmlFrontend::from_xml_str(xib_xml)
        .unwrap_or_else(|_| MacSceneIR::from_nodes(NodeId(1), [440.0, 310.0], Vec::new(), CompiledConstraintSchedule::default()))
}

/// Decodes an Xcode `ibtool --compile` binary `.nib` archive directly into [`MacSceneIR`].
pub fn compile_nib_scene(nib_bytes: &[u8], fallback_xib: &str) -> MacSceneIR {
    let mapper = egui_macview::scene::CustomClassMapper::default_studio_mappings();
    NibArchiveFrontend::from_bytes(nib_bytes, &mapper).unwrap_or_else(|_| compile_xib_scene(fallback_xib))
}

/// Returns the compiled [`MacSceneIR`] for an embedded `.nib` or `.xib` showcase asset by file name.
pub fn compile_embedded_showcase_by_name(name: &str) -> Option<MacSceneIR> {
    if name.starts_with("5_Binary_Varint") {
        let mapper = egui_macview::scene::CustomClassMapper::default_studio_mappings();
        return NibArchiveFrontend::from_bytes(&build_real_varint_nib_archive(), &mapper).ok();
    }
    let stem = name.trim_end_matches(".nib").trim_end_matches(".xib");
    let (nib, xml) = match stem {
        "1_Vector_and_Transform_Inspector" => (NIB_VECTOR_INSPECTOR, XIB_VECTOR_INSPECTOR),
        "2_Theme_and_Appearance_Studio" => (NIB_THEME_STUDIO, XIB_THEME_STUDIO),
        "3_AVKit_Audio_Mastering_Strip" => (NIB_AVKIT_MASTERING, XIB_AVKIT_MASTERING),
        "4_Publishing_and_Asset_Queue" => (NIB_PUBLISHING_QUEUE, XIB_PUBLISHING_QUEUE),
        "1_Typography_and_Grid_Inspector" => (NIB_TYPOGRAPHY_GRID, XIB_TYPOGRAPHY_GRID),
        "2_Appearance_and_Theme_Settings" => (NIB_APPEARANCE_PREFS, XIB_APPEARANCE_PREFS),
        "Right_Inspector_Default" => (NIB_RIGHT_INSPECTOR, XIB_RIGHT_INSPECTOR),
        _ => return None,
    };
    if name.ends_with(".xib") {
        Some(compile_xib_scene(xml))
    } else {
        Some(compile_nib_scene(nib, xml))
    }
}

/// Returns the compiled [`MacSceneIR`] for the Right Inspector (`Right_Inspector_Default.nib`).
pub fn compile_right_inspector_scene() -> MacSceneIR {
    static RIGHT_SCENE: std::sync::OnceLock<MacSceneIR> = std::sync::OnceLock::new();
    RIGHT_SCENE.get_or_init(|| compile_nib_scene(NIB_RIGHT_INSPECTOR, XIB_RIGHT_INSPECTOR)).clone()
}

/// Returns the compiled [`MacSceneIR`] and seeded [`ShowcaseSceneDataStore`] for each of the 6 Gallery screens (`0..=5`).
pub fn compile_gallery_screen_scenes() -> Vec<(std::sync::Arc<MacSceneIR>, ShowcaseSceneDataStore)> {
    static SCREEN_SCENES: std::sync::OnceLock<Vec<(std::sync::Arc<MacSceneIR>, ShowcaseSceneDataStore)>> = std::sync::OnceLock::new();
    SCREEN_SCENES
        .get_or_init(|| {
            [
                (NIB_VECTOR_INSPECTOR, XIB_VECTOR_INSPECTOR),
                (NIB_TYPOGRAPHY_GRID, XIB_TYPOGRAPHY_GRID),
                (NIB_APPEARANCE_PREFS, XIB_APPEARANCE_PREFS),
                (NIB_PUBLISHING_QUEUE, XIB_PUBLISHING_QUEUE),
                (NIB_AVKIT_MASTERING, XIB_AVKIT_MASTERING),
                (NIB_THEME_STUDIO, XIB_THEME_STUDIO),
            ]
            .into_iter()
            .map(|(nib, xml)| {
                let scene = std::sync::Arc::new(compile_nib_scene(nib, xml));
                let mut store = ShowcaseSceneDataStore::default();
                store.seed_from_scene(&scene);
                (scene, store)
            })
            .collect()
        })
        .clone()
}

struct CenterSceneBridge<'a, 'b> {
    center: &'a mut super::CenterState,
    scene: &'a MacSceneIR,
    screen: super::GalleryScreen,
    store: &'a mut ShowcaseSceneDataStore,
    args: &'a super::center_state::CenterRenderArgs<'b>,
    toasts: &'a std::cell::RefCell<egui_macview::MacToastManager>,
}

const STUDIO_RULE_CRITERIA: &[&[&str]] = &[
    &["Name", "Kind", "Created Date", "File Size"],
    &["contains", "is exactly", "begins with", "ends with"],
    &["ProRes", "MOV", "HEVC", "PNG"],
];

impl CenterSceneBridge<'_, '_> {
    fn is_in_binding_rows(&self, id: NodeId, row_bindings: &[&str]) -> bool {
        self.scene.field_rows().iter().any(|row| {
            row.nodes.contains(&id)
                && row.nodes.iter().any(|&rid| {
                    self.scene
                        .nodes()
                        .iter()
                        .find(|n| n.id == rid)
                        .and_then(|n| n.binding_path.as_deref())
                        .is_some_and(|bp| row_bindings.contains(&bp))
                })
        })
    }
}

impl MacSceneDataSource for CenterSceneBridge<'_, '_> {
    fn get_bool(&self, id: BindingId) -> bool {
        self.store.get_bool(id)
    }
    fn set_bool(&mut self, id: BindingId, val: bool) {
        self.store.set_bool(id, val);
    }
    fn get_f64(&self, id: BindingId) -> f64 {
        self.store.get_f64(id)
    }
    fn set_f64(&mut self, id: BindingId, val: f64) {
        self.store.set_f64(id, val);
    }
    fn get_i64(&self, id: BindingId) -> i64 {
        self.store.get_i64(id)
    }
    fn set_i64(&mut self, id: BindingId, val: i64) {
        self.store.set_i64(id, val);
    }
    fn get_str(&self, id: BindingId) -> &str {
        self.store.get_str(id)
    }
    fn set_str(&mut self, id: BindingId, val: &str) {
        self.store.set_str(id, val);
    }
    fn get_options(&self, id: BindingId) -> &[String] {
        self.store.get_options(id)
    }
    fn get_color(&self, id: BindingId) -> Option<egui::Color32> {
        self.store.get_color(id)
    }
    fn set_color(&mut self, id: BindingId, col: egui::Color32) {
        self.store.set_color(id, col);
    }
    fn on_action(&mut self, action: ActionId, sender: NodeId) {
        if let Some(sel) = self.store.action_names.get(&action).map(String::as_str) {
            if sel == "undoTransform:" {
                let mut card = self.args.card_cell.borrow_mut();
                if self.args.undo_manager.borrow_mut().undo(&mut card) {
                    *self.args.last_stable_card.borrow_mut() = card.clone();
                }
            } else if sel == "redoTransform:" {
                let mut card = self.args.card_cell.borrow_mut();
                if self.args.undo_manager.borrow_mut().redo(&mut card) {
                    *self.args.last_stable_card.borrow_mut() = card.clone();
                }
            }
        }
        self.store.on_action(action, sender);
    }
    fn is_node_enabled(&self, id: NodeId, _ib_object_id: &str, default_enabled: bool) -> bool {
        if self.screen == super::GalleryScreen::AdvancedControls && self.is_in_binding_rows(id, &["nibWidthExpr", "nibCheckRegular"]) {
            self.center.nib_inspector_enabled && default_enabled
        } else if self.screen == super::GalleryScreen::DesignTokensWcag && self.is_in_binding_rows(id, &["matrixInput"]) {
            self.center.showcase_controls_enabled && default_enabled
        } else {
            default_enabled
        }
    }
    fn resolve_control_size(&self, id: NodeId, _ib_object_id: &str, default_size: MacControlSize) -> MacControlSize {
        let Some(node) = self.scene.nodes().iter().find(|n| n.id == id) else {
            return default_size;
        };
        if self.screen == super::GalleryScreen::DesignTokensWcag && self.is_in_binding_rows(id, &["matrixInput"]) {
            match self.center.showcase_control_size_idx {
                1 => MacControlSize::Small,
                2 => MacControlSize::Mini,
                _ => MacControlSize::Regular,
            }
        } else if self.screen == super::GalleryScreen::AdvancedControls
            && self.is_in_binding_rows(id, &["nibWidthExpr", "nibCheckRegular"])
            && matches!(node.kind, SceneNodeKind::TextField | SceneNodeKind::PopUpButton { .. } | SceneNodeKind::ComboBox { .. })
        {
            self.center.resolved_state6_control_size()
        } else {
            default_size
        }
    }
    fn resolve_level_discrete(&self, _id: NodeId, _ib_object_id: &str, _default_discrete: bool) -> bool {
        self.center.media_discrete_vu
    }
    fn resolve_level_thresholds(&self, _id: NodeId, _ib_object_id: &str, _default_thresholds: bool) -> bool {
        self.center.media_threshold_vu
    }
    fn get_rule_criteria(&self, _id: NodeId, _ib_object_id: &str) -> &[&[&str]] {
        STUDIO_RULE_CRITERIA
    }
    fn render_tab_item(&mut self, ui: &mut Ui, theme: &MacTheme, _id: NodeId, _ib_object_id: &str, tab_idx: usize, rect: Rect) {
        self.center.render_state6_tab_item(ui, theme, tab_idx, rect);
    }
    fn render_custom_slot(&mut self, ui: &mut Ui, theme: &MacTheme, slot_class: &str, rect: Rect) {
        let mut card = self.args.card_cell.borrow_mut();
        match slot_class {
            "NSAffineVectorCanvasView" => self.center.render_state1_canvas_slot(ui, theme, &mut card, self.args, self.toasts, rect),
            "NSCupertinoSymbolGridView" => self.center.render_symbol_grid_slot(ui, theme, self.toasts, rect),
            "NSFixedClippingPreviewView" => self.center.render_fixed_clipping_slot(ui, theme, &card, rect),
            "NSRowAndScrollPreviewView" => self.center.render_row_scroll_slot(ui, theme, rect),
            "NSWcagContrastSampleView" => self.center.render_wcag_contrast_sample_slot(ui, theme, rect),
            "NSFinderBrowserSurfaceView" => self.center.render_finder_browser_slot(ui, theme, self.toasts, rect),
            "NSAvKitVideoAndScrubberView" => self.center.render_avkit_video_scrubber_slot(ui, theme, rect),
            _ => {}
        }
    }
}

impl super::CenterState {
    /// Queries an option or segment label directly from the compiled `.xib`/`.nib` `MacSceneIR` (`REQ-013-07.3`).
    pub fn scene_node_option(&self, screen_idx: usize, ib_id: &str, opt_idx: usize) -> &str {
        self.screen_nib_scenes
            .get(screen_idx)
            .and_then(|(sc, _)| sc.nodes().iter().find(|n| n.ib_object_id == ib_id))
            .and_then(|n| match &n.kind {
                SceneNodeKind::PopUpButton { options } | SceneNodeKind::ComboBox { options } => {
                    options.get(opt_idx).or_else(|| options.first()).map(String::as_str)
                }
                SceneNodeKind::SegmentedControl { segments } => {
                    segments.get(opt_idx).or_else(|| segments.first()).map(String::as_str)
                }
                _ => None,
            })
            .unwrap_or("")
    }

    /// Renders the active screen's compiled `.xib` / `.nib` scene via [`MacSceneIR::show`] (`REQ-013-02L`, `REQ-013-02P`).
    pub fn render_active_screen_nib(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        args: super::center_state::CenterRenderArgs<'_>,
        screen: super::GalleryScreen,
        toasts: &std::cell::RefCell<egui_macview::MacToastManager>,
    ) {
        let idx = (screen as usize).min(self.screen_nib_scenes.len().saturating_sub(1));
        let search_q = self.search_query.clone();
        let Some((scene_ref, store_ref)) = self.screen_nib_scenes.get_mut(idx) else {
            return;
        };
        let scene = std::sync::Arc::clone(scene_ref);
        let mut store = std::mem::take(store_ref);
        let mut bridge = CenterSceneBridge { center: self, scene: &scene, screen, store: &mut store, args: &args, toasts };
        scene.show(ui, theme, &mut bridge, egui_macview::scene::LayoutPolicy::ExactAppleNib, 0, &search_q);
        if let Some((_, dst_store)) = self.screen_nib_scenes.get_mut(idx) {
            *dst_store = store;
        }
    }
}

/// Resolves the `assets/nib_showcase` directory relative to either the current working directory or `CARGO_MANIFEST_DIR`.
pub fn resolve_showcase_nib_dir() -> std::path::PathBuf {
    let cwd_dir = std::env::current_dir().ok().map(|d| d.join("assets").join("nib_showcase"));
    if let Some(d) = cwd_dir.filter(|d| d.exists()) {
        return d;
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("nib_showcase")
}

const NIB_VECTOR_INSPECTOR: &[u8] = include_bytes!("../../assets/nib_showcase/1_Vector_and_Transform_Inspector.nib");
const NIB_THEME_STUDIO: &[u8] = include_bytes!("../../assets/nib_showcase/2_Theme_and_Appearance_Studio.nib");
const NIB_AVKIT_MASTERING: &[u8] = include_bytes!("../../assets/nib_showcase/3_AVKit_Audio_Mastering_Strip.nib");
const NIB_PUBLISHING_QUEUE: &[u8] = include_bytes!("../../assets/nib_showcase/4_Publishing_and_Asset_Queue.nib");
const NIB_TYPOGRAPHY_GRID: &[u8] = include_bytes!("../../assets/nib_showcase/1_Typography_and_Grid_Inspector.nib");
const NIB_APPEARANCE_PREFS: &[u8] = include_bytes!("../../assets/nib_showcase/2_Appearance_and_Theme_Settings.nib");
const NIB_RIGHT_INSPECTOR: &[u8] = include_bytes!("../../assets/nib_showcase/Right_Inspector_Default.nib");

/// Ensures all binary `.nib` showcase files in `assets/nib_showcase/` exist on disk using the real `ibtool`-compiled archives.
pub fn ensure_showcase_nib_files(dir: &Path) {
    static VARINT_NIB: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    let varint_bytes = VARINT_NIB.get_or_init(build_real_varint_nib_archive);
    let files: [(&str, &[u8]); 6] = [
        ("1_Vector_and_Transform_Inspector.nib", NIB_VECTOR_INSPECTOR),
        ("2_Theme_and_Appearance_Studio.nib", NIB_THEME_STUDIO),
        ("3_AVKit_Audio_Mastering_Strip.nib", NIB_AVKIT_MASTERING),
        ("4_Publishing_and_Asset_Queue.nib", NIB_PUBLISHING_QUEUE),
        ("5_Binary_Varint_Inspector.nib", varint_bytes.as_slice()),
        ("Right_Inspector_Default.nib", NIB_RIGHT_INSPECTOR),
    ];
    let _ = std::fs::create_dir_all(dir);
    for (name, bytes) in files {
        let p = dir.join(name);
        if !p.exists() {
            let _ = std::fs::write(p, bytes);
        }
    }
}
