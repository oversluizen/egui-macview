//! `macview_gallery` — Interactive Desktop Showcase Example.
//!
//! Grounded in Apple macOS AppKit Human Interface Guidelines (HIG)
//! and W3C DTCG Token Catalogues (`tokens/design_tokens.json`).
//!
//! Demonstrates the complete macOS AppKit suite on Windows 11:
//! - Window Framing: Borderless macOS unified window with interactive traffic lights & window dragging.
//! - Dynamic Theme Toggle: Aqua (Light) <-> DarkAqua (Dark).
//! - 3-Tier Inspector Architecture:
//!   - Mode 0: Default NIB Document Inspector (260pt standard property pane).
//!   - Mode 1: Compact Pro HUD Palette (178x63pt floating semi-translucent dark infobox).
//!   - Mode 2: Expanded Pro Inspector (380pt multi-deck tabs: Attributes, Metrics, Advanced Matrix).
//! - Interactive Canvas: Real-time vector card transformed live by Affine Matrix (rotation, scale, skew)
//!   and pulsed by `CASpring` harmonic oscillator motion physics.
//! - Window Layout via `MacSplitView`: Left Sidebar, Center Canvas, Right Inspector.

#![cfg_attr(not(test), windows_subsystem = "windows")]

use egui::{
    pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Pos2, Rect, Sense,
    Stroke, StrokeKind, Ui, ViewportCommand,
};
use egui_macview::{
    setup_symbols_font,
    theme::{MacAppearance, MacTheme, WindowControlsStyle},
    CASpring, CommandPaletteItem, MacBackgroundStyle, MacButtonCell, MacCanvasFooter,
    MacCanvasFooterAction, MacColorWell, MacCommandPalette, MacFixedView, MacForm, MacInputCell,
    MacMenuBar, MacOutlineView, MacPopUpButton, MacPopover, MacPopoverArrowEdge, MacRow,
    MacScrollView, MacSearchField, MacSegmentedControl, MacSlider, MacSplitOrientation,
    MacSplitView, MacStackOrientation, MacStackView, MacStepper, MacSymbol, MacThumbnailCell,
    MacToggle, NSMenu, CANVAS_FOOTER_HEIGHT,
};
use std::cell::RefCell;

#[allow(dead_code)]
/// Formats uppercase section headers with optical tracking (thin-spaces between letters and wider word separation).
fn format_tracked_small_caps(text: &str) -> String {
    text.to_uppercase()
        .split_whitespace()
        .map(|word| {
            word.chars()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("\u{2009}")
        })
        .collect::<Vec<_>>()
        .join("   ")
}

fn setup_system_or_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let font_dir = std::path::Path::new(&local_app_data)
        .join("Microsoft")
        .join("Windows")
        .join("Fonts");

    let sf_pro_path = font_dir.join("SF-Pro-Text-Regular.otf");
    let inter_path = font_dir.join("Inter-Variable.ttf");

    if sf_pro_path.exists() {
        if let Ok(bytes) = std::fs::read(&sf_pro_path) {
            fonts.font_data.insert(
                "sf_pro".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "sf_pro".to_owned());
        }
    } else if inter_path.exists() {
        if let Ok(bytes) = std::fs::read(&inter_path) {
            fonts.font_data.insert(
                "inter".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "inter".to_owned());
        }
    }

    let sf_mono_path = font_dir.join("SF-Mono-Regular.otf");
    let cascadia_mono_path = std::path::Path::new("C:\\Windows\\Fonts").join("CascadiaMono.ttf");
    let cascadia_code_path = std::path::Path::new("C:\\Windows\\Fonts").join("CascadiaCode.ttf");
    let consolas_path = std::path::Path::new("C:\\Windows\\Fonts").join("consola.ttf");

    if sf_mono_path.exists() {
        if let Ok(bytes) = std::fs::read(&sf_mono_path) {
            fonts.font_data.insert(
                "sf_mono".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "sf_mono".to_owned());
        }
    } else if cascadia_mono_path.exists() {
        if let Ok(bytes) = std::fs::read(&cascadia_mono_path) {
            fonts.font_data.insert(
                "cascadia_mono".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "cascadia_mono".to_owned());
        }
    } else if cascadia_code_path.exists() {
        if let Ok(bytes) = std::fs::read(&cascadia_code_path) {
            fonts.font_data.insert(
                "cascadia_code".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "cascadia_code".to_owned());
        }
    } else if consolas_path.exists() {
        if let Ok(bytes) = std::fs::read(&consolas_path) {
            fonts.font_data.insert(
                "consolas".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "consolas".to_owned());
        }
    }

    // Register monochrome vector AppKit symbols font (CupertinoIcons)
    setup_symbols_font(&mut fonts);

    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result<()> {
    let renderer = match std::env::var("MACVIEW_RENDERER").as_deref() {
        Ok("glow") => eframe::Renderer::Glow,
        Ok("wgpu") => eframe::Renderer::Wgpu,
        _ => {
            #[cfg(target_os = "macos")]
            {
                eframe::Renderer::Glow
            }
            #[cfg(not(target_os = "macos"))]
            {
                eframe::Renderer::Wgpu
            }
        }
    };

    #[cfg(target_os = "macos")]
    let decorations = false;
    #[cfg(not(target_os = "macos"))]
    let decorations = true;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 720.0])
            .with_min_inner_size([880.0, 520.0])
            .with_decorations(decorations)
            .with_transparent(false)
            .with_active(true)
            .with_visible(true)
            .with_title(format!(
                "macview_gallery v{} (AppKit macOS 15 Parity)",
                egui_macview::VERSION
            )),
        renderer,
        ..Default::default()
    };
    eframe::run_native(
        "macview_gallery",
        options,
        Box::new(|cc| {
            setup_system_or_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(GalleryApp::new()))
        }),
    )
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

/// Dynamic properties of the interactive Canvas Vector Card,
/// edited live via either Default NIB, Compact HUD, or Expanded Pro Inspector.
#[derive(Clone)]
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

struct SidebarState {
    expanded_components: bool,
    expanded_tokens: bool,
    expanded_inspector: bool,
    selected_node: String,
    vis_canvas: bool,
    lock_canvas: bool,
    vis_vector: bool,
    lock_vector: bool,
    vis_tokens: bool,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            expanded_components: true,
            expanded_tokens: true,
            expanded_inspector: true,
            selected_node: "Interactive Canvas".to_string(),
            vis_canvas: true,
            lock_canvas: false,
            vis_vector: true,
            lock_vector: false,
            vis_tokens: true,
        }
    }
}

struct CenterState {
    search_query: String,
    spring: CASpring,
    spring_start_time: Option<f64>,
    popover_open: bool,
    bounce_spring: CASpring,
    bounce_start_time: Option<f64>,
    hud_pos: Pos2,
    fixed_grid_snap: bool,
    fixed_seg_idx: usize,
    row_popup_idx: usize,
    row_seg_idx: usize,
}

impl Default for CenterState {
    fn default() -> Self {
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
            hud_pos: Pos2::new(30.0, 36.0),
            fixed_grid_snap: true,
            fixed_seg_idx: 0,
            row_popup_idx: 0,
            row_seg_idx: 1,
        }
    }
}

struct InspectorState {
    slider_value: f32,
    stepper_value: f64,
    stepper_wrap: bool,
    popup_selected: usize,
    unit_segmented_idx: usize,
    snapping_enabled: bool,
    expanded_tab: ExpandedProTab,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            slider_value: 65.0,
            stepper_value: 3.5,
            stepper_wrap: true,
            popup_selected: 0,
            unit_segmented_idx: 0,
            snapping_enabled: true,
            expanded_tab: ExpandedProTab::Attributes,
        }
    }
}

struct GalleryApp {
    appearance: MacAppearance,
    window_controls_style: WindowControlsStyle,
    inspector_mode: InspectorMode,
    last_inspector_mode: InspectorMode,
    is_maximized: bool,
    left_split_pos: f32,
    inspector_width: f32,
    card: RefCell<CardModel>,
    sidebar: SidebarState,
    center: CenterState,
    inspector: InspectorState,
    menus: Vec<NSMenu>,
    open_menu_idx: Option<usize>,
    highlighted_item_idx: Option<usize>,
    palette: MacCommandPalette,
}

impl GalleryApp {
    fn new() -> Self {
        let palette_items = vec![
            CommandPaletteItem::new("theme.aqua", "Appearance: Switch to Aqua (Light)")
                .category("Theme")
                .keywords(&["light", "aqua", "theme"]),
            CommandPaletteItem::new("theme.dark", "Appearance: Switch to DarkAqua (Dark)")
                .category("Theme")
                .keywords(&["dark", "darkaqua", "theme"]),
            CommandPaletteItem::new("mode.nib", "Inspector: Default NIB (260pt)")
                .category("Inspector")
                .keywords(&["nib", "inspector", "default"]),
            CommandPaletteItem::new("mode.hud", "Inspector: Compact HUD Palette")
                .category("Inspector")
                .keywords(&["hud", "compact", "floating"]),
            CommandPaletteItem::new("mode.pro", "Inspector: Expanded Pro (380pt)")
                .category("Inspector")
                .keywords(&["pro", "expanded", "matrix"]),
            CommandPaletteItem::new("canvas.pulse", "Physics: Trigger CASpring Pulse")
                .category("Motion")
                .keywords(&["spring", "bounce", "caspring", "physics"]),
            CommandPaletteItem::new("matrix.reset", "Canvas: Reset Affine Transform to Identity")
                .category("Canvas")
                .keywords(&["matrix", "reset", "identity"]),
            CommandPaletteItem::new("zoom.50", "View: Zoom to 50%")
                .category("View")
                .shortcut("Ctrl+-"),
            CommandPaletteItem::new("zoom.100", "View: Zoom to 100% (Actual Size)")
                .category("View")
                .shortcut("Ctrl+0"),
            CommandPaletteItem::new("zoom.200", "View: Zoom to 200%")
                .category("View")
                .shortcut("Ctrl++"),
            CommandPaletteItem::new("help.shortcuts", "Help: Command Palette")
                .category("Help")
                .shortcut("⌘K / Ctrl+K"),
        ];

        Self {
            appearance: MacAppearance::Aqua,
            window_controls_style: WindowControlsStyle::AppleTrafficLights,
            inspector_mode: InspectorMode::ExpandedPro,
            last_inspector_mode: InspectorMode::ExpandedPro,
            is_maximized: false,
            left_split_pos: 240.0,
            inspector_width: 360.0,
            card: RefCell::new(CardModel::default()),
            sidebar: SidebarState::default(),
            center: CenterState::default(),
            inspector: InspectorState::default(),
            menus: NSMenu::default_menu_bar(),
            open_menu_idx: None,
            highlighted_item_idx: None,
            palette: MacCommandPalette::new(palette_items),
        }
    }

