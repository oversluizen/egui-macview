//! Authentic macOS AppKit Pop-Up Button (NSPopUpButton equivalent).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors or dimensions.

use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    Align2, Area, Color32, CornerRadius, FontId, Key, Order, Pos2, Rect, Response, Sense, Stroke,
    StrokeKind, Ui, Vec2, Widget,
};

/// Authentic macOS AppKit Pop-Up Button (`NSPopUpButton` equivalent).
///
/// Features authentic dual stacked chevrons, floating menu overlay,
/// keyboard navigation (Up/Down arrows, Space, Return, Escape), and theme-driven styling.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacPopUpButton, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut selected = 0;
/// let options = ["Option 1", "Option 2", "Option 3"];
///
/// MacPopUpButton::new(&mut selected, &options)
///     .theme(&theme)
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacPopUpButton<'a> {
    selected_index: &'a mut usize,
    items: &'a [&'a str],
    theme: Option<&'a MacTheme>,
    width: Option<f32>,
    height: Option<f32>,
    corner_radius: Option<f32>,
    enabled: bool,
}

impl<'a> MacPopUpButton<'a> {
    /// Creates a new popup dropdown button for an item slice and a mutable selected index.
    pub fn new(selected_index: &'a mut usize, items: &'a [&'a str]) -> Self {
        Self {
            selected_index,
            items,
            theme: None,
            width: None,
            height: None,
            corner_radius: None,
            enabled: true,
        }
    }

    /// Sets the dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Overrides default width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides default height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Overrides default corner radius.
    pub fn corner_radius(mut self, r: f32) -> Self {
        self.corner_radius = Some(r);
        self
    }

    /// Enables or disables the control.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Renders the popup button inside the specified UI container.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacPopUpButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let metrics = theme.popup_button();
        let height = self.height.unwrap_or_else(|| metrics.height.points());
        let corner_radius = self
            .corner_radius
            .unwrap_or_else(|| metrics.corner_radius.points());
        let padding_h = metrics.padding_horizontal.points();
        let chevron_size = metrics.chevron_size.points();
        let chevron_stroke = metrics.chevron_stroke_width.points();
        let chevron_spacing = metrics.chevron_spacing.points();

        let font_id = FontId::proportional(metrics.font_size.points());
        if !self.items.is_empty() && *self.selected_index >= self.items.len() {
            *self.selected_index = self.items.len() - 1;
        }
        let current_text = self.items.get(*self.selected_index).copied().unwrap_or("");

        let text_layout =
            ui.painter()
                .layout_no_wrap(current_text.to_owned(), font_id.clone(), Color32::WHITE);
        let desired_width = self.width.unwrap_or_else(|| {
            text_layout.size().x + padding_h * 2.0 + chevron_size + chevron_spacing
        });

        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (raw_rect, mut response) =
            ui.allocate_exact_size(Vec2::new(desired_width, height), sense);
        let rect = subpixel_snap_rect(raw_rect, ui.ctx().pixels_per_point());

        // Keyboard arrow navigation when focused
        if self.enabled && response.has_focus() {
            if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
                if *self.selected_index + 1 < self.items.len() {
                    *self.selected_index += 1;
                    response.mark_changed();
                }
            } else if ui.input(|i| i.key_pressed(Key::ArrowUp)) && *self.selected_index > 0 {
                *self.selected_index -= 1;
                response.mark_changed();
            }
        }

        let painter = ui.painter_at(rect);

        // Dynamic background
        let bg_color = if !self.enabled {
            theme.control_background()
        } else if response.is_pointer_button_down_on() {
            theme.selected_control_inactive()
        } else if response.hovered() {
            theme.control()
        } else {
            theme.control_background()
        };

        let border_stroke = if response.has_focus() {
            Stroke::new(metrics.border_width.points(), theme.control_accent())
        } else {
            Stroke::new(metrics.border_width.points(), theme.separator())
        };

        // Outer focus ring
        if self.enabled && response.has_focus() {
            let ring = theme.focus_ring();
            let focus_rect = rect.expand(ring.offset.points());
            let focus_radius = corner_radius + ring.corner_radius_delta.points();
            painter.rect_stroke(
                focus_rect,
                CornerRadius::same(focus_radius as u8),
                Stroke::new(ring.stroke_width.points(), theme.keyboard_focus_indicator()),
                StrokeKind::Outside,
            );
        }

