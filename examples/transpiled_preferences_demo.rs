//! `transpiled_preferences_demo` — Standalone macOS Preferences Window.
//!
//! Grounded in Apple macOS AppKit Human Interface Guidelines (HIG), GL-1015, GL-1016, and GL-1017.
//! Directly executes the deterministically transpiled Rust code from macOS NIBs:
//! 1. `GSAppearancePreferences` (macOS Settings Dialog)
//! 2. `PanelViewStrokeSettings` (macOS Inspector Panel)

#![cfg_attr(not(test), windows_subsystem = "windows")]

use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, FontId, Id, Rect, Response, Sense, Stroke,
    StrokeKind, Ui, ViewportCommand,
};
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    MacInputCell, MacPopUpButton, MacSegmentedControl, MacStackOrientation, MacStackView,
    MacToggle,
};

// ----------------------------------------------------------------------------
// Transpiled NIB 1: GSAppearancePreferences
// ----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsNibGSAppearancePreferencesState {
    pub display_unicode_value: bool,
    pub always_use_light_appearance: bool,
    pub text_view_width: String,
    pub handle_size: usize,
    pub info_box_size: usize,
    pub display_mark_cloud: bool,
    pub always_use_light_appearance_1: bool,
}

impl Default for SettingsNibGSAppearancePreferencesState {
    fn default() -> Self {
        Self {
            display_unicode_value: false,
            always_use_light_appearance: true,
            text_view_width: "120".to_string(),
            handle_size: 1,
            info_box_size: 1,
            display_mark_cloud: false,
            always_use_light_appearance_1: true,
        }
    }
}

impl SettingsNibGSAppearancePreferencesState {
    pub fn render(&mut self, ui: &mut Ui, theme: &MacTheme) -> Response {
        MacStackView::new(MacStackOrientation::Vertical)
            .theme(theme)
            .spacing(theme.stack_view().default_spacing.points())
            .padding(theme.stack_view().edge_padding.points())
            .show_separators(true)
            .show(ui, |stack| {
                stack.add_section(Some("Font View"), |sec_ui| {
                    sec_ui.horizontal(|h| {
                        h.add(MacToggle::new(&mut self.display_unicode_value).theme(theme));
                        h.label("Display Unicode value");
                    });
                    sec_ui.horizontal(|h| {
                        h.add(MacToggle::new(&mut self.always_use_light_appearance).theme(theme));
                        h.label("Always use light appearance");
                    });
                });
                stack.add_section(Some("Edit View"), |sec_ui| {
                    sec_ui.horizontal(|h| {
                        h.label("Text View width:");
                        MacInputCell::with_id(Id::NULL.with("_NS:67"), &mut self.text_view_width)
                            .theme(theme)
                            .width(90.0)
                            .show(h);
                    });
                    sec_ui.add_space(2.0);
                    sec_ui.horizontal(|h| {
                        h.label("Handle size:");
                        MacPopUpButton::new(&mut self.handle_size, &["Small", "Normal", "Large"])
                            .theme(theme)
                            .width(110.0)
                            .show(h);
                    });
                    sec_ui.add_space(2.0);
                    sec_ui.horizontal(|h| {
                        h.label("Info box size:");
                        MacPopUpButton::new(
                            &mut self.info_box_size,
                            &["Small", "Normal", "Large", "Extra Large"],
                        )
                        .theme(theme)
                        .width(110.0)
                        .show(h);
                    });
                    sec_ui.add_space(4.0);
                    sec_ui.horizontal(|h| {
                        h.add(MacToggle::new(&mut self.display_mark_cloud).theme(theme));
                        h.label("Display mark cloud");
                    });
                    sec_ui.horizontal(|h| {
                        h.add(MacToggle::new(&mut self.always_use_light_appearance_1).theme(theme));
                        h.label("Always use light appearance in preview");
                    });
                });
            });
        ui.allocate_response(ui.available_size(), Sense::hover())
    }
}

// ----------------------------------------------------------------------------
// Transpiled NIB 2: PanelViewStrokeSettings
// ----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsNibPanelViewStrokeSettingsState {
    pub w: String,
    pub h: String,
    pub selected_segment_2: usize,
    pub input_6: String,
    pub selected_segment_7: usize,
    pub selected_segment_13: usize,
}

impl Default for SettingsNibPanelViewStrokeSettingsState {
    fn default() -> Self {
        Self {
            w: "24".to_string(),
            h: "18".to_string(),
            selected_segment_2: 1,
            input_6: "50%".to_string(),
            selected_segment_7: 2,
            selected_segment_13: 1,
        }
    }
}