    fn handle_menu_action(&mut self, action: &str, ctx: &egui::Context) {
        match action {
            "theme.aqua" => self.appearance = MacAppearance::Aqua,
            "theme.dark" => self.appearance = MacAppearance::DarkAqua,
            "mode.nib" => self.inspector_mode = InspectorMode::DefaultNib,
            "mode.hud" => self.inspector_mode = InspectorMode::CompactHud,
            "mode.pro" => self.inspector_mode = InspectorMode::ExpandedPro,
            "canvas.pulse" => {
                self.center.bounce_start_time = Some(0.0);
            }
            "matrix.reset" => {
                let mut c = self.card.borrow_mut();
                c.rotation_deg = 0.0;
                c.scale_pct = 100.0;
                c.skew_deg = 0.0;
            }
            "zoom.50" => self.card.borrow_mut().scale_pct = 50.0,
            "zoom.100" => self.card.borrow_mut().scale_pct = 100.0,
            "zoom.200" => self.card.borrow_mut().scale_pct = 200.0,
            "view.zoom_in" => {
                let mut c = self.card.borrow_mut();
                c.scale_pct = (c.scale_pct + 25.0).min(400.0);
            }
            "view.zoom_out" => {
                let mut c = self.card.borrow_mut();
                c.scale_pct = (c.scale_pct - 25.0).max(25.0);
            }
            "view.zoom_reset" => self.card.borrow_mut().scale_pct = 100.0,
            "view.toggle_inspector" => {
                self.inspector_mode = match self.inspector_mode {
                    InspectorMode::ExpandedPro => InspectorMode::DefaultNib,
                    _ => InspectorMode::ExpandedPro,
                };
            }
            "view.fullscreen" | "window.zoom" => {
                self.is_maximized = !self.is_maximized;
                ctx.send_viewport_cmd(ViewportCommand::Maximized(self.is_maximized));
            }
            "file.close" => {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            "window.minimize" => {
                ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
            }
            "help.shortcuts" => {
                self.palette.open = true;
            }
            _ => {}
        }
    }
}

impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let card_fill = self.card.borrow().fill_color;
        let theme = MacTheme::new(self.appearance)
            .with_accent(card_fill)
            .with_window_controls_style(self.window_controls_style);
        theme.apply_to_egui(ui.ctx());

        let full_rect = ui.available_rect_before_wrap();
        let window_rounding = CornerRadius::same(10);

        // Fill background with authentic macOS window background & 10pt rounded border
        let bg_color = theme.window_background();
        let painter = ui.painter_at(full_rect);
        painter.rect(
            full_rect,
            window_rounding,
            bg_color,
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );

        // 1. Unified macOS Toolbar Header
        let toolbar_height = theme.toolbar_metrics().height_unified.points();
        let toolbar_rect =
            Rect::from_min_size(full_rect.min, vec2(full_rect.width(), toolbar_height));
        self.render_toolbar(ui, toolbar_rect, &theme);

        // 2. Top Application Menu Bar (SPEC-007 §2)
        let menu_bar_h = 24.0;
        let menu_bar_rect = Rect::from_min_size(
            pos2(full_rect.min.x, full_rect.min.y + toolbar_height),
            vec2(full_rect.width(), menu_bar_h),
        );
        let menu_painter = ui.painter_at(menu_bar_rect);
        menu_painter.rect_filled(menu_bar_rect, CornerRadius::ZERO, theme.window_background());
        menu_painter.line_segment(
            [
                pos2(menu_bar_rect.min.x, menu_bar_rect.max.y),
                pos2(menu_bar_rect.max.x, menu_bar_rect.max.y),
            ],
            Stroke::new(1.0, theme.separator()),
        );

        let mut menu_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(menu_bar_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        menu_ui.add_space(8.0);
        let menu_bar = MacMenuBar::new(
            &self.menus,
            &mut self.open_menu_idx,
            &mut self.highlighted_item_idx,
        )
        .theme(&theme);

        if let Some(action) = menu_bar.show(&mut menu_ui) {
            self.handle_menu_action(&action, ui.ctx());
        }

        // Global Command Palette Shortcut Handling (Cmd+K / Ctrl+K)
        let cmd = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
        if cmd && ui.input(|i| i.key_pressed(egui::Key::K)) {
            self.palette.toggle();
        }

        // Modal Command Palette HUD (Spotlight)
        if let Some(action) = self.palette.show(ui.ctx(), &theme) {
            self.handle_menu_action(&action, ui.ctx());
        }

        // 3. Main Work Area (below toolbar and menu bar) via MacSplitView
        let work_rect = Rect::from_min_max(
            pos2(
                full_rect.min.x,
                full_rect.min.y + toolbar_height + menu_bar_h,
            ),
            pos2(full_rect.max.x, full_rect.max.y),
        );

        let mut work_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(work_rect)
                .layout(Layout::left_to_right(Align::Min)),
        );

        let mut left_split_pos = self.left_split_pos;

        // Auto-adapt target inspector width if inspector mode changed
        let target_inspector_width = match self.inspector_mode {
            InspectorMode::DefaultNib => 320.0,
            InspectorMode::CompactHud => 240.0,
            InspectorMode::ExpandedPro => 360.0,
        };
        if self.last_inspector_mode != self.inspector_mode {
            self.inspector_width = target_inspector_width;
            self.last_inspector_mode = self.inspector_mode;
        }

        let available_body_w = work_rect.width() - left_split_pos;
        let min_canvas_w = 340.0;
        let min_inspector_w = match self.inspector_mode {
            InspectorMode::ExpandedPro => 340.0,
            InspectorMode::DefaultNib => 280.0,
            InspectorMode::CompactHud => 220.0,
        };

        // AppKit NSSplitViewController right-anchoring:
        // The right inspector pane preserves its fixed/preferred width, while the center
        // canvas is elastic and dynamically absorbs 100% of window resize deltas.
        let max_inspector_w = (available_body_w - min_canvas_w).max(min_inspector_w);
        let current_inspector_w = self.inspector_width.clamp(min_inspector_w, max_inspector_w);
        let mut right_split_pos = (available_body_w - current_inspector_w).max(min_canvas_w);

        let sidebar = &mut self.sidebar;
        let center = &mut self.center;
        let card = &self.card;
        let inspector = &mut self.inspector;
        let inspector_mode = self.inspector_mode;

        // Sync sidebar node selection with inspector mode
        if sidebar.selected_node == "Compact HUD Preset"
            && inspector_mode != InspectorMode::CompactHud
        {
            self.inspector_mode = InspectorMode::CompactHud;
        }

        // Primary Split: Left Sidebar vs (Center View + Right Inspector)
        MacSplitView::with_pos(
            "gallery_left_split",
            MacSplitOrientation::Horizontal,
            &mut left_split_pos,
        )
        .theme(&theme)
        .min_first(180.0)
        .max_first(360.0)
        .min_second(620.0)
        .show(
            &mut work_ui,
            |sidebar_ui| {
                sidebar.render(sidebar_ui, &theme, &mut self.inspector_mode);
            },
            |body_ui| {
                // Secondary Split: Center View vs Right Inspector
                MacSplitView::with_pos(
                    "gallery_right_split",
                    MacSplitOrientation::Horizontal,
                    &mut right_split_pos,
                )
                .theme(&theme)
                .min_first(min_canvas_w)
                .min_second(min_inspector_w)
                .show(
                    body_ui,
                    |center_ui| {
                        center.render(center_ui, &theme, card, inspector_mode);
                    },
                    |inspector_ui| {
                        inspector.render(inspector_ui, &theme, card, inspector_mode);
                    },
                );
            },
        );

        self.left_split_pos = left_split_pos;
        self.inspector_width = (available_body_w - right_split_pos).clamp(min_inspector_w, max_inspector_w);
    }
}

impl GalleryApp {
    /// Renders the authentic AppKit unified window toolbar with interactive traffic lights / Windows caption buttons,
    /// window dragging, inspector mode selector, window controls style switcher, and theme toggle.
    fn render_toolbar(&mut self, ui: &mut Ui, rect: Rect, theme: &MacTheme) {
        let painter = ui.painter_at(rect);
        let border_stroke = Stroke::new(1.0, theme.separator());

        // Background with top rounded corners
        let top_corners = CornerRadius {
            nw: 10,
            ne: 10,
            sw: 0,
            se: 0,
        };
        painter.rect_filled(rect, top_corners, theme.control_background());
        painter.line_segment(
            [pos2(rect.min.x, rect.max.y), pos2(rect.max.x, rect.max.y)],
            border_stroke,
        );

        // Window drag interaction on empty toolbar background
        let toolbar_bg_resp = ui.interact(
            rect,
            ui.scope_id().with("mac_toolbar_drag"),
            Sense::click_and_drag(),
        );
        if toolbar_bg_resp.drag_started() {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }
        if toolbar_bg_resp.double_clicked() {
            self.is_maximized = !self.is_maximized;
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Maximized(self.is_maximized));
        }

        let traffic_inset = 18.0;
        let traffic_y = rect.center().y;
        let traffic_radius = theme.toolbar_metrics().traffic_light_radius.points();
        let traffic_spacing = theme.toolbar_metrics().traffic_light_spacing.points();

        let close_pos = pos2(rect.min.x + traffic_inset, traffic_y);
        let min_pos = pos2(
            close_pos.x + traffic_radius * 2.0 + traffic_spacing,
            traffic_y,
        );
        let zoom_pos = pos2(
            min_pos.x + traffic_radius * 2.0 + traffic_spacing,
            traffic_y,
        );

        // Zone 1 (Leading Zone): Apple Traffic Lights (Leading / Left)
        let leading_end_x = if self.window_controls_style == WindowControlsStyle::AppleTrafficLights {
            let (close_fill, close_border) = theme.traffic_light_close();
            let (min_fill, min_border) = theme.traffic_light_minimize();
            let (zoom_fill, zoom_border) = theme.traffic_light_zoom();

            let traffic_cluster_rect = Rect::from_min_max(
                pos2(close_pos.x - traffic_radius - 2.0, rect.min.y),
                pos2(zoom_pos.x + traffic_radius + 6.0, rect.max.y),
            );
            let cluster_hovered = ui.rect_contains_pointer(traffic_cluster_rect);

            // Close button
            let close_rect = Rect::from_center_size(close_pos, vec2(14.0, 14.0));
            let close_resp = ui.interact(
                close_rect,
                ui.scope_id().with("traffic_close"),
                Sense::click(),
            );
            if close_resp.clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            }
            painter.circle(
                close_pos,
                traffic_radius,
                close_fill,
                Stroke::new(0.5, close_border),
            );
            if cluster_hovered {
                let symbol_col = Color32::from_black_alpha(160);
                let d = 2.5;
                painter.line_segment(
                    [
                        pos2(close_pos.x - d, close_pos.y - d),
                        pos2(close_pos.x + d, close_pos.y + d),
                    ],
                    Stroke::new(1.0, symbol_col),
                );
                painter.line_segment(
                    [
                        pos2(close_pos.x - d, close_pos.y + d),
                        pos2(close_pos.x + d, close_pos.y - d),
                    ],
                    Stroke::new(1.0, symbol_col),
                );
            }

