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
    contrast_ratio, relative_luminance, setup_symbols_font,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GalleryScreen {
    #[default]
    CanvasPhysics = 0,
    VectorAssets = 1,
    DesignTokensWcag = 2,
    LayoutEngines = 3,
}

impl GalleryScreen {
    pub const ALL: [Self; 4] = [
        Self::CanvasPhysics,
        Self::VectorAssets,
        Self::DesignTokensWcag,
        Self::LayoutEngines,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::CanvasPhysics => "Interactive Canvas & Physics",
            Self::VectorAssets => "Vector Assets Gallery",
            Self::DesignTokensWcag => "W3C Design Tokens & WCAG Engine",
            Self::LayoutEngines => "Layout Engines Playground",
        }
    }

    pub fn breadcrumb_path(&self) -> [&'static str; 3] {
        match self {
            Self::CanvasPhysics => {
                ["macview_gallery", "Screen 1", "Interactive Canvas & Physics"]
            }
            Self::VectorAssets => {
                ["macview_gallery", "Screen 2", "Vector Assets Gallery"]
            }
            Self::DesignTokensWcag => {
                ["macview_gallery", "Screen 3", "W3C Design Tokens & WCAG Engine"]
            }
            Self::LayoutEngines => {
                ["macview_gallery", "Screen 4", "Layout Engines Playground"]
            }
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
    expanded_inspector: bool,
    selected_node: String,
    active_screen: GalleryScreen,
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
            expanded_inspector: true,
            selected_node: "Interactive Canvas & Physics".to_string(),
            active_screen: GalleryScreen::CanvasPhysics,
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
    selected_symbol: MacSymbol,
    token_tab_idx: usize,
    wcag_fg: Color32,
    wcag_bg: Color32,
    row_container_width: f32,
    cascade_start_time: Option<f64>,
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
            selected_symbol: MacSymbol::Search,
            token_tab_idx: 0,
            wcag_fg: Color32::from_rgb(255, 255, 255),
            wcag_bg: Color32::from_rgb(0, 122, 255),
            row_container_width: 420.0,
            cascade_start_time: None,
        }
    }
}

struct InspectorState {
    popup_selected: usize,
    unit_segmented_idx: usize,
    snapping_enabled: bool,
    expanded_tab: ExpandedProTab,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
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
    active_screen: GalleryScreen,
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
            CommandPaletteItem::new("screen.canvas", "Screen 1: Interactive Canvas & Physics")
                .category("Screens")
                .keywords(&["canvas", "physics", "affine", "screen 1"]),
            CommandPaletteItem::new("screen.vector", "Screen 2: Vector Assets Gallery")
                .category("Screens")
                .keywords(&["vector", "symbols", "cupertino", "screen 2"]),
            CommandPaletteItem::new("screen.tokens", "Screen 3: W3C Design Tokens & WCAG Engine")
                .category("Screens")
                .keywords(&["tokens", "wcag", "contrast", "screen 3"]),
            CommandPaletteItem::new("screen.layout", "Screen 4: Layout Engines Playground")
                .category("Screens")
                .keywords(&["layout", "fixedview", "macrow", "screen 4"]),
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
            active_screen: GalleryScreen::CanvasPhysics,
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
            "screen.canvas" => self.active_screen = GalleryScreen::CanvasPhysics,
            "screen.vector" => self.active_screen = GalleryScreen::VectorAssets,
            "screen.tokens" => self.active_screen = GalleryScreen::DesignTokensWcag,
            "screen.layout" => self.active_screen = GalleryScreen::LayoutEngines,
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

