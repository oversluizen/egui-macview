//! Window chrome, frameless 8-way resize hit-testing, menu actions, and unified `MacToolbar`.

use crate::{GalleryApp, GalleryScreen, InspectorMode};
use egui::{pos2, Align, CursorIcon, Layout, PointerButton, Pos2, Rect, ResizeDirection, Ui, ViewportCommand};
use egui_macview::{
    theme::{MacAppearance, MacTheme, WindowControlsStyle},
    MacButtonCell, MacSegmentedControl, MacSymbol, MacToolbar,
};

/// Evaluates 8-way frameless resize direction while protecting caption button hit zones.
pub fn detect_frameless_resize_dir(
    style: WindowControlsStyle,
    is_maximized: bool,
    pos: Pos2,
    full_rect: Rect,
    theme: &MacTheme,
) -> Option<(ResizeDirection, CursorIcon)> {
    if style == WindowControlsStyle::NativeHost || is_maximized {
        return None;
    }
    if style == WindowControlsStyle::WindowsCaptionButtons {
        let tb = theme.toolbar_metrics();
        let caption_rect = Rect::from_min_max(
            pos2(full_rect.max.x - tb.caption_button_width.points() * 3.0, full_rect.min.y),
            pos2(full_rect.max.x, full_rect.min.y + tb.caption_button_height.points()),
        );
        if caption_rect.contains(pos) {
            return None;
        }
    }
    let border = 6.0;
    let left = (pos.x - full_rect.min.x).abs() <= border;
    let right = (full_rect.max.x - pos.x).abs() <= border;
    let top = (pos.y - full_rect.min.y).abs() <= border;
    let bottom = (full_rect.max.y - pos.y).abs() <= border;

    match (left, right, top, bottom) {
        (true, _, true, _) => Some((ResizeDirection::NorthWest, CursorIcon::ResizeNorthWest)),
        (_, true, true, _) => Some((ResizeDirection::NorthEast, CursorIcon::ResizeNorthEast)),
        (true, _, _, true) => Some((ResizeDirection::SouthWest, CursorIcon::ResizeSouthWest)),
        (_, true, _, true) => Some((ResizeDirection::SouthEast, CursorIcon::ResizeSouthEast)),
        (true, _, _, _) => Some((ResizeDirection::West, CursorIcon::ResizeWest)),
        (_, true, _, _) => Some((ResizeDirection::East, CursorIcon::ResizeEast)),
        (_, _, true, _) => Some((ResizeDirection::North, CursorIcon::ResizeNorth)),
        (_, _, _, true) => Some((ResizeDirection::South, CursorIcon::ResizeSouth)),
        _ => None,
    }
}

impl GalleryApp {
    pub(crate) fn handle_frameless_resize(&self, ui: &Ui, full_rect: Rect, theme: &MacTheme) -> bool {
        let Some(pos) = ui.input(|i| i.pointer.hover_pos()) else { return false };
        let Some((dir, cursor)) = detect_frameless_resize_dir(
            self.window_controls_style, self.is_maximized, pos, full_rect, theme,
        ) else {
            return false;
        };
        ui.ctx().set_cursor_icon(cursor);
        if ui.input(|i| i.pointer.button_pressed(PointerButton::Primary)) {
            ui.ctx().send_viewport_cmd(ViewportCommand::BeginResize(dir));
        }
        true
    }