            // Minimize button
            let min_rect = Rect::from_center_size(min_pos, vec2(14.0, 14.0));
            let min_resp = ui.interact(min_rect, ui.scope_id().with("traffic_min"), Sense::click());
            if min_resp.clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
            }
            painter.circle(
                min_pos,
                traffic_radius,
                min_fill,
                Stroke::new(0.5, min_border),
            );
            if cluster_hovered {
                let symbol_col = Color32::from_black_alpha(160);
                painter.line_segment(
                    [
                        pos2(min_pos.x - 2.8, min_pos.y),
                        pos2(min_pos.x + 2.8, min_pos.y),
                    ],
                    Stroke::new(1.1, symbol_col),
                );
            }

            // Zoom/Maximize button
            let zoom_rect = Rect::from_center_size(zoom_pos, vec2(14.0, 14.0));
            let zoom_resp = ui.interact(
                zoom_rect,
                ui.scope_id().with("traffic_zoom"),
                Sense::click(),
            );
            if zoom_resp.clicked() {
                self.is_maximized = !self.is_maximized;
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::Maximized(self.is_maximized));
            }
            painter.circle(
                zoom_pos,
                traffic_radius,
                zoom_fill,
                Stroke::new(0.5, zoom_border),
            );
            if cluster_hovered {
                let symbol_col = Color32::from_black_alpha(160);
                painter.line_segment(
                    [
                        pos2(zoom_pos.x - 2.5, zoom_pos.y),
                        pos2(zoom_pos.x + 2.5, zoom_pos.y),
                    ],
                    Stroke::new(1.0, symbol_col),
                );
                painter.line_segment(
                    [
                        pos2(zoom_pos.x, zoom_pos.y - 2.5),
                        pos2(zoom_pos.x, zoom_pos.y + 2.5),
                    ],
                    Stroke::new(1.0, symbol_col),
                );
            }
            zoom_pos.x + traffic_radius + 16.0
        } else {
            rect.min.x + 16.0
        };

        // Zone 3 (Trailing Zone): Windows 11 Caption Buttons (if enabled) + Trailing Controls
        let btn_w = theme.toolbar_metrics().caption_button_width.points();
        let icon_box_size = theme.toolbar_metrics().caption_icon_size.points();
        let half_icon = icon_box_size * 0.5;

        let right_offset = if self.window_controls_style
            == WindowControlsStyle::WindowsCaptionButtons
        {
            // Three contiguous caption buttons flush to the top-right boundary (Fitts's Law):
            // Close: [max.x - 46, max.x]
            let close_rect = Rect::from_min_max(
                pos2(rect.max.x - btn_w, rect.min.y),
                pos2(rect.max.x, rect.max.y),
            );
            // Maximize / Restore: [max.x - 92, max.x - 46]
            let max_rect = Rect::from_min_max(
                pos2(rect.max.x - btn_w * 2.0, rect.min.y),
                pos2(rect.max.x - btn_w, rect.max.y),
            );
            // Minimize: [max.x - 138, max.x - 92]
            let min_rect = Rect::from_min_max(
                pos2(rect.max.x - btn_w * 3.0, rect.min.y),
                pos2(rect.max.x - btn_w * 2.0, rect.max.y),
            );

            // 1. Close Button
            let close_resp = ui.interact(
                close_rect,
                ui.scope_id().with("win11_caption_close"),
                Sense::click(),
            );
            if close_resp.clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            }
            let (close_bg, close_symbol_col) = if close_resp.is_pointer_button_down_on() {
                (theme.windows_caption_close_pressed_bg(), Color32::WHITE)
            } else if close_resp.hovered() {
                (theme.windows_caption_close_hover_bg(), Color32::WHITE)
            } else {
                (Color32::TRANSPARENT, theme.label())
            };
            if close_bg != Color32::TRANSPARENT {
                painter.rect_filled(
                    close_rect,
                    CornerRadius {
                        nw: 0,
                        ne: 10,
                        sw: 0,
                        se: 0,
                    },
                    close_bg,
                );
            }
            let c_center = close_rect.center();
            painter.line_segment(
                [
                    pos2(c_center.x - half_icon, c_center.y - half_icon),
                    pos2(c_center.x + half_icon, c_center.y + half_icon),
                ],
                Stroke::new(1.0, close_symbol_col),
            );
            painter.line_segment(
                [
                    pos2(c_center.x - half_icon, c_center.y + half_icon),
                    pos2(c_center.x + half_icon, c_center.y - half_icon),
                ],
                Stroke::new(1.0, close_symbol_col),
            );

            // 2. Maximize / Restore Button
            let max_resp = ui.interact(
                max_rect,
                ui.scope_id().with("win11_caption_max"),
                Sense::click(),
            );
            if max_resp.clicked() {
                self.is_maximized = !self.is_maximized;
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::Maximized(self.is_maximized));
            }
            let (max_bg, max_symbol_col) = if max_resp.is_pointer_button_down_on() {
                (theme.windows_caption_pressed_bg(), theme.label())
            } else if max_resp.hovered() {
                (theme.windows_caption_hover_bg(), theme.label())
            } else {
                (Color32::TRANSPARENT, theme.label())
            };
            if max_bg != Color32::TRANSPARENT {
                painter.rect_filled(max_rect, CornerRadius::ZERO, max_bg);
            }
            let m_center = max_rect.center();
            if self.is_maximized {
                // Restore icon (dual nested squares)
                let back_sq_min = pos2(m_center.x - 3.0, m_center.y - 5.0);
                let back_sq_max = pos2(m_center.x + 5.0, m_center.y + 3.0);
                painter.line_segment(
                    [back_sq_min, pos2(back_sq_max.x, back_sq_min.y)],
                    Stroke::new(1.0, max_symbol_col),
                );
                painter.line_segment(
                    [pos2(back_sq_max.x, back_sq_min.y), back_sq_max],
                    Stroke::new(1.0, max_symbol_col),
                );
                painter.line_segment(
                    [pos2(back_sq_min.x + 2.0, back_sq_max.y), back_sq_max],
                    Stroke::new(1.0, max_symbol_col),
                );
                painter.line_segment(
                    [back_sq_min, pos2(back_sq_min.x, back_sq_min.y + 2.0)],
                    Stroke::new(1.0, max_symbol_col),
                );
                let front_sq = Rect::from_min_max(
                    pos2(m_center.x - 5.0, m_center.y - 3.0),
                    pos2(m_center.x + 3.0, m_center.y + 5.0),
                );
                painter.rect_stroke(
                    front_sq,
                    CornerRadius::ZERO,
                    Stroke::new(1.0, max_symbol_col),
                    StrokeKind::Inside,
                );
            } else {
                let box_rect = Rect::from_center_size(m_center, vec2(icon_box_size, icon_box_size));
                painter.rect_stroke(
                    box_rect,
                    CornerRadius::ZERO,
                    Stroke::new(1.0, max_symbol_col),
                    StrokeKind::Inside,
                );
            }

            // 3. Minimize Button
            let min_resp = ui.interact(
                min_rect,
                ui.scope_id().with("win11_caption_min"),
                Sense::click(),
            );
            if min_resp.clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
            }
            let (min_bg, min_symbol_col) = if min_resp.is_pointer_button_down_on() {
                (theme.windows_caption_pressed_bg(), theme.label())
            } else if min_resp.hovered() {
                (theme.windows_caption_hover_bg(), theme.label())
            } else {
                (Color32::TRANSPARENT, theme.label())
            };
            if min_bg != Color32::TRANSPARENT {
                painter.rect_filled(min_rect, CornerRadius::ZERO, min_bg);
            }
            let min_center = min_rect.center();
            painter.line_segment(
                [
                    pos2(min_center.x - half_icon, min_center.y),
                    pos2(min_center.x + half_icon, min_center.y),
                ],
                Stroke::new(1.0, min_symbol_col),
            );

            btn_w * 3.0 + 12.0
        } else {
            16.0
        };

        // Trailing Controls via Layout::right_to_left
        let right_area = Rect::from_min_max(
            pos2(leading_end_x + 8.0, rect.min.y),
            pos2(rect.max.x - right_offset, rect.max.y),
        );

        let mut trailing_start_x = rect.max.x - right_offset;
        if right_area.width() > 0.0 {
            let mut right_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(right_area)
                    .layout(Layout::right_to_left(Align::Center)),
            );

            // 1. Dynamic Theme Toggle [ Aqua | DarkAqua ]
            let current_theme_idx = match self.appearance {
                MacAppearance::Aqua => 0,
                MacAppearance::DarkAqua => 1,
            };
            MacSegmentedControl::new(&["Aqua", "DarkAqua"], current_theme_idx)
                .theme(theme)
                .height(24.0)
                .show(&mut right_ui, |idx| {
                    self.appearance = match idx {
                        0 => MacAppearance::Aqua,
                        _ => MacAppearance::DarkAqua,
                    };
                });

            right_ui.add_space(8.0);

            // 2. Strategy 2 Configurable Window Controls Style Switcher [ Apple | Windows | Native ]
            let current_style_idx = match self.window_controls_style {
                WindowControlsStyle::AppleTrafficLights => 0,
                WindowControlsStyle::WindowsCaptionButtons => 1,
                WindowControlsStyle::NativeHost => 2,
            };
            MacSegmentedControl::new(&["Apple", "Windows", "Native"], current_style_idx)
                .theme(theme)
                .height(24.0)
                .show(&mut right_ui, |idx| {
                    self.window_controls_style = match idx {
                        0 => WindowControlsStyle::AppleTrafficLights,
                        1 => WindowControlsStyle::WindowsCaptionButtons,
                        _ => WindowControlsStyle::NativeHost,
                    };
                });

            right_ui.add_space(8.0);

            // 3. Center Toolbar: Inspector Mode Switcher [ Default NIB | Compact HUD | Expanded Pro ]
            let current_mode_idx = match self.inspector_mode {
                InspectorMode::DefaultNib => 0,
                InspectorMode::CompactHud => 1,
                InspectorMode::ExpandedPro => 2,
            };
            MacSegmentedControl::new(
                &["Default NIB", "Compact HUD", "Expanded Pro"],
                current_mode_idx,
            )
            .theme(theme)
            .height(24.0)
            .show(&mut right_ui, |idx| {
                self.inspector_mode = match idx {
                    0 => InspectorMode::DefaultNib,
                    1 => InspectorMode::CompactHud,
                    _ => InspectorMode::ExpandedPro,
                };
            });

            right_ui.add_space(8.0);

            // 4. Spotlight Command Palette Button [ Search ⌘K ]
            let search_label = format!("{} ⌘K", MacSymbol::Search.as_str());
            let spot_btn = right_ui.add(
                MacButtonCell::new(&search_label)
                    .theme(theme)
                    .height(24.0)
                    .selected(self.palette.open),
            );
            if spot_btn.clicked() {
                self.palette.toggle();
            }

            if right_ui.min_rect().is_positive() {
                trailing_start_x = right_ui.min_rect().left();
            }
        }

        // Zone 2 (Center Zone): Window Title (AppKit 3-zone flex layout with dynamic centering & truncation)
        let title_min_x = leading_end_x + 8.0;
        let title_max_x = trailing_start_x - 12.0;
        let available_title_w = title_max_x - title_min_x;

        if available_title_w >= 40.0 {
            let title_font = FontId::proportional(theme.text_field().regular.font_size.points());
            let title_color = theme.label();
            let title_str = format!(
                "macview_gallery v{} (AppKit macOS 15 Parity)",
                egui_macview::VERSION
            );

            let full_galley = painter.layout_no_wrap(title_str.clone(), title_font.clone(), title_color);
            let full_w = full_galley.size().x;

            let center_x = rect.center().x;
            let centered_min_x = center_x - full_w * 0.5;
            let centered_max_x = center_x + full_w * 0.5;

            // Center if space allows without colliding with leading or trailing controls;
            // otherwise leading-align or gracefully truncate with an ellipsis.
            if full_w <= available_title_w && centered_min_x >= title_min_x && centered_max_x <= title_max_x {
                let pos = pos2(centered_min_x, traffic_y - full_galley.size().y * 0.5);
                painter.galley(pos, full_galley, title_color);
            } else if full_w <= available_title_w {
                let pos = pos2(title_min_x, traffic_y - full_galley.size().y * 0.5);
                painter.galley(pos, full_galley, title_color);
            } else {
                let format = egui::TextFormat {
                    font_id: title_font,
                    color: title_color,
                    ..Default::default()
                };
                let mut job = egui::text::LayoutJob::single_section(title_str, format);
                job.wrap.max_width = available_title_w;
                job.wrap.max_rows = 1;
                job.wrap.break_anywhere = false;
                let truncated_galley = painter.layout_job(job);
                let pos = pos2(title_min_x, traffic_y - truncated_galley.size().y * 0.5);
                painter.galley(pos, truncated_galley, title_color);
            }
        }
    }
}

