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
pub(crate) mod gallery;

use egui::{pos2, vec2, Align, CornerRadius, Layout, Rect, Stroke, StrokeKind};
use egui_macview::{
    theme::{MacAppearance, MacTheme, WindowControlsStyle},
    CommandPaletteItem, MacCommandPalette, MacMenuBar, MacSplitOrientation, MacSplitView, MacToastManager, NSMenu, NSUndoManager,
};
#[allow(unused_imports)]
pub use gallery::{
    convert_card_units, detect_frameless_resize_dir, format_tracked_small_caps, setup_system_or_custom_fonts, unit_scale_and_suffix,
    CardModel, ExpandedProTab, GalleryScreen, InspectorMode,
};
use gallery::{inspector::InspectorState, CenterState, SidebarState};
use std::cell::{Cell, RefCell};

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let renderer = if std::env::var("MACVIEW_RENDERER").as_deref() == Ok("wgpu") { eframe::Renderer::Wgpu } else { eframe::Renderer::Glow };
    let mut wgpu_setup = eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle();
    wgpu_setup.instance_descriptor.flags = eframe::wgpu::InstanceFlags::empty();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1160.0, 740.0])
            .with_min_inner_size([900.0, 540.0])
            .with_decorations(false)
            .with_active(true)
            .with_visible(true)
            .with_title(format!("macview_gallery v{} (AppKit & AVKit Studio Showcase)", egui_macview::VERSION)),
        renderer,
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(wgpu_setup),
            ..Default::default()
        },
        system_font_fallback: false,
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native(
        "macview_gallery",
        options,
        Box::new(|cc| {
            gallery::audio_media::NativeAudioOutput::enable_os_audio(true);
            setup_system_or_custom_fonts(&cc.egui_ctx);
            let boxed_app = std::thread::Builder::new()
                .name("gallery_init".into())
                .stack_size(8 * 1024 * 1024)
                .spawn(|| Box::new(GalleryApp::new()))
                .ok()
                .and_then(|h| h.join().ok())
                .unwrap_or_else(|| Box::new(GalleryApp::new()));
            if let Some(url) = gallery::mobile_web_server::spawn_background_mobile_server() {
                boxed_app.toasts.borrow_mut().info(format!("Mobile/Tablet Browser: {url}"));
            }
            Ok(boxed_app)
        }),
    )
}

pub(crate) struct GalleryApp {
    pub(crate) appearance: MacAppearance,
    pub(crate) window_controls_style: WindowControlsStyle,
    pub(crate) last_window_controls_style: Option<WindowControlsStyle>,
    pub(crate) pending_decoration_resync_frames: u8,
    pub(crate) pending_target_inner_size: Option<egui::Vec2>,
    pub(crate) inspector_mode: InspectorMode,
    pub(crate) last_inspector_mode: InspectorMode,
    pub(crate) active_screen: GalleryScreen,
    pub(crate) is_maximized: bool,
    pub(crate) is_minimized: bool,
    pub(crate) is_window_closed: bool,
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
    pub(crate) mobile_compact_active: bool,
    pub(crate) mobile_pane_idx: usize,
}