    pub(crate) fn handle_menu_action(&mut self, action: &str, ctx: &egui::Context) {
        match action {
            "edit.undo" if self.active_screen == GalleryScreen::AvKitMediaStudio => {
                if self.center.undo_avkit() {
                    self.toasts.borrow_mut().info("AVKit: Undone adjustment");
                }
            }
            "edit.undo" => {
                let mut c = self.card.borrow_mut();
                if self.undo_manager.borrow_mut().undo(&mut *c) {
                    *self.last_stable_card.borrow_mut() = c.clone();
                    self.toasts.borrow_mut().info("Undone");
                }
            }
            "edit.redo" if self.active_screen == GalleryScreen::AvKitMediaStudio => {
                if self.center.redo_avkit() {
                    self.toasts.borrow_mut().info("AVKit: Redone adjustment");
                }
            }
            "edit.redo" => {
                let mut c = self.card.borrow_mut();
                if self.undo_manager.borrow_mut().redo(&mut *c) {
                    *self.last_stable_card.borrow_mut() = c.clone();
                    self.toasts.borrow_mut().info("Redone");
                }
            }
            "screen.canvas" => self.active_screen = GalleryScreen::CanvasPhysics,
            "screen.vector" => self.active_screen = GalleryScreen::VectorAssets,
            "screen.tokens" => self.active_screen = GalleryScreen::DesignTokensWcag,
            "screen.finder" | "screen.layout" => self.active_screen = GalleryScreen::FinderDataBrowser,
            "screen.avkit" => self.active_screen = GalleryScreen::AvKitMediaStudio,
            "screen.advanced" => self.active_screen = GalleryScreen::AdvancedControls,
            "theme.aqua" => self.appearance = MacAppearance::Aqua,
            "theme.anthracite" => self.appearance = MacAppearance::Anthracite,
            "theme.dark" => self.appearance = MacAppearance::DarkAqua,
            "canvas.pulse" => self.center.bounce_start_time = Some(0.0),
            "matrix.reset" => {
                let mut c = self.card.borrow_mut();
                (c.rotation_deg, c.scale_pct, c.skew_deg) = (0.0, 100.0, 0.0);
                self.toasts.borrow_mut().info("Transform reset");
            }
            "view.zoom_in" => {
                let mut c = self.card.borrow_mut();
                c.scale_pct = (c.scale_pct + 25.0).clamp(25.0, 200.0);
                self.toasts.borrow_mut().info(format!("Zoom: {:.0}%", c.scale_pct));
            }
            "view.zoom_out" => {
                let mut c = self.card.borrow_mut();
                c.scale_pct = (c.scale_pct - 25.0).clamp(25.0, 200.0);
                self.toasts.borrow_mut().info(format!("Zoom: {:.0}%", c.scale_pct));
            }
            "view.zoom_reset" => {
                self.card.borrow_mut().scale_pct = 100.0;
                self.toasts.borrow_mut().info("Zoom: 100%");
            }
            "view.toggle_inspector" => {
                self.inspector_mode = match self.inspector_mode {
                    InspectorMode::DefaultNib => InspectorMode::CompactHud,
                    InspectorMode::CompactHud => InspectorMode::ExpandedPro,
                    InspectorMode::ExpandedPro => InspectorMode::DefaultNib,
                };
                self.toasts.borrow_mut().info("Inspector preset cycled");
            }
            "view.fullscreen" | "window.zoom" => {
                self.is_maximized = !self.is_maximized;
                ctx.send_viewport_cmd(ViewportCommand::Maximized(self.is_maximized));
            }
            "window.bring_all_to_front" => {
                ctx.send_viewport_cmd(ViewportCommand::Focus);
                self.toasts.borrow_mut().info("Brought window to front");
            }
            "file.new" => {
                *self.card.borrow_mut() = crate::CardModel::default();
                self.toasts.borrow_mut().info("Reset card to defaults");
            }
            "file.open" => {
                self.active_screen = GalleryScreen::FinderDataBrowser;
                self.toasts.borrow_mut().info("Opened Finder Browser");
            }
            "file.save" => self.toasts.borrow_mut().success("Document saved"),
            "file.export" => {
                self.active_screen = GalleryScreen::FinderDataBrowser;
                (self.center.is_publishing, self.center.publish_progress) = (true, 0.0);
                self.toasts.borrow_mut().info("Publishing started...");
            }
            "edit.cut" | "edit.copy" => {
                let c = self.card.borrow();
                ctx.copy_text(format!("width: {}; height: {}; radius: {};", c.width, c.height, c.radius));
                self.toasts.borrow_mut().info("Copied card style");
            }
            "edit.paste" => self.toasts.borrow_mut().info("Pasted style"),
            "edit.select_all" => self.toasts.borrow_mut().info("Selected canvas layer"),
            "file.close" => ctx.send_viewport_cmd(ViewportCommand::Close),
            "window.minimize" => ctx.send_viewport_cmd(ViewportCommand::Minimized(true)),
            "help.shortcuts" => self.palette.open = true,
            "help.documentation" | "help.about" => {
                self.toasts.borrow_mut().info(format!("egui-macview v{}", egui_macview::VERSION));
            }
            _ => {}
        }
    }