impl SettingsNibPanelViewStrokeSettingsState {
    pub fn render(&mut self, ui: &mut Ui, theme: &MacTheme) -> Response {
        MacStackView::new(MacStackOrientation::Vertical)
            .theme(theme)
            .spacing(theme.stack_view().default_spacing.points())
            .padding(theme.stack_view().edge_padding.points())
            .show_separators(true)
            .show(ui, |stack| {
                stack.add_section(Some("Dimensions & Alignment"), |sec_ui| {
                    sec_ui.horizontal(|h| {
                        h.label("Width:");
                        MacInputCell::with_id(Id::NULL.with("w"), &mut self.w)
                            .theme(theme)
                            .width(60.0)
                            .show(h);
                        h.add_space(8.0);
                        h.label("Height:");
                        MacInputCell::with_id(Id::NULL.with("h"), &mut self.h)
                            .theme(theme)
                            .width(60.0)
                            .show(h);
                    });
                    sec_ui.add_space(4.0);
                    sec_ui.horizontal(|h| {
                        h.label("Align:");
                        MacSegmentedControl::new(
                            &["Left", "Center", "Right"],
                            self.selected_segment_2,
                        )
                        .theme(theme)
                        .show(h, |idx| self.selected_segment_2 = idx);
                    });
                });

                stack.add_section(Some("Stroke Caps & Joins"), |sec_ui| {
                    sec_ui.horizontal(|h| {
                        h.label("Cap:");
                        MacSegmentedControl::new(
                            &["Butt", "Square", "Round", "Axis"],
                            self.selected_segment_7,
                        )
                        .theme(theme)
                        .show(h, |idx| self.selected_segment_7 = idx);
                    });
                    sec_ui.add_space(4.0);
                    sec_ui.horizontal(|h| {
                        h.label("Join:");
                        MacSegmentedControl::new(
                            &["Bevel", "Round", "Miter"],
                            self.selected_segment_13,
                        )
                        .theme(theme)
                        .show(h, |idx| self.selected_segment_13 = idx);
                    });
                    sec_ui.add_space(4.0);
                    sec_ui.horizontal(|h| {
                        h.label("Dash Ratio:");
                        MacInputCell::with_id(Id::NULL.with("_NS:38"), &mut self.input_6)
                            .theme(theme)
                            .width(60.0)
                            .show(h);
                    });
                });
            });
        ui.allocate_response(ui.available_size(), Sense::hover())
    }
}

// ----------------------------------------------------------------------------
// Standalone Preferences Window Application
// ----------------------------------------------------------------------------
struct PreferencesApp {
    theme: MacTheme,
    selected_tab: usize,
    appearance_state: SettingsNibGSAppearancePreferencesState,
    stroke_state: SettingsNibPanelViewStrokeSettingsState,
}

impl PreferencesApp {
    fn new() -> Self {
        Self {
            theme: MacTheme::new(MacAppearance::DarkAqua),
            selected_tab: 0,
            appearance_state: SettingsNibGSAppearancePreferencesState::default(),
            stroke_state: SettingsNibPanelViewStrokeSettingsState::default(),
        }
    }
}