impl GalleryApp {
    pub(crate) fn new() -> Self {
        let palette_items = vec![
            CommandPaletteItem::new("edit.undo", "Edit: Undo").category("Edit").shortcut("⌘Z / Ctrl+Z"),
            CommandPaletteItem::new("edit.redo", "Edit: Redo").category("Edit").shortcut("⌘⇧Z / Ctrl+Shift+Z"),
            CommandPaletteItem::new("screen.canvas", "State 1 (v0.2): Vector & Inspector Studio").category("States"),
            CommandPaletteItem::new("screen.vector", "State 2 (v0.3): Symbols & Layout Lab").category("States"),
            CommandPaletteItem::new("screen.tokens", "State 3 (v0.4.1): Command & State Lab").category("States"),
            CommandPaletteItem::new("screen.finder", "State 4 (v0.5): Finder & Publishing Browser").category("States"),
            CommandPaletteItem::new("screen.avkit", "State 5 (v0.5): AVKit Media & Timeline Studio").category("States"),
            CommandPaletteItem::new("screen.advanced", "State 6 (v0.6.8): Advanced & AppKit NIB Controls").category("States"),
            CommandPaletteItem::new("theme.aqua", "Appearance: Switch to Aqua (Light)").category("Theme"),
            CommandPaletteItem::new("theme.anthracite", "Appearance: Switch to Anthracite (Pro Graphite)").category("Theme"),
            CommandPaletteItem::new("theme.dark", "Appearance: Switch to DarkAqua (Dark)").category("Theme"),
            CommandPaletteItem::new("canvas.pulse", "Physics: Trigger CASpring Pulse").category("Motion"),
            CommandPaletteItem::new("matrix.reset", "Canvas: Reset Affine Transform to Identity").category("Canvas"),
        ];
        let default_card = CardModel::default();
        let start_screen = if std::env::var("MACVIEW_OPEN_NIB_SHOWCASE").as_deref() == Ok("1") {
            GalleryScreen::FinderDataBrowser
        } else {
            GalleryScreen::CanvasPhysics
        };
        let sidebar = SidebarState { active_screen: start_screen, selected_node: start_screen.title().to_string(), ..Default::default() };
        Self {
            appearance: MacAppearance::Aqua,
            window_controls_style: WindowControlsStyle::AppleTrafficLights,
            last_window_controls_style: Some(WindowControlsStyle::AppleTrafficLights),
            pending_decoration_resync_frames: 0,
            pending_target_inner_size: None,
            inspector_mode: InspectorMode::ExpandedPro,
            last_inspector_mode: InspectorMode::ExpandedPro,
            active_screen: start_screen,
            is_maximized: false,
            is_minimized: false,
            is_window_closed: false,
            left_split_pos: 265.0,
            inspector_width: 380.0,
            card: RefCell::new(default_card.clone()),
            undo_manager: RefCell::new(NSUndoManager::new()),
            last_stable_card: RefCell::new(default_card),
            is_dragging_card: Cell::new(false),
            drag_start_card: RefCell::new(None),
            active_text_focus: Cell::new(None),
            text_focus_start_card: RefCell::new(None),
            toasts: RefCell::new(MacToastManager::new()),
            sidebar,
            center: CenterState::default(),
            inspector: InspectorState::default(),
            menus: NSMenu::default_menu_bar(),
            open_menu_idx: None,
            highlighted_item_idx: None,
            palette: MacCommandPalette::new(palette_items),
            mobile_compact_active: false,
            mobile_pane_idx: 0,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.render_ui(ui);
    }
}

impl GalleryApp {
    pub(crate) fn render_ui(&mut self, ui: &mut egui::Ui) {
        if let Some(host_maximized) = ui.input(|i| i.viewport().maximized) {
            self.is_maximized = host_maximized;
        }
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

        ui.painter_at(full_rect).rect(
            full_rect,
            CornerRadius::ZERO,
            theme.window_background(),
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );

        let toolbar_height = theme.toolbar_metrics().height_unified.points();
        let toolbar_rect = Rect::from_min_size(full_rect.min, vec2(full_rect.width(), toolbar_height));
        self.render_toolbar(ui, toolbar_rect, &theme, on_resize_border);

        let menu_bar_h = 24.0;
        let menu_bar_rect =
            Rect::from_min_size(pos2(full_rect.min.x, full_rect.min.y + toolbar_height), vec2(full_rect.width(), menu_bar_h));
        let mp = ui.painter_at(menu_bar_rect);
        mp.rect_filled(menu_bar_rect, CornerRadius::ZERO, theme.window_background());
        mp.line_segment(
            [pos2(menu_bar_rect.min.x, menu_bar_rect.max.y), pos2(menu_bar_rect.max.x, menu_bar_rect.max.y)],
            Stroke::new(1.0, theme.separator()),
        );

        let menu_inner_rect = Rect::from_min_max(pos2(menu_bar_rect.min.x + theme.stack_view().default_spacing.points(), menu_bar_rect.min.y), menu_bar_rect.max);
        let mut menu_ui = ui.new_child(
            egui::UiBuilder::new().id_salt("gallery_menu_bar").max_rect(menu_inner_rect).layout(Layout::left_to_right(Align::Center)),
        );
        if let Some(action) =
            MacMenuBar::new(&self.menus, &mut self.open_menu_idx, &mut self.highlighted_item_idx).theme(&theme).show(&mut menu_ui)
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

        let work_rect = Rect::from_min_max(pos2(full_rect.min.x, full_rect.min.y + toolbar_height + menu_bar_h), full_rect.max);
        let mut work_ui =
            ui.new_child(egui::UiBuilder::new().id_salt("gallery_work_area").max_rect(work_rect).layout(Layout::left_to_right(Align::Min)));

        let mut left_split_pos = self.left_split_pos;
        if self.last_inspector_mode != self.inspector_mode {
            self.inspector_width = match self.inspector_mode {
                InspectorMode::DefaultNib => 340.0,
                InspectorMode::CompactHud => 260.0,
                InspectorMode::ExpandedPro => 380.0,
            };
            self.last_inspector_mode = self.inspector_mode;
        }

        let available_body_w = work_rect.width() - left_split_pos;
        let min_canvas_w = 260.0;
        let min_inspector_w = match self.inspector_mode {
            InspectorMode::ExpandedPro => 260.0,
            InspectorMode::DefaultNib => 240.0,
            InspectorMode::CompactHud => 200.0,
        };
        let max_inspector_w = (available_body_w - min_canvas_w).max(min_inspector_w);
        let current_inspector_w = self.inspector_width.clamp(min_inspector_w, max_inspector_w);
        let mut right_split_pos = (available_body_w - current_inspector_w).max(min_canvas_w);

        self.center.sync_all_nib_stores(
            &mut gallery::nib_window_studio::NibViewportSync {
                card_cell: &self.card,
                appearance: &mut self.appearance,
                window_controls_style: &mut self.window_controls_style,
                inspector_mode: &mut self.inspector_mode,
                active_screen: &mut self.active_screen,
                sidebar: &mut self.sidebar,
                inspector: &mut self.inspector,
            },
            &self.toasts,
            !ui.ctx().egui_wants_keyboard_input(),
        );
        self.mobile_compact_active = work_rect.width() < 760.0;
        if self.is_window_closed || self.is_minimized {
            work_ui.vertical_centered(|v_ui| {
                v_ui.spacing_mut().item_spacing.y = theme.stack_view().default_spacing.points();
                let msg = if self.is_window_closed {
                    "macOS AppWindow Closed — Click below to reopen session"
                } else {
                    "Window Minimized to Dock — Click below to restore"
                };
                v_ui.label(egui::RichText::new(msg).size(14.0).color(theme.secondary_label()));
                if v_ui.add(egui_macview::MacButtonCell::new("Reopen AppWindow in Workspace").theme(&theme).selected(true)).clicked() {
                    self.is_window_closed = false;
                    self.is_minimized = false;
                }
            });
        } else if self.mobile_compact_active {
            self.render_mobile_compact_work_area(&mut work_ui, &theme);
        } else if self.inspector_mode == InspectorMode::CompactHud {
            // Compact Mode: collapse both left and right sidebars so the canvas fills 100% of the workspace width
            let mut s = self.active_screen;
            let args = gallery::center_state::CenterRenderArgs {
                card_cell: &self.card,
                undo_manager: &self.undo_manager,
                last_stable_card: &self.last_stable_card,
                inspector_mode: self.inspector_mode,
                snapping_enabled: self.inspector.snapping_enabled,
                unit_segmented_idx: self.inspector.unit_segmented_idx,
                vis_canvas: self.sidebar.vis_canvas,
                lock_canvas: self.sidebar.lock_canvas,
            };
            self.center.render(&mut work_ui, &theme, args, &mut s, &self.toasts);
            if self.active_screen != s {
                (self.active_screen, self.sidebar.active_screen, self.sidebar.selected_node) = (s, s, s.title().to_string());
                self.center.media_playing = s == GalleryScreen::AvKitMediaStudio;
                self.center.is_publishing = false;
                if !self.center.media_playing {
                    gallery::audio_media::NativeAudioOutput::stop();
                }
            }
        } else {
            let (sidebar, center, card, inspector) = (&mut self.sidebar, &mut self.center, &self.card, &mut self.inspector);
            let (inspector_mode_cell, active_screen_cell) = (Cell::new(self.inspector_mode), Cell::new(self.active_screen));
            let (vis_canvas_cell, lock_canvas_cell) = (Cell::new(sidebar.vis_canvas), Cell::new(sidebar.lock_canvas));
            let (snapping_enabled, unit_segmented_idx) = (inspector.snapping_enabled, inspector.unit_segmented_idx);

            MacSplitView::with_pos("gallery_left_split", MacSplitOrientation::Horizontal, &mut left_split_pos)
                .theme(&theme)
                .min_first(180.0)
                .max_first(440.0)
                .min_second(420.0)
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
                        let snippet_ctx = gallery::snippet::SnippetContext::from(&*center);
                        MacSplitView::with_pos("gallery_right_split", MacSplitOrientation::Horizontal, &mut right_split_pos)
                            .theme(&theme)
                            .min_first(min_canvas_w)
                            .min_second(min_inspector_w)
                            .show(
                                body_ui,
                                |center_ui| {
                                    let mut s = active_screen_cell.get();
                                    let args = gallery::center_state::CenterRenderArgs {
                                        card_cell: card,
                                        undo_manager: &self.undo_manager,
                                        last_stable_card: &self.last_stable_card,
                                        inspector_mode: inspector_mode_cell.get(),
                                        snapping_enabled,
                                        unit_segmented_idx,
                                        vis_canvas,
                                        lock_canvas,
                                    };
                                    center.render(center_ui, &theme, args, &mut s, &self.toasts);
                                    active_screen_cell.set(s);
                                },
                                |inspector_ui| {
                                    let mut m = inspector_mode_cell.get();
                                    inspector.render(
                                        inspector_ui,
                                        &theme,
                                        card,
                                        (&mut m, active_screen_cell.get()),
                                        snippet_ctx,
                                        &self.toasts,
                                    );
                                    inspector_mode_cell.set(m);
                                },
                            );
                    },
                );

