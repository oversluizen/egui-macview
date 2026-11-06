//! Authentic macOS AppKit Color Well (NSColorWell equivalent).
//!
//! Grounded in macOS HIG and W3C DTCG tokens via `MacTheme`.
//! Fully data-driven: zero hardcoded colors or dimensions.

use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    color_picker::{color_picker_color32, Alpha},
    Area, Color32, CornerRadius, Key, Order, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    Vec2, Widget,
};

/// Authentic macOS AppKit Color Well (`NSColorWell` equivalent).
///
/// Features authentic inset color swatch, beveled well border,
/// popup color picker panel, and keyboard Space/Return trigger and Escape dismissal.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacColorWell, MacTheme};
/// use egui::Color32;
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut color = Color32::from_rgb(0, 122, 255);
///
/// MacColorWell::new(&mut color)
///     .theme(&theme)
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacColorWell<'a> {
    color: &'a mut Color32,
    theme: Option<&'a MacTheme>,
    width: Option<f32>,
    height: Option<f32>,
    corner_radius: Option<f32>,
    border_width: Option<f32>,
    show_alpha: bool,
    enabled: bool,
}

impl<'a> MacColorWell<'a> {
    /// Creates a new color well bound to a mutable `Color32` reference.
    pub fn new(color: &'a mut Color32) -> Self {
        Self {
            color,
            theme: None,
            width: None,
            height: None,
            corner_radius: None,
            border_width: None,
            show_alpha: true,
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

    /// Overrides border stroke width.
    pub fn border_width(mut self, w: f32) -> Self {
        self.border_width = Some(w);
        self
    }

    /// Toggles alpha channel transparency preview.
    pub fn show_alpha(mut self, show: bool) -> Self {
        self.show_alpha = show;
        self
    }

    /// Enables or disables the control.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Renders the color well inside the specified UI container.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacColorWell<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let metrics = theme.color_well();
        let width = self.width.unwrap_or_else(|| metrics.width.points());
        let height = self.height.unwrap_or_else(|| metrics.height.points());
        let corner_radius = self
            .corner_radius
            .unwrap_or_else(|| metrics.corner_radius.points());
        let border_width = self
            .border_width
            .unwrap_or_else(|| metrics.border_width.points());
        let swatch_inset = metrics.swatch_inset.points();
        let swatch_radius = metrics.swatch_radius.points();

        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (raw_rect, mut response) = ui.allocate_exact_size(Vec2::new(width, height), sense);
        let rect = subpixel_snap_rect(raw_rect, ui.ctx().pixels_per_point());

        let painter = ui.painter_at(rect);

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

        // Outer bezel
        let bg_color = if response.is_pointer_button_down_on() {
            theme.selected_control_inactive()
        } else if response.hovered() {
            theme.control()
        } else {
            theme.control_background()
        };

        let border_stroke = if response.has_focus() {
            Stroke::new(border_width, theme.control_accent())
        } else {
            Stroke::new(border_width, theme.separator())
        };

        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            bg_color,
            border_stroke,
            StrokeKind::Inside,
        );

        // Inner color swatch compartment
        let swatch_rect = rect.shrink(swatch_inset);
        if swatch_rect.width() > 0.0 && swatch_rect.height() > 0.0 {
            // Draw checkerboard underlay if color is translucent and alpha is enabled
            if self.show_alpha && self.color.a() < 255 {
                let cell_size = metrics.checker_grid_size.points();
                let cols = (swatch_rect.width() / cell_size).ceil() as usize;
                let rows = (swatch_rect.height() / cell_size).ceil() as usize;
                for r in 0..rows {
                    for c in 0..cols {
                        let is_dark = (r + c) % 2 == 1;
                        let tile_color = if is_dark {
                            theme.control()
                        } else {
                            theme.control_background()
                        };
                        let tile_min = Pos2::new(
                            (swatch_rect.left() + (c as f32) * cell_size).min(swatch_rect.right()),
                            (swatch_rect.top() + (r as f32) * cell_size).min(swatch_rect.bottom()),
                        );
                        let tile_max = Pos2::new(
                            (tile_min.x + cell_size).min(swatch_rect.right()),
                            (tile_min.y + cell_size).min(swatch_rect.bottom()),
                        );
                        painter.rect_filled(
                            Rect::from_min_max(tile_min, tile_max),
                            CornerRadius::ZERO,
                            tile_color,
                        );
                    }
                }
            }

            // Fill color swatch
            painter.rect(
                swatch_rect,
                CornerRadius::same(swatch_radius as u8),
                *self.color,
                Stroke::new(border_width, theme.separator()),
                StrokeKind::Inside,
            );
        }

        // Color picker popup handling
        let is_open_id = response.id.with("mac_color_picker_is_open");
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
            let picker_dim = metrics.picker_size.points();
            let picker_size = Vec2::new(picker_dim, picker_dim);
            let popup_gap = metrics.popup_gap.points();
            let screen_rect = ui
                .ctx()
                .input(|i| i.raw.screen_rect)
                .unwrap_or_else(|| ui.clip_rect());
            let popup_y = if rect.bottom() + popup_gap + picker_size.y > screen_rect.bottom()
                && rect.top() - popup_gap - picker_size.y >= screen_rect.top()
            {
                rect.top() - popup_gap - picker_size.y
            } else {
                rect.bottom() + popup_gap
            };
            let popup_x = if rect.left() + picker_size.x > screen_rect.right() {
                (screen_rect.right() - picker_size.x).max(screen_rect.left())
            } else {
                rect.left()
            };
            let popup_pos = Pos2::new(popup_x, popup_y);
            let area_id = response.id.with("mac_color_picker_area");
            Area::new(area_id)
                .order(Order::Foreground)
                .fixed_pos(popup_pos)
                .show(ui.ctx(), |picker_ui| {
                    let frame = egui::Frame::window(picker_ui.style())
                        .fill(theme.window_background())
                        .stroke(Stroke::new(
                            metrics.border_width.points(),
                            theme.separator(),
                        ))
                        .corner_radius(CornerRadius::same(
                            metrics.popup_corner_radius.points() as u8
                        ));

                    frame.show(picker_ui, |inner_ui| {
                        let alpha_mode = if self.show_alpha {
                            Alpha::OnlyBlend
                        } else {
                            Alpha::Opaque
                        };
                        if color_picker_color32(inner_ui, self.color, alpha_mode) {
                            response.mark_changed();
                        }
                    });
                });

            // Dismiss when pressing Escape or clicking outside
            if ui.input(|i| i.key_pressed(Key::Escape)) {
                ui.data_mut(|d| d.insert_temp(is_open_id, false));
            } else if ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary))
                && !response.clicked()
            {
                let popup_rect = Rect::from_min_size(popup_pos, picker_size);
                if !ui.rect_contains_pointer(popup_rect) {
                    ui.data_mut(|d| d.insert_temp(is_open_id, false));
                }
            }
        }

        response
    }
}
