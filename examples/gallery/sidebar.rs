//! Left Sidebar (`MacOutlineView` 5-State Studio Navigator + 3 Inspector Presets).

use crate::{format_tracked_small_caps, GalleryScreen, InspectorMode};
use egui::{CornerRadius, Stroke, StrokeKind, Ui};
use egui_macview::{theme::MacTheme, MacOutlineView, MacToastManager};
use std::cell::RefCell;

/// State for the Left Sidebar (`MacOutlineView` 5-State Studio Navigator + Inspector Presets).
pub struct SidebarState {
    pub expanded_components: bool,
    pub expanded_inspector: bool,
    pub selected_node: String,
    pub active_screen: GalleryScreen,
    pub vis_canvas: bool,
    pub lock_canvas: bool,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            expanded_components: true,
            expanded_inspector: true,
            selected_node: GalleryScreen::CanvasPhysics.title().to_string(),
            active_screen: GalleryScreen::CanvasPhysics,
            vis_canvas: true,
            lock_canvas: false,
        }
    }
}

impl SidebarState {
    /// Renders the Left Sidebar with 5 Studio States (`v0.1` – `v0.5`) and 3 Inspector Presets.
    pub fn render(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        inspector_mode_out: &mut InspectorMode,
        active_screen_out: &mut GalleryScreen,
        toasts: &RefCell<MacToastManager>,
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
                let resp_comp = tree.row(
                    "node_screens_root",
                    0,
                    true,
                    &mut self.expanded_components,
                    false,
                    |row| {
                        row.header(&format_tracked_small_caps("Studio States (v0.2-v0.6)"));
                    },
                );
                if resp_comp.clicked {
                    self.expanded_components = !self.expanded_components;
                }

                if self.expanded_components {
                    for screen in GalleryScreen::ALL {
                        let is_sel = self.active_screen == screen;
                        let node_id = match screen {
                            GalleryScreen::CanvasPhysics => "node_screen_canvas",
                            GalleryScreen::VectorAssets => "node_screen_vector",
                            GalleryScreen::DesignTokensWcag => "node_screen_tokens",
                            GalleryScreen::FinderDataBrowser => "node_screen_finder",
                            GalleryScreen::AvKitMediaStudio => "node_screen_avkit",
                            GalleryScreen::AdvancedControls => "node_screen_advanced",
                        };
                        let resp = tree.row(node_id, 1, false, &mut false, is_sel, |row| {
                            let bg_style = theme.selected_control_background_style();
                            row.thumbnail(|painter, rect| {
                                let icon_color = if is_sel {
                                    bg_style.primary_text_color(theme)
                                } else {
                                    match screen {
                                        GalleryScreen::CanvasPhysics => theme.control_accent(),
                                        GalleryScreen::VectorAssets => theme.system_green(),
                                        GalleryScreen::DesignTokensWcag => {
                                            theme.traffic_light_minimize().0
                                        }
                                        GalleryScreen::FinderDataBrowser => theme.control_accent(),
                                        GalleryScreen::AvKitMediaStudio => {
                                            theme.traffic_light_close().0
                                        }
                                        GalleryScreen::AdvancedControls => theme.system_orange(),
                                    }
                                };
                                painter.rect_stroke(
                                    rect.shrink(2.5),
                                    CornerRadius::same(2),
                                    Stroke::new(1.2, icon_color),
                                    StrokeKind::Inside,
                                );
                                painter.circle_filled(rect.center(), 2.5, icon_color);
                            });
                            row.label(screen.sidebar_label());

                            if screen == GalleryScreen::CanvasPhysics {
                                let prev_vis = self.vis_canvas;
                                row.accessory_custom(
                                    "vis_canvas",
                                    &mut self.vis_canvas,
                                    |painter, rect, active| {
                                        let col = if active {
                                            if is_sel {
                                                bg_style.primary_text_color(theme)
                                            } else {
                                                theme.control_accent()
                                            }
                                        } else {
                                            theme.placeholder_text()
                                        };
                                        painter.circle_filled(rect.center(), 3.5, col);
                                    },
                                );
                                if self.vis_canvas != prev_vis {
                                    let msg = if self.vis_canvas {
                                        "Canvas layer visible"
                                    } else {
                                        "Canvas layer hidden"
                                    };
                                    toasts.borrow_mut().info(msg);
                                }
                                let prev_lock = self.lock_canvas;
                                row.accessory_custom(
                                    "lock_canvas",
                                    &mut self.lock_canvas,
                                    |painter, rect, active| {
                                        let col = if active {
                                            theme.traffic_light_close().0
                                        } else {
                                            theme.placeholder_text()
                                        };
                                        painter.rect_filled(
                                            rect.shrink(4.0),
                                            CornerRadius::same(1),
                                            col,
                                        );
                                    },
                                );
                                if self.lock_canvas != prev_lock {
                                    let msg = if self.lock_canvas {
                                        "Canvas layer locked"
                                    } else {
                                        "Canvas layer unlocked"
                                    };
                                    toasts.borrow_mut().info(msg);
                                }
                            }
                        });
                        if resp.clicked {
                            self.active_screen = screen;
                            self.selected_node = screen.title().to_string();
                            *active_screen_out = screen;
                        }
                    }
                }

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
                    for (node_id, mode, label) in [
                        ("node_nib", InspectorMode::DefaultNib, "Default NIB"),
                        ("node_hud", InspectorMode::CompactHud, "Compact HUD"),
                        ("node_exp", InspectorMode::ExpandedPro, "Expanded Pro"),
                    ] {
                        let is_sel = *inspector_mode_out == mode;
                        let resp = tree.row(node_id, 1, false, &mut false, is_sel, |row| {
                            let bg_style = theme.selected_control_background_style();
                            row.thumbnail(|painter, rect| {
                                let col = if is_sel {
                                    bg_style.primary_text_color(theme)
                                } else {
                                    theme.control_accent()
                                };
                                painter.rect_stroke(
                                    rect.shrink(3.0),
                                    CornerRadius::same(2),
                                    Stroke::new(1.0, col),
                                    StrokeKind::Inside,
                                );
                            });
                            row.label(label);
                        });
                        if resp.clicked {
                            self.selected_node = label.to_string();
                            *inspector_mode_out = mode;
                            toasts.borrow_mut().info("Preset applied");
                        }
                    }
                }
            });
    }
}