            self.inspector_mode = inspector_mode_cell.get();
            let new_screen = active_screen_cell.get();
            if self.active_screen != new_screen {
                (self.active_screen, self.sidebar.active_screen, self.sidebar.selected_node) =
                    (new_screen, new_screen, new_screen.title().to_string());
                self.center.media_playing = new_screen == GalleryScreen::AvKitMediaStudio;
                self.center.is_publishing = false;
                if !self.center.media_playing {
                    gallery::audio_media::NativeAudioOutput::stop();
                }
            }
            self.left_split_pos = left_split_pos;
            self.inspector_width = (available_body_w - right_split_pos).clamp(min_inspector_w, max_inspector_w);
        }

        if !ui.input(|i| i.pointer.any_down())
            && !self.is_dragging_card.get()
            && focused_id.is_none()
            && *self.card.borrow() != *self.last_stable_card.borrow()
        {
            let current = self.card.borrow().clone();
            let prev = std::mem::replace(&mut *self.last_stable_card.borrow_mut(), current);
            self.undo_manager.borrow_mut().push_snapshot(prev);
        }
        self.center.sync_and_render_nib_viewport(
            ui.ctx(),
            &theme,
            gallery::nib_window_studio::NibViewportSync {
                card_cell: &self.card,
                appearance: &mut self.appearance,
                window_controls_style: &mut self.window_controls_style,
                inspector_mode: &mut self.inspector_mode,
                active_screen: &mut self.active_screen,
                sidebar: &mut self.sidebar,
                inspector: &mut self.inspector,
            },
            &self.toasts,
        );
        self.toasts.borrow_mut().show(ui.ctx(), &theme);
    }
}