impl SidebarState {
    /// Renders the Left Sidebar with hierarchical `MacOutlineView`, `MacThumbnailCell` vector drawings,
    /// and `accessory_custom` buttons.
    fn render(&mut self, ui: &mut Ui, theme: &MacTheme, inspector_mode_out: &mut InspectorMode) {
        let avail_rect = ui.available_rect_before_wrap();
        ui.painter_at(avail_rect).rect_filled(
            avail_rect,
            CornerRadius::ZERO,
            theme.window_background(),
        );

        MacOutlineView::new("gallery_sidebar_outline")
            .theme(theme)
            .show(ui, |tree| {
                // Section 1: Components Hierarchy
                let resp_comp = tree.row(
                    "node_comp_root",
                    0,
                    true,
                    &mut self.expanded_components,
                    false,
                    |row| {
                        row.header(&format_tracked_small_caps("Components"));
                    },
                );
                if resp_comp.clicked {
                    self.expanded_components = !self.expanded_components;
                }

                if self.expanded_components {
                    // Child 1.1: Interactive Canvas
                    let sel_canvas = self.selected_node == "Interactive Canvas";
                    let resp = tree.row("node_canvas", 1, false, &mut false, sel_canvas, |row| {
                        let bg_style = theme.selected_control_background_style();
                        row.thumbnail(|painter, rect| {
                            let icon_color = if sel_canvas {
                                bg_style.primary_text_color(theme)
                            } else {
                                theme.control_accent()
                            };
                            // Vector Artboard / Canvas Icon
                            painter.rect_stroke(
                                rect.shrink(2.5),
                                CornerRadius::same(2),
                                Stroke::new(1.2, icon_color),
                                StrokeKind::Inside,
                            );
                            painter.rect_filled(
                                rect.shrink(5.0),
                                CornerRadius::same(1),
                                icon_color,
                            );
                        });
                        row.label("Interactive Canvas");
                        row.accessory_custom(
                            "vis_canvas",
                            &mut self.vis_canvas,
                            |painter, rect, active| {
                                let color = if active {
                                    if sel_canvas {
                                        bg_style.primary_text_color(theme)
                                    } else {
                                        theme.control_accent()
                                    }
                                } else {
                                    if sel_canvas {
                                        bg_style.secondary_text_color(theme)
                                    } else {
                                        theme.placeholder_text()
                                    }
                                };
                                painter.circle_filled(rect.center(), 3.5, color);
                            },
                        );
                        row.accessory_custom(
                            "lock_canvas",
                            &mut self.lock_canvas,
                            |painter, rect, active| {
                                let color = if active {
                                    if sel_canvas {
                                        bg_style.primary_text_color(theme)
                                    } else {
                                        theme.selected_control()
                                    }
                                } else {
                                    if sel_canvas {
                                        bg_style.secondary_text_color(theme)
                                    } else {
                                        theme.placeholder_text()
                                    }
                                };
                                painter.rect_filled(rect.shrink(4.0), CornerRadius::same(1), color);
                            },
                        );
                    });
                    if resp.is_selected {
                        self.selected_node = "Interactive Canvas".to_string();
                    }

                    // Child 1.2: Vector Assets
                    let sel_vector = self.selected_node == "Vector Assets";
                    let resp = tree.row("node_vector", 1, false, &mut false, sel_vector, |row| {
                        let bg_style = theme.selected_control_background_style();
                        row.thumbnail(|painter, rect| {
                            let icon_color = if sel_vector {
                                bg_style.primary_text_color(theme)
                            } else {
                                theme.system_green()
                            };
                            painter.line_segment(
                                [
                                    pos2(rect.min.x + 3.0, rect.max.y - 3.0),
                                    pos2(rect.max.x - 3.0, rect.min.y + 3.0),
                                ],
                                Stroke::new(1.5, icon_color),
                            );
                        });
                        row.label("Vector Assets");
                        row.accessory_custom(
                            "vis_vector",
                            &mut self.vis_vector,
                            |painter, rect, active| {
                                let color = if active {
                                    if sel_vector {
                                        bg_style.primary_text_color(theme)
                                    } else {
                                        theme.system_green()
                                    }
                                } else {
                                    if sel_vector {
                                        bg_style.secondary_text_color(theme)
                                    } else {
                                        theme.placeholder_text()
                                    }
                                };
                                painter.circle_filled(rect.center(), 3.5, color);
                            },
                        );
                        row.accessory_custom(
                            "lock_vector",
                            &mut self.lock_vector,
                            |painter, rect, active| {
                                let color = if active {
                                    if sel_vector {
                                        bg_style.primary_text_color(theme)
                                    } else {
                                        theme.selected_control()
                                    }
                                } else {
                                    if sel_vector {
                                        bg_style.secondary_text_color(theme)
                                    } else {
                                        theme.placeholder_text()
                                    }
                                };
                                painter.rect_filled(rect.shrink(4.0), CornerRadius::same(1), color);
                            },
                        );
                    });
                    if resp.is_selected {
                        self.selected_node = "Vector Assets".to_string();
                    }
                }

                // Section 2: Tokens
                let resp_tokens = tree.row(
                    "node_tokens_root",
                    0,
                    true,
                    &mut self.expanded_tokens,
                    false,
                    |row| {
                        row.header(&format_tracked_small_caps("Design Tokens"));
                    },
                );
                if resp_tokens.clicked {
                    self.expanded_tokens = !self.expanded_tokens;
                }

                if self.expanded_tokens {
                    let sel_tokens = self.selected_node == "W3C DTCG Tokens";
                    let resp = tree.row("node_dtcg", 1, false, &mut false, sel_tokens, |row| {
                        let bg_style = theme.selected_control_background_style();
                        row.thumbnail(|painter, rect| {
                            let icon_color = if sel_tokens {
                                bg_style.primary_text_color(theme)
                            } else {
                                theme.selected_control()
                            };
                            painter.circle_filled(rect.center(), 6.0, icon_color);
                        });
                        row.label("W3C DTCG Tokens");
                        row.accessory_custom(
                            "vis_tokens",
                            &mut self.vis_tokens,
                            |painter, rect, active| {
                                let color = if active {
                                    if sel_tokens {
                                        bg_style.primary_text_color(theme)
                                    } else {
                                        theme.control_accent()
                                    }
                                } else {
                                    if sel_tokens {
                                        bg_style.secondary_text_color(theme)
                                    } else {
                                        theme.placeholder_text()
                                    }
                                };
                                painter.circle_filled(rect.center(), 3.5, color);
                            },
                        );
                    });
                    if resp.is_selected {
                        self.selected_node = "W3C DTCG Tokens".to_string();
                    }
                }

                // Section 3: Inspector Presets
                let resp_insp = tree.row(
                    "node_inspector_root",
                    0,
                    true,
                    &mut self.expanded_inspector,
                    false,
                    |row| {
                        row.header(&format_tracked_small_caps("Inspector Presets"));
                    },
                );
                if resp_insp.clicked {
                    self.expanded_inspector = !self.expanded_inspector;
                }

                if self.expanded_inspector {
                    // Mode 0: Default NIB
                    let sel_nib = self.selected_node == "Default NIB Inspector";
                    let resp_nib = tree.row("node_nib", 1, false, &mut false, sel_nib, |row| {
                        let bg_style = theme.selected_control_background_style();
                        row.thumbnail(|painter, rect| {
                            let icon_color = if sel_nib {
                                bg_style.primary_text_color(theme)
                            } else {
                                theme.control_accent()
                            };
                            painter.rect_stroke(
                                rect.shrink(3.0),
                                CornerRadius::same(2),
                                Stroke::new(1.0, icon_color),
                                StrokeKind::Inside,
                            );
                        });
                        row.label("Default NIB (260pt)");
                    });
                    if resp_nib.is_selected {
                        self.selected_node = "Default NIB Inspector".to_string();
                        *inspector_mode_out = InspectorMode::DefaultNib;
                    }

                    // Mode 1: Compact HUD
                    let sel_hud = self.selected_node == "Compact HUD Preset";
                    let resp_hud = tree.row("node_hud", 1, false, &mut false, sel_hud, |row| {
                        let bg_style = theme.selected_control_background_style();
                        row.thumbnail(|painter, rect| {
                            let icon_color = if sel_hud {
                                bg_style.primary_text_color(theme)
                            } else {
                                Color32::from_black_alpha(200)
                            };
                            painter.rect_filled(
                                rect.shrink(3.0),
                                CornerRadius::same(2),
                                icon_color,
                            );
                        });
                        row.label("Compact HUD (178x63)");
                    });
                    if resp_hud.is_selected {
                        self.selected_node = "Compact HUD Preset".to_string();
                        *inspector_mode_out = InspectorMode::CompactHud;
                    }

                    // Mode 2: Expanded Pro
                    let sel_exp = self.selected_node == "Expanded Pro Preset";
                    let resp_exp = tree.row("node_exp", 1, false, &mut false, sel_exp, |row| {
                        let bg_style = theme.selected_control_background_style();
                        row.thumbnail(|painter, rect| {
                            let icon_color = if sel_exp {
                                bg_style.primary_text_color(theme)
                            } else {
                                theme.selected_control()
                            };
                            painter.rect_filled(
                                rect.shrink(3.0),
                                CornerRadius::same(2),
                                icon_color,
                            );
                        });
                        row.label("Expanded Pro (380pt)");
                    });
                    if resp_exp.is_selected {
                        self.selected_node = "Expanded Pro Preset".to_string();
                        *inspector_mode_out = InspectorMode::ExpandedPro;
                    }
                }
            });
    }
}

