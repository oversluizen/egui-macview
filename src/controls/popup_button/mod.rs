//! Authentic macOS AppKit Pop-Up Button (`NSPopUpButton` equivalent).

mod dropdown;

use crate::controls::MacControlSize;
use crate::layout::{subpixel_snap, subpixel_snap_rect};
use crate::theme::MacTheme;
use egui::{
    Color32, CornerRadius, FontId, Key, Modifiers, Pos2, Rect, Response, Sense, Stroke, StrokeKind,
    Ui, Vec2, Widget,
};

/// Authentic macOS AppKit Pop-Up Button (`NSPopUpButton` equivalent).
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacPopUpButton, MacTheme};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut selected = 0;
/// let options = ["Option 1", "Option 2", "Option 3"];
/// MacPopUpButton::new(&mut selected, &options).theme(&theme).show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacPopUpButton<'a> {
    id: Option<egui::Id>,
    selected_index: &'a mut usize,
    items: &'a [&'a str],
    theme: Option<&'a MacTheme>,
    width: Option<f32>,
    height: Option<f32>,
    corner_radius: Option<f32>,
    control_size: MacControlSize,
    fixed_rect: Option<Rect>,
    accent_badge: bool,
    enabled: bool,
}

impl<'a> MacPopUpButton<'a> {
    /// Creates a new popup dropdown button for an item slice and a mutable selected index.
    pub fn new(selected_index: &'a mut usize, items: &'a [&'a str]) -> Self {
        Self {
            id: None,
            selected_index,
            items,
            theme: None,
            width: None,
            height: None,
            corner_radius: None,
            control_size: MacControlSize::Regular,
            fixed_rect: None,
            accent_badge: false,
            enabled: true,
        }
    }

    /// Sets an explicit widget `Id`.
    pub fn id(mut self, id_source: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id = Some(egui::Id::NULL.with(id_source));
        self
    }

    /// Sets the dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets the AppKit control size (`Regular`, `Small`, or `Mini`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Pins the pop-up button to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
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

    /// Enables or disables the right-hand accent button badge (`NSPopUpButtonCell` classic/accent style).
    pub fn accent_badge(mut self, enabled: bool) -> Self {
        self.accent_badge = enabled;
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
        let theme = MacTheme::resolve(self.theme, ui.ctx());
        let metrics = theme.popup_button();
        let lm = theme.label_metrics();

        let (default_h, visual_h, font_sz, baseline_from_top, ascender) = match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => (
                metrics.height.points(),
                metrics.regular_visual_height.points(),
                metrics.font_size.points(),
                metrics.regular_first_baseline_offset.points(),
                lm.regular_ascender.points(),
            ),
            MacControlSize::Small | MacControlSize::CompactInfobox => (
                theme.text_field().small.height.points(),
                metrics.small_visual_height.points(),
                theme.text_field().small.font_size.points(),
                metrics.small_first_baseline_offset.points(),
                lm.small_ascender.points(),
            ),
            MacControlSize::Mini => (
                theme.text_field().mini.height.points(),
                metrics.mini_visual_height.points(),
                theme.text_field().mini.font_size.points(),
                metrics.mini_first_baseline_offset.points(),
                lm.mini_ascender.points(),
            ),
        };
        let height = self.height.unwrap_or(default_h);
        let corner_radius = self.corner_radius.unwrap_or_else(|| metrics.corner_radius.points());
        let padding_h = metrics.padding_horizontal.points();
        let chevron_size = metrics.chevron_size.points();
        let chevron_stroke = metrics.chevron_stroke_width.points();
        let chevron_spacing = metrics.chevron_spacing.points();

        let font_id = FontId::proportional(font_sz);
        if !self.items.is_empty() && *self.selected_index >= self.items.len() {
            *self.selected_index = self.items.len() - 1;
        }
        let current_text = self.items.get(*self.selected_index).copied().unwrap_or("");
        let current_galley = theme.layout_tracked_galley(ui, current_text, font_id.clone(), Color32::PLACEHOLDER, f32::INFINITY);
        let mut cached_items: Option<(f32, Vec<std::sync::Arc<egui::Galley>>)> = None;
        let layout_items = |ui: &Ui, cur: &std::sync::Arc<egui::Galley>, cache: &mut Option<(f32, Vec<std::sync::Arc<egui::Galley>>)>| {
            if let Some(existing) = cache.clone() {
                return existing;
            }
            let mut max_w = cur.size().x;
            let mut galleys = Vec::with_capacity(self.items.len());
            for &item in self.items {
                let g = if std::ptr::eq(item, current_text) {
                    cur.clone()
                } else {
                    theme.layout_tracked_galley(ui, item, font_id.clone(), Color32::PLACEHOLDER, f32::INFINITY)
                };
                max_w = max_w.max(g.size().x);
                galleys.push(g);
            }
            let w = max_w + padding_h * 2.0 + chevron_size + chevron_spacing;
            *cache = Some((w, galleys.clone()));
            (w, galleys)
        };
        let desired_width = self.fixed_rect.map(|r| r.width()).or(self.width)
            .unwrap_or_else(|| layout_items(ui, &current_galley, &mut cached_items).0);

        let sense = if self.enabled { Sense::click() } else { Sense::hover() };
        let ppp = ui.ctx().pixels_per_point();
        let (frame_rect, visual_rect, mut response) = if let Some(fixed) = self.fixed_rect {
            let id = self.id.unwrap_or_else(|| ui.next_auto_id().with("mac_popup_fixed"));
            let resp = ui.interact(fixed, id, sense);
            let sf = subpixel_snap_rect(fixed, ppp);
            let bh = fixed.height().min(visual_h);
            (sf, subpixel_snap_rect(Rect::from_center_size(sf.center(), Vec2::new(sf.width(), bh)), ppp), resp)
        } else if let Some(explicit_id) = self.id {
            let (r, _) = ui.allocate_exact_size(Vec2::new(desired_width, height), Sense::hover());
            let resp = ui.interact(r, explicit_id, sense);
            let snapped = subpixel_snap_rect(r, ppp);
            (snapped, snapped, resp)
        } else {
            let (r, resp) = ui.allocate_exact_size(Vec2::new(desired_width, height), sense);
            let snapped = subpixel_snap_rect(r, ppp);
            (snapped, snapped, resp)
        };

        if self.enabled && response.has_focus() {
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                if *self.selected_index + 1 < self.items.len() {
                    *self.selected_index += 1;
                    response.mark_changed();
                }
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) && *self.selected_index > 0 {
                *self.selected_index -= 1;
                response.mark_changed();
            }
        }