impl eframe::App for PreferencesApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let is_dark = self.theme.appearance == MacAppearance::DarkAqua;
        self.theme.apply_to_egui(ui.ctx());

        let full_rect = ui.available_rect_before_wrap();
        let painter = ui.painter_at(full_rect);

        // Window background & border
        painter.rect(
            full_rect,
            CornerRadius::same(10),
            self.theme.window_background(),
            Stroke::new(1.0, self.theme.separator()),
            StrokeKind::Inside,
        );

        // Titlebar area
        let titlebar_rect = Rect::from_min_size(full_rect.min, vec2(full_rect.width(), 46.0));
        let titlebar_response = ui.interact(
            titlebar_rect,
            ui.scope_id().with("pref_window_drag"),
            Sense::click_and_drag(),
        );
        if titlebar_response.dragged() {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }

        // Apple Traffic Lights
        let traffic_y = titlebar_rect.center().y;
        let traffic_radius = 6.0;
        let close_pos = pos2(titlebar_rect.left() + 18.0, traffic_y);
        let min_pos = pos2(close_pos.x + 20.0, traffic_y);
        let zoom_pos = pos2(min_pos.x + 20.0, traffic_y);

        let (close_fill, close_border) = self.theme.traffic_light_close();
        let (min_fill, min_border) = self.theme.traffic_light_minimize();
        let (zoom_fill, zoom_border) = self.theme.traffic_light_zoom();

        let cluster_rect = Rect::from_min_max(
            pos2(close_pos.x - 8.0, traffic_y - 8.0),
            pos2(zoom_pos.x + 8.0, traffic_y + 8.0),
        );
        let cluster_hovered = ui.rect_contains_pointer(cluster_rect);

        // Close
        let close_resp = ui.interact(
            Rect::from_center_size(close_pos, vec2(14.0, 14.0)),
            ui.scope_id().with("tl_close"),
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

        // Minimize
        let min_resp = ui.interact(
            Rect::from_center_size(min_pos, vec2(14.0, 14.0)),
            ui.scope_id().with("tl_min"),
            Sense::click(),
        );
        if min_resp.clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
        }
        painter.circle(
            min_pos,
            traffic_radius,
            min_fill,
            Stroke::new(0.5, min_border),
        );

        // Zoom
        painter.circle(
            zoom_pos,
            traffic_radius,
            zoom_fill,
            Stroke::new(0.5, zoom_border),
        );

        if cluster_hovered {
            let sym = Color32::from_black_alpha(160);
            painter.line_segment(
                [
                    pos2(close_pos.x - 2.5, close_pos.y - 2.5),
                    pos2(close_pos.x + 2.5, close_pos.y + 2.5),
                ],
                Stroke::new(1.0, sym),
            );
            painter.line_segment(
                [
                    pos2(close_pos.x - 2.5, close_pos.y + 2.5),
                    pos2(close_pos.x + 2.5, close_pos.y - 2.5),
                ],
                Stroke::new(1.0, sym),
            );
            painter.line_segment(
                [
                    pos2(min_pos.x - 3.0, min_pos.y),
                    pos2(min_pos.x + 3.0, min_pos.y),
                ],
                Stroke::new(1.0, sym),
            );
        }

        // Window Title
        painter.text(
            titlebar_rect.center(),
            Align2::CENTER_CENTER,
            "Glyphs Preferences (Transpiled NIBs)",
            FontId::proportional(13.0),
            if is_dark {
                Color32::from_rgb(235, 235, 240)
            } else {
                Color32::from_rgb(30, 30, 32)
            },
        );

        // Dark/Light toggle on top right
        let toggle_rect = Rect::from_min_size(
            pos2(
                titlebar_rect.right() - 88.0,
                titlebar_rect.center().y - 10.0,
            ),
            vec2(76.0, 20.0),
        );
        let mut toggle_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("prefs_dark_toggle")
                .max_rect(toggle_rect),
        );
        toggle_ui.horizontal(|h| {
            let mut dark_mode = is_dark;
            if h.add(MacToggle::new(&mut dark_mode).theme(&self.theme))
                .changed()
            {
                self.theme = if dark_mode {
                    MacTheme::new(MacAppearance::DarkAqua)
                } else {
                    MacTheme::new(MacAppearance::Aqua)
                };
            }
            h.label(if is_dark { "Dark" } else { "Light" });
        });

        // Titlebar separator
        painter.line_segment(
            [
                pos2(titlebar_rect.left(), titlebar_rect.bottom()),
                pos2(titlebar_rect.right(), titlebar_rect.bottom()),
            ],
            Stroke::new(1.0, self.theme.separator()),
        );

        // Toolbar / Tab Segmented Control
        let toolbar_rect = Rect::from_min_size(
            pos2(full_rect.left() + 24.0, titlebar_rect.bottom() + 10.0),
            vec2(full_rect.width() - 48.0, 28.0),
        );
        let mut tab_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("prefs_tab_bar")
                .max_rect(toolbar_rect),
        );
        tab_ui.centered_and_justified(|c| {
            MacSegmentedControl::new(
                &["Appearance (GSAppearance)", "Stroke Settings (PanelView)"],
                self.selected_tab,
            )
            .theme(&self.theme)
            .show(c, |idx| self.selected_tab = idx);
        });

        // Content Area
        let content_rect = Rect::from_min_max(
            pos2(full_rect.left() + 20.0, toolbar_rect.bottom() + 12.0),
            pos2(full_rect.right() - 20.0, full_rect.bottom() - 16.0),
        );
        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("prefs_content")
                .max_rect(content_rect),
        );

        if self.selected_tab == 0 {
            self.appearance_state.render(&mut content_ui, &self.theme);
        } else {
            self.stroke_state.render(&mut content_ui, &self.theme);
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([540.0, 480.0])
            .with_min_inner_size([460.0, 380.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_title("Glyphs Preferences — egui-macview"),
        ..Default::default()
    };
    eframe::run_native(
        "transpiled_preferences_demo",
        options,
        Box::new(|_cc| Ok(Box::new(PreferencesApp::new()))),
    )
}
