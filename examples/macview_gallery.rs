//! `macview_gallery` — Comprehensive macOS AppKit & AVKit Capability Showcase (`v0.1` – `v0.5`).
//!
//! Organized into 5 interactive Studio States (`GalleryScreen`):
//! 1. **State 1 (`v0.1–v0.2` — Vector & Inspector Studio)**: 2D Affine Canvas, live Inspector, `MacFloatingInfoBox`.
//! 2. **State 2 (`v0.3` — Cupertino Symbols & Layout Lab)**: 14 `MacSymbol` glyphs, `MacFixedView`, `MacRow`, `MacScrollView`.
//! 3. **State 3 (`v0.4–v0.4.1` — Command, Menu & State Integrity Lab)**: `NSUndoManager`, `MacControlSize`, WCAG 2.1 AA Engine.
//! 4. **State 4 (`v0.5` — Finder & Publishing Data Browser)**: `MacPathControl`, `MacTableView`, `MacTokenField`, `MacProgressIndicator`.
//! 5. **State 5 (`v0.5` — AVKit Media & Audio Timeline Studio)**: `MacTrimScrubber` & stereo `MacLevelIndicator` VU meters.

#![cfg_attr(not(test), windows_subsystem = "windows")]

#[path = "gallery/mod.rs"]
mod gallery;

use egui::{pos2, vec2, Align, CornerRadius, Layout, Rect, Stroke, StrokeKind};
use egui_macview::{
    setup_symbols_font,
    theme::{MacAppearance, MacTheme, WindowControlsStyle},
    CommandPaletteItem, MacCommandPalette, MacMenuBar, MacSplitOrientation, MacSplitView,
    MacToastManager, NSMenu, NSUndoManager,
};
pub use gallery::{detect_frameless_resize_dir, CardModel};
use gallery::{inspector::InspectorState, CenterState, SidebarState};
use std::cell::{Cell, RefCell};

/// Formats uppercase section headers with optical tracking.
pub fn format_tracked_small_caps(text: &str) -> String {
    text.to_uppercase()
        .split_whitespace()
        .map(|w| w.chars().map(|c| c.to_string()).collect::<Vec<_>>().join("\u{2009}"))
        .collect::<Vec<_>>()
        .join("   ")
}