impl CenterState {
    /// Renders the Center View containing Search, Live Interactive Vector Card with real-time Affine Transform &
    /// CASpring bounce physics, the floating Compact HUD palette, and AppKit layout reference showcases.
    fn render(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        card_cell: &RefCell<CardModel>,
        inspector_mode: InspectorMode,
    ) {
        let mut card = card_cell.borrow_mut();
        let avail_rect = ui.available_rect_before_wrap();
        ui.painter_at(avail_rect).rect_filled(
            avail_rect,
            CornerRadius::ZERO,
            theme.control_background(),
        );

        let padding = 16.0;

        // Top Search Bar
        let search_h = 24.0;
        let search_rect = Rect::from_min_size(
            pos2(avail_rect.min.x + padding, avail_rect.min.y + padding),
            vec2(avail_rect.width() - padding * 2.0, search_h),
        );

        let mut search_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(search_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );

        MacSearchField::new(&mut self.search_query)
            .theme(theme)
            .placeholder("Filter components, tokens, Affine transforms, physics...")
            .width(search_rect.width())
            .corner_radius(6.0)
            .show(&mut search_ui);

        // Mount Canvas Footer (SPEC-007 §3) at Canvas Bottom
        let footer_h = CANVAS_FOOTER_HEIGHT; // 24.0
        let footer_rect = Rect::from_min_max(
            pos2(avail_rect.min.x, avail_rect.max.y - footer_h),
            avail_rect.max,
        );

        let mut footer_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(footer_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        let breadcrumbs = ["macview_gallery ▸ Canvas ▸ Hero Vector Card"];
        let cursor_pos = ui
            .input(|i| i.pointer.hover_pos())
            .map(|p| pos2(p.x - avail_rect.min.x, p.y - avail_rect.min.y));
        let current_zoom = card.scale_pct / 100.0;
        let footer_action = MacCanvasFooter::new(&breadcrumbs, cursor_pos, current_zoom)
            .theme(theme)
            .height(footer_h)
            .show(&mut footer_ui, footer_rect.width());

        if let Some(MacCanvasFooterAction::SelectZoom(z)) = footer_action {
            card.scale_pct = z * 100.0;
        }

        // Scrollable Canvas Area via MacScrollView (above footer)
        let scroll_top = search_rect.max.y + 12.0;
        let scroll_rect = Rect::from_min_max(
            pos2(avail_rect.min.x, scroll_top),
            pos2(avail_rect.max.x, avail_rect.max.y - footer_h),
        );

        let mut scroll_container_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(scroll_rect)
                .layout(Layout::top_down(Align::Min)),
        );

        MacScrollView::new("center_canvas_scroll")
            .theme(theme)
            .vertical(true)
            .horizontal(false)
            .max_height(scroll_rect.height())
            .max_width(scroll_rect.width())
            .show(&mut scroll_container_ui, |content_ui| {
                content_ui.add_space(4.0);

                // ====================================================================
                // 1. Interactive AppKit Vector Card Canvas (Live Real-Time Feedback)
                // ====================================================================
                let label_font = FontId::proportional(12.0);
                content_ui.label(
                    egui::RichText::new("1. Interactive Canvas — Live AppKit Vector Card")
                        .font(label_font.clone())
                        .strong(),
                );
                content_ui.label(
                    egui::RichText::new(
                        "Transforms live with the Inspector (Width, Height, Affine 2D Rotation, Scale, Color, CASpring Physics).",
                    )
                    .size(11.0),
                );
                content_ui.add_space(6.0);

                let canvas_w = (content_ui.available_width() - padding * 2.0).max(340.0);
                let canvas_h = 240.0;
                let (canvas_rect, _) = content_ui.allocate_exact_size(
                    vec2(canvas_w, canvas_h),
                    Sense::hover(),
                );

                let cp = content_ui.painter_at(canvas_rect);

                // Canvas backdrop grid
                cp.rect(
                    canvas_rect,
                    CornerRadius::same(8),
                    theme.window_background(),
                    Stroke::new(1.0, theme.separator()),
                    StrokeKind::Inside,
                );

                // Subtle grid lines
                let grid_step = 24.0;
                let grid_stroke = Stroke::new(0.5, theme.separator().gamma_multiply(0.4));
                let mut gx = canvas_rect.min.x + grid_step;
                while gx < canvas_rect.max.x {
                    cp.line_segment([pos2(gx, canvas_rect.min.y), pos2(gx, canvas_rect.max.y)], grid_stroke);
                    gx += grid_step;
                }
                let mut gy = canvas_rect.min.y + grid_step;
                while gy < canvas_rect.max.y {
                    cp.line_segment([pos2(canvas_rect.min.x, gy), pos2(canvas_rect.max.x, gy)], grid_stroke);
                    gy += grid_step;
                }

                // Calculate Live Card Geometry & Affine Transform
                let card_w: f32 = card.width.parse::<f32>().unwrap_or(240.0).clamp(60.0, 500.0);
                let card_h: f32 = card.height.parse::<f32>().unwrap_or(140.0).clamp(40.0, 300.0);
                let card_radius: f32 = card.radius.parse::<f32>().unwrap_or(10.0).clamp(0.0, 40.0);

                // CASpring bounce computation
                let cur_time = content_ui.input(|i| i.time);
                let bounce_scale = if let Some(start_t) = self.bounce_start_time {
                    let elapsed = (cur_time - start_t) as f32;
                    let dur = self.bounce_spring.settling_duration();
                    if elapsed < dur {
                        content_ui.ctx().request_repaint();
                        let p = self.bounce_spring.evaluate(elapsed);
                        // Harmonic overshoot pulse
                        1.0 + 0.20 * (1.0 - p).abs() * (elapsed * 12.0).sin()
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };

                let theta = card.rotation_deg.to_radians();
                let scale_factor = (card.scale_pct / 100.0) * bounce_scale;
                let skew_val = card.skew_deg.to_radians().tan();

                // Affine 2x2 Matrix components:
                let a = scale_factor * theta.cos();
                let b = -scale_factor * theta.sin() + skew_val;
                let c = scale_factor * theta.sin();
                let d = scale_factor * theta.cos();

                let card_center = canvas_rect.center();

                // 4 corners in local space relative to center
                let hw = card_w * 0.5;
                let hh = card_h * 0.5;
                let corners_local = [
                    pos2(-hw, -hh),
                    pos2(hw, -hh),
                    pos2(hw, hh),
                    pos2(-hw, hh),
                ];

                let corners_world: Vec<Pos2> = corners_local
                    .iter()
                    .map(|pt| {
                        let tx = a * pt.x + b * pt.y;
                        let ty = c * pt.x + d * pt.y;
                        pos2(card_center.x + tx, card_center.y + ty)
                    })
                    .collect();

                // Fill color with opacity
                let fill_alpha = (card.opacity / 100.0).clamp(0.0, 1.0);
                let card_fill = card.fill_color.gamma_multiply(fill_alpha);

                if (theta.abs() < 1e-4) && (skew_val.abs() < 1e-4) {
                    // Axis-aligned: render with smooth rounded rect & ambient shadow
                    let rect_w = card_w * scale_factor;
                    let rect_h = card_h * scale_factor;
                    let card_rect = Rect::from_center_size(card_center, vec2(rect_w, rect_h));

                    if card.shadow_enabled {
                        let shadow_rect = card_rect.translate(vec2(0.0, 4.0));
                        cp.rect_filled(
                            shadow_rect,
                            CornerRadius::same(card_radius as u8 + 2),
                            Color32::from_black_alpha(40),
                        );
                    }

                    cp.rect(
                        card_rect,
                        CornerRadius::same(card_radius as u8),
                        card_fill,
                        Stroke::new(1.5, theme.control_accent()),
                        StrokeKind::Inside,
                    );

                    // Content inside card with MacBackgroundStyle contrast guarantee
                    let card_bg_style = MacBackgroundStyle::from_color(card_fill);
                    let text_primary = card_bg_style.primary_text_color(theme);
                    let text_secondary = card_bg_style.secondary_text_color(theme);

                    let card_p = content_ui.painter_at(card_rect);
                    card_p.text(
                        pos2(card_rect.center().x, card_rect.center().y - 12.0),
                        Align2::CENTER_CENTER,
                        &card.layer_name,
                        FontId::proportional(13.0),
                        text_primary,
                    );
                    card_p.text(
                        pos2(card_rect.center().x, card_rect.center().y + 12.0),
                        Align2::CENTER_CENTER,
                        format!("{:.0} × {:.0} pt | Scale: {:.0}%", card_w, card_h, card.scale_pct),
                        FontId::proportional(11.0),
                        text_secondary,
                    );
                } else {
                    // Affine transformed: render polygon
                    if card.shadow_enabled {
                        let shadow_pts: Vec<Pos2> = corners_world
                            .iter()
                            .map(|pt| pos2(pt.x, pt.y + 4.0))
                            .collect();
                        cp.add(egui::epaint::Shape::convex_polygon(
                            shadow_pts,
                            Color32::from_black_alpha(40),
                            Stroke::NONE,
                        ));
                    }

                    cp.add(egui::epaint::Shape::convex_polygon(
                        corners_world.clone(),
                        card_fill,
                        Stroke::new(1.5, theme.control_accent()),
                    ));

                    // Center label with MacBackgroundStyle contrast guarantee
                    let card_bg_style = MacBackgroundStyle::from_color(card_fill);
                    let text_primary = card_bg_style.primary_text_color(theme);
                    let text_secondary = card_bg_style.secondary_text_color(theme);

                    cp.text(
                        pos2(card_center.x, card_center.y - 10.0),
                        Align2::CENTER_CENTER,
                        &card.layer_name,
                        FontId::proportional(13.0),
                        text_primary,
                    );
                    cp.text(
                        pos2(card_center.x, card_center.y + 10.0),
                        Align2::CENTER_CENTER,
                        format!("θ: {:.1}° | S: {:.0}%", card.rotation_deg, card.scale_pct),
                        FontId::proportional(11.0),
                        text_secondary,
                    );
                }

                // Interactive Bounce Trigger Button
                content_ui.add_space(8.0);
                content_ui.horizontal(|h| {
                    let bounce_btn = h.add(
                        MacButtonCell::new("Trigger CASpring Pulse (Damped Harmonic Oscillator)")
                            .theme(theme)
                            .selected(self.bounce_start_time.is_some()),
                    );
                    if bounce_btn.clicked() {
                        self.bounce_start_time = Some(h.input(|i| i.time));
                    }
                    h.label(egui::RichText::new("Perceptual: T₀=0.35s, ζ=0.65").size(10.5));
                });

                // ====================================================================
                // Floating Compact HUD Palette (Mode 1 or Overlay)
                // ====================================================================
                if inspector_mode == InspectorMode::CompactHud {
                    let hud_w = 178.0;
                    let hud_h = 63.0;

                    let hud_min = pos2(canvas_rect.min.x + self.hud_pos.x, canvas_rect.min.y + self.hud_pos.y);
                    let hud_rect = Rect::from_min_size(hud_min, vec2(hud_w, hud_h));

                    // Draw floating HUD window with NSHUDWindowMask dark translucent aesthetic
                    let hud_painter = content_ui.painter_at(hud_rect);

                    // HUD drop shadow
                    hud_painter.rect_filled(
                        hud_rect.translate(vec2(0.0, 3.0)),
                        CornerRadius::same(8),
                        Color32::from_black_alpha(90),
                    );
                    // HUD body (85% translucent dark background)
                    hud_painter.rect(
                        hud_rect,
                        CornerRadius::same(6),
                        Color32::from_rgba_premultiplied(26, 26, 28, 220),
                        Stroke::new(1.0, Color32::from_rgb(60, 60, 62)),
                        StrokeKind::Inside,
                    );

                    // HUD Header Bar (22pt)
                    let header_h = 22.0;
                    let header_rect = Rect::from_min_size(hud_rect.min, vec2(hud_w, header_h));
                    hud_painter.line_segment(
                        [pos2(hud_rect.min.x, hud_rect.min.y + header_h), pos2(hud_rect.max.x, hud_rect.min.y + header_h)],
                        Stroke::new(1.0, Color32::from_rgb(48, 48, 50)),
                    );

                    // Header drag interaction
                    let header_resp = content_ui.interact(
                        header_rect,
                        content_ui.scope_id().with("hud_header_drag"),
                        Sense::click_and_drag(),
                    );
                    if header_resp.dragged() {
                        self.hud_pos.x = (self.hud_pos.x + header_resp.drag_delta().x).clamp(0.0, canvas_w - hud_w);
                        self.hud_pos.y = (self.hud_pos.y + header_resp.drag_delta().y).clamp(0.0, canvas_h - hud_h);
                    }

                    // Header dot & title
                    hud_painter.circle_filled(pos2(hud_rect.min.x + 10.0, hud_rect.min.y + 11.0), 3.0, Color32::from_rgb(180, 180, 184));
                    hud_painter.text(
                        pos2(hud_rect.min.x + 20.0, hud_rect.min.y + 11.0),
                        Align2::LEFT_CENTER,
                        "HUD Palette",
                        FontId::proportional(10.5),
                        Color32::WHITE,
                    );

                    // HUD Compact Inputs (two rows of 18pt compact inputs)
                    let row1_y = hud_rect.min.y + header_h + 3.0;
                    let row1_rect = Rect::from_min_size(pos2(hud_rect.min.x + 6.0, row1_y), vec2(hud_w - 12.0, 18.0));
                    let mut row1_ui = content_ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(row1_rect)
                            .layout(Layout::left_to_right(Align::Center)),
                    );
                    row1_ui.label(egui::RichText::new("W:").size(10.0).color(Color32::from_rgb(180, 180, 184)));
                    MacInputCell::with_id(Id::NULL.with("hud_w"), &mut card.width)
                        .theme(theme)
                        .width(42.0)
                        .show(&mut row1_ui);
                    row1_ui.add_space(4.0);
                    row1_ui.label(egui::RichText::new("H:").size(10.0).color(Color32::from_rgb(180, 180, 184)));
                    MacInputCell::with_id(Id::NULL.with("hud_h"), &mut card.height)
                        .theme(theme)
                        .width(42.0)
                        .show(&mut row1_ui);

                    let row2_y = row1_y + 18.0 + 1.0;
                    let row2_rect = Rect::from_min_size(pos2(hud_rect.min.x + 6.0, row2_y), vec2(hud_w - 12.0, 18.0));
                    let mut row2_ui = content_ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(row2_rect)
                            .layout(Layout::left_to_right(Align::Center)),
                    );
                    row2_ui.label(egui::RichText::new("R:").size(10.0).color(Color32::from_rgb(180, 180, 184)));
                    MacInputCell::with_id(Id::NULL.with("hud_r"), &mut card.radius)
                        .theme(theme)
                        .width(42.0)
                        .show(&mut row2_ui);
                    row2_ui.add_space(4.0);
                    row2_ui.label(egui::RichText::new("θ:").size(10.0).color(Color32::from_rgb(180, 180, 184)));
                    MacSlider::new(&mut card.rotation_deg, 0.0..=360.0)
                        .theme(theme)
                        .width(55.0)
                        .show(&mut row2_ui);
                }

                content_ui.add_space(16.0);

                // ====================================================================
                // 2. MacFixedView Demonstration
                // ====================================================================
                content_ui.label(
                    egui::RichText::new("2. MacFixedView — Absolute Point Placement")
                        .font(label_font.clone())
                        .strong(),
                );
                content_ui.label(
                    egui::RichText::new(
                        "Preserves exact NIB coordinates with zero margin inflation or subpixel clipping.",
                    )
                    .size(11.0),
                );
                content_ui.add_space(4.0);

                let fixed_canvas_w = (content_ui.available_width() - padding * 2.0).max(300.0);
                let fixed_canvas_h = 90.0;
                let (fixed_container_rect, _) = content_ui.allocate_exact_size(
                    vec2(fixed_canvas_w, fixed_canvas_h),
                    Sense::hover(),
                );

                content_ui.painter_at(fixed_container_rect).rect(
                    fixed_container_rect,
                    CornerRadius::same(6),
                    theme.window_background(),
                    Stroke::new(1.0, theme.separator()),
                    StrokeKind::Inside,
                );

                let fixed_view = MacFixedView::new(content_ui, fixed_container_rect);
                let subview_a = fixed_view.subview_rect(pos2(12.0, 12.0), vec2(100.0, 24.0));
                let subview_b = fixed_view.subview_rect(pos2(124.0, 12.0), vec2(150.0, 24.0));
                let subview_c = fixed_view.subview_rect(pos2(12.0, 48.0), vec2(262.0, 24.0));

                let mut ui_a = content_ui.new_child(egui::UiBuilder::new().max_rect(subview_a));
                ui_a.add(MacButtonCell::new("Primary Action").theme(theme));

                let mut ui_b = content_ui.new_child(egui::UiBuilder::new().max_rect(subview_b));
                ui_b.horizontal(|h| {
                    h.label(egui::RichText::new("Grid Snap:").size(11.0));
                    h.add(MacToggle::new(&mut self.fixed_grid_snap).theme(theme));
                });

                let mut ui_c = content_ui.new_child(egui::UiBuilder::new().max_rect(subview_c));
                MacSegmentedControl::new(
                    &["Vector Card", "Raster Mask", "Core Animation"],
                    self.fixed_seg_idx,
                )
                .theme(theme)
                .height(24.0)
                .show(&mut ui_c, |idx| {
                    self.fixed_seg_idx = idx;
                });

                content_ui.add_space(16.0);

                // ====================================================================
                // 3. MacRow Demonstration
                // ====================================================================
                content_ui.label(
                    egui::RichText::new("3. MacRow — Auto-Budgeting Compartment Layout")
                        .font(label_font.clone())
                        .strong(),
                );
                content_ui.label(
                    egui::RichText::new(
                        "Deterministic space allocation dividing container budget across fixed and flex cells.",
                    )
                    .size(11.0),
                );
                content_ui.add_space(4.0);

                let row_canvas_w = (content_ui.available_width() - padding * 2.0).max(300.0);
                let row_canvas_h = 32.0;
                let (row_rect, _) = content_ui.allocate_exact_size(
                    vec2(row_canvas_w, row_canvas_h),
                    Sense::hover(),
                );

                let mut row = MacRow::new(row_rect.min.y, 24.0, row_canvas_w, 6.0);
                let (x0, y0, w0, h0) = row.allocate_cell(80.0);
                let (x1, y1, w1, h1) = row.allocate_cell(110.0);
                let rem_w = row.remaining_width();
                let (x2, y2, w2, h2) = row.allocate_cell(rem_w.min(150.0));

                let r0 = Rect::from_min_size(pos2(row_rect.min.x + x0, y0), vec2(w0, h0));
                let r1 = Rect::from_min_size(pos2(row_rect.min.x + x1, y1), vec2(w1, h1));
                let r2 = Rect::from_min_size(pos2(row_rect.min.x + x2, y2), vec2(w2, h2));

                let mut ui_r0 = content_ui.new_child(egui::UiBuilder::new().max_rect(r0));
                ui_r0.add(MacButtonCell::new("Commit").theme(theme));

                let mut ui_r1 = content_ui.new_child(egui::UiBuilder::new().max_rect(r1));
                MacPopUpButton::new(&mut self.row_popup_idx, &["Preset A", "Preset B", "Preset C"])
                    .theme(theme)
                    .width(w1)
                    .show(&mut ui_r1);

                let mut ui_r2 = content_ui.new_child(egui::UiBuilder::new().max_rect(r2));
                MacSegmentedControl::new(&["Fit", "Fill", "Stretch"], self.row_seg_idx)
                    .theme(theme)
                    .height(24.0)
                    .show(&mut ui_r2, |idx| {
                        self.row_seg_idx = idx;
                    });

                content_ui.add_space(16.0);

                // ====================================================================
                // 4. MacPopover & CASpring Physics Showcase
                // ====================================================================
                content_ui.label(
                    egui::RichText::new("4. MacPopover & CASpring Harmonic Oscillator")
                        .font(label_font.clone())
                        .strong(),
                );
                content_ui.label(
                    egui::RichText::new(
                        "Analytical damped harmonic oscillator micro-animation with live waveform oscillation plot.",
                    )
                    .size(11.0),
                );
                content_ui.add_space(6.0);

                // Trigger button
                let btn_text = if self.popover_open {
                    "Dismiss Spring Popover"
                } else {
                    "Present Spring Popover"
                };
                let pop_btn = content_ui.add(
                    MacButtonCell::new(btn_text)
                        .theme(theme),
                );
                if pop_btn.clicked() {
                    self.popover_open = !self.popover_open;
                    self.spring_start_time = Some(content_ui.input(|i| i.time));
                }

                // Physics update
                let current_time = content_ui.input(|i| i.time);
                let (scale, alpha) = if let Some(start_t) = self.spring_start_time {
                    let elapsed = (current_time - start_t) as f32;
                    let duration = self.spring.settling_duration();
                    let progress = self.spring.evaluate(elapsed);

                    if elapsed < duration {
                        content_ui.ctx().request_repaint();
                    }

                    if self.popover_open {
                        (0.80 + 0.20 * progress, progress.clamp(0.0, 1.0))
                    } else {
                        (1.0 - 0.20 * progress, (1.0 - progress).clamp(0.0, 1.0))
                    }
                } else if self.popover_open {
                    (1.0, 1.0)
                } else {
                    (0.0, 0.0)
                };

                // Render Popover if visible
                if alpha > 0.01 {
                    content_ui.add_space(8.0);
                    let pop_w = 320.0;
                    let pop_h = 160.0;
                    let (pop_rect, _) = content_ui
                        .allocate_exact_size(vec2(pop_w, pop_h), Sense::hover());

                    MacPopover::new()
                        .theme(theme)
                        .arrow_edge(MacPopoverArrowEdge::Top)
                        .scale(scale)
                        .alpha(alpha)
                        .show(content_ui, pop_rect, |pop_ui| {
                            pop_ui.add_space(6.0);
                            pop_ui.label(
                                egui::RichText::new("CASpring Harmonic Oscillator")
                                    .size(12.0)
                                    .strong(),
                            );
                            pop_ui.label(
                                egui::RichText::new(format!(
                                    "Response (T₀): {:.2}s | Damping (ζ): {:.2}",
                                    self.spring.response, self.spring.damping_ratio
                                ))
                                .size(10.5),
                            );
                            pop_ui.label(
                                egui::RichText::new(format!(
                                    "Settling Time: {:.3}s | Angular Freq ω₀: {:.2} rad/s",
                                    self.spring.settling_duration(),
                                    self.spring.omega_0()
                                ))
                                .size(10.5),
                            );

                            pop_ui.add_space(4.0);
                            // Draw mini waveform of oscillation
                            let wave_rect = Rect::from_min_size(
                                pop_ui.cursor().min,
                                vec2(pop_w - 24.0, 48.0),
                            );
                            pop_ui.allocate_exact_size(
                                vec2(pop_w - 24.0, 48.0),
                                Sense::hover(),
                            );

                            let wp = pop_ui.painter_at(wave_rect);
                            wp.rect_filled(
                                wave_rect,
                                CornerRadius::same(3),
                                theme.control(),
                            );

                            let sample_count = 60;
                            let duration = self.spring.settling_duration().min(1.2);
                            let mut prev_pt = pos2(wave_rect.min.x, wave_rect.max.y);

                            for i in 0..=sample_count {
                                let frac = i as f32 / sample_count as f32;
                                let t = frac * duration;
                                let disp = self.spring.evaluate(t).clamp(-0.5, 1.5);
                                let px = wave_rect.min.x + frac * wave_rect.width();
                                let py = wave_rect.max.y
                                    - disp * (wave_rect.height() * 0.7)
                                    - 6.0;
                                let pt = pos2(px, py);

                                if i > 0 {
                                    wp.line_segment(
                                        [prev_pt, pt],
                                        Stroke::new(1.5, theme.control_accent()),
                                    );
                                }
                                prev_pt = pt;
                            }
                        });
                }

                content_ui.add_space(24.0);
            });
    }
}