        // Draw bezel
        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            bg_color,
            border_stroke,
            StrokeKind::Inside,
        );

        // Draw text label
        let text_color = if !self.enabled {
            theme.placeholder_text()
        } else {
            theme.label()
        };

        let label_pos = Pos2::new(rect.left() + padding_h, rect.center().y);
        painter.text(
            label_pos,
            Align2::LEFT_CENTER,
            current_text,
            font_id.clone(),
            text_color,
        );

        // Draw trailing double chevrons (up/down pair)
        let chevron_color = if !self.enabled {
            theme.placeholder_text()
        } else {
            theme.separator()
        };
        let stroke = Stroke::new(chevron_stroke, chevron_color);

        let cx = rect.right() - padding_h - chevron_size * 0.5;
        let cy = rect.center().y;
        let half_w = chevron_size * 0.5;
        let delta_y = metrics.chevron_offset_y.points();

        // Up chevron ^
        let up_tip = Pos2::new(cx, cy - delta_y);
        let up_left = Pos2::new(cx - half_w, cy);
        let up_right = Pos2::new(cx + half_w, cy);
        painter.line_segment([up_left, up_tip], stroke);
        painter.line_segment([up_tip, up_right], stroke);

        // Down chevron v
        let down_tip = Pos2::new(cx, cy + delta_y);
        let down_left = Pos2::new(cx - half_w, cy);
        let down_right = Pos2::new(cx + half_w, cy);
        painter.line_segment([down_left, down_tip], stroke);
        painter.line_segment([down_tip, down_right], stroke);

        // Popup overlay handling
        let is_open_id = response.id.with("mac_popup_is_open");
        let mut is_popup_open = ui.data(|d| d.get_temp::<bool>(is_open_id).unwrap_or(false));

        if self.enabled {
            if response.clicked() {
                response.request_focus();
                is_popup_open = !is_popup_open;
                ui.data_mut(|d| d.insert_temp(is_open_id, is_popup_open));
            } else if response.has_focus()
                && ui.input(|i| i.key_pressed(Key::Space) || i.key_pressed(Key::Enter))
            {
                is_popup_open = !is_popup_open;
                ui.data_mut(|d| d.insert_temp(is_open_id, is_popup_open));
            }
        }

        if self.enabled && is_popup_open {
            let menu_width = desired_width.max(metrics.menu_min_width.points());
            let item_height = height.max(metrics.menu_item_height.points());
            let menu_padding = metrics.menu_item_padding.points();
            let total_menu_height = item_height * (self.items.len() as f32) + menu_padding;

            let screen_rect = ui
                .ctx()
                .input(|i| i.raw.screen_rect)
                .unwrap_or_else(|| ui.clip_rect());
            let popup_gap = metrics.popup_gap.points();
            let popup_y = if rect.bottom() + popup_gap + total_menu_height > screen_rect.bottom()
                && rect.top() - popup_gap - total_menu_height >= screen_rect.top()
            {
                rect.top() - popup_gap - total_menu_height
            } else {
                rect.bottom() + popup_gap
            };
            let popup_x = if rect.left() + menu_width > screen_rect.right() {
                (screen_rect.right() - menu_width).max(screen_rect.left())
            } else {
                rect.left()
            };
            let popup_pos = Pos2::new(popup_x, popup_y);
            let area_id = response.id.with("mac_popup_area");
            Area::new(area_id)
                .order(Order::Foreground)
                .fixed_pos(popup_pos)
                .show(ui.ctx(), |menu_ui| {
                    let menu_rect =
                        Rect::from_min_size(popup_pos, Vec2::new(menu_width, total_menu_height));
                    let menu_painter = menu_ui.painter_at(menu_rect);

                    // Panel background & shadow/border
                    menu_painter.rect(
                        menu_rect,
                        CornerRadius::same(corner_radius as u8),
                        theme.window_background(),
                        Stroke::new(metrics.border_width.points(), theme.separator()),
                        StrokeKind::Inside,
                    );

                    let item_margin = metrics.menu_item_padding.points();
                    for (idx, item) in self.items.iter().enumerate() {
                        let item_rect = Rect::from_min_size(
                            Pos2::new(
                                menu_rect.left() + item_margin,
                                menu_rect.top() + item_margin + (idx as f32) * item_height,
                            ),
                            Vec2::new(menu_width - menu_padding, item_height),
                        );
                        let is_hovered = menu_ui.rect_contains_pointer(item_rect);
                        let is_active = idx == *self.selected_index;

                        if is_hovered || is_active {
                            let item_bg = if is_hovered {
                                theme.control_accent()
                            } else {
                                theme.control()
                            };
                            menu_painter.rect_filled(
                                item_rect,
                                CornerRadius::same(corner_radius as u8),
                                item_bg,
                            );
                        }

                        let item_text_color = if is_hovered {
                            theme.control_background()
                        } else {
                            theme.label()
                        };

                        menu_painter.text(
                            Pos2::new(item_rect.left() + menu_padding, item_rect.center().y),
                            Align2::LEFT_CENTER,
                            item,
                            font_id.clone(),
                            item_text_color,
                        );

                        if menu_ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary))
                            && is_hovered
                        {
                            *self.selected_index = idx;
                            menu_ui.data_mut(|d| d.insert_temp(is_open_id, false));
                            response.mark_changed();
                        }
                    }
                });

            // Dismiss popup when pressing Escape or clicking outside
            if ui.input(|i| i.key_pressed(Key::Escape)) {
                ui.data_mut(|d| d.insert_temp(is_open_id, false));
            } else if ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary))
                && !response.clicked()
            {
                let popup_bounds =
                    Rect::from_min_size(popup_pos, Vec2::new(menu_width, total_menu_height));
                if !ui.rect_contains_pointer(popup_bounds) {
                    ui.data_mut(|d| d.insert_temp(is_open_id, false));
                }
            }
        }

        response
    }
}