    pub(crate) fn render_toolbar(&mut self, ui: &mut Ui, rect: Rect, theme: &MacTheme, on_resize_border: bool) {
        let (mut is_max, mut next_appearance, mut next_style) = (self.is_maximized, self.appearance, self.window_controls_style);
        let (mut next_mode, mut next_screen_idx, mut toggle_palette, palette_open) = (self.inspector_mode, self.active_screen as usize, false, self.palette.open);

        let tb_resp = MacToolbar::new("")
            .id("gallery_main_toolbar")
            .theme(theme)
            .window_controls_style(self.window_controls_style)
            .maximized(&mut is_max)
            .show_in_rect(ui, rect, |tb_ui| {
                let right_area = tb_ui.max_rect();
                if right_area.width() > 0.0 {
                    let mut right_ui = tb_ui.new_child(
                        egui::UiBuilder::new()
                            .id_salt("gallery_toolbar_right_ui")
                            .max_rect(right_area)
                            .layout(Layout::right_to_left(Align::Center)),
                    );
                    right_ui.spacing_mut().item_spacing.x = 6.0;

                    let current_theme_idx = match next_appearance {
                        MacAppearance::Aqua => 0,
                        MacAppearance::Anthracite => 1,
                        MacAppearance::DarkAqua => 2,
                    };
                    MacSegmentedControl::new(&["Aqua", "Anthracite", "DarkAqua"], current_theme_idx)
                        .theme(theme)
                        .height(24.0)
                        .show(&mut right_ui, |idx| {
                            next_appearance = [MacAppearance::Aqua, MacAppearance::Anthracite, MacAppearance::DarkAqua][idx.min(2)];
                        });

                    let current_style_idx = match next_style {
                        WindowControlsStyle::AppleTrafficLights => 0,
                        WindowControlsStyle::WindowsCaptionButtons => 1,
                        WindowControlsStyle::NativeHost => 2,
                    };
                    MacSegmentedControl::new(&["Apple", "Windows", "Native"], current_style_idx)
                        .theme(theme)
                        .height(24.0)
                        .show(&mut right_ui, |idx| {
                            next_style = [WindowControlsStyle::AppleTrafficLights, WindowControlsStyle::WindowsCaptionButtons, WindowControlsStyle::NativeHost][idx.min(2)];
                        });

                    MacSegmentedControl::new(&["Default", "Compact", "Expanded"], next_mode as usize)
                        .theme(theme)
                        .height(24.0)
                        .show(&mut right_ui, |idx| {
                            next_mode = [InspectorMode::DefaultNib, InspectorMode::CompactHud, InspectorMode::ExpandedPro][idx.min(2)];
                        });

                    MacSegmentedControl::new(&["v0.2", "v0.3", "v0.4", "Finder", "AVKit"], next_screen_idx)
                        .theme(theme)
                        .height(24.0)
                        .show(&mut right_ui, |idx| next_screen_idx = idx);

                    let k_str = if cfg!(target_os = "macos") { "⌘K" } else { "Ctrl+K" };
                    let search_label = format!("{} {}", MacSymbol::Search.as_str(), k_str);
                    if right_ui.add(MacButtonCell::new(&search_label).theme(theme).height(24.0).selected(palette_open)).clicked() {
                        toggle_palette = true;
                    }
                }
            });

        (self.appearance, self.window_controls_style, self.inspector_mode) = (next_appearance, next_style, next_mode);
        self.active_screen = GalleryScreen::ALL[next_screen_idx.min(GalleryScreen::ALL.len() - 1)];
        (self.sidebar.active_screen, self.is_maximized) = (self.active_screen, is_max);

        if self.last_window_controls_style != Some(self.window_controls_style) {
            let prev_was_native = self.last_window_controls_style == Some(WindowControlsStyle::NativeHost);
            self.last_window_controls_style = Some(self.window_controls_style);
            let native_decorations = self.window_controls_style == WindowControlsStyle::NativeHost;
            if prev_was_native != native_decorations || self.pending_decoration_resync_frames > 0 {
                let target_size = if self.pending_decoration_resync_frames == 0 {
                    let cur = ui.input(|i| {
                        let vp = i.viewport();
                        if prev_was_native { vp.outer_rect.or(vp.inner_rect) } else { vp.inner_rect.or(vp.outer_rect) }.map(|r| r.size())
                    }).unwrap_or_else(|| ui.ctx().content_rect().size());
                    egui::vec2(cur.x.max(900.0), cur.y.max(540.0))
                } else {
                    self.pending_target_inner_size.unwrap_or_else(|| egui::vec2(1160.0, 740.0))
                };
                ui.ctx().send_viewport_cmd(ViewportCommand::Decorations(native_decorations));
                if self.is_maximized {
                    self.pending_decoration_resync_frames = 0;
                    self.pending_target_inner_size = None;
                    ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
                } else {
                    self.pending_target_inner_size = Some(target_size);
                    self.pending_decoration_resync_frames = 1;
                    ui.ctx().send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(target_size.x + 1.0, target_size.y + 1.0)));
                }
                ui.ctx().send_viewport_cmd(ViewportCommand::Focus);
                ui.ctx().request_repaint();
            } else {
                ui.ctx().send_viewport_cmd(ViewportCommand::Decorations(native_decorations));
                ui.ctx().send_viewport_cmd(ViewportCommand::Focus);
            }
        } else if self.pending_decoration_resync_frames > 0 {
            self.pending_decoration_resync_frames = 0;
            if let Some(target_size) = self.pending_target_inner_size.take() {
                if !self.is_maximized {
                    ui.ctx().send_viewport_cmd(ViewportCommand::InnerSize(target_size));
                }
            }
            ui.ctx().send_viewport_cmd(ViewportCommand::Focus);
            ui.ctx().request_repaint();
        }
        if toggle_palette {
            self.palette.toggle();
        }
        if tb_resp.close_clicked {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
        if tb_resp.minimize_clicked {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
        }
        if tb_resp.zoom_toggled {
            ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(self.is_maximized));
        }
        if tb_resp.drag_started && !on_resize_border {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }
    }
}