impl InspectorState {
    /// Renders the Right Inspector with 3-Mode Presets:
    /// - Mode 0: Default NIB Document Inspector (260pt standard property pane).
    /// - Mode 1: Compact HUD (compact properties).
    /// - Mode 2: Expanded Pro Inspector (380pt with multi-deck tabs: Attributes, Metrics, Matrix).
    fn render(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        card_cell: &RefCell<CardModel>,
        mode: InspectorMode,
    ) {
        let mut card = card_cell.borrow_mut();
        let avail_rect = ui.available_rect_before_wrap();
        ui.painter_at(avail_rect).rect_filled(
            avail_rect,
            CornerRadius::ZERO,
            theme.window_background(),
        );

        MacScrollView::new("gallery_inspector_scroll")
            .theme(theme)
            .vertical(true)
            .horizontal(false)
            .max_height(avail_rect.height())
            .max_width(avail_rect.width())
            .show(ui, |scroll_ui| {
                scroll_ui.with_layout(Layout::top_down(Align::Min), |s_ui| {
                    s_ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    match mode {
                        InspectorMode::ExpandedPro => {
                            self.render_expanded_pro(s_ui, theme, &mut card);
                        }
                        _ => {
                            self.render_default_nib(s_ui, theme, &mut card);
                        }
                    }
                });
            });
    }