fn setup_system_or_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let font_dir = std::path::Path::new(&local_app_data).join("Microsoft\\Windows\\Fonts");
    for (name, path, family) in [
        ("sf_pro", font_dir.join("SF-Pro-Text-Regular.otf"), egui::FontFamily::Proportional),
        ("inter", font_dir.join("Inter-Variable.ttf"), egui::FontFamily::Proportional),
        ("sf_mono", font_dir.join("SF-Mono-Regular.otf"), egui::FontFamily::Monospace),
        ("cascadia_mono", std::path::Path::new("C:\\Windows\\Fonts\\CascadiaMono.ttf").to_path_buf(), egui::FontFamily::Monospace),
    ] {
        if let Ok(bytes) = std::fs::read(&path) {
            fonts.font_data.insert(name.to_owned(), std::sync::Arc::new(egui::FontData::from_owned(bytes)));
            fonts.families.entry(family).or_default().insert(0, name.to_owned());
        }
    }
    setup_symbols_font(&mut fonts);
    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result<()> {
    let renderer = match std::env::var("MACVIEW_RENDERER").as_deref() {
        Ok("glow") => eframe::Renderer::Glow,
        Ok("wgpu") => eframe::Renderer::Wgpu,
        _ if cfg!(target_os = "macos") => eframe::Renderer::Glow,
        _ => eframe::Renderer::Wgpu,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1160.0, 740.0])
            .with_min_inner_size([900.0, 540.0])
            .with_decorations(false)
            .with_title(format!("macview_gallery v{} (AppKit & AVKit Studio Showcase)", egui_macview::VERSION)),
        renderer,
        ..Default::default()
    };
    eframe::run_native("macview_gallery", options, Box::new(|cc| {
        setup_system_or_custom_fonts(&cc.egui_ctx);
        Ok(Box::new(GalleryApp::new()))
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GalleryScreen {
    #[default]
    CanvasPhysics = 0,
    VectorAssets = 1,
    DesignTokensWcag = 2,
    FinderDataBrowser = 3,
    AvKitMediaStudio = 4,
}

impl GalleryScreen {
    pub const ALL: [Self; 5] = [
        Self::CanvasPhysics,
        Self::VectorAssets,
        Self::DesignTokensWcag,
        Self::FinderDataBrowser,
        Self::AvKitMediaStudio,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::CanvasPhysics => "v0.1–v0.2: Vector & Inspector Studio",
            Self::VectorAssets => "v0.3: Symbols & Layout Lab",
            Self::DesignTokensWcag => "v0.4–v0.4.1: Command & State Lab",
            Self::FinderDataBrowser => "v0.5: Finder & Publishing Browser",
            Self::AvKitMediaStudio => "v0.5: AVKit Media & Timeline Studio",
        }
    }

    pub fn sidebar_label(&self) -> &'static str {
        match self {
            Self::CanvasPhysics => "1. Vector Studio (v0.2)",
            Self::VectorAssets => "2. Symbols & Layout (v0.3)",
            Self::DesignTokensWcag => "3. Command & State (v0.4)",
            Self::FinderDataBrowser => "4. Finder & Publish (v0.5)",
            Self::AvKitMediaStudio => "5. AVKit Timeline (v0.5)",
        }
    }

    pub fn search_placeholder(&self) -> &'static str {
        match self {
            Self::CanvasPhysics => "Search canvas geometry, transforms, CASpring physics...",
            Self::VectorAssets => "Filter 14 AppKit vector symbols (e.g. search, chevron, gear)...",
            Self::DesignTokensWcag => "Filter W3C design tokens (e.g. button, metric, motion)...",
            Self::FinderDataBrowser => "Filter v0.5 table assets by name, codec, or NSTokenField tag...",
            Self::AvKitMediaStudio => "Search AVKit timeline scenes and audio channels...",
        }
    }

    pub fn breadcrumb_path(&self) -> [&'static str; 3] {
        match self {
            Self::CanvasPhysics => ["macview_gallery", "State 1 (v0.2)", "Vector & Inspector Studio"],
            Self::VectorAssets => ["macview_gallery", "State 2 (v0.3)", "Symbols & Layout Lab"],
            Self::DesignTokensWcag => ["macview_gallery", "State 3 (v0.4.1)", "Command & State Lab"],
            Self::FinderDataBrowser => ["macview_gallery", "State 4 (v0.5)", "Finder & Publishing"],
            Self::AvKitMediaStudio => ["macview_gallery", "State 5 (v0.5)", "AVKit Media Timeline"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorMode {
    DefaultNib = 0,
    CompactHud = 1,
    ExpandedPro = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandedProTab {
    Attributes = 0,
    Metrics = 1,
    Matrix = 2,
}

pub(crate) struct GalleryApp {
    pub(crate) appearance: MacAppearance,
    pub(crate) window_controls_style: WindowControlsStyle,
    pub(crate) last_window_controls_style: Option<WindowControlsStyle>,
    pub(crate) inspector_mode: InspectorMode,
    pub(crate) last_inspector_mode: InspectorMode,
    pub(crate) active_screen: GalleryScreen,
    pub(crate) is_maximized: bool,
    pub(crate) left_split_pos: f32,
    pub(crate) inspector_width: f32,
    pub(crate) card: RefCell<CardModel>,
    pub(crate) undo_manager: RefCell<NSUndoManager<CardModel>>,
    pub(crate) last_stable_card: RefCell<CardModel>,
    pub(crate) is_dragging_card: Cell<bool>,
    pub(crate) drag_start_card: RefCell<Option<CardModel>>,
    pub(crate) active_text_focus: Cell<Option<egui::Id>>,
    pub(crate) text_focus_start_card: RefCell<Option<CardModel>>,
    pub(crate) toasts: RefCell<MacToastManager>,
    pub(crate) sidebar: SidebarState,
    pub(crate) center: CenterState,
    pub(crate) inspector: InspectorState,
    pub(crate) menus: Vec<NSMenu>,
    pub(crate) open_menu_idx: Option<usize>,
    pub(crate) highlighted_item_idx: Option<usize>,
    pub(crate) palette: MacCommandPalette,
}

impl GalleryApp {
    fn new() -> Self {
        let palette_items = vec![
            CommandPaletteItem::new("edit.undo", "Edit: Undo").category("Edit").shortcut("⌘Z / Ctrl+Z"),
            CommandPaletteItem::new("edit.redo", "Edit: Redo").category("Edit").shortcut("⌘⇧Z / Ctrl+Shift+Z"),
            CommandPaletteItem::new("screen.canvas", "State 1 (v0.2): Vector & Inspector Studio").category("States"),
            CommandPaletteItem::new("screen.vector", "State 2 (v0.3): Symbols & Layout Lab").category("States"),
            CommandPaletteItem::new("screen.tokens", "State 3 (v0.4.1): Command & State Lab").category("States"),
            CommandPaletteItem::new("screen.finder", "State 4 (v0.5): Finder & Publishing Browser").category("States"),
            CommandPaletteItem::new("screen.avkit", "State 5 (v0.5): AVKit Media & Timeline Studio").category("States"),
            CommandPaletteItem::new("theme.aqua", "Appearance: Switch to Aqua (Light)").category("Theme"),
            CommandPaletteItem::new("theme.dark", "Appearance: Switch to DarkAqua (Dark)").category("Theme"),
            CommandPaletteItem::new("canvas.pulse", "Physics: Trigger CASpring Pulse").category("Motion"),
            CommandPaletteItem::new("matrix.reset", "Canvas: Reset Affine Transform to Identity").category("Canvas"),
        ];
        let default_card = CardModel::default();
        Self {
            appearance: MacAppearance::Aqua,
            window_controls_style: WindowControlsStyle::AppleTrafficLights,
            last_window_controls_style: None,
            inspector_mode: InspectorMode::ExpandedPro,
            last_inspector_mode: InspectorMode::ExpandedPro,
            active_screen: GalleryScreen::CanvasPhysics,
            is_maximized: false,
            left_split_pos: 235.0,
            inspector_width: 350.0,
            card: RefCell::new(default_card.clone()),
            undo_manager: RefCell::new(NSUndoManager::new()),
            last_stable_card: RefCell::new(default_card),
            is_dragging_card: Cell::new(false),
            drag_start_card: RefCell::new(None),
            active_text_focus: Cell::new(None),
            text_focus_start_card: RefCell::new(None),
            toasts: RefCell::new(MacToastManager::new()),
            sidebar: SidebarState::default(),
            center: CenterState::default(),
            inspector: InspectorState::default(),
            menus: NSMenu::default_menu_bar(),
            open_menu_idx: None,
            highlighted_item_idx: None,
            palette: MacCommandPalette::new(palette_items),
        }
    }
}

impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let focused_id = ui.memory(|m| m.focused());
        let prev_focused = self.active_text_focus.get();
        if prev_focused.is_some() && prev_focused != focused_id {
            if let Some(start_card) = self.text_focus_start_card.borrow_mut().take() {
                let current = self.card.borrow().clone();
                if current != start_card {
                    self.undo_manager.borrow_mut().push_snapshot(start_card);
                    *self.last_stable_card.borrow_mut() = current;
                }
            }
        }
        if focused_id.is_some() && prev_focused != focused_id {
            *self.text_focus_start_card.borrow_mut() = Some(self.last_stable_card.borrow().clone());
        }
        self.active_text_focus.set(focused_id);

        if ui.input(|i| i.pointer.any_down()) {
            if !self.is_dragging_card.get() {
                self.is_dragging_card.set(true);
                let initial = self.last_stable_card.borrow().clone();
                *self.drag_start_card.borrow_mut() = Some(initial.clone());
                self.undo_manager.borrow_mut().begin_grouping(initial);
            }
        } else if self.is_dragging_card.get() {
            self.is_dragging_card.set(false);
            let _ = self.drag_start_card.borrow_mut().take();
            let current = self.card.borrow().clone();
            if self.undo_manager.borrow_mut().end_grouping(&current) {
                *self.last_stable_card.borrow_mut() = current;
            }
        }

        let theme = MacTheme::new(self.appearance)
            .with_accent(self.card.borrow().fill_color)
            .with_window_controls_style(self.window_controls_style);
        theme.apply_to_egui(ui.ctx());

        let full_rect = ui.available_rect_before_wrap();
        let on_resize_border = self.handle_frameless_resize(ui, full_rect, &theme);

        let rounding = if self.window_controls_style == WindowControlsStyle::NativeHost {
            CornerRadius::ZERO
        } else {
            CornerRadius::same(10)
        };
        ui.painter_at(full_rect).rect(
            full_rect,
            rounding,
            theme.window_background(),
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );

        let toolbar_height = theme.toolbar_metrics().height_unified.points();
        let toolbar_rect = Rect::from_min_size(full_rect.min, vec2(full_rect.width(), toolbar_height));
        self.render_toolbar(ui, toolbar_rect, &theme, on_resize_border);

        let menu_bar_h = 24.0;
        let menu_bar_rect = Rect::from_min_size(
            pos2(full_rect.min.x, full_rect.min.y + toolbar_height),
            vec2(full_rect.width(), menu_bar_h),
        );
        let mp = ui.painter_at(menu_bar_rect);
        mp.rect_filled(menu_bar_rect, CornerRadius::ZERO, theme.window_background());
        mp.line_segment(
            [pos2(menu_bar_rect.min.x, menu_bar_rect.max.y), pos2(menu_bar_rect.max.x, menu_bar_rect.max.y)],
            Stroke::new(1.0, theme.separator()),
        );

        let mut menu_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("gallery_menu_bar")
                .max_rect(menu_bar_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        menu_ui.add_space(8.0);
        if let Some(action) = MacMenuBar::new(&self.menus, &mut self.open_menu_idx, &mut self.highlighted_item_idx)
            .theme(&theme)
            .show(&mut menu_ui)
        {
            self.handle_menu_action(&action, ui.ctx());
        }

        if ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::COMMAND, egui::Key::K)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::K)
                || i.consume_key(egui::Modifiers::MAC_CMD, egui::Key::K)
        }) {
            self.palette.toggle();
        }
        if let Some(action) = self.palette.show(ui.ctx(), &theme) {
            self.handle_menu_action(&action, ui.ctx());
        }

        let work_rect = Rect::from_min_max(
            pos2(full_rect.min.x, full_rect.min.y + toolbar_height + menu_bar_h),
            full_rect.max,
        );
        let mut work_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("gallery_work_area")
                .max_rect(work_rect)
                .layout(Layout::left_to_right(Align::Min)),
        );

        let mut left_split_pos = self.left_split_pos;
        if self.last_inspector_mode != self.inspector_mode {
            self.inspector_width = match self.inspector_mode {
                InspectorMode::DefaultNib => 310.0,
                InspectorMode::CompactHud => 240.0,
                InspectorMode::ExpandedPro => 350.0,
            };
            self.last_inspector_mode = self.inspector_mode;
        }

        let available_body_w = work_rect.width() - left_split_pos;
        let min_canvas_w = 340.0;
        let min_inspector_w = match self.inspector_mode {
            InspectorMode::ExpandedPro => 330.0,
            InspectorMode::DefaultNib => 280.0,
            InspectorMode::CompactHud => 220.0,
        };
        let max_inspector_w = (available_body_w - min_canvas_w).max(min_inspector_w);
        let current_inspector_w = self.inspector_width.clamp(min_inspector_w, max_inspector_w);
        let mut right_split_pos = (available_body_w - current_inspector_w).max(min_canvas_w);

        let sidebar = &mut self.sidebar;
        let center = &mut self.center;
        let card = &self.card;
        let inspector = &mut self.inspector;
        let inspector_mode_cell = Cell::new(self.inspector_mode);
        let active_screen_cell = Cell::new(self.active_screen);
        let vis_canvas_cell = Cell::new(sidebar.vis_canvas);
        let lock_canvas_cell = Cell::new(sidebar.lock_canvas);
        let snapping_enabled = inspector.snapping_enabled;
        let unit_segmented_idx = inspector.unit_segmented_idx;

        MacSplitView::with_pos("gallery_left_split", MacSplitOrientation::Horizontal, &mut left_split_pos)
            .theme(&theme)
            .min_first(220.0)
            .max_first(340.0)
            .min_second(620.0)
            .show(
                &mut work_ui,
                |sidebar_ui| {
                    let (mut m, mut s) = (inspector_mode_cell.get(), active_screen_cell.get());
                    sidebar.render(sidebar_ui, &theme, &mut m, &mut s, &self.toasts);
                    inspector_mode_cell.set(m);
                    active_screen_cell.set(s);
                    vis_canvas_cell.set(sidebar.vis_canvas);
                    lock_canvas_cell.set(sidebar.lock_canvas);
                },
                |body_ui| {
                    let (vis_canvas, lock_canvas) = (vis_canvas_cell.get(), lock_canvas_cell.get());
                    MacSplitView::with_pos("gallery_right_split", MacSplitOrientation::Horizontal, &mut right_split_pos)
                        .theme(&theme)
                        .min_first(min_canvas_w)
                        .min_second(min_inspector_w)
                        .show(
                            body_ui,
                            |center_ui| {
                                let mut s = active_screen_cell.get();
                                center.render(
                                    center_ui, &theme, card, &self.undo_manager, &self.last_stable_card,
                                    inspector_mode_cell.get(), &mut s, snapping_enabled, unit_segmented_idx,
                                    vis_canvas, lock_canvas, &self.toasts,
                                );
                                active_screen_cell.set(s);
                            },
                            |inspector_ui| {
                                let mut m = inspector_mode_cell.get();
                                inspector.render(inspector_ui, &theme, card, &mut m, &self.toasts);
                                inspector_mode_cell.set(m);
                            },
                        );
                },
            );

        self.inspector_mode = inspector_mode_cell.get();
        let new_screen = active_screen_cell.get();
        if self.active_screen != new_screen {
            self.active_screen = new_screen;
            self.sidebar.active_screen = new_screen;
            self.sidebar.selected_node = new_screen.title().to_string();
        }
        self.left_split_pos = left_split_pos;
        self.inspector_width = (available_body_w - right_split_pos).clamp(min_inspector_w, max_inspector_w);

        if !ui.input(|i| i.pointer.any_down()) && !self.is_dragging_card.get() && focused_id.is_none() {
            let current = self.card.borrow().clone();
            let last_stable = self.last_stable_card.borrow().clone();
            if current != last_stable {
                self.undo_manager.borrow_mut().push_snapshot(last_stable);
                *self.last_stable_card.borrow_mut() = current;
            }
        }
        self.toasts.borrow_mut().show(ui.ctx(), &theme);
    }
}