        // Global Command Palette Shortcut Handling (Cmd+K on macOS, Ctrl+K / Super+K on Windows/Linux)
        let toggle_palette = ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::COMMAND, egui::Key::K)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::K)
                || i.consume_key(egui::Modifiers::MAC_CMD, egui::Key::K)
        });
        if toggle_palette {
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
        let inspector_mode_cell = std::cell::Cell::new(self.inspector_mode);
        let active_screen_cell = std::cell::Cell::new(self.active_screen);
        let snapping_enabled = inspector.snapping_enabled;

        // Primary Split: Left Sidebar vs (Center View + Right Inspector)
        MacSplitView::with_pos(
            "gallery_left_split",
            MacSplitOrientation::Horizontal,
            &mut left_split_pos,
        )
        .theme(&theme)
        .min_first(220.0)
        .max_first(360.0)
        .min_second(620.0)
        .show(
            &mut work_ui,
            |sidebar_ui| {
                let mut m = inspector_mode_cell.get();
                let mut s = active_screen_cell.get();
                sidebar.render(sidebar_ui, &theme, &mut m, &mut s);
                inspector_mode_cell.set(m);
                active_screen_cell.set(s);
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
                        center.render(
                            center_ui,
                            &theme,
                            card,
                            inspector_mode_cell.get(),
                            active_screen_cell.get(),
                            snapping_enabled,
                        );
                    },
                    |inspector_ui| {
                        let mut m = inspector_mode_cell.get();
                        inspector.render(inspector_ui, &theme, card, &mut m);
                        inspector_mode_cell.set(m);
                    },
                );
            },
        );

        self.inspector_mode = inspector_mode_cell.get();
        self.active_screen = active_screen_cell.get();

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
            let mode_def = format!("{} Default", MacSymbol::Gear.as_str());
            let mode_hud = format!("{} Compact", MacSymbol::Sliders.as_str());
            let mode_pro = format!("{} Expanded", MacSymbol::Plus.as_str());
            let mode_items = [mode_def.as_str(), mode_hud.as_str(), mode_pro.as_str()];
            MacSegmentedControl::new(
                &mode_items,
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

            // 4. Spotlight Command Palette Button [ Search ⌘K on macOS, Ctrl+K on Windows ]
            let k_str = if cfg!(target_os = "macos") { "⌘K" } else { "Ctrl+K" };
            let search_label = format!("{} {}", MacSymbol::Search.as_str(), k_str);
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
    /// 4 AppKit capability screens, and `accessory_custom` buttons.
    fn render(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        inspector_mode_out: &mut InspectorMode,
        active_screen_out: &mut GalleryScreen,
    ) {
        let avail_rect = ui.available_rect_before_wrap();
        ui.painter_at(avail_rect).rect_filled(
            avail_rect,
            CornerRadius::ZERO,
            theme.window_background(),
        );

        MacOutlineView::new("gallery_sidebar_outline")
            .theme(theme)
            .show(ui, |tree| {
                // Section 1: Screens Navigator Hierarchy
                let resp_comp = tree.row(
                    "node_screens_root",
                    0,
                    true,
                    &mut self.expanded_components,
                    false,
                    |row| {
                        row.header(&format_tracked_small_caps("Screens"));
                    },
                );
                if resp_comp.clicked {
                    self.expanded_components = !self.expanded_components;
                }

                if self.expanded_components {
                    // Screen 1: Interactive Canvas & Physics
                    let sel_canvas = self.active_screen == GalleryScreen::CanvasPhysics;
                    let resp = tree.row("node_screen_canvas", 1, false, &mut false, sel_canvas, |row| {
                        let bg_style = theme.selected_control_background_style();
                        row.thumbnail(|painter, rect| {
                            let icon_color = if sel_canvas {
                                bg_style.primary_text_color(theme)
                            } else {
                                theme.control_accent()
                            };
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
                        row.label("Interactive Canvas & Physics");
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
                    if resp.clicked {
                        self.active_screen = GalleryScreen::CanvasPhysics;
                        self.selected_node = "Interactive Canvas & Physics".to_string();
                        *active_screen_out = GalleryScreen::CanvasPhysics;
                    }

                    // Screen 2: Vector Assets Gallery
                    let sel_vector = self.active_screen == GalleryScreen::VectorAssets;
                    let resp = tree.row("node_screen_vector", 1, false, &mut false, sel_vector, |row| {
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
                        row.label("Vector Assets Gallery");
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
                    if resp.clicked {
                        self.active_screen = GalleryScreen::VectorAssets;
                        self.selected_node = "Vector Assets Gallery".to_string();
                        *active_screen_out = GalleryScreen::VectorAssets;
                    }

                    // Screen 3: W3C Design Tokens & WCAG Engine
                    let sel_tokens = self.active_screen == GalleryScreen::DesignTokensWcag;
                    let resp = tree.row("node_screen_tokens", 1, false, &mut false, sel_tokens, |row| {
                        let bg_style = theme.selected_control_background_style();
                        row.thumbnail(|painter, rect| {
                            let icon_color = if sel_tokens {
                                bg_style.primary_text_color(theme)
                            } else {
                                theme.control_accent()
                            };
                            painter.circle_filled(rect.center(), 5.0, icon_color);
                        });
                        row.label("W3C Design Tokens & WCAG Engine");
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
                    if resp.clicked {
                        self.active_screen = GalleryScreen::DesignTokensWcag;
                        self.selected_node = "W3C Design Tokens & WCAG Engine".to_string();
                        *active_screen_out = GalleryScreen::DesignTokensWcag;
                    }

                    // Screen 4: Layout Engines Playground
                    let sel_layout = self.active_screen == GalleryScreen::LayoutEngines;
                    let resp = tree.row("node_screen_layout", 1, false, &mut false, sel_layout, |row| {
                        let bg_style = theme.selected_control_background_style();
                        row.thumbnail(|painter, rect| {
                            let icon_color = if sel_layout {
                                bg_style.primary_text_color(theme)
                            } else {
                                theme.traffic_light_minimize().0
                            };
                            painter.rect_stroke(
                                Rect::from_min_size(pos2(rect.min.x + 2.0, rect.min.y + 3.0), vec2(5.0, 10.0)),
                                CornerRadius::same(1),
                                Stroke::new(1.0, icon_color),
                                StrokeKind::Inside,
                            );
                            painter.rect_stroke(
                                Rect::from_min_size(pos2(rect.min.x + 9.0, rect.min.y + 3.0), vec2(5.0, 10.0)),
                                CornerRadius::same(1),
                                Stroke::new(1.0, icon_color),
                                StrokeKind::Inside,
                            );
                        });
                        row.label("Layout Engines Playground");
                    });
                    if resp.clicked {
                        self.active_screen = GalleryScreen::LayoutEngines;
                        self.selected_node = "Layout Engines Playground".to_string();
                        *active_screen_out = GalleryScreen::LayoutEngines;
                    }
                }

                // Section 2: Inspector Presets
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
                    let sel_nib = *inspector_mode_out == InspectorMode::DefaultNib;
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
                        row.label("Default NIB");
                    });
                    if resp_nib.clicked {
                        self.selected_node = "Default NIB".to_string();
                        *inspector_mode_out = InspectorMode::DefaultNib;
                    }

                    // Mode 1: Compact HUD
                    let sel_hud = *inspector_mode_out == InspectorMode::CompactHud;
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
                        row.label("Compact HUD");
                    });
                    if resp_hud.clicked {
                        self.selected_node = "Compact HUD".to_string();
                        *inspector_mode_out = InspectorMode::CompactHud;
                    }

                    // Mode 2: Expanded Pro
                    let sel_exp = *inspector_mode_out == InspectorMode::ExpandedPro;
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
                        row.label("Expanded Pro");
                    });
                    if resp_exp.clicked {
                        self.selected_node = "Expanded Pro".to_string();
                        *inspector_mode_out = InspectorMode::ExpandedPro;
                    }
                }
            });
    }
}

impl CenterState {
    /// Renders the Center View containing Search, the active Master-Detail AppKit capability screen,
    /// and the Canvas Footer with reactive breadcrumbs.
    fn render(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        card_cell: &RefCell<CardModel>,
        inspector_mode: InspectorMode,
        active_screen: GalleryScreen,
        snapping_enabled: bool,
    ) {
        let mut card = card_cell.borrow_mut();
        let avail_rect = ui.available_rect_before_wrap();
        ui.painter_at(avail_rect).rect_filled(
            avail_rect,
            CornerRadius::ZERO,
            theme.control_background(),
        );

        let padding = 16.0;

        // Top Search Bar (Synchronized across screens)
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

        let placeholder = match active_screen {
            GalleryScreen::CanvasPhysics => "Search canvas geometry, transforms, CASpring physics...",
            GalleryScreen::VectorAssets => "Filter 14 AppKit vector symbols (e.g. search, chevron, gear)...",
            GalleryScreen::DesignTokensWcag => "Filter W3C design tokens (e.g. button, metric, motion)...",
            GalleryScreen::LayoutEngines => "Filter layout engines (FixedView, MacRow auto-budgeting)...",
        };

        MacSearchField::new(&mut self.search_query)
            .theme(theme)
            .placeholder(placeholder)
            .width(search_rect.width())
            .corner_radius(6.0)
            .show(&mut search_ui);

        // Mount Canvas Footer (SPEC-007 §3) at Canvas Bottom with synchronized breadcrumbs
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
        let breadcrumbs = active_screen.breadcrumb_path();
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
                match active_screen {
                    GalleryScreen::CanvasPhysics => {
                        self.render_screen_canvas_physics(
                            content_ui,
                            theme,
                            &mut card,
                            inspector_mode,
                            snapping_enabled,
                        );
                    }
                    GalleryScreen::VectorAssets => {
                        self.render_screen_vector_assets(content_ui, theme);
                    }
                    GalleryScreen::DesignTokensWcag => {
                        self.render_screen_tokens_wcag(content_ui, theme);
                    }
                    GalleryScreen::LayoutEngines => {
                        self.render_screen_layout_engines(content_ui, theme);
                    }
                }
            });
    }
    /// Screen 1: Interactive Canvas & Physics
    /// Showcases 2D Affine Matrix live transforms, rotation, scale, grid snap with snapping_enabled,
    /// CASpring damped harmonic oscillator pulse, and interactive canvas click.
    fn render_screen_canvas_physics(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        card: &mut CardModel,
        inspector_mode: InspectorMode,
        snapping_enabled: bool,
    ) {
        let label_font = FontId::proportional(12.0);
        content_ui.label(
            egui::RichText::new("1. Interactive Canvas & Physics — Live AppKit Vector Card")
                .font(label_font.clone())
                .strong(),
        );
        content_ui.label(
            egui::RichText::new(
                "Transforms live with the Inspector (Width, Height, Affine 2D Rotation, Scale, Color, CASpring Physics). Click canvas to pulse.",
            )
            .size(11.0),
        );
        content_ui.add_space(6.0);

        let padding = 16.0;
        let canvas_w = (content_ui.available_width() - padding * 2.0).max(340.0);
        let canvas_h = 240.0;
        let (canvas_rect, canvas_resp) = content_ui.allocate_exact_size(
            vec2(canvas_w, canvas_h),
            Sense::click(),
        );

        let canvas_clicked = canvas_resp.clicked();
        if canvas_clicked {
            self.bounce_start_time = Some(content_ui.input(|i| i.time));
        }

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
        let raw_card_w: f32 = card.width.parse::<f32>().unwrap_or(240.0).clamp(60.0, 500.0);
        let card_h: f32 = card.height.parse::<f32>().unwrap_or(140.0).clamp(40.0, 300.0);
        let card_radius: f32 = card.radius.parse::<f32>().unwrap_or(10.0).clamp(0.0, 40.0);

        // Layout & Alignment Mask application: 0: Left, 1: Center, 2: Right, 3: Fill
        let (effective_card_w, card_center_x) = match card.alignment_idx {
            0 => (raw_card_w, canvas_rect.min.x + 24.0 + raw_card_w * 0.5),
            2 => (raw_card_w, canvas_rect.max.x - 24.0 - raw_card_w * 0.5),
            3 => (
                (canvas_rect.width() - 48.0).max(60.0),
                canvas_rect.center().x,
            ),
            _ => (raw_card_w, canvas_rect.center().x), // 1: Center
        };
        let card_w = effective_card_w;

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

        // Grid snapping logic for rotation and scale when enabled
        let effective_rot = if snapping_enabled {
            (card.rotation_deg / 15.0).round() * 15.0
        } else {
            card.rotation_deg
        };
        let effective_scale = if snapping_enabled {
            (card.scale_pct / 25.0).round() * 25.0
        } else {
            card.scale_pct
        };

        let theta = effective_rot.to_radians();
        let scale_factor = (effective_scale / 100.0) * bounce_scale;
        let skew_val = card.skew_deg.to_radians().tan();

        // Affine 2x2 Matrix components:
        let a = scale_factor * theta.cos();
        let b = -scale_factor * theta.sin() + skew_val;
        let c = scale_factor * theta.sin();
        let d = scale_factor * theta.cos();

        let card_center = pos2(card_center_x, canvas_rect.center().y);

        let hw = effective_card_w * 0.5;
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
                format!("{:.0} × {:.0} pt | Scale: {:.0}%", card_w, card_h, effective_scale),
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
                format!("θ: {:.1}° | S: {:.0}%", effective_rot, effective_scale),
                FontId::proportional(11.0),
                text_secondary,
            );
        }

        // Interactive Controls Bar below canvas
        content_ui.add_space(8.0);
        content_ui.horizontal(|h| {
            let pulse_label = format!("{} Trigger CASpring Pulse", MacSymbol::ArrowClockwise.as_str());
            let bounce_btn = h.add(
                MacButtonCell::new(&pulse_label)
                    .theme(theme)
                    .selected(self.bounce_start_time.is_some()),
            );
            if bounce_btn.clicked() {
                self.bounce_start_time = Some(h.input(|i| i.time));
            }

            let reset_label = format!("{} Reset Identity", MacSymbol::ArrowCounterClockwise.as_str());
            let reset_btn = h.add(MacButtonCell::new(&reset_label).theme(theme));
            if reset_btn.clicked() {
                card.rotation_deg = 0.0;
                card.scale_pct = 100.0;
                card.skew_deg = 0.0;
            }

            let snap_status = if snapping_enabled {
                "Snap: 15° / 25%"
            } else {
                "Snap: Off"
            };
            h.label(egui::RichText::new(snap_status).size(10.5).color(theme.secondary_label()));
            h.label(egui::RichText::new("T₀=0.35s, ζ=0.65").size(10.5).color(theme.placeholder_text()));
        });

        // Floating Compact HUD Palette (when Compact HUD mode active)
        if inspector_mode == InspectorMode::CompactHud {
            let hud_w = 178.0;
            let hud_h = 63.0;

            let hud_min = pos2(canvas_rect.min.x + self.hud_pos.x, canvas_rect.min.y + self.hud_pos.y);
            let hud_rect = Rect::from_min_size(hud_min, vec2(hud_w, hud_h));

            let hud_painter = content_ui.painter_at(hud_rect);

            hud_painter.rect_filled(
                hud_rect.translate(vec2(0.0, 3.0)),
                CornerRadius::same(8),
                Color32::from_black_alpha(90),
            );
            hud_painter.rect(
                hud_rect,
                CornerRadius::same(6),
                Color32::from_rgba_premultiplied(26, 26, 28, 220),
                Stroke::new(1.0, Color32::from_rgb(60, 60, 62)),
                StrokeKind::Inside,
            );

            let header_h = 22.0;
            let header_rect = Rect::from_min_size(hud_rect.min, vec2(hud_w, header_h));
            hud_painter.line_segment(
                [pos2(hud_rect.min.x, hud_rect.min.y + header_h), pos2(hud_rect.max.x, hud_rect.min.y + header_h)],
                Stroke::new(1.0, Color32::from_rgb(48, 48, 50)),
            );

            let header_resp = content_ui.interact(
                header_rect,
                content_ui.scope_id().with("hud_header_drag"),
                Sense::click_and_drag(),
            );
            if header_resp.dragged() {
                self.hud_pos.x = (self.hud_pos.x + header_resp.drag_delta().x).clamp(0.0, canvas_w - hud_w);
                self.hud_pos.y = (self.hud_pos.y + header_resp.drag_delta().y).clamp(0.0, canvas_h - hud_h);
            }

            hud_painter.circle_filled(pos2(hud_rect.min.x + 10.0, hud_rect.min.y + 11.0), 3.0, Color32::from_rgb(180, 180, 184));
            hud_painter.text(
                pos2(hud_rect.min.x + 20.0, hud_rect.min.y + 11.0),
                Align2::LEFT_CENTER,
                "HUD Palette",
                FontId::proportional(10.5),
                Color32::WHITE,
            );

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

        // CASpring Damped Harmonic Oscillator Waveform Popover
        content_ui.add_space(16.0);
        content_ui.label(
            egui::RichText::new("CASpring Harmonic Oscillator Waveform")
                .font(label_font)
                .strong(),
        );
        content_ui.label(
            egui::RichText::new(
                "Analytical damped harmonic oscillator micro-animation with live waveform oscillation plot.",
            )
            .size(11.0),
        );
        content_ui.add_space(6.0);

        let btn_text = if self.popover_open {
            "Dismiss Spring Popover"
        } else {
            "Present Spring Popover"
        };
        let pop_btn = content_ui.add(MacButtonCell::new(btn_text).theme(theme));
        if pop_btn.clicked() {
            self.popover_open = !self.popover_open;
            self.spring_start_time = Some(content_ui.input(|i| i.time));
        }

        let (scale, alpha) = if let Some(start_t) = self.spring_start_time {
            let elapsed = (cur_time - start_t) as f32;
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

        if alpha > 0.01 {
            content_ui.add_space(8.0);
            let pop_w = 320.0;
            let pop_h = 160.0;
            let (pop_rect, _) = content_ui.allocate_exact_size(vec2(pop_w, pop_h), Sense::hover());

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
                        let py = wave_rect.max.y - disp * (wave_rect.height() * 0.7) - 6.0;
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
    }

    /// Screen 2: Vector Assets Gallery
    /// Displays an interactive grid of all 14 `MacSymbol` symbols from `MacSymbol::ALL`,
    /// search filtering box, thumbnail preview cards, and metadata inspector.
    fn render_screen_vector_assets(&mut self, content_ui: &mut Ui, theme: &MacTheme) {
        let label_font = FontId::proportional(12.0);
        content_ui.label(
            egui::RichText::new("2. Vector Assets Gallery — CupertinoIcons TrueType Engine")
                .font(label_font.clone())
                .strong(),
        );
        content_ui.label(
            egui::RichText::new(
                "Resolution-independent AppKit vector iconography powered by CupertinoIcons TrueType font. Replaces host OS emojis with authentic theme-aware vector glyphs.",
            )
            .size(11.0),
        );
        content_ui.add_space(8.0);

        let q = self.search_query.trim().to_lowercase();
        let symbols_meta: [(MacSymbol, &'static str, &'static str, &'static str); 14] = [
            (MacSymbol::Search, "Search", "Search fields, Spotlight", "U+F4A5"),
            (MacSymbol::ChevronRight, "Chevron Right", "Navigation forward, disclosure", "U+F3D3"),
            (MacSymbol::ChevronLeft, "Chevron Left", "Navigation back", "U+F3D2"),
            (MacSymbol::ChevronDown, "Chevron Down", "Popup buttons, dropdown menus", "U+F5D5"),
            (MacSymbol::ChevronUp, "Chevron Up", "Steppers, navigation up", "U+F5E5"),
            (MacSymbol::SidebarLeading, "Sidebar Leading", "Sidebar toggle, window layout", "U+F7D6"),
            (MacSymbol::Gear, "Gear", "Settings, preferences, metrics", "U+F43C"),
            (MacSymbol::Sliders, "Sliders", "Filters, display adjustments", "U+F7DC"),
            (MacSymbol::Checkmark, "Checkmark", "Menu checkmarks, verification", "U+F3FD"),
            (MacSymbol::Xmark, "Xmark", "Close button, cancel, clear", "U+F404"),
            (MacSymbol::Plus, "Plus", "Add layer, expand, zoom in", "U+F489"),
            (MacSymbol::Minus, "Minus", "Remove layer, collapse, zoom out", "U+F70F"),
            (MacSymbol::ArrowClockwise, "Arrow Clockwise", "Reload, refresh, repeat", "U+F49A"),
            (MacSymbol::ArrowCounterClockwise, "Arrow Counter Clockwise", "Undo action, reset identity", "U+F21C"),
        ];

        let filtered_symbols: Vec<(MacSymbol, &'static str, &'static str, &'static str)> = symbols_meta
            .iter()
            .copied()
            .filter(|(sym, name, usage, codepoint)| {
                if q.is_empty() {
                    return true;
                }
                name.to_lowercase().contains(&q)
                    || usage.to_lowercase().contains(&q)
                    || codepoint.to_lowercase().contains(&q)
                    || format!("{:?}", sym).to_lowercase().contains(&q)
            })
            .collect();

        // Count readout
        content_ui.horizontal(|h| {
            h.label(
                egui::RichText::new(format!(
                    "Showing {} of 14 MacSymbol vector assets",
                    filtered_symbols.len()
                ))
                .size(11.0)
                .color(theme.secondary_label()),
            );
            if !q.is_empty() {
                h.label(
                    egui::RichText::new(format!("(filtered by \"{}\")", q))
                        .size(10.5)
                        .color(theme.control_accent()),
                );
            }
        });
        content_ui.add_space(6.0);

        // Responsive Grid of symbols
        let item_w = 88.0f32;
        let item_h = 72.0f32;
        let spacing = 8.0f32;
        let avail_w = content_ui.available_width().max(300.0);
        let cols = ((avail_w + spacing) / (item_w + spacing)).floor().max(2.0) as usize;

        egui::Grid::new("vector_symbols_grid")
            .spacing(vec2(spacing, spacing))
            .show(content_ui, |grid_ui| {
                for (col_idx, (sym, name, _, _)) in filtered_symbols.iter().enumerate() {
                    let is_selected = *sym == self.selected_symbol;
                    let (rect, resp) = grid_ui.allocate_exact_size(vec2(item_w, item_h), Sense::click());

                    if resp.clicked() {
                        self.selected_symbol = *sym;
                    }

                    let p = grid_ui.painter_at(rect);
                    let bg_color = if is_selected {
                        theme.control_accent().gamma_multiply(0.2)
                    } else if resp.hovered() {
                        theme.control().gamma_multiply(0.6)
                    } else {
                        theme.window_background()
                    };
                    let border_color = if is_selected {
                        theme.control_accent()
                    } else {
                        theme.separator()
                    };

                    p.rect(
                        rect,
                        CornerRadius::same(6),
                        bg_color,
                        Stroke::new(if is_selected { 1.5 } else { 1.0 }, border_color),
                        StrokeKind::Inside,
                    );

                    let glyph_color = if is_selected {
                        theme.control_accent()
                    } else {
                        theme.label()
                    };

                    // Draw monochrome symbol glyph in CupertinoIcons font
                    p.text(
                        pos2(rect.center().x, rect.min.y + 24.0),
                        Align2::CENTER_CENTER,
                        sym.as_str(),
                        FontId::proportional(22.0),
                        glyph_color,
                    );

                    // Draw symbol short name
                    p.text(
                        pos2(rect.center().x, rect.max.y - 14.0),
                        Align2::CENTER_CENTER,
                        *name,
                        FontId::proportional(10.0),
                        if is_selected { theme.label() } else { theme.secondary_label() },
                    );

                    if (col_idx + 1) % cols == 0 {
                        grid_ui.end_row();
                    }
                }
            });

        // Selected Symbol Detail & Metadata Card
        content_ui.add_space(16.0);
        content_ui.label(
            egui::RichText::new("Selected Vector Symbol Metadata & TrueType Inspector")
                .font(label_font)
                .strong(),
        );
        content_ui.add_space(6.0);

        let sel_sym = self.selected_symbol;
        let (_, sel_name, sel_usage, sel_cp) = symbols_meta
            .iter()
            .copied()
            .find(|(s, _, _, _)| *s == sel_sym)
            .unwrap_or(symbols_meta[0]);

        let detail_card_w = (content_ui.available_width() - 32.0).max(300.0);
        let detail_card_h = 130.0;
        let (detail_rect, _) = content_ui.allocate_exact_size(vec2(detail_card_w, detail_card_h), Sense::hover());
        let dp = content_ui.painter_at(detail_rect);

        dp.rect(
            detail_rect,
            CornerRadius::same(8),
            theme.window_background(),
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );

        // Left preview box (large 52pt glyph)
        let preview_box_w = 90.0;
        let preview_rect = Rect::from_min_size(detail_rect.min, vec2(preview_box_w, detail_card_h));
        dp.rect_filled(
            preview_rect,
            CornerRadius { nw: 8, sw: 8, ne: 0, se: 0 },
            theme.control().gamma_multiply(0.4),
        );
        dp.line_segment(
            [pos2(preview_rect.max.x, preview_rect.min.y), pos2(preview_rect.max.x, preview_rect.max.y)],
            Stroke::new(1.0, theme.separator()),
        );
        dp.text(
            preview_rect.center(),
            Align2::CENTER_CENTER,
            sel_sym.as_str(),
            FontId::proportional(48.0),
            theme.control_accent(),
        );

        // Right metadata details
        let meta_x = preview_rect.max.x + 14.0;
        let start_y = detail_rect.min.y + 14.0;
        let line_spacing = 18.0;

        dp.text(
            pos2(meta_x, start_y),
            Align2::LEFT_TOP,
            format!("MacSymbol::{:?} — {}", sel_sym, sel_name),
            FontId::proportional(13.0),
            theme.label(),
        );

        let rows = [
            ("Unicode Codepoint", format!("{} (hex) | Dec: {}", sel_cp, sel_sym.codepoint())),
            ("Character Literal", format!("{:?}", sel_sym.as_char())),
            ("Unicode Block", "Private Use Area (PUA) 0xE000..=0xF8FF".to_string()),
            ("AppKit Usage", sel_usage.to_string()),
            ("Font Asset", "CupertinoIcons.ttf (251 KB embedded TrueType font)".to_string()),
            ("Anti-Emoji Guard", "COMPLIANT — Monochrome vector glyph (zero host OS emoji bleed)".to_string()),
        ];

        for (idx, (label, val)) in rows.iter().enumerate() {
            let row_y = start_y + 22.0 + idx as f32 * line_spacing;
            if row_y + 14.0 > detail_rect.max.y {
                break;
            }
            dp.text(
                pos2(meta_x, row_y),
                Align2::LEFT_TOP,
                format!("{}: ", label),
                FontId::proportional(10.5),
                theme.secondary_label(),
            );
            dp.text(
                pos2(meta_x + 120.0, row_y),
                Align2::LEFT_TOP,
                val,
                FontId::proportional(10.5),
                theme.label(),
            );
        }

        content_ui.add_space(24.0);
    }

    /// Screen 3: W3C Design Tokens & WCAG Engine
    /// Demonstrates interactive token catalogue, relative luminance formula L <= 0.179,
    /// and real-time WCAG AA contrast evaluation with pass/fail badges.
    fn render_screen_tokens_wcag(&mut self, content_ui: &mut Ui, theme: &MacTheme) {
        let label_font = FontId::proportional(12.0);
        content_ui.label(
            egui::RichText::new("3. W3C Design Tokens & WCAG Engine — Token Catalogue & Contrast Engine")
                .font(label_font.clone())
                .strong(),
        );
        content_ui.label(
            egui::RichText::new(
                "Grounded in W3C DTCG token schema and WCAG 2.1 §1.4.3 Level AA contrast verification with AppKit NSBackgroundStyle (L \u{2264} 0.179).",
            )
            .size(11.0),
        );
        content_ui.add_space(8.0);

        // Section A: Interactive Real-Time WCAG 2.1 AA Contrast Engine
        content_ui.label(
            egui::RichText::new("Real-Time WCAG 2.1 AA Contrast Evaluation Engine")
                .font(label_font.clone())
                .strong(),
        );
        content_ui.add_space(4.0);

        let card_w = (content_ui.available_width() - 32.0).max(300.0);
        let (eval_rect, _) = content_ui.allocate_exact_size(vec2(card_w, 180.0), Sense::hover());
        let ep = content_ui.painter_at(eval_rect);

        ep.rect(
            eval_rect,
            CornerRadius::same(8),
            theme.window_background(),
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );

        // Child UI inside the evaluation container
        let mut eval_ui = content_ui.new_child(
            egui::UiBuilder::new()
                .max_rect(eval_rect.shrink(10.0))
                .layout(Layout::top_down(Align::Min)),
        );

        eval_ui.horizontal(|h| {
            h.label(egui::RichText::new("Foreground:").size(11.0).strong());
            h.add(MacColorWell::new(&mut self.wcag_fg).theme(theme));
            h.add_space(12.0);
            h.label(egui::RichText::new("Background:").size(11.0).strong());
            h.add(MacColorWell::new(&mut self.wcag_bg).theme(theme));
            h.add_space(16.0);

            // Presets
            if h.add(MacButtonCell::new("Label on Window").theme(theme)).clicked() {
                self.wcag_fg = theme.label();
                self.wcag_bg = theme.window_background();
            }
            if h.add(MacButtonCell::new("White on Accent").theme(theme)).clicked() {
                self.wcag_fg = Color32::WHITE;
                self.wcag_bg = theme.control_accent();
            }
            if h.add(MacButtonCell::new("Secondary on Control").theme(theme)).clicked() {
                self.wcag_fg = theme.secondary_label();
                self.wcag_bg = theme.control_background();
            }
            if h.add(MacButtonCell::new("Dark on Accent").theme(theme)).clicked() {
                self.wcag_fg = Color32::from_rgb(30, 30, 30);
                self.wcag_bg = theme.control_accent();
            }
        });
        eval_ui.add_space(8.0);

        // Math calculations
        let l_fg = relative_luminance(self.wcag_fg);
        let l_bg = relative_luminance(self.wcag_bg);
        let ratio = contrast_ratio(self.wcag_fg, self.wcag_bg);
        let bg_style = MacBackgroundStyle::from_color(self.wcag_bg);

        eval_ui.horizontal(|h| {
            h.label(
                egui::RichText::new(format!(
                    "L(fg) = {:.4} | L(bg) = {:.4} | Contrast Ratio: {:.2}:1",
                    l_fg, l_bg, ratio
                ))
                .size(11.5)
                .strong(),
            );
        });

        eval_ui.horizontal(|h| {
            let style_str = match bg_style {
                MacBackgroundStyle::Dark => "NSBackgroundStyle::Dark (L \u{2264} 0.179 \u{2192} White Text Guarantees \u{2265} 4.5:1)",
                MacBackgroundStyle::Light => "NSBackgroundStyle::Light (L > 0.179 \u{2192} Dark Text Guarantees \u{2265} 4.5:1)",
            };
            h.label(egui::RichText::new(format!("AppKit Background Style: {}", style_str)).size(10.5).color(theme.secondary_label()));
        });
        eval_ui.add_space(8.0);

        // WCAG AA Pass/Fail Badges
        eval_ui.horizontal(|h| {
            let normal_pass = ratio >= 4.5;
            let large_pass = ratio >= 3.0;
            let ui_pass = ratio >= 3.0;

            let badge = |ui: &mut Ui, label: &str, pass: bool| {
                let text_color = if pass { Color32::from_rgb(40, 180, 80) } else { Color32::from_rgb(230, 60, 60) };
                let bg_color = text_color.gamma_multiply(0.15);
                let (rect, _) = ui.allocate_exact_size(vec2(140.0, 22.0), Sense::hover());
                let p = ui.painter_at(rect);
                p.rect(rect, CornerRadius::same(4), bg_color, Stroke::new(1.0, text_color.gamma_multiply(0.5)), StrokeKind::Inside);
                p.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    format!("{}: {}", label, if pass { "PASS \u{2713}" } else { "FAIL \u{2717}" }),
                    FontId::proportional(10.5),
                    text_color,
                );
            };

            badge(h, "Normal Text (\u{2265} 4.5:1)", normal_pass);
            h.add_space(6.0);
            badge(h, "Large Text (\u{2265} 3.0:1)", large_pass);
            h.add_space(6.0);
            badge(h, "UI Elements (\u{2265} 3.0:1)", ui_pass);
        });
        eval_ui.add_space(8.0);

        // Live preview swatch
        let (sample_rect, _) = eval_ui.allocate_exact_size(vec2(card_w - 24.0, 32.0), Sense::hover());
        let sp = eval_ui.painter_at(sample_rect);
        sp.rect(
            sample_rect,
            CornerRadius::same(5),
            self.wcag_bg,
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );
        sp.text(
            sample_rect.center(),
            Align2::CENTER_CENTER,
            "The quick brown fox jumps over the lazy dog (WCAG 2.1 AA Sample)",
            FontId::proportional(12.0),
            self.wcag_fg,
        );

        // Section B: W3C DTCG Design Token Catalogue
        content_ui.add_space(16.0);
        content_ui.label(
            egui::RichText::new("W3C DTCG Token Catalogue (tokens/design_tokens.json)")
                .font(label_font)
                .strong(),
        );
        content_ui.add_space(4.0);

        MacSegmentedControl::new(
            &["Control Metrics", "Semantic Palettes", "Motion Parameters"],
            self.token_tab_idx,
        )
        .theme(theme)
        .height(24.0)
        .show(content_ui, |idx| {
            self.token_tab_idx = idx;
        });

        content_ui.add_space(8.0);

        let q = self.search_query.trim().to_lowercase();

        match self.token_tab_idx {
            0 => {
                // Control Metrics table
                let tokens = [
                    ("button.height", "24.0 pt", "Standard push button height"),
                    ("button.corner_radius", "6.0 pt", "Rounded corner radius"),
                    ("button.min_width", "68.0 pt", "Minimum button horizontal budget"),
                    ("outline_view.row_height", "24.0 pt", "Sidebar row height"),
                    ("outline_view.indent_step", "16.0 pt", "Hierarchy indentation step"),
                    ("toolbar.height", "52.0 pt", "Unified AppKit toolbar height"),
                    ("toolbar.traffic_light_radius", "6.0 pt", "Close/Minimize/Zoom traffic lights"),
                    ("toolbar.traffic_light_spacing", "8.0 pt", "Spacing between window controls"),
                    ("slider.track_height", "4.0 pt", "Continuous slider track thickness"),
                    ("slider.thumb_radius", "9.0 pt", "Slider draggable knob radius"),
                    ("stepper.width", "16.0 pt", "Vertical stepper control width"),
                    ("stepper.height", "24.0 pt", "Vertical stepper control height"),
                    ("popover.corner_radius", "8.0 pt", "Popover rounded corner radius"),
                    ("popover.arrow_size", "8.0 pt", "Popover arrow indicator height"),
                    ("canvas_footer.height", "24.0 pt", "SPEC-007 Canvas footer deterministic height"),
                ];

                for (name, val, desc) in tokens.iter() {
                    if !q.is_empty() && !name.to_lowercase().contains(&q) && !desc.to_lowercase().contains(&q) {
                        continue;
                    }
                    content_ui.horizontal(|h| {
                        h.label(egui::RichText::new(*name).size(11.0).monospace().color(theme.control_accent()));
                        h.label(egui::RichText::new(*val).size(11.0).strong());
                        h.label(egui::RichText::new(format!("— {}", desc)).size(10.5).color(theme.secondary_label()));
                    });
                }
            }
            1 => {
                // Semantic Palettes
                let colors = [
                    ("window_background", theme.window_background()),
                    ("control_background", theme.control_background()),
                    ("control_accent", theme.control_accent()),
                    ("selected_control", theme.selected_control()),
                    ("label", theme.label()),
                    ("secondary_label", theme.secondary_label()),
                    ("separator", theme.separator()),
                    ("system_green", theme.system_green()),
                    ("traffic_light_close", theme.traffic_light_close().0),
                    ("traffic_light_minimize", theme.traffic_light_minimize().0),
                    ("traffic_light_zoom", theme.traffic_light_zoom().0),
                ];

                for (name, c) in colors.iter() {
                    if !q.is_empty() && !name.to_lowercase().contains(&q) {
                        continue;
                    }
                    content_ui.horizontal(|h| {
                        let (rect, _) = h.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                        h.painter().rect_filled(rect, CornerRadius::same(3), *c);
                        h.painter().rect_stroke(rect, CornerRadius::same(3), Stroke::new(1.0, theme.separator()), StrokeKind::Inside);
                        h.label(egui::RichText::new(*name).size(11.0).monospace());
                        h.label(egui::RichText::new(format!("rgb({}, {}, {}) | #{:02X}{:02X}{:02X}", c.r(), c.g(), c.b(), c.r(), c.g(), c.b())).size(10.5).color(theme.secondary_label()));
                    });
                }
            }
            _ => {
                // Motion Parameters
                let motion = [
                    ("motion.hud_panel_transition.response", "0.30 s", "CASpring response duration T₀"),
                    ("motion.hud_panel_transition.damping_ratio", "0.70", "Damping ratio ζ (perceptual settling)"),
                    ("motion.hud_panel_transition.settling_duration", "0.48 s", "Duration to settle within ε=1e-3"),
                    ("motion.selection_spring.response", "0.20 s", "Quick selection indicator snap"),
                    ("motion.selection_spring.damping_ratio", "0.85", "Critically damped selection motion"),
                    ("motion.bounce_spring.response", "0.35 s", "Harmonic oscillator pulse response"),
                    ("motion.bounce_spring.damping_ratio", "0.65", "Lightly damped harmonic pulse with overshoot"),
                ];

                for (name, val, desc) in motion.iter() {
                    if !q.is_empty() && !name.to_lowercase().contains(&q) && !desc.to_lowercase().contains(&q) {
                        continue;
                    }
                    content_ui.horizontal(|h| {
                        h.label(egui::RichText::new(*name).size(11.0).monospace().color(theme.control_accent()));
                        h.label(egui::RichText::new(*val).size(11.0).strong());
                        h.label(egui::RichText::new(format!("— {}", desc)).size(10.5).color(theme.secondary_label()));
                    });
                }
            }
        }

        content_ui.add_space(24.0);
    }

    /// Screen 4: Layout Engines Playground
    /// Demonstrates interactive MacFixedView mode switching (Vector Card, live Raster Mask clipping with with_clip_rect,
    /// Core Animation cascading spring) and MacRow container width slider with colored compartments auto-budgeting.
    fn render_screen_layout_engines(&mut self, content_ui: &mut Ui, theme: &MacTheme) {
        let label_font = FontId::proportional(12.0);
        content_ui.label(
            egui::RichText::new("4. Layout Engines Playground — MacFixedView & MacRow")
                .font(label_font.clone())
                .strong(),
        );
        content_ui.label(
            egui::RichText::new(
                "Deterministic point-coordinate placement via MacFixedView and real-time auto-budgeting container compartment division via MacRow.",
            )
            .size(11.0),
        );
        content_ui.add_space(8.0);

        // Section 1: MacFixedView
        content_ui.label(
            egui::RichText::new("MacFixedView — Absolute Coordinate Placement & Mode Switching")
                .font(label_font.clone())
                .strong(),
        );
        content_ui.add_space(4.0);

        MacSegmentedControl::new(
            &["Vector Card", "Raster Mask", "Core Animation"],
            self.fixed_seg_idx,
        )
        .theme(theme)
        .height(24.0)
        .show(content_ui, |idx| {
            self.fixed_seg_idx = idx;
        });

        content_ui.add_space(6.0);

        let fixed_w = (content_ui.available_width() - 32.0).max(320.0);
        let fixed_h = 130.0;
        let (fixed_container_rect, _) = content_ui.allocate_exact_size(vec2(fixed_w, fixed_h), Sense::hover());
        let fp = content_ui.painter_at(fixed_container_rect);

        fp.rect(
            fixed_container_rect,
            CornerRadius::same(8),
            theme.window_background(),
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );

        let cur_time = content_ui.input(|i| i.time);

        match self.fixed_seg_idx {
            0 => {
                // Mode 0: Vector Card placement
                let mut fixed_view = MacFixedView::new(content_ui, fixed_container_rect);
                fixed_view.subview(pos2(14.0, 14.0), vec2(150.0, 48.0), |ui_a| {
                    ui_a.add(MacButtonCell::new("Vector Action A").theme(theme));
                });

                fixed_view.subview(pos2(174.0, 14.0), vec2(150.0, 48.0), |ui_b| {
                    ui_b.horizontal(|h| {
                        h.label(egui::RichText::new("Grid Snap:").size(11.0));
                        h.add(MacToggle::new(&mut self.fixed_grid_snap).theme(theme));
                    });
                });

                fixed_view.subview(pos2(14.0, 72.0), vec2(310.0, 44.0), |ui_c| {
                    ui_c.label(
                        egui::RichText::new("Point placement exact coordinates: (14, 14), (174, 14), (14, 72). Zero layout margin jitter.")
                            .size(10.5)
                            .color(theme.secondary_label()),
                    );
                });
            }
            1 => {
                // Mode 1: Raster Mask clipping with with_clip_rect
                let clip_margin_h = 24.0;
                let clip_margin_v = 16.0;
                let clip_rect = Rect::from_min_max(
                    pos2(fixed_container_rect.min.x + clip_margin_h, fixed_container_rect.min.y + clip_margin_v),
                    pos2(fixed_container_rect.max.x - clip_margin_h, fixed_container_rect.max.y - clip_margin_v),
                );

                // Hardware raster mask clipping via painter.with_clip_rect
                let clipped_painter = content_ui.painter().with_clip_rect(clip_rect);

                // Background inside mask
                clipped_painter.rect_filled(clip_rect, CornerRadius::same(6), Color32::from_rgb(22, 24, 28));

                // Oversized diagonal stripes extending beyond clip bounds
                let mut sx = clip_rect.min.x - 60.0;
                let stripe_w = 14.0;
                let stripe_gap = 22.0;
                let stripe_color = theme.control_accent().gamma_multiply(0.45);

                while sx < clip_rect.max.x + 60.0 {
                    clipped_painter.line_segment(
                        [pos2(sx, clip_rect.max.y + 20.0), pos2(sx + 50.0, clip_rect.min.y - 20.0)],
                        Stroke::new(stripe_w, stripe_color),
                    );
                    sx += stripe_gap + stripe_w;
                }

                // Centered badge inside mask
                clipped_painter.rect_filled(
                    Rect::from_center_size(clip_rect.center(), vec2(220.0, 32.0)),
                    CornerRadius::same(4),
                    theme.control_background().gamma_multiply(0.9),
                );
                clipped_painter.text(
                    clip_rect.center(),
                    Align2::CENTER_CENTER,
                    "Hardware Raster Mask \u{2014} with_clip_rect()",
                    FontId::proportional(11.0),
                    theme.label(),
                );

                // Outer border of clip rect to visually prove zero bleed
                fp.rect_stroke(clip_rect, CornerRadius::same(6), Stroke::new(1.5, theme.control_accent()), StrokeKind::Inside);
            }
            _ => {
                // Mode 2: Core Animation Cascading Spring
                let mut fixed_view = MacFixedView::new(content_ui, fixed_container_rect);
                let clicked = fixed_view.subview(pos2(14.0, 14.0), vec2(240.0, 24.0), |btn_ui| {
                    btn_ui.add(MacButtonCell::new("Trigger Cascading Spring").theme(theme)).clicked()
                });
                if clicked {
                    self.cascade_start_time = Some(cur_time);
                }

                let card_w = 88.0f32;
                let card_h = 60.0f32;
                let base_y = fixed_container_rect.min.y + 48.0;

                for idx in 0..3 {
                    let delay = idx as f32 * 0.08;
                    let (disp_y, alpha) = if let Some(start_t) = self.cascade_start_time {
                        let elapsed = (cur_time - start_t) as f32;
                        let t_card = (elapsed - delay).max(0.0);
                        let dur = self.bounce_spring.settling_duration();
                        if elapsed < dur + 0.3 {
                            fixed_view.ui.ctx().request_repaint();
                        }
                        let p = self.bounce_spring.evaluate(t_card);
                        ((1.0 - p) * 20.0, p.clamp(0.0, 1.0))
                    } else {
                        (0.0, 1.0)
                    };

                    let cx = fixed_container_rect.min.x + 14.0 + idx as f32 * (card_w + 14.0);
                    let cy = base_y + disp_y;
                    let c_rect = Rect::from_min_size(pos2(cx, cy), vec2(card_w, card_h));

                    fp.rect(
                        c_rect,
                        CornerRadius::same(6),
                        theme.control_accent().gamma_multiply(0.2 * alpha),
                        Stroke::new(1.0, theme.control_accent().gamma_multiply(alpha)),
                        StrokeKind::Inside,
                    );
                    fp.text(
                        pos2(c_rect.center().x, c_rect.min.y + 18.0),
                        Align2::CENTER_CENTER,
                        format!("Layer {}", idx + 1),
                        FontId::proportional(11.0),
                        theme.label().gamma_multiply(alpha),
                    );
                    fp.text(
                        pos2(c_rect.center().x, c_rect.max.y - 16.0),
                        Align2::CENTER_CENTER,
                        format!("\u{03C4} = {:.2}s", delay),
                        FontId::proportional(10.0),
                        theme.secondary_label().gamma_multiply(alpha),
                    );
                }
            }
        }

        // Section 2: MacRow Auto-Budgeting Container
        content_ui.add_space(16.0);
        content_ui.label(
            egui::RichText::new("MacRow — Real-Time Auto-Budgeting Compartment Layout")
                .font(label_font)
                .strong(),
        );
        content_ui.label(
            egui::RichText::new(
                "Adjust container budget width slider. Colored compartments automatically allocate space without clipping or overlap.",
            )
            .size(11.0),
        );
        content_ui.add_space(6.0);

        // Container Width Slider
        content_ui.horizontal(|h| {
            h.label(egui::RichText::new("Container Budget:").size(11.0).strong());
            MacSlider::new(&mut self.row_container_width, 240.0..=640.0)
                .theme(theme)
                .width(200.0)
                .show(h);
            h.label(egui::RichText::new(format!("{:.0} pt", self.row_container_width)).size(11.0).strong());
        });
        content_ui.add_space(8.0);

        let row_container_h = 36.0;
        let (row_alloc_rect, _) = content_ui.allocate_exact_size(vec2(self.row_container_width, row_container_h), Sense::hover());
        let rp = content_ui.painter_at(row_alloc_rect);

        // Container bounding box
        rp.rect(
            row_alloc_rect,
            CornerRadius::same(6),
            theme.window_background(),
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );

        let mut row = MacRow::new(row_alloc_rect.min.y + 4.0, 28.0, self.row_container_width, 6.0);
        let (x0, y0, w0, h0) = row.allocate_cell(90.0);
        let (x1, y1, w1, h1) = row.allocate_cell(120.0);
        let rem_w = row.remaining_width();
        let (x2, y2, w2, h2) = row.allocate_cell(rem_w.max(30.0));

        let cell0_rect = Rect::from_min_size(pos2(row_alloc_rect.min.x + x0, y0), vec2(w0, h0));
        let cell1_rect = Rect::from_min_size(pos2(row_alloc_rect.min.x + x1, y1), vec2(w1, h1));
        let cell2_rect = Rect::from_min_size(pos2(row_alloc_rect.min.x + x2, y2), vec2(w2, h2));

        // Compartment 0: Fixed 90pt (Blue)
        rp.rect_filled(cell0_rect, CornerRadius::same(4), Color32::from_rgb(30, 80, 160));
        rp.text(cell0_rect.center(), Align2::CENTER_CENTER, "Fixed: 90pt", FontId::proportional(10.5), Color32::WHITE);

        // Compartment 1: Fixed 120pt (Purple)
        rp.rect_filled(cell1_rect, CornerRadius::same(4), Color32::from_rgb(110, 50, 160));
        rp.text(cell1_rect.center(), Align2::CENTER_CENTER, "Fixed: 120pt", FontId::proportional(10.5), Color32::WHITE);

        // Compartment 2: Dynamic Flex (Teal/Emerald)
        rp.rect_filled(cell2_rect, CornerRadius::same(4), Color32::from_rgb(20, 140, 110));
        rp.text(cell2_rect.center(), Align2::CENTER_CENTER, format!("Flex: {:.0}pt", w2), FontId::proportional(10.5), Color32::WHITE);

        content_ui.add_space(6.0);
        content_ui.label(
            egui::RichText::new(format!(
                "Budget: {:.0} pt | Fixed: 90pt + 120pt + 12pt gap = 222pt | Remaining Flex: {:.0} pt (\u{2713} Zero Overlap)",
                self.row_container_width, rem_w
            ))
            .size(10.5)
            .color(theme.secondary_label()),
        );

        content_ui.add_space(24.0);
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
        mode: &mut InspectorMode,
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
                    match *mode {
                        InspectorMode::ExpandedPro => {
                            self.render_expanded_pro(s_ui, theme, &mut card);
                        }
                        _ => {
                            self.render_default_nib(s_ui, theme, &mut card, mode);
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

        let t_attr = format!("{} Attributes", MacSymbol::Sliders.as_str());
        let t_metr = format!("{} Metrics", MacSymbol::Gear.as_str());
        let t_matr = format!("{} Matrix", MacSymbol::ArrowClockwise.as_str());
        let tab_items = [t_attr.as_str(), t_metr.as_str(), t_matr.as_str()];

        ui.vertical_centered(|c_ui| {
            MacSegmentedControl::new(&tab_items, tab_idx)
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
                        let reset_label = format!("{} Reset to Identity Matrix", MacSymbol::ArrowCounterClockwise.as_str());
                        let reset_btn =
                            sec_ui.add(MacButtonCell::new(&reset_label).theme(theme));
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
    fn render_default_nib(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        card: &mut CardModel,
        mode: &mut InspectorMode,
    ) {
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
                            form.row_label(egui::RichText::new("Card Scale:").size(11.0), |ui| {
                                MacSlider::new(&mut card.scale_pct, 25.0..=200.0)
                                    .theme(theme)
                                    .ticks(7, true)
                                    .width(110.0)
                                    .show(ui);
                                ui.label(
                                    egui::RichText::new(format!("{:.0}%", card.scale_pct)).size(10.5),
                                );
                            });
                            form.row_label(egui::RichText::new("Corner Radius:").size(11.0), |ui| {
                                let mut r = card.radius.parse::<f64>().unwrap_or(10.0);
                                let prev_r = r;
                                MacStepper::new(&mut r, 0.0..=40.0, 1.0)
                                    .theme(theme)
                                    .wraps(false)
                                    .show(ui);
                                if (r - prev_r).abs() > 0.01 {
                                    card.radius = format!("{:.0}", r);
                                }
                                let unit_str = match self.unit_segmented_idx {
                                    1 => "px",
                                    2 => "%",
                                    _ => "pt",
                                };
                                ui.label(
                                    egui::RichText::new(format!("{:.0} {}", r, unit_str)).size(10.5),
                                );
                            });
                        });
                });

                // Section 3: Selection Primitives (MacPopUpButton & MacSegmentedControl)
                stack.add_section(Some("Selection (PopUp & Segmented)"), |sec_ui| {
                    let popup_items = ["Default NIB", "Compact HUD", "Expanded Pro"];
                    self.popup_selected = match *mode {
                        InspectorMode::DefaultNib => 0,
                        InspectorMode::CompactHud => 1,
                        InspectorMode::ExpandedPro => 2,
                    };
                    MacForm::new("nib_selection_form")
                        .theme(theme)
                        .show(sec_ui, |form| {
                            form.row_label(egui::RichText::new("Preset:").size(11.0), |ui| {
                                let prev_sel = self.popup_selected;
                                MacPopUpButton::new(&mut self.popup_selected, &popup_items)
                                    .theme(theme)
                                    .width(135.0)
                                    .show(ui);
                                if self.popup_selected != prev_sel {
                                    *mode = match self.popup_selected {
                                        0 => InspectorMode::DefaultNib,
                                        1 => InspectorMode::CompactHud,
                                        _ => InspectorMode::ExpandedPro,
                                    };
                                }
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