    /// Renders the Expanded Pro Inspector (Mode 2) with 3 top segmented tabs:
    /// `[ Attributes | Metrics | Matrix ]`.
    fn render_expanded_pro(&mut self, ui: &mut Ui, theme: &MacTheme, card: &mut CardModel) {
        // Top Multi-Deck Tab Selector - Centered across inspector width according to Apple HIG
        ui.add_space(8.0);
        let tab_idx = match self.expanded_tab {
            ExpandedProTab::Attributes => 0,
            ExpandedProTab::Metrics => 1,
            ExpandedProTab::Matrix => 2,
        };

        ui.vertical_centered(|c_ui| {
            MacSegmentedControl::new(&["Attributes", "Metrics", "Matrix"], tab_idx)
                .theme(theme)
                .height(24.0)
                .show(c_ui, |idx| {
                    self.expanded_tab = match idx {
                        0 => ExpandedProTab::Attributes,
                        1 => ExpandedProTab::Metrics,
                        _ => ExpandedProTab::Matrix,
                    };
                });
        });
        ui.add_space(6.0);

        MacStackView::new(MacStackOrientation::Vertical)
            .theme(theme)
            .spacing(10.0)
            .padding(12.0)
            .show_separators(true)
            .show(ui, |stack| match self.expanded_tab {
                ExpandedProTab::Attributes => {
                    stack.add_section(Some("Layer Identity"), |sec_ui| {
                        MacForm::new("exp_layer_identity_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(egui::RichText::new("Name:").size(11.0), |ui| {
                                    MacInputCell::with_id(
                                        Id::NULL.with("exp_layer_name"),
                                        &mut card.layer_name,
                                    )
                                    .theme(theme)
                                    .width(180.0)
                                    .show(ui);
                                });
                            });
                    });

                    stack.add_section(Some("Appearance & Blending"), |sec_ui| {
                        let blend_modes = ["Normal", "Multiply", "Screen", "Overlay"];
                        MacForm::new("exp_appearance_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(egui::RichText::new("Opacity:").size(11.0), |ui| {
                                    MacSlider::new(&mut card.opacity, 0.0..=100.0)
                                        .theme(theme)
                                        .ticks(5, true)
                                        .width(120.0)
                                        .show(ui);
                                    ui.label(
                                        egui::RichText::new(format!("{:.0}%", card.opacity)).size(10.5),
                                    );
                                });
                                form.row_label(egui::RichText::new("Blend:").size(11.0), |ui| {
                                    MacPopUpButton::new(&mut card.blend_mode_idx, &blend_modes)
                                        .theme(theme)
                                        .width(140.0)
                                        .show(ui);
                                });
                            });
                    });

                    stack.add_section(Some("Color Styles"), |sec_ui| {
                        MacForm::new("exp_color_styles_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(egui::RichText::new("Fill Color:").size(11.0), |ui| {
                                    MacColorWell::new(&mut card.fill_color).theme(theme).show(ui);
                                });
                                form.row_label(egui::RichText::new("Alpha Well:").size(11.0), |ui| {
                                    MacColorWell::new(&mut card.alpha_color)
                                        .theme(theme)
                                        .show_alpha(true)
                                        .show(ui);
                                });
                            });
                    });

                    stack.add_section(Some("Shadow & Effects"), |sec_ui| {
                        MacForm::new("exp_shadow_effects_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(egui::RichText::new("Drop Shadow:").size(11.0), |ui| {
                                    ui.add(MacToggle::new(&mut card.shadow_enabled).theme(theme));
                                });
                                if card.shadow_enabled {
                                    form.row_label(egui::RichText::new("Blur Radius:").size(11.0), |ui| {
                                        MacStepper::new(&mut card.shadow_blur, 0.0..=24.0, 1.0)
                                            .theme(theme)
                                            .show(ui);
                                        ui.label(
                                            egui::RichText::new(format!("{:.0} pt", card.shadow_blur))
                                                .size(10.5),
                                        );
                                    });
                                }
                            });
                    });
                }
                ExpandedProTab::Metrics => {
                    let id_w = Id::NULL.with("exp_input_w");
                    let id_h = Id::NULL.with("exp_input_h");
                    let id_r = Id::NULL.with("exp_input_r");

                    stack.add_section(Some("Dimensions & Corner Radius"), |sec_ui| {
                        MacForm::new("exp_dimensions_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(egui::RichText::new("Width (W):").size(11.0), |ui| {
                                    MacInputCell::with_id(id_w, &mut card.width)
                                        .theme(theme)
                                        .width(64.0)
                                        .next_focus(id_h)
                                        .prev_focus(id_r)
                                        .show(ui);
                                });
                                form.row_label(egui::RichText::new("Height (H):").size(11.0), |ui| {
                                    MacInputCell::with_id(id_h, &mut card.height)
                                        .theme(theme)
                                        .width(64.0)
                                        .next_focus(id_r)
                                        .prev_focus(id_w)
                                        .show(ui);
                                });
                                form.row_label(egui::RichText::new("Radius:").size(11.0), |ui| {
                                    MacInputCell::with_id(id_r, &mut card.radius)
                                        .theme(theme)
                                        .width(64.0)
                                        .next_focus(id_w)
                                        .prev_focus(id_h)
                                        .show(ui);
                                });
                                form.row_label(egui::RichText::new("Constrain Aspect:").size(11.0), |ui| {
                                    ui.add(MacToggle::new(&mut card.constrain_proportions).theme(theme));
                                });
                            });
                    });

                    stack.add_section(Some("Layout & Alignment Mask"), |sec_ui| {
                        MacForm::new("exp_align_mask_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(egui::RichText::new("Align:").size(11.0), |ui| {
                                    MacSegmentedControl::new(
                                        &["Left", "Center", "Right", "Fill"],
                                        card.alignment_idx,
                                    )
                                    .theme(theme)
                                    .show(ui, |idx| {
                                        card.alignment_idx = idx;
                                    });
                                });
                            });
                    });
                }
                ExpandedProTab::Matrix => {
                    // Authentic AppKit Affine Transform Deck
                    stack.add_section(Some("2x2 Affine Transform Matrix [a, b; c, d]"), |sec_ui| {
                        let theta = card.rotation_deg.to_radians();
                        let scale = card.scale_pct / 100.0;
                        let skew = card.skew_deg.to_radians().tan();

                        let a = scale * theta.cos();
                        let b = -scale * theta.sin() + skew;
                        let c = scale * theta.sin();
                        let d = scale * theta.cos();

                        let matrix_box = sec_ui.available_rect_before_wrap();
                        let m_rect =
                            Rect::from_min_size(matrix_box.min, vec2(matrix_box.width(), 48.0));
                        let mp = sec_ui.painter_at(m_rect);
                        mp.rect_filled(m_rect, CornerRadius::same(4), theme.control());
                        mp.rect_stroke(
                            m_rect,
                            CornerRadius::same(4),
                            Stroke::new(1.0, theme.separator()),
                            StrokeKind::Inside,
                        );

                        let font = FontId::monospace(11.0);
                        let row1 = format!("│ {:>+6.3}   {:>+6.3} │", a, b);
                        let row2 = format!("│ {:>+6.3}   {:>+6.3} │", c, d);
                        mp.text(
                            pos2(m_rect.center().x, m_rect.min.y + 14.0),
                            Align2::CENTER_CENTER,
                            row1,
                            font.clone(),
                            theme.label(),
                        );
                        mp.text(
                            pos2(m_rect.center().x, m_rect.min.y + 34.0),
                            Align2::CENTER_CENTER,
                            row2,
                            font,
                            theme.label(),
                        );
                        sec_ui.allocate_exact_size(vec2(m_rect.width(), 50.0), Sense::hover());
                    });

                    stack.add_section(Some("Continuous Transformation Controls"), |sec_ui| {
                        let mut rot_f64 = card.rotation_deg as f64;
                        MacForm::new("exp_matrix_controls_form")
                            .theme(theme)
                            .show(sec_ui, |form| {
                                form.row_label(egui::RichText::new("Rotation:").size(11.0), |ui| {
                                    MacSlider::new(&mut card.rotation_deg, 0.0..=360.0)
                                        .theme(theme)
                                        .ticks(8, true)
                                        .width(120.0)
                                        .show(ui);
                                    ui.label(
                                        egui::RichText::new(format!("{:.0}°", card.rotation_deg))
                                            .size(10.5),
                                    );
                                });

                                form.row_label(egui::RichText::new("Fine Step:").size(11.0), |ui| {
                                    if ui.add(
                                        MacStepper::new(&mut rot_f64, 0.0..=360.0, 5.0)
                                            .theme(theme)
                                            .wraps(true),
                                    )
                                    .changed()
                                    {
                                        card.rotation_deg = rot_f64 as f32;
                                    }
                                    ui.label(egui::RichText::new("±5.0°").size(10.5));
                                });

                                form.row_label(egui::RichText::new("Scale:").size(11.0), |ui| {
                                    MacSlider::new(&mut card.scale_pct, 50.0..=200.0)
                                        .theme(theme)
                                        .ticks(6, true)
                                        .width(120.0)
                                        .show(ui);
                                    ui.label(
                                        egui::RichText::new(format!("{:.0}%", card.scale_pct))
                                            .size(10.5),
                                    );
                                });

                                form.row_label(egui::RichText::new("Skew / Shear:").size(11.0), |ui| {
                                    MacSlider::new(&mut card.skew_deg, -45.0..=45.0)
                                        .theme(theme)
                                        .width(120.0)
                                        .show(ui);
                                    ui.label(
                                        egui::RichText::new(format!("{:.0}°", card.skew_deg))
                                            .size(10.5),
                                    );
                                });
                            });

                        sec_ui.add_space(6.0);
                        let reset_btn =
                            sec_ui.add(MacButtonCell::new("Reset to Identity Matrix").theme(theme));
                        if reset_btn.clicked() {
                            card.rotation_deg = 0.0;
                            card.scale_pct = 100.0;
                            card.skew_deg = 0.0;
                        }
                    });
                }
            });
    }

    /// Renders the standard Default NIB Document Inspector (Mode 0).
    fn render_default_nib(&mut self, ui: &mut Ui, theme: &MacTheme, card: &mut CardModel) {
        let input_id_w = Id::NULL.with("nib_input_w");
        let input_id_h = Id::NULL.with("nib_input_h");
        let input_id_r = Id::NULL.with("nib_input_r");

        MacStackView::new(MacStackOrientation::Vertical)
            .theme(theme)
            .spacing(10.0)
            .padding(12.0)
            .show_separators(true)
            .show(ui, |stack| {
                // Section 0: Preview Badge (MacThumbnailCell)
                stack.add_section(Some("Preview (MacThumbnailCell)"), |sec_ui| {
                    MacForm::new("nib_preview_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Vector Badge:").size(11.0), |ui| {
                                MacThumbnailCell::new()
                                    .theme(theme)
                                    .selected(true)
                                    .custom_draw(|painter, rect| {
                                        painter.circle_filled(rect.center(), 7.0, theme.control_accent());
                                        painter.rect_filled(
                                            rect.shrink(5.0),
                                            CornerRadius::same(2),
                                            theme.window_background(),
                                        );
                                    })
                                    .show(ui);
                            });
                        });
                });

                // Section 1: Geometry & Tab-Chain (MacInputCell)
                stack.add_section(Some("Geometry & Tab-Chain (MacInputCell)"), |sec_ui| {
                    MacForm::new("nib_geometry_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Width:").size(11.0), |ui| {
                                MacInputCell::with_id(input_id_w, &mut card.width)
                                    .theme(theme)
                                    .width(64.0)
                                    .next_focus(input_id_h)
                                    .prev_focus(input_id_r)
                                    .placeholder("Width")
                                    .show(ui);
                            });
                            form.row_label(egui::RichText::new("Height:").size(11.0), |ui| {
                                MacInputCell::with_id(input_id_h, &mut card.height)
                                    .theme(theme)
                                    .width(64.0)
                                    .next_focus(input_id_r)
                                    .prev_focus(input_id_w)
                                    .placeholder("Height")
                                    .show(ui);
                            });
                            form.row_label(egui::RichText::new("Corner Radius:").size(11.0), |ui| {
                                MacInputCell::with_id(input_id_r, &mut card.radius)
                                    .theme(theme)
                                    .width(64.0)
                                    .next_focus(input_id_w)
                                    .prev_focus(input_id_h)
                                    .placeholder("Radius")
                                    .show(ui);
                            });
                        });
                });

                // Section 2: Continuous Tracking (MacSlider & MacStepper)
                stack.add_section(Some("Continuous Controls (Slider & Stepper)"), |sec_ui| {
                    MacForm::new("nib_continuous_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Slider:").size(11.0), |ui| {
                                MacSlider::new(&mut self.slider_value, 0.0..=100.0)
                                    .theme(theme)
                                    .ticks(5, true)
                                    .width(110.0)
                                    .show(ui);
                                ui.label(
                                    egui::RichText::new(format!("{:.0}%", self.slider_value)).size(10.5),
                                );
                            });
                            form.row_label(egui::RichText::new("Stepper:").size(11.0), |ui| {
                                MacStepper::new(&mut self.stepper_value, 0.0..=10.0, 0.5)
                                    .theme(theme)
                                    .wraps(self.stepper_wrap)
                                    .show(ui);
                                ui.label(
                                    egui::RichText::new(format!("{:.1} pt", self.stepper_value)).size(10.5),
                                );
                            });
                        });
                });

                // Section 3: Selection Primitives (MacPopUpButton & MacSegmentedControl)
                stack.add_section(Some("Selection (PopUp & Segmented)"), |sec_ui| {
                    let popup_items = ["Default NIB", "Compact HUD", "Expanded Inspector"];
                    MacForm::new("nib_selection_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Preset:").size(11.0), |ui| {
                                MacPopUpButton::new(&mut self.popup_selected, &popup_items)
                                    .theme(theme)
                                    .width(135.0)
                                    .show(ui);
                            });
                            form.row_label(egui::RichText::new("Units:").size(11.0), |ui| {
                                MacSegmentedControl::new(
                                    &["Points", "Pixels", "Percent"],
                                    self.unit_segmented_idx,
                                )
                                .theme(theme)
                                .show(ui, |idx| {
                                    self.unit_segmented_idx = idx;
                                });
                            });
                        });
                });

                // Section 4: State & Appearance (MacToggle & MacColorWell)
                stack.add_section(Some("State & Color (Toggle & ColorWell)"), |sec_ui| {
                    MacForm::new("nib_state_color_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Subpixel Snap:").size(11.0), |ui| {
                                ui.add(MacToggle::new(&mut self.snapping_enabled).theme(theme));
                            });
                            form.row_label(egui::RichText::new("Accent Swatch:").size(11.0), |ui| {
                                MacColorWell::new(&mut card.fill_color).theme(theme).show(ui);
                            });
                            form.row_label(egui::RichText::new("Alpha Well:").size(11.0), |ui| {
                                MacColorWell::new(&mut card.alpha_color)
                                    .theme(theme)
                                    .show_alpha(true)
                                    .show(ui);
                            });
                        });
                });
            });
    }
}