        let painter = ui.painter_at(frame_rect);
        let bg_color = if !self.enabled {
            theme.control_background()
        } else if response.is_pointer_button_down_on() {
            theme.selected_control_inactive()
        } else if response.hovered() {
            theme.control()
        } else {
            theme.control_background()
        };
        let border_w = metrics.border_width.points();
        let border_col = if response.has_focus() { theme.control_accent() } else { theme.separator() };
        if self.enabled && response.has_focus() {
            theme.paint_focus_ring(&painter, visual_rect, corner_radius);
        }
        painter.rect(visual_rect, CornerRadius::same(corner_radius as u8), bg_color, Stroke::new(border_w, border_col), StrokeKind::Inside);

        let text_color = if !self.enabled { theme.secondary_label() } else { theme.label() };
        let text_top_y = subpixel_snap(visual_rect.top() + (baseline_from_top - ascender), ppp);
        let badge_inset = metrics.popup_gap.points();
        let badge_w = chevron_size + chevron_spacing + badge_inset;
        let text_right = if self.accent_badge {
            (visual_rect.right() - badge_inset - badge_w - badge_inset).max(visual_rect.left())
        } else {
            (visual_rect.right() - padding_h - chevron_size - chevron_spacing).max(visual_rect.left())
        };
        let text_clip = Rect::from_min_max(visual_rect.min, Pos2::new(text_right, visual_rect.max.y));
        painter.with_clip_rect(text_clip).galley(Pos2::new(visual_rect.left() + padding_h, text_top_y), current_galley.clone(), text_color);

        let cx = if self.accent_badge {
            let badge_rect = Rect::from_min_max(
                Pos2::new((visual_rect.right() - badge_inset - badge_w).max(visual_rect.left()), visual_rect.top() + badge_inset),
                Pos2::new(visual_rect.right() - badge_inset, visual_rect.bottom() - badge_inset),
            );
            let badge_fill = if !self.enabled {
                theme.control().gamma_multiply(theme.disabled_opacity())
            } else if response.is_pointer_button_down_on() {
                theme.selected_control()
            } else {
                theme.control_accent()
            };
            painter.rect_filled(badge_rect, CornerRadius::same((corner_radius - border_w).max(0.0) as u8), badge_fill);
            badge_rect.center().x
        } else {
            visual_rect.right() - padding_h - chevron_size * 0.5
        };

        // Draw trailing double separated chevrons (up `⌃` and down `⌄`)
        let chevron_color = if !self.enabled {
            theme.secondary_label()
        } else if self.accent_badge {
            theme.on_accent_text()
        } else {
            theme.label()
        };
        let stroke = Stroke::new(chevron_stroke, chevron_color);
        let cy = visual_rect.center().y;
        let half_w = metrics.chevron_half_width.points();
        let gap = metrics.chevron_gap_vertical.points();
        let chev_h = metrics.chevron_height.points();

        painter.line_segment([Pos2::new(cx - half_w, cy - gap), Pos2::new(cx, cy - gap - chev_h)], stroke);
        painter.line_segment([Pos2::new(cx, cy - gap - chev_h), Pos2::new(cx + half_w, cy - gap)], stroke);
        painter.line_segment([Pos2::new(cx - half_w, cy + gap), Pos2::new(cx, cy + gap + chev_h)], stroke);
        painter.line_segment([Pos2::new(cx, cy + gap + chev_h), Pos2::new(cx + half_w, cy + gap)], stroke);

        dropdown::handle_popup_dropdown(
            ui, &theme, visual_rect, &mut response, self.selected_index, self.items,
            self.enabled, desired_width, visual_rect.height(), corner_radius,
            &current_galley, &mut cached_items, layout_items,
        );
        response
    }
}
